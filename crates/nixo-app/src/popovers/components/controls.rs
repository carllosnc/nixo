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
use crate::popovers::style::menu_style;

// ────────────────────────────── MenuToggle ───────────────────────────────

/// Linha com interruptor ligado/desligado (toggle switch) no padrão OLED com animação fluida.
pub(crate) struct MenuToggle<'a> {
    title: &'a str,
    active: bool,
}

impl<'a> MenuToggle<'a> {
    pub(crate) fn new(title: &'a str, active: bool) -> Self {
        Self { title, active }
    }
}

/// Interpola linearmente entre duas cores RGBA.
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgba_premultiplied(
        (f32::from(a.r()) + (f32::from(b.r()) - f32::from(a.r())) * t + 0.5) as u8,
        (f32::from(a.g()) + (f32::from(b.g()) - f32::from(a.g())) * t + 0.5) as u8,
        (f32::from(a.b()) + (f32::from(b.b()) - f32::from(a.b())) * t + 0.5) as u8,
        (f32::from(a.a()) + (f32::from(b.a()) - f32::from(a.a())) * t + 0.5) as u8,
    )
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
        let track_fill_off = Color32::from_rgb(32, 32, 38);
        let track_fill = lerp_color(track_fill_off, ACCENT_COLOR, t);

        let stroke_off = if hovered {
            Color32::from_rgb(90, 90, 102)
        } else {
            Color32::from_rgb(58, 58, 66)
        };
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

        let thumb_color = lerp_color(Color32::from_rgb(175, 175, 185), Color32::WHITE, t);

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

    // Botão Menos [-]
    let minus_resp = ui
        .allocate_rect(minus_rect, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Atrasar (-0.1s)");
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
