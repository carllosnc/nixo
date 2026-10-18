use egui::{Align2, CursorIcon, RichText, Sense, Stroke, Ui, vec2};

use super::tokens::*;
use crate::lucide;

/// Renderiza um estado vazio elegante e minimalista.
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
            let btn = egui::Button::new(RichText::new(act).font(font_body()).color(TEXT_PRIMARY))
                .fill(SURFACE_MED)
                .stroke(Stroke::NONE)
                .corner_radius(RADIUS_MD);

            let resp = ui
                .add_sized([180.0, 34.0], btn)
                .on_hover_cursor(CursorIcon::PointingHand);
            if resp.hovered() {
                ui.painter().rect_filled(resp.rect, RADIUS_MD, SURFACE_HIGH);
            }
            action_resp = Some(resp);
        }
    });

    action_resp
}
