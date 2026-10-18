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

    let font = font_body();
    let icon_font = if has_text {
        lucide::font_id(16.5)
    } else {
        lucide::font_id(18.0)
    };

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

    // Transição suave de hover e press usando animação temporal (150ms estilo CSS transition)
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("btn_hover"), hovered, 0.15);
    let press_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("btn_press"), pressed, 0.08);

    let (bg_color, text_color, icon_color) = match variant {
        ButtonVariant::Primary => {
            let bg = lerp_color(NEUTRAL_100, NEUTRAL_50, hover_t);
            let bg = lerp_color(bg, NEUTRAL_300, press_t);
            (bg, NEUTRAL_950, NEUTRAL_950)
        }
        ButtonVariant::Secondary => {
            let bg = lerp_color(NEUTRAL_800, NEUTRAL_700, hover_t);
            let bg = lerp_color(bg, NEUTRAL_600, press_t);
            let tc = lerp_color(NEUTRAL_100, NEUTRAL_50, hover_t);
            let ic = lerp_color(NEUTRAL_400, NEUTRAL_200, hover_t);
            (bg, tc, ic)
        }
        ButtonVariant::Ghost => {
            let bg = lerp_color(Color32::TRANSPARENT, NEUTRAL_800, hover_t);
            let bg = lerp_color(bg, NEUTRAL_700, press_t);
            let tc = lerp_color(NEUTRAL_300, NEUTRAL_50, hover_t);
            let ic = lerp_color(NEUTRAL_400, NEUTRAL_200, hover_t);
            (bg, tc, ic)
        }
        ButtonVariant::Danger => {
            let bg = lerp_color(DANGER_SURFACE, DANGER_HOVER, hover_t);
            let bg = lerp_color(bg, Color32::from_rgb(79, 30, 36), press_t);
            let fg = lerp_color(DANGER_TEXT, Color32::from_rgb(252, 165, 165), hover_t);
            (bg, fg, fg)
        }
    };

    // 1. Fundo Full Rounded (Pill / Circle)
    ui.painter().rect_filled(rect, RADIUS_FULL, bg_color);

    // 2. Conteúdo perfeitamente centralizado
    if has_icon && !has_text {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            icon_str,
            icon_font,
            icon_color,
        );
    } else {
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

/// Botão de ícone circular padronizado com hover suave e tooltip.
#[inline]
pub fn icon_button(ui: &mut Ui, variant: ButtonVariant, icon: &str, tooltip: &str) -> Response {
    render_button(ui, variant, Some(icon), "", 38.0).on_hover_text(tooltip)
}

/// Atalho para botão de ícone primário (branco com contraste alto).
#[inline]
pub fn icon_primary(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    icon_button(ui, ButtonVariant::Primary, icon, tooltip)
}

/// Atalho para botão de ícone secundário (neutro dark).
#[inline]
pub fn icon_secondary(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    icon_button(ui, ButtonVariant::Secondary, icon, tooltip)
}

/// Atalho para botão de ícone ghost transparente.
#[inline]
pub fn icon_ghost(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    icon_button(ui, ButtonVariant::Ghost, icon, tooltip)
}
