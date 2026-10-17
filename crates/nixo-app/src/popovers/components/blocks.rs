//! Blocos compostos do menu: container, avisos, estado vazio e listas de escolha.

use std::borrow::Cow;

use egui::{
    Context, FontId, Frame, Id, InnerResponse, Pos2, Rect, Response, RichText, Sense, Ui,
    UiBuilder, Widget, vec2,
};

use super::icons::LeadingIcon;
use super::rows::{MenuDivider, MenuOption, SubmenuHeader};
use crate::popovers::style::menu_style;

// ────────────────────────────── PopoverFrame ─────────────────────────────

#[derive(Clone, Copy)]
struct HeightMorphState {
    from_h: f32,
    target_h: f32,
    start_time: f64,
}

impl HeightMorphState {
    fn eval(self, now: f64, duration: f64) -> (f32, bool) {
        if (self.target_h - self.from_h).abs() <= 0.5 || duration <= 0.0 {
            return (self.target_h, false);
        }
        let t = ((now - self.start_time) / duration).clamp(0.0, 1.0) as f32;
        let eased = 1.0 - (1.0 - t).powi(3);
        let h = self.from_h + (self.target_h - self.from_h) * eased;
        (h, t < 1.0)
    }
}

/// Container visual (fundo, borda, sombra, padding e morphing de altura) do menu.
pub(crate) struct PopoverFrame;

impl PopoverFrame {
    fn morph_id() -> Id {
        Id::new("nixo_settings_popover_height_morph")
    }

    /// Limpa o estado de altura animada quando o menu é fechado.
    pub(crate) fn clear_height(ctx: &Context) {
        ctx.data_mut(|d| d.remove::<HeightMorphState>(Self::morph_id()));
    }

    /// Retorna a altura externa atual (interpolada com cubic ease-out) do quadro.
    pub(crate) fn current_height(ctx: &Context) -> f32 {
        let st = menu_style();
        let now = ctx.input(|i| i.time);
        let Some(state) = ctx.data(|d| d.get_temp::<HeightMorphState>(Self::morph_id())) else {
            return st.initial_main_height();
        };
        let (h, animating) = state.eval(now, f64::from(st.height_morph_duration));
        if animating {
            ctx.request_repaint();
        }
        h
    }

    pub(crate) fn show<R>(
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let st = menu_style();
        let ctx = ui.ctx().clone();
        let has_prev = ctx.data(|d| d.get_temp::<HeightMorphState>(Self::morph_id()).is_some());
        let pre_outer_h = Self::current_height(&ctx);

        let frame = Frame::NONE
            .fill(st.popover_fill)
            .stroke(st.popover_stroke)
            .corner_radius(st.popover_radius)
            .shadow(st.popover_shadow)
            .inner_margin(st.popover_padding);

        let where_to_put_background = ui.painter().add(egui::Shape::Noop);
        let outer_min = ui.available_rect_before_wrap().min;

        let stroke_w = st.popover_stroke.width;
        let pad_l = f32::from(st.popover_padding.left) + stroke_w;
        let pad_r = f32::from(st.popover_padding.right) + stroke_w;
        let pad_t = f32::from(st.popover_padding.top) + stroke_w;
        let pad_b = f32::from(st.popover_padding.bottom) + stroke_w;
        let inner_w = (st.popover_width - pad_l - pad_r).max(0.0);

        let max_content_rect =
            Rect::from_min_size(outer_min + vec2(pad_l, pad_t), vec2(inner_w, 4000.0));
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(max_content_rect));
        if has_prev {
            let clip = Rect::from_min_size(outer_min, vec2(st.popover_width, pre_outer_h))
                .shrink(stroke_w + 2.0);
            content_ui.set_clip_rect(clip);
        }
        content_ui.spacing_mut().item_spacing.y = st.item_spacing_y;

        let ret = add_contents(&mut content_ui);

        let natural_content_rect = content_ui.min_rect();
        let measured_outer_h = natural_content_rect.height() + pad_t + pad_b;
        let now = ctx.input(|i| i.time);
        let (current_outer_h, needs_repaint) = ctx.data_mut(|d| {
            let id = Self::morph_id();
            match d.get_temp::<HeightMorphState>(id) {
                Some(s) if (s.target_h - measured_outer_h).abs() > 0.5 => {
                    d.insert_temp(
                        id,
                        HeightMorphState {
                            from_h: pre_outer_h,
                            target_h: measured_outer_h,
                            start_time: now,
                        },
                    );
                    (pre_outer_h, true)
                }
                Some(_) => (pre_outer_h, false),
                None => {
                    d.insert_temp(
                        id,
                        HeightMorphState {
                            from_h: measured_outer_h,
                            target_h: measured_outer_h,
                            start_time: now,
                        },
                    );
                    let changed = (measured_outer_h - pre_outer_h).abs() > 0.5;
                    (measured_outer_h, changed)
                }
            }
        });
        if needs_repaint {
            ctx.request_repaint();
        }

