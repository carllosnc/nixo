use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::epaint::{Mesh, Vertex};
use egui::{
    Align, Align2, Color32, ColorImage, Context, CornerRadius, FontId, Id, Key, Layout, Margin,
    Pos2, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2, ViewportCommand,
    vec2,
};
use nixo_core::{MediaBackend, PlaybackState, PlayerCommand, PlayerStatus, format_duration};
use tracing::error;

const ACCENT_COLOR: Color32 = Color32::from_rgb(255, 0, 0); // Vermelho característico do YouTube
const BG_OLED_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const OPAQUE_PANEL_BG: Color32 = Color32::from_rgb(12, 13, 17);
/// Tempo para ocultar o OSD quando o mouse fica parado sobre o vídeo (estilo YouTube).
const OSD_VIDEO_IDLE_TIMEOUT: Duration = Duration::from_millis(2000);
/// Tempo para ocultar o OSD rapidamente quando o mouse sai da janela do player.
const OSD_MOUSE_LEAVE_TIMEOUT: Duration = Duration::from_millis(300);

pub struct NixoApp {
    backend: Box<dyn MediaBackend>,
    status: PlayerStatus,
    video_texture: Option<TextureHandle>,
    preview_texture: Option<TextureHandle>,
    last_frame_id: u64,
    is_fullscreen: bool,
    last_mouse_move: Instant,
    mouse_left_window_at: Option<Instant>,
    last_pointer_pos: Option<Pos2>,
    seek_override: Option<(f64, Instant)>,
    last_seek_dispatch: Instant,
    error_banner: Option<String>,
}

