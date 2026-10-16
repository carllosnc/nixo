use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_double, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use libloading::Library;
use nixo_core::{
    MediaBackend, MediaMetadata, NixoError, PlaybackState, PlayerCommand, PlayerStatus, VideoFrame,
};
use tracing::{debug, info};

#[repr(C)]
struct MpvHandle {
    _private: [u8; 0],
}

#[repr(C)]
struct MpvRenderContext {
    _private: [u8; 0],
}

#[repr(C)]
struct MpvRenderParam {
    type_: c_int,
    data: *mut c_void,
}

const MPV_FORMAT_FLAG: c_int = 3;
const MPV_FORMAT_INT64: c_int = 4;
const MPV_FORMAT_DOUBLE: c_int = 5;

const MPV_RENDER_PARAM_INVALID: c_int = 0;
const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
const MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME: c_int = 12;
const MPV_RENDER_PARAM_SW_SIZE: c_int = 17;
const MPV_RENDER_PARAM_SW_FORMAT: c_int = 18;
const MPV_RENDER_PARAM_SW_STRIDE: c_int = 19;
const MPV_RENDER_PARAM_SW_POINTER: c_int = 20;

const MPV_RENDER_UPDATE_FRAME: u64 = 1;

type MpvRenderUpdateFn = unsafe extern "C" fn(*mut c_void);

type FnMpvCreate = unsafe extern "C" fn() -> *mut MpvHandle;
type FnMpvInitialize = unsafe extern "C" fn(*mut MpvHandle) -> c_int;
type FnMpvTerminateDestroy = unsafe extern "C" fn(*mut MpvHandle);
type FnMpvCommand = unsafe extern "C" fn(*mut MpvHandle, *const *const c_char) -> c_int;
type FnMpvSetOptionString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, *const c_char) -> c_int;
type FnMpvSetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *const c_void) -> c_int;
type FnMpvGetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *mut c_void) -> c_int;
type FnMpvRenderContextCreate =
    unsafe extern "C" fn(*mut *mut MpvRenderContext, *mut MpvHandle, *mut MpvRenderParam) -> c_int;
type FnMpvRenderContextSetUpdateCallback =
    unsafe extern "C" fn(*mut MpvRenderContext, Option<MpvRenderUpdateFn>, *mut c_void);
type FnMpvRenderContextUpdate = unsafe extern "C" fn(*mut MpvRenderContext) -> u64;
type FnMpvRenderContextRender =
    unsafe extern "C" fn(*mut MpvRenderContext, *mut MpvRenderParam) -> c_int;
type FnMpvRenderContextFree = unsafe extern "C" fn(*mut MpvRenderContext);

unsafe extern "C" fn on_mpv_render_update(ctx: *mut c_void) {
    if !ctx.is_null() {
        let flag = unsafe { &*(ctx as *const AtomicBool) };
        flag.store(true, Ordering::Release);
    }
}

struct MpvApi {
    _lib: Library,
    create: FnMpvCreate,
    initialize: FnMpvInitialize,
    terminate_destroy: FnMpvTerminateDestroy,
    command: FnMpvCommand,
    set_option_string: FnMpvSetOptionString,
    set_property: FnMpvSetProperty,
    get_property: FnMpvGetProperty,
    render_context_create: FnMpvRenderContextCreate,
    render_context_set_update_callback: FnMpvRenderContextSetUpdateCallback,
    render_context_update: FnMpvRenderContextUpdate,
    render_context_render: FnMpvRenderContextRender,
    render_context_free: FnMpvRenderContextFree,
}

impl MpvApi {
    fn candidate_paths() -> Vec<PathBuf> {
        let mut list = Vec::new();

        if cfg!(target_os = "windows") {
            if let Ok(exe) = std::env::current_exe()
                && let Some(dir) = exe.parent()
            {
                list.push(dir.join("libmpv-2.dll"));
            }
            if let Ok(cwd) = std::env::current_dir() {
                list.push(cwd.join("libmpv-2.dll"));
                list.push(cwd.join("vendor").join("mpv").join("libmpv-2.dll"));
            }
            let workspace_vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("vendor")
                .join("mpv")
                .join("libmpv-2.dll");
            list.push(workspace_vendor);
            list.push(PathBuf::from("libmpv-2.dll"));
            list.push(PathBuf::from("mpv-2.dll"));
            list.push(PathBuf::from("mpv-1.dll"));
        } else if cfg!(target_os = "macos") {
            list.push(PathBuf::from("libmpv.2.dylib"));
            list.push(PathBuf::from("libmpv.dylib"));
            list.push(PathBuf::from("/opt/homebrew/lib/libmpv.dylib"));
            list.push(PathBuf::from("/usr/local/lib/libmpv.dylib"));
        } else {
            list.push(PathBuf::from("libmpv.so.2"));
            list.push(PathBuf::from("libmpv.so"));
            list.push(PathBuf::from("libmpv.so.1"));
        }

        list
    }

