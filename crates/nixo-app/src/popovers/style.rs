//! Design tokens do menu de configurações.
//!
//! Todas as cores, dimensões e fontes usadas pelos componentes de
//! `popovers::components` são definidas aqui. Para restilizar o menu inteiro,
//! altere apenas [`MenuStyle::nixo_dark`].

use std::sync::LazyLock;

use egui::{Color32, CornerRadius, FontId, Margin, Shadow, Stroke, Vec2, vec2};

use crate::app::ACCENT_COLOR;
use crate::theme::font_bold;

/// Par de cores (preenchimento + contorno) de um badge de ícone.
#[derive(Clone, Copy)]
pub(crate) struct BadgeColors {
    pub fill: Color32,
    pub stroke: Stroke,
}

/// Conjunto completo de tokens visuais do menu de configurações.
pub(crate) struct MenuStyle {
    // ── Container (popover) ──
    pub popover_width: f32,
    pub popover_margin_left: f32,
    pub popover_margin_right: f32,
    pub popover_anchor_gap: f32,
    pub popover_fill: Color32,
    pub popover_stroke: Stroke,
    pub popover_radius: CornerRadius,
    pub popover_shadow: Shadow,
    pub popover_padding: Margin,
    pub item_spacing_y: f32,
    pub height_morph_duration: f32,

    // ── Linhas (full-rounded / pílula) ──
    pub row_radius: CornerRadius,
    pub row_hover_fill: Color32,
    pub row_selected_fill: Color32,
    pub row_padding_x: f32,
    pub option_height: f32,
    pub nav_height: f32,
    pub header_height: f32,
    pub section_title_height: f32,

    // ── Texto ──
    pub text_hover: Color32,
    pub text_option: Color32,
    pub text_nav_title: Color32,
    pub text_nav_value: Color32,
    pub text_header: Color32,
    pub text_section_title: Color32,
    pub text_muted: Color32,
    pub font_option: FontId,
    pub font_option_selected: FontId,
    pub font_option_emphasis: FontId,
    pub font_label: FontId,
    pub font_value: FontId,
    pub font_muted: FontId,

    // ── Opção: marcador de seleção, ícone e texto ──
    pub check_offset_x: f32,
    pub check_stroke: Stroke,
    pub icon_offset_x: f32,
    pub text_offset_x: f32,
    pub text_offset_x_with_icon: f32,

    // ── Chevrons ──
    pub chevron_inset: f32,
    pub chevron_size: f32,
    pub chevron_stroke_width: f32,
    pub chevron_idle: Color32,
    pub nav_value_gap: f32,
    pub header_text_gap: f32,

    // ── Ícones ──
    pub badge_size: Vec2,
    pub action_badge_size: Vec2,
    pub badge_radius: CornerRadius,
    pub action_badge_radius: CornerRadius,
    pub badge_neutral: BadgeColors,
    pub badge_subtle: BadgeColors,
    pub badge_accent: BadgeColors,
    pub icon_fg: Color32,
    pub icon_accent: Color32,
    pub icon_glyph_stroke: f32,
    pub icon_circle_radius: f32,
    pub icon_disabled_fill: Color32,
    pub icon_disabled_stroke: Stroke,
    pub icon_swatch_ring: Stroke,
    pub font_icon_glyph: FontId,

    // ── Divisor ──
    pub divider_height: f32,
    pub divider_inset: f32,
    pub divider_stroke: Stroke,

    // ── Banner de aviso (sem borda e sem ícone) ──
    pub warning_fill: Color32,
    pub warning_text: Color32,
    pub warning_radius: CornerRadius,
    pub warning_padding: Margin,
    pub warning_margin: Margin,
    pub warning_font: FontId,

    // ── Botão centralizado (full-rounded, sem ícone) ──
    pub button_height: f32,
    pub button_inset_x: f32,
    pub button_margin_y: f32,
    pub button_radius: CornerRadius,
    pub button_fill: Color32,
    pub button_fill_hover: Color32,
    pub button_fill_active: Color32,
    pub button_text: Color32,
    pub button_font: FontId,
}

