use std::time::Duration;

use egui::epaint::{Mesh, Vertex};
use egui::{
    Align2, Color32, ColorImage, Context, CornerRadius, FontId, Id, Margin, Pos2, Rect, RichText,
    Sense, Stroke, TextureOptions, Ui, vec2,
};
use nixo_core::{PlaybackState, format_duration};

use crate::app::{ACCENT_COLOR, NixoApp};

impl NixoApp {
    /// Renderiza a cápsula superior translúcida com o título da mídia (estilo YouTube/minimalista).
    pub(crate) fn render_top_pill(&self, ui: &mut Ui) {
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
    pub(crate) fn render_compact_slider(
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

            let active_t =
                ui.ctx()
                    .animate_bool_with_time(response.id.with("slider_anim"), is_active, 0.12);

            let current_rail_h = egui::lerp(rail_height..=(rail_height + 1.5), active_t);

            // 1. Trilho de fundo (rail não preenchido)
            let rail_rect = Rect::from_min_max(
                Pos2::new(rect.left(), center_y - current_rail_h * 0.5),
                Pos2::new(rect.right(), center_y + current_rail_h * 0.5),
            );
            ui.painter().rect_filled(
                rail_rect,
                CornerRadius::same(1),
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
                    CornerRadius::same(1),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                );
            }

            // 2. Trilho preenchido (progresso)
            let handle_x = rect.left() + normalized * rect.width();
            let filled_rect = Rect::from_min_max(
                Pos2::new(rect.left(), center_y - current_rail_h * 0.5),
                Pos2::new(handle_x, center_y + current_rail_h * 0.5),
            );
            ui.painter()
                .rect_filled(filled_rect, CornerRadius::same(1), ACCENT_COLOR);

            // 3. Círculo do indicador: menor, exatamente na cor da barra (ACCENT_COLOR) e sem borda
            let radius = egui::lerp(handle_radius..=(handle_radius + 1.8), active_t);
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
    pub(crate) fn render_seek_preview_tooltip(
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
    pub(crate) fn render_player_tooltip(
        ctx: &Context,
        id: Id,
        anchor_rect: Rect,
        text: &str,
        alpha: f32,
    ) {
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
}

pub(crate) fn vertical_gradient_mesh(y0: f32, y1: f32, w: f32, c0: Color32, c1: Color32) -> Mesh {
    Mesh {
        indices: vec![0, 1, 2, 2, 3, 0],
        vertices: vec![
            Vertex {
                pos: Pos2::new(0.0, y0),
                uv: Pos2::ZERO,
                color: c0,
            },
            Vertex {
                pos: Pos2::new(w, y0),
                uv: Pos2::ZERO,
                color: c0,
            },
            Vertex {
                pos: Pos2::new(w, y1),
                uv: Pos2::ZERO,
                color: c1,
            },
            Vertex {
                pos: Pos2::new(0.0, y1),
                uv: Pos2::ZERO,
                color: c1,
            },
        ],
        ..Default::default()
    }
}
