use std::path::PathBuf;
use std::time::{Duration, Instant};

use nixo_core::{
    MediaBackend, MediaMetadata, NixoError, PlaybackState, PlayerCommand, PlayerStatus,
    SubtitleStyle, TrackInfo, VideoFrame,
};

/// Motor sintético de fallback (Padrão de Teste 60 FPS) usado quando a biblioteca dinâmica
/// `libmpv` ainda não está instalada no sistema operacional, permitindo testar todo o pipeline
/// de texturas GPU (wgpu), seekbar, controles de reprodução e OSD imediatamente.
pub struct SyntheticBackend {
    state: PlaybackState,
    position: Duration,
    duration: Duration,
    volume: f32,
    muted: bool,
    speed: f32,
    metadata: Option<MediaMetadata>,
    last_tick: Instant,
    frame_counter: u64,
    dirty_frame: bool,
    audio_tracks: Vec<TrackInfo>,
    subtitle_tracks: Vec<TrackInfo>,
    subtitles_visible: bool,
    subtitle_style: SubtitleStyle,
}

impl Default for SyntheticBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntheticBackend {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: Duration::ZERO,
            duration: Duration::from_secs(180),
            volume: 80.0,
            muted: false,
            speed: 1.0,
            metadata: None,
            last_tick: Instant::now(),
            frame_counter: 0,
            dirty_frame: false,
            audio_tracks: vec![
                TrackInfo {
                    id: 1,
                    title: Some("Português (Original)".to_string()),
                    lang: Some("por".to_string()),
                    selected: true,
                },
                TrackInfo {
                    id: 2,
                    title: Some("English (Stereo)".to_string()),
                    lang: Some("eng".to_string()),
                    selected: false,
                },
            ],
            subtitle_tracks: vec![
                TrackInfo {
                    id: 1,
                    title: Some("Português [SRT]".to_string()),
                    lang: Some("por".to_string()),
                    selected: false,
                },
                TrackInfo {
                    id: 2,
                    title: Some("English [CC]".to_string()),
                    lang: Some("eng".to_string()),
                    selected: false,
                },
            ],
            subtitles_visible: true,
            subtitle_style: SubtitleStyle::default(),
        }
    }

    fn advance_clock(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if self.state == PlaybackState::Playing {
            let scaled = Duration::from_secs_f64(elapsed.as_secs_f64() * f64::from(self.speed));
            self.position = (self.position + scaled).min(self.duration);
            if self.position >= self.duration {
                self.state = PlaybackState::Ended;
            }
        }
    }
}

