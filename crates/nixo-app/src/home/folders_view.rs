//! Visão das pastas de mídia favoritas adicionadas à biblioteca.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;

/// Renderiza a lista de pastas de mídia configuradas pelo usuário.
pub fn render_folders_view(
    ui: &mut Ui,
    folders: &[PathBuf],
    folder_counts: &HashMap<PathBuf, usize>,
) -> HomeAction {
    let mut action = HomeAction::None;

    if folders.is_empty() {
        let resp = render_empty_state(
            ui,
            lucide::FOLDER_OPEN,
            "Nenhuma pasta de mídia adicionada",
            "Adicione suas pastas de vídeos ou animes para navegar e assistir diretamente por aqui.",
            Some("+ Selecionar Pasta de Mídia"),
        );
        if let Some(r) = resp
            && r.clicked()
        {
            action = HomeAction::AddFolder;
        }
    } else {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Pastas de Mídia")
                .font(font_section_title())
                .color(Color32::WHITE),
        );
        ui.add_space(14.0);

        let card_w = 310.0;
        let card_h = 120.0;
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

                    // Transição suave de hover do card de pasta (150ms)
                    let hover_t = ui.ctx().animate_bool_with_time(
                        resp.id.with("folder_hover"),
                        hovered,
                        0.15,
                    );
                    let bg = lerp_color(SURFACE_LOW, SURFACE_HIGH, hover_t);
                    let icon_color = lerp_color(TEXT_SECONDARY, Color32::WHITE, hover_t);
                    let title_color = lerp_color(TEXT_PRIMARY, Color32::WHITE, hover_t);

                    ui.painter().rect_filled(rect, RADIUS_MD, bg);

                    // Ícone Lucide da pasta sem wrapper
                    let icon_pos = Pos2::new(rect.left() + 20.0, rect.top() + 32.0);
                    ui.painter().text(
                        icon_pos,
                        Align2::LEFT_CENTER,
                        lucide::FOLDER,
                        lucide::font_id(26.0),
                        icon_color,
                    );

                    // Botão sutil de remover pasta
                    let remove_rect = Rect::from_min_size(
                        Pos2::new(rect.right() - 32.0, rect.top() + 12.0),
                        vec2(22.0, 22.0),
                    );
                    let rem_resp = ui
                        .allocate_rect(remove_rect, Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text("Remover pasta da biblioteca");

                    let rem_hover_t = ui.ctx().animate_bool_with_time(
                        rem_resp.id.with("rem_hover"),
                        rem_resp.hovered(),
                        0.12,
                    );
                    if rem_hover_t > 0.0 {
                        let rem_bg = lerp_color(Color32::TRANSPARENT, DANGER_SURFACE, rem_hover_t);
                        ui.painter().rect_filled(remove_rect, RADIUS_SM, rem_bg);
                    }
                    let rem_fg = lerp_color(TEXT_MUTED, DANGER_TEXT, rem_hover_t);
                    ui.painter().text(
                        remove_rect.center(),
                        Align2::CENTER_CENTER,
                        lucide::X,
                        lucide::font_id(13.5),
                        rem_fg,
                    );

                    if rem_resp.clicked() {
                        action = HomeAction::RemoveFolder(folder.clone());
                    } else if resp.clicked() {
                        action = HomeAction::OpenFolder(folder.clone());
                    }

                    // Nome da pasta
                    ui.painter().text(
                        Pos2::new(rect.left() + 56.0, rect.top() + 23.0),
                        Align2::LEFT_CENTER,
                        folder_name,
                        font_card_title(),
                        title_color,
                    );

                    // Caminho da pasta
                    let path_str = folder.to_string_lossy();
                    let display_path = if path_str.len() > 34 {
                        format!("...{}", &path_str[path_str.len() - 31..])
                    } else {
                        path_str.to_string()
                    };

                    ui.painter().text(
                        Pos2::new(rect.left() + 56.0, rect.top() + 44.0),
                        Align2::LEFT_CENTER,
                        display_path,
                        font_xs(),
                        TEXT_MUTED,
                    );

                    // Contador de itens
                    ui.painter().text(
                        Pos2::new(rect.left() + 20.0, rect.top() + 92.0),
                        Align2::LEFT_CENTER,
                        count_text,
                        font_sm(),
                        TEXT_SECONDARY,
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