        let anim_outer_rect =
            Rect::from_min_size(outer_min, vec2(st.popover_width, current_outer_h));
        let anim_inner_h = (current_outer_h - pad_t - pad_b).max(0.0);
        let anim_content_rect = Rect::from_min_size(
            Pos2::new(outer_min.x + pad_l, outer_min.y + pad_t),
            vec2(inner_w, anim_inner_h),
        );
        if ui.is_rect_visible(anim_outer_rect) {
            ui.painter()
                .set(where_to_put_background, frame.paint(anim_content_rect));
        }
        let response = ui.allocate_rect(anim_outer_rect, Sense::hover());
        InnerResponse::new(ret, response)
    }
}

// ───────────────────────────── WarningBanner ─────────────────────────────

/// Banner de aviso destacado (ex.: problemas de saúde da legenda).
pub(crate) struct WarningBanner<'a> {
    message: &'a str,
}

impl<'a> WarningBanner<'a> {
    pub(crate) fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl Widget for WarningBanner<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        Frame::NONE
            .fill(st.warning_fill)
            .corner_radius(st.warning_radius)
            .inner_margin(st.warning_padding)
            .outer_margin(st.warning_margin)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    RichText::new(self.message)
                        .font(st.warning_font.clone())
                        .color(st.warning_text),
                );
            })
            .response
    }
}

// ────────────────────────────── EmptyState ───────────────────────────────

/// Texto discreto exibido quando uma lista não possui itens.
pub(crate) struct EmptyState<'a> {
    message: &'a str,
}

impl<'a> EmptyState<'a> {
    pub(crate) fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl Widget for EmptyState<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        ui.horizontal(|ui| {
            ui.add_space(st.row_padding_x);
            ui.label(
                RichText::new(self.message)
                    .font(st.font_muted.clone())
                    .color(st.text_muted),
            );
        })
        .response
    }
}

// ─────────────────────────── Submenu header ──────────────────────────────

/// Cabeçalho de submenu seguido de divisor. Retorna `true` ao clicar em voltar.
pub(crate) fn submenu_header(ui: &mut Ui, title: &str) -> bool {
    let back = ui.add(SubmenuHeader::new(title)).clicked();
    ui.add(MenuDivider);
    back
}

// ────────────────────────────── ChoiceList ───────────────────────────────

/// Item de uma lista de escolha exclusiva, associado a um valor `T`.
pub(crate) struct Choice<'a, T> {
    label: Cow<'a, str>,
    value: T,
    selected: bool,
    icon: LeadingIcon,
    font: Option<FontId>,
}

impl<'a, T> Choice<'a, T> {
    pub(crate) fn new(label: impl Into<Cow<'a, str>>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
            selected: false,
            icon: LeadingIcon::None,
            font: None,
        }
    }

    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub(crate) fn icon(mut self, icon: LeadingIcon) -> Self {
        self.icon = icon;
        self
    }

    pub(crate) fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }
}

/// Renderiza uma lista de [`MenuOption`] e retorna o valor do item clicado.
pub(crate) fn choice_list<'a, T>(
    ui: &mut Ui,
    choices: impl IntoIterator<Item = Choice<'a, T>>,
) -> Option<T> {
    let mut picked = None;
    for choice in choices {
        let option = MenuOption::new(&choice.label)
            .selected(choice.selected)
            .icon(choice.icon)
            .font(choice.font);
        if ui.add(option).clicked() {
            picked = Some(choice.value);
        }
    }
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_frame_height_morph_interpolates_without_deadlock() {
        let ctx = Context::default();
        crate::NixoApp::configure_fonts(&ctx);

        let run_frame = |time: f64, rows: usize| -> f32 {
            let mut raw = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            raw.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0)));
            let mut h = 0.0_f32;
            let _ = ctx.run(raw, |ctx| {
                egui::Area::new(Id::new("test_popover"))
                    .fixed_pos(Pos2::new(100.0, 100.0))
                    .show(ctx, |ui| {
                        ui.set_width(menu_style().popover_width);
                        let resp = PopoverFrame::show(ui, |ui| {
                            for i in 0..rows {
                                let _ = ui.add(MenuOption::new(&format!("Item {i}")));
                            }
                        });
                        h = resp.response.rect.height();
                    });
            });
            h
        };

        let h_small = run_frame(0.0, 3);
        let h_start_morph = run_frame(0.016, 8);
        assert!((h_start_morph - h_small).abs() < 1.0);

        let h_mid = run_frame(0.016 + 0.09, 8);
        assert!(h_mid > h_small + 20.0);

        let h_final = run_frame(0.016 + 0.30, 8);
        assert!(h_final > h_mid);

        PopoverFrame::clear_height(&ctx);
    }
}
