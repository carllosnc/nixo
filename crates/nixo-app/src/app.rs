use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, ColorImage, Context, CornerRadius, Id, Margin, Pos2, Rect, RichText, Sense,
    Stroke, TextureHandle, TextureOptions, Vec2, vec2,
};
use nixo_core::{MediaBackend, PlaybackState, PlayerCommand, PlayerStatus};
use tracing::{error, warn};

pub use crate::popovers::SettingsTab;
use crate::settings::{
    AppSettings, PlaybackSettings, SettingsStore, SubtitleSettings, WindowSettings,
};
use crate::widgets::{youtube_bottom_scrim_mesh, youtube_top_scrim_mesh};

pub(crate) const ACCENT_COLOR: Color32 = Color32::from_rgb(55, 167, 33); // Verde da marca Nixo (#37A721)
pub(crate) const BG_OLED_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
/// Tempo para ocultar o OSD quando o mouse fica parado sobre o vídeo (estilo YouTube).
const OSD_VIDEO_IDLE_TIMEOUT: Duration = Duration::from_millis(2000);
/// Tempo para ocultar o OSD rapidamente quando o mouse sai da janela do player.
const OSD_MOUSE_LEAVE_TIMEOUT: Duration = Duration::from_millis(300);

pub struct NixoApp {
    pub(crate) backend: Box<dyn MediaBackend>,
    pub(crate) status: PlayerStatus,
    pub(crate) video_texture: Option<TextureHandle>,
    pub(crate) preview_texture: Option<TextureHandle>,
    pub(crate) last_frame_id: u64,
    pub(crate) is_fullscreen: bool,
    pub(crate) last_mouse_move: Instant,
    pub(crate) mouse_left_window_at: Option<Instant>,
    pub(crate) last_pointer_pos: Option<Pos2>,
    pub(crate) seek_override: Option<(f64, Instant)>,
    pub(crate) last_seek_dispatch: Instant,
    pub(crate) error_banner: Option<String>,
    pub(crate) settings_tab: SettingsTab,
    pub(crate) titlebar_configured: bool,
    pub(crate) settings_store: SettingsStore,
    pub(crate) language: crate::i18n::Language,
    pub(crate) always_on_top: bool,
    pub(crate) aspect_ratio_fit: bool,
    pub(crate) saved_aspect_ratio_fit: bool,
    pub(crate) last_window_size: Vec2,
    pub(crate) toast: Option<crate::toast::ToastNotification>,
    pub(crate) home_state: crate::home::HomeState,
    pub(crate) media_folders: Vec<PathBuf>,
    pub(crate) playlist: nixo_core::Playlist,
    pub(crate) watch_history: HashMap<PathBuf, crate::settings::WatchHistoryEntry>,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) pending_resume: Option<Duration>,
    pub(crate) last_history_record: Instant,
    pub(crate) settings_dirty: bool,
    pub(crate) playlist_drawer_open: bool,
    pub(crate) playlist_focused_index: usize,
    pub(crate) playlist_keyboard_nav_active: bool,
    pub(crate) playlist_scroll_to_index: Option<usize>,
    pub(crate) is_mini_player: bool,
    pub(crate) saved_window_size: Option<Vec2>,
    pub(crate) saved_always_on_top: bool,
}

