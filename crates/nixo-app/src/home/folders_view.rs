//! Visão das pastas de mídia favoritas adicionadas à biblioteca.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, Vec2, vec2,
};

use super::header::HomeAction;
use crate::app::ACCENT_COLOR;

/// Renderiza a lista de pastas de mídia configuradas pelo usuário.
pub fn render_folders_view(
    ui: &mut Ui,
    folders: &[PathBuf],
    folder_counts: &HashMap<PathBuf, usize>,
) -> HomeAction {
    let mut action = HomeAction::None;

    if folders.is_empty() {
        let avail_rect = ui.available_rect_before_wrap();
        let card_size = Vec2::new(480.0, 240.0).min(avail_rect.size() * 0.9);
        let card_rect = Rect::from_center_size(avail_rect.center(), card_size);

        ui.painter().rect_filled(
            card_rect,
            CornerRadius::same(16),
            Color32::from_rgb(20, 22, 28),
        );
        ui.painter().rect_stroke(
            card_rect,
            CornerRadius::same(16),
            Stroke::new(1.0_f32, Color32::from_rgb(44, 48, 60)),
            StrokeKind::Inside,
        );

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(card_rect.shrink(24.0))
                .layout(Layout::top_down(Align::Center)),
        );

        child_ui.add_space(10.0);
        child_ui.label(
            RichText::new("📁 Nenhuma pasta de mídia adicionada")
                .font(FontId::proportional(16.0))
                .strong()
                .color(Color32::WHITE),
        );
        child_ui.add_space(8.0);
        child_ui.label(
            RichText::new("Adicione suas pastas de vídeos ou animes para navegar e assistir diretamente por aqui.")
                .font(FontId::proportional(13.5))
                .color(Color32::from_gray(170)),
        );
        child_ui.add_space(20.0);

        let add_btn = egui::Button::new(
            RichText::new("+ Selecionar Pasta de Mídia")
                .font(FontId::proportional(14.0))
                .strong()
                .color(Color32::WHITE),
        )
        .fill(ACCENT_COLOR)
        .corner_radius(CornerRadius::same(10));

        if child_ui.add_sized([240.0, 40.0], add_btn).clicked() {
            action = HomeAction::AddFolder;
        }

        child_ui.add_space(12.0);
        child_ui.label(
            RichText::new("ou arraste e solte um arquivo de mídia nesta janela")
                .font(FontId::proportional(12.5))
                .color(Color32::from_gray(130)),
        );
    } else {
        ui.add_space(8.0);
        ui.label(
            RichText::new("SUAS PASTAS DE MÍDIA")
                .font(FontId::proportional(12.0))
                .strong()
                .color(Color32::from_gray(140)),
        );
        ui.add_space(12.0);

        let card_w = 260.0;
        let card_h = 100.0;
        let spacing = 14.0;
        let avail_w = ui.available_width().max(card_w);
        let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("folders_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, folder) in folders.iter().enumerate() {
                    let folder_name = folder
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Pasta");

                    let count_text = match folder_counts.get(folder) {
                        Some(c) => format!("{c} itens"),
                        None => "calculando...".to_string(),
                    };

                    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
                    let hovered = resp.hovered();

                    let bg = if hovered {
                        Color32::from_rgb(32, 35, 46)
                    } else {
                        Color32::from_rgb(20, 22, 29)
                    };
                    let border_st = if hovered {
                        Stroke::new(1.2_f32, ACCENT_COLOR)
                    } else {
                        Stroke::new(1.0_f32, Color32::from_rgb(42, 45, 56))
                    };

                    ui.painter().rect_filled(rect, CornerRadius::same(12), bg);
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(12),
                        border_st,
                        StrokeKind::Inside,
                    );

                    let icon_center = Pos2::new(rect.left() + 26.0, rect.top() + 26.0);
                    paint_folder_icon(ui, icon_center, ACCENT_COLOR);

                    let remove_rect = Rect::from_min_size(
                        Pos2::new(rect.right() - 26.0, rect.top() + 8.0),
                        vec2(18.0, 18.0),
                    );
                    let rem_resp = ui
                        .allocate_rect(remove_rect, Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text("Remover pasta da biblioteca");

                    if rem_resp.hovered() {
                        ui.painter().circle_filled(
                            remove_rect.center(),
                            9.0,
                            Color32::from_rgb(60, 25, 25),
                        );
                        ui.painter().text(
                            remove_rect.center(),
                            Align2::CENTER_CENTER,
                            "×",
                            FontId::proportional(13.0),
                            Color32::from_rgb(255, 120, 120),
                        );
                    } else {
                        ui.painter().text(
                            remove_rect.center(),
                            Align2::CENTER_CENTER,
                            "×",
                            FontId::proportional(12.0),
                            Color32::from_gray(120),
                        );
                    }

                    if rem_resp.clicked() {
                        action = HomeAction::RemoveFolder(folder.clone());
                    } else if resp.clicked() {
                        action = HomeAction::OpenFolder(folder.clone());
                    }

                    ui.painter().text(
                        Pos2::new(rect.left() + 50.0, rect.top() + 20.0),
                        Align2::LEFT_CENTER,
                        folder_name,
                        FontId::proportional(14.5),
                        Color32::WHITE,
                    );

                    let path_str = folder.to_string_lossy();
                    let display_path = if path_str.len() > 34 {
                        format!("...{}", &path_str[path_str.len() - 31..])
                    } else {
                        path_str.to_string()
                    };

                    ui.painter().text(
                        Pos2::new(rect.left() + 14.0, rect.top() + 56.0),
                        Align2::LEFT_CENTER,
                        display_path,
                        FontId::proportional(11.5),
                        Color32::from_gray(140),
                    );

                    ui.painter().text(
                        Pos2::new(rect.left() + 14.0, rect.top() + 78.0),
                        Align2::LEFT_CENTER,
                        count_text,
                        FontId::proportional(11.5),
                        ACCENT_COLOR,
                    );

                    resp.on_hover_cursor(CursorIcon::PointingHand);

                    if (idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    action
}

fn paint_folder_icon(ui: &Ui, center: Pos2, color: Color32) {
    let scale = 0.85_f32;
    let p =
        |x: f32, y: f32| Pos2::new(center.x + (x - 12.0) * scale, center.y + (y - 12.0) * scale);
    let stroke = Stroke::new(1.6_f32, color);
    let pts = vec![
        p(2.5, 7.5),
        p(9.0, 7.5),
        p(11.0, 10.0),
        p(21.5, 10.0),
        p(21.5, 19.5),
        p(2.5, 19.5),
    ];
    ui.painter().add(egui::Shape::closed_line(pts, stroke));
}
