//! Telas de estilo de legenda: visão geral e seletores de cada propriedade.
//!
//! Todos os seletores usam o mesmo componente ([`style_picker`]), que combina
//! cabeçalho de submenu + lista de escolha + envio de `SetSubtitleStyle`.

use std::borrow::Cow;

use egui::Ui;
use nixo_core::{
    PlayerCommand, SubtitleBackgroundStyle, SubtitleBorderStyle, SubtitleShadowStyle, SubtitleStyle,
};

use super::MenuActions;
use super::components::{
    Choice, LeadingIcon, MenuButton, MenuDivider, MenuOption, NavRow, WarningBanner, choice_list,
    menu_slider, submenu_header,
};
use crate::app::{NixoApp, SettingsTab};
use crate::i18n::Translations;

const SIZE_PRESETS: [u16; 6] = [50, 75, 100, 150, 200, 300];
const PREVIEW_FONT_SIZE: f32 = 14.0;

impl NixoApp {
    pub(super) fn render_subtitle_style_tabs(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let style = &self.status.subtitle_style;
        let t = self.t();
        match self.settings_tab {
            SettingsTab::SubtitleStyle => self.render_style_overview(ui, actions),
            SettingsTab::SubtitleFont => font_picker(ui, actions, style, t),
            SettingsTab::SubtitleWeight => weight_picker(ui, actions, style, t),
            SettingsTab::SubtitleSize => size_picker(ui, actions, style, t),
            SettingsTab::SubtitleColor => color_picker(ui, actions, style, t),
            SettingsTab::SubtitleBorder => border_picker(ui, actions, style, t),
            SettingsTab::SubtitleBackground => background_picker(ui, actions, style, t),
            SettingsTab::SubtitleShadow => shadow_picker(ui, actions, style, t),
            SettingsTab::SubtitlePosition => position_picker(ui, actions, style, t),
            _ => {}
        }
    }

    fn render_style_overview(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.style_title) {
            actions.navigate(SettingsTab::Subtitles);
        }

        if let Some(warning) = self.subtitle_health_warning() {
            ui.add(WarningBanner::new(&warning));
        }

        let style = &self.status.subtitle_style;
        let weight = if style.bold {
            t.style_weight_bold
        } else {
            t.style_weight_normal
        };
        let rows: [(&str, Cow<str>, SettingsTab); 8] = [
            (
                t.style_font,
                Cow::Borrowed(&style.font_family),
                SettingsTab::SubtitleFont,
            ),
            (
                t.style_font_weight,
                weight.into(),
                SettingsTab::SubtitleWeight,
            ),
            (
                t.style_size,
                format!("{}%", style.scale_percent).into(),
                SettingsTab::SubtitleSize,
            ),
            (
                t.style_text_color,
                color_label(style, t).into(),
                SettingsTab::SubtitleColor,
            ),
            (
                t.style_border,
                border_label(style.border, t).into(),
                SettingsTab::SubtitleBorder,
            ),
            (
                t.style_background,
                background_label(style, t).into(),
                SettingsTab::SubtitleBackground,
            ),
            (
                t.style_shadow,
                shadow_label(style.shadow, t).into(),
                SettingsTab::SubtitleShadow,
            ),
            (
                t.style_position,
                position_label(style, t),
                SettingsTab::SubtitlePosition,
            ),
        ];
        for (title, value, tab) in rows {
            if ui.add(NavRow::new(title, &value)).clicked() {
                actions.navigate(tab);
            }
        }

        ui.add(MenuDivider);
        if ui.add(MenuButton::new(t.style_reset_default)).clicked() {
            actions.send(PlayerCommand::SetSubtitleStyle(SubtitleStyle::default()));
        }
    }
}

/// Componente genérico de seletor de propriedade do estilo de legenda.
fn style_picker<'a, T>(
    ui: &mut Ui,
    actions: &mut MenuActions,
    title: &str,
    style: &SubtitleStyle,
    choices: impl IntoIterator<Item = Choice<'a, T>>,
    apply: impl FnOnce(&mut SubtitleStyle, T),
) {
    if submenu_header(ui, title) {
        actions.navigate(SettingsTab::SubtitleStyle);
    }
    if let Some(value) = choice_list(ui, choices) {
        let mut next = style.clone();
        apply(&mut next, value);
        actions.send(PlayerCommand::SetSubtitleStyle(next));
    }
}

fn font_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let ctx = ui.ctx().clone();
    let choices = SubtitleStyle::FONT_CHOICES.map(|name| {
        Choice::new(name, name)
            .selected(style.font_family == name)
            .font(NixoApp::subtitle_font_id(
                &ctx,
                name,
                style.bold,
                PREVIEW_FONT_SIZE,
            ))
    });
    style_picker(ui, actions, t.style_font, style, choices, |s, name| {
        s.font_family = name.to_string();
    });
}

fn weight_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let ctx = ui.ctx().clone();
    let choices =
        [(t.style_weight_normal, false), (t.style_weight_bold, true)].map(|(label, bold)| {
            Choice::new(label, bold)
                .selected(style.bold == bold)
                .font(NixoApp::subtitle_font_id(
                    &ctx,
                    &style.font_family,
                    bold,
                    PREVIEW_FONT_SIZE,
                ))
        });
    style_picker(
        ui,
        actions,
        t.style_font_weight,
        style,
        choices,
        |s, bold| {
            s.bold = bold;
        },
    );
}

fn size_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let choices = SIZE_PRESETS.map(|size| {
        let label = if size == 100 {
            format!("100% ({})", t.speed_normal)
        } else {
            format!("{size}%")
        };
        Choice::new(label, size).selected(style.scale_percent == size)
    });
    style_picker(ui, actions, t.style_size, style, choices, |s, size| {
        s.scale_percent = size;
    });
}

