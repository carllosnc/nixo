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
    Choice, LeadingIcon, MenuButton, MenuDivider, NavRow, WarningBanner, choice_list,
    submenu_header,
};
use crate::app::{NixoApp, SettingsTab};

const SIZE_PRESETS: [u16; 6] = [50, 75, 100, 150, 200, 300];
const BACKGROUND_PRESETS: [(&str, SubtitleBackgroundStyle, u8); 4] = [
    ("Nenhum", SubtitleBackgroundStyle::None, 60),
    ("Caixa 50%", SubtitleBackgroundStyle::Box, 50),
    ("Caixa 75%", SubtitleBackgroundStyle::Box, 75),
    ("Caixa sólida", SubtitleBackgroundStyle::Box, 100),
];
const POSITION_PRESETS: [(&str, u8); 3] = [("Inferior", 8), ("Elevada", 20), ("Superior", 82)];
const PREVIEW_FONT_SIZE: f32 = 14.0;

impl NixoApp {
    pub(super) fn render_subtitle_style_tabs(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let style = &self.status.subtitle_style;
        match self.settings_tab {
            SettingsTab::SubtitleStyle => self.render_style_overview(ui, actions),
            SettingsTab::SubtitleFont => font_picker(ui, actions, style),
            SettingsTab::SubtitleWeight => weight_picker(ui, actions, style),
            SettingsTab::SubtitleSize => size_picker(ui, actions, style),
            SettingsTab::SubtitleColor => color_picker(ui, actions, style),
            SettingsTab::SubtitleBorder => border_picker(ui, actions, style),
            SettingsTab::SubtitleBackground => background_picker(ui, actions, style),
            SettingsTab::SubtitleShadow => shadow_picker(ui, actions, style),
            SettingsTab::SubtitlePosition => position_picker(ui, actions, style),
            _ => {}
        }
    }

    fn render_style_overview(&self, ui: &mut Ui, actions: &mut MenuActions) {
        if submenu_header(ui, "ESTILO DA LEGENDA") {
            actions.navigate(SettingsTab::Subtitles);
        }

        if let Some(warning) = self.status.subtitle_health.warning_message() {
            ui.add(WarningBanner::new(&warning));
        }

        let style = &self.status.subtitle_style;
        let weight = if style.bold { "Bold" } else { "Normal" };
        let rows: [(&str, Cow<str>, SettingsTab); 8] = [
            (
                "FONTE",
                Cow::Borrowed(&style.font_family),
                SettingsTab::SubtitleFont,
            ),
            ("PESO DA FONTE", weight.into(), SettingsTab::SubtitleWeight),
            (
                "TAMANHO",
                format!("{}%", style.scale_percent).into(),
                SettingsTab::SubtitleSize,
            ),
            (
                "COR DO TEXTO",
                color_label(style).into(),
                SettingsTab::SubtitleColor,
            ),
            (
                "CONTORNO",
                style.border.label().into(),
                SettingsTab::SubtitleBorder,
            ),
            (
                "FUNDO",
                background_label(style).into(),
                SettingsTab::SubtitleBackground,
            ),
            (
                "SOMBRA",
                style.shadow.label().into(),
                SettingsTab::SubtitleShadow,
            ),
            (
                "POSIÇÃO",
                position_label(style).into(),
                SettingsTab::SubtitlePosition,
            ),
        ];
        for (title, value, tab) in rows {
            if ui.add(NavRow::new(title, &value)).clicked() {
                actions.navigate(tab);
            }
        }

        ui.add(MenuDivider);
        if ui.add(MenuButton::new("Redefinir padrão")).clicked() {
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

fn font_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
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
    style_picker(ui, actions, "FONTE", style, choices, |s, name| {
        s.font_family = name.to_string();
    });
}

fn weight_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let ctx = ui.ctx().clone();
    let choices = [("Normal", false), ("Bold", true)].map(|(label, bold)| {
        Choice::new(label, bold)
            .selected(style.bold == bold)
            .font(NixoApp::subtitle_font_id(
                &ctx,
                &style.font_family,
                bold,
                PREVIEW_FONT_SIZE,
            ))
    });
    style_picker(ui, actions, "PESO DA FONTE", style, choices, |s, bold| {
        s.bold = bold;
    });
}

fn size_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = SIZE_PRESETS.map(|size| {
        let label = if size == 100 {
            "100% (padrão)".to_string()
        } else {
            format!("{size}%")
        };
        Choice::new(label, size).selected(style.scale_percent == size)
    });
    style_picker(ui, actions, "TAMANHO", style, choices, |s, size| {
        s.scale_percent = size;
    });
}

fn color_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = SubtitleStyle::COLOR_SWATCHES.map(|(name, rgba)| {
        Choice::new(name, rgba)
            .selected(same_rgb(style.text_color, rgba))
            .icon(LeadingIcon::ColorSwatch(rgba))
    });
    style_picker(ui, actions, "COR DO TEXTO", style, choices, |s, rgba| {
        s.text_color = rgba;
    });
}

fn border_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = SubtitleBorderStyle::ALL
        .map(|border| Choice::new(border.label(), border).selected(style.border == border));
    style_picker(ui, actions, "CONTORNO", style, choices, |s, border| {
        s.border = border;
    });
}

fn background_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = BACKGROUND_PRESETS.map(|(label, kind, opacity)| {
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
        "FUNDO",
        style,
        choices,
        |s, (kind, opacity)| {
            s.background = kind;
            s.background_opacity = opacity;
        },
    );
}

fn shadow_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = SubtitleShadowStyle::ALL
        .map(|shadow| Choice::new(shadow.label(), shadow).selected(style.shadow == shadow));
    style_picker(ui, actions, "SOMBRA", style, choices, |s, shadow| {
        s.shadow = shadow;
    });
}

fn position_picker(ui: &mut Ui, actions: &mut MenuActions, style: &SubtitleStyle) {
    let choices = POSITION_PRESETS.map(|(label, offset)| {
        Choice::new(label, offset).selected(style.bottom_offset_percent == offset)
    });
    style_picker(ui, actions, "POSIÇÃO", style, choices, |s, offset| {
        s.bottom_offset_percent = offset;
    });
}

fn same_rgb(a: [u8; 4], b: [u8; 4]) -> bool {
    a[..3] == b[..3]
}

fn color_label(style: &SubtitleStyle) -> &'static str {
    SubtitleStyle::COLOR_SWATCHES
        .iter()
        .find(|(_, rgba)| same_rgb(style.text_color, *rgba))
        .map_or("Personalizada", |(name, _)| *name)
}

fn background_label(style: &SubtitleStyle) -> &'static str {
    match style.background {
        SubtitleBackgroundStyle::None => "Nenhum",
        _ if style.background_opacity <= 60 => "Caixa 50%",
        _ if style.background_opacity <= 85 => "Caixa 75%",
        _ => "Caixa sólida",
    }
}

fn position_label(style: &SubtitleStyle) -> &'static str {
    match style.bottom_offset_percent {
        0..=12 => "Inferior",
        13..=35 => "Elevada",
        _ => "Superior",
    }
}
