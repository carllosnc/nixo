//! Controles interativos do menu de configurações: interruptores e seletores de incremento.
//!
//! - [`MenuToggle`]: interruptor liga/desliga estilo pílula OLED.
//! - [`delay_stepper`]: micro-stepper para ajuste fino de sincronia com botões e reset por clique.

use egui::{
    Align2, CornerRadius, CursorIcon, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget,
    vec2,
};

use super::primitives::{allocate_row, hover_color, paint_row_hover, paint_text};
use crate::popovers::style::menu_style;

// ────────────────────────────── MenuToggle ───────────────────────────────

/// Linha com interruptor ligado/desligado (toggle switch) no padrão OLED.
pub(crate) struct MenuToggle<'a> {
    title: &'a str,
    active: bool,
}

impl<'a> MenuToggle<'a> {
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

        let switch_w = 34.0;
        let switch_h = 18.0;
        let switch_right = rect.right() - st.row_padding_x;
        let switch_rect = Rect::from_center_size(
            Pos2::new(switch_right - switch_w * 0.5, cy),
            vec2(switch_w, switch_h),
        );

        let track_bg = if self.active {
            st.button_fill_hover
        } else {
            st.row_selected_fill
        };
        let track_stroke = Stroke::new(
            1.0_f32,
            if self.active {
                st.text_hover
            } else {
                st.popover_stroke.color
            },
        );
        ui.painter().rect(
            switch_rect,
            CornerRadius::same(9),
            track_bg,
            track_stroke,
            StrokeKind::Inside,
        );

        let thumb_r = 6.0;
        let thumb_x = if self.active {
            switch_rect.right() - 2.5 - thumb_r
        } else {
            switch_rect.left() + 2.5 + thumb_r
        };
        let thumb_color = if self.active {
            st.text_hover
        } else {
            st.chevron_idle
        };
        ui.painter()
            .circle_filled(Pos2::new(thumb_x, cy), thumb_r, thumb_color);

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

    let btn_size = vec2(24.0, 22.0);
    let right_edge = rect.right() - st.row_padding_x;

    let plus_rect = Rect::from_center_size(Pos2::new(right_edge - btn_size.x * 0.5, cy), btn_size);
    let val_w = 48.0;
    let val_rect = Rect::from_center_size(
        Pos2::new(plus_rect.left() - 4.0 - val_w * 0.5, cy),
        vec2(val_w, btn_size.y),
    );
    let minus_rect = Rect::from_center_size(
        Pos2::new(val_rect.left() - 4.0 - btn_size.x * 0.5, cy),
        btn_size,
    );

    let mut new_delay = None;

    // Botão Menos [-]
    let minus_resp = ui.allocate_rect(minus_rect, Sense::click());
    let minus_hover = minus_resp.hovered();
    let minus_bg = if minus_resp.is_pointer_button_down_on() {
        st.button_fill_active
    } else if minus_hover {
        st.button_fill_hover
    } else {
        st.row_selected_fill
    };
    ui.painter().rect(
        minus_rect,
        CornerRadius::same(99),
        minus_bg,
        Stroke::new(1.0_f32, st.popover_stroke.color),
        StrokeKind::Inside,
    );
    ui.painter().line_segment(
        [
            Pos2::new(minus_rect.center().x - 4.0, cy),
            Pos2::new(minus_rect.center().x + 4.0, cy),
        ],
        Stroke::new(1.4_f32, hover_color(minus_hover, st.text_option)),
    );
    if minus_resp.clicked() {
        new_delay = Some(((current_delay - 0.1) * 10.0).round() / 10.0);
    }

    // Badge de Valor (clique reseta para 0.0s)
    let val_resp = ui.allocate_rect(val_rect, Sense::click());
    let val_hover = val_resp.hovered();
    let val_text = if current_delay.abs() < 0.01 {
        "0.0s".to_string()
    } else {
        format!("{current_delay:+.1}s")
    };
    let val_bg = if val_hover {
        st.button_fill_hover
    } else {
        st.row_selected_fill
    };
    ui.painter().rect(
        val_rect,
        CornerRadius::same(99),
        val_bg,
        Stroke::new(1.0_f32, st.popover_stroke.color),
        StrokeKind::Inside,
    );
    paint_text(
        ui,
        &val_text,
        st.font_value.clone(),
        hover_color(val_hover, st.text_hover),
        val_rect.center(),
        Align2::CENTER_CENTER,
        val_rect.width(),
    );
    if val_resp.clicked() && current_delay.abs() >= 0.01 {
        new_delay = Some(0.0);
    }

    // Botão Mais [+]
    let plus_resp = ui.allocate_rect(plus_rect, Sense::click());
    let plus_hover = plus_resp.hovered();
    let plus_bg = if plus_resp.is_pointer_button_down_on() {
        st.button_fill_active
    } else if plus_hover {
        st.button_fill_hover
    } else {
        st.row_selected_fill
    };
    ui.painter().rect(
        plus_rect,
        CornerRadius::same(99),
        plus_bg,
        Stroke::new(1.0_f32, st.popover_stroke.color),
        StrokeKind::Inside,
    );
    let px = plus_rect.center().x;
    ui.painter().line_segment(
        [Pos2::new(px - 4.0, cy), Pos2::new(px + 4.0, cy)],
        Stroke::new(1.4_f32, hover_color(plus_hover, st.text_option)),
    );
    ui.painter().line_segment(
        [Pos2::new(px, cy - 4.0), Pos2::new(px, cy + 4.0)],
        Stroke::new(1.4_f32, hover_color(plus_hover, st.text_option)),
    );
    if plus_resp.clicked() {
        new_delay = Some(((current_delay + 0.1) * 10.0).round() / 10.0);
    }

    new_delay
}
