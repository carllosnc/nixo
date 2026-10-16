use std::ffi::{CStr, CString, c_char, c_double, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
type FnMpvRenderContextCreate = unsafe extern "C" fn(
    *mut *mut MpvRenderContext,
    *mut MpvHandle,
    *mut MpvRenderParam,
) -> c_int;
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
            if let Ok(exe) = std::env::current_exe() {
                if let Some(dir) = exe.parent() {
                    list.push(dir.join("libmpv-2.dll"));
                }
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

pub struct MpvBackend {
    api: MpvApi,
    handle: *mut MpvHandle,
    render_ctx: *mut MpvRenderContext,
    has_update: Arc<AtomicBool>,
    current_path: Option<PathBuf>,
    frame_counter: u64,
    first_frame_pending: bool,
    pixel_buffer: Vec<u8>,
}

// SAFETY: O handle do mpv é acessado de forma exclusiva através de `&mut self` protegido pela trait.
unsafe impl Send for MpvBackend {}

impl MpvBackend {
    pub fn try_new() -> Result<Self, NixoError> {
        let api = MpvApi::load()?;

        unsafe {
            let handle = (api.create)();
            if handle.is_null() {
                return Err(NixoError::InitError("mpv_create retornou nulo".into()));
            }

            // Configura o mpv para renderizar no mpv_render_context sem bloquear a thread de UI
            Self::set_opt(&api, handle, c"vo", c"libmpv");
            Self::set_opt(&api, handle, c"hwdec", c"auto-copy");
            Self::set_opt(&api, handle, c"keep-open", c"yes");
            Self::set_opt(&api, handle, c"video-timing-offset", c"0");

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

            Ok(Self {
                api,
                handle,
                render_ctx,
                has_update,
                current_path: None,
                frame_counter: 0,
                first_frame_pending: false,
                pixel_buffer: Vec::new(),
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
        "libmpv (Hardware Accelerated)"
    }

    fn execute(&mut self, command: PlayerCommand) -> Result<(), NixoError> {
        match command {
            PlayerCommand::LoadFile(path) => {
                let path_str = path.to_string_lossy().into_owned();
                self.load_uri(&path_str, path)?;
                debug!("Arquivo carregado no libmpv: {path_str}");
            }
            PlayerCommand::Play => {
                if self.current_path.is_none() {
                    // Sinal de teste FFmpeg lavfi integrado ao libmpv caso nenhum arquivo tenha sido aberto ainda
                    self.load_uri(
                        "av://lavfi:testsrc2=size=1280x720:rate=60:duration=300",
                        PathBuf::from("Sinal de Teste FFmpeg/libmpv (720p60)"),
                    )?;
                } else {
                    if self.get_flag(c"eof-reached").unwrap_or(false) {
                        self.set_double(c"time-pos", 0.0);
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
            }
            PlayerCommand::Seek(pos) => {
                self.set_double(c"time-pos", pos.as_secs_f64());
                self.first_frame_pending = true;
            }
            PlayerCommand::SeekRelative(delta_secs) => {
                let current = self.get_double(c"time-pos").unwrap_or(0.0);
                let target = (current + delta_secs).max(0.0);
                self.set_double(c"time-pos", target);
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
        Ok(())
    }

    fn status(&mut self) -> PlayerStatus {
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

        let pos_secs = self.get_double(c"time-pos").unwrap_or(0.0).max(0.0);
        let dur_secs = self.get_double(c"duration").unwrap_or(0.0).max(0.0);
        let volume = self.get_double(c"volume").unwrap_or(80.0) as f32;
        let muted = self.get_flag(c"mute").unwrap_or(false);
        let speed = self.get_double(c"speed").unwrap_or(1.0) as f32;

        let width = self.get_i64(c"width").unwrap_or(0).max(0) as u32;
        let height = self.get_i64(c"height").unwrap_or(0).max(0) as u32;

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

        PlayerStatus {
            state,
            position: Duration::from_secs_f64(pos_secs),
            duration: Duration::from_secs_f64(dur_secs),
            volume,
            muted,
            speed,
            metadata,
            backend_name: self.name(),
        }
    }

    fn poll_video_frame(&mut self, target_size: (u32, u32)) -> Option<VideoFrame> {
        self.current_path.as_ref()?;

        let notified = self.has_update.swap(false, Ordering::AcqRel);
        if !notified && !self.first_frame_pending {
            return None;
        }

        let flags = unsafe { (self.api.render_context_update)(self.render_ctx) };
        let has_new_frame = (flags & MPV_RENDER_UPDATE_FRAME) != 0;
        if !has_new_frame {
            return None;
        }

        // Calcula a resolução ideal preservando a proporção real do vídeo
        let vid_w = self.get_i64(c"dwidth").or_else(|| self.get_i64(c"width")).unwrap_or(1280).max(16) as u32;
        let vid_h = self.get_i64(c"dheight").or_else(|| self.get_i64(c"height")).unwrap_or(720).max(16) as u32;

        let max_w = target_size.0.clamp(320, 1920).min(vid_w);
        let width = (max_w & !3).max(64); // Múltiplo de 4 pixels
        let height = (((width as f64) * (vid_h as f64) / (vid_w as f64)) as u32).clamp(64, 1080);

        let stride: usize = (width as usize) * 4;
        let total_bytes = stride * (height as usize);
        if self.pixel_buffer.len() != total_bytes {
            self.pixel_buffer.resize(total_bytes, 255);
        }

        let mut size_param: [c_int; 2] = [width as c_int, height as c_int];
        // mpv_render_param documenta "rgb0" como formato nativo de 4 bytes por pixel para SW renderer
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

        let rc =
            unsafe { (self.api.render_context_render)(self.render_ctx, params.as_mut_ptr()) };
        if rc < 0 {
            return None;
        }

        // Garante que o canal Alpha (byte 3 do formato rgb0) seja 255 (opaco) para a textura RGBA8 da GPU
        for px in self.pixel_buffer.chunks_exact_mut(4) {
            px[3] = 255;
        }

        self.first_frame_pending = false;
        self.frame_counter += 1;

        Some(VideoFrame {
            width,
            height,
            rgba: self.pixel_buffer.clone(),
            frame_id: self.frame_counter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_backend_loads_and_renders_frame() {
        let mut backend = MpvBackend::try_new().expect("Deveria carregar libmpv-2.dll de vendor/mpv");
        assert_eq!(backend.name(), "libmpv (Hardware Accelerated)");

        backend
            .execute(PlayerCommand::Play)
            .expect("Deveria iniciar sinal lavfi testsrc2 no libmpv");

        // Aguarda até 2 segundos para o primeiro frame ser decodificado pelo libmpv
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