impl MenuStyle {
    /// Tema escuro OLED full-black do Nixo.
    fn nixo_dark() -> Self {
        Self {
            popover_width: 356.0,
            popover_margin_left: 16.0,
            popover_margin_right: 18.0,
            popover_anchor_gap: 10.0,
            popover_fill: Color32::BLACK,
            popover_stroke: Stroke::new(1.0_f32, Color32::from_rgb(48, 48, 54)),
            popover_radius: CornerRadius::same(16),
            popover_shadow: Shadow {
                offset: [0, 14],
                blur: 32,
                spread: 0,
                color: Color32::from_black_alpha(200),
            },
            popover_padding: Margin::symmetric(12, 11),
            item_spacing_y: 3.0,
            height_morph_duration: 0.18,

            row_radius: CornerRadius::same(99),
            row_hover_fill: Color32::from_rgb(34, 34, 40),
            row_selected_fill: Color32::from_rgb(22, 22, 27),
            row_padding_x: 14.0,
            option_height: 37.0,
            nav_height: 38.0,
            header_height: 34.0,
            section_title_height: 26.0,

            text_hover: Color32::WHITE,
            text_option: Color32::from_rgb(195, 195, 202),
            text_nav_title: Color32::from_rgb(150, 150, 158),
            text_nav_value: Color32::from_rgb(225, 225, 230),
            text_header: Color32::from_rgb(215, 215, 220),
            text_section_title: Color32::from_rgb(160, 160, 168),
            text_muted: Color32::from_gray(150),
            font_option: FontId::proportional(14.0),
            font_option_selected: font_bold(14.0),
            font_option_emphasis: font_bold(13.5),
            font_label: font_bold(12.0),
            font_value: FontId::proportional(13.5),
            font_muted: FontId::proportional(13.0),

            check_offset_x: 18.0,
            check_stroke: Stroke::new(2.0_f32, ACCENT_COLOR),
            icon_offset_x: 43.0,
            text_offset_x: 38.0,
            text_offset_x_with_icon: 66.0,

            chevron_inset: 12.0,
            chevron_size: 4.5,
            chevron_stroke_width: 1.7,
            chevron_idle: Color32::from_rgb(115, 115, 125),
            nav_value_gap: 12.0,
            header_text_gap: 12.0,

            badge_size: vec2(26.0, 18.0),
            action_badge_size: vec2(28.0, 20.0),
            badge_radius: CornerRadius::same(4),
            action_badge_radius: CornerRadius::same(6),
            badge_neutral: BadgeColors {
                fill: Color32::from_rgb(26, 26, 32),
                stroke: Stroke::new(1.2_f32, Color32::from_rgb(68, 68, 80)),
            },
            badge_subtle: BadgeColors {
                fill: Color32::from_rgb(26, 26, 32),
                stroke: Stroke::new(1.2_f32, Color32::from_rgb(72, 72, 86)),
            },
            badge_accent: BadgeColors {
                fill: Color32::from_rgb(14, 42, 12),
                stroke: Stroke::new(1.2_f32, Color32::from_rgb(44, 190, 16)),
            },
            icon_fg: Color32::WHITE,
            icon_accent: ACCENT_COLOR,
            icon_glyph_stroke: 1.85,
            icon_circle_radius: 8.5,
            icon_disabled_fill: Color32::from_rgb(22, 22, 27),
            icon_disabled_stroke: Stroke::new(1.75_f32, Color32::from_rgb(180, 180, 192)),
            icon_swatch_ring: Stroke::new(1.2_f32, Color32::from_rgb(90, 90, 102)),
            font_icon_glyph: font_bold(12.5),

            divider_height: 9.0,
            divider_inset: 4.0,
            divider_stroke: Stroke::new(1.0_f32, Color32::from_rgb(48, 48, 54)),

            warning_fill: Color32::from_rgb(64, 15, 17),
            warning_text: Color32::from_rgb(242, 195, 195),
            warning_radius: CornerRadius::same(10),
            warning_padding: Margin::symmetric(12, 9),
            warning_margin: Margin::symmetric(4, 4),
            warning_font: FontId::proportional(12.5),

            button_height: 38.0,
            button_inset_x: 4.0,
            button_margin_y: 2.0,
            button_radius: CornerRadius::same(99),
            button_fill: Color32::from_rgb(26, 26, 31),
            button_fill_hover: Color32::from_rgb(40, 40, 48),
            button_fill_active: Color32::from_rgb(50, 50, 60),
            button_text: Color32::from_rgb(225, 225, 232),
            button_font: font_bold(13.5),
        }
    }

    /// Altura externa exata da aba raiz (`SettingsTab::Main`), usada para inicializar o morph sem salto no primeiro frame.
    pub(crate) fn initial_main_height(&self) -> f32 {
        let inner_h = self.section_title_height
            + 2.0 * self.divider_height
            + 5.0 * self.nav_height
            + 6.0 * self.item_spacing_y;
        let pad_y = f32::from(self.popover_padding.top) + f32::from(self.popover_padding.bottom);
        inner_h + pad_y + 2.0 * self.popover_stroke.width
    }
}

/// Retorna os tokens ativos do menu.
pub(crate) fn menu_style() -> &'static MenuStyle {
    static STYLE: LazyLock<MenuStyle> = LazyLock::new(MenuStyle::nixo_dark);
    &STYLE
}
