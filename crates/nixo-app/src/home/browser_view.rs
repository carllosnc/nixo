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
    folder_texture: Option<&egui::TextureHandle>,
    video_clapper_texture: Option<&egui::TextureHandle>,
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

        let item_h = 64.0;
        let spacing = 8.0;
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

            // Ícone da subpasta à esquerda: usa o ícone 3D nativo ampliado com padding confortável
            let text_x = if let Some(tex) = folder_texture {
                let icon_size = vec2(46.0, 40.0);
                let icon_rect = Rect::from_center_size(
                    Pos2::new(rect.left() + 41.0, rect.center().y),
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
                let icon_pos = Pos2::new(rect.left() + 22.0, rect.center().y);
                ui.painter().text(
                    icon_pos,
                    Align2::LEFT_CENTER,
                    folder_icon,
                    lucide::font_id(28.0),
                    icon_color,
                );
                rect.left() + 64.0
            };

            // Nome da subpasta centralizado
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

            // Chevron sutil à direita indicando navegação com padding arejado
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

        let item_h = 76.0;
        let spacing = 8.0;
        let item_w = ui.available_width();

        for item in &content.files {
            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let hovered = resp.hovered();

            let hover_t =
                ui.ctx()
                    .animate_bool_with_time(resp.id.with("file_hover"), hovered, 0.15);
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            // Dispara requisição de metadados de vídeo/áudio em background se necessário
            if matches!(
                item.kind,
                super::scanner::MediaKind::Video | super::scanner::MediaKind::Audio
            ) {
                thumbnails.request(&item.path);
            }

            // Ícone do arquivo de mídia: usa o ícone clapperboard ampliado nos vídeos
            let text_x = if matches!(item.kind, super::scanner::MediaKind::Video)
                && let Some(tex) = video_clapper_texture
            {
                let icon_size = vec2(50.0, 46.0);
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
                let default_ic = match item.kind {
                    super::scanner::MediaKind::Video => lucide::FILM,
                    super::scanner::MediaKind::Audio => lucide::MUSIC,
                    super::scanner::MediaKind::Directory => lucide::FOLDER,
                };
                let icon_pos = Pos2::new(rect.left() + 24.0, rect.center().y);
                ui.painter().text(
                    icon_pos,
                    Align2::LEFT_CENTER,
                    default_ic,
                    lucide::font_id(30.0),
                    lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t),
                );
                rect.left() + 68.0
            };

            // Nome do arquivo (Linha 1 do item)
            let clean_name = item
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&item.name);

            let right_reserve = 100.0;
            let max_w = (rect.right() - right_reserve - text_x).max(40.0);

            let title_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.top() + 25.0),
                Align2::LEFT_CENTER,
                clean_name,
                font_body(),
                title_color,
                max_w,
            );

            // Linha de descrição com badge do tipo de mídia, duração e resolução (Linha 2)
            let desc_y = rect.top() + 51.0;

            // 1. Badge com tipo de mídia (extensão ex.: MKV, MP4)
            let badge_text = if !item.extension.is_empty() {
                item.extension.to_uppercase()
            } else {
                match item.kind {
                    super::scanner::MediaKind::Video => "VÍDEO".to_string(),
                    super::scanner::MediaKind::Audio => "ÁUDIO".to_string(),
                    super::scanner::MediaKind::Directory => "PASTA".to_string(),
                }
            };

            let badge_font = font_xs_bold();
            let badge_text_w = ui
                .painter()
                .layout_no_wrap(badge_text.clone(), badge_font.clone(), Color32::WHITE)
                .size()
                .x;
            let badge_w = (badge_text_w + 10.0).max(28.0);
            let badge_rect =
                Rect::from_min_size(Pos2::new(text_x, desc_y - 8.0), vec2(badge_w, 16.0));

            let badge_bg = lerp_color(
                Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                Color32::from_rgba_unmultiplied(255, 255, 255, 28),
                hover_t,
            );
            let badge_fg = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            ui.painter()
                .rect_filled(badge_rect, egui::CornerRadius::same(3), badge_bg);
            ui.painter().text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                &badge_text,
                badge_font,
                badge_fg,
            );

            // 2. Metadados de duração e resolução
            let details = thumbnails.get_details(&item.path);
            let dur_str = details.and_then(|d| d.duration_formatted());
            let res_str = details.and_then(|d| d.resolution_formatted()).or_else(|| {
                super::thumbnail_manager::parse_resolution_from_name(&item.name).map(str::to_string)
            });

            let mut meta_x = badge_rect.right() + 8.0;
            let meta_color = lerp_color(TEXT_MUTED, TEXT_SECONDARY, hover_t);

            if let Some(dur) = dur_str {
                ui.painter().text(
                    Pos2::new(meta_x, desc_y),
                    Align2::LEFT_CENTER,
                    "•",
                    font_xs(),
                    TEXT_MUTED,
                );
                meta_x += 10.0;

                let dur_galley = ui.painter().layout_no_wrap(dur, font_xs(), meta_color);
                let dur_w = dur_galley.size().x;
                ui.painter().galley(
                    Pos2::new(meta_x, desc_y - dur_galley.size().y * 0.5),
                    dur_galley,
                    meta_color,
                );
                meta_x += dur_w + 8.0;
            }

            if let Some(res) = res_str {
                ui.painter().text(
                    Pos2::new(meta_x, desc_y),
                    Align2::LEFT_CENTER,
                    "•",
                    font_xs(),
                    TEXT_MUTED,
                );
                meta_x += 10.0;

                let res_galley = ui.painter().layout_no_wrap(res, font_xs(), meta_color);
                ui.painter().galley(
                    Pos2::new(meta_x, desc_y - res_galley.size().y * 0.5),
                    res_galley,
                    meta_color,
                );
            }

            // Tamanho do arquivo alinhado à direita com padding arejado
            let size_color = lerp_color(TEXT_MUTED, TEXT_SECONDARY, hover_t);
            ui.painter().text(
                Pos2::new(rect.right() - 24.0, rect.center().y),
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
