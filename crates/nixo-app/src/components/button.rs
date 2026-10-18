use egui::{Color32, CursorIcon, Response, RichText, Stroke, Ui};

use super::tokens::*;

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

/// Renderiza um botão padronizado com suporte opcional a ícone Lucide e rótulo de texto.
pub fn render_button(
    ui: &mut Ui,
    variant: ButtonVariant,
    icon: Option<&str>,
    text: &str,
    height: f32,
) -> Response {
    let font = font_body();
    let text_content = match icon {
        Some(ic) if !text.is_empty() => format!("{ic}  {text}"),
        Some(ic) => ic.to_string(),
        None => text.to_string(),
    };

    let (bg_idle, bg_hover, bg_active, text_color) = match variant {
        ButtonVariant::Primary => (
            ACCENT_NEUTRAL_FILL,
            ACCENT_NEUTRAL_HOVER,
            Color32::from_rgb(220, 220, 226),
            ACCENT_NEUTRAL_TEXT,
        ),
        ButtonVariant::Secondary => (SURFACE_MED, SURFACE_HIGH, SURFACE_ACTIVE, TEXT_PRIMARY),
        ButtonVariant::Ghost => (
            Color32::TRANSPARENT,
            SURFACE_MED,
            SURFACE_HIGH,
            TEXT_SECONDARY,
        ),
        ButtonVariant::Danger => (DANGER_SURFACE, DANGER_HOVER, DANGER_HOVER, DANGER_TEXT),
    };

    let rich = RichText::new(text_content).font(font).color(text_color);

    let btn = egui::Button::new(rich)
        .fill(bg_idle)
        .stroke(Stroke::NONE)
        .corner_radius(RADIUS_MD)
        .min_size(egui::vec2(0.0, height));

    let resp = ui.add(btn);

    // Ajusta visualização do cursor para mãozinha apontadora
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

    // Se estiver hovered ou active, repinta a superfície correspondente
    if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(resp.rect, RADIUS_MD, bg_active);
    } else if resp.hovered() {
        ui.painter().rect_filled(resp.rect, RADIUS_MD, bg_hover);
    }

    resp
}

/// Atalho para botão de ação primária neutra.
#[inline]
pub fn primary(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Primary, icon, text, 34.0)
}

/// Atalho para botão de ação secundária neutra.
#[inline]
pub fn secondary(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Secondary, icon, text, 34.0)
}

/// Atalho para botão ghost transparente com hover sutil.
#[inline]
pub fn ghost(ui: &mut Ui, icon: Option<&str>, text: &str) -> Response {
    render_button(ui, ButtonVariant::Ghost, icon, text, 34.0)
}
