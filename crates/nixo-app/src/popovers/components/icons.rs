//! Ícones vetoriais à esquerda das opções do menu (`LeadingIcon`).
//!
//! Cada variante tem um único pintor dedicado. Cores e tamanhos vêm de
//! [`MenuStyle`](crate::popovers::style::MenuStyle); apenas a geometria fina de
//! cada glifo fica aqui.

use egui::{Align2, Color32, Pos2, Rect, Shape, Stroke, StrokeKind, Ui};
use nixo_core::FlagCode;

use crate::app::NixoApp;
use crate::popovers::style::{BadgeColors, menu_style};

/// Ícone exibido à esquerda do rótulo de uma [`MenuOption`](super::MenuOption).
#[derive(Clone, Copy, Default)]
pub(crate) enum LeadingIcon {
    #[default]
    None,
    Flag(FlagCode),
    GenericSubtitle,
    GenericAudio,
    Disabled,
    AddExternalSubtitle,
    SubtitleStyle,
    ColorSwatch([u8; 4]),
}

impl LeadingIcon {
    /// Ícone de faixa: bandeira do idioma ou o ícone genérico informado.
    pub(crate) fn for_track(flag: FlagCode, generic: Self) -> Self {
        match flag {
            FlagCode::Generic => generic,
            flag => Self::Flag(flag),
        }
    }

    pub(crate) fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// Pinta o ícone centrado em `c` usando a cor padrão de glifo.
    pub(crate) fn paint(self, ui: &Ui, c: Pos2) {
        let st = menu_style();
        let fg = st.icon_fg;
        match self {
            Self::None => {}
            Self::Flag(flag) => {
                NixoApp::render_vector_flag(ui.painter(), badge_rect(c), flag);
            }
            Self::GenericSubtitle => {
                paint_badge(ui, c, st.badge_neutral);
                paint_subtitle_glyph(ui, c, fg);
            }
            Self::GenericAudio => {
                paint_badge(ui, c, st.badge_neutral);
                paint_speaker_glyph(ui, c, fg);
            }
            Self::Disabled => paint_disabled(ui, c),
            Self::AddExternalSubtitle => {
                paint_action_badge(ui, c, st.badge_subtle);
                paint_plus_glyph(ui, c, fg);
            }
            Self::SubtitleStyle => {
                paint_action_badge(ui, c, st.badge_accent);
                ui.painter().text(
                    Pos2::new(c.x, c.y - 0.5),
                    Align2::CENTER_CENTER,
                    "Aa",
                    st.font_icon_glyph.clone(),
                    st.icon_accent,
                );
            }
            Self::ColorSwatch(rgba) => paint_swatch(ui, c, rgba),
        }
    }
}

fn badge_rect(c: Pos2) -> Rect {
    Rect::from_center_size(c, menu_style().badge_size)
}

fn paint_badge(ui: &Ui, c: Pos2, colors: BadgeColors) {
    let st = menu_style();
    let rect = badge_rect(c);
    ui.painter().rect_filled(rect, st.badge_radius, colors.fill);
    ui.painter()
        .rect_stroke(rect, st.badge_radius, colors.stroke, StrokeKind::Inside);
}

fn paint_action_badge(ui: &Ui, c: Pos2, colors: BadgeColors) {
    let st = menu_style();
    let rect = Rect::from_center_size(c, st.action_badge_size);
    ui.painter()
        .rect_filled(rect, st.action_badge_radius, colors.fill);
    ui.painter().rect_stroke(
        rect,
        st.action_badge_radius,
        colors.stroke,
        StrokeKind::Inside,
    );
}

fn paint_subtitle_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let stroke = Stroke::new(menu_style().icon_glyph_stroke, fg);
    let painter = ui.painter();
    // Linha superior: dois blocos de diálogo (estilo Closed Captions)
    painter.line_segment(
        [
            Pos2::new(c.x - 6.8, c.y - 2.6),
            Pos2::new(c.x - 1.4, c.y - 2.6),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + 1.4, c.y - 2.6),
            Pos2::new(c.x + 6.8, c.y - 2.6),
        ],
        stroke,
    );
    // Linha inferior: bloco longo + bloco curto
    painter.line_segment(
        [
            Pos2::new(c.x - 6.8, c.y + 2.6),
            Pos2::new(c.x + 2.2, c.y + 2.6),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + 4.6, c.y + 2.6),
            Pos2::new(c.x + 6.8, c.y + 2.6),
        ],
        stroke,
    );
}

fn paint_speaker_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let painter = ui.painter();
    let body = vec![
        Pos2::new(c.x - 5.8, c.y - 2.3),
        Pos2::new(c.x - 2.8, c.y - 2.3),
        Pos2::new(c.x + 0.5, c.y - 5.2),
        Pos2::new(c.x + 0.5, c.y + 5.2),
        Pos2::new(c.x - 2.8, c.y + 2.3),
        Pos2::new(c.x - 5.8, c.y + 2.3),
    ];
    painter.add(Shape::convex_polygon(body, fg, Stroke::NONE));

    // Onda sonora suave em arco contínuo
    let wave_stroke = Stroke::new(1.5_f32, fg);
    let arc_pts: Vec<Pos2> = (-3..=3)
        .map(|i| {
            let angle = (i as f32 / 3.0) * 0.72_f32;
            Pos2::new(c.x + 1.4 + 3.6 * angle.cos(), c.y + 3.6 * angle.sin())
        })
        .collect();
    painter.add(Shape::line(arc_pts, wave_stroke));
}

fn paint_plus_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let stroke = Stroke::new(2.0_f32, fg);
    let arm = 4.8_f32;
    ui.painter().line_segment(
        [Pos2::new(c.x, c.y - arm), Pos2::new(c.x, c.y + arm)],
        stroke,
    );
    ui.painter().line_segment(
        [Pos2::new(c.x - arm, c.y), Pos2::new(c.x + arm, c.y)],
        stroke,
    );
}

fn paint_disabled(ui: &Ui, c: Pos2) {
    let st = menu_style();
    let r = st.icon_circle_radius;
    ui.painter().circle_filled(c, r, st.icon_disabled_fill);
    ui.painter().circle_stroke(c, r, st.icon_disabled_stroke);
    let d = (r - 0.6) * std::f32::consts::FRAC_1_SQRT_2;
    ui.painter().line_segment(
        [Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)],
        st.icon_disabled_stroke,
    );
}

fn paint_swatch(ui: &Ui, c: Pos2, rgba: [u8; 4]) {
    let st = menu_style();
    let r = st.icon_circle_radius;
    ui.painter()
        .circle_filled(c, r, Color32::from_rgb(rgba[0], rgba[1], rgba[2]));
    ui.painter().circle_stroke(c, r, st.icon_swatch_ring);
}
