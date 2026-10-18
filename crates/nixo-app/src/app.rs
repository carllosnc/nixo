use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, ColorImage, Context, CornerRadius, Id, Margin, Pos2, Rect, RichText, Sense,
    Stroke, TextureHandle, TextureOptions, Vec2, vec2,
};
use nixo_core::{MediaBackend, PlaybackState, PlayerCommand, PlayerStatus};
use tracing::{error, warn};

use crate::settings::{
    AppSettings, PlaybackSettings, SettingsStore, SubtitleSettings, WindowSettings,
};
use crate::widgets::vertical_gradient_mesh;

pub(crate) const ACCENT_COLOR: Color32 = Color32::from_rgb(55, 167, 33); // Verde da marca Nixo (#37A721)
pub(crate) const BG_OLED_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
/// Tempo para ocultar o OSD quando o mouse fica parado sobre o vídeo (estilo YouTube).
const OSD_VIDEO_IDLE_TIMEOUT: Duration = Duration::from_millis(2000);
/// Tempo para ocultar o OSD rapidamente quando o mouse sai da janela do player.
const OSD_MOUSE_LEAVE_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Main,
    Video,
    Speed,
    Audio,
    Subtitles,
    SubtitleStyle,
    SubtitleFont,
    SubtitleWeight,
    SubtitleSize,
    SubtitleColor,
    SubtitleBorder,
    SubtitleBackground,
    SubtitleShadow,
    SubtitlePosition,
}

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
    pub(crate) always_on_top: bool,
    pub(crate) aspect_ratio_fit: bool,
    pub(crate) last_window_size: Vec2,
    pub(crate) toast: Option<crate::toast::ToastNotification>,
    pub(crate) home_state: crate::home::HomeState,
    pub(crate) media_folders: Vec<PathBuf>,
}

impl NixoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        Self::configure_theme(&cc.egui_ctx);

        let (settings_store, settings) = SettingsStore::load_default();

        let mut backend = nixo_engine::create_default_backend();
        Self::apply_settings(backend.as_mut(), &settings);

        if let Some(path) = initial_file
            && let Err(err) = backend.execute(PlayerCommand::LoadFile(path))
        {
            error!("Erro ao abrir arquivo inicial: {err}");
        }

        let status = backend.status();

        let always_on_top = settings.window.always_on_top;
        if always_on_top {
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
        }
        let aspect_ratio_fit = settings.window.aspect_ratio_fit;

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
            always_on_top,
            aspect_ratio_fit,
            last_window_size: Vec2::ZERO,
            toast: None,
            home_state: crate::home::HomeState::new(),
            media_folders: settings.library.folders.clone(),
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
            playback: PlaybackSettings {
                volume: self.status.volume,
                muted: self.status.muted,
            },
            subtitles: SubtitleSettings {
                visible: self.status.subtitles_visible,
                style: self.status.subtitle_style.clone(),
            },
            window: WindowSettings {
                always_on_top: self.always_on_top,
                aspect_ratio_fit: self.aspect_ratio_fit,
            },
            library: crate::settings::LibrarySettings {
                folders: self.media_folders.clone(),
            },
        }
    }

    pub(crate) fn dispatch(&mut self, cmd: PlayerCommand) {
        if matches!(cmd, PlayerCommand::LoadFile(_) | PlayerCommand::Stop) {
            self.seek_override = None;
            self.preview_texture = None;
            if matches!(cmd, PlayerCommand::Stop) {
                self.video_texture = None;
            }
        }
        if let Err(err) = self.backend.execute(cmd) {
            error!("Comando falhou: {err}");
            self.error_banner = Some(err.to_string());
        } else {
            self.error_banner = None;
        }
        self.status = self.backend.status();
    }

    pub(crate) fn go_home(&mut self) {
        self.video_texture = None;
        self.home_state.go_root();
        self.dispatch(PlayerCommand::Stop);
    }

    pub(crate) fn handle_home_action(&mut self, action: crate::home::HomeAction) {
        use crate::home::HomeAction::*;
        match action {
            None => {}
            AddFolder => {
                if let Some(folder) = rfd::FileDialog::new().pick_folder()
                    && !self.media_folders.contains(&folder)
                {
                    self.media_folders.push(folder.clone());
                    self.settings_store.observe(self.current_settings());
                    self.home_state.scanner.invalidate(&folder);
                }
            }
            RemoveFolder(folder) => {
                self.media_folders.retain(|f| f != &folder);
                self.settings_store.observe(self.current_settings());
            }
            OpenFolder(path) => self.home_state.navigate_to(path),
            GoBack => self.home_state.go_back(),
            GoRoot => self.home_state.go_root(),
            OpenFile => self.open_file_dialog(),
            PlayFile(path) => self.dispatch(PlayerCommand::LoadFile(path)),
            PlayTestSignal => self.dispatch(PlayerCommand::Play),
        }
    }

    pub(crate) fn update_video_texture(&mut self, ctx: &Context, viewport_size: Vec2) {
        let ppp = ctx.pixels_per_point().max(1.0);
        let target_w = ((viewport_size.x * ppp).round() as u32).clamp(640, 2560);
        let target_h = ((viewport_size.y * ppp).round() as u32).clamp(360, 1440);

        if let Some(frame) = self.backend.poll_video_frame((target_w, target_h)) {
            self.last_frame_id = frame.frame_id;
            let image = ColorImage::from_rgba_premultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );

            match &mut self.video_texture {
                Some(tex) => tex.set(image, TextureOptions::LINEAR),
                None => {
                    let tex = ctx.load_texture("nixo-video-surface", image, TextureOptions::LINEAR);
                    self.video_texture = Some(tex);
                }
            }
        }
    }

    pub(crate) fn show_toast(&mut self, text: impl Into<String>, path: Option<PathBuf>) {
        self.toast = Some(crate::toast::ToastNotification::new(text.into(), path));
    }

    pub(crate) fn take_screenshot(&mut self) {
        let path = crate::toast::prepare_screenshot_path();
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        self.dispatch(PlayerCommand::TakeScreenshot(path.clone()));
        crate::toast::copy_screenshot_to_clipboard_async(path.clone());
        self.show_toast(format!("Captura copiada e salva: {filename}"), Some(path));
    }
}

