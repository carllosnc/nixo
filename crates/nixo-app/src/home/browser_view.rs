//! Visão detalhada de dentro de uma pasta: subpastas e grid de arquivos executáveis.

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, Vec2, vec2,
};

use super::header::HomeAction;
use super::scanner::{FolderContent, MediaKind};
use crate::app::ACCENT_COLOR;

/// Renderiza o conteúdo da pasta aberta (subpastas e arquivos de mídia executáveis).
pub fn render_folder_browser(ui: &mut Ui, content: &FolderContent) -> HomeAction {
    let mut action = HomeAction::None;

    if content.is_loading {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.add_space(10.0);
            ui.label(
                RichText::new("Carregando itens da pasta...")
                    .font(FontId::proportional(14.0))
                    .color(Color32::from_gray(180)),
            );
        });
        return action;
    }

    // Seção de Subpastas (se houver)
    if !content.subdirs.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("SUBPASTAS")
                .font(FontId::proportional(11.5))
                .strong()
                .color(Color32::from_gray(135)),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            for dir in &content.subdirs {
                let btn = egui::Button::new(
                    RichText::new(format!("📁 {}", dir.name))
                        .font(FontId::proportional(13.0))
                        .color(Color32::from_gray(230)),
                )
                .fill(Color32::from_rgb(26, 28, 36))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(46, 50, 62)))
                .corner_radius(CornerRadius::same(8));

                if ui
                    .add(btn)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    action = HomeAction::OpenFolder(dir.path.clone());
                }
            }
        });
        ui.add_space(14.0);
    }

    // Seção de Arquivos de Mídia
    ui.label(
        RichText::new(format!("ARQUIVOS EXECUTÁVEIS ({})", content.files.len()))
            .font(FontId::proportional(11.5))
            .strong()
            .color(Color32::from_gray(135)),
    );
    ui.add_space(10.0);

    if content.files.is_empty() {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            let box_size = Vec2::new(420.0, 90.0);
            let (empty_box, _) = ui.allocate_exact_size(box_size, Sense::hover());
            ui.painter().rect_filled(
                empty_box,
                CornerRadius::same(12),
                Color32::from_rgb(20, 22, 28),
            );
            ui.painter().rect_stroke(
                empty_box,
                CornerRadius::same(12),
                Stroke::new(1.0_f32, Color32::from_rgb(42, 45, 56)),
                StrokeKind::Inside,
            );
            ui.painter().text(
                empty_box.center(),
                Align2::CENTER_CENTER,
                "Nenhum arquivo executável de mídia encontrado nesta pasta.",
                FontId::proportional(13.0),
                Color32::from_gray(180),
            );
        });
    } else {
        let card_w = 210.0;
        let card_h = 135.0;
        let spacing = 12.0;
        let avail_w = ui.available_width().max(card_w);
        let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("files_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, item) in content.files.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
                    let hovered = resp.hovered();

                    let bg = if hovered {
                        Color32::from_rgb(30, 33, 42)
                    } else {
                        Color32::from_rgb(18, 20, 26)
                    };
                    let border_st = if hovered {
                        Stroke::new(1.4_f32, ACCENT_COLOR)
                    } else {
                        Stroke::new(1.0_f32, Color32::from_rgb(38, 41, 52))
                    };

                    ui.painter().rect_filled(rect, CornerRadius::same(10), bg);
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(10),
                        border_st,
                        StrokeKind::Inside,
                    );

                    let thumb_h = 80.0;
                    let thumb_rect = Rect::from_min_size(rect.min, vec2(card_w, thumb_h));
                    ui.painter().rect_filled(
                        thumb_rect,
                        CornerRadius {
                            nw: 10,
                            ne: 10,
                            sw: 0,
                            se: 0,
                        },
                        Color32::from_rgb(24, 26, 34),
                    );

                    let center = thumb_rect.center();
                    paint_media_icon(ui, center, item.kind, hovered);

                    let badge_rect = Rect::from_min_size(
                        Pos2::new(thumb_rect.right() - 44.0, thumb_rect.top() + 6.0),
                        vec2(38.0, 16.0),
                    );
                    ui.painter().rect_filled(
                        badge_rect,
                        CornerRadius::same(4),
                        Color32::from_black_alpha(180),
                    );
                    ui.painter().text(
                        badge_rect.center(),
                        Align2::CENTER_CENTER,
                        &item.extension,
                        FontId::proportional(10.0),
                        Color32::from_gray(210),
                    );

                    let clean_name = item
                        .path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&item.name);

                    let display_name = if clean_name.len() > 24 {
                        format!("{}...", &clean_name[..22])
                    } else {
                        clean_name.to_string()
                    };

                    ui.painter().text(
                        Pos2::new(rect.left() + 10.0, rect.top() + thumb_h + 16.0),
                        Align2::LEFT_CENTER,
                        display_name,
                        FontId::proportional(12.5),
                        if hovered {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(220)
                        },
                    );

                    ui.painter().text(
                        Pos2::new(rect.left() + 10.0, rect.top() + thumb_h + 34.0),
                        Align2::LEFT_CENTER,
                        &item.size_formatted,
                        FontId::proportional(11.0),
                        Color32::from_gray(140),
                    );

                    if resp.clicked() {
                        action = HomeAction::PlayFile(item.path.clone());
                    }

                    resp.on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(&item.name);

                    if (idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    action
}

fn paint_media_icon(ui: &Ui, center: Pos2, kind: MediaKind, hovered: bool) {
    let fg = if hovered {
        ACCENT_COLOR
    } else {
        Color32::from_gray(150)
    };

    let scale = 0.88_f32;
    let p =
        |x: f32, y: f32| Pos2::new(center.x + (x - 12.0) * scale, center.y + (y - 12.0) * scale);

    match kind {
        MediaKind::Video => {
            let pts = vec![p(7.0, 6.0), p(18.5, 12.0), p(7.0, 18.0)];
            ui.painter().add(egui::Shape::convex_polygon(
                pts,
                fg,
                Stroke::new(1.5_f32, fg),
            ));
        }
        MediaKind::Audio | MediaKind::Directory => {
            ui.painter()
                .circle_stroke(center, 12.0 * scale, Stroke::new(1.6_f32, fg));
        }
    }
}
