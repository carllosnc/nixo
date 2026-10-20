mod ffi;
mod preview;
mod properties;
#[cfg(test)]
mod tests;
pub mod thumbnail;
pub mod thumbnail_cache;

use std::ffi::{CString, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use nixo_core::{
    MediaBackend, NixoError, PlayerCommand, PlayerStatus, SubtitleStyle, VideoAspect, VideoFit,
    VideoFrame,
};
use tracing::debug;

use self::ffi::{
    MPV_RENDER_PARAM_API_TYPE, MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME, MPV_RENDER_PARAM_INVALID,
    MPV_RENDER_PARAM_SW_FORMAT, MPV_RENDER_PARAM_SW_POINTER, MPV_RENDER_PARAM_SW_SIZE,
    MPV_RENDER_PARAM_SW_STRIDE, MPV_RENDER_UPDATE_FRAME, MpvApi, MpvHandle, MpvRenderContext,
    MpvRenderParam, on_mpv_render_update,
};
use self::preview::PreviewWorker;

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
    last_render_size: (u32, u32),
    pending_seek: Option<(f64, Instant)>,
    preview_worker: PreviewWorker,
    subtitle_style: SubtitleStyle,
    sub_track_activated_at: Instant,
    sub_cues_seen: u32,
    aspect_ratio: VideoAspect,
    video_fit: VideoFit,
    cached_audio_tracks: Vec<nixo_core::TrackInfo>,
    cached_subtitle_tracks: Vec<nixo_core::TrackInfo>,
    cached_selected_sub_image_codec: Option<String>,
    last_tracks_poll: Instant,
    tracks_dirty: bool,
}

// SAFETY: O handle do mpv é acessado de forma exclusiva através de `&mut self` protegido pela trait.
unsafe impl Send for MpvBackend {}

impl MpvBackend {
    pub fn try_new() -> Result<Self, NixoError> {
        let api = Arc::new(MpvApi::load()?);

        // SAFETY: Inicialização do handle e do contexto de renderização do libmpv com ponteiros verificados.
        unsafe {
            let handle = (api.create)();
            if handle.is_null() {
                return Err(NixoError::InitError("mpv_create retornou nulo".into()));
            }

            for (k, v) in [
                (c"vo", c"libmpv"),
                (c"hwdec", c"auto-safe"),
                (c"vd-lavc-threads", c"0"),
                (c"sws-scaler", c"bicubic"),
                (c"sws-fast", c"no"),
                (c"blend-subtitles", c"no"),
                (c"sub-gauss", c"0.6"),
                (c"sub-gray", c"yes"),
                (c"keep-open", c"yes"),
                (c"video-timing-offset", c"0"),
                (c"hr-seek", c"default"),
                (c"hr-seek-framedrop", c"yes"),
                (c"demuxer-seekable-cache", c"yes"),
                (c"demuxer-max-bytes", c"150MiB"),
                (c"demuxer-max-back-bytes", c"50MiB"),
                (c"sub-use-margins", c"yes"),
                (c"sub-pos", c"92"),
                (c"sub-margin-y", c"52"),
                (c"screenshot-format", c"png"),
                (c"screenshot-png-compression", c"4"),
                (c"screenshot-tag-colorspace", c"yes"),
            ] {
                api.set_opt(handle, k, v);
            }

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
            let default_style = SubtitleStyle::default();

            let mut instance = Self {
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
                last_render_size: (0, 0),
                pending_seek: None,
                preview_worker,
                subtitle_style: default_style.clone(),
                sub_track_activated_at: Instant::now(),
                sub_cues_seen: 0,
                aspect_ratio: VideoAspect::default(),
                video_fit: VideoFit::default(),
                cached_audio_tracks: Vec::new(),
                cached_subtitle_tracks: Vec::new(),
                cached_selected_sub_image_codec: None,
                last_tracks_poll: Instant::now() - Duration::from_secs(10),
                tracks_dirty: true,
            };
            instance.apply_subtitle_style(default_style);
            Ok(instance)
        }
    }