impl NixoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        Self::configure_theme(&cc.egui_ctx);

        let (settings_store, settings) = SettingsStore::load_default();

        let mut backend = nixo_engine::create_default_backend();
        Self::apply_settings(backend.as_mut(), &settings);

        let mut pending_resume = None;
        let mut playlist = nixo_core::Playlist::new();
        playlist.set_repeat_mode(settings.playback.repeat_mode);

        if let Some(path) = initial_file.as_ref() {
            if let Some(dir) = path.parent() {
                playlist = nixo_core::Playlist::from_folder_with_active(dir, path);
                playlist.set_repeat_mode(settings.playback.repeat_mode);
            }
            if let Some(entry) = settings.watch_history.get(path)
                && entry.position_secs > 5.0
                && !entry.is_completed()
            {
                pending_resume = Some(Duration::from_secs_f64(entry.position_secs));
            }
            if let Err(err) = backend.execute(PlayerCommand::LoadFile(path.clone())) {
                error!("Erro ao abrir arquivo inicial: {err}");
            }
        }

        let status = backend.status();

        if settings.window.always_on_top {
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
        }

        Self {
            backend,
            status,
            video_texture: None,
            preview_texture: None,
            last_frame_id: 0,
            is_fullscreen: false,
            last_mouse_move: Instant::now(),
            mouse_left_window_at: None,
            last_pointer_pos: None,
            seek_override: None,
            last_seek_dispatch: Instant::now(),
            error_banner: None,
            settings_tab: SettingsTab::Main,
            titlebar_configured: false,
            settings_store,
            language: settings.language,
            always_on_top: settings.window.always_on_top,
            aspect_ratio_fit: false,
            saved_aspect_ratio_fit: false,
            last_window_size: Vec2::ZERO,
            toast: None,
            home_state: crate::home::HomeState::new(),
            media_folders: settings.library.folders.clone(),
            playlist,
            watch_history: settings.watch_history.clone(),
            current_file: initial_file,
            pending_resume,
            last_history_record: Instant::now(),
            settings_dirty: false,
            playlist_drawer_open: false,
            playlist_focused_index: 0,
            playlist_keyboard_nav_active: false,
            playlist_scroll_to_index: None,
            is_mini_player: false,
            saved_window_size: None,
            saved_always_on_top: settings.window.always_on_top,
        }
    }

    /// Aplica as preferências persistidas ao motor de mídia (executado uma vez na inicialização).
    fn apply_settings(backend: &mut dyn MediaBackend, settings: &AppSettings) {
        fn run(backend: &mut dyn MediaBackend, cmd: PlayerCommand) {
            if let Err(err) = backend.execute(cmd) {
                warn!("Falha ao aplicar configuração salva: {err}");
            }
        }

        run(backend, PlayerCommand::SetVolume(settings.playback.volume));
        run(
            backend,
            PlayerCommand::SetSubtitleStyle(settings.subtitles.style.clone()),
        );

        // Mudo e visibilidade só possuem comandos de alternância: aplica apenas se divergirem.
        let status = backend.status();
        if status.muted != settings.playback.muted {
            run(backend, PlayerCommand::ToggleMute);
        }
        if status.subtitles_visible != settings.subtitles.visible {
            run(backend, PlayerCommand::ToggleSubtitleVisibility);
        }
    }

    /// Extrai do estado atual do player as preferências que devem ser persistidas.
    pub(crate) fn current_settings(&self) -> AppSettings {
        AppSettings {
            language: self.language,
            playback: PlaybackSettings {
                volume: self.status.volume,
                muted: self.status.muted,
                repeat_mode: self.playlist.repeat_mode(),
                autoplay: true,
            },
            subtitles: SubtitleSettings {
                visible: self.status.subtitles_visible,
                style: self.status.subtitle_style.clone(),
            },
            window: WindowSettings {
                always_on_top: self.always_on_top,
                aspect_ratio_fit: false,
            },
            library: crate::settings::LibrarySettings {
                folders: self.media_folders.clone(),
            },
            watch_history: self.watch_history.clone(),
        }
    }

    #[inline]
    pub(crate) fn t(&self) -> &'static crate::i18n::Translations {
        self.language.t()
    }

    /// Registra o progresso atual do vídeo ativo no histórico de reprodução.
    pub(crate) fn record_current_playback(&mut self) {
        if let Some(file) = &self.current_file {
            let pos = self.status.position.as_secs_f64();
            let dur = self.status.duration.as_secs_f64();
            if dur > 1.0 && pos > 0.0 {
                let entry = crate::settings::WatchHistoryEntry {
                    position_secs: pos,
                    duration_secs: dur,
                    last_watched_unix: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                };
                self.watch_history.insert(file.clone(), entry);
                self.settings_store.observe(self.current_settings());
                self.last_history_record = Instant::now();
            }
        }
    }

    pub(crate) fn dispatch(&mut self, cmd: PlayerCommand) {
        if matches!(
            cmd,
            PlayerCommand::SetVolume(_)
                | PlayerCommand::ToggleMute
                | PlayerCommand::SetSubtitleStyle(_)
                | PlayerCommand::ToggleSubtitleVisibility
        ) {
            self.settings_dirty = true;
        }

        if matches!(cmd, PlayerCommand::LoadFile(_) | PlayerCommand::Stop)
            || (matches!(cmd, PlayerCommand::Play) && self.status.state == PlaybackState::Stopped)
        {
            self.seek_override = None;
            self.preview_texture = None;
            if matches!(cmd, PlayerCommand::Stop) {
                self.record_current_playback();
                self.current_file = None;
                self.pending_resume = None;
                self.video_texture = None;
                self.aspect_ratio_fit = false;
                self.saved_aspect_ratio_fit = false;
                self.playlist = nixo_core::Playlist::new();
            } else {
                if let PlayerCommand::LoadFile(ref path) = cmd {
                    self.record_current_playback();
                    self.current_file = Some(path.clone());
                    self.pending_resume = self
                        .watch_history
                        .get(path)
                        .filter(|e| e.position_secs > 5.0 && !e.is_completed())
                        .map(|e| Duration::from_secs_f64(e.position_secs));
                    self.sync_playlist_for_file(path);
                }
                self.aspect_ratio_fit = self.saved_aspect_ratio_fit;
            }
        }
        match self.backend.execute(cmd) {
            Ok(()) => self.error_banner = None,
            Err(err) => {
                error!("Comando falhou: {err}");
                self.error_banner = Some(err.to_string());
            }
        }
        self.status = self.backend.status();
    }

    pub(crate) fn update_video_texture(&mut self, ctx: &Context, viewport_size: Vec2) {
        let ppp = ctx.pixels_per_point().max(1.0);
        let target_w = ((viewport_size.x * ppp).round() as u32).clamp(640, 2560);
        let target_h = ((viewport_size.y * ppp).round() as u32).clamp(360, 1440);

        if let Some(frame) = self.backend.poll_video_frame((target_w, target_h)) {
            self.last_frame_id = frame.frame_id;
            let (w, h) = (frame.width as usize, frame.height as usize);
            let total_pixels = w * h;

            // Zero-copy: converte o buffer RGBA diretamente em ColorImage com o mesmo layout de memória
            let image = match bytemuck::try_cast_vec::<u8, Color32>(frame.rgba) {
                Ok(pixels) if pixels.len() == total_pixels => ColorImage {
                    size: [w, h],
                    pixels,
                },
                Ok(pixels) => ColorImage::from_rgba_premultiplied(
                    [w, h],
                    &bytemuck::cast_vec::<Color32, u8>(pixels),
                ),
                Err((_, rgba)) => ColorImage::from_rgba_premultiplied([w, h], &rgba),
            };

            match &mut self.video_texture {
                Some(tex) => tex.set(image, TextureOptions::LINEAR),
                None => {
                    let tex = ctx.load_texture("nixo-video-surface", image, TextureOptions::LINEAR);
                    self.video_texture = Some(tex);
                }
            }
        }
    }
}