    fn load() -> Result<Self, NixoError> {
        let candidates = Self::candidate_paths();

        let mut last_err = String::new();
        for candidate in candidates {
            // SAFETY: Carregamento dinâmico da biblioteca compartilhada oficial do libmpv.
            match unsafe { Library::new(&candidate) } {
                Ok(lib) => unsafe {
                    let create = *lib
                        .get::<FnMpvCreate>(b"mpv_create\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let initialize = *lib
                        .get::<FnMpvInitialize>(b"mpv_initialize\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let terminate_destroy = *lib
                        .get::<FnMpvTerminateDestroy>(b"mpv_terminate_destroy\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let command = *lib
                        .get::<FnMpvCommand>(b"mpv_command\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let set_option_string = *lib
                        .get::<FnMpvSetOptionString>(b"mpv_set_option_string\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let set_property = *lib
                        .get::<FnMpvSetProperty>(b"mpv_set_property\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let get_property = *lib
                        .get::<FnMpvGetProperty>(b"mpv_get_property\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let render_context_create = *lib
                        .get::<FnMpvRenderContextCreate>(b"mpv_render_context_create\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let render_context_set_update_callback = *lib
                        .get::<FnMpvRenderContextSetUpdateCallback>(
                            b"mpv_render_context_set_update_callback\0",
                        )
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let render_context_update = *lib
                        .get::<FnMpvRenderContextUpdate>(b"mpv_render_context_update\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let render_context_render = *lib
                        .get::<FnMpvRenderContextRender>(b"mpv_render_context_render\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;
                    let render_context_free = *lib
                        .get::<FnMpvRenderContextFree>(b"mpv_render_context_free\0")
                        .map_err(|e| NixoError::InitError(e.to_string()))?;

                    info!(
                        "Biblioteca dinâmica libmpv carregada com sucesso: {}",
                        candidate.display()
                    );
                    return Ok(Self {
                        _lib: lib,
                        create,
                        initialize,
                        terminate_destroy,
                        command,
                        set_option_string,
                        set_property,
                        get_property,
                        render_context_create,
                        render_context_set_update_callback,
                        render_context_update,
                        render_context_render,
                        render_context_free,
                    });
                },
                Err(e) => {
                    last_err = e.to_string();
                }
            }
        }

        Err(NixoError::InitError(format!(
            "Não foi possível encontrar libmpv no sistema ({last_err})"
        )))
    }
}

// SAFETY: MpvApi armazena apenas o handle da DLL e ponteiros de funções C thread-safe do libmpv.
unsafe impl Send for MpvApi {}
// SAFETY: Ponteiros de funções C do libmpv podem ser chamados concorrentemente em handles distintos.
unsafe impl Sync for MpvApi {}

struct PreviewShared {
    target_uri: Mutex<Option<String>>,
    target_sec: AtomicU32,
    cache: Mutex<BTreeMap<u32, VideoFrame>>,
    shutdown: AtomicBool,
}

struct PreviewWorker {
    shared: Arc<PreviewShared>,
    thread: Option<std::thread::JoinHandle<()>>,
    last_requested_sec: u32,
    last_returned_frame_id: u64,
}

impl PreviewWorker {
    fn spawn(api: Arc<MpvApi>) -> Self {
        let shared = Arc::new(PreviewShared {
            target_uri: Mutex::new(None),
            target_sec: AtomicU32::new(u32::MAX),
            cache: Mutex::new(BTreeMap::new()),
            shutdown: AtomicBool::new(false),
        });

        let worker_shared = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("nixo-preview-worker".into())
            .spawn(move || Self::run_loop(api, worker_shared))
            .ok();

        Self {
            shared,
            thread,
            last_requested_sec: u32::MAX,
            last_returned_frame_id: 0,
        }
    }

    fn set_uri(&mut self, uri: Option<String>) {
        if let Ok(mut guard) = self.shared.target_uri.lock() {
            *guard = uri;
        }
        if let Ok(mut guard) = self.shared.cache.lock() {
            guard.clear();
        }
        self.last_requested_sec = u32::MAX;
        self.last_returned_frame_id = 0;
        self.shared.target_sec.store(u32::MAX, Ordering::Release);
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
    }

    fn request_and_poll(&mut self, position: Duration) -> Option<VideoFrame> {
        let sec = position.as_secs_f64().round().max(0.0) as u32;

        if self.last_requested_sec != sec {
            self.last_requested_sec = sec;
            self.shared.target_sec.store(sec, Ordering::Release);
            if let Some(t) = &self.thread {
                t.thread().unpark();
            }
        }

        // Busca instantânea (0ms) no cache de storyboard em RAM:
        // Retorna o frame exato do segundo solicitado ou o keyframe pré-aquecido mais próximo.
        if let Ok(cache) = self.shared.cache.try_lock() {
            let before = cache.range(..=sec).next_back();
            let after = cache.range(sec..).next();

            let best = match (before, after) {
                (Some((&k1, f1)), Some((&k2, f2))) => {
                    if sec.abs_diff(k1) <= sec.abs_diff(k2) {
                        Some(f1)
                    } else {
                        Some(f2)
                    }
                }
                (Some((_, f1)), None) => Some(f1),
                (None, Some((_, f2))) => Some(f2),
                (None, None) => None,
            };

            if let Some(frame) = best
                && frame.frame_id != self.last_returned_frame_id
            {
                self.last_returned_frame_id = frame.frame_id;
                return Some(frame.clone());
            }
        }

        None
    }

    #[allow(clippy::too_many_lines)]
    fn run_loop(api: Arc<MpvApi>, shared: Arc<PreviewShared>) {
        // SAFETY: Cria uma instância secundária isolada e silenciosa do libmpv exclusiva para miniaturas.
        unsafe {
            let handle = (api.create)();
            if handle.is_null() {
                return;
            }

            // Otimizações extremas para extração de miniaturas (seek de keyframe em ~5-15ms):
            // - demuxer-readahead-secs=0 e cache=no evitam leitura extra de disco após cada seek
            // - vd-lavc-skipidct=all + vd-lavc-skiploopfilter=all aceleram a decodificação H.264/HEVC
            // - vd-lavc-threads=4 paraleliza o decode do keyframe
            MpvBackend::set_opt(&api, handle, c"vo", c"libmpv");
            MpvBackend::set_opt(&api, handle, c"hwdec", c"no");
            MpvBackend::set_opt(&api, handle, c"aid", c"no");
            MpvBackend::set_opt(&api, handle, c"sid", c"no");
            MpvBackend::set_opt(&api, handle, c"sub", c"no");
            MpvBackend::set_opt(&api, handle, c"audio", c"no");
            MpvBackend::set_opt(&api, handle, c"pause", c"yes");
            MpvBackend::set_opt(&api, handle, c"keep-open", c"yes");
            MpvBackend::set_opt(&api, handle, c"hr-seek", c"no");
            MpvBackend::set_opt(&api, handle, c"demuxer-readahead-secs", c"0");
            MpvBackend::set_opt(&api, handle, c"cache", c"no");
            MpvBackend::set_opt(&api, handle, c"vd-lavc-fast", c"yes");
            MpvBackend::set_opt(&api, handle, c"vd-lavc-skiploopfilter", c"all");
            MpvBackend::set_opt(&api, handle, c"vd-lavc-skipidct", c"all");
            MpvBackend::set_opt(&api, handle, c"vd-lavc-threads", c"4");
            MpvBackend::set_opt(&api, handle, c"sws-scaler", c"fast-bilinear");
            MpvBackend::set_opt(&api, handle, c"sws-fast", c"yes");
            MpvBackend::set_opt(&api, handle, c"video-timing-offset", c"0");

            if (api.initialize)(handle) < 0 {
                (api.terminate_destroy)(handle);
                return;
            }

            let sw_api = c"sw";
            let mut init_params = [
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_API_TYPE,
                    data: sw_api.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut MpvRenderContext = ptr::null_mut();
            if (api.render_context_create)(&mut render_ctx, handle, init_params.as_mut_ptr()) < 0
                || render_ctx.is_null()
            {
                (api.terminate_destroy)(handle);
                return;
            }

            let has_update = Arc::new(AtomicBool::new(false));
            let cb_ctx = Arc::as_ptr(&has_update) as *mut c_void;
            (api.render_context_set_update_callback)(
                render_ctx,
                Some(on_mpv_render_update),
                cb_ctx,
            );

            let width: u32 = 160;
            let height: u32 = 90;
            let stride: usize = (width as usize) * 4;
            let mut pixel_buf = vec![255u8; stride * (height as usize)];
            let mut loaded_uri: Option<String> = None;
            let mut last_rendered_sec: u32 = u32::MAX;
            let mut frame_seq: u64 = 0;
            let mut storyboard_queue: Vec<u32> = Vec::new();
            let mut storyboard_built = false;

            while !shared.shutdown.load(Ordering::Acquire) {
                let desired_uri = shared.target_uri.lock().ok().and_then(|g| g.clone());
                if desired_uri != loaded_uri {
                    loaded_uri = desired_uri.clone();
                    last_rendered_sec = u32::MAX;
                    storyboard_queue.clear();
                    storyboard_built = false;

                    if let Some(uri) = desired_uri {
                        if let Ok(c_uri) = CString::new(uri) {
                            let c_cmd = c"loadfile";
                            let args = [c_cmd.as_ptr(), c_uri.as_ptr(), ptr::null()];
                            let _ = (api.command)(handle, args.as_ptr());
                            let pause_val: c_int = 1;
                            let _ = (api.set_property)(
                                handle,
                                c"pause".as_ptr(),
                                MPV_FORMAT_FLAG,
                                (&raw const pause_val).cast::<c_void>(),
                            );
                        }
                    } else {
                        let c_cmd = c"stop";
                        let args = [c_cmd.as_ptr(), ptr::null()];
                        let _ = (api.command)(handle, args.as_ptr());
                    }
                }

                // Constrói a fila de pré-aquecimento (Storyboard em 2 passes: 32 pontos grossos + 96 finos)
                if loaded_uri.is_some() && !storyboard_built {
                    let mut dur: c_double = 0.0;
                    let rc = (api.get_property)(
                        handle,
                        c"duration".as_ptr(),
                        MPV_FORMAT_DOUBLE,
                        (&raw mut dur).cast::<c_void>(),
                    );
                    if rc >= 0 && dur > 1.0 {
                        storyboard_built = true;
                        let total_s = dur as u32;
                        // Passada 1: 32 keyframes distribuídos pelo filme inteiro (pronto em ~0.3s)
                        for i in (0..32).rev() {
                            storyboard_queue.push((total_s * i) / 32);
                        }
                        // Passada 2: 96 keyframes intermediários
                        for i in (0..96).rev() {
                            storyboard_queue.push((total_s * i) / 96);
                        }
                    }
                }

                let req_sec = shared.target_sec.load(Ordering::Acquire);
                let already_cached = req_sec != u32::MAX
                    && shared.cache.lock().is_ok_and(|c| c.contains_key(&req_sec));

                let next_sec_to_decode = if loaded_uri.is_none() {
                    None
                } else if req_sec != u32::MAX && !already_cached && req_sec != last_rendered_sec {
                    // Prioridade máxima: posição exata onde o mouse do usuário está agora
                    Some(req_sec)
                } else {
                    // Em segundo plano: pré-aquece o storyboard do vídeo para resposta 0ms
                    let mut candidate = None;
                    while let Some(s) = storyboard_queue.pop() {
                        let cached = shared.cache.lock().is_ok_and(|c| c.contains_key(&s));
                        if !cached {
                            candidate = Some(s);
                            break;
                        }
                    }
                    candidate
                };

                if let Some(target_sec) = next_sec_to_decode {
                    // Usa o comando `seek <sec> absolute+keyframes` para salto imediato no índice do container
                    if let Ok(sec_cstr) = CString::new(target_sec.to_string()) {
                        let c_seek = c"seek";
                        let c_mode = c"absolute+keyframes";
                        let args = [
                            c_seek.as_ptr(),
                            sec_cstr.as_ptr(),
                            c_mode.as_ptr(),
                            ptr::null(),
                        ];
                        let _ = (api.command)(handle, args.as_ptr());
                    }

                    let start = Instant::now();
                    let mut rendered = false;
                    // Não interrompemos o decode no meio quando o mouse se move: deixamos o quadro atual
                    // terminar (~10-20ms) para que as miniaturas fluam continuamente a ~50 FPS durante o movimento!
                    while start.elapsed() < Duration::from_millis(90)
                        && !shared.shutdown.load(Ordering::Acquire)
                    {
                        let notified = has_update.swap(false, Ordering::AcqRel);
                        if notified {
                            let flags = (api.render_context_update)(render_ctx);
                            if (flags & MPV_RENDER_UPDATE_FRAME) != 0 {
                                let mut size_param: [c_int; 2] = [width as c_int, height as c_int];
                                let format_cstr = c"rgb0";
                                let mut stride_param: usize = stride;
                                let mut block_for_target_time: c_int = 0;

                                let mut r_params = [
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_SW_SIZE,
                                        data: size_param.as_mut_ptr().cast::<c_void>(),
                                    },
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_SW_FORMAT,
                                        data: format_cstr.as_ptr() as *mut c_void,
                                    },
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_SW_STRIDE,
                                        data: (&raw mut stride_param).cast::<c_void>(),
                                    },
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_SW_POINTER,
                                        data: pixel_buf.as_mut_ptr().cast::<c_void>(),
                                    },
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME,
                                        data: (&raw mut block_for_target_time).cast::<c_void>(),
                                    },
                                    MpvRenderParam {
                                        type_: MPV_RENDER_PARAM_INVALID,
                                        data: ptr::null_mut(),
                                    },
                                ];

                                if (api.render_context_render)(render_ctx, r_params.as_mut_ptr())
                                    >= 0
                                {
                                    for px in pixel_buf.as_chunks_mut::<4>().0 {
                                        px[3] = 255;
                                    }
                                    frame_seq += 1;
                                    let frame = VideoFrame {
                                        width,
                                        height,
                                        rgba: pixel_buf.clone(),
                                        frame_id: frame_seq,
                                    };
                                    if let Ok(mut cache) = shared.cache.lock() {
                                        // Limita o cache a ~360 miniaturas (~20 MB de RAM)
                                        if cache.len() < 360 || target_sec == req_sec {
                                            cache.insert(target_sec, frame);
                                        }
                                    }
                                    rendered = true;
                                    break;
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(1));
                    }

                    if rendered {
                        last_rendered_sec = target_sec;
                    }
                } else {
                    std::thread::park_timeout(Duration::from_millis(30));
                }
            }

            (api.render_context_set_update_callback)(render_ctx, None, ptr::null_mut());
            (api.render_context_free)(render_ctx);
            (api.terminate_destroy)(handle);
        }
    }
}

impl Drop for PreviewWorker {
    fn drop(&mut self) {
        self.shared.shutdown.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
            let _ = t.join();
        }
    }
}

pub struct MpvBackend {
    api: Arc<MpvApi>,
    handle: *mut MpvHandle,
    render_ctx: *mut MpvRenderContext,
    has_update: Arc<AtomicBool>,
    current_path: Option<PathBuf>,
    frame_counter: u64,
    first_frame_pending: bool,
    pixel_buffer: Vec<u8>,
    cached_status: PlayerStatus,
    last_status_poll: Instant,
    status_dirty: bool,
    cached_vid_size: (u32, u32),
    pending_seek: Option<(f64, Instant)>,
    preview_worker: PreviewWorker,
}

// SAFETY: O handle do mpv é acessado de forma exclusiva através de `&mut self` protegido pela trait.
unsafe impl Send for MpvBackend {}

impl MpvBackend {
    pub fn try_new() -> Result<Self, NixoError> {
        let api = Arc::new(MpvApi::load()?);

        unsafe {
            let handle = (api.create)();
            if handle.is_null() {
                return Err(NixoError::InitError("mpv_create retornou nulo".into()));
            }

            // Otimizado para baixa latência e alta taxa de quadros no pipeline de textura wgpu:
            // - vd-lavc-threads=0 usa todos os núcleos da CPU evitando stalls de readback VRAM->RAM no renderer SW
            // - sws-scaler=fast-bilinear + sws-fast=yes acelera a conversão YUV->RGB0 com SIMD AVX2
            // - video-timing-offset=0 impede que mpv_render_context_render bloqueie a thread de UI
            // - hr-seek=yes garante seek preciso no ponto clicado sem recuar para keyframes distantes
            Self::set_opt(&api, handle, c"vo", c"libmpv");
            Self::set_opt(&api, handle, c"hwdec", c"no");
            Self::set_opt(&api, handle, c"vd-lavc-threads", c"0");
            Self::set_opt(&api, handle, c"sws-scaler", c"fast-bilinear");
            Self::set_opt(&api, handle, c"sws-fast", c"yes");
            Self::set_opt(&api, handle, c"keep-open", c"yes");
            Self::set_opt(&api, handle, c"video-timing-offset", c"0");
            Self::set_opt(&api, handle, c"hr-seek", c"yes");

            let init_rc = (api.initialize)(handle);
            if init_rc < 0 {
                (api.terminate_destroy)(handle);
                return Err(NixoError::InitError(format!(
                    "mpv_initialize falhou com código {init_rc}"
                )));
            }

            let sw_api = c"sw";
            let mut params = [
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_API_TYPE,
                    data: sw_api.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut MpvRenderContext = ptr::null_mut();
            let rc = (api.render_context_create)(&mut render_ctx, handle, params.as_mut_ptr());
            if rc < 0 || render_ctx.is_null() {
                (api.terminate_destroy)(handle);
                return Err(NixoError::InitError(format!(
                    "mpv_render_context_create (sw) falhou com código {rc}"
                )));
            }

            let has_update = Arc::new(AtomicBool::new(false));
            let cb_ctx = Arc::as_ptr(&has_update) as *mut c_void;
            (api.render_context_set_update_callback)(
                render_ctx,
                Some(on_mpv_render_update),
                cb_ctx,
            );

            let preview_worker = PreviewWorker::spawn(Arc::clone(&api));

            Ok(Self {
                api,
                handle,
                render_ctx,
                has_update,
                current_path: None,
                frame_counter: 0,
                first_frame_pending: false,
                pixel_buffer: Vec::new(),
                cached_status: PlayerStatus {
                    backend_name: "libmpv",
                    ..Default::default()
                },
                last_status_poll: Instant::now(),
                status_dirty: true,
                cached_vid_size: (1280, 720),
                pending_seek: None,
                preview_worker,
            })
        }
    }

    unsafe fn set_opt(api: &MpvApi, handle: *mut MpvHandle, key: &CStr, val: &CStr) {
        unsafe {
            let _ = (api.set_option_string)(handle, key.as_ptr(), val.as_ptr());
        }
    }

    fn load_uri(&mut self, uri: &str, display_path: PathBuf) -> Result<(), NixoError> {
        let c_cmd = c"loadfile";
        let c_uri = CString::new(uri).map_err(|e| NixoError::InvalidMedia(e.to_string()))?;
        let args = [c_cmd.as_ptr(), c_uri.as_ptr(), ptr::null()];
        let rc = unsafe { (self.api.command)(self.handle, args.as_ptr()) };
        if rc < 0 {
            return Err(NixoError::PlaybackError(format!(
                "loadfile falhou (código {rc})"
            )));
        }
        self.set_flag(c"pause", false);
        self.current_path = Some(display_path);
        self.first_frame_pending = true;
        self.status_dirty = true;
        self.preview_worker.set_uri(Some(uri.to_string()));
        Ok(())
    }

    fn get_double(&self, prop: &CStr) -> Option<f64> {
        let mut val: c_double = 0.0;
        let rc = unsafe {
            (self.api.get_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_DOUBLE,
                (&raw mut val).cast::<c_void>(),
            )
        };
        if rc >= 0 { Some(val) } else { None }
    }

    fn get_i64(&self, prop: &CStr) -> Option<i64> {
        let mut val: i64 = 0;
        let rc = unsafe {
            (self.api.get_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_INT64,
                (&raw mut val).cast::<c_void>(),
            )
        };
        if rc >= 0 { Some(val) } else { None }
    }

    fn get_flag(&self, prop: &CStr) -> Option<bool> {
        let mut val: c_int = 0;
        let rc = unsafe {
            (self.api.get_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_FLAG,
                (&raw mut val).cast::<c_void>(),
            )
        };
        if rc >= 0 { Some(val != 0) } else { None }
    }

    fn set_double(&self, prop: &CStr, val: f64) {
        let c_val: c_double = val;
        unsafe {
            let _ = (self.api.set_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_DOUBLE,
                (&raw const c_val).cast::<c_void>(),
            );
        }
    }

    fn set_flag(&self, prop: &CStr, val: bool) {
        let c_val: c_int = i32::from(val);
        unsafe {
            let _ = (self.api.set_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_FLAG,
                (&raw const c_val).cast::<c_void>(),
            );
        }
    }

    fn refresh_status(&mut self) {
        let idle = self.get_flag(c"idle-active").unwrap_or(true);
        let eof = self.get_flag(c"eof-reached").unwrap_or(false);
        let paused = self.get_flag(c"pause").unwrap_or(false);

        let state = if idle && self.current_path.is_none() {
            PlaybackState::Stopped
        } else if eof {
            PlaybackState::Ended
        } else if paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        };

        let raw_pos_secs = self.get_double(c"time-pos").unwrap_or(0.0).max(0.0);
        let is_seeking = self.get_flag(c"seeking").unwrap_or(false);
        let pos_secs = if let Some((target_secs, seek_instant)) = self.pending_seek {
            let elapsed = seek_instant.elapsed();
            let close_enough = (raw_pos_secs - target_secs).abs() <= 1.0;
            // Enquanto o demuxer do libmpv processa o seek assíncrono no container (ex: MKV),
            // `time-pos` ainda retorna o timestamp antigo por alguns quadros. Mantemos a posição
            // alvo até o seek concluir ou aproximar-se do destino.
            if elapsed < Duration::from_millis(850) && (is_seeking || !close_enough) {
                target_secs
            } else {
                self.pending_seek = None;
                raw_pos_secs
            }
        } else {
            raw_pos_secs
        };

        let dur_secs = self.get_double(c"duration").unwrap_or(0.0).max(0.0);
        let volume = self.get_double(c"volume").unwrap_or(80.0) as f32;
        let muted = self.get_flag(c"mute").unwrap_or(false);
        let speed = self.get_double(c"speed").unwrap_or(1.0) as f32;

        let width = self
            .get_i64(c"dwidth")
            .or_else(|| self.get_i64(c"width"))
            .unwrap_or(0)
            .max(0) as u32;
        let height = self
            .get_i64(c"dheight")
            .or_else(|| self.get_i64(c"height"))
            .unwrap_or(0)
            .max(0) as u32;

        if width >= 64 && height >= 64 {
            self.cached_vid_size = (width, height);
        }

        let metadata = self.current_path.as_ref().map(|path| MediaMetadata {
            title: path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Vídeo".to_string()),
            path: path.clone(),
            duration: Duration::from_secs_f64(dur_secs),
            video_width: width,
            video_height: height,
        });

        self.cached_status = PlayerStatus {
            state,
            position: Duration::from_secs_f64(pos_secs),
            duration: Duration::from_secs_f64(dur_secs),
            volume,
            muted,
            speed,
            metadata,
            backend_name: self.name(),
        };
        self.last_status_poll = Instant::now();
        self.status_dirty = false;
    }
}

impl Drop for MpvBackend {
    fn drop(&mut self) {
        unsafe {
            if !self.render_ctx.is_null() {
                (self.api.render_context_set_update_callback)(
                    self.render_ctx,
                    None,
                    ptr::null_mut(),
                );
                (self.api.render_context_free)(self.render_ctx);
            }
            if !self.handle.is_null() {
                (self.api.terminate_destroy)(self.handle);
            }
        }
    }
}

impl MediaBackend for MpvBackend {
    fn name(&self) -> &'static str {
        "libmpv"
    }

    fn execute(&mut self, command: PlayerCommand) -> Result<(), NixoError> {
        match command {
            PlayerCommand::LoadFile(path) => {
                let path_str = path.to_string_lossy().into_owned();
                self.pending_seek = None;
                self.load_uri(&path_str, path)?;
                debug!("Arquivo carregado no libmpv: {path_str}");
            }
            PlayerCommand::Play => {
                if self.current_path.is_none() {
                    self.load_uri(
                        "av://lavfi:testsrc2=size=1280x720:rate=60:duration=300",
                        PathBuf::from("Sinal de Teste FFmpeg/libmpv (720p60)"),
                    )?;
                } else {
                    if self.get_flag(c"eof-reached").unwrap_or(false) {
                        self.set_double(c"time-pos", 0.0);
                        self.pending_seek = Some((0.0, Instant::now()));
                    }
                    self.set_flag(c"pause", false);
                }
            }
            PlayerCommand::Pause => self.set_flag(c"pause", true),
            PlayerCommand::TogglePlayPause => {
                if self.current_path.is_none() {
                    self.execute(PlayerCommand::Play)?;
                } else {
                    let eof = self.get_flag(c"eof-reached").unwrap_or(false);
                    if eof {
                        self.set_double(c"time-pos", 0.0);
                        self.pending_seek = Some((0.0, Instant::now()));
                        self.set_flag(c"pause", false);
                    } else {
                        let paused = self.get_flag(c"pause").unwrap_or(false);
                        self.set_flag(c"pause", !paused);
                    }
                }
            }
            PlayerCommand::Stop => {
                let c_cmd = c"stop";
                let args = [c_cmd.as_ptr(), ptr::null()];
                unsafe {
                    let _ = (self.api.command)(self.handle, args.as_ptr());
                }
                self.current_path = None;
                self.pending_seek = None;
                self.preview_worker.set_uri(None);
            }
            PlayerCommand::Seek(pos) => {
                let target = pos.as_secs_f64().max(0.0);
                self.set_double(c"time-pos", target);
                self.pending_seek = Some((target, Instant::now()));
                self.cached_status.position = Duration::from_secs_f64(target);
                self.first_frame_pending = true;
            }
            PlayerCommand::SeekRelative(delta_secs) => {
                let current = self
                    .pending_seek
                    .map(|(t, _)| t)
                    .or_else(|| self.get_double(c"time-pos"))
                    .unwrap_or(0.0);
                let dur = self.cached_status.duration.as_secs_f64();
                let target = if dur > 0.0 {
                    (current + delta_secs).clamp(0.0, dur)
                } else {
                    (current + delta_secs).max(0.0)
                };
                self.set_double(c"time-pos", target);
                self.pending_seek = Some((target, Instant::now()));
                self.cached_status.position = Duration::from_secs_f64(target);
                self.first_frame_pending = true;
            }
            PlayerCommand::SetVolume(vol) => {
                self.set_double(c"volume", f64::from(vol.clamp(0.0, 100.0)));
            }
            PlayerCommand::ToggleMute => {
                let muted = self.get_flag(c"mute").unwrap_or(false);
                self.set_flag(c"mute", !muted);
            }
            PlayerCommand::SetSpeed(speed) => {
                self.set_double(c"speed", f64::from(speed.clamp(0.25, 4.0)));
            }
        }
        self.status_dirty = true;
        Ok(())
    }

    fn status(&mut self) -> PlayerStatus {
        let poll_interval = if self.pending_seek.is_some() {
            Duration::from_millis(30)
        } else {
            Duration::from_millis(100)
        };
        // Evita bloquear o mutex interno do libmpv 60+ vezes por segundo durante renderização
        if self.status_dirty || self.last_status_poll.elapsed() >= poll_interval {
            self.refresh_status();
        }
        self.cached_status.clone()
    }

    fn poll_video_frame(&mut self, _target_size: (u32, u32)) -> Option<VideoFrame> {
        self.current_path.as_ref()?;

        let notified = self.has_update.swap(false, Ordering::AcqRel);
        if !notified && !self.first_frame_pending && self.pending_seek.is_none() {
            return None;
        }

        let flags = unsafe { (self.api.render_context_update)(self.render_ctx) };
        let has_new_frame = (flags & MPV_RENDER_UPDATE_FRAME) != 0;
        if !has_new_frame {
            return None;
        }

        // Mantém resolução de textura fixa e alinhada a 16 pixels (stride múltiplo de 64 bytes para SIMD AVX2).
        // O escalonamento para Fullscreen (1080p/1440p/4K) é feito gratuitamente pela GPU via shader wgpu!
        let (vid_w, vid_h) = self.cached_vid_size;
        let base_w = vid_w.clamp(320, 1280);
        let width = ((base_w + 15) & !15).max(64); // Stride (width * 4) garantido múltiplo de 64 bytes
        let height =
            (((width as f64) * (vid_h as f64) / (vid_w.max(1) as f64)) as u32).clamp(64, 1080) & !1;

        let stride: usize = (width as usize) * 4;
        let total_bytes = stride * (height as usize);
        if self.pixel_buffer.len() != total_bytes {
            self.pixel_buffer.resize(total_bytes, 255);
        }

        let mut size_param: [c_int; 2] = [width as c_int, height as c_int];
        let format_cstr = c"rgb0";
        let mut stride_param: usize = stride;
        let mut block_for_target_time: c_int = 0;

        let mut params = [
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_SW_SIZE,
                data: size_param.as_mut_ptr().cast::<c_void>(),
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_SW_FORMAT,
                data: format_cstr.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_SW_STRIDE,
                data: (&raw mut stride_param).cast::<c_void>(),
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_SW_POINTER,
                data: self.pixel_buffer.as_mut_ptr().cast::<c_void>(),
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME,
                data: (&raw mut block_for_target_time).cast::<c_void>(),
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let rc = unsafe { (self.api.render_context_render)(self.render_ctx, params.as_mut_ptr()) };
        if rc < 0 {
            return None;
        }

        // Define o canal Alpha = 255 em blocos fixos de 4 bytes (vetorizado via SIMD com opt-level >= 2)
        for px in self.pixel_buffer.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }

        self.first_frame_pending = self.pending_seek.is_some();
        self.frame_counter += 1;

        Some(VideoFrame {
            width,
            height,
            rgba: self.pixel_buffer.clone(),
            frame_id: self.frame_counter,
        })
    }

    fn poll_preview_frame(&mut self, position: Duration) -> Option<VideoFrame> {
        self.current_path.as_ref()?;
        self.preview_worker.request_and_poll(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_backend_loads_and_renders_frame() {
        let Ok(mut backend) = MpvBackend::try_new() else {
            // Em ambientes de CI sem a biblioteca nativa libmpv instalada, o fallback é testado em lib.rs
            return;
        };
        assert_eq!(backend.name(), "libmpv");

        backend
            .execute(PlayerCommand::Play)
            .expect("Deveria iniciar sinal lavfi testsrc2 no libmpv");

        let mut rendered = None;
        for _ in 0..40 {
            if let Some(frame) = backend.poll_video_frame((640, 360)) {
                rendered = Some(frame);
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let frame = rendered.expect("libmpv deveria ter renderizado ao menos 1 VideoFrame RGBA8");
        assert!(frame.width >= 64);
        assert!(frame.height >= 64);
        assert_eq!(frame.rgba.len(), (frame.width * frame.height * 4) as usize);
        assert_eq!(frame.rgba[3], 255);
    }
}
