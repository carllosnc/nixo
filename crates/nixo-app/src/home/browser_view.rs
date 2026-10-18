//! Visão detalhada de dentro de uma pasta: subpastas e grid de arquivos executáveis.

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use super::scanner::FolderContent;
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;

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
                    .font(font_body())
                    .color(TEXT_SECONDARY),
            );
        });
        return action;
    }

    // Seção de Subpastas estruturada em grade limpa (sem tags desordenadas)
    if !content.subdirs.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Subpastas ({})", content.subdirs.len()))
                .font(font_section_title())
                .color(Color32::WHITE),
        );
        ui.add_space(12.0);

        let chip_w = 230.0;
        let chip_h = 42.0;
        let spacing = 10.0;
        let avail_w = ui.available_width().max(chip_w);
        let cols = ((avail_w + spacing) / (chip_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("subfolders_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, dir) in content.subdirs.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(chip_w, chip_h), Sense::click());
                    let hovered = resp.hovered();

                    let bg = if hovered { SURFACE_HIGH } else { SURFACE_LOW };

                    ui.painter().rect_filled(rect, RADIUS_MD, bg);

                    // Ícone Lucide de pasta sem wrapper
                    ui.painter().text(
                        Pos2::new(rect.left() + 14.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        lucide::FOLDER,
                        lucide::font_id(16.0),
                        if hovered {
                            Color32::WHITE
                        } else {
                            TEXT_SECONDARY
                        },
                    );

                    // Nome da subpasta truncado
                    let name_x = rect.left() + 38.0;
                    let max_chars = 24;
                    let display_name = if dir.name.len() > max_chars {
                        format!("{}...", &dir.name[..max_chars - 2])
                    } else {
                        dir.name.clone()
                    };

                    ui.painter().text(
                        Pos2::new(name_x, rect.center().y),
                        Align2::LEFT_CENTER,
                        display_name,
                        font_body(),
                        if hovered {
                            Color32::WHITE
                        } else {
                            TEXT_PRIMARY
                        },
                    );

                    if resp.clicked() {
                        action = HomeAction::OpenFolder(dir.path.clone());
                    }

                    resp.on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(&dir.name);

                    if (idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(16.0);
    }

    // Seção de Arquivos de Mídia (exibida apenas quando houver arquivos na pasta)
    if !content.files.is_empty() {
        if !content.subdirs.is_empty() {
            ui.label(
                RichText::new(format!("Arquivos ({})", content.files.len()))
                    .font(font_section_title())
                    .color(Color32::WHITE),
            );
            ui.add_space(12.0);
        }

        let card_w = 210.0;
        let card_h = 138.0;
        let spacing = 12.0;
        let avail_w = ui.available_width().max(card_w);
        let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("files_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, item) in content.files.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
                    let hovered = resp.hovered();

                    let bg = if hovered { SURFACE_HIGH } else { SURFACE_LOW };

                    ui.painter().rect_filled(rect, RADIUS_MD, bg);

                    // Área de thumbnail com cantos arredondados no topo
                    let thumb_h = 82.0;
                    let thumb_rect = Rect::from_min_size(rect.min, vec2(card_w, thumb_h));
                    ui.painter().rect_filled(
                        thumb_rect,
                        egui::CornerRadius {
                            nw: RADIUS_MD.nw,
                            ne: RADIUS_MD.ne,
                            sw: 0,
                            se: 0,
                        },
                        SURFACE_MED,
                    );

                    // Ícone central de Play / Mídia no Lucide
                    let center = thumb_rect.center();
                    let (circle_bg, icon_str, icon_fg) = if hovered {
                        (ACCENT_NEUTRAL_FILL, lucide::PLAY, ACCENT_NEUTRAL_TEXT)
                    } else {
                        let ic = match item.kind {
                            super::scanner::MediaKind::Video => lucide::FILM,
                            super::scanner::MediaKind::Audio => lucide::MUSIC,
                            super::scanner::MediaKind::Directory => lucide::FOLDER,
                        };
                        (SURFACE_LOW, ic, TEXT_SECONDARY)
                    };
                    ui.painter().circle_filled(center, 18.0, circle_bg);
                    ui.painter().text(
                        Pos2::new(center.x + if hovered { 1.0 } else { 0.0 }, center.y),
                        Align2::CENTER_CENTER,
                        icon_str,
                        lucide::font_id(16.0),
                        icon_fg,
                    );

                    // Badge com extensão (ex.: MP4, MKV)
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(thumb_rect.right() - 44.0, thumb_rect.top() + 6.0),
                        vec2(38.0, 16.0),
                    );
                    ui.painter()
                        .rect_filled(badge_rect, RADIUS_SM, Color32::from_black_alpha(190));
                    ui.painter().text(
                        badge_rect.center(),
                        Align2::CENTER_CENTER,
                        &item.extension,
                        font_xs(),
                        TEXT_SECONDARY,
                    );

                    // Nome do arquivo
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
                        font_body(),
                        if hovered {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        },
                    );

                    // Tamanho / Metadados
                    ui.painter().text(
                        Pos2::new(rect.left() + 10.0, rect.top() + thumb_h + 34.0),
                        Align2::LEFT_CENTER,
                        &item.size_formatted,
                        font_xs(),
                        TEXT_MUTED,
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
    } else if content.subdirs.is_empty() {
        render_empty_state(
            ui,
            lucide::FOLDER_OPEN,
            "Esta pasta está vazia",
            "Nenhum arquivo de mídia ou subpasta foi encontrado aqui.",
            None,
        );
    }

    action
}
