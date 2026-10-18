//! Componente de Empty State espaçoso e neutro, sem caixas com bordas duras.

use egui::{Align2, RichText, Sense, Ui, vec2};

use super::tokens::*;
use crate::lucide;

/// Renderiza um estado vazio elegante e minimalista com cantos full rounded.
pub fn render_empty_state(
    ui: &mut Ui,
    icon: &str,
    title: &str,
    description: &str,
    action_text: Option<&str>,
) -> Option<egui::Response> {
    let mut action_resp = None;

    ui.add_space(28.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
        ui.painter().rect_filled(icon_rect, RADIUS_LG, SURFACE_MED);

        ui.painter().text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            icon,
            lucide::font_id(24.0),
            TEXT_SECONDARY,
        );

        ui.add_space(14.0);
        ui.label(
            RichText::new(title)
                .font(font_title())
                .strong()
                .color(TEXT_PRIMARY),
        );

        ui.add_space(4.0);
        ui.label(
            RichText::new(description)
                .font(font_body())
                .color(TEXT_SECONDARY),
        );

        if let Some(act) = action_text {
            ui.add_space(16.0);
            let resp = super::button::primary(ui, Some(lucide::PLUS), act);
            action_resp = Some(resp);
        }
    });

    action_resp
}
