//! Visão detalhada de dentro de uma pasta: subpastas e grid de arquivos executáveis.

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use super::scanner::FolderContent;
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;

/// Renderiza o conteúdo da pasta aberta (subpastas e arquivos de mídia executáveis).
pub fn render_folder_browser(
    ui: &mut Ui,
    content: &FolderContent,
    thumbnails: &mut super::ThumbnailManager,
) -> HomeAction {
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

    // Seção de Subpastas estruturada em lista limpa e eficiente
    if !content.subdirs.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Subpastas ({})", content.subdirs.len()))
                .font(font_section_title())
                .color(Color32::WHITE),
        );
        ui.add_space(12.0);

        let item_h = 56.0;
        let spacing = 6.0;
        let item_w = ui.available_width();

        for dir in &content.subdirs {
            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let hovered = resp.hovered();

            let hover_t =
                ui.ctx()
                    .animate_bool_with_time(resp.id.with("chip_hover"), hovered, 0.15);
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            let icon_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            let text_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);

            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            // Container visual da pasta à esquerda
            let icon_box_w = 64.0;
            let icon_box_h = 40.0;
            let icon_box_rect = Rect::from_min_size(
                Pos2::new(rect.left() + 8.0, rect.top() + (item_h - icon_box_h) * 0.5),
                vec2(icon_box_w, icon_box_h),
            );
            ui.painter()
                .rect_filled(icon_box_rect, RADIUS_SM, SURFACE_MED);

            let folder_icon = if hover_t > 0.4 {
                lucide::FOLDER_OPEN
            } else {
                lucide::FOLDER
            };
            ui.painter().text(
                icon_box_rect.center(),
                Align2::CENTER_CENTER,
                folder_icon,
                lucide::font_id(20.0),
                icon_color,
            );

            // Nome da subpasta centralizado
            let text_x = icon_box_rect.right() + 14.0;
            let max_w = (rect.right() - 60.0 - text_x).max(40.0);
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.center().y),
                Align2::LEFT_CENTER,
                &dir.name,
                font_body(),
                text_color,
                max_w,
            );

            // Chevron sutil à direita indicando navegação
            let chevron_pos = Pos2::new(rect.right() - 24.0, rect.center().y);
            let chevron_color = lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t);
            ui.painter().text(
                chevron_pos,
                Align2::CENTER_CENTER,
                lucide::CHEVRON_RIGHT,
                lucide::font_id(16.0),
                chevron_color,
            );

            if resp.clicked() {
                action = HomeAction::OpenFolder(dir.path.clone());
            }

            resp.on_hover_cursor(CursorIcon::PointingHand);

            ui.add_space(spacing);
        }

        ui.add_space(18.0);
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

        let item_h = 56.0;
        let spacing = 6.0;
        let item_w = ui.available_width();

        for item in &content.files {
            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let hovered = resp.hovered();

            let hover_t =
                ui.ctx()
                    .animate_bool_with_time(resp.id.with("file_hover"), hovered, 0.15);
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            // Miniatura 16:9 limpa à esquerda
            let thumb_w = 78.0;
            let thumb_h = 44.0;
            let thumb_rect = Rect::from_min_size(
                Pos2::new(rect.left() + 8.0, rect.top() + (item_h - thumb_h) * 0.5),
                vec2(thumb_w, thumb_h),
            );

            if matches!(item.kind, super::scanner::MediaKind::Video) {
                thumbnails.request(&item.path);
            }

            if let Some(tex) = thumbnails.get(&item.path) {
                ui.painter().image(
                    tex.id(),
                    thumb_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                if hover_t > 0.05 {
                    let overlay = Color32::from_black_alpha((hover_t * 110.0) as u8);
                    ui.painter().rect_filled(thumb_rect, RADIUS_SM, overlay);

                    let center = thumb_rect.center();
                    let circle_bg = lerp_color(
                        Color32::from_rgba_unmultiplied(20, 20, 25, 0),
                        ACCENT_NEUTRAL_FILL,
                        hover_t,
                    );
                    let icon_fg = lerp_color(
                        Color32::from_rgba_unmultiplied(255, 255, 255, 0),
                        ACCENT_NEUTRAL_TEXT,
                        hover_t,
                    );
                    ui.painter().circle_filled(center, 13.0, circle_bg);
                    ui.painter().text(
                        Pos2::new(center.x + 1.0, center.y),
                        Align2::CENTER_CENTER,
                        lucide::PLAY,
                        lucide::font_id(12.0),
                        icon_fg,
                    );
                }
            } else {
                ui.painter().rect_filled(thumb_rect, RADIUS_SM, SURFACE_MED);

                // Ícone central de Play / Mídia no Lucide com transição de hover
                let center = thumb_rect.center();
                let default_ic = match item.kind {
                    super::scanner::MediaKind::Video => lucide::FILM,
                    super::scanner::MediaKind::Audio => lucide::MUSIC,
                    super::scanner::MediaKind::Directory => lucide::FOLDER,
                };
                let icon_str = if hover_t > 0.4 {
                    lucide::PLAY
                } else {
                    default_ic
                };
                let circle_bg = lerp_color(SURFACE_LOW, ACCENT_NEUTRAL_FILL, hover_t);
                let icon_fg = lerp_color(TEXT_SECONDARY, ACCENT_NEUTRAL_TEXT, hover_t);

                ui.painter().circle_filled(center, 14.0, circle_bg);
                ui.painter().text(
                    Pos2::new(center.x + if hover_t > 0.4 { 1.0 } else { 0.0 }, center.y),
                    Align2::CENTER_CENTER,
                    icon_str,
                    lucide::font_id(13.0),
                    icon_fg,
                );
            }

            // Nome do arquivo centralizado verticalmente com tipografia nítida
            let clean_name = item
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&item.name);

            let text_x = thumb_rect.right() + 14.0;
            let right_reserve = 100.0;
            let max_w = (rect.right() - right_reserve - text_x).max(40.0);

            let title_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.center().y),
                Align2::LEFT_CENTER,
                clean_name,
                font_body(),
                title_color,
                max_w,
            );

            // Tamanho do arquivo alinhado à direita de forma limpa
            let size_color = lerp_color(TEXT_MUTED, TEXT_SECONDARY, hover_t);
            ui.painter().text(
                Pos2::new(rect.right() - 20.0, rect.center().y),
                Align2::RIGHT_CENTER,
                &item.size_formatted,
                font_sm(),
                size_color,
            );

            if resp.clicked() {
                action = HomeAction::PlayFile(item.path.clone());
            }

            resp.on_hover_cursor(CursorIcon::PointingHand);

            ui.add_space(spacing);
        }
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
