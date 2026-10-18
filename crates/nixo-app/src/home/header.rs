//! Cabeçalho superior da Home com navegação, breadcrumbs e botões de ação rápida.

use std::path::{Path, PathBuf};

use egui::{Align, Color32, CornerRadius, CursorIcon, FontId, Layout, RichText, Ui};

use crate::app::ACCENT_COLOR;

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
}

/// Renderiza o cabeçalho superior da Home.
pub fn render_header(
    ui: &mut Ui,
    current_path: Option<&Path>,
    breadcrumbs: &[PathBuf],
) -> HomeAction {
    let mut action = HomeAction::None;

    ui.horizontal(|ui| {
        if let Some(_path) = current_path {
            let back_btn = egui::Button::new(
                RichText::new("← Voltar")
                    .font(FontId::proportional(13.5))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(32, 35, 44))
            .corner_radius(CornerRadius::same(8));

            if ui.add_sized([86.0, 32.0], back_btn).clicked() {
                action = HomeAction::GoBack;
            }

            ui.add_space(8.0);

            let root_link = ui
                .link(
                    RichText::new("Início")
                        .font(FontId::proportional(13.5))
                        .color(ACCENT_COLOR),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if root_link.clicked() {
                action = HomeAction::GoRoot;
            }

            for crumb in breadcrumbs {
                ui.label(RichText::new("/").color(Color32::from_gray(120)));
                let crumb_name = crumb
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Pasta");

                let is_current = current_path.is_some_and(|p| p == crumb);
                let crumb_text = RichText::new(crumb_name)
                    .font(FontId::proportional(13.5))
                    .color(if is_current {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(185)
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
                RichText::new("N I X O")
                    .font(FontId::proportional(18.0))
                    .strong()
                    .color(ACCENT_COLOR),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Biblioteca de Mídia")
                    .font(FontId::proportional(13.5))
                    .color(Color32::from_gray(150)),
            );
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let test_btn = egui::Button::new(
                RichText::new("▶ Testar")
                    .font(FontId::proportional(13.0))
                    .color(Color32::from_gray(210)),
            )
            .fill(Color32::from_rgb(26, 28, 35))
            .corner_radius(CornerRadius::same(8));
            if ui.add_sized([84.0, 32.0], test_btn).clicked() {
                action = HomeAction::PlayTestSignal;
            }

            ui.add_space(4.0);

            let open_btn = egui::Button::new(
                RichText::new("📂 Abrir Arquivo")
                    .font(FontId::proportional(13.0))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(34, 37, 46))
            .corner_radius(CornerRadius::same(8));
            if ui.add_sized([126.0, 32.0], open_btn).clicked() {
                action = HomeAction::OpenFile;
            }

            ui.add_space(4.0);

            let add_btn = egui::Button::new(
                RichText::new("+ Adicionar Pasta")
                    .font(FontId::proportional(13.0))
                    .color(Color32::WHITE),
            )
            .fill(ACCENT_COLOR)
            .corner_radius(CornerRadius::same(8));
            if ui.add_sized([138.0, 32.0], add_btn).clicked() {
                action = HomeAction::AddFolder;
            }
        });
    });

    action
}