impl eframe::App for NixoApp {
    fn on_exit(&mut self) {
        self.settings_store.observe(self.current_settings());
        self.settings_store.flush();
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.titlebar_configured {
            self.titlebar_configured = Self::apply_windows_titlebar_color();
        }

        self.status = self.backend.status();

        // Persistência: detecta mudanças e grava em background após o debounce.
        self.settings_store.observe(self.current_settings());
        if let Some(wait) = self.settings_store.time_until_save() {
            ctx.request_repaint_after(wait);
        }

        self.handle_shortcuts_and_drops(ctx);

        // Mantém a proporção exata da janela ao redimensionar quando 'aspect_ratio_fit' está ativo
        let screen_size = ctx.screen_rect().size();
        if !self.is_fullscreen
            && self.aspect_ratio_fit
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

        // 1. Painel Central: Canvas de vídeo ou Tela Inicial (Home)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG_OLED_BLACK))
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
                        let (draw_w, draw_h) = match self.status.video_fit {
                            nixo_core::VideoFit::Fit => {
                                let mut w = rect.width();
                                let mut h = w / aspect;
                                if h > rect.height() {
                                    h = rect.height();
                                    w = h * aspect;
                                }
                                (w, h)
                            }
                            nixo_core::VideoFit::Fill => {
                                let mut w = rect.width();
                                let mut h = w / aspect;
                                if h < rect.height() {
                                    h = rect.height();
                                    w = h * aspect;
                                }
                                (w, h)
                            }
                        };

                        let video_rect =
                            Rect::from_center_size(rect.center(), Vec2::new(draw_w, draw_h));
                        ui.painter().with_clip_rect(rect).image(
                            tex.id(),
                            video_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );

                        // Gradiente vertical estilo YouTube (vignette suave) com fade in / fade out
                        if osd_alpha > 0.005 {
                            let w = rect.width();
                            let h = rect.height();

                            let top_alpha = (160.0 * osd_alpha).round() as u8;
                            let top_h = 90.0_f32.min(h * 0.25);
                            let top_c = Color32::from_rgba_unmultiplied(0, 0, 0, top_alpha);
                            ui.painter().add(egui::Shape::mesh(vertical_gradient_mesh(
                                0.0,
                                top_h,
                                w,
                                top_c,
                                Color32::TRANSPARENT,
                            )));

                            let bottom_alpha = (220.0 * osd_alpha).round() as u8;
                            let bottom_h = 145.0_f32.min(h * 0.40);
                            let bottom_c = Color32::from_rgba_unmultiplied(0, 0, 0, bottom_alpha);
                            ui.painter().add(egui::Shape::mesh(vertical_gradient_mesh(
                                h - bottom_h,
                                h,
                                w,
                                Color32::TRANSPARENT,
                                bottom_c,
                            )));
                        }
                    }
                } else {
                    let rect = ui.max_rect();
                    let action = self.home_state.render(ui, &self.media_folders, rect);
                    self.handle_home_action(action);
                }
            });

        // 2. Elementos Flutuantes (OSD): Renderizados com animação de fade suave
        if osd_alpha > 0.005 && self.status.state != PlaybackState::Stopped {
            // A. Título e metadados no topo (integrado ao gradiente superior)
            if self.status.state != PlaybackState::Stopped || self.status.metadata.is_some() {
                egui::Area::new(Id::new("floating_top_pill"))
                    .anchor(Align2::CENTER_TOP, vec2(0.0, 16.0))
                    .interactable(false)
                    .show(ctx, |ui| {
                        ui.set_opacity(osd_alpha);
                        egui::Frame::NONE
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .inner_margin(Margin::symmetric(16, 8))
                            .show(ui, |ui| {
                                self.render_top_pill(ui);
                            });
                    });
            }

            // B. Barra Inferior Completa: estende-se de ponta a ponta integrada ao gradiente inferior do YouTube
            egui::Area::new(Id::new("bottom_controls_bar"))
                .anchor(Align2::LEFT_BOTTOM, vec2(0.0, 0.0))
                .interactable(osd_alpha > 0.15)
                .show(ctx, |ui| {
                    ui.set_opacity(osd_alpha);
                    let screen_w = ui.ctx().screen_rect().width();
                    egui::Frame::NONE
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(Margin {
                            left: 14,
                            right: 14,
                            top: 6,
                            bottom: 10,
                        })
                        .show(ui, |ui| {
                            ui.set_width(screen_w - 28.0);
                            self.render_bottom_controls(ctx, ui);
                        });
                });
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
                            ui.label(RichText::new(format!("⚠ {err}")).color(Color32::WHITE));
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
                    crate::toast::render_toast(ui, &mut self.toast, ctx.screen_rect());
                });
        }

        // Sincroniza repintura com o VSync quando reproduzindo ou concluindo um seek assíncrono
        if self.status.state == PlaybackState::Playing || self.seek_override.is_some() {
            ctx.request_repaint();
        } else if ctx.memory(|m| m.any_popup_open()) {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
}
