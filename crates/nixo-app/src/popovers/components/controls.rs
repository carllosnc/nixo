//! Controles interativos do menu de configurações: interruptores e seletores de incremento.
//!
//! - [`MenuToggle`]: interruptor liga/desliga estilo pílula OLED.
//! - [`delay_stepper`]: micro-stepper para ajuste fino de sincronia com botões e reset por clique.

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Widget, vec2,
};

use super::primitives::{allocate_row, hover_color, paint_row_hover, paint_text};
use crate::app::ACCENT_COLOR;
use crate::components::tokens::*;
use crate::popovers::style::menu_style;
use crate::theme::font_bold;

// ────────────────────────────── MenuToggle ───────────────────────────────

/// Linha com interruptor ligado/desligado (toggle switch) no padrão OLED com animação fluida.
#[allow(dead_code)]
pub(crate) struct MenuToggle<'a> {
    title: &'a str,
    active: bool,
}

impl<'a> MenuToggle<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(title: &'a str, active: bool) -> Self {
        Self { title, active }
    }
}

impl Widget for MenuToggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.nav_height, Sense::click());
        let hovered = resp.hovered();
        paint_row_hover(ui, rect, hovered);

        let cy = rect.center().y;
        paint_text(
            ui,
            self.title,
            st.font_label.clone(),
            hover_color(hovered, st.text_nav_title),
            Pos2::new(rect.left() + st.row_padding_x, cy),
            Align2::LEFT_CENTER,
            f32::INFINITY,
        );

        let switch_w = 36.0;
        let switch_h = 20.0;
        let switch_right = rect.right() - st.row_padding_x;
        let switch_rect = Rect::from_center_size(
            Pos2::new(switch_right - switch_w * 0.5, cy),
            vec2(switch_w, switch_h),
        );

        // Animação de transição fluida do interruptor (0.0 = desligado, 1.0 = ligado)
        let t = ui
            .ctx()
            .animate_bool_with_time(resp.id.with("switch_t"), self.active, 0.18);

        let is_pressed = resp.is_pointer_button_down_on();
        let stretch =
            ui.ctx()
                .animate_bool_with_time(resp.id.with("switch_stretch"), is_pressed, 0.10);

        // Cores interpoladas para transição suave de fundo e borda
        let track_fill_off = SURFACE_CARD_HOVER;
        let track_fill = lerp_color(track_fill_off, ACCENT_COLOR, t);

        let stroke_off = if hovered { NEUTRAL_600 } else { NEUTRAL_700 };
        let track_stroke = Stroke::new(1.2_f32, lerp_color(stroke_off, ACCENT_COLOR, t));

        ui.painter().rect(
            switch_rect,
            CornerRadius::same(10),
            track_fill,
            track_stroke,
            StrokeKind::Inside,
        );

        let thumb_r = 6.8;
        let x_off = switch_rect.left() + 3.2 + thumb_r;
        let x_on = switch_rect.right() - 3.2 - thumb_r;
        let thumb_x = egui::lerp(x_off..=x_on, t);

        let thumb_color = lerp_color(TEXT_SECONDARY, NEUTRAL_50, t);

        // Sombra suave sob o botão quando ativo
        if t > 0.05 {
            let shadow_alpha = (50.0 * t) as u8;
            ui.painter().circle_filled(
                Pos2::new(thumb_x, cy + 1.0),
                thumb_r,
                Color32::from_black_alpha(shadow_alpha),
            );
        }

        // Leve estiramento físico elástico ao pressionar
        let thumb_w = thumb_r * 2.0 + stretch * 2.5;
        let thumb_rect =
            Rect::from_center_size(Pos2::new(thumb_x, cy), vec2(thumb_w, thumb_r * 2.0));
        ui.painter()
            .rect_filled(thumb_rect, CornerRadius::same(thumb_r as u8), thumb_color);

        resp.on_hover_cursor(CursorIcon::PointingHand)
    }
}

// ───────────────────────────── delay_stepper ─────────────────────────────

