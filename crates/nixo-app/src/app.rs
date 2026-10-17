use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::epaint::{Mesh, Vertex};
use egui::{
    Align, Align2, Color32, ColorImage, Context, CornerRadius, FontId, Id, Layout, Margin, Pos2,
    Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions, Vec2, vec2,
};
use nixo_core::{MediaBackend, PlaybackState, PlayerCommand, PlayerStatus};
use tracing::{error, warn};

use crate::settings::{
    AppSettings, PlaybackSettings, SettingsStore, SubtitleSettings, WindowSettings,
};

pub(crate) const ACCENT_COLOR: Color32 = Color32::from_rgb(47, 229, 11); // Verde vibrante da marca Nixo (#2FE50B)
pub(crate) const BG_OLED_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
pub(crate) const OPAQUE_PANEL_BG: Color32 = Color32::from_rgb(12, 13, 17);
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
            },
        }
    }

    pub(crate) fn dispatch(&mut self, cmd: PlayerCommand) {
        if matches!(cmd, PlayerCommand::LoadFile(_) | PlayerCommand::Stop) {
            self.seek_override = None;
            self.preview_texture = None;
        }
        if let Err(err) = self.backend.execute(cmd) {
            error!("Comando falhou: {err}");
            self.error_banner = Some(err.to_string());
        } else {
            self.error_banner = None;
        }
        self.status = self.backend.status();
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

        // 1. Painel Central: Canvas de vídeo Edge-to-Edge (100% da área útil da janela)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG_OLED_BLACK))
            .show(ctx, |ui| {
                let avail = ui.available_size();
                self.update_video_texture(ctx, avail);

                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
                let screen_h = ctx.screen_rect().height();
                let is_over_controls = osd_alpha > 0.15
                    && pointer_pos
                        .is_some_and(|pos| pos.y >= (screen_h - 90.0).max(0.0) || pos.y <= 55.0);

                let (rect, response) = ui.allocate_exact_size(avail, Sense::click());
                if response.double_clicked() && !is_over_controls {
                    self.toggle_fullscreen(ctx);
                } else if response.clicked()
                    && !is_over_controls
                    && self.status.state != PlaybackState::Stopped
                {
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

                        // 1. Gradiente Superior Suave (escuro no topo, transparente para baixo)
                        let top_alpha = (160.0 * osd_alpha).round() as u8;
                        let top_gradient_h = 90.0_f32.min(h * 0.25);
                        let top_mesh = Mesh {
                            indices: vec![0, 1, 2, 2, 3, 0],
                            vertices: vec![
                                Vertex {
                                    pos: Pos2::new(0.0, 0.0),
                                    uv: Pos2::ZERO,
                                    color: Color32::from_rgba_unmultiplied(0, 0, 0, top_alpha),
                                },
                                Vertex {
                                    pos: Pos2::new(w, 0.0),
                                    uv: Pos2::ZERO,
                                    color: Color32::from_rgba_unmultiplied(0, 0, 0, top_alpha),
                                },
                                Vertex {
                                    pos: Pos2::new(w, top_gradient_h),
                                    uv: Pos2::ZERO,
                                    color: Color32::TRANSPARENT,
                                },
                                Vertex {
                                    pos: Pos2::new(0.0, top_gradient_h),
                                    uv: Pos2::ZERO,
                                    color: Color32::TRANSPARENT,
                                },
                            ],
                            ..Default::default()
                        };
                        ui.painter().add(egui::Shape::mesh(top_mesh));

                        // 2. Gradiente Inferior Estilo YouTube (transparente acima, preto profundo suave na base)
                        let bottom_alpha = (220.0 * osd_alpha).round() as u8;
                        let bottom_gradient_h = 145.0_f32.min(h * 0.40);
                        let b_top = h - bottom_gradient_h;
                        let bottom_mesh = Mesh {
                            indices: vec![0, 1, 2, 2, 3, 0],
                            vertices: vec![
                                Vertex {
                                    pos: Pos2::new(0.0, b_top),
                                    uv: Pos2::ZERO,
                                    color: Color32::TRANSPARENT,
                                },
                                Vertex {
                                    pos: Pos2::new(w, b_top),
                                    uv: Pos2::ZERO,
                                    color: Color32::TRANSPARENT,
                                },
                                Vertex {
                                    pos: Pos2::new(w, h),
                                    uv: Pos2::ZERO,
                                    color: Color32::from_rgba_unmultiplied(0, 0, 0, bottom_alpha),
                                },
                                Vertex {
                                    pos: Pos2::new(0.0, h),
                                    uv: Pos2::ZERO,
                                    color: Color32::from_rgba_unmultiplied(0, 0, 0, bottom_alpha),
                                },
                            ],
                            ..Default::default()
                        };
                        ui.painter().add(egui::Shape::mesh(bottom_mesh));
                    }
                } else {
                    // Empty State: Tela de boas-vindas minimalista em vidro translúcido sem bordas brancas
                    let card_size = Vec2::new(420.0, 200.0).min(rect.size() * 0.9);
                    let card_rect = Rect::from_center_size(rect.center(), card_size);

                    ui.painter().rect(
                        card_rect,
                        CornerRadius::same(18),
                        OPAQUE_PANEL_BG,
                        Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );

                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(card_rect.shrink(22.0))
                            .layout(Layout::top_down(Align::Center)),
                    );

                    child_ui.add_space(4.0);
                    child_ui.label(
                        RichText::new("N I X O")
                            .font(FontId::proportional(18.0))
                            .strong()
                            .color(ACCENT_COLOR),
                    );
                    child_ui.add_space(8.0);
                    child_ui.label(
                        RichText::new("Arraste e solte um arquivo de vídeo aqui")
                            .font(FontId::proportional(13.5))
                            .color(Color32::from_gray(190)),
                    );
                    child_ui.add_space(18.0);

                    child_ui.horizontal(|ui| {
                        let total_btn_w = 270.0;
                        let pad = ((ui.available_width() - total_btn_w) * 0.5).max(0.0);
                        ui.add_space(pad);

                        let open_btn = egui::Button::new(
                            RichText::new("📂 Abrir Arquivo")
                                .font(FontId::proportional(13.0))
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(38, 41, 52))
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([130.0, 34.0], open_btn).clicked() {
                            self.open_file_dialog();
                        }

                        let test_btn = egui::Button::new(
                            RichText::new("▶ Testar Sinal")
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_gray(215)),
                        )
                        .fill(Color32::from_rgb(26, 28, 35))
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([125.0, 34.0], test_btn).clicked() {
                            self.dispatch(PlayerCommand::Play);
                        }
                    });
                }
            });

        // 2. Elementos Flutuantes (OSD): Renderizados com animação de fade suave
        if osd_alpha > 0.005 {
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

        // Sincroniza repintura com o VSync quando reproduzindo ou concluindo um seek assíncrono
        if self.status.state == PlaybackState::Playing || self.seek_override.is_some() {
            ctx.request_repaint();
        } else if ctx.memory(|m| m.any_popup_open()) {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
}
