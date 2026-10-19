use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use super::scanner::FolderContent;
use crate::app::ACCENT_COLOR;
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;
use crate::settings::WatchHistoryEntry;

/// Texturas decorativas compartilhadas para renderização do navegador de pastas.
#[derive(Clone, Copy, Default)]
pub struct BrowserTextures<'a> {
    pub folder: Option<&'a egui::TextureHandle>,
    pub video_clapper: Option<&'a egui::TextureHandle>,
}

/// Renderiza o conteúdo da pasta aberta (subpastas e arquivos de mídia executáveis).
pub fn render_folder_browser(
    ui: &mut Ui,
    content: &FolderContent,
    thumbnails: &mut super::ThumbnailManager,
    textures: BrowserTextures<'_>,
    selected_index: &mut usize,
    keyboard_nav_active: &mut bool,
    watch_history: &HashMap<PathBuf, WatchHistoryEntry>,
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

        for (idx, dir) in content.subdirs.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let is_selected = idx == *selected_index && *keyboard_nav_active;
            let hovered = resp.hovered() && !*keyboard_nav_active;

            if is_selected {
                resp.scroll_to_me(Some(egui::Align::Center));
            }

            let hover_t = ui.ctx().animate_bool_with_time(
                resp.id.with("chip_hover"),
                hovered || is_selected,
                0.15,
            );
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            let icon_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            let text_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);

            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            if is_selected {
                ui.painter().rect_stroke(
                    rect,
                    RADIUS_MD,
                    egui::Stroke::new(1.5_f32, ACCENT_COLOR),
                    egui::StrokeKind::Inside,
                );
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 2.0, rect.top() + (item_h - 28.0) * 0.5),
                    vec2(3.5, 28.0),
                );
                ui.painter()
                    .rect_filled(indicator_rect, egui::CornerRadius::same(2), ACCENT_COLOR);
            }

            // Ícone da subpasta à esquerda: usa o ícone 3D nativo ampliado com padding confortável
            let text_x = if let Some(tex) = textures.folder {
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
                *selected_index = idx;
                *keyboard_nav_active = false;
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

        let item_h = 100.0;
        let spacing = 8.0;
        let item_w = ui.available_width();

        for (f_idx, item) in content.files.iter().enumerate() {
            let global_idx = content.subdirs.len() + f_idx;
            let (rect, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
            let is_selected = global_idx == *selected_index && *keyboard_nav_active;
            let hovered = resp.hovered() && !*keyboard_nav_active;

            if is_selected {
                resp.scroll_to_me(Some(egui::Align::Center));
            }

            let is_visible = ui.is_rect_visible(rect);
            let is_video = matches!(item.kind, super::scanner::MediaKind::Video);

            if !is_visible {
                ui.add_space(spacing);
                continue;
            }

            let hover_t = ui.ctx().animate_bool_with_time(
                resp.id.with("file_hover"),
                hovered || is_selected,
                0.15,
            );
            let bg = lerp_color(SURFACE_LOW, SURFACE_CARD_HOVER, hover_t);
            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            if is_selected {
                ui.painter().rect_stroke(
                    rect,
                    RADIUS_MD,
                    egui::Stroke::new(1.5_f32, ACCENT_COLOR),
                    egui::StrokeKind::Inside,
                );
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 2.0, rect.top() + (item_h - 40.0) * 0.5),
                    vec2(3.5, 40.0),
                );
                ui.painter()
                    .rect_filled(indicator_rect, egui::CornerRadius::same(2), ACCENT_COLOR);
            }

            // Dispara requisição de metadados em background para itens visíveis na tela
            if is_video {
                thumbnails.request(&item.path);
                if hovered || is_selected {
                    thumbnails.prioritize(&item.path);
                }
            } else if matches!(item.kind, super::scanner::MediaKind::Audio) {
                thumbnails.request(&item.path);
            }

            // Preview interativo 16:9 ampliado à esquerda no lugar do ícone
            let text_x = if is_video {
                let thumb_w = 144.0_f32;
                let thumb_h = 81.0_f32;
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 14.0, rect.top() + (item_h - thumb_h) * 0.5),
                    vec2(thumb_w, thumb_h),
                );

                if let Some(tex) = thumbnails.get(&item.path) {
                    ui.painter().image(
                        tex.id(),
                        thumb_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter()
                        .rect_filled(thumb_rect, egui::CornerRadius::ZERO, SURFACE_MED);
                    if let Some(tex) = textures.video_clapper {
                        let ic_size = vec2(46.0, 42.0);
                        let ic_rect = Rect::from_center_size(thumb_rect.center(), ic_size);
                        let tint = lerp_color(
                            Color32::from_white_alpha(90),
                            Color32::from_white_alpha(170),
                            hover_t,
                        );
                        ui.painter().image(
                            tex.id(),
                            ic_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            tint,
                        );
                    } else {
                        let ic_col = if hovered {
                            Color32::from_white_alpha(60)
                        } else {
                            Color32::from_white_alpha(30)
                        };
                        ui.painter().text(
                            thumb_rect.center(),
                            Align2::CENTER_CENTER,
                            lucide::FILM,
                            lucide::font_id(28.0),
                            ic_col,
                        );
                    }
                }

                if let Some(entry) = watch_history.get(&item.path) {
                    let progress = entry.progress();
                    if progress > 0.015 && !entry.is_completed() {
                        let bar_h = 3.5_f32;
                        let bar_bg = Rect::from_min_size(
                            Pos2::new(thumb_rect.left(), thumb_rect.bottom() - bar_h),
                            vec2(thumb_rect.width(), bar_h),
                        );
                        ui.painter().rect_filled(
                            bar_bg,
                            egui::CornerRadius::ZERO,
                            Color32::from_black_alpha(170),
                        );
                        let fill_w = thumb_rect.width() * progress;
                        let fill_rect = Rect::from_min_size(
                            Pos2::new(thumb_rect.left(), thumb_rect.bottom() - bar_h),
                            vec2(fill_w, bar_h),
                        );
                        ui.painter()
                            .rect_filled(fill_rect, egui::CornerRadius::ZERO, ACCENT_COLOR);
                    }
                }

                thumb_rect.right() + 18.0
            } else {
                let default_ic = match item.kind {
                    super::scanner::MediaKind::Video => lucide::FILM,
                    super::scanner::MediaKind::Audio => lucide::MUSIC,
                    super::scanner::MediaKind::Directory => lucide::FOLDER,
                };
                let icon_pos = Pos2::new(rect.left() + 28.0, rect.center().y);
                ui.painter().text(
                    icon_pos,
                    Align2::LEFT_CENTER,
                    default_ic,
                    lucide::font_id(30.0),
                    lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t),
                );
                rect.left() + 72.0
            };

            // Nome do arquivo (Linha 1 do item)
            let clean_name = item
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&item.name);

            let max_w = (rect.right() - 100.0 - text_x).max(40.0);
            let title_color = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, hover_t);
            paint_truncated_text(
                ui,
                Pos2::new(text_x, rect.top() + 34.0),
                Align2::LEFT_CENTER,
                clean_name,
                font_body(),
                title_color,
                max_w,
            );

            // Linha de descrição com badge do tipo de mídia, duração e resolução (Linha 2)
            let desc_y = rect.top() + 66.0;

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

            if let Some(entry) = watch_history.get(&item.path) {
                let resume_badge = if entry.is_completed() {
                    Some(("Assistido ✓".to_string(), Color32::from_rgb(52, 199, 89)))
                } else if entry.position_secs > 5.0 {
                    let pos_dur = std::time::Duration::from_secs_f64(entry.position_secs);
                    let pos_str = nixo_core::format_duration(pos_dur);
                    let pct = (entry.progress() * 100.0).round() as u32;
                    Some((format!("Parou em {pos_str} ({pct}%)"), ACCENT_COLOR))
                } else {
                    None
                };

                if let Some((resume_text, resume_color)) = resume_badge {
                    ui.painter().text(
                        Pos2::new(meta_x, desc_y),
                        Align2::LEFT_CENTER,
                        "•",
                        font_xs(),
                        TEXT_MUTED,
                    );
                    meta_x += 10.0;

                    let resume_galley =
                        ui.painter()
                            .layout_no_wrap(resume_text, font_xs(), resume_color);
                    let resume_w = resume_galley.size().x;
                    ui.painter().galley(
                        Pos2::new(meta_x, desc_y - resume_galley.size().y * 0.5),
                        resume_galley,
                        resume_color,
                    );
                    meta_x += resume_w + 8.0;
                }
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
                *selected_index = global_idx;
                *keyboard_nav_active = false;
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
