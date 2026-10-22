use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Margin, Pos2, Rect, RichText, ScrollArea,
    Sense, Stroke, Ui, Vec2, vec2,
};
use nixo_core::{PlaybackState, RepeatMode};

use crate::app::{ACCENT_COLOR, NixoApp};
use crate::lucide;
use crate::theme::font_bold;

const DRAWER_WIDTH: f32 = 360.0;
const ANIM_DURATION: f64 = 0.26;

/// Estado de animação de deslizamento lateral do drawer com tempo real e interpolação Quartic Ease-Out.
#[derive(Clone, Copy)]
struct DrawerAnimState {
    from_progress: f32,
    target_progress: f32,
    start_time: f64,
}

impl DrawerAnimState {
    fn eval(self, now: f64, duration: f64) -> (f32, bool) {
        if duration <= 0.0 || (self.target_progress - self.from_progress).abs() <= 0.0001 {
            return (self.target_progress, false);
        }
        let t = ((now - self.start_time) / duration).clamp(0.0, 1.0) as f32;
        // Curva Quartic Ease-Out: arranque instantâneo com desaceleração sedosa
        let inv = 1.0 - t;
        let eased = 1.0 - inv * inv * inv * inv;
        let p = self.from_progress + (self.target_progress - self.from_progress) * eased;
        (p, t < 1.0)
    }
}

impl NixoApp {
    /// Renderiza a gaveta lateral (drawer) de playlist sobre o vídeo com animação deslizante suave da direita.
    pub(crate) fn render_playlist_drawer(&mut self, ctx: &Context) {
        if self.status.state == PlaybackState::Stopped {
            self.playlist_drawer_open = false;
            return;
        }

        let anim_id = Id::new("playlist_drawer_slide_anim_state");
        let target = if self.playlist_drawer_open { 1.0 } else { 0.0 };
        let now = ctx.input(|i| i.time);

        let (progress, animating) = ctx.data_mut(|d| {
            let state = d.get_temp_mut_or_insert_with(anim_id, || DrawerAnimState {
                from_progress: 0.0,
                target_progress: target,
                start_time: now,
            });

            if (state.target_progress - target).abs() > 0.001 {
                let (cur, _) = state.eval(now, ANIM_DURATION);
                state.from_progress = cur;
                state.target_progress = target;
                state.start_time = now;
            }

            state.eval(now, ANIM_DURATION)
        });

        // Garante repintura fluida a 60 FPS durante a abertura e o fechamento
        if animating {
            ctx.request_repaint();
        }

        if progress <= 0.001 && !self.playlist_drawer_open {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let drawer_w = DRAWER_WIDTH.min(screen_rect.width() * 0.85);
        let offset_x = (1.0 - progress) * (drawer_w + 24.0);

        // 1. Backdrop translúcido (clique fora fecha o drawer)
        let backdrop_alpha = (progress * 125.0).round() as u8;
        if backdrop_alpha > 0 {
            let backdrop_resp = egui::Area::new(Id::new("playlist_drawer_backdrop"))
                .order(egui::Order::Middle)
                .constrain(false)
                .fade_in(false)
                .fixed_pos(screen_rect.min)
                .interactable(self.playlist_drawer_open)
                .show(ctx, |ui| {
                    let (rect, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(0, 0, 0, backdrop_alpha),
                    );
                    resp
                })
                .inner;

            if backdrop_resp.clicked() {
                self.playlist_drawer_open = false;
            }
        }

        // 2. Painel deslizante do Drawer
        // Importante: .constrain(false) permite que o container deslize a partir de fora da tela
        // sem ser travado/clampado nas bordas da janela pelo egui.
        let drawer_pos = Pos2::new(screen_rect.right() - drawer_w + offset_x, screen_rect.top());

        let mut item_to_play = None;

        egui::Area::new(Id::new("playlist_drawer_panel"))
            .order(egui::Order::Foreground)
            .constrain(false)
            .fade_in(false)
            .fixed_pos(drawer_pos)
            .interactable(self.playlist_drawer_open)
            .show(ctx, |ui| {
                ui.set_width(drawer_w);
                ui.set_height(screen_rect.height());

                // Sem bordas duras: elevação premium através de preenchimento OLED escuro e sombra suave à esquerda
                let shadow_alpha = (progress * 165.0).round() as u8;
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(13, 14, 18, 246))
                    .stroke(Stroke::NONE)
                    .shadow(egui::Shadow {
                        offset: [-18, 0],
                        blur: 32,
                        spread: 0,
                        color: Color32::from_black_alpha(shadow_alpha),
                    })
                    .corner_radius(CornerRadius {
                        nw: 16,
                        sw: 16,
                        ne: 0,
                        se: 0,
                    })
                    .inner_margin(Margin {
                        left: 18,
                        right: 18,
                        top: 20,
                        bottom: 18,
                    })
                    .show(ui, |ui| {
                        self.render_drawer_header(ui);
                        ui.add_space(14.0);

                        // Linha divisória ultra sutil e moderna
                        let (sep_rect, _) =
                            ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                        ui.painter().rect_filled(
                            sep_rect,
                            CornerRadius::ZERO,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                        );
                        ui.add_space(10.0);

                        item_to_play = self.render_drawer_list(ui);
                    });
            });

        if let Some(idx) = item_to_play {
            self.play_playlist_index(idx);
        }
    }

