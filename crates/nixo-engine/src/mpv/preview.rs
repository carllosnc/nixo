use std::collections::BTreeMap;
use std::ffi::{CString, c_double, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nixo_core::VideoFrame;

use super::ffi::{
    MPV_FORMAT_DOUBLE, MPV_FORMAT_FLAG, MPV_RENDER_PARAM_API_TYPE,
    MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME, MPV_RENDER_PARAM_INVALID, MPV_RENDER_PARAM_SW_FORMAT,
    MPV_RENDER_PARAM_SW_POINTER, MPV_RENDER_PARAM_SW_SIZE, MPV_RENDER_PARAM_SW_STRIDE,
    MPV_RENDER_UPDATE_FRAME, MpvApi, MpvRenderContext, MpvRenderParam, on_mpv_render_update,
};

struct PreviewShared {
    target_uri: Mutex<Option<String>>,
    target_sec: AtomicU32,
    cache: Mutex<BTreeMap<u32, VideoFrame>>,
    shutdown: AtomicBool,
}

pub(super) struct PreviewWorker {
    shared: Arc<PreviewShared>,
    thread: Option<std::thread::JoinHandle<()>>,
    last_requested_sec: u32,
    last_returned_frame_id: u64,
}

impl PreviewWorker {
    pub(super) fn spawn(api: Arc<MpvApi>) -> Self {
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

    pub(super) fn set_uri(&mut self, uri: Option<String>) {
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

    pub(super) fn request_and_poll(&mut self, position: Duration) -> Option<VideoFrame> {
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
            api.set_opt(handle, c"vo", c"libmpv");
            api.set_opt(handle, c"hwdec", c"no");
            api.set_opt(handle, c"aid", c"no");
            api.set_opt(handle, c"sid", c"no");
            api.set_opt(handle, c"sub", c"no");
            api.set_opt(handle, c"audio", c"no");
            api.set_opt(handle, c"pause", c"yes");
            api.set_opt(handle, c"keep-open", c"yes");
            api.set_opt(handle, c"hr-seek", c"no");
            api.set_opt(handle, c"demuxer-readahead-secs", c"0");
            api.set_opt(handle, c"cache", c"no");
            api.set_opt(handle, c"vd-lavc-fast", c"yes");
            api.set_opt(handle, c"vd-lavc-skiploopfilter", c"all");
            api.set_opt(handle, c"vd-lavc-skipidct", c"all");
            api.set_opt(handle, c"vd-lavc-threads", c"4");
            api.set_opt(handle, c"sws-scaler", c"fast-bilinear");
            api.set_opt(handle, c"sws-fast", c"yes");
            api.set_opt(handle, c"video-timing-offset", c"0");

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