impl eframe::App for NixoApp {
    fn on_exit(&mut self) {
        self.record_current_playback();
        self.aspect_ratio_fit = false;
        self.saved_aspect_ratio_fit = false;
        self.settings_store.observe(self.current_settings());
        self.settings_store.flush();
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.titlebar_configured {
            self.titlebar_configured = Self::apply_windows_titlebar_color();
        }

        self.status = self.backend.status();
        if self.status.state == PlaybackState::Stopped {
            self.aspect_ratio_fit = false;
            self.saved_aspect_ratio_fit = false;
        } else if self.status.state == PlaybackState::Ended {
            self.handle_playback_ended();
        }

        if let Some(target) = self.pending_resume
            && self.status.state != PlaybackState::Stopped
            && self.status.duration >= Duration::from_secs(3)
        {
            self.pending_resume = None;
            self.dispatch(PlayerCommand::Seek(target));
            let secs = target.as_secs();
            self.show_toast(
                crate::toast::ToastIcon::Play,
                format!("Retomando de {:02}:{:02}", secs / 60, secs % 60),
            );
        }

        if self.status.state == PlaybackState::Playing
            && self.last_history_record.elapsed() >= Duration::from_secs(5)
        {
            self.record_current_playback();
        }

        // Persistência: detecta mudanças e grava em background após o debounce.
        if self.settings_dirty {
            self.settings_dirty = false;
            self.settings_store.observe(self.current_settings());
        }
        if let Some(wait) = self.settings_store.time_until_save() {
            ctx.request_repaint_after(wait);
        }

        self.handle_shortcuts_and_drops(ctx);

        // Mantém a proporção exata da janela ao redimensionar quando 'aspect_ratio_fit' está ativo
        let screen_size = ctx.screen_rect().size();
        if !self.is_fullscreen
            && self.aspect_ratio_fit
            && self.status.state != PlaybackState::Stopped
            && screen_size.x > 100.0
            && screen_size.y > 100.0
            && (self.last_window_size.x > 0.0 && self.last_window_size.y > 0.0)
        {
            let delta = screen_size - self.last_window_size;
            if delta.x.abs() > 1.0 || delta.y.abs() > 1.0 {
                let aspect = self.active_video_aspect();
                let current_aspect = screen_size.x / screen_size.y;
                if (current_aspect - aspect).abs() / aspect > 0.012 {
                    let target_size = if delta.x.abs() >= delta.y.abs() {
                        let w = screen_size.x.max(320.0);
                        Vec2::new(w, (w / aspect).round().max(180.0))
                    } else {
                        let h = screen_size.y.max(180.0);
                        Vec2::new((h * aspect).round().max(320.0), h)
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
                }
            }
        }
        self.last_window_size = screen_size;

        // Comportamento de hover idêntico ao YouTube:
        // 1. Se o mouse saiu da janela: oculta rapidamente após 300ms (OSD_MOUSE_LEAVE_TIMEOUT).
        // 2. Se o mouse está parado sobre o vídeo (fora da barra): oculta a barra e o cursor após 2s (OSD_VIDEO_IDLE_TIMEOUT).
        // 3. Se o mouse está sobre a barra de controles, arrastando um slider ou com menu aberto: nunca oculta.
        let should_show_osd = self.status.state != PlaybackState::Playing
            || (self.mouse_left_window_at.is_none()
                && self.last_mouse_move.elapsed() <= OSD_VIDEO_IDLE_TIMEOUT)
            || self
                .mouse_left_window_at
                .is_some_and(|t| t.elapsed() <= OSD_MOUSE_LEAVE_TIMEOUT);

        // Suave animação de fade in / fade out (280ms)
        let osd_alpha =
            ctx.animate_bool_with_time(Id::new("nixo_osd_fade_alpha"), should_show_osd, 0.28);

        if !should_show_osd && osd_alpha < 0.05 && self.mouse_left_window_at.is_none() {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }

        let panel_bg = if self.status.state != PlaybackState::Stopped {
            BG_OLED_BLACK
        } else {
            crate::components::tokens::BG_BASE
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(panel_bg))
            .show(ctx, |ui| {
                if self.status.state != PlaybackState::Stopped {
                    let avail = ui.available_size();
                    self.update_video_texture(ctx, avail);

                    let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
                    let screen_h = ctx.screen_rect().height();
                    let is_over_controls = osd_alpha > 0.15
                        && pointer_pos.is_some_and(|pos| {
                            pos.y >= (screen_h - 90.0).max(0.0) || pos.y <= 55.0
                        });

                    let (rect, response) = ui.allocate_exact_size(avail, Sense::click());
                    if response.double_clicked() && !is_over_controls {
                        self.toggle_fullscreen(ctx);
                    } else if response.clicked() && !is_over_controls {
                        self.dispatch(PlayerCommand::TogglePlayPause);
                    }

                    if let Some(tex) = &self.video_texture {
                        // Preserva proporção ou preenche tela sobre fundo preto OLED
                        let tex_size = tex.size_vec2();
                        let aspect = self
                            .status
                            .aspect_ratio
                            .ratio_f64()
                            .map(|r| r as f32)
                            .unwrap_or_else(|| (tex_size.x / tex_size.y.max(1.0)).max(0.1));
                        let (mut draw_w, mut draw_h) = (rect.width(), rect.width() / aspect);
                        let needs_h_clamp = match self.status.video_fit {
                            nixo_core::VideoFit::Fit => draw_h > rect.height(),
                            nixo_core::VideoFit::Fill => draw_h < rect.height(),
                        };
                        if needs_h_clamp {
                            draw_h = rect.height();
                            draw_w = draw_h * aspect;
                        }

                        let video_rect =
                            Rect::from_center_size(rect.center(), Vec2::new(draw_w, draw_h));
                        ui.painter().with_clip_rect(rect).image(
                            tex.id(),
                            video_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }

                    // Gradiente vertical estilo YouTube (vignette suave) com fade in / fade out
                    if osd_alpha > 0.005 {
                        let painter = ui.painter();
                        painter.add(egui::Shape::mesh(youtube_top_scrim_mesh(rect, osd_alpha)));
                        painter.add(egui::Shape::mesh(youtube_bottom_scrim_mesh(
                            rect, osd_alpha,
                        )));
                    }
                } else {
                    let rect = ui.max_rect();
                    let action = self.home_state.render(
                        ui,
                        &self.media_folders,
                        &self.watch_history,
                        rect,
                        self.t(),
                        self.language,
                    );
                    self.handle_home_action(action);
                }
            });

        // 2. Elementos Flutuantes (OSD): Renderizados com animação de fade suave
        if self.is_mini_player {
            self.render_mini_player_overlay(ctx);
        } else {
            self.render_floating_osd(ctx, osd_alpha);
            self.render_playlist_drawer(ctx);
        }

        // C. Banner de Erro Flutuante (se houver)
        if let Some(err) = &self.error_banner {
            egui::Area::new(Id::new("floating_error_banner"))
                .anchor(Align2::CENTER_TOP, vec2(0.0, 52.0))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(110, 20, 20))
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(14, 8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("⚠ {err}")).color(Color32::WHITE))
                        });
                });
        }

        // D. Notificação Toast Flutuante (ex: Captura de tela realizada)
        if self.toast.is_some() {
            egui::Area::new(Id::new("floating_toast_layer"))
                .order(egui::Order::Foreground)
                .interactable(true)
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    crate::toast::render_toast(ui, &mut self.toast, ctx.screen_rect())
                });
        }

        // Se a janela estiver minimizada, desacelera para taxa ultrabaixa (economizando 100% de GPU)
        if ctx.input(|i| i.viewport().minimized.unwrap_or(false)) {
            ctx.request_repaint_after(Duration::from_millis(250));
            return;
        }

        // Sincroniza repintura com taxa máxima de 60 FPS (16ms) mesmo em monitores de 144Hz/240Hz,
        // poupando GPU e eliminando multiplicação de carga em qualquer animação ou OSD visível.
        if self.status.state == PlaybackState::Playing
            || self.seek_override.is_some()
            || osd_alpha > 0.005
            || self.toast.is_some()
            || ctx.memory(|m| m.any_popup_open())
            || self.playlist_drawer_open
        {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
