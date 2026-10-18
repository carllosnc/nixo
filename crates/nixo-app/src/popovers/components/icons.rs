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
                paint_folder_plus_glyph(ui, c, fg);
            }
            Self::SubtitleStyle => {
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

fn paint_subtitle_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let stroke = Stroke::new(menu_style().icon_glyph_stroke, fg);
    let painter = ui.painter();
    // Lucide 'captions' dialogue line pairs
    // Linha superior (y = c.y - 2.5): curto + longo
    painter.line_segment(
        [
            Pos2::new(c.x - 6.5, c.y - 2.5),
            Pos2::new(c.x - 3.2, c.y - 2.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + 0.8, c.y - 2.5),
            Pos2::new(c.x + 6.5, c.y - 2.5),
        ],
        stroke,
    );
    // Linha inferior (y = c.y + 2.5): longo + curto
    painter.line_segment(
        [
            Pos2::new(c.x - 6.5, c.y + 2.5),
            Pos2::new(c.x + 0.5, c.y + 2.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + 4.2, c.y + 2.5),
            Pos2::new(c.x + 6.5, c.y + 2.5),
        ],
        stroke,
    );
}

fn paint_speaker_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let painter = ui.painter();
    // Lucide speaker body
    let body = vec![
        Pos2::new(c.x - 6.0, c.y - 2.2),
        Pos2::new(c.x - 3.0, c.y - 2.2),
        Pos2::new(c.x + 1.0, c.y - 5.5),
        Pos2::new(c.x + 1.0, c.y + 5.5),
        Pos2::new(c.x - 3.0, c.y + 2.2),
        Pos2::new(c.x - 6.0, c.y + 2.2),
    ];
    painter.add(Shape::convex_polygon(body, fg, Stroke::NONE));

    // Lucide sound wave arc
    let wave_stroke = Stroke::new(1.5_f32, fg);
    let arc_pts: Vec<Pos2> = (-3..=3)
        .map(|i| {
            let angle = (i as f32 / 3.0) * 0.78_f32;
            Pos2::new(c.x + 1.2 + 3.8 * angle.cos(), c.y + 3.8 * angle.sin())
        })
        .collect();
    painter.add(Shape::line(arc_pts, wave_stroke));
}

fn paint_folder_plus_glyph(ui: &Ui, c: Pos2, fg: Color32) {
    let scale = 0.82_f32;
    let p = |x: f32, y: f32| Pos2::new(c.x + (x - 12.0) * scale, c.y + (y - 12.0) * scale);
    let stroke = Stroke::new(1.65_f32, fg);
    let painter = ui.painter();

    // Lucide 'folder-plus': silhueta canônica de pasta
    let pts = vec![
        p(4.0, 4.0),
        p(8.0, 4.0),
        p(10.2, 6.5),
        p(20.0, 6.5),
        p(21.5, 8.0),
        p(21.5, 18.5),
        p(20.0, 20.0),
        p(4.0, 20.0),
        p(2.5, 18.5),
        p(2.5, 5.5),
    ];
    painter.add(Shape::closed_line(pts, stroke));

    // '+' centralizado
    let plus_stroke = Stroke::new(1.75_f32, fg);
    painter.line_segment([p(12.0, 10.0), p(12.0, 16.0)], plus_stroke);
    painter.line_segment([p(9.0, 13.0), p(15.0, 13.0)], plus_stroke);
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