    fn load_uri(&mut self, uri: &str, display_path: PathBuf) -> Result<(), NixoError> {
        let c_cmd = c"loadfile";
        let c_uri = CString::new(uri).map_err(|e| NixoError::InvalidMedia(e.to_string()))?;
        let args = [c_cmd.as_ptr(), c_uri.as_ptr(), ptr::null()];
        // SAFETY: `args` é um array terminado em nulo de ponteiros de string C válidos.
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
        self.tracks_dirty = true;
        self.reset_subtitle_health_tracking();
        self.preview_worker.set_uri(Some(uri.to_string()));
        Ok(())
    }

    fn mpv_cmd(&self, args: &[&std::ffi::CStr]) {
        let mut ptrs: Vec<*const std::ffi::c_char> = args.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(ptr::null());
        // SAFETY: `ptrs` é array terminado em nulo de strings C válidas passadas ao libmpv.
        unsafe {
            let _ = (self.api.command)(self.handle, ptrs.as_ptr());
        }
    }

    fn command_seek(&self, val: &str, mode: &std::ffi::CStr) {
        if let Ok(c_val) = CString::new(val) {
            self.mpv_cmd(&[c"seek", &c_val, mode]);
        }
    }
}

impl Drop for MpvBackend {
    fn drop(&mut self) {
        // SAFETY: Desregistra o callback e libera os recursos do libmpv na ordem correta (render_ctx antes do handle).
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
                } else if self.get_flag(c"eof-reached").unwrap_or(false) {
                    self.set_double(c"time-pos", 0.0);
                    self.pending_seek = Some((0.0, Instant::now()));
                    self.set_flag(c"pause", false);
                } else {
                    let paused = self.get_flag(c"pause").unwrap_or(false);
                    self.set_flag(c"pause", !paused);
                }
            }
            PlayerCommand::Stop => {
                self.mpv_cmd(&[c"stop"]);
                self.current_path = None;
                self.pending_seek = None;
                self.tracks_dirty = true;
                self.cached_audio_tracks.clear();
                self.cached_subtitle_tracks.clear();
                self.cached_selected_sub_image_codec = None;
                self.reset_subtitle_health_tracking();
                self.preview_worker.set_uri(None);
            }
            PlayerCommand::Seek(pos) => {
                let target = pos.as_secs_f64().max(0.0);
                self.command_seek(&format!("{target:.3}"), c"absolute");
                self.pending_seek = Some((target, Instant::now()));
                self.cached_status.position = Duration::from_secs_f64(target);
                self.first_frame_pending = true;
            }
            PlayerCommand::SeekRelative(delta_secs) => {
                self.command_seek(&format!("{delta_secs:.3}"), c"relative");
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
            PlayerCommand::SetAudioTrack(id) => {
                if let Some(id) = id {
                    self.set_i64(c"aid", id);
                } else {
                    self.set_property_string(c"aid", c"no");
                }
                self.tracks_dirty = true;
            }
            PlayerCommand::SetSubtitleTrack(id) => {
                if let Some(id) = id {
                    self.set_i64(c"sid", id);
                    self.set_flag(c"sub-visibility", true);
                } else {
                    self.set_property_string(c"sid", c"no");
                }
                self.tracks_dirty = true;
                self.reset_subtitle_health_tracking();
            }
            PlayerCommand::ToggleSubtitleVisibility => {
                let current = self.get_flag(c"sub-visibility").unwrap_or(true);
                self.set_flag(c"sub-visibility", !current);
                self.tracks_dirty = true;
            }
            PlayerCommand::CycleAudioTrack => {
                self.mpv_cmd(&[c"cycle", c"aid"]);
                self.tracks_dirty = true;
            }
            PlayerCommand::CycleSubtitleTrack => {
                self.mpv_cmd(&[c"cycle", c"sid"]);
                self.tracks_dirty = true;
                self.reset_subtitle_health_tracking();
            }
            PlayerCommand::LoadSubtitleFile(path) => {
                if let Ok(c_path) = CString::new(path.to_string_lossy().into_owned()) {
                    self.mpv_cmd(&[c"sub-add", &c_path, c"select"]);
                    self.set_flag(c"sub-visibility", true);
                    self.tracks_dirty = true;
                    self.reset_subtitle_health_tracking();
                }
            }
            PlayerCommand::SetSubtitleStyle(style) => self.apply_subtitle_style(style),
            PlayerCommand::SetSubDelay(delay) => self.set_double(c"sub-delay", delay),
            PlayerCommand::SetAudioDelay(delay) => self.set_double(c"audio-delay", delay),
            PlayerCommand::SetAspectRatio(aspect) => self.set_aspect_ratio(aspect),
            PlayerCommand::SetVideoFit(fit) => self.set_video_fit(fit),
            PlayerCommand::TakeScreenshot(path) => self.take_screenshot(&path),
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
        if self.status_dirty || self.last_status_poll.elapsed() >= poll_interval {
            self.refresh_status();
        }
        self.cached_status.clone()
    }

    fn poll_video_frame(&mut self, target_size: (u32, u32)) -> Option<VideoFrame> {
        self.current_path.as_ref()?;

        let (vid_w, vid_h) = self.cached_vid_size;
        let aspect = (vid_w.max(1) as f64) / (vid_h.max(1) as f64);

        // Se há legendas ativas, garante resolução suficiente para rasterização nítida;
        // caso contrário, adapta-se diretamente à resolução física da viewport (economizando CPU/RAM).
        let has_subs = self.cached_status.subtitles_visible
            && self
                .cached_status
                .subtitle_tracks
                .iter()
                .any(|t| t.selected);
        let (box_w, box_h) = if has_subs {
            (
                (target_size.0 as f64).clamp(1280.0, 2560.0),
                (target_size.1 as f64).clamp(720.0, 1440.0),
            )
        } else {
            (
                (target_size.0 as f64).clamp(640.0, 2560.0),
                (target_size.1 as f64).clamp(360.0, 1440.0),
            )
        };
        let mut fit_w = box_w;
        let mut fit_h = fit_w / aspect;
        if fit_h > box_h {
            fit_h = box_h;
            fit_w = fit_h * aspect;
        }

        // Alinha a largura a múltiplos de 16 pixels (stride múltiplo de 64 bytes para SIMD AVX2)
        let width = (((fit_w.round() as u32) + 15) & !15).clamp(64, 2560);
        let height = ((width as f64 / aspect).round() as u32).clamp(64, 1440) & !1;
        let size_changed =
            self.last_render_size != (0, 0) && self.last_render_size != (width, height);

        let notified = self.has_update.swap(false, Ordering::AcqRel);
        if !notified && !self.first_frame_pending && self.pending_seek.is_none() && !size_changed {
            return None;
        }

        // SAFETY: Consulta flags de atualização do contexto de renderização válido do libmpv.
        let flags = unsafe { (self.api.render_context_update)(self.render_ctx) };
        let has_new_frame = (flags & MPV_RENDER_UPDATE_FRAME) != 0;
        let can_rerender_existing =
            self.frame_counter > 0 && (self.first_frame_pending || size_changed);
        if !has_new_frame && !can_rerender_existing {
            return None;
        }

        let stride: usize = (width as usize) * 4;
        let total_bytes = stride * (height as usize);
        if self.pixel_buffer.len() != total_bytes {
            self.pixel_buffer.resize(total_bytes, 255);
        }

        let mut size_param: [c_int; 2] = [width as c_int, height as c_int];
        let format_cstr = c"rgba";
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

        // SAFETY: Renderiza o quadro atual no `self.pixel_buffer` pré-alocado com tamanho `stride * height`.
        let rc = unsafe { (self.api.render_context_render)(self.render_ctx, params.as_mut_ptr()) };
        if rc < 0 {
            return None;
        }

        self.last_render_size = (width, height);
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