    /// Renderiza o cabeçalho superior do drawer com título, chip de contagem e controles refinados.
    fn render_drawer_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Ícone de playlist dentro de um container sutil
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
            ui.painter().rect_filled(
                icon_rect,
                CornerRadius::same(7),
                Color32::from_rgba_unmultiplied(55, 167, 33, 26),
            );
            ui.painter().text(
                icon_rect.center(),
                Align2::CENTER_CENTER,
                lucide::LIST_VIDEO,
                lucide::font_id(16.0),
                ACCENT_COLOR,
            );

            // Título
            ui.label(
                RichText::new(self.t().playlist_drawer_title)
                    .font(FontId::proportional(16.0))
                    .color(Color32::WHITE)
                    .strong(),
            );

            // Chip elegante com contagem de faixas
            let count_text = self.playlist.len().to_string();
            let badge_font = FontId::monospace(11.5);
            let galley =
                ui.painter()
                    .layout_no_wrap(count_text, badge_font, Color32::from_gray(190));
            let badge_size = vec2(galley.size().x + 12.0, 20.0);
            let (badge_rect, _) = ui.allocate_exact_size(badge_size, Sense::hover());
            ui.painter().rect_filled(
                badge_rect,
                CornerRadius::same(99),
                Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            );
            ui.painter().galley(
                Pos2::new(
                    badge_rect.center().x - galley.size().x * 0.5,
                    badge_rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                Color32::from_gray(190),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Botão de fechar (X) estilo ghost com hover iluminado
                let (close_rect, close_resp) =
                    ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
                if close_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(
                        close_rect,
                        CornerRadius::same(7),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 20),
                    );
                }
                let close_color = if close_resp.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(180)
                };
                ui.painter().text(
                    close_rect.center(),
                    Align2::CENTER_CENTER,
                    lucide::X,
                    lucide::font_id(14.0),
                    close_color,
                );
                if close_resp.clicked() {
                    self.playlist_drawer_open = false;
                }

                ui.add_space(4.0);

                // Alternador de repetição compacto
                let (rep_rect, rep_resp) =
                    ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
                let rep_active = self.playlist.repeat_mode() != RepeatMode::Off;
                if rep_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(
                        rep_rect,
                        CornerRadius::same(7),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 20),
                    );
                } else if rep_active {
                    ui.painter().rect_filled(
                        rep_rect,
                        CornerRadius::same(7),
                        Color32::from_rgba_unmultiplied(55, 167, 33, 26),
                    );
                }

                let rep_icon = match self.playlist.repeat_mode() {
                    RepeatMode::One => lucide::REPEAT_1,
                    _ => lucide::REPEAT,
                };
                let rep_color = if rep_active {
                    ACCENT_COLOR
                } else if rep_resp.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(180)
                };
                ui.painter().text(
                    rep_rect.center(),
                    Align2::CENTER_CENTER,
                    rep_icon,
                    lucide::font_id(14.0),
                    rep_color,
                );
                if rep_resp.clicked() {
                    self.cycle_repeat_mode();
                }
            });
        });
    }

    /// Renderiza a lista de reprodução com rolagem, estados visuais modernos e seleção de faixa.
    fn render_drawer_list(&mut self, ui: &mut Ui) -> Option<usize> {
        let mut selected_idx = None;
        let cur_active_idx = self.playlist.current_index().unwrap_or(0);
        let items_len = self.playlist.len();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                for idx in 0..items_len {
                    let is_current = idx == cur_active_idx;
                    let path = &self.playlist.items()[idx];
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Vídeo");

                    let item_h = 42.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), item_h), Sense::click());

                    let hovered = resp.hovered();
                    if hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let painter = ui.painter();

                    // Fundo e destaque do item (sem bordas duras)
                    if is_current {
                        // Preenchimento translúcido com o verde de acento do Nixo
                        painter.rect_filled(
                            rect,
                            CornerRadius::same(8),
                            Color32::from_rgba_unmultiplied(34, 197, 94, 28),
                        );
                        // Indicador vertical esquerdo moderno em pílula
                        let indicator_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 2.5, rect.center().y - 10.0),
                            vec2(3.5, 20.0),
                        );
                        painter.rect_filled(indicator_rect, CornerRadius::same(2), ACCENT_COLOR);
                    } else if hovered {
                        painter.rect_filled(
                            rect,
                            CornerRadius::same(8),
                            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                        );
                    }

                    // Indicador / Número ordinal
                    let badge_x = rect.left() + 12.0;
                    let text_y = rect.center().y;

                    if is_current {
                        painter.text(
                            Pos2::new(badge_x + 6.0, text_y),
                            Align2::CENTER_CENTER,
                            lucide::PLAY,
                            lucide::font_id(13.0),
                            ACCENT_COLOR,
                        );
                    } else {
                        let num_color = if hovered {
                            Color32::from_gray(210)
                        } else {
                            Color32::from_gray(120)
                        };
                        painter.text(
                            Pos2::new(badge_x + 6.0, text_y),
                            Align2::CENTER_CENTER,
                            format!("{:02}", idx + 1),
                            FontId::monospace(11.5),
                            num_color,
                        );
                    }

                    // Tipografia do título do arquivo
                    let title_font = if is_current {
                        font_bold(13.5)
                    } else {
                        FontId::proportional(13.5)
                    };

                    let title_color = if is_current || hovered {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(185)
                    };

                    let title_rect = Rect::from_min_max(
                        Pos2::new(rect.left() + 34.0, rect.top() + 4.0),
                        Pos2::new(rect.right() - 8.0, rect.bottom() - 4.0),
                    );

                    let mut title_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(title_rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );

                    let label_resp = title_ui.add(
                        egui::Label::new(
                            RichText::new(file_name).font(title_font).color(title_color),
                        )
                        .truncate(),
                    );

                    if resp.hovered() || label_resp.hovered() {
                        let tooltip = if is_current {
                            format!("{}: {file_name}", self.t().playlist_item_playing)
                        } else {
                            file_name.to_string()
                        };
                        resp.clone().on_hover_text(tooltip);
                    }

                    if resp.clicked() {
                        selected_idx = Some(idx);
                    }
                }
            });

        selected_idx
    }
}
