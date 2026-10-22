//! Tela de Ajustes de Imagem (Equalizador de Vídeo) no menu de configurações.

use egui::Ui;
use nixo_core::{PlayerCommand, VideoEqualizer};

use super::SettingsTab;
use super::components::{MenuButton, MenuDivider, menu_slider, submenu_header};
use crate::app::NixoApp;

impl NixoApp {
    pub(super) fn render_tab_video_equalizer(&self, ui: &mut Ui, actions: &mut super::MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.video_equalizer_title) {
            actions.navigate(SettingsTab::Video);
        }

        let mut current_eq = self.status.video_equalizer;
        let mut changed = false;

        // Brilho (-100 a +100)
        if let Some(val) = menu_slider(
            ui,
            t.video_brightness,
            current_eq.brightness as f64,
            -100.0..=100.0,
            1.0,
            |v| format!("{v:+.0}"),
        ) {
            current_eq.brightness = val.round() as i32;
            changed = true;
        }

        // Contraste (-100 a +100)
        if let Some(val) = menu_slider(
            ui,
            t.video_contrast,
            current_eq.contrast as f64,
            -100.0..=100.0,
            1.0,
            |v| format!("{v:+.0}"),
        ) {
            current_eq.contrast = val.round() as i32;
            changed = true;
        }

        // Saturação (-100 a +100)
        if let Some(val) = menu_slider(
            ui,
            t.video_saturation,
            current_eq.saturation as f64,
            -100.0..=100.0,
            1.0,
            |v| format!("{v:+.0}"),
        ) {
            current_eq.saturation = val.round() as i32;
            changed = true;
        }

        // Gama (-100 a +100)
        if let Some(val) = menu_slider(
            ui,
            t.video_gamma,
            current_eq.gamma as f64,
            -100.0..=100.0,
            1.0,
            |v| format!("{v:+.0}"),
        ) {
            current_eq.gamma = val.round() as i32;
            changed = true;
        }

        if changed {
            actions.send(PlayerCommand::SetVideoEqualizer(current_eq));
        }

        if !current_eq.is_default() {
            ui.add(MenuDivider);
            if ui.add(MenuButton::new(t.video_reset_eq)).clicked() {
                actions.send(PlayerCommand::SetVideoEqualizer(VideoEqualizer::default()));
            }
        }
    }
}