/// Linha de controle de sincronia com botões `[-]`, valor clicável (reset) e `[+]`.
pub(crate) fn delay_stepper(ui: &mut Ui, label: &str, current_delay: f64) -> Option<f64> {
    let st = menu_style();
    let (rect, _) = allocate_row(ui, st.nav_height, Sense::hover());
    let cy = rect.center().y;

    paint_text(
        ui,
        label,
        st.font_label.clone(),
        st.text_nav_title,
        Pos2::new(rect.left() + st.row_padding_x, cy),
        Align2::LEFT_CENTER,
        f32::INFINITY,
    );

    let btn_size = vec2(32.0, 28.0);
    let right_edge = rect.right() - st.row_padding_x;

    let plus_rect = Rect::from_center_size(Pos2::new(right_edge - btn_size.x * 0.5, cy), btn_size);
    let val_w = 64.0;
    let val_rect = Rect::from_center_size(
        Pos2::new(plus_rect.left() - 5.0 - val_w * 0.5, cy),
        vec2(val_w, btn_size.y),
    );
    let minus_rect = Rect::from_center_size(
        Pos2::new(val_rect.left() - 5.0 - btn_size.x * 0.5, cy),
        btn_size,
    );

    let mut new_delay = None;

    let btn_radius = RADIUS_SM;
    let btn_idle_bg = SURFACE_MED;
    let btn_hover_bg = SURFACE_HIGH;
    let btn_active_bg = SURFACE_ACTIVE;

    // Botão Menos [-]
    let minus_resp = ui
        .allocate_rect(minus_rect, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Atrasar (-0.1s)");
    let minus_hover = minus_resp.hovered();
    let minus_bg = if minus_resp.is_pointer_button_down_on() {
        btn_active_bg
    } else if minus_hover {
        btn_hover_bg
    } else {
        btn_idle_bg
    };
    ui.painter().rect_filled(minus_rect, btn_radius, minus_bg);
    ui.painter().line_segment(
        [
            Pos2::new(minus_rect.center().x - 5.5, cy),
            Pos2::new(minus_rect.center().x + 5.5, cy),
        ],
        Stroke::new(1.8_f32, hover_color(minus_hover, st.text_option)),
    );
    if minus_resp.clicked() {
        new_delay = Some(((current_delay - 0.1) * 10.0).round() / 10.0);
    }

    // Badge de Valor (clique reseta para 0.0s)
    let val_resp = ui
        .allocate_rect(val_rect, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Clique para redefinir para 0.0s");
    let val_hover = val_resp.hovered();
    let val_text = if current_delay.abs() < 0.01 {
        "0.0s".to_string()
    } else {
        format!("{current_delay:+.1}s")
    };
    let val_bg = if val_resp.is_pointer_button_down_on() {
        btn_active_bg
    } else if val_hover {
        btn_hover_bg
    } else {
        btn_idle_bg
    };
    ui.painter().rect_filled(val_rect, btn_radius, val_bg);
    paint_text(
        ui,
        &val_text,
        egui::FontId::proportional(13.5),
        hover_color(val_hover, st.text_hover),
        val_rect.center(),
        Align2::CENTER_CENTER,
        val_rect.width(),
    );
    if val_resp.clicked() && current_delay.abs() >= 0.01 {
        new_delay = Some(0.0);
    }

    // Botão Mais [+]
    let plus_resp = ui
        .allocate_rect(plus_rect, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Adiantar (+0.1s)");
    let plus_hover = plus_resp.hovered();
    let plus_bg = if plus_resp.is_pointer_button_down_on() {
        btn_active_bg
    } else if plus_hover {
        btn_hover_bg
    } else {
        btn_idle_bg
    };
    ui.painter().rect_filled(plus_rect, btn_radius, plus_bg);
    let px = plus_rect.center().x;
    ui.painter().line_segment(
        [Pos2::new(px - 5.5, cy), Pos2::new(px + 5.5, cy)],
        Stroke::new(1.8_f32, hover_color(plus_hover, st.text_option)),
    );
    ui.painter().line_segment(
        [Pos2::new(px, cy - 5.5), Pos2::new(px, cy + 5.5)],
        Stroke::new(1.8_f32, hover_color(plus_hover, st.text_option)),
    );
    if plus_resp.clicked() {
        new_delay = Some(((current_delay + 0.1) * 10.0).round() / 10.0);
    }

    new_delay
}

// ───────────────────────────── menu_slider ─────────────────────────────

/// Linha de controle com rótulo, valor atual e slider interativo com botões de passo [-] e [+].
pub(crate) fn menu_slider(
    ui: &mut Ui,
    label: &str,
    current_value: f64,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    format_val: impl Fn(f64) -> String,
) -> Option<f64> {
    let st = menu_style();
    let (rect, _) = allocate_row(ui, 54.0, Sense::hover());

    let top_y = rect.top() + 11.0;
    let bottom_y = rect.top() + 38.0;

    // Rótulo na esquerda
    paint_text(
        ui,
        label,
        st.font_label.clone(),
        st.text_nav_title,
        Pos2::new(rect.left() + st.row_padding_x, top_y),
        Align2::LEFT_CENTER,
        f32::INFINITY,
    );

    // Valor formatado na direita
    let val_str = format_val(current_value);
    paint_text(
        ui,
        &val_str,
        font_bold(13.0),
        ACCENT_COLOR,
        Pos2::new(rect.right() - st.row_padding_x, top_y),
        Align2::RIGHT_CENTER,
        f32::INFINITY,
    );

    let min = *range.start();
    let max = *range.end();
    let span = (max - min).max(0.0001);

    let mut new_val = None;

    let btn_size = vec2(28.0, 26.0);
    let btn_radius = RADIUS_SM;
    let btn_idle_bg = SURFACE_MED;
    let btn_hover_bg = SURFACE_HIGH;
    let btn_active_bg = SURFACE_ACTIVE;

    // Botão Menos [-]
    let minus_center = Pos2::new(rect.left() + st.row_padding_x + btn_size.x * 0.5, bottom_y);
    let minus_rect = Rect::from_center_size(minus_center, btn_size);
    let minus_resp = ui
        .allocate_rect(minus_rect, Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let minus_hover = minus_resp.hovered();
    let minus_bg = if minus_resp.is_pointer_button_down_on() {
        btn_active_bg
    } else if minus_hover {
        btn_hover_bg
    } else {
        btn_idle_bg
    };
    ui.painter().rect_filled(minus_rect, btn_radius, minus_bg);
    ui.painter().line_segment(
        [
            Pos2::new(minus_center.x - 5.0, bottom_y),
            Pos2::new(minus_center.x + 5.0, bottom_y),
        ],
        Stroke::new(1.8_f32, hover_color(minus_hover, st.text_option)),
    );
    if minus_resp.clicked() {
        let stepped = ((current_value - step - min) / step).round() * step + min;
        new_val = Some(stepped.clamp(min, max));
    }

    // Botão Mais [+]
    let plus_center = Pos2::new(rect.right() - st.row_padding_x - btn_size.x * 0.5, bottom_y);
    let plus_rect = Rect::from_center_size(plus_center, btn_size);
    let plus_resp = ui
        .allocate_rect(plus_rect, Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let plus_hover = plus_resp.hovered();
    let plus_bg = if plus_resp.is_pointer_button_down_on() {
        btn_active_bg
    } else if plus_hover {
        btn_hover_bg
    } else {
        btn_idle_bg
    };
    ui.painter().rect_filled(plus_rect, btn_radius, plus_bg);
    let px = plus_center.x;
    ui.painter().line_segment(
        [Pos2::new(px - 5.0, bottom_y), Pos2::new(px + 5.0, bottom_y)],
        Stroke::new(1.8_f32, hover_color(plus_hover, st.text_option)),
    );
    ui.painter().line_segment(
        [Pos2::new(px, bottom_y - 5.0), Pos2::new(px, bottom_y + 5.0)],
        Stroke::new(1.8_f32, hover_color(plus_hover, st.text_option)),
    );
    if plus_resp.clicked() {
        let stepped = ((current_value + step - min) / step).round() * step + min;
        new_val = Some(stepped.clamp(min, max));
    }

    // Trilho central do Slider
    let track_left = minus_rect.right() + 10.0;
    let track_right = plus_rect.left() - 10.0;
    let track_w = (track_right - track_left).max(10.0);
    let track_hit_rect = Rect::from_min_max(
        Pos2::new(track_left, bottom_y - 12.0),
        Pos2::new(track_right, bottom_y + 12.0),
    );
    let track_resp = ui
        .allocate_rect(track_hit_rect, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::PointingHand);

    let is_interacting =
        track_resp.is_pointer_button_down_on() || track_resp.clicked() || track_resp.dragged();

    if is_interacting && let Some(mouse_pos) = track_resp.interact_pointer_pos() {
        let normalized = ((mouse_pos.x - track_left) / track_w).clamp(0.0, 1.0);
        let raw = min + (normalized as f64) * span;
        let stepped = ((raw - min) / step).round() * step + min;
        new_val = Some(stepped.clamp(min, max));
    }

    // Desenha o trilho
    let norm = ((current_value - min) / span).clamp(0.0, 1.0) as f32;
    let thumb_x = track_left + norm * track_w;

    let is_active = track_resp.hovered() || is_interacting;
    let anim_t = ui
        .ctx()
        .animate_bool_with_time(track_resp.id.with("slider_t"), is_active, 0.12);

    let rail_h = egui::lerp(3.5..=4.5, anim_t);
    let rail_bg = Rect::from_min_max(
        Pos2::new(track_left, bottom_y - rail_h * 0.5),
        Pos2::new(track_right, bottom_y + rail_h * 0.5),
    );
    ui.painter()
        .rect_filled(rail_bg, CornerRadius::same(2), SURFACE_HIGH);

    let rail_fill = Rect::from_min_max(
        Pos2::new(track_left, bottom_y - rail_h * 0.5),
        Pos2::new(thumb_x, bottom_y + rail_h * 0.5),
    );
    ui.painter()
        .rect_filled(rail_fill, CornerRadius::same(2), ACCENT_COLOR);

    // Indicador (thumb)
    let thumb_r = egui::lerp(6.5..=8.0, anim_t);
    ui.painter().circle_filled(
        Pos2::new(thumb_x, bottom_y),
        thumb_r + 1.5,
        Color32::from_black_alpha(90),
    );
    ui.painter()
        .circle_filled(Pos2::new(thumb_x, bottom_y), thumb_r, ACCENT_COLOR);
    if anim_t > 0.05 {
        ui.painter()
            .circle_filled(Pos2::new(thumb_x, bottom_y), thumb_r * 0.45, NEUTRAL_50);
    }

    if let Some(v) = new_val {
        let clamped = v.clamp(min, max);
        if (clamped - current_value).abs() > 0.0001 {
            return Some(clamped);
        }
    }

    None
}
