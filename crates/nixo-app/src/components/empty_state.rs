//! Componente de Empty State minimalista, espaçoso e neutro.

use egui::{Align2, Color32, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::tokens::*;
use crate::home::HomeAction;
use crate::lucide;

/// Renderiza um estado vazio elegante e minimalista com cantos full rounded (para pastas/subpastas vazias).
pub fn render_empty_state(
    ui: &mut Ui,
    icon: &str,
    title: &str,
    description: &str,
    action_text: Option<&str>,
) -> Option<egui::Response> {
    let mut action_resp = None;

    ui.add_space(28.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
        ui.painter().rect_filled(icon_rect, RADIUS_LG, SURFACE_MED);

        ui.painter().text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            icon,
            lucide::font_id(24.0),
            TEXT_SECONDARY,
        );

        ui.add_space(14.0);
        ui.label(
            RichText::new(title)
                .font(font_title())
                .strong()
                .color(TEXT_PRIMARY),
        );

        ui.add_space(4.0);
        ui.label(
            RichText::new(description)
                .font(font_body())
                .color(TEXT_SECONDARY),
        );

        if let Some(act) = action_text {
            ui.add_space(16.0);
            let resp = super::button::secondary(ui, Some(lucide::PLUS), act);
            action_resp = Some(resp);
        }
    });

    action_resp
}

/// Renderiza o estado vazio limpo da tela inicial: apenas ícone, título, descrição e botão de ação.
pub fn render_hero_empty_state(
    ui: &mut Ui,
    folder_texture: Option<&egui::TextureHandle>,
    t: &crate::i18n::Translations,
) -> HomeAction {
    let mut action = HomeAction::None;

    // Espaçamento vertical generoso para posicionar no centro visual da janela
    let screen_h = ui.available_height();
    let top_space = (screen_h * 0.18).clamp(32.0, 110.0);
    ui.add_space(top_space);

    ui.vertical_centered(|ui| {
        // 1. Ícone heróico diretamente sobre o fundo, sem wrapper
        let icon_size = vec2(76.0, 68.0);
        let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

        if let Some(tex) = folder_texture {
            ui.painter().image(
                tex.id(),
                icon_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(
                icon_rect.center(),
                Align2::CENTER_CENTER,
                lucide::FOLDER_OPEN,
                lucide::font_id(44.0),
                TEXT_SECONDARY,
            );
        }

        ui.add_space(16.0);

        // 2. Título
        ui.label(
            RichText::new(t.empty_hero_title)
                .font(font_title())
                .strong()
                .color(TEXT_PRIMARY),
        );

        ui.add_space(6.0);

        // 3. Descrição
        let desc_max_w = 480.0_f32.min(ui.available_width() - 48.0);
        ui.scope(|ui| {
            ui.set_max_width(desc_max_w);
            ui.label(
                RichText::new(t.empty_hero_subtitle)
                    .font(font_body())
                    .color(TEXT_SECONDARY),
            );
        });

        ui.add_space(20.0);

        // 4. Botão
        let resp = super::button::secondary(ui, Some(lucide::FOLDER_PLUS), t.empty_folders_action);
        if resp.clicked() {
            action = HomeAction::AddFolder;
        }
    });

    action
}
