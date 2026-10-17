//! Linhas do menu: cada tipo de linha é um widget único (`egui::Widget`).
//!
//! - [`MenuOption`]: opção selecionável com marcador, ícone e chevron opcional.
//! - [`NavRow`]: linha de navegação "TÍTULO … valor >".
//! - [`SubmenuHeader`]: cabeçalho "< TÍTULO" para voltar à tela anterior.
//! - [`SectionTitle`]: título estático de seção.
//! - [`MenuDivider`]: separador horizontal.
//! - [`MenuButton`]: botão de ação centralizado no quadro.

use egui::{Align2, CursorIcon, FontId, Pos2, Rect, Response, Sense, Ui, Widget, vec2};

use super::icons::LeadingIcon;
use super::primitives::{
    ChevronDir, allocate_row, hover_color, layout_single_line, paint_checkmark, paint_chevron,
    paint_galley, paint_row_hover, paint_text,
};
use crate::popovers::style::menu_style;

// ─────────────────────────────── MenuOption ───────────────────────────────

/// Opção selecionável do menu.
///
/// ```ignore
/// ui.add(MenuOption::new("Normal").selected(true).icon(LeadingIcon::Disabled));
/// ```
pub(crate) struct MenuOption<'a> {
    label: &'a str,
    selected: bool,
    icon: LeadingIcon,
    font: Option<FontId>,
    chevron: bool,
}

impl<'a> MenuOption<'a> {
    pub(crate) fn new(label: &'a str) -> Self {
        Self {
            label,
            selected: false,
            icon: LeadingIcon::None,
            font: None,
            chevron: false,
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

    /// Substitui a fonte padrão do rótulo (ex.: prévia de fonte de legenda).
    pub(crate) fn font(mut self, font: Option<FontId>) -> Self {
        self.font = font;
        self
    }

    /// Exibe um chevron `>` indicando que a opção abre um submenu.
    pub(crate) fn chevron(mut self) -> Self {
        self.chevron = true;
        self
    }
}

impl Widget for MenuOption<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.option_height, Sense::click());
        let hovered = resp.hovered();
        if hovered {
            paint_row_hover(ui, rect, true);
        } else if self.selected {
            ui.painter()
                .rect_filled(rect, st.row_radius, st.row_selected_fill);
        }

        let cy = rect.center().y;
        if self.selected {
            paint_checkmark(ui, Pos2::new(rect.left() + st.check_offset_x, cy));
        }

        let text_x = if self.icon.is_none() {
            rect.left() + st.text_offset_x
        } else {
            self.icon
                .paint(ui, Pos2::new(rect.left() + st.icon_offset_x, cy));
            rect.left() + st.text_offset_x_with_icon
        };

        let color = if self.selected {
            st.text_hover
        } else {
            hover_color(hovered, st.text_option)
        };
        let font = self.font.unwrap_or_else(|| {
            if self.selected {
                st.font_option_selected.clone()
            } else {
                st.font_option.clone()
            }
        });

        let trailing = if self.chevron {
            st.chevron_inset + st.chevron_size + st.row_padding_x
        } else {
            st.row_padding_x
        };
        paint_text(
            ui,
            self.label,
            font,
            color,
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            rect.right() - trailing - text_x,
        );

        if self.chevron {
            let tip = Pos2::new(rect.right() - st.chevron_inset, cy);
            paint_chevron(
                ui,
                tip,
                ChevronDir::Right,
                hover_color(hovered, st.chevron_idle),
            );
        }

        resp.on_hover_cursor(CursorIcon::PointingHand)
    }
}

// ──────────────────────────────── NavRow ─────────────────────────────────

/// Linha de navegação: título à esquerda, valor atual à direita e chevron `>`.
pub(crate) struct NavRow<'a> {
    title: &'a str,
    value: &'a str,
}

impl<'a> NavRow<'a> {
    pub(crate) fn new(title: &'a str, value: &'a str) -> Self {
        Self { title, value }
    }
}

