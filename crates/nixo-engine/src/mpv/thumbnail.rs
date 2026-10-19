//! Extrator assíncrono de miniaturas de vídeo em background usando o motor libmpv.
//!
//! Executa a leitura de arquivos e renderização de keyframes em uma thread isolada,
//! gerando quadros RGBA prontos para upload direto na GPU sem travar a interface.

use std::collections::HashSet;
use std::ffi::{CString, c_double, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use nixo_core::VideoFrame;

use super::ffi::{
    MPV_FORMAT_DOUBLE, MPV_FORMAT_FLAG, MPV_FORMAT_INT64, MPV_RENDER_PARAM_API_TYPE,
    MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME, MPV_RENDER_PARAM_INVALID, MPV_RENDER_PARAM_SW_FORMAT,
    MPV_RENDER_PARAM_SW_POINTER, MPV_RENDER_PARAM_SW_SIZE, MPV_RENDER_PARAM_SW_STRIDE,
    MPV_RENDER_UPDATE_FRAME, MpvApi, MpvRenderContext, MpvRenderParam, on_mpv_render_update,
};

/// Metadados e miniaturas de cenas extraídas assincronamente pelo worker do libmpv.
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub path: PathBuf,
    pub frames: Vec<VideoFrame>,
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Serviço em background para extração não-bloqueante de miniaturas de vídeo.
pub struct ThumbnailService {
    req_tx: Sender<PathBuf>,
    priority_tx: Sender<PathBuf>,
    res_rx: Receiver<ThumbnailResult>,
    shutdown: Arc<AtomicBool>,
}

impl ThumbnailService {
    /// Tenta inicializar o serviço de miniaturas.
    /// Retorna `None` caso o `libmpv` não esteja disponível no sistema.
    pub fn new() -> Option<Self> {
        let api = Arc::new(MpvApi::load().ok()?);
        let (req_tx, req_rx) = channel::<PathBuf>();
        let (priority_tx, priority_rx) = channel::<PathBuf>();
        let (res_tx, res_rx) = channel::<ThumbnailResult>();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);

        thread::Builder::new()
            .name("nixo-thumbnail-worker".into())
            .spawn(move || {
                Self::worker_loop(api, req_rx, priority_rx, res_tx, worker_shutdown);
            })
            .ok()?;

        Some(Self {
            req_tx,
            priority_tx,
            res_rx,
            shutdown,
        })
    }

    /// Enfileira uma requisição normal de miniatura.
    pub fn request(&self, path: PathBuf) {
        let _ = self.req_tx.send(path);
    }

    /// Enfileira uma requisição de alta prioridade (ex.: item sob hover do mouse).
    pub fn prioritize(&self, path: PathBuf) {
        let _ = self.priority_tx.send(path);
    }

    /// Retorna uma miniatura recém-processada, se houver alguma pronta.
    pub fn try_recv(&self) -> Option<ThumbnailResult> {
        self.res_rx.try_recv().ok()
    }

    #[allow(clippy::too_many_lines)]
    fn worker_loop(
        api: Arc<MpvApi>,
        req_rx: Receiver<PathBuf>,
        priority_rx: Receiver<PathBuf>,
        res_tx: Sender<ThumbnailResult>,
        shutdown: Arc<AtomicBool>,
    ) {
        // SAFETY: Cria uma instância dedicada e silenciosa do libmpv exclusiva para miniaturas.
        unsafe {
            let handle = (api.create)();
            if handle.is_null() {
                return;
            }

            for (k, v) in [
                (c"vo", c"libmpv"),
                (c"hwdec", c"no"),
                (c"aid", c"no"),
                (c"sid", c"no"),
                (c"sub", c"no"),
                (c"audio", c"no"),
                (c"pause", c"yes"),
                (c"keep-open", c"yes"),
                (c"hr-seek", c"no"),
                (c"demuxer-readahead-secs", c"0"),
                (c"cache", c"no"),
                (c"vd-lavc-fast", c"yes"),
                (c"vd-lavc-skiploopfilter", c"all"),
                (c"vd-lavc-skipidct", c"all"),
                (c"vd-lavc-threads", c"4"),
                (c"sws-scaler", c"fast-bilinear"),
                (c"sws-fast", c"yes"),
                (c"video-timing-offset", c"0"),
            ] {
                api.set_opt(handle, k, v);
            }

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

            let thumb_w = 160u32;
            let thumb_h = 90u32;
            let stride = (thumb_w as usize) * 4;
            let mut pixel_buf = vec![255u8; stride * (thumb_h as usize)];
            let mut frame_id = 0u64;
            let mut processed = HashSet::new();

            while !shutdown.load(Ordering::Acquire) {
                // Prioridade 1: itens com hover ativo. Prioridade 2: fila padrão
                let path = match priority_rx.try_recv() {
                    Ok(p) => p,
                    Err(_) => match req_rx.recv_timeout(Duration::from_millis(80)) {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                };

                if processed.contains(&path) {
                    continue;
                }
                if processed.len() > 1000 {
                    processed.clear();
                }
                processed.insert(path.clone());

                let Ok(c_path) = CString::new(path.to_string_lossy().as_bytes()) else {
                    continue;
                };

                let c_cmd = c"loadfile";
                let args = [c_cmd.as_ptr(), c_path.as_ptr(), ptr::null()];
                let _ = (api.command)(handle, args.as_ptr());

                let pause_val: c_int = 1;
                let _ = (api.set_property)(
                    handle,
                    c"pause".as_ptr(),
                    MPV_FORMAT_FLAG,
                    (&raw const pause_val).cast::<c_void>(),
                );

                // Drena atualizações residuais do arquivo anterior
                let _ = (api.render_context_update)(render_ctx);
                has_update.store(false, Ordering::Release);

                // Aguarda leitura dos metadados essenciais (duração e resolução)
                let start_wait = Instant::now();
                let mut found_dur: Option<f64> = None;
                let mut found_w: Option<u32> = None;
                let mut found_h: Option<u32> = None;

                while start_wait.elapsed() < Duration::from_millis(1200)
                    && !shutdown.load(Ordering::Acquire)
                {
                    let mut dur: c_double = 0.0;
                    if (api.get_property)(
                        handle,
                        c"duration".as_ptr(),
                        MPV_FORMAT_DOUBLE,
                        (&raw mut dur).cast::<c_void>(),
                    ) >= 0
                        && dur > 0.0
                    {
                        found_dur = Some(dur);
                    }

                    let mut vid_w: i64 = 0;
                    if (api.get_property)(
                        handle,
                        c"dwidth".as_ptr(),
                        MPV_FORMAT_INT64,
                        (&raw mut vid_w).cast::<c_void>(),
                    ) >= 0
                        && vid_w > 0
                    {
                        found_w = Some(vid_w as u32);
                    }

                    let mut vid_h: i64 = 0;
                    if (api.get_property)(
                        handle,
                        c"dheight".as_ptr(),
                        MPV_FORMAT_INT64,
                        (&raw mut vid_h).cast::<c_void>(),
                    ) >= 0
                        && vid_h > 0
                    {
                        found_h = Some(vid_h as u32);
                    }

                    // Se obteve duração e dimensões, ou passou de 120ms com duração, prossegue
                    if found_dur.is_some()
                        && (found_w.is_some() || start_wait.elapsed() > Duration::from_millis(120))
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }

                let dur = found_dur.unwrap_or(0.0);
                let ratios: &[f64] = if dur >= 120.0 {
                    &[0.15, 0.50, 0.85]
                } else if dur >= 30.0 {
                    &[0.20, 0.50, 0.80]
                } else if dur >= 10.0 {
                    &[0.30, 0.70]
                } else {
                    &[0.50]
                };

                let mut captured_frames = Vec::with_capacity(ratios.len());

                // Extração dos quadros de cena do vídeo (evita os primeiros segundos para eliminar telas pretas)
                for &ratio in ratios {
                    if shutdown.load(Ordering::Acquire) {
                        break;
                    }

                    let target_sec = if dur >= 120.0 {
                        (dur * ratio).clamp(10.0, (dur - 5.0).max(10.0))
                    } else if dur >= 30.0 {
                        (dur * ratio).clamp(5.0, (dur - 3.0).max(5.0))
                    } else if dur >= 10.0 {
                        (dur * ratio).clamp(3.0, (dur - 1.5).max(3.0))
                    } else if dur > 0.0 {
                        (dur * 0.5).max(1.5)
                    } else {
                        // Se a duração não foi obtida, busca aos 10s para evitar vinhetas pretas
                        10.0
                    };

                    if let Ok(sec_cstr) = CString::new(format!("{target_sec:.1}")) {
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

                    // Aguarda o quadro renderizado pelo libmpv (resolvendo imediatamente ao ficar pronto)
                    let render_start = Instant::now();
                    while render_start.elapsed() < Duration::from_millis(350)
                        && !shutdown.load(Ordering::Acquire)
                    {
                        let flags = (api.render_context_update)(render_ctx);
                        if (flags & MPV_RENDER_UPDATE_FRAME) != 0 {
                            let mut size_param: [c_int; 2] = [thumb_w as c_int, thumb_h as c_int];
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

                            if (api.render_context_render)(render_ctx, r_params.as_mut_ptr()) >= 0 {
                                for px in pixel_buf.as_chunks_mut::<4>().0 {
                                    px[3] = 255;
                                }
                                frame_id += 1;
                                captured_frames.push(VideoFrame {
                                    width: thumb_w,
                                    height: thumb_h,
                                    rgba: pixel_buf.clone(),
                                    frame_id,
                                });
                                break;
                            }
                        }
                        thread::sleep(Duration::from_millis(3));
                    }
                }

                let _ = res_tx.send(ThumbnailResult {
                    path: path.clone(),
                    frames: captured_frames,
                    duration_secs: found_dur,
                    width: found_w,
                    height: found_h,
                });

                // Libera o arquivo atual antes da próxima requisição
                let c_stop = c"stop";
                let args = [c_stop.as_ptr(), ptr::null()];
                let _ = (api.command)(handle, args.as_ptr());
            }

            (api.render_context_free)(render_ctx);
            (api.terminate_destroy)(handle);
        }
    }
}

impl Drop for ThumbnailService {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
    }
}
