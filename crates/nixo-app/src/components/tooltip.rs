use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Margin, Pos2, Rect, Response, RichText,
    Stroke,
};

use crate::theme::font_bold;

/// Exibe o tooltip padronizado para uma `Response` de UI com animação suave de fade e posicionamento automático.
pub fn show_tooltip(ctx: &Context, response: &Response, text: &str) {
    if text.is_empty() {
        return;
    }
    let tt_id = response.id.with("nixo_pill_tooltip");
    let is_hovered = response.hovered();
    let alpha = ctx.animate_bool_with_time(tt_id, is_hovered, 0.12);
    render_tooltip(ctx, tt_id, response.rect, text, alpha);
}

/// Renderiza o tooltip flutuante full-rounded com tonalidade escura, alto contraste e posicionamento adaptativo.
pub fn render_tooltip(ctx: &Context, id: Id, anchor_rect: Rect, text: &str, alpha: f32) {
    if alpha <= 0.05 || text.is_empty() {
        return;
    }

    let font_id = FontId::proportional(12.0);
    let text_w = ctx.fonts(|f| {
        f.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE)
            .size()
            .x
    });
    let pill_w = text_w + 26.0;
    let half_w = pill_w * 0.5;

    let screen = ctx.screen_rect();
    let clamped_x = anchor_rect
        .center()
        .x
        .clamp(screen.left() + half_w + 8.0, screen.right() - half_w - 8.0);

    // Inversão adaptativa caso o elemento esteja muito próximo do topo da janela
    let show_below = anchor_rect.top() - 36.0 < screen.top();
    let (pivot, anchor_pos) = if show_below {
        (
            Align2::CENTER_TOP,
            Pos2::new(
                clamped_x,
                (anchor_rect.bottom() + 8.0).min(screen.bottom() - 10.0),
            ),
        )
    } else {
        (
            Align2::CENTER_BOTTOM,
            Pos2::new(
                clamped_x,
                (anchor_rect.top() - 8.0).max(screen.top() + 10.0),
            ),
        )
    };

    let bg_color = Color32::from_rgba_unmultiplied(20, 22, 28, 248);
    let stroke_border = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    let radius = CornerRadius::same(99);

    egui::Area::new(id)
        .pivot(pivot)
        .fixed_pos(anchor_pos)
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_opacity(alpha);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

            // Container em formato de cápsula (full rounded) com tonalidade escura e elevação
            egui::Frame::NONE
                .fill(bg_color)
                .stroke(stroke_border)
                .corner_radius(radius)
                .shadow(egui::Shadow {
                    offset: [0, 4],
                    blur: 14,
                    spread: 0,
                    color: Color32::from_black_alpha(130),
                })
                .inner_margin(Margin::symmetric(11, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if let Some(pos) = text.find(" (") {
                            let title = &text[..pos];
                            let shortcut = &text[pos..];
                            ui.add(
                                egui::Label::new(
                                    RichText::new(title)
                                        .font(font_bold(12.0))
                                        .color(Color32::WHITE),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(shortcut)
                                        .font(FontId::proportional(11.5))
                                        .color(Color32::from_rgb(165, 170, 180)),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        } else if let Some(pos) = text.find(": ") {
                            let prefix = &text[..=pos];
                            let val = &text[pos + 1..];
                            ui.add(
                                egui::Label::new(
                                    RichText::new(prefix)
                                        .font(FontId::proportional(12.0))
                                        .color(Color32::from_rgb(165, 170, 180)),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(val)
                                        .font(font_bold(12.0))
                                        .color(Color32::WHITE),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        } else {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(text)
                                        .font(font_bold(12.0))
                                        .color(Color32::WHITE),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        }
                    });
                });
        });
}
