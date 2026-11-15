use egui::{Align2, Color32, Context, CornerRadius, CursorIcon, Id, Margin, Sense, Stroke, vec2};
use nixo_core::{PlaybackState, PlayerCommand};

use crate::app::NixoApp;
use crate::icons::draw_replay_icon;

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

        // C. Botão central de Replay ao término do vídeo
        if self.status.state == PlaybackState::Ended
            && !self.playlist_drawer_open
            && !ctx.memory(|m| m.any_popup_open())
        {
            self.render_ended_replay_overlay(ctx, osd_alpha);
        }
    }

    /// Renderiza o botão central de Replay quando a mídia chega ao fim (`PlaybackState::Ended`).
    fn render_ended_replay_overlay(&mut self, ctx: &Context, osd_alpha: f32) {
        egui::Area::new(Id::new("ended_replay_overlay"))
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .interactable(osd_alpha > 0.15)
            .show(ctx, |ui| {
                ui.set_opacity(osd_alpha);
                let btn_size = vec2(68.0, 68.0);
                let (rect, response) = ui.allocate_exact_size(btn_size, Sense::click());
                let response = response.on_hover_cursor(CursorIcon::PointingHand);

                let hovered = response.hovered();
                let pressed = response.is_pointer_button_down_on();

                let bg = if pressed {
                    Color32::from_rgba_unmultiplied(56, 56, 62, 230)
                } else if hovered {
                    Color32::from_rgba_unmultiplied(34, 34, 38, 215)
                } else {
                    Color32::from_rgba_unmultiplied(16, 16, 18, 185)
                };
                let fg = if hovered || pressed {
                    Color32::WHITE
                } else {
                    Color32::from_gray(230)
                };

                let radius = if pressed {
                    btn_size.x * 0.47
                } else {
                    btn_size.x * 0.5
                };
                ui.painter().circle_filled(rect.center(), radius, bg);
                draw_replay_icon(
                    ui.painter(),
                    rect.center(),
                    if pressed { 1.28 } else { 1.35 },
                    fg,
                );

                crate::components::show_tooltip(ctx, &response, self.t().replay_tooltip);

                if response.clicked() {
                    self.dispatch(PlayerCommand::TogglePlayPause);
                }
            });
    }
}