impl MediaBackend for SyntheticBackend {
    fn name(&self) -> &'static str {
        "Synthetic GPU Preview (Instale libmpv para decodificação real)"
    }

    fn execute(&mut self, command: PlayerCommand) -> Result<(), NixoError> {
        self.advance_clock();
        match command {
            PlayerCommand::LoadFile(path) => {
                let title = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Sinal de Vídeo Nixo".to_string());
                self.duration = Duration::from_secs(180);
                self.position = Duration::ZERO;
                self.state = PlaybackState::Playing;
                self.metadata = Some(MediaMetadata {
                    title,
                    path,
                    duration: self.duration,
                    video_width: 1280,
                    video_height: 720,
                });
                self.dirty_frame = true;
            }
            PlayerCommand::Play => {
                if self.metadata.is_none() {
                    self.execute(PlayerCommand::LoadFile(PathBuf::from(
                        "Demo_Nixo_720p60.mp4",
                    )))?;
                } else {
                    if self.state == PlaybackState::Ended {
                        self.position = Duration::ZERO;
                    }
                    self.state = PlaybackState::Playing;
                }
            }
            PlayerCommand::Pause => {
                if self.state == PlaybackState::Playing {
                    self.state = PlaybackState::Paused;
                }
            }
            PlayerCommand::TogglePlayPause => match self.state {
                PlaybackState::Playing => self.state = PlaybackState::Paused,
                PlaybackState::Paused | PlaybackState::Ended => {
                    if self.state == PlaybackState::Ended {
                        self.position = Duration::ZERO;
                    }
                    self.state = PlaybackState::Playing;
                }
                PlaybackState::Stopped => {
                    self.execute(PlayerCommand::LoadFile(PathBuf::from(
                        "Demo_Nixo_720p60.mp4",
                    )))?;
                }
            },
            PlayerCommand::Stop => {
                self.state = PlaybackState::Stopped;
                self.position = Duration::ZERO;
                self.metadata = None;
            }
            PlayerCommand::Seek(pos) => {
                self.position = pos.min(self.duration);
                if self.state == PlaybackState::Ended && self.position < self.duration {
                    self.state = PlaybackState::Paused;
                }
                self.dirty_frame = true;
            }
            PlayerCommand::SeekRelative(delta_secs) => {
                let next = (self.position.as_secs_f64() + delta_secs)
                    .clamp(0.0, self.duration.as_secs_f64());
                self.position = Duration::from_secs_f64(next);
                self.dirty_frame = true;
            }
            PlayerCommand::SetVolume(vol) => {
                self.volume = vol.clamp(0.0, 100.0);
            }
            PlayerCommand::ToggleMute => {
                self.muted = !self.muted;
            }
            PlayerCommand::SetSpeed(speed) => {
                self.speed = speed.clamp(0.25, 4.0);
            }
            PlayerCommand::SetAudioTrack(id) => {
                for track in &mut self.audio_tracks {
                    track.selected = Some(track.id) == id;
                }
            }
            PlayerCommand::SetSubtitleTrack(id) => {
                for track in &mut self.subtitle_tracks {
                    track.selected = Some(track.id) == id;
                }
                if id.is_some() {
                    self.subtitles_visible = true;
                }
            }
            PlayerCommand::ToggleSubtitleVisibility => {
                self.subtitles_visible = !self.subtitles_visible;
            }
            PlayerCommand::CycleAudioTrack => {
                if !self.audio_tracks.is_empty() {
                    let current_idx = self.audio_tracks.iter().position(|t| t.selected);
                    let next_idx = match current_idx {
                        Some(idx) => (idx + 1) % self.audio_tracks.len(),
                        None => 0,
                    };
                    for (i, track) in self.audio_tracks.iter_mut().enumerate() {
                        track.selected = i == next_idx;
                    }
                }
            }
            PlayerCommand::CycleSubtitleTrack => {
                // Ciclo: Desativado -> Faixa 1 -> Faixa 2 -> ... -> Desativado
                let current_idx = self
                    .subtitle_tracks
                    .iter()
                    .position(|t| t.selected && self.subtitles_visible);
                match current_idx {
                    Some(idx) => {
                        if idx + 1 < self.subtitle_tracks.len() {
                            for (i, track) in self.subtitle_tracks.iter_mut().enumerate() {
                                track.selected = i == idx + 1;
                            }
                            self.subtitles_visible = true;
                        } else {
                            for track in &mut self.subtitle_tracks {
                                track.selected = false;
                            }
                            self.subtitles_visible = false;
                        }
                    }
                    None => {
                        if !self.subtitle_tracks.is_empty() {
                            for (i, track) in self.subtitle_tracks.iter_mut().enumerate() {
                                track.selected = i == 0;
                            }
                            self.subtitles_visible = true;
                        }
                    }
                }
            }
            PlayerCommand::LoadSubtitleFile(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Legenda Externa".to_string());
                let next_id = (self.subtitle_tracks.len() as i64) + 1;
                for track in &mut self.subtitle_tracks {
                    track.selected = false;
                }
                self.subtitle_tracks.push(TrackInfo {
                    id: next_id,
                    title: Some(name),
                    lang: Some("ext".to_string()),
                    selected: true,
                });
                self.subtitles_visible = true;
            }
            PlayerCommand::SetSubtitleStyle(style) => {
                self.subtitle_style = style.sanitized();
            }
        }
        Ok(())
    }

    fn status(&mut self) -> PlayerStatus {
        self.advance_clock();
        PlayerStatus {
            state: self.state,
            position: self.position,
            duration: self.duration,
            volume: self.volume,
            muted: self.muted,
            speed: self.speed,
            metadata: self.metadata.clone(),
            backend_name: self.name(),
            audio_tracks: self.audio_tracks.clone(),
            subtitle_tracks: self.subtitle_tracks.clone(),
            subtitles_visible: self.subtitles_visible,
            subtitle_style: self.subtitle_style.clone(),
            subtitle_health: nixo_core::SubtitleHealth::Ok,
        }
    }

    fn poll_video_frame(&mut self, _target_size: (u32, u32)) -> Option<VideoFrame> {
        self.advance_clock();
        if self.state == PlaybackState::Stopped {
            return None;
        }
        if self.state != PlaybackState::Playing && !self.dirty_frame {
            return None;
        }
        self.dirty_frame = false;

        let width: u32 = 640;
        let height: u32 = 360;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        let t = self.position.as_secs_f32() * 2.2;
        let progress =
            (self.position.as_secs_f32() / self.duration.as_secs_f32().max(1.0)).clamp(0.0, 1.0);
        let scan_x = ((width as f32) * progress) as u32;

        for y in 0..height {
            let vf = (y as f32) / (height as f32);
            for x in 0..width {
                let uf = (x as f32) / (width as f32);
                let idx = ((y * width + x) * 4) as usize;

                // Gradiente animado estilo cinema/aurora com marcador de varredura temporal
                let wave1 = (uf * 6.0 + t).sin() * 0.5 + 0.5;
                let wave2 = (vf * 5.0 - t * 0.7).cos() * 0.5 + 0.5;
                let mut r = (24.0 + 70.0 * wave1 * (1.0 - vf * 0.4)) as u8;
                let mut g = (30.0 + 110.0 * wave2 * uf) as u8;
                let mut b = (65.0 + 155.0 * (0.5 * wave1 + 0.5 * wave2)) as u8;

                // Linha vertical indicando a posição exata do vídeo no frame
                if x.abs_diff(scan_x) <= 1 {
                    r = 255;
                    g = 120;
                    b = 60;
                }

                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }

        self.frame_counter += 1;
        Some(VideoFrame {
            width,
            height,
            rgba,
            frame_id: self.frame_counter,
        })
    }

    fn poll_preview_frame(&mut self, position: Duration) -> Option<VideoFrame> {
        if self.state == PlaybackState::Stopped && self.metadata.is_none() {
            return None;
        }

        let width: u32 = 160;
        let height: u32 = 90;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        let clamped_pos = position.min(self.duration);
        let t = clamped_pos.as_secs_f32() * 2.2;
        let progress =
            (clamped_pos.as_secs_f32() / self.duration.as_secs_f32().max(1.0)).clamp(0.0, 1.0);
        let scan_x = ((width as f32) * progress) as u32;

        for y in 0..height {
            let vf = (y as f32) / (height as f32);
            for x in 0..width {
                let uf = (x as f32) / (width as f32);
                let idx = ((y * width + x) * 4) as usize;

                let wave1 = (uf * 6.0 + t).sin() * 0.5 + 0.5;
                let wave2 = (vf * 5.0 - t * 0.7).cos() * 0.5 + 0.5;
                let mut r = (24.0 + 70.0 * wave1 * (1.0 - vf * 0.4)) as u8;
                let mut g = (30.0 + 110.0 * wave2 * uf) as u8;
                let mut b = (65.0 + 155.0 * (0.5 * wave1 + 0.5 * wave2)) as u8;

                if x.abs_diff(scan_x) <= 1 {
                    r = 255;
                    g = 120;
                    b = 60;
                }

                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }

        let bucket_id = (clamped_pos.as_millis() as u64 / 100).max(1);
        Some(VideoFrame {
            width,
            height,
            rgba,
            frame_id: bucket_id,
        })
    }
}
