//! Telas principais do menu: raiz, velocidade, áudio, legendas e idioma.
//!
//! As telas apenas compõem componentes de [`super::components`]; nenhuma
//! pintura direta acontece aqui.

use std::iter::once;

use egui::Ui;
use nixo_core::PlayerCommand;

use super::MenuActions;
use super::components::{
    Choice, EmptyState, LeadingIcon, MenuDivider, MenuOption, MenuToggle, NavRow, SectionTitle,
    WarningBanner, choice_list, delay_stepper, submenu_header,
};
use super::style::menu_style;
use crate::app::{NixoApp, SettingsTab};
use crate::i18n::Language;

const SPEED_PRESETS: [f32; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

fn is_normal_speed(speed: f32) -> bool {
    (speed - 1.0).abs() < 0.01
}

impl NixoApp {
    pub(super) fn render_tab_main(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        ui.add(SectionTitle::new(t.settings_title));
        ui.add(MenuDivider);

        let rows = [
            (t.menu_video, self.video_summary(), SettingsTab::Video),
            (t.menu_speed, self.speed_summary(), SettingsTab::Speed),
            (t.menu_audio, self.audio_summary(), SettingsTab::Audio),
            (
                t.menu_subtitles,
                self.subtitle_summary(),
                SettingsTab::Subtitles,
            ),
            (
                t.menu_language,
                self.language.display_name().to_string(),
                SettingsTab::Language,
            ),
        ];
        for (title, value, tab) in rows {
            if ui.add(NavRow::new(title, &value)).clicked() {
                actions.navigate(tab);
            }
        }

        ui.add(MenuDivider);
        if ui
            .add(MenuToggle::new(t.menu_always_on_top, self.always_on_top))
            .clicked()
        {
            actions.toggle_always_on_top = true;
        }
    }

    pub(super) fn render_tab_speed(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.speed_title) {
            actions.navigate(SettingsTab::Main);
        }

        let speed = self.status.speed;
        let choices = SPEED_PRESETS.map(|preset| {
            let label = if is_normal_speed(preset) {
                t.speed_normal.to_string()
            } else {
                format!("{preset}")
            };
            Choice::new(label, preset).selected((speed - preset).abs() < 0.01)
        });
        if let Some(preset) = choice_list(ui, choices) {
            actions.send_and_close(PlayerCommand::SetSpeed(preset));
        }
    }

    pub(super) fn render_tab_video(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.menu_video) {
            actions.navigate(SettingsTab::Main);
        }

        ui.add(SectionTitle::new(t.video_framing));
        let fit_choices = nixo_core::VideoFit::ALL.map(|fit| {
            let label = match fit {
                nixo_core::VideoFit::Fit => t.video_fit_screen,
                nixo_core::VideoFit::Fill => t.video_fill_screen,
            };
            Choice::new(label, fit).selected(self.status.video_fit == fit)
        });
        if let Some(fit) = choice_list(ui, fit_choices) {
            actions.send(PlayerCommand::SetVideoFit(fit));
        }

        ui.add(MenuDivider);
        ui.add(SectionTitle::new(t.video_aspect_ratio));
        let aspect_choices = nixo_core::VideoAspect::ALL.map(|aspect| {
            let label = match aspect {
                nixo_core::VideoAspect::Auto => t.video_aspect_auto,
                nixo_core::VideoAspect::Ratio16_9 => t.video_aspect_16_9,
                nixo_core::VideoAspect::Ratio4_3 => t.video_aspect_4_3,
                nixo_core::VideoAspect::Ratio235_1 => t.video_aspect_235_1,
            };
            Choice::new(label, aspect).selected(self.status.aspect_ratio == aspect)
        });
        if let Some(aspect) = choice_list(ui, aspect_choices) {
            actions.send(PlayerCommand::SetAspectRatio(aspect));
        }

        ui.add(MenuDivider);
        let eq_summary = if self.status.video_equalizer.is_default() {
            t.speed_normal
        } else {
            "✓"
        };
        if ui
            .add(NavRow::new(t.video_equalizer_title, eq_summary))
            .clicked()
        {
            actions.navigate(SettingsTab::VideoEqualizer);
        }
    }

    pub(super) fn render_tab_audio(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.audio_track_title) {
            actions.navigate(SettingsTab::Main);
        }

        let tracks = &self.status.audio_tracks;
        if tracks.is_empty() {
            ui.add(EmptyState::new(t.audio_no_tracks));
            return;
        }

        let none_selected = !tracks.iter().any(|tr| tr.selected);
        let choices = tracks
            .iter()
            .map(|track| {
                Choice::new(
                    track.localized_display_name(t.audio_track_prefix),
                    Some(track.id),
                )
                .selected(track.selected)
                .icon(LeadingIcon::for_track(
                    track.flag_code(),
                    LeadingIcon::GenericAudio,
                ))
            })
            .chain(once(
                Choice::new(t.audio_disable, None)
                    .selected(none_selected)
                    .icon(LeadingIcon::Disabled),
            ));
        if let Some(track_id) = choice_list(ui, choices) {
            actions.send_and_close(PlayerCommand::SetAudioTrack(track_id));
        }

        ui.add(MenuDivider);
        if let Some(delay) = delay_stepper(ui, t.audio_sync, self.status.audio_delay) {
            actions.send(PlayerCommand::SetAudioDelay(delay));
        }
    }

    pub(super) fn render_tab_subtitles(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.subtitles_title) {
            actions.navigate(SettingsTab::Main);
        }

        if let Some(warning) = self.subtitle_health_warning() {
            ui.add(WarningBanner::new(&warning));
        }

        let visible = self.status.subtitles_visible;
        let tracks = &self.status.subtitle_tracks;
        let is_off = !visible || !tracks.iter().any(|tr| tr.selected);
        let choices = once(
            Choice::new(t.subtitles_disabled, None)
                .selected(is_off)
                .icon(LeadingIcon::Disabled),
        )
        .chain(tracks.iter().map(|track| {
            Choice::new(
                track.localized_display_name(t.audio_track_prefix),
                Some(track.id),
            )
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
        if let Some(delay) = delay_stepper(ui, t.audio_sync, self.status.sub_delay) {
            actions.send(PlayerCommand::SetSubDelay(delay));
        }

        ui.add(MenuDivider);
        let load =
            MenuOption::new(t.subtitles_load_external).icon(LeadingIcon::AddExternalSubtitle);
        if ui.add(load).clicked()
            && let Some(path) = pick_subtitle_file(t)
        {
            actions.send_and_close(PlayerCommand::LoadSubtitleFile(path));
        }

        ui.add(MenuDivider);
        let style_entry = MenuOption::new(t.subtitles_style)
            .icon(LeadingIcon::SubtitleStyle)
            .font(Some(menu_style().font_option_emphasis.clone()))
            .chevron();
        if ui.add(style_entry).clicked() {
            actions.navigate(SettingsTab::SubtitleStyle);
        }
    }

    pub(super) fn render_tab_language(&self, ui: &mut Ui, actions: &mut MenuActions) {
        let t = self.t();
        if submenu_header(ui, t.menu_language) {
            actions.navigate(SettingsTab::Main);
        }

        let choices = Language::ALL.map(|lang| {
            Choice::new(lang.display_name(), lang)
                .selected(self.language == lang)
                .icon(LeadingIcon::Flag(lang.flag()))
        });
        if let Some(lang) = choice_list(ui, choices) {
            actions.set_language = Some(lang);
        }
    }

    pub(crate) fn subtitle_health_warning(&self) -> Option<String> {
        let t = self.t();
        match &self.status.subtitle_health {
            nixo_core::SubtitleHealth::Ok => None,
            nixo_core::SubtitleHealth::ImageBased { codec } => {
                Some(t.health_image_based_desc(codec))
            }
            nixo_core::SubtitleHealth::InactiveTrack => Some(t.health_inactive_desc.to_string()),
        }
    }

    fn video_summary(&self) -> String {
        let t = self.t();
        match (self.status.video_fit, self.status.aspect_ratio) {
            (nixo_core::VideoFit::Fill, _) => t.video_fill_screen.to_string(),
            (nixo_core::VideoFit::Fit, nixo_core::VideoAspect::Auto) => {
                t.video_summary_fit_auto.to_string()
            }
            (nixo_core::VideoFit::Fit, aspect) => match aspect {
                nixo_core::VideoAspect::Auto => t.video_aspect_auto.to_string(),
                nixo_core::VideoAspect::Ratio16_9 => t.video_aspect_16_9.to_string(),
                nixo_core::VideoAspect::Ratio4_3 => t.video_aspect_4_3.to_string(),
                nixo_core::VideoAspect::Ratio235_1 => t.video_aspect_235_1.to_string(),
            },
        }
    }

    fn speed_summary(&self) -> String {
        let speed = self.status.speed;
        if is_normal_speed(speed) {
            self.t().speed_normal.to_string()
        } else {
            format!("{speed:.2}x")
        }
    }

    fn audio_summary(&self) -> String {
        let tracks = &self.status.audio_tracks;
        let t = self.t();
        match tracks.iter().find(|tr| tr.selected) {
            Some(track) => track.localized_display_name(t.audio_track_prefix),
            None if tracks.is_empty() => t.audio_none.to_string(),
            None => t.audio_disabled.to_string(),
        }
    }

    fn subtitle_summary(&self) -> String {
        let t = self.t();
        self.status
            .subtitle_tracks
            .iter()
            .find(|tr| tr.selected && self.status.subtitles_visible)
            .map(|tr| tr.localized_display_name(t.audio_track_prefix))
            .unwrap_or_else(|| t.subtitles_disabled.to_string())
    }
}

fn pick_subtitle_file(t: &crate::i18n::Translations) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title(t.subtitles_dialog_title)
        .add_filter(t.filter_subtitles, &["srt", "vtt", "ass", "ssa", "sub"])
        .add_filter(t.filter_all_files, &["*"])
        .pick_file()
}
