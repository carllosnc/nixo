//! Blocos compostos do menu: container, avisos, estado vazio e listas de escolha.

use std::borrow::Cow;

use egui::{FontId, Frame, InnerResponse, Response, RichText, Ui, Widget};

use super::icons::LeadingIcon;
use super::rows::{MenuDivider, MenuOption, SubmenuHeader};
use crate::popovers::style::menu_style;

// ────────────────────────────── PopoverFrame ─────────────────────────────

/// Container visual (fundo, borda, sombra e padding) do menu.
pub(crate) struct PopoverFrame;

impl PopoverFrame {
    pub(crate) fn show<R>(
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let st = menu_style();
        Frame::NONE
            .fill(st.popover_fill)
            .stroke(st.popover_stroke)
            .corner_radius(st.popover_radius)
            .shadow(st.popover_shadow)
            .inner_margin(st.popover_padding)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = st.item_spacing_y;
                add_contents(ui)
            })
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
