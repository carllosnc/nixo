//! Visão das pastas de mídia favoritas adicionadas à biblioteca.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use crate::app::ACCENT_COLOR;
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;

/// Formata o caminho de exibição da pasta para evitar redundâncias visuais na interface.
pub fn format_folder_display_path(folder: &Path) -> String {
    let path_str = folder.to_string_lossy();
    if let Some(parent) = folder.parent() {
        let p_str = parent.to_string_lossy();
        if p_str.ends_with(":\\") || p_str == "/" || p_str.is_empty() {
            path_str.to_string()
        } else {
            p_str.to_string()
        }
    } else {
        path_str.to_string()
    }
}

/// Renderiza a lista de pastas de mídia configuradas pelo usuário em formato de lista moderno.
pub fn render_folders_view(
    ui: &mut Ui,
    folders: &[PathBuf],
    folder_counts: &HashMap<PathBuf, usize>,
    folder_texture: Option<&egui::TextureHandle>,
    selected_index: &mut usize,
    keyboard_nav_active: &mut bool,
    t: &crate::i18n::Translations,
) -> HomeAction {
    let mut action = HomeAction::None;

    if folders.is_empty() {
        let resp = render_empty_state(
            ui,
            lucide::FOLDER_OPEN,
            t.empty_folders_title,
            t.empty_folders_desc,
            Some(t.empty_folders_action),
        );
        if let Some(r) = resp
            && r.clicked()
        {
            action = HomeAction::AddFolder;
        }
    } else {
        ui.add_space(8.0);
        ui.label(
            RichText::new(t.folders_title)
                .font(font_section_title())
                .color(Color32::WHITE),
        );
        ui.add_space(14.0);

        let item_h = 76.0;
        let spacing = 8.0;
        let item_w = ui.available_width();

        for (idx, folder) in folders.iter().enumerate() {
            let folder_name = folder
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(t.header_folder_fallback);

            let count_text = t.folder_items_count(folder_counts.get(folder));
            let display_path = format_folder_display_path(folder);

            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let is_selected = idx == *selected_index && *keyboard_nav_active;
            let hovered = resp.hovered() && !*keyboard_nav_active;

            if is_selected {
                resp.scroll_to_me(Some(egui::Align::Center));
            }

            // Transição suave de hover ou seleção do card de pasta (150ms)
            let hover_t = ui.ctx().animate_bool_with_time(
                resp.id.with("folder_hover"),
                hovered || is_selected,
                0.15,
            );
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            let icon_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            let title_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);

            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            if is_selected {
                ui.painter().rect_stroke(
                    rect,
                    RADIUS_MD,
                    egui::Stroke::new(1.5_f32, ACCENT_COLOR),
                    egui::StrokeKind::Inside,
                );
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 2.0, rect.top() + (item_h - 32.0) * 0.5),
                    vec2(3.5, 32.0),
                );
                ui.painter()
                    .rect_filled(indicator_rect, egui::CornerRadius::same(2), ACCENT_COLOR);
            }

            // Ícone da pasta à esquerda: usa o ícone 3D nativo ampliado com padding generoso
            let text_x = if let Some(tex) = folder_texture {
                let icon_size = vec2(50.0, 44.0);
                let icon_rect = Rect::from_center_size(
                    Pos2::new(rect.left() + 43.0, rect.center().y),
                    icon_size,
                );
                let tint = lerp_color(Color32::from_white_alpha(225), Color32::WHITE, hover_t);
                ui.painter().image(
                    tex.id(),
                    icon_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    tint,
                );
                icon_rect.right() + 16.0
            } else {
                let folder_icon = if hover_t > 0.4 {
                    lucide::FOLDER_OPEN
                } else {
                    lucide::FOLDER
                };
                let icon_pos = Pos2::new(rect.left() + 24.0, rect.center().y);
                ui.painter().text(
                    icon_pos,
                    Align2::LEFT_CENTER,
                    folder_icon,
                    lucide::font_id(32.0),
                    icon_color,
                );
                rect.left() + 68.0
            };

            // Nome e caminho centralizados
            let right_reserve = 190.0;
            let max_w = (rect.right() - right_reserve - text_x).max(40.0);

            // Nome da pasta
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.top() + 26.0),
                Align2::LEFT_CENTER,
                folder_name,
                font_body(),
                title_color,
                max_w,
            );

            // Caminho da pasta no disco
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.top() + 50.0),
                Align2::LEFT_CENTER,
                &display_path,
                font_xs(),
                TEXT_MUTED,
                max_w,
            );

            // Contador de itens à direita com padding arejado
            let count_color = lerp_color(TEXT_MUTED, TEXT_SECONDARY, hover_t);
            ui.painter().text(
                Pos2::new(rect.right() - 64.0, rect.center().y),
                Align2::RIGHT_CENTER,
                &count_text,
                font_sm(),
                count_color,
            );

            // Botão sutil de remover pasta na extremidade direita com padding confortável
            let remove_rect = Rect::from_min_size(
                Pos2::new(rect.right() - 42.0, rect.top() + (item_h - 26.0) * 0.5),
                vec2(26.0, 26.0),
            );
            let rem_resp = ui
                .interact(remove_rect, resp.id.with("rem"), Sense::click())
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
                lucide::font_id(15.0),
                rem_fg,
            );

            if rem_resp.clicked() {
                action = HomeAction::RemoveFolder(folder.clone());
            } else if resp.clicked() {
                *selected_index = idx;
                *keyboard_nav_active = false;
                action = HomeAction::OpenFolder(folder.clone());
            }

            resp.on_hover_cursor(CursorIcon::PointingHand);

            ui.add_space(spacing);
        }
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_folder_count() {
        let t = crate::i18n::Language::Portuguese.t();
        assert_eq!(t.folder_items_count(None), "calculando...");
        assert_eq!(t.folder_items_count(Some(&1)), "1 item");
        assert_eq!(t.folder_items_count(Some(&12)), "12 itens");
    }

    #[test]
    fn test_format_folder_display_path() {
        let path = PathBuf::from("D:\\Media\\Filmes");
        let formatted = format_folder_display_path(&path);
        assert!(!formatted.is_empty());
    }
}
