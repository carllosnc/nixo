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
                (NEUTRAL_300, NEUTRAL_950, NEUTRAL_950)
            } else if hovered {
                (NEUTRAL_50, NEUTRAL_950, NEUTRAL_950)
            } else {
                (NEUTRAL_100, NEUTRAL_950, NEUTRAL_950)
            }
        }
        ButtonVariant::Secondary => {
            if pressed {
                (NEUTRAL_600, NEUTRAL_50, NEUTRAL_50)
            } else if hovered {
                (NEUTRAL_700, NEUTRAL_50, NEUTRAL_200)
            } else {
                (NEUTRAL_800, NEUTRAL_100, NEUTRAL_400)
            }
        }
        ButtonVariant::Ghost => {
            if pressed {
                (NEUTRAL_700, NEUTRAL_50, NEUTRAL_50)
            } else if hovered {
                (NEUTRAL_800, NEUTRAL_50, NEUTRAL_200)
            } else {
                (Color32::TRANSPARENT, NEUTRAL_300, NEUTRAL_400)
            }
        }
        ButtonVariant::Danger => {
            if pressed {
                (
                    Color32::from_rgb(79, 30, 36),
                    Color32::from_rgb(254, 202, 202),
                    Color32::from_rgb(254, 202, 202),
                )
            } else if hovered {
                (
                    DANGER_HOVER,
                    Color32::from_rgb(252, 165, 165),
                    Color32::from_rgb(252, 165, 165),
                )
            } else {
                (DANGER_SURFACE, DANGER_TEXT, DANGER_TEXT)
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
