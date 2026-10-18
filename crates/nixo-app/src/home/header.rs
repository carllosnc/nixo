//! Cabeçalho superior da Home com navegação, breadcrumbs e botões de ação rápida.

use std::path::{Path, PathBuf};

use egui::{Align, Color32, CursorIcon, Layout, RichText, Sense, Stroke, Ui, vec2};

use crate::components::tokens::*;
use crate::components::{primary, secondary};
use crate::lucide;

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
) -> HomeAction {
    let mut action = HomeAction::None;

    ui.horizontal(|ui| {
        if let Some(_path) = current_path {
            if secondary(ui, Some(lucide::ARROW_LEFT), "Voltar").clicked() {
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
                .on_hover_cursor(CursorIcon::PointingHand);
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
                let crumb_text = RichText::new(crumb_name)
                    .font(font_body())
                    .color(if is_current {
                        TEXT_PRIMARY
                    } else {
                        TEXT_SECONDARY
                    });

                let link_resp = ui
                    .link(crumb_text)
                    .on_hover_cursor(CursorIcon::PointingHand);
                if link_resp.clicked() && !is_current {
                    action = HomeAction::OpenFolder(crumb.clone());
                }
            }
        } else {
            ui.label(
                RichText::new("Nixo")
                    .font(font_display())
                    .color(Color32::WHITE),
            );
            ui.add_space(10.0);
            ui.label(
                RichText::new("Biblioteca de Mídia")
                    .font(font_body())
                    .color(TEXT_SECONDARY),
            );
        }

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
