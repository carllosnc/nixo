//! Telas principais do menu: raiz, velocidade, áudio e legendas.
//!
//! As telas apenas compõem componentes de [`super::components`]; nenhuma
//! pintura direta acontece aqui.

use std::iter::once;

use egui::Ui;
use nixo_core::PlayerCommand;

use super::MenuActions;
use super::components::{
    Choice, EmptyState, LeadingIcon, MenuDivider, MenuOption, NavRow, SectionTitle, choice_list,
    submenu_header,
};
use super::style::menu_style;
use crate::app::{NixoApp, SettingsTab};

const SPEED_PRESETS: [f32; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

fn is_normal_speed(speed: f32) -> bool {
    (speed - 1.0).abs() < 0.01
}

impl NixoApp {
    pub(super) fn render_tab_main(&self, ui: &mut Ui, actions: &mut MenuActions) {
        ui.add(SectionTitle::new("CONFIGURAÇÕES"));
        ui.add(MenuDivider);

        let rows = [
            ("VELOCIDADE", self.speed_summary(), SettingsTab::Speed),
            ("ÁUDIO", self.audio_summary(), SettingsTab::Audio),
            ("LEGENDAS", self.subtitle_summary(), SettingsTab::Subtitles),
        ];
        for (title, value, tab) in rows {
            if ui.add(NavRow::new(title, &value)).clicked() {
                actions.navigate(tab);
            }
        }
    }

    pub(super) fn render_tab_speed(&self, ui: &mut Ui, actions: &mut MenuActions) {
        if submenu_header(ui, "VELOCIDADE") {
            actions.navigate(SettingsTab::Main);
        }

        let speed = self.status.speed;
        let choices = SPEED_PRESETS.map(|preset| {
            let label = if is_normal_speed(preset) {
                "Normal".to_string()
            } else {
                format!("{preset}")
            };
            Choice::new(label, preset).selected((speed - preset).abs() < 0.01)
        });
        if let Some(preset) = choice_list(ui, choices) {
            actions.send_and_close(PlayerCommand::SetSpeed(preset));
        }
    }

    pub(super) fn render_tab_audio(&self, ui: &mut Ui, actions: &mut MenuActions) {
        if submenu_header(ui, "FAIXA DE ÁUDIO") {
            actions.navigate(SettingsTab::Main);
        }

        let tracks = &self.status.audio_tracks;
        if tracks.is_empty() {
            ui.add(EmptyState::new("Nenhuma faixa de áudio detectada"));
            return;
        }

        let none_selected = !tracks.iter().any(|t| t.selected);
        let choices = tracks
            .iter()
            .map(|track| {
                Choice::new(track.display_name(), Some(track.id))
                    .selected(track.selected)
                    .icon(LeadingIcon::for_track(
                        track.flag_code(),
                        LeadingIcon::GenericAudio,
                    ))
            })
            .chain(once(
                Choice::new("Desativar áudio", None)
                    .selected(none_selected)
                    .icon(LeadingIcon::Disabled),
            ));
        if let Some(track_id) = choice_list(ui, choices) {
            actions.send_and_close(PlayerCommand::SetAudioTrack(track_id));
        }
    }

    pub(super) fn render_tab_subtitles(&self, ui: &mut Ui, actions: &mut MenuActions) {
        if submenu_header(ui, "LEGENDAS") {
            actions.navigate(SettingsTab::Main);
        }

        let visible = self.status.subtitles_visible;
        let tracks = &self.status.subtitle_tracks;
        let is_off = !visible || !tracks.iter().any(|t| t.selected);
        let choices = once(
            Choice::new("Desativadas", None)
                .selected(is_off)
                .icon(LeadingIcon::Disabled),
        )
        .chain(tracks.iter().map(|track| {
            Choice::new(track.display_name(), Some(track.id))
                .selected(track.selected && visible)
                .icon(LeadingIcon::for_track(
                    track.flag_code(),
                    LeadingIcon::GenericSubtitle,
                ))
        }));
        if let Some(track_id) = choice_list(ui, choices) {
            actions.send_and_close(PlayerCommand::SetSubtitleTrack(track_id));
        }

        ui.add(MenuDivider);
        let load =
            MenuOption::new("Carregar legenda externa...").icon(LeadingIcon::AddExternalSubtitle);
        if ui.add(load).clicked()
            && let Some(path) = pick_subtitle_file()
        {
            actions.send_and_close(PlayerCommand::LoadSubtitleFile(path));
        }

        ui.add(MenuDivider);
        let style_entry = MenuOption::new("Estilo da legenda")
            .icon(LeadingIcon::SubtitleStyle)
            .font(Some(menu_style().font_option_emphasis.clone()))
            .chevron();
        if ui.add(style_entry).clicked() {
            actions.navigate(SettingsTab::SubtitleStyle);
        }
    }

    fn speed_summary(&self) -> String {
        let speed = self.status.speed;
        if is_normal_speed(speed) {
            "Normal".to_string()
        } else {
            format!("{speed:.2}x")
        }
    }

    fn audio_summary(&self) -> String {
        let tracks = &self.status.audio_tracks;
        match tracks.iter().find(|t| t.selected) {
            Some(track) => track.display_name(),
            None if tracks.is_empty() => "Nenhuma".to_string(),
            None => "Desativado".to_string(),
        }
    }

    fn subtitle_summary(&self) -> String {
        self.status
            .subtitle_tracks
            .iter()
            .find(|t| t.selected && self.status.subtitles_visible)
            .map(|t| t.display_name())
            .unwrap_or_else(|| "Desativadas".to_string())
    }
}

fn pick_subtitle_file() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title("Carregar Legenda Externa no Nixo")
        .add_filter("Legendas", &["srt", "vtt", "ass", "ssa", "sub"])
        .add_filter("Todos os Arquivos", &["*"])
        .pick_file()
}
