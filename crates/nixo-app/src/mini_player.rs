//! Gerenciamento e interface do Modo Mini-Player (Picture-in-Picture) do Nixo.

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, Id, Pos2, Rect, Sense, Stroke, StrokeKind,
    ViewportCommand, vec2,
};
use nixo_core::{PlaybackState, PlayerCommand};

use crate::app::{ACCENT_COLOR, NixoApp};
use crate::components::show_tooltip;
use crate::components::tokens::{NEUTRAL_300, TEXT_PRIMARY};
use crate::lucide;

impl NixoApp {
    /// Alterna entre a janela padrão e o modo flutuante Mini-Player (Picture-in-Picture).
    pub(crate) fn toggle_mini_player(&mut self, ctx: &egui::Context) {
        if self.is_mini_player {
            self.is_mini_player = false;
            self.always_on_top = self.saved_always_on_top;
            let level = if self.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));

            let size = self
                .saved_window_size
                .unwrap_or_else(|| vec2(1280.0, 720.0));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
            Self::apply_windows_titlebar_color();
        } else {
            if self.is_fullscreen {
                self.toggle_fullscreen(ctx);
            }
            self.saved_window_size = Some(ctx.screen_rect().size());
            self.saved_always_on_top = self.always_on_top;
            self.is_mini_player = true;
            self.always_on_top = true;
            self.playlist_drawer_open = false;

            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));

            let aspect = self.active_video_aspect();
            let target_w = 384.0_f32;
            let target_h = (target_w / aspect).round().clamp(180.0, 320.0);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(target_w, target_h)));
        }
    }

    /// Renderiza os controles flutuantes ultra-minimalistas sobre o vídeo em modo Mini-Player.
    pub(crate) fn render_mini_player_overlay(&mut self, ctx: &egui::Context) {
        if !self.is_mini_player || self.status.state == PlaybackState::Stopped {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let is_hovered = pointer_pos.is_some();

        let alpha =
            ctx.animate_bool_with_time(Id::new("mini_player_hover_alpha"), is_hovered, 0.18);

        egui::Area::new(Id::new("mini_player_drag_and_controls"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(screen_rect.width());
                ui.set_height(screen_rect.height());

                // Aloca toda a tela para permitir arraste da janela (window drag) e duplo clique para restaurar
                let base_resp = ui.allocate_rect(screen_rect, Sense::click_and_drag());
                if base_resp.drag_started() {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                }
                if base_resp.double_clicked() {
                    self.toggle_mini_player(ctx);
                    return;
                }

                // Linha de progresso no rodapé da janela (sempre visível discretamente)
                let dur = self.status.duration.as_secs_f32();
                if dur > 0.0 {
                    let progress = (self.status.position.as_secs_f32() / dur).clamp(0.0, 1.0);
                    let bar_y = screen_rect.bottom() - 3.0;
                    let bar_w = screen_rect.width() * progress;
                    ui.painter().rect_filled(
                        Rect::from_min_max(
                            Pos2::new(screen_rect.left(), bar_y),
                            Pos2::new(screen_rect.right(), screen_rect.bottom()),
                        ),
                        CornerRadius::ZERO,
                        Color32::from_black_alpha(100),
                    );
                    ui.painter().rect_filled(
                        Rect::from_min_max(
                            Pos2::new(screen_rect.left(), bar_y),
                            Pos2::new(screen_rect.left() + bar_w, screen_rect.bottom()),
                        ),
                        CornerRadius::ZERO,
                        ACCENT_COLOR,
                    );
                }

                if alpha < 0.02 {
                    return;
                }

                // Scrim escuro suave sobre o vídeo ao aproximar o cursor
                let scrim_alpha = (140.0 * alpha) as u8;
                ui.painter().rect_filled(
                    screen_rect,
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(scrim_alpha),
                );

                // Botão de Restaurar Janela (canto superior esquerdo)
                let restore_rect = Rect::from_min_size(
                    Pos2::new(screen_rect.left() + 10.0, screen_rect.top() + 10.0),
                    vec2(32.0, 32.0),
                );
                let restore_resp = ui
                    .allocate_rect(restore_rect, Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                show_tooltip(ctx, &restore_resp, self.t().mini_player_restore);

                let restore_bg = if restore_resp.is_pointer_button_down_on() {
                    Color32::from_white_alpha(50)
                } else if restore_resp.hovered() {
                    Color32::from_white_alpha(35)
                } else {
                    Color32::from_black_alpha(120)
                };
                ui.painter()
                    .rect_filled(restore_rect, CornerRadius::same(99), restore_bg);
                ui.painter().text(
                    restore_rect.center(),
                    Align2::CENTER_CENTER,
                    lucide::MAXIMIZE,
                    egui::FontId::new(14.0, egui::FontFamily::Name("lucide".into())),
                    TEXT_PRIMARY,
                );
                if restore_resp.clicked() {
                    self.toggle_mini_player(ctx);
                }

                // Botão de Voltar para Home / Fechar vídeo (canto superior direito)
                let close_rect = Rect::from_min_size(
                    Pos2::new(screen_rect.right() - 42.0, screen_rect.top() + 10.0),
                    vec2(32.0, 32.0),
                );
                let close_resp = ui
                    .allocate_rect(close_rect, Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                show_tooltip(ctx, &close_resp, self.t().home_tooltip);

                let close_bg = if close_resp.is_pointer_button_down_on() {
                    Color32::from_white_alpha(50)
                } else if close_resp.hovered() {
                    Color32::from_white_alpha(35)
                } else {
                    Color32::from_black_alpha(120)
                };
                ui.painter()
                    .rect_filled(close_rect, CornerRadius::same(99), close_bg);
                ui.painter().text(
                    close_rect.center(),
                    Align2::CENTER_CENTER,
                    lucide::X,
                    egui::FontId::new(15.0, egui::FontFamily::Name("lucide".into())),
                    TEXT_PRIMARY,
                );
                if close_resp.clicked() {
                    self.toggle_mini_player(ctx);
                    self.go_home();
                }

                // Botão Play/Pause Central
                let play_rect = Rect::from_center_size(screen_rect.center(), vec2(46.0, 46.0));
                let play_resp = ui
                    .allocate_rect(play_rect, Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);

                let is_playing = self.status.state == PlaybackState::Playing;
                let play_tooltip = if is_playing {
                    self.t().pause_tooltip
                } else {
                    self.t().play_tooltip
                };
                show_tooltip(ctx, &play_resp, play_tooltip);

                let play_bg = if play_resp.is_pointer_button_down_on() {
                    Color32::from_rgb(45, 142, 26)
                } else if play_resp.hovered() {
                    Color32::from_rgb(52, 175, 30)
                } else {
                    Color32::from_black_alpha(150)
                };
                ui.painter().rect(
                    play_rect,
                    CornerRadius::same(99),
                    play_bg,
                    Stroke::new(1.0_f32, Color32::from_white_alpha(30)),
                    StrokeKind::Inside,
                );

                let play_glyph = if is_playing {
                    lucide::PAUSE
                } else {
                    lucide::PLAY
                };
                let glyph_color = if play_resp.hovered() {
                    Color32::WHITE
                } else {
                    NEUTRAL_300
                };
                ui.painter().text(
                    play_rect.center(),
                    Align2::CENTER_CENTER,
                    play_glyph,
                    egui::FontId::new(18.0, egui::FontFamily::Name("lucide".into())),
                    glyph_color,
                );

                if play_resp.clicked() {
                    self.dispatch(PlayerCommand::TogglePlayPause);
                }
            });
    }

    pub(crate) fn active_video_aspect(&self) -> f32 {
        self.status
            .aspect_ratio
            .ratio_f64()
            .map(|r| r as f32)
            .unwrap_or_else(|| {
                if let Some(tex) = &self.video_texture {
                    let sz = tex.size_vec2();
                    (sz.x / sz.y.max(1.0)).max(0.1)
                } else {
                    16.0 / 9.0
                }
            })
            .clamp(0.2, 5.0)
    }

    pub(crate) fn snap_window_to_aspect(&mut self, ctx: &egui::Context) {
        if self.is_fullscreen {
            return;
        }
        let current_size = ctx.screen_rect().size();
        if current_size.x <= 100.0 || current_size.y <= 100.0 {
            return;
        }
        let aspect = self.active_video_aspect();
        let target_h = (current_size.x / aspect).round().max(180.0);
        let target_size = egui::Vec2::new(current_size.x, target_h);
        self.last_window_size = target_size;
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(target_size));
    }

    pub(crate) fn toggle_aspect_ratio_fit(&mut self, ctx: &egui::Context) {
        if self.status.state == PlaybackState::Stopped {
            return;
        }
        self.aspect_ratio_fit = !self.aspect_ratio_fit;
        self.saved_aspect_ratio_fit = self.aspect_ratio_fit;
        if self.aspect_ratio_fit {
            self.snap_window_to_aspect(ctx);
        }
    }
}
