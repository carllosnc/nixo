use egui::{Align2, Color32, Context, CornerRadius, Id, Margin, Stroke, vec2};
use nixo_core::PlaybackState;

use crate::app::NixoApp;

impl NixoApp {
    /// Renderiza os elementos flutuantes do OSD (topo e barra inferior) com suporte a transparência e opacidade.
    pub(crate) fn render_floating_osd(&mut self, ctx: &Context, osd_alpha: f32) {
        if osd_alpha <= 0.005 || self.status.state == PlaybackState::Stopped {
            return;
        }

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

        // B. Barra Inferior Completa: estende-se de ponta a ponta integrada ao gradiente inferior
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
}