fn color_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let choices = SubtitleStyle::COLOR_SWATCHES.map(|(name, rgba)| {
        let label = t.color_name(name);
        Choice::new(label, rgba)
            .selected(same_rgb(style.text_color, rgba))
            .icon(LeadingIcon::ColorSwatch(rgba))
    });
    style_picker(
        ui,
        actions,
        t.style_text_color,
        style,
        choices,
        |s, rgba| {
            s.text_color = rgba;
        },
    );
}

fn border_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let choices = SubtitleBorderStyle::ALL.map(|border| {
        let label = border_label(border, t);
        Choice::new(label, border).selected(style.border == border)
    });
    style_picker(ui, actions, t.style_border, style, choices, |s, border| {
        s.border = border;
    });
}

fn background_picker(
    ui: &mut Ui,
    actions: &mut MenuActions,
    style: &SubtitleStyle,
    t: &Translations,
) {
    let presets = [
        (t.style_bg_none, SubtitleBackgroundStyle::None, 60),
        (t.style_bg_box_50, SubtitleBackgroundStyle::Box, 50),
        (t.style_bg_box_75, SubtitleBackgroundStyle::Box, 75),
        (t.style_bg_box_solid, SubtitleBackgroundStyle::Box, 100),
    ];
    let choices = presets.map(|(label, kind, opacity)| {
        let selected = match kind {
            SubtitleBackgroundStyle::None => style.background == SubtitleBackgroundStyle::None,
            _ => {
                style.background != SubtitleBackgroundStyle::None
                    && style.background_opacity == opacity
            }
        };
        Choice::new(label, (kind, opacity)).selected(selected)
    });
    style_picker(
        ui,
        actions,
        t.style_background,
        style,
        choices,
        |s, (kind, opacity)| {
            s.background = kind;
            s.background_opacity = opacity;
        },
    );
}

fn shadow_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle, t: &Translations) {
    let choices = SubtitleShadowStyle::ALL.map(|shadow| {
        let label = shadow_label(shadow, t);
        Choice::new(label, shadow).selected(style.shadow == shadow)
    });
    style_picker(ui, actions, t.style_shadow, style, choices, |s, shadow| {
        s.shadow = shadow;
    });
}

fn position_picker(
    ui: &mut Ui,
    actions: &mut MenuActions,
    style: &SubtitleStyle,
    t: &Translations,
) {
    if submenu_header(ui, t.style_position) {
        actions.navigate(SettingsTab::SubtitleStyle);
    }

    let cur_pos = f64::from(style.bottom_offset_percent);
    if let Some(new_pos) = menu_slider(ui, t.style_subtitle_height, cur_pos, 0.0..=90.0, 1.0, |v| {
        let val = v.round() as u8;
        match val {
            8 => t.style_pos_bottom.to_string(),
            20 => t.style_pos_raised.to_string(),
            82 => t.style_pos_top.to_string(),
            _ => format!("{val}%"),
        }
    }) {
        let mut next = style.clone();
        next.bottom_offset_percent = new_pos.round() as u8;
        actions.send(PlayerCommand::SetSubtitleStyle(next));
    }

    ui.add(MenuDivider);

    let position_presets = [
        (t.style_pos_bottom, 8),
        (t.style_pos_raised, 20),
        (t.style_pos_top, 82),
    ];
    for (label, offset) in position_presets {
        let selected = style.bottom_offset_percent == offset;
        if ui
            .add(MenuOption::new(label).radio(true).selected(selected))
            .clicked()
        {
            let mut next = style.clone();
            next.bottom_offset_percent = offset;
            actions.send(PlayerCommand::SetSubtitleStyle(next));
        }
    }

    ui.add(MenuDivider);
    if ui.add(MenuButton::new(t.style_reset_default)).clicked() {
        let mut next = style.clone();
        next.bottom_offset_percent = 8;
        actions.send(PlayerCommand::SetSubtitleStyle(next));
    }
}

fn same_rgb(a: [u8; 4], b: [u8; 4]) -> bool {
    a[..3] == b[..3]
}

fn color_label<'a>(style: &SubtitleStyle, t: &'a Translations) -> &'a str {
    SubtitleStyle::COLOR_SWATCHES
        .iter()
        .find(|(_, rgba)| same_rgb(style.text_color, *rgba))
        .map_or(t.color_white, |(name, _)| t.color_name(name))
}

fn border_label(border: SubtitleBorderStyle, t: &Translations) -> &'static str {
    match border {
        SubtitleBorderStyle::None => t.style_border_none,
        SubtitleBorderStyle::Thin => t.style_border_thin,
        SubtitleBorderStyle::Medium => t.style_border_medium,
        SubtitleBorderStyle::Thick => t.style_border_thick,
    }
}

fn background_label(style: &SubtitleStyle, t: &Translations) -> &'static str {
    match style.background {
        SubtitleBackgroundStyle::None => t.style_bg_none,
        _ if style.background_opacity <= 60 => t.style_bg_box_50,
        _ if style.background_opacity <= 85 => t.style_bg_box_75,
        _ => t.style_bg_box_solid,
    }
}

fn shadow_label(shadow: SubtitleShadowStyle, t: &Translations) -> &'static str {
    match shadow {
        SubtitleShadowStyle::None => t.style_shadow_none,
        SubtitleShadowStyle::Light => t.style_shadow_light,
        SubtitleShadowStyle::Strong => t.style_shadow_strong,
    }
}

fn position_label(style: &SubtitleStyle, t: &Translations) -> Cow<'static, str> {
    match style.bottom_offset_percent {
        8 => t.style_pos_bottom.into(),
        20 => t.style_pos_raised.into(),
        82 => t.style_pos_top.into(),
        other => format!("{other}%").into(),
    }
}
