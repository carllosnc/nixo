use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Margin, Pos2, Rect, ScrollArea, Sense,
    Stroke, Ui, Vec2, vec2,
};
use nixo_core::{PlaybackState, RepeatMode};

use crate::app::{ACCENT_COLOR, NixoApp};
use crate::components::paint_truncated_text;
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

        // 1. Backdrop transparente (clique fora fecha o drawer sem escurecer o vídeo)
        if self.playlist_drawer_open {
            let backdrop_resp = egui::Area::new(Id::new("playlist_drawer_backdrop"))
                .order(egui::Order::Middle)
                .constrain(false)
                .fade_in(false)
                .fixed_pos(screen_rect.min)
                .interactable(true)
                .show(ctx, |ui| {
                    let (_, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                    resp
                })
                .inner;

            if backdrop_resp.clicked() {
                self.playlist_drawer_open = false;
                self.playlist_keyboard_nav_active = false;
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

                // Sem sombreamento: preenchimento escuro limpo com bordas retas (sem radius)
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(13, 14, 18, 246))
                    .stroke(Stroke::NONE)
                    .shadow(egui::Shadow::NONE)
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(Margin::ZERO)
                    .show(ui, |ui| {
                        // Cabeçalho com padding interno
                        egui::Frame::NONE
                            .inner_margin(Margin {
                                left: 18,
                                right: 18,
                                top: 20,
                                bottom: 0,
                            })
                            .show(ui, |ui| {
                                self.render_drawer_header(ui);
                                ui.add_space(14.0);

                                // Linha divisória ultra sutil e moderna
                                let (sep_rect, _) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 1.0),
                                    Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    sep_rect,
                                    CornerRadius::ZERO,
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 6),
                                );
                            });

                        ui.add_space(6.0);

                        item_to_play = self.render_drawer_list(ui);
                    });
            });

        if let Some(idx) = item_to_play {
            self.play_playlist_index(idx);
        }
    }

    /// Renderiza o cabeçalho superior do drawer com título, chip de contagem e controles perfeitamente alinhados no mesmo centro vertical.
    fn render_drawer_header(&mut self, ui: &mut Ui) {
        let header_h = 28.0;
        let (header_rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), header_h), Sense::hover());
        let center_y = header_rect.center().y;
        let painter = ui.painter();

        // 1. Botão de Fechar (extrema direita, alinhado pelo center_y)
        let close_rect = Rect::from_center_size(
            Pos2::new(header_rect.right() - 14.0, center_y),
            Vec2::splat(28.0),
        );
        let close_resp = ui.interact(close_rect, ui.id().with("close_drawer_btn"), Sense::click());
        if close_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            painter.rect_filled(
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
        painter.text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            lucide::X,
            lucide::font_id(14.0),
            close_color,
        );
        crate::components::show_tooltip(ui.ctx(), &close_resp, self.t().playlist_close_tooltip);
        if close_resp.clicked() {
            self.playlist_drawer_open = false;
            self.playlist_keyboard_nav_active = false;
        }

        // 2. Alternador de repetição (à esquerda do fechar, alinhado pelo center_y)
        let rep_rect = Rect::from_center_size(
            Pos2::new(close_rect.left() - 4.0 - 14.0, center_y),
            Vec2::splat(28.0),
        );
        let rep_resp = ui.interact(rep_rect, ui.id().with("rep_drawer_btn"), Sense::click());
        let rep_active = self.playlist.repeat_mode() != RepeatMode::Off;
        if rep_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            painter.rect_filled(
                rep_rect,
                CornerRadius::same(7),
                Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            );
        } else if rep_active {
            painter.rect_filled(
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
        painter.text(
            rep_rect.center(),
            Align2::CENTER_CENTER,
            rep_icon,
            lucide::font_id(14.0),
            rep_color,
        );
        let rep_tooltip = match self.playlist.repeat_mode() {
            RepeatMode::Off => self.t().repeat_off_tooltip,
            RepeatMode::All => self.t().repeat_all_tooltip,
            RepeatMode::One => self.t().repeat_one_tooltip,
        };
        crate::components::show_tooltip(ui.ctx(), &rep_resp, rep_tooltip);
        if rep_resp.clicked() {
            self.cycle_repeat_mode();
        }

        // 3. Ícone de Playlist (esquerda, perfeitamente centralizado em center_y)
        let mut cur_x = header_rect.left();
        let icon_font = lucide::font_id(18.0);
        let icon_galley =
            painter.layout_no_wrap(lucide::LIST_VIDEO.to_string(), icon_font, ACCENT_COLOR);
        let icon_w = icon_galley.size().x;
        painter.galley(
            Pos2::new(cur_x, center_y - icon_galley.size().y * 0.5),
            icon_galley,
            ACCENT_COLOR,
        );
        cur_x += icon_w + 8.0;

        // 4. Título "Playlist" (perfeitamente centralizado em center_y)
        let title_font = font_bold(16.0);
        let title_galley = painter.layout_no_wrap(
            self.t().playlist_drawer_title.to_string(),
            title_font,
            Color32::WHITE,
        );
        let title_w = title_galley.size().x;
        painter.galley(
            Pos2::new(cur_x, center_y - title_galley.size().y * 0.5),
            title_galley,
            Color32::WHITE,
        );
        cur_x += title_w + 8.0;

        // 5. Badge elegante com contagem de faixas (perfeitamente centralizado em center_y)
        let count_text = self.playlist.len().to_string();
        let badge_font = FontId::monospace(11.5);
        let badge_galley = painter.layout_no_wrap(count_text, badge_font, Color32::from_gray(190));
        let badge_h = 20.0;
        let badge_w = badge_galley.size().x + 12.0;
        let badge_rect = Rect::from_center_size(
            Pos2::new(cur_x + badge_w * 0.5, center_y),
            vec2(badge_w, badge_h),
        );
        painter.rect_filled(
            badge_rect,
            CornerRadius::same(99),
            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
        );
        painter.galley(
            Pos2::new(
                badge_rect.center().x - badge_galley.size().x * 0.5,
                center_y - badge_galley.size().y * 0.5,
            ),
            badge_galley,
            Color32::from_gray(190),
        );
    }

    /// Renderiza a lista de reprodução virtualizada com alta performance, auto-scroll e seleção de faixa.
    fn render_drawer_list(&mut self, ui: &mut Ui) -> Option<usize> {
        let mut selected_idx = None;
        let cur_active_idx = self.playlist.current_index().unwrap_or(0);
        let focused_idx = self.playlist_focused_index;
        let kb_active = self.playlist_keyboard_nav_active;
        let items_len = self.playlist.len();

        if items_len == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(
                    egui::RichText::new(lucide::LIST_VIDEO)
                        .font(lucide::font_id(32.0))
                        .color(Color32::from_gray(80)),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Nenhuma mídia na fila")
                        .font(FontId::proportional(13.0))
                        .color(Color32::from_gray(120)),
                );
            });
            return None;
        }

        let item_h = 42.0;
        let spacing_y = 4.0;
        let row_total_h = item_h + spacing_y;

        let mut scroll = ScrollArea::vertical()
            .id_salt("playlist_drawer_scroll_area")
            .auto_shrink([false, false]);

        // Auto-scroll para a faixa atual ao abrir ou para a faixa focada por teclado
        if let Some(target_idx) = self.playlist_scroll_to_index.take() {
            let total_h = (items_len as f32 * row_total_h) - spacing_y;
            let max_scroll = (total_h - ui.available_height()).max(0.0);
            let target_y =
                (target_idx as f32 * row_total_h) - (ui.available_height() * 0.5) + (item_h * 0.5);
            scroll = scroll.vertical_scroll_offset(target_y.clamp(0.0, max_scroll));
        }

        ui.spacing_mut().item_spacing.y = spacing_y;

        scroll.show_rows(ui, item_h, items_len, |ui, row_range| {
            for idx in row_range {
                let is_current = idx == cur_active_idx;
                let is_focused = idx == focused_idx && kb_active;
                let path = &self.playlist.items()[idx];
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Vídeo");

                let (full_rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), item_h), Sense::hover());

                // Margens internas: 18px à esquerda e 10px à direita (scrollbar na borda extrema)
                let rect = Rect::from_min_max(
                    Pos2::new(full_rect.left() + 18.0, full_rect.top()),
                    Pos2::new(full_rect.right() - 10.0, full_rect.bottom()),
                );

                let resp = ui.interact(rect, ui.id().with(idx), Sense::click());

                let hovered = resp.hovered();
                if hovered {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let painter = ui.painter();

                // Fundo e destaque do item (formato pílula / full rounded border)
                let pill_radius = CornerRadius::same(99);
                if is_current {
                    let fill_color = if is_focused {
                        Color32::from_rgba_unmultiplied(34, 197, 94, 44)
                    } else {
                        Color32::from_rgba_unmultiplied(34, 197, 94, 28)
                    };
                    painter.rect_filled(rect, pill_radius, fill_color);
                    if is_focused {
                        painter.rect_stroke(
                            rect,
                            pill_radius,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(34, 197, 94, 130)),
                            egui::StrokeKind::Inside,
                        );
                    }
                } else if is_focused {
                    painter.rect_filled(
                        rect,
                        pill_radius,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                    );
                    painter.rect_stroke(
                        rect,
                        pill_radius,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 45)),
                        egui::StrokeKind::Inside,
                    );
                } else if hovered {
                    painter.rect_filled(
                        rect,
                        pill_radius,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                    );
                }

                // Indicador / Número ordinal
                let badge_x = rect.left() + 14.0;
                let text_y = rect.center().y;

                if is_current {
                    painter.text(
                        Pos2::new(badge_x + 6.0, text_y),
                        Align2::CENTER_CENTER,
                        lucide::PLAY,
                        lucide::font_id(13.5),
                        ACCENT_COLOR,
                    );
                } else {
                    let num_color = if is_focused || hovered {
                        Color32::from_gray(210)
                    } else {
                        Color32::from_gray(120)
                    };
                    painter.text(
                        Pos2::new(badge_x + 6.0, text_y),
                        Align2::CENTER_CENTER,
                        format!("{:02}", idx + 1),
                        FontId::monospace(13.0),
                        num_color,
                    );
                }

                // Tipografia do título do arquivo (sem seleção de texto e sem tooltip)
                let title_font = if is_current {
                    font_bold(13.5)
                } else {
                    FontId::proportional(13.5)
                };

                let title_color = if is_current || is_focused || hovered {
                    Color32::WHITE
                } else {
                    Color32::from_gray(185)
                };

                let text_x = rect.left() + 40.0;
                let max_w = (rect.right() - 14.0 - text_x).max(20.0);
                paint_truncated_text(
                    ui,
                    Pos2::new(text_x, rect.center().y),
                    Align2::LEFT_CENTER,
                    file_name,
                    title_font,
                    title_color,
                    max_w,
                );

                if resp.clicked() {
                    self.playlist_focused_index = idx;
                    self.playlist_keyboard_nav_active = false;
                    selected_idx = Some(idx);
                }
            }
        });

        selected_idx
    }
}
