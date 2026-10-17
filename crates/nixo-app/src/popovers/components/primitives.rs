//! Primitivas de pintura compartilhadas por todos os componentes do menu.
//!
//! Nenhum componente desenha hover, chevron, marcador ou texto por conta
//! própria: todos passam por estas funções, garantindo uma única implementação
//! de cada elemento visual.

use std::sync::Arc;

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Align2, Color32, FontId, Galley, Pos2, Rect, Response, Sense, Stroke, Ui, vec2};

use crate::popovers::style::menu_style;

/// Direção para onde a ponta do chevron aponta.
#[derive(Clone, Copy)]
pub(crate) enum ChevronDir {
    Left,
    Right,
}

/// Reserva uma linha com a largura disponível e a altura informada.
pub(crate) fn allocate_row(ui: &mut Ui, height: f32, sense: Sense) -> (Rect, Response) {
    ui.allocate_exact_size(vec2(ui.available_width(), height), sense)
}

/// Pinta o fundo de hover padrão de uma linha clicável.
pub(crate) fn paint_row_hover(ui: &Ui, rect: Rect, hovered: bool) {
    if hovered {
        let st = menu_style();
        ui.painter()
            .rect_filled(rect, st.row_radius, st.row_hover_fill);
    }
}

/// Retorna a cor de destaque em hover ou a cor em repouso.
pub(crate) fn hover_color(hovered: bool, idle: Color32) -> Color32 {
    if hovered {
        menu_style().text_hover
    } else {
        idle
    }
}

/// Pinta um chevron (`<` ou `>`) com a ponta em `tip` usando linha contínua (vértice limpo).
pub(crate) fn paint_chevron(ui: &Ui, tip: Pos2, dir: ChevronDir, color: Color32) {
    let st = menu_style();
    let s = st.chevron_size;
    let dx = match dir {
        ChevronDir::Right => -s,
        ChevronDir::Left => s,
    };
    let stroke = Stroke::new(st.chevron_stroke_width, color);
    ui.painter().add(egui::Shape::line(
        vec![
            Pos2::new(tip.x + dx, tip.y - s),
            tip,
            Pos2::new(tip.x + dx, tip.y + s),
        ],
        stroke,
    ));
}

/// Pinta o marcador de seleção (✓) centrado em `center` com vértice contínuo.
pub(crate) fn paint_checkmark(ui: &Ui, center: Pos2) {
    let stroke = menu_style().check_stroke;
    let (x, y) = (center.x, center.y);
    ui.painter().add(egui::Shape::line(
        vec![
            Pos2::new(x - 4.8, y + 0.2),
            Pos2::new(x - 1.4, y + 3.6),
            Pos2::new(x + 5.2, y - 3.8),
        ],
        stroke,
    ));
}

/// Pinta o indicador de botão de rádio (`○` inativo ou `⦿` ativo em ACCENT_COLOR) centrado em `center`.
pub(crate) fn paint_radio_button(ui: &Ui, center: Pos2, selected: bool, hovered: bool) {
    let r_outer = 5.8;
    if selected {
        let accent = menu_style().icon_accent;
        ui.painter()
            .circle_stroke(center, r_outer, Stroke::new(1.85_f32, accent));
        ui.painter().circle_filled(center, 2.8, accent);
    } else {
        let ring_color = if hovered {
            Color32::from_rgb(145, 145, 155)
        } else {
            Color32::from_rgb(72, 72, 80)
        };
        ui.painter()
            .circle_stroke(center, r_outer, Stroke::new(1.35_f32, ring_color));
    }
}

/// Faz o layout de uma linha única de texto, truncando com `…` pela largura real.
pub(crate) fn layout_single_line(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), TextFormat::simple(font, color));
    job.wrap = TextWrapping {
        max_width: max_width.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.fonts(|f| f.layout_job(job))
}

/// Pinta um galley já formatado alinhado em relação a `anchor`.
pub(crate) fn paint_galley(ui: &Ui, galley: Arc<Galley>, anchor: Pos2, align: Align2) {
    let rect = align.anchor_size(anchor, galley.size());
    ui.painter().galley(rect.min, galley, Color32::PLACEHOLDER);
}

/// Atalho: layout + pintura de uma linha de texto truncada.
pub(crate) fn paint_text(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    anchor: Pos2,
    align: Align2,
    max_width: f32,
) -> Rect {
    let galley = layout_single_line(ui, text, font, color, max_width);
    let rect = align.anchor_size(anchor, galley.size());
    paint_galley(ui, galley, anchor, align);
    rect
}
