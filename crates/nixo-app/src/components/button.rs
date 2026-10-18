use egui::{Align2, Color32, CornerRadius, CursorIcon, Pos2, Response, Sense, Ui, vec2};

use super::tokens::*;
use crate::lucide;

/// Variante visual do botão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Ação primária destacada com contraste neutro alto.
    Primary,
    /// Ação secundária com preenchimento neutro sutil sem bordas.
    Secondary,
    /// Botão transparente com hover sutil.
    Ghost,
    /// Ação destrutiva discreta (ex.: remover pasta).
    Danger,
}

/// Renderiza um botão padronizado com cantos full-rounded, espaçamento ótimo entre ícone e texto e hover suave.
pub fn render_button(
    ui: &mut Ui,
    variant: ButtonVariant,
    icon: Option<&str>,
    text: &str,
    height: f32,
) -> Response {
    let font = font_body();
    let icon_font = lucide::font_id(16.5);

    let has_icon = icon.is_some_and(|ic| !ic.is_empty());
    let has_text = !text.is_empty();

    let icon_str = icon.unwrap_or("");
    let icon_galley = if has_icon {
        Some(
            ui.painter()
                .layout_no_wrap(icon_str.to_string(), icon_font.clone(), Color32::WHITE),
        )
    } else {
        None
    };

    let text_galley = if has_text {
        Some(
            ui.painter()
                .layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE),
        )
    } else {
        None
    };

    let icon_w = icon_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let text_w = text_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let gap = if has_icon && has_text { 8.0 } else { 0.0 };
    let content_w = icon_w + gap + text_w;

    let pad_x = if has_text {
        18.0
    } else {
        (height - icon_w) / 2.0
    };
    let total_w = (content_w + pad_x * 2.0).max(height);

    let (rect, resp) = ui.allocate_exact_size(vec2(total_w, height), Sense::click());
    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();

    let (bg_color, text_color, icon_color) = match variant {
        ButtonVariant::Primary => {
            if pressed {
                (
                    Color32::from_rgb(205, 205, 214),
                    Color32::from_rgb(14, 15, 18),
                    Color32::from_rgb(14, 15, 18),
                )
            } else if hovered {
                (
                    Color32::from_rgb(248, 248, 252),
                    Color32::from_rgb(12, 13, 16),
                    Color32::from_rgb(12, 13, 16),
                )
            } else {
                (
                    Color32::from_rgb(226, 226, 232),
                    Color32::from_rgb(18, 19, 23),
                    Color32::from_rgb(18, 19, 23),
                )
            }
        }
        ButtonVariant::Secondary => {
            if pressed {
                (
                    Color32::from_rgb(46, 50, 64),
                    Color32::WHITE,
                    Color32::WHITE,
                )
            } else if hovered {
                (
                    Color32::from_rgb(36, 39, 50),
                    Color32::WHITE,
                    Color32::from_rgb(220, 222, 230),
                )
            } else {
                (
                    Color32::from_rgb(24, 26, 34),
                    Color32::from_rgb(220, 222, 230),
                    Color32::from_rgb(165, 168, 180),
                )
            }
        }
        ButtonVariant::Ghost => {
            if pressed {
                (
                    Color32::from_rgb(36, 39, 50),
                    Color32::WHITE,
                    Color32::WHITE,
                )
            } else if hovered {
                (
                    Color32::from_rgb(26, 28, 36),
                    Color32::WHITE,
                    Color32::from_rgb(220, 222, 230),
                )
            } else {
                (
                    Color32::TRANSPARENT,
                    Color32::from_rgb(175, 178, 190),
                    Color32::from_rgb(145, 148, 160),
                )
            }
        }
        ButtonVariant::Danger => {
            if pressed {
                (
                    Color32::from_rgb(60, 26, 30),
                    Color32::from_rgb(255, 165, 170),
                    Color32::from_rgb(255, 165, 170),
                )
            } else if hovered {
                (
                    Color32::from_rgb(48, 22, 26),
                    Color32::from_rgb(255, 148, 154),
                    Color32::from_rgb(255, 148, 154),
                )
            } else {
                (
                    Color32::from_rgb(34, 18, 21),
                    Color32::from_rgb(240, 128, 134),
                    Color32::from_rgb(240, 128, 134),
                )
            }
        }
    };

    // 1. Fundo Full Rounded (Pill)
    ui.painter().rect_filled(rect, RADIUS_FULL, bg_color);

    // 2. Conteúdo centralizado com espaçamento de 8px entre ícone e texto
    let start_x = rect.left() + (rect.width() - content_w) / 2.0;

    let mut current_x = start_x;
    if has_icon {
        ui.painter().text(
            Pos2::new(current_x, rect.center().y),
            Align2::LEFT_CENTER,
            icon_str,
            icon_font,
            icon_color,
        );
        current_x += icon_w + gap;
    }

    if has_text {
        ui.painter().text(
            Pos2::new(current_x, rect.center().y),
            Align2::LEFT_CENTER,
            text,
            font,
            text_color,
        );
    }

    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Atalho para botão de ação primária neutra (full rounded).
#[inline]
pub fn primary(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Primary, icon, text, 38.0)
}

/// Atalho para botão de ação secundária neutra (full rounded).
#[inline]
pub fn secondary(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Secondary, icon, text, 38.0)
}

/// Atalho para botão ghost transparente com hover sutil (full rounded).
#[inline]
pub fn ghost(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Ghost, icon, text, 38.0)
}
