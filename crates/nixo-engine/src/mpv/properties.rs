use std::ffi::{CStr, CString, c_double, c_int, c_void};
use std::path::Path;
use std::ptr;
use std::time::{Duration, Instant};

use nixo_core::{
    MediaBackend, MediaMetadata, PlaybackState, PlayerStatus, SubtitleBackgroundStyle,
    SubtitleHealth, SubtitleStyle, TrackInfo, VideoAspect, VideoFit,
};

use super::MpvBackend;
use super::ffi::{MPV_FORMAT_DOUBLE, MPV_FORMAT_FLAG, MPV_FORMAT_INT64};

impl MpvBackend {
    pub(super) fn reset_subtitle_health_tracking(&mut self) {
        self.sub_track_activated_at = Instant::now();
        self.sub_cues_seen = 0;
        self.sub_style_changed_while_empty = false;
    }

    pub(super) fn get_double(&self, prop: &CStr) -> Option<f64> {
        let mut val: c_double = 0.0;
        // SAFETY: Chamada FFI para `mpv_get_property` com formato `DOUBLE` e ponteiro de saída válido.
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

    pub(super) fn get_i64(&self, prop: &CStr) -> Option<i64> {
        let mut val: i64 = 0;
        // SAFETY: Chamada FFI para `mpv_get_property` com formato `INT64` e ponteiro de saída válido.
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

    pub(super) fn get_flag(&self, prop: &CStr) -> Option<bool> {
        let mut val: c_int = 0;
        // SAFETY: Chamada FFI para `mpv_get_property` com formato `FLAG` e ponteiro de saída válido.
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

    pub(super) fn set_double(&self, prop: &CStr, val: f64) {
        let c_val: c_double = val;
        // SAFETY: Chamada FFI para `mpv_set_property` com formato `DOUBLE` e ponteiro válido.
        unsafe {
            let _ = (self.api.set_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_DOUBLE,
                (&raw const c_val).cast::<c_void>(),
            );
        }
    }

    pub(super) fn set_flag(&self, prop: &CStr, val: bool) {
        let c_val: c_int = i32::from(val);
        // SAFETY: Chamada FFI para `mpv_set_property` com formato `FLAG` e ponteiro válido.
        unsafe {
            let _ = (self.api.set_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_FLAG,
                (&raw const c_val).cast::<c_void>(),
            );
        }
    }

    pub(super) fn set_i64(&self, prop: &CStr, val: i64) {
        let c_val: i64 = val;
        // SAFETY: Chamada FFI para `mpv_set_property` com formato `INT64` e ponteiro válido.
        unsafe {
            let _ = (self.api.set_property)(
                self.handle,
                prop.as_ptr(),
                MPV_FORMAT_INT64,
                (&raw const c_val).cast::<c_void>(),
            );
        }
    }

    pub(super) fn get_string(&self, prop: &CStr) -> Option<String> {
        // SAFETY: Chamada FFI para `mpv_get_property_string`. A string retornada deve ser liberada com `mpv_free`.
        let ptr = unsafe { (self.api.get_property_string)(self.handle, prop.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            // SAFETY: `ptr` é ponteiro válido retornado pelo libmpv terminado em nulo.
            let s = unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned();
            // SAFETY: Liberação de memória retornada pela API com a função oficial `mpv_free`.
            unsafe { (self.api.free)(ptr.cast()) };
            Some(s)
        }
    }

    pub(super) fn set_property_string(&self, prop: &CStr, val: &CStr) {
        // SAFETY: Chamada FFI para `mpv_set_property_string` com ponteiros de string C válidos e terminados em nulo.
        unsafe {
            let _ = (self.api.set_property_string)(self.handle, prop.as_ptr(), val.as_ptr());
        }
    }

    pub(super) fn resolve_subtitle_fonts_dir() -> Option<std::path::PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(exe) = std::env::current_exe()
            && let Some(exe_dir) = exe.parent()
        {
            candidates.push(exe_dir.join("fonts"));
            candidates.push(exe_dir.join("assets").join("fonts"));
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("assets").join("fonts"));
                if let Some(grandparent) = parent.parent() {
                    candidates.push(grandparent.join("assets").join("fonts"));
                }
            }
        }
        candidates.push(std::path::PathBuf::from("assets").join("fonts"));
        candidates.push(std::path::PathBuf::from("fonts"));
        candidates.push(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("assets")
                .join("fonts"),
        );

        candidates.into_iter().find(|dir| dir.is_dir())
    }

    pub(super) fn apply_subtitle_style(&mut self, style: SubtitleStyle) {
        let style = style.sanitized();

        if let Some(fonts_dir) = Self::resolve_subtitle_fonts_dir() {
            let dir_str = fonts_dir.to_string_lossy();
            let current_dir = self.get_string(c"sub-fonts-dir").unwrap_or_default();
            if current_dir != dir_str.as_ref()
                && let Ok(c_dir) = CString::new(dir_str.as_ref())
            {
                self.set_property_string(c"sub-fonts-dir", &c_dir);
            }
        }

        if let Ok(c_font) = CString::new(style.font_family.as_str()) {
            self.set_property_string(c"sub-font", &c_font);
        }

        self.set_double(c"sub-scale", f64::from(style.scale_percent) / 100.0);

        let [r, g, b, _] = style.text_color;
        if let Ok(c_col) = CString::new(format!("#{r:02X}{g:02X}{b:02X}")) {
            self.set_property_string(c"sub-color", &c_col);
        }

        self.set_flag(c"sub-bold", style.bold);

        match style.background {
            SubtitleBackgroundStyle::None => {
                self.set_property_string(c"sub-border-style", c"outline-and-shadow");
                self.set_double(c"sub-border-size", style.border.border_size_px());
                self.set_double(c"sub-shadow-offset", style.shadow.offset_px());
                self.set_property_string(c"sub-back-color", c"#00000000");
            }
            SubtitleBackgroundStyle::Box | SubtitleBackgroundStyle::Strip => {
                self.set_property_string(c"sub-border-style", c"background-box");
                self.set_double(c"sub-border-size", style.border.border_size_px().max(3.5));
                self.set_double(c"sub-shadow-offset", style.shadow.offset_px());
                let alpha = ((u16::from(style.background_opacity) * 255 + 50) / 100).min(255) as u8;
                if let Ok(c_back) = CString::new(format!("#{alpha:02X}000000")) {
                    self.set_property_string(c"sub-back-color", &c_back);
                }
            }
        }

        let sub_pos = 100_i64.saturating_sub(i64::from(style.bottom_offset_percent));
        self.set_i64(c"sub-pos", sub_pos.clamp(10, 100));
        self.set_double(c"sub-blur", 0.25);
        self.set_flag(c"embeddedfonts", !style.override_ass);

        if style.override_ass {
            self.set_property_string(c"sub-ass-override", c"strip");
            let bold_val = if style.bold { -1 } else { 0 };
            let border_style_val = match style.background {
                SubtitleBackgroundStyle::None => 1,
                SubtitleBackgroundStyle::Box | SubtitleBackgroundStyle::Strip => 3,
            };
            let outline_val = match style.background {
                SubtitleBackgroundStyle::None => style.border.border_size_px(),
                SubtitleBackgroundStyle::Box | SubtitleBackgroundStyle::Strip => {
                    style.border.border_size_px().max(3.5)
                }
            };
            let shadow_val = style.shadow.offset_px();
            // Formato ASS de cor: &HAABBGGRR (Alpha invertido: 00 = opaco, FF = transparente)
            let primary_ass = format!("&H00{b:02X}{g:02X}{r:02X}");
            let back_alpha_ass = match style.background {
                SubtitleBackgroundStyle::None => 0x00_u8,
                SubtitleBackgroundStyle::Box | SubtitleBackgroundStyle::Strip => {
                    let opacity_u8 =
                        ((u16::from(style.background_opacity) * 255 + 50) / 100).min(255) as u8;
                    255_u8.saturating_sub(opacity_u8)
                }
            };
            let back_ass = format!("&H{back_alpha_ass:02X}000000");
            let overrides = format!(
                "FontName={},Bold={bold_val},PrimaryColour={primary_ass},OutlineColour={back_ass},BackColour={back_ass},BorderStyle={border_style_val},Outline={outline_val:.1},Shadow={shadow_val:.1},Blur=0.25",
                style.font_family
            );
            if let Ok(c_ov) = CString::new(overrides) {
                self.set_property_string(c"sub-ass-style-overrides", &c_ov);
            }
        } else {
            self.set_property_string(c"sub-ass-override", c"scale");
            self.set_property_string(c"sub-ass-style-overrides", c"");
        }

        if self.current_path.is_some() && self.sub_cues_seen == 0 {
            let active_text = self.get_string(c"sub-text").unwrap_or_default();
            if active_text.trim().is_empty() {
                self.sub_style_changed_while_empty = true;
            }
        }

        self.subtitle_style = style;
        self.first_frame_pending = true;
    }

    pub(super) fn refresh_status(&mut self) {
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
            if elapsed < Duration::from_millis(450)
                && (is_seeking || (!close_enough && elapsed < Duration::from_millis(150)))
            {
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

        let mut audio_tracks = Vec::new();
        let mut subtitle_tracks = Vec::new();
        let mut selected_sub_image_codec: Option<String> = None;
        let track_count = self.get_i64(c"track-list/count").unwrap_or(0);
        for i in 0..track_count {
            let prop_type_str = format!("track-list/{i}/type\0");
            let prop_id_str = format!("track-list/{i}/id\0");
            let prop_title_str = format!("track-list/{i}/title\0");
            let prop_lang_str = format!("track-list/{i}/lang\0");
            let prop_sel_str = format!("track-list/{i}/selected\0");

            let track_type = CStr::from_bytes_with_nul(prop_type_str.as_bytes())
                .ok()
                .and_then(|c| self.get_string(c));
            let id = CStr::from_bytes_with_nul(prop_id_str.as_bytes())
                .ok()
                .and_then(|c| self.get_i64(c))
                .unwrap_or(i + 1);
            let title = CStr::from_bytes_with_nul(prop_title_str.as_bytes())
                .ok()
                .and_then(|c| self.get_string(c));
            let lang = CStr::from_bytes_with_nul(prop_lang_str.as_bytes())
                .ok()
                .and_then(|c| self.get_string(c));
            let selected = CStr::from_bytes_with_nul(prop_sel_str.as_bytes())
                .ok()
                .and_then(|c| self.get_flag(c))
                .unwrap_or(false);

            let track = TrackInfo {
                id,
                title,
                lang,
                selected,
            };

            match track_type.as_deref() {
                Some("audio") => audio_tracks.push(track),
                Some("sub") => {
                    if selected {
                        let prop_img = format!("track-list/{i}/image\0");
                        let prop_codec = format!("track-list/{i}/codec\0");
                        let is_img = CStr::from_bytes_with_nul(prop_img.as_bytes())
                            .ok()
                            .and_then(|c| self.get_flag(c))
                            .unwrap_or(false);
                        let codec = CStr::from_bytes_with_nul(prop_codec.as_bytes())
                            .ok()
                            .and_then(|c| self.get_string(c));
                        let is_bitmap_codec = matches!(
                            codec.as_deref(),
                            Some(
                                "dvd_subtitle"
                                    | "hdmv_pgs_subtitle"
                                    | "pgssub"
                                    | "dvb_subtitle"
                                    | "xsub"
                                    | "vobsub"
                            )
                        );
                        if is_img || is_bitmap_codec {
                            selected_sub_image_codec =
                                Some(codec.unwrap_or_else(|| "bitmap".to_string()));
                        }
                    }
                    subtitle_tracks.push(track);
                }
                _ => {}
            }
        }
        let subtitles_visible = self.get_flag(c"sub-visibility").unwrap_or(true);
        let has_selected_sub = subtitle_tracks.iter().any(|t| t.selected);

        if let Some(sub_text) = self.get_string(c"sub-text")
            && !sub_text.trim().is_empty()
        {
            self.sub_cues_seen = self.sub_cues_seen.saturating_add(1);
            self.sub_style_changed_while_empty = false;
        }

        let is_inactive_track = self.sub_style_changed_while_empty
            || (has_selected_sub
                && self.sub_cues_seen == 0
                && pos_secs > 10.0
                && self.sub_track_activated_at.elapsed() >= Duration::from_secs(8));

        let subtitle_health = if self.current_path.is_none() || !subtitles_visible {
            SubtitleHealth::Ok
        } else if let Some(codec) = selected_sub_image_codec {
            SubtitleHealth::ImageBased { codec }
        } else if self.sub_cues_seen == 0 && is_inactive_track {
            SubtitleHealth::InactiveTrack
        } else {
            SubtitleHealth::Ok
        };

        let sub_delay = self.get_double(c"sub-delay").unwrap_or(0.0);
        let audio_delay = self.get_double(c"audio-delay").unwrap_or(0.0);

        self.cached_status = PlayerStatus {
            state,
            position: Duration::from_secs_f64(pos_secs),
            duration: Duration::from_secs_f64(dur_secs),
            volume,
            muted,
            speed,
            metadata,
            backend_name: self.name(),
            audio_tracks,
            subtitle_tracks,
            subtitles_visible,
            subtitle_style: self.subtitle_style.clone(),
            subtitle_health,
            sub_delay,
            audio_delay,
            aspect_ratio: self.aspect_ratio,
            video_fit: self.video_fit,
        };
        self.last_status_poll = Instant::now();
        self.status_dirty = false;
    }

    pub(super) fn set_aspect_ratio(&mut self, aspect: VideoAspect) {
        self.aspect_ratio = aspect;
        match aspect {
            VideoAspect::Auto => {
                self.set_property_string(c"video-aspect-override", c"-1");
            }
            VideoAspect::Ratio16_9 => {
                self.set_property_string(c"video-aspect-override", c"16:9");
            }
            VideoAspect::Ratio4_3 => {
                self.set_property_string(c"video-aspect-override", c"4:3");
            }
            VideoAspect::Ratio235_1 => {
                self.set_property_string(c"video-aspect-override", c"2.35:1");
            }
        }
        self.first_frame_pending = true;
    }

    pub(super) fn set_video_fit(&mut self, fit: VideoFit) {
        self.video_fit = fit;
        let panscan = match fit {
            VideoFit::Fit => 0.0,
            VideoFit::Fill => 1.0,
        };
        self.set_double(c"panscan", panscan);
        self.first_frame_pending = true;
    }

    pub(super) fn take_screenshot(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let path_str = path.to_string_lossy().into_owned();
        if let Ok(c_path) = CString::new(path_str) {
            let args = [
                c"screenshot-to-file".as_ptr(),
                c_path.as_ptr(),
                c"subtitles".as_ptr(),
                ptr::null(),
            ];
            // SAFETY: `args` é um array terminado em ponteiro nulo de strings C válidas passadas ao libmpv.
            unsafe {
                let _ = (self.api.command)(self.handle, args.as_ptr());
            }
        }
    }
}