impl Widget for NavRow<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.nav_height, Sense::click());
        let hovered = resp.hovered();
        paint_row_hover(ui, rect, hovered);

        let cy = rect.center().y;
        let title_rect = paint_text(
            ui,
            self.title,
            st.font_label.clone(),
            hover_color(hovered, st.text_nav_title),
            Pos2::new(rect.left() + st.row_padding_x, cy),
            Align2::LEFT_CENTER,
            f32::INFINITY,
        );

        let chevron_tip = Pos2::new(rect.right() - st.chevron_inset, cy);
        paint_chevron(
            ui,
            chevron_tip,
            ChevronDir::Right,
            hover_color(hovered, st.chevron_idle),
        );

        let value_right = chevron_tip.x - st.nav_value_gap;
        paint_text(
            ui,
            self.value,
            st.font_value.clone(),
            hover_color(hovered, st.text_nav_value),
            Pos2::new(value_right, cy),
            Align2::RIGHT_CENTER,
            value_right - title_rect.right() - st.nav_value_gap,
        );

        resp.on_hover_cursor(CursorIcon::PointingHand)
    }
}

// ───────────────────────────── SubmenuHeader ─────────────────────────────

/// Cabeçalho de submenu com seta de voltar `<`.
pub(crate) struct SubmenuHeader<'a> {
    title: &'a str,
}

impl<'a> SubmenuHeader<'a> {
    pub(crate) fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl Widget for SubmenuHeader<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.header_height, Sense::click());
        let hovered = resp.hovered();
        paint_row_hover(ui, rect, hovered);

        let cy = rect.center().y;
        let color = hover_color(hovered, st.text_header);
        let tip = Pos2::new(rect.left() + st.row_padding_x, cy);
        paint_chevron(ui, tip, ChevronDir::Left, color);

        let text_x = tip.x + st.header_text_gap;
        paint_text(
            ui,
            self.title,
            st.font_label.clone(),
            color,
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            rect.right() - st.row_padding_x - text_x,
        );

        resp.on_hover_cursor(CursorIcon::PointingHand)
    }
}

// ────────────────────────────── SectionTitle ─────────────────────────────

/// Título estático de seção (não interativo).
pub(crate) struct SectionTitle<'a> {
    title: &'a str,
}

impl<'a> SectionTitle<'a> {
    pub(crate) fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl Widget for SectionTitle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.section_title_height, Sense::hover());
        let text_x = rect.left() + st.row_padding_x;
        paint_text(
            ui,
            self.title,
            st.font_label.clone(),
            st.text_section_title,
            Pos2::new(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            rect.right() - st.row_padding_x - text_x,
        );
        resp
    }
}

// ────────────────────────────── MenuDivider ──────────────────────────────

/// Separador horizontal entre grupos de linhas.
pub(crate) struct MenuDivider;

impl Widget for MenuDivider {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let (rect, resp) = allocate_row(ui, st.divider_height, Sense::hover());
        let y = rect.center().y;
        ui.painter().line_segment(
            [
                Pos2::new(rect.left() + st.divider_inset, y),
                Pos2::new(rect.right() - st.divider_inset, y),
            ],
            st.divider_stroke,
        );
        resp
    }
}

// ────────────────────────────── MenuButton ───────────────────────────────

/// Botão de ação centralizado full-rounded no quadro do menu (ex.: "Redefinir padrão").
pub(crate) struct MenuButton<'a> {
    label: &'a str,
}

impl<'a> MenuButton<'a> {
    pub(crate) fn new(label: &'a str) -> Self {
        Self { label }
    }
}

impl Widget for MenuButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let st = menu_style();
        let total_h = st.button_height + st.button_margin_y * 2.0;
        let (row_rect, resp) = allocate_row(ui, total_h, Sense::click());
        let btn_rect = Rect::from_min_max(
            Pos2::new(
                row_rect.left() + st.button_inset_x,
                row_rect.top() + st.button_margin_y,
            ),
            Pos2::new(
                row_rect.right() - st.button_inset_x,
                row_rect.bottom() - st.button_margin_y,
            ),
        );

        let hovered = resp.hovered();
        let pressed = resp.is_pointer_button_down_on();
        let bg = if pressed {
            st.button_fill_active
        } else if hovered {
            st.button_fill_hover
        } else {
            st.button_fill
        };
        ui.painter().rect_filled(btn_rect, st.button_radius, bg);

        let fg = hover_color(hovered || pressed, st.button_text);
        let galley = layout_single_line(
            ui,
            self.label,
            st.button_font.clone(),
            fg,
            btn_rect.width() - st.row_padding_x * 2.0,
        );
        paint_galley(ui, galley, btn_rect.center(), Align2::CENTER_CENTER);

        let _ = vec2;
        resp.on_hover_cursor(CursorIcon::PointingHand)
    }
}