impl NixoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        Self::configure_theme(&cc.egui_ctx);
        cc.egui_ctx
            .send_viewport_cmd(ViewportCommand::Visible(true));
        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Focus);

        let mut backend = nixo_engine::create_default_backend();
        if let Some(path) = initial_file
            && let Err(err) = backend.execute(PlayerCommand::LoadFile(path))
        {
            error!("Erro ao abrir arquivo inicial: {err}");
        }

        let status = backend.status();

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
        }
    }

    fn configure_theme(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_OLED_BLACK;
        visuals.window_fill = Color32::from_rgba_premultiplied(24, 24, 28, 248);
        visuals.window_stroke =
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25));
        visuals.window_corner_radius = CornerRadius::same(12);
        visuals.menu_corner_radius = CornerRadius::same(12);
        visuals.extreme_bg_color = BG_OLED_BLACK;
        visuals.selection.bg_fill = ACCENT_COLOR;
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 41, 52);
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 28, 35);
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        ctx.set_visuals(visuals);

        ctx.style_mut(|style| {
            style.spacing.slider_rail_height = 3.0;
            style.interaction.tooltip_delay = 0.12;
        });
    }

    fn dispatch(&mut self, cmd: PlayerCommand) {
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

    fn open_file_dialog(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Abrir Vídeo no Nixo")
            .add_filter(
                "Arquivos de Vídeo",
                &["mp4", "mkv", "webm", "avi", "mov", "m4v", "ts", "flv"],
            )
            .add_filter("Todos os Arquivos", &["*"])
            .pick_file();

        if let Some(path) = picked {
            self.dispatch(PlayerCommand::LoadFile(path));
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &Context) {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.is_fullscreen));
    }

    fn handle_shortcuts_and_drops(&mut self, ctx: &Context) {
        // Suporte a Drag & Drop de arquivos de vídeo direto na janela
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            if let Some(path) = file.path {
                self.dispatch(PlayerCommand::LoadFile(path));
                break;
            }
        }

        // Máquina de estados de hover inspirada no YouTube:
        // 1. Mouse fora da janela -> oculta rápido (300ms).
        // 2. Mouse sobre os controles/popups/arrastando -> nunca oculta.
        // 3. Mouse parado sobre o vídeo -> oculta após 2s (e esconde o cursor).
        let (pointer_pos, any_down) = ctx.input(|i| (i.pointer.hover_pos(), i.pointer.any_down()));
        let screen_h = ctx.screen_rect().height();
        let any_popup_open = ctx.memory(|m| m.any_popup_open());

        if let Some(pos) = pointer_pos {
            self.mouse_left_window_at = None;

            // Verifica se o mouse entrou na janela ou se moveu
            let moved = self
                .last_pointer_pos
                .is_none_or(|prev| (pos - prev).length_sq() > 0.25);
            if moved {
                self.last_mouse_move = Instant::now();
            }
            self.last_pointer_pos = Some(pos);

            // Se estiver sobre a barra inferior (últimos 90px), topo, arrastando ou com popup aberto, trava visível
            let is_over_controls = pos.y >= (screen_h - 90.0).max(0.0) || pos.y <= 55.0;
            if is_over_controls || any_down || any_popup_open || ctx.is_pointer_over_area() {
                self.last_mouse_move = Instant::now();
            }
        } else {
            self.last_pointer_pos = None;
            if any_down || any_popup_open {
                self.mouse_left_window_at = None;
                self.last_mouse_move = Instant::now();
            } else if self.mouse_left_window_at.is_none() {
                self.mouse_left_window_at = Some(Instant::now());
            }
        }

        // Atalhos de teclado padrão de players de vídeo
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.dispatch(PlayerCommand::TogglePlayPause);
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.dispatch(PlayerCommand::SeekRelative(5.0));
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            self.dispatch(PlayerCommand::SeekRelative(-5.0));
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            let next_vol = (self.status.volume + 5.0).min(100.0);
            self.dispatch(PlayerCommand::SetVolume(next_vol));
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            let next_vol = (self.status.volume - 5.0).max(0.0);
            self.dispatch(PlayerCommand::SetVolume(next_vol));
        }
        if ctx.input(|i| i.key_pressed(Key::M)) {
            self.dispatch(PlayerCommand::ToggleMute);
        }
        if ctx.input(|i| i.key_pressed(Key::F) || i.key_pressed(Key::F11)) {
            self.toggle_fullscreen(ctx);
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.is_fullscreen {
            self.toggle_fullscreen(ctx);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::O)) {
            self.open_file_dialog();
        }
    }

    fn update_video_texture(&mut self, ctx: &Context, viewport_size: Vec2) {
        let target_w = (viewport_size.x as u32).clamp(320, 1920);
        let target_h = (viewport_size.y as u32).clamp(180, 1080);

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

    /// Renderiza a cápsula superior translúcida com o título da mídia (estilo YouTube/minimalista).
    fn render_top_pill(&self, ui: &mut Ui) {
        let title_text = if let Some(meta) = &self.status.metadata {
            &meta.title
        } else if self.status.state != PlaybackState::Stopped {
            "Sinal de Teste GPU (60 FPS)"
        } else {
            "Nixo"
        };

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 9.0;

            // Título mais legível e presente
            ui.label(
                RichText::new(title_text)
                    .font(FontId::proportional(16.5))
                    .color(Color32::WHITE)
                    .strong(),
            );

            if let Some(meta) = &self.status.metadata
                && meta.video_width > 0
                && meta.video_height > 0
            {
                let badge = if meta.video_width >= 3840 {
                    "4K"
                } else if meta.video_width >= 1920 {
                    "FHD"
                } else {
                    "HD"
                };
                ui.label(
                    RichText::new(format!("[{badge}]"))
                        .font(FontId::monospace(12.5))
                        .color(Color32::from_rgb(175, 185, 205)),
                );
            }
        });
    }

    /// Renderiza um slider elegante, fino e com círculo menor na cor da barra (sem borda).
    fn render_compact_slider(
        ui: &mut Ui,
        value: &mut f64,
        range: std::ops::RangeInclusive<f64>,
        width: f32,
        rail_height: f32,
        handle_radius: f32,
    ) -> egui::Response {
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(width, 24.0), Sense::click_and_drag());

        let min = *range.start();
        let max = *range.end();
        let span = (max - min).max(0.0001);

        let is_pressing =
            response.is_pointer_button_down_on() || response.clicked() || response.dragged();

        if is_pressing && let Some(mouse_pos) = response.interact_pointer_pos() {
            let normalized = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            *value = min + (normalized as f64) * span;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let normalized = ((*value - min) / span).clamp(0.0, 1.0) as f32;
            let center_y = rect.center().y;
            let is_active = response.hovered() || is_pressing;
            let current_rail_h = if is_active {
                rail_height + 2.0
            } else {
                rail_height
            };

            // 1. Trilho de fundo (rail não preenchido)
            let rail_rect = Rect::from_min_max(
                Pos2::new(rect.left(), center_y - current_rail_h * 0.5),
                Pos2::new(rect.right(), center_y + current_rail_h * 0.5),
            );
            ui.painter().rect_filled(
                rail_rect,
                CornerRadius::same(2),
                Color32::from_rgb(38, 41, 52),
            );

            // 1b. Indicador de hover na trilha (estilo YouTube: faixa clara até a posição do mouse)
            if response.hovered()
                && let Some(hover_pos) = ui.ctx().input(|i| i.pointer.hover_pos())
            {
                let hover_x = hover_pos.x.clamp(rect.left(), rect.right());
                let hover_rect = Rect::from_min_max(
                    Pos2::new(rect.left(), center_y - current_rail_h * 0.5),
                    Pos2::new(hover_x, center_y + current_rail_h * 0.5),
                );
                ui.painter().rect_filled(
                    hover_rect,
                    CornerRadius::same(2),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 55),
                );
            }

            // 2. Trilho preenchido (progresso)
            let handle_x = rect.left() + normalized * rect.width();
            let filled_rect = Rect::from_min_max(
                Pos2::new(rect.left(), center_y - current_rail_h * 0.5),
                Pos2::new(handle_x, center_y + current_rail_h * 0.5),
            );
            ui.painter()
                .rect_filled(filled_rect, CornerRadius::same(2), ACCENT_COLOR);

            // 3. Círculo do indicador: menor, exatamente na cor da barra (ACCENT_COLOR) e sem borda!
            let radius = if is_active {
                handle_radius + 1.5
            } else {
                handle_radius
            };
            ui.painter().circle(
                Pos2::new(handle_x, center_y),
                radius,
                ACCENT_COLOR,
                Stroke::NONE,
            );
        }

        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }

    /// Renderiza o card flutuante de pré-visualização de frame (160x90 + timestamp, estilo YouTube) acima da seekbar.
    fn render_seek_preview_tooltip(
        &mut self,
        ctx: &Context,
        slider_rect: Rect,
        mouse_x: f32,
        hover_secs: f64,
    ) {
        let preview_dur = Duration::from_secs_f64(hover_secs);
        if let Some(frame) = self.backend.poll_preview_frame(preview_dur) {
            let color_image = ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );
            if let Some(tex) = &mut self.preview_texture {
                tex.set(color_image, TextureOptions::LINEAR);
            } else {
                self.preview_texture = Some(ctx.load_texture(
                    "seek_preview_thumb",
                    color_image,
                    TextureOptions::LINEAR,
                ));
            }
        }
        // Solicita repaint enquanto o tooltip estiver ativo
        ctx.request_repaint_after(Duration::from_millis(16));

        let thumb_w = 176.0;
        let thumb_h = 99.0;
        let pad = 4.0;
        let card_w = thumb_w + pad * 2.0;
        let card_h = thumb_h + 28.0;

        let screen_rect = ctx.screen_rect();
        let clamped_x = mouse_x.clamp(
            screen_rect.left() + card_w * 0.5 + 8.0,
            screen_rect.right() - card_w * 0.5 - 8.0,
        );
        let card_top_left = Pos2::new(clamped_x - card_w * 0.5, slider_rect.top() - card_h - 10.0);

        egui::Area::new(Id::new("seekbar_frame_preview"))
            .fixed_pos(card_top_left)
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                let (card_rect, _) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::hover());

                // Fundo escuro translúcido com borda fina elegante
                ui.painter().rect(
                    card_rect,
                    CornerRadius::same(8),
                    Color32::from_rgba_premultiplied(8, 9, 12, 235),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 42)),
                    egui::StrokeKind::Inside,
                );

                let thumb_rect = Rect::from_min_size(
                    Pos2::new(card_rect.left() + pad, card_rect.top() + pad),
                    vec2(thumb_w, thumb_h),
                );

                if let Some(tex) = &self.preview_texture {
                    ui.painter().image(
                        tex.id(),
                        thumb_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter().rect_filled(
                        thumb_rect,
                        CornerRadius::same(5),
                        Color32::from_rgb(18, 20, 26),
                    );
                }

                // Timestamp centralizado abaixo da miniatura
                let time_str = format_duration(preview_dur);
                let text_center = Pos2::new(
                    card_rect.center().x,
                    thumb_rect.bottom() + (card_rect.bottom() - thumb_rect.bottom()) * 0.5,
                );
                ui.painter().text(
                    text_center,
                    Align2::CENTER_CENTER,
                    time_str,
                    FontId::monospace(13.0),
                    Color32::WHITE,
                );
            });
    }

    /// Renderiza um tooltip flutuante estilo YouTube (cápsula escura elegante, sem quebra de linha, posicionado com precisão).
    fn render_player_tooltip(ctx: &Context, id: Id, anchor_rect: Rect, text: &str, alpha: f32) {
        if alpha <= 0.05 || text.is_empty() {
            return;
        }

        let font_id = FontId::proportional(12.0);
        let text_w = ctx.fonts(|f| {
            f.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE)
                .size()
                .x
        });
        let pill_w = text_w + 24.0; // Margem interna horizontal (10px de cada lado + 4px folga)
        let half_w = pill_w * 0.5;

        let screen = ctx.screen_rect();
        let clamped_x = anchor_rect
            .center()
            .x
            .clamp(screen.left() + half_w + 8.0, screen.right() - half_w - 8.0);
        let anchor_pos = Pos2::new(
            clamped_x,
            (anchor_rect.top() - 8.0).max(screen.top() + 10.0),
        );

        egui::Area::new(id)
            .pivot(Align2::CENTER_BOTTOM)
            .fixed_pos(anchor_pos)
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_opacity(alpha);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                // Estilo exato do YouTube: fundo cinza escuro translúcido com cantos 5px e borda hairline
                egui::Frame::NONE
                    .fill(Color32::from_rgba_premultiplied(28, 28, 28, 235))
                    .stroke(Stroke::new(
                        1.0_f32,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                    ))
                    .corner_radius(CornerRadius::same(5))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            if let Some(pos) = text.find(" (") {
                                let title = &text[..pos];
                                let shortcut = &text[pos..];
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(title)
                                            .font(font_id.clone())
                                            .color(Color32::WHITE),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(shortcut)
                                            .font(FontId::proportional(11.5))
                                            .color(Color32::from_gray(165)),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            } else if let Some(pos) = text.find(": ") {
                                let prefix = &text[..=pos];
                                let val = &text[pos + 1..];
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(prefix)
                                            .font(font_id.clone())
                                            .color(Color32::from_gray(185)),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(val)
                                            .font(font_id)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            } else {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(text).font(font_id).color(Color32::WHITE),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            }
                        });
                    });
            });
    }

    /// Renderiza um botão Ghost minimalista (estilo YouTube/IINA) com ícone vetorial desenhado diretamente na GPU.
    fn render_ghost_icon_btn(
        ui: &mut Ui,
        size: Vec2,
        icon: ControlIcon,
        tooltip: &str,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered();
            let pressed = response.is_pointer_button_down_on();

            // Fundo Ghost: invisível em repouso, destaque circular/arredondado translúcido no hover/clique
            if pressed {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 38),
                );
            } else if hovered {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                );
            }

            let fg = if hovered || pressed {
                Color32::WHITE
            } else {
                Color32::from_gray(220)
            };

            let c = rect.center();
            let painter = ui.painter();

            match icon {
                ControlIcon::Play => {
                    // Triângulo de Play opticamente centralizado (+1.5px à direita), maior e nítido
                    let pts = vec![
                        Pos2::new(c.x - 5.5, c.y - 8.0),
                        Pos2::new(c.x + 8.5, c.y),
                        Pos2::new(c.x - 5.5, c.y + 8.0),
                    ];
                    painter.add(egui::Shape::convex_polygon(pts, fg, Stroke::NONE));
                }
                ControlIcon::Pause => {
                    // Duas barras verticais simétricas proporcionais
                    let bar_w = 4.0;
                    let bar_h = 16.0;
                    let gap = 4.0;
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x - gap, c.y), vec2(bar_w, bar_h)),
                        CornerRadius::same(1),
                        fg,
                    );
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x + gap, c.y), vec2(bar_w, bar_h)),
                        CornerRadius::same(1),
                        fg,
                    );
                }
                ControlIcon::NextTrack => {
                    // Ícone Next estilo YouTube: triângulo apontando para direita + barra vertical
                    let pts = vec![
                        Pos2::new(c.x - 6.0, c.y - 7.5),
                        Pos2::new(c.x + 2.5, c.y),
                        Pos2::new(c.x - 6.0, c.y + 7.5),
                    ];
                    painter.add(egui::Shape::convex_polygon(pts, fg, Stroke::NONE));
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x + 5.5, c.y), vec2(2.8, 15.0)),
                        CornerRadius::same(1),
                        fg,
                    );
                }
                ControlIcon::SettingsGear => {
                    // Ícone de engrenagem estilo YouTube ampliado
                    let r_outer = 7.0;
                    let r_inner = 3.2;
                    let stroke = Stroke::new(2.0_f32, fg);
                    painter.circle_stroke(c, r_inner, stroke);
                    // Dentes da engrenagem (6 eixos com dentes bem definidos)
                    for i in 0..6 {
                        let angle = (i as f32) * std::f32::consts::PI / 3.0;
                        let p1 = Pos2::new(
                            c.x + (r_inner + 0.8) * angle.cos(),
                            c.y + (r_inner + 0.8) * angle.sin(),
                        );
                        let p2 =
                            Pos2::new(c.x + r_outer * angle.cos(), c.y + r_outer * angle.sin());
                        painter.line_segment([p1, p2], Stroke::new(2.8_f32, fg));
                    }
                }
                ControlIcon::Volume { level, muted } => {
                    // Corpo do alto-falante ampliado
                    let sx = c.x - 3.5;
                    painter.rect_filled(
                        Rect::from_min_max(
                            Pos2::new(sx - 6.0, c.y - 3.2),
                            Pos2::new(sx - 2.2, c.y + 3.2),
                        ),
                        CornerRadius::same(1),
                        fg,
                    );
                    let cone = vec![
                        Pos2::new(sx - 2.5, c.y - 3.2),
                        Pos2::new(sx + 2.8, c.y - 7.8),
                        Pos2::new(sx + 2.8, c.y + 7.8),
                        Pos2::new(sx - 2.5, c.y + 3.2),
                    ];
                    painter.add(egui::Shape::convex_polygon(cone, fg, Stroke::NONE));

                    let stroke = Stroke::new(2.0_f32, fg);
                    if muted || level <= 0.1 {
                        // Ícone 'X' de mudo ao lado do cone
                        let x0 = sx + 5.5;
                        let x1 = sx + 11.5;
                        painter.line_segment(
                            [Pos2::new(x0, c.y - 3.5), Pos2::new(x1, c.y + 3.5)],
                            stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(x1, c.y - 3.5), Pos2::new(x0, c.y + 3.5)],
                            stroke,
                        );
                    } else {
                        // Onda sonora interna (curva suave)
                        let origin = Pos2::new(sx + 1.5, c.y);
                        let wave1: Vec<Pos2> = (-3..=3)
                            .map(|i| {
                                let angle = (i as f32) * 0.22;
                                Pos2::new(
                                    origin.x + 5.5 * angle.cos(),
                                    origin.y + 5.5 * angle.sin(),
                                )
                            })
                            .collect();
                        painter.add(egui::Shape::line(wave1, stroke));

                        // Onda sonora externa (para volume >= 50%)
                        if level >= 50.0 {
                            let wave2: Vec<Pos2> = (-3..=3)
                                .map(|i| {
                                    let angle = (i as f32) * 0.24;
                                    Pos2::new(
                                        origin.x + 9.2 * angle.cos(),
                                        origin.y + 9.2 * angle.sin(),
                                    )
                                })
                                .collect();
                            painter.add(egui::Shape::line(wave2, stroke));
                        }
                    }
                }
                ControlIcon::OpenFile => {
                    // Ícone de pasta vetorial ampliado
                    let stroke = Stroke::new(2.0_f32, fg);
                    let body = Rect::from_min_max(
                        Pos2::new(c.x - 8.0, c.y - 4.2),
                        Pos2::new(c.x + 8.0, c.y + 6.5),
                    );
                    painter.rect_stroke(
                        body,
                        CornerRadius::same(2),
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                    // Aba superior esquerda da pasta
                    painter.line_segment(
                        [
                            Pos2::new(c.x - 7.5, c.y - 4.2),
                            Pos2::new(c.x - 6.5, c.y - 6.5),
                        ],
                        stroke,
                    );
                    painter.line_segment(
                        [
                            Pos2::new(c.x - 6.5, c.y - 6.5),
                            Pos2::new(c.x - 1.5, c.y - 6.5),
                        ],
                        stroke,
                    );
                    painter.line_segment(
                        [
                            Pos2::new(c.x - 1.5, c.y - 6.5),
                            Pos2::new(c.x - 0.2, c.y - 4.2),
                        ],
                        stroke,
                    );
                }
                ControlIcon::Fullscreen { is_fullscreen } => {
                    // Cantoneiras estilo YouTube (4 cantos em L) ampliadas
                    let r = 7.0;
                    let arm = 4.4;
                    let stroke = Stroke::new(2.0_f32, fg);

                    if !is_fullscreen {
                        // Apontando para fora (Expandir)
                        // Sup-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y - r),
                                Pos2::new(c.x - r + arm, c.y - r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y - r),
                                Pos2::new(c.x - r, c.y - r + arm),
                            ],
                            stroke,
                        );
                        // Sup-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y - r),
                                Pos2::new(c.x + r - arm, c.y - r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y - r),
                                Pos2::new(c.x + r, c.y - r + arm),
                            ],
                            stroke,
                        );
                        // Inf-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y + r),
                                Pos2::new(c.x - r + arm, c.y + r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y + r),
                                Pos2::new(c.x - r, c.y + r - arm),
                            ],
                            stroke,
                        );
                        // Inf-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y + r),
                                Pos2::new(c.x + r - arm, c.y + r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y + r),
                                Pos2::new(c.x + r, c.y + r - arm),
                            ],
                            stroke,
                        );
                    } else {
                        // Apontando para dentro (Sair da Tela Cheia)
                        let inner = r - arm + 1.0;
                        // Sup-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y - inner),
                                Pos2::new(c.x - r - 0.5, c.y - inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y - inner),
                                Pos2::new(c.x - inner, c.y - r - 0.5),
                            ],
                            stroke,
                        );
                        // Sup-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y - inner),
                                Pos2::new(c.x + r + 0.5, c.y - inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y - inner),
                                Pos2::new(c.x + inner, c.y - r - 0.5),
                            ],
                            stroke,
                        );
                        // Inf-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y + inner),
                                Pos2::new(c.x - r - 0.5, c.y + inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y + inner),
                                Pos2::new(c.x - inner, c.y + r + 0.5),
                            ],
                            stroke,
                        );
                        // Inf-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y + inner),
                                Pos2::new(c.x + r + 0.5, c.y + inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y + inner),
                                Pos2::new(c.x + inner, c.y + r + 0.5),
                            ],
                            stroke,
                        );
                    }
                }
            }
        }

        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // Tooltip flutuante centralizado estilo YouTube/IINA: cápsula escura elegante logo acima do botão
        if !tooltip.is_empty() {
            let tt_id = response.id.with("yt_tooltip");
            let is_hovered = response.hovered();
            let tt_alpha = ui.ctx().animate_bool_with_time(tt_id, is_hovered, 0.12);
            Self::render_player_tooltip(ui.ctx(), tt_id, rect, tooltip, tt_alpha);
        }

        response
    }

    /// Renderiza a barra inferior de ponta a ponta com seekbar, timestamps e controles de mídia.
    fn render_bottom_controls(&mut self, ctx: &Context, ui: &mut Ui) {
        let total_secs = self.status.duration.as_secs_f64().max(0.001);
        let backend_secs = self.status.position.as_secs_f64().clamp(0.0, total_secs);

        // Mantém a posição clicada/arrastada enquanto o demuxer do vídeo conclui o seek assíncrono,
        // impedindo que a barra laranja "volte" para o tempo antigo por alguns quadros.
        let mut current_secs = if let Some((target_secs, seek_instant)) = self.seek_override {
            if seek_instant.elapsed() < Duration::from_millis(400)
                && (backend_secs - target_secs).abs() > 0.5
            {
                target_secs.clamp(0.0, total_secs)
            } else {
                self.seek_override = None;
                backend_secs
            }
        } else {
            backend_secs
        };

        const BTN_HEIGHT: f32 = 40.0;

        // 1. Linha Superior: Seekbar 100% de ponta a ponta (estilo YouTube)
        let total_w = ui.available_width();
        let slider_resp =
            Self::render_compact_slider(ui, &mut current_secs, 0.0..=total_secs, total_w, 4.0, 6.0);
        let is_dragging = slider_resp.dragged();
        let drag_ended = slider_resp.drag_stopped();

        if slider_resp.changed() {
            let target_dur = Duration::from_secs_f64(current_secs);
            self.seek_override = Some((current_secs, Instant::now()));
            self.status.position = target_dur;

            // Se for um clique direto (não arrasto contínuo), dispara o seek imediatamente na hora.
            // Se estiver arrastando (scrubbing), aplica throttling de 60ms para fluidez sem sobrecarregar o demuxer.
            let should_dispatch = if is_dragging {
                self.last_seek_dispatch.elapsed() >= Duration::from_millis(60)
            } else {
                true
            };

            if should_dispatch {
                self.last_seek_dispatch = Instant::now();
                self.dispatch(PlayerCommand::Seek(target_dur));
            }
        } else if drag_ended && let Some((target_secs, _)) = self.seek_override {
            // Ao soltar o mouse após arrastar, garante que a posição final exata seja enviada
            if self.last_seek_dispatch.elapsed() >= Duration::from_millis(30) {
                self.seek_override = Some((target_secs, Instant::now()));
                self.last_seek_dispatch = Instant::now();
                self.dispatch(PlayerCommand::Seek(Duration::from_secs_f64(target_secs)));
            }
        }

        // Pré-visualização de frame (160x90 + timestamp) estilo YouTube ao passar o mouse na seekbar
        if (slider_resp.hovered() || slider_resp.is_pointer_button_down_on())
            && (self.status.state != PlaybackState::Stopped || self.status.metadata.is_some())
            && let Some(pointer_pos) =
                ctx.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()))
        {
            let s_rect = slider_resp.rect;
            let norm = ((pointer_pos.x - s_rect.left()) / s_rect.width()).clamp(0.0, 1.0);
            let hover_secs = (norm as f64) * total_secs;
            self.render_seek_preview_tooltip(ctx, s_rect, pointer_pos.x, hover_secs);
        }

        ui.add_space(2.0);

        // 2. Linha Inferior: Controles organizados exatamente como no YouTube
        // Esquerda: Play/Pause | Próximo | Mute + Volume Slider | 14:28 / 42:15
        // Direita: Velocidade (⚙ Engrenagem) | Abrir Arquivo | Tela Cheia
        let (row_rect, _resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), BTN_HEIGHT), Sense::hover());

        // A. Bloco da Esquerda (Play, Próximo, Volume, Tempo)
        let left_w = (row_rect.width() - 155.0).max(280.0);
        let left_rect = Rect::from_min_size(row_rect.left_top(), vec2(left_w, BTN_HEIGHT));
        let mut left_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(left_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        left_ui.spacing_mut().item_spacing.x = 6.0;
        left_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        // 1. Play / Pause
        let play_icon = match self.status.state {
            PlaybackState::Playing => ControlIcon::Pause,
            _ => ControlIcon::Play,
        };
        let play_tooltip = match self.status.state {
            PlaybackState::Playing => "Pausar (Espaço)",
            _ => "Reproduzir (Espaço)",
        };
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(42.0, BTN_HEIGHT),
            play_icon,
            play_tooltip,
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::TogglePlayPause);
        }

        // 2. Próximo (+5s)
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(38.0, BTN_HEIGHT),
            ControlIcon::NextTrack,
            "Avançar 5s (Seta Direita)",
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::SeekRelative(5.0));
        }

        // 3. Ícone de Volume / Mute
        let vol_tooltip = if self.status.muted {
            "Ativar som (M)"
        } else {
            "Desativar som (M)"
        };
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(38.0, BTN_HEIGHT),
            ControlIcon::Volume {
                level: self.status.volume,
                muted: self.status.muted,
            },
            vol_tooltip,
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::ToggleMute);
        }

        // 4. Slider de Volume inline (estilo YouTube: diretamente ao lado do alto-falante)
        let vol = if self.status.muted {
            0.0
        } else {
            self.status.volume
        };
        let mut vol_f64 = vol as f64;
        let vol_resp =
            Self::render_compact_slider(&mut left_ui, &mut vol_f64, 0.0..=100.0, 85.0, 4.0, 5.0);
        if vol_resp.changed() {
            let next_vol = vol_f64 as f32;
            if self.status.muted && next_vol > 0.0 {
                self.dispatch(PlayerCommand::ToggleMute);
            }
            self.dispatch(PlayerCommand::SetVolume(next_vol));
        }

        // Tooltip flutuante do slider de volume ("Volume: 80%" / "Volume: Mudo")
        let is_vol_active =
            vol_resp.hovered() || vol_resp.dragged() || vol_resp.is_pointer_button_down_on();
        let vol_tt_id = vol_resp.id.with("vol_slider_tooltip");
        let vol_alpha = ctx.animate_bool_with_time(vol_tt_id, is_vol_active, 0.12);
        let vol_text = if self.status.muted {
            "Volume: Mudo".to_string()
        } else {
            format!("Volume: {:.0}%", self.status.volume)
        };
        Self::render_player_tooltip(ctx, vol_tt_id, vol_resp.rect, &vol_text, vol_alpha);

        left_ui.add_space(8.0);

        // 5. Timestamps no estilo YouTube: "14:28 / 42:15" alinhado à esquerda após o volume
        let display_pos = Duration::from_secs_f64(current_secs);
        let time_text = format!(
            "{} / {}",
            format_duration(display_pos),
            format_duration(self.status.duration)
        );
        left_ui.label(
            RichText::new(time_text)
                .font(FontId::monospace(13.5))
                .color(Color32::from_gray(215)),
        );

        // B. Bloco da Direita (Configurações ⚙, Abrir Arquivo, Tela Cheia)
        let right_w = 145.0;
        let right_rect = Rect::from_min_size(
            Pos2::new(row_rect.right() - right_w, row_rect.top()),
            vec2(right_w, BTN_HEIGHT),
        );
        let mut right_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(right_rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        right_ui.spacing_mut().item_spacing.x = 6.0;
        right_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        // 1. Tela Cheia (na extremidade direita)
        let fs_tooltip = if self.is_fullscreen {
            "Sair da tela cheia (F)"
        } else {
            "Tela cheia (F)"
        };
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::Fullscreen {
                is_fullscreen: self.is_fullscreen,
            },
            fs_tooltip,
        )
        .clicked()
        {
            self.toggle_fullscreen(ctx);
        }

        // 2. Abrir Arquivo (Ctrl+O)
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::OpenFile,
            "Abrir arquivo (Ctrl+O)",
        )
        .clicked()
        {
            self.open_file_dialog();
        }

        // 3. Configurações / Velocidade (Ícone de Engrenagem ⚙ estilo YouTube)
        let speed = self.status.speed;
        let speed_tooltip = format!("Configurações ({speed:.2}x)");
        let gear_resp = Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::SettingsGear,
            &speed_tooltip,
        );

        let popup_id = Id::new("settings_speed_popup_menu");
        if gear_resp.clicked() {
            ctx.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        if ctx.memory(|mem| mem.is_popup_open(popup_id)) {
            let screen = ctx.screen_rect();
            let popup_w = 205.0;
            // Posiciona a popup acima do botão de configurações, garantindo margem confortável de 18px da borda da janela
            let clamped_x = (gear_resp.rect.center().x).clamp(
                screen.left() + popup_w * 0.5 + 16.0,
                screen.right() - popup_w * 0.5 - 18.0,
            );
            let anchor_pos = Pos2::new(clamped_x, gear_resp.rect.top() - 10.0);

            let mut should_close = false;
            let mut popup_rect = Rect::NOTHING;

            egui::Area::new(popup_id)
                .pivot(Align2::CENTER_BOTTOM)
                .fixed_pos(anchor_pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.set_width(popup_w);
                    let frame_resp = egui::Frame::NONE
                        .fill(Color32::from_rgba_premultiplied(26, 26, 26, 245))
                        .stroke(Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 28),
                        ))
                        .corner_radius(CornerRadius::same(12))
                        .shadow(egui::Shadow {
                            offset: [0, 8],
                            blur: 16,
                            spread: 0,
                            color: Color32::from_black_alpha(160),
                        })
                        .inner_margin(Margin::symmetric(8, 8))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;

                            // Título estilo YouTube
                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("Velocidade da reprodução")
                                        .font(FontId::proportional(12.5))
                                        .color(Color32::from_gray(175)),
                                );
                            });

                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(3.0);

                            // Opções de velocidade no layout idêntico ao YouTube (check lateral vetorial + hover suave)
                            for preset in [0.25_f32, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0] {
                                let is_selected = (speed - preset).abs() < 0.01;
                                let label_text = if (preset - 1.0).abs() < 0.01 {
                                    "Normal".to_string()
                                } else {
                                    format!("{preset}")
                                };

                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 32.0),
                                    Sense::click(),
                                );
                                let row_hovered = row_resp.hovered();

                                if row_hovered {
                                    ui.painter().rect_filled(
                                        row_rect,
                                        CornerRadius::same(7),
                                        Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                                    );
                                }

                                let center_y = row_rect.center().y;
                                let check_x = row_rect.left() + 16.0;
                                let text_x = row_rect.left() + 34.0;

                                // Ícone vetorial nítido de Checkmark (substitui o glifo Unicode ausente que causava o □ quadrado)
                                if is_selected {
                                    let stroke = Stroke::new(2.0_f32, Color32::WHITE);
                                    let painter = ui.painter();
                                    painter.line_segment(
                                        [
                                            Pos2::new(check_x - 5.0, center_y - 0.5),
                                            Pos2::new(check_x - 1.5, center_y + 3.5),
                                        ],
                                        stroke,
                                    );
                                    painter.line_segment(
                                        [
                                            Pos2::new(check_x - 1.5, center_y + 3.5),
                                            Pos2::new(check_x + 5.5, center_y - 4.5),
                                        ],
                                        stroke,
                                    );
                                }

                                let text_color = if is_selected || row_hovered {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(220)
                                };

                                ui.painter().text(
                                    Pos2::new(text_x, center_y),
                                    Align2::LEFT_CENTER,
                                    label_text,
                                    FontId::proportional(13.5),
                                    text_color,
                                );

                                let row_resp =
                                    row_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                                if row_resp.clicked() {
                                    self.dispatch(PlayerCommand::SetSpeed(preset));
                                    should_close = true;
                                }
                            }
                        });
                    popup_rect = frame_resp.response.rect;
                });

            // Fecha ao clicar fora ou pressionar Escape
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                should_close = true;
            }
            if ctx.input(|i| i.pointer.any_pressed())
                && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
                && !popup_rect.contains(pos)
                && !gear_resp.rect.contains(pos)
            {
                should_close = true;
            }

            if should_close {
                ctx.memory_mut(|mem| mem.close_popup());
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ControlIcon {
    Play,
    Pause,
    NextTrack,
    Volume { level: f32, muted: bool },
    SettingsGear,
    OpenFile,
    Fullscreen { is_fullscreen: bool },
}

impl eframe::App for NixoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.status = self.backend.status();
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
                    // Preserva proporção do vídeo sobre fundo preto OLED (letterbox/pillarbox)
                    let tex_size = tex.size_vec2();
                    let aspect = (tex_size.x / tex_size.y.max(1.0)).max(0.1);
                    let mut draw_w = rect.width();
                    let mut draw_h = draw_w / aspect;
                    if draw_h > rect.height() {
                        draw_h = rect.height();
                        draw_w = draw_h * aspect;
                    }

                    let video_rect =
                        Rect::from_center_size(rect.center(), Vec2::new(draw_w, draw_h));
                    ui.painter().image(
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
        }
    }
}
