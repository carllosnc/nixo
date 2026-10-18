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

        let chip_w = 250.0;
        let chip_h = 50.0;
        let spacing = 12.0;
        let avail_w = ui.available_width().max(chip_w);
        let cols = ((avail_w + spacing) / (chip_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("subfolders_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, dir) in content.subdirs.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(chip_w, chip_h), Sense::click());
                    let hovered = resp.hovered();

                    let hover_t =
                        ui.ctx()
                            .animate_bool_with_time(resp.id.with("chip_hover"), hovered, 0.15);
                    let bg = lerp_color(SURFACE_LOW, SURFACE_HIGH, hover_t);
                    let icon_color = lerp_color(TEXT_SECONDARY, Color32::WHITE, hover_t);
                    let text_color = lerp_color(TEXT_PRIMARY, Color32::WHITE, hover_t);

                    ui.painter().rect_filled(rect, RADIUS_MD, bg);

                    // Ícone Lucide de pasta sem wrapper
                    ui.painter().text(
                        Pos2::new(rect.left() + 16.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        lucide::FOLDER,
                        lucide::font_id(20.0),
                        icon_color,
                    );

                    // Nome da subpasta truncado com precisão de pixel
                    let name_x = rect.left() + 46.0;
                    let max_w = (rect.right() - 14.0) - name_x;
                    paint_truncated_text(
                        ui,
                        Pos2::new(name_x, rect.center().y),
                        Align2::LEFT_CENTER,
                        &dir.name,
                        font_body(),
                        text_color,
                        max_w,
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

        let card_w = 230.0;
        let card_h = 160.0;
        let spacing = 14.0;
        let avail_w = ui.available_width().max(card_w);
        let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        egui::Grid::new("files_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (idx, item) in content.files.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
                    let hovered = resp.hovered();

                    let hover_t =
                        ui.ctx()
                            .animate_bool_with_time(resp.id.with("file_hover"), hovered, 0.15);
                    let bg = lerp_color(SURFACE_LOW, SURFACE_HIGH, hover_t);
                    ui.painter().rect_filled(rect, RADIUS_MD, bg);

                    // Área de thumbnail com cantos arredondados no topo
                    let thumb_h = 96.0;
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

                    ui.painter().circle_filled(center, 22.0, circle_bg);
                    ui.painter().text(
                        Pos2::new(center.x + if hover_t > 0.4 { 1.0 } else { 0.0 }, center.y),
                        Align2::CENTER_CENTER,
                        icon_str,
                        lucide::font_id(18.0),
                        icon_fg,
                    );

                    // Badge com extensão (ex.: MP4, MKV)
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(thumb_rect.right() - 48.0, thumb_rect.top() + 8.0),
                        vec2(40.0, 18.0),
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

                    let text_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
                    let max_w = rect.width() - 24.0;

                    // Nome do arquivo com truncamento seguro
                    paint_truncated_text(
                        ui,
                        Pos2::new(rect.left() + 12.0, rect.top() + thumb_h + 18.0),
                        Align2::LEFT_CENTER,
                        clean_name,
                        font_body(),
                        text_color,
                        max_w,
                    );

                    // Tamanho / Metadados
                    paint_truncated_text(
                        ui,
                        Pos2::new(rect.left() + 12.0, rect.top() + thumb_h + 38.0),
                        Align2::LEFT_CENTER,
                        &item.size_formatted,
                        font_xs(),
                        TEXT_MUTED,
                        max_w,
                    );

                    if resp.clicked() {
                        action = HomeAction::PlayFile(item.path.clone());
                    }

                    resp.on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(format!("{}\n{}", item.name, item.size_formatted));

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
