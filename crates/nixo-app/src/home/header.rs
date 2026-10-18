//! Cabeçalho superior da Home com navegação, breadcrumbs e botões de ação rápida.

use std::path::{Path, PathBuf};

use egui::{
    Align, Align2, Color32, CursorIcon, Layout, Pos2, Rect, RichText, Sense, Stroke, TextureHandle,
    Ui, vec2,
};

use crate::components::tokens::*;
use crate::components::{primary, secondary};
use crate::lucide;

const LOGO_PNG: &[u8] = include_bytes!("../../../../assets/logo.png");

/// Carrega a imagem da logo pequena do Nixo a partir dos bytes embutidos.
pub fn load_logo_image() -> Option<egui::ColorImage> {
    let img = image::load_from_memory_with_format(LOGO_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(size, &rgba))
}

/// Ação disparada por interações do usuário na tela inicial.
#[derive(Debug, Clone)]
pub enum HomeAction {
    None,
    AddFolder,
    RemoveFolder(PathBuf),
    OpenFolder(PathBuf),
    GoBack,
    GoRoot,
    OpenFile,
    PlayFile(PathBuf),
    PlayTestSignal,
    OpenDesignSystem,
    CloseDesignSystem,
}

/// Renderiza o cabeçalho superior da Home no padrão Lucide e tons neutros sem bordas.
pub fn render_header(
    ui: &mut Ui,
    current_path: Option<&Path>,
    breadcrumbs: &[PathBuf],
    logo_texture: Option<&TextureHandle>,
) -> HomeAction {
    let mut action = HomeAction::None;

    ui.horizontal(|ui| {
        if let Some(_path) = current_path {
            if secondary(ui, Some(lucide::ARROW_LEFT), "")
                .on_hover_text("Voltar")
                .clicked()
            {
                action = HomeAction::GoBack;
            }

            ui.add_space(10.0);

            // Breadcrumb limpo
            let root_link = ui
                .link(
                    RichText::new("Início")
                        .font(font_body())
                        .color(TEXT_SECONDARY),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Ir para a tela inicial");
            if root_link.clicked() {
                action = HomeAction::GoRoot;
            }

            for crumb in breadcrumbs {
                ui.label(
                    RichText::new(lucide::CHEVRON_RIGHT)
                        .font(lucide::font_id(12.0))
                        .color(TEXT_MUTED),
                );
                let crumb_name = crumb
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Pasta");

                let is_current = current_path.is_some_and(|p| p == crumb);
                let display_name = truncate_breadcrumb(crumb_name, 22);

                let crumb_text =
                    RichText::new(display_name)
                        .font(font_body())
                        .color(if is_current {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        });

                let link_resp = ui
                    .link(crumb_text)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text(crumb_name);
                if link_resp.clicked() && !is_current {
                    action = HomeAction::OpenFolder(crumb.clone());
                }
            }
        } else if let Some(tex) = logo_texture {
            let header_h = 38.0_f32;
            let logo_h = 18.0_f32;
            let logo_w = (logo_h * (1024.0_f32 / 270.0_f32)).round();
            let (rect, resp) = ui.allocate_exact_size(vec2(logo_w, header_h), Sense::click());
            let logo_rect = Rect::from_center_size(rect.center(), vec2(logo_w, logo_h));
            ui.painter().image(
                tex.id(),
                logo_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            if resp.clicked() {
                action = HomeAction::GoRoot;
            }
            resp.on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Nixo Media Player");
        } else {
            let header_h = 38.0_f32;
            let (rect, _) = ui.allocate_exact_size(vec2(60.0, header_h), Sense::hover());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Nixo",
                font_card_title(),
                Color32::WHITE,
            );
        }

        // Os botões de ação rápida são exibidos exclusivamente na tela inicial (Home)
        if current_path.is_none() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if secondary(ui, Some(lucide::SPARKLES), "Design System").clicked() {
                    action = HomeAction::OpenDesignSystem;
                }

                ui.add_space(6.0);

                if secondary(ui, Some(lucide::FLASK_CONICAL), "Testar").clicked() {
                    action = HomeAction::PlayTestSignal;
                }

                ui.add_space(6.0);

                if secondary(ui, Some(lucide::FOLDER_OPEN), "Arquivo").clicked() {
                    action = HomeAction::OpenFile;
                }

                ui.add_space(6.0);

                if primary(ui, Some(lucide::FOLDER_PLUS), "Pasta").clicked() {
                    action = HomeAction::AddFolder;
                }
            });
        }
    });

    ui.add_space(14.0);

    // Linha divisória horizontal sutil do cabeçalho
    let (sep_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().line_segment(
        [sep_rect.left_center(), sep_rect.right_center()],
        Stroke::new(1.0_f32, SURFACE_MED),
    );

    action
}

/// Trunca o texto do breadcrumb com reticências se exceder a contagem máxima de caracteres.
fn truncate_breadcrumb(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        let s: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        let trimmed = s.trim_end_matches([' ', '-', '_', '.']);
        format!("{trimmed}…")
    } else {
        text.to_string()
    }
}
