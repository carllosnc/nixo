use nixo_core::{PlayerCommand, Playlist, RepeatMode};
use std::path::Path;
use std::time::Duration;

use crate::app::NixoApp;

impl NixoApp {
    /// Sincroniza a playlist atual com a pasta do arquivo aberto caso o arquivo pertença
    /// a outro diretório ou se a playlist estiver vazia.
    pub(crate) fn sync_playlist_for_file(&mut self, path: &Path) {
        let current_dir = path.parent();
        let current_pl_dir = self.playlist.current().and_then(|p| p.parent());

        if current_dir != current_pl_dir || self.playlist.is_empty() {
            if let Some(dir) = current_dir {
                let current_repeat = self.playlist.repeat_mode();
                self.playlist = Playlist::from_folder_with_active(dir, path);
                self.playlist.set_repeat_mode(current_repeat);
            }
        } else {
            self.playlist.set_current_by_path(path);
        }
    }

    /// Executa a transição para o próximo vídeo da lista.
    pub(crate) fn play_next(&mut self) {
        if let Some(next_file) = self.playlist.advance_next() {
            let file_name = next_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            self.dispatch(PlayerCommand::LoadFile(next_file));
            self.show_toast(crate::toast::ToastIcon::Play, file_name);
        }
    }

    /// Executa a transição para o vídeo anterior da lista.
    pub(crate) fn play_previous(&mut self) {
        if let Some(prev_file) = self.playlist.advance_previous() {
            let file_name = prev_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            self.dispatch(PlayerCommand::LoadFile(prev_file));
            self.show_toast(crate::toast::ToastIcon::Play, file_name);
        }
    }

    /// Alterna sequencialmente o modo de repetição (Off -> All -> One -> Off)
    /// e exibe notificação toast na interface.
    pub(crate) fn cycle_repeat_mode(&mut self) {
        let mode = self.playlist.cycle_repeat_mode();
        self.settings_dirty = true;
        let toast_msg = match mode {
            RepeatMode::Off => self.t().repeat_off_tooltip,
            RepeatMode::All => self.t().repeat_all_tooltip,
            RepeatMode::One => self.t().repeat_one_tooltip,
        };
        self.show_toast(crate::toast::ToastIcon::Play, toast_msg);
    }

    /// Trata o evento de fim de mídia (`PlaybackState::Ended`).
    pub(crate) fn handle_playback_ended(&mut self) {
        match self.playlist.repeat_mode() {
            RepeatMode::One => {
                self.dispatch(PlayerCommand::Seek(Duration::ZERO));
                self.dispatch(PlayerCommand::Play);
            }
            RepeatMode::All => {
                self.play_next();
            }
            RepeatMode::Off => {
                if self.playlist.has_next() {
                    self.play_next();
                } else {
                    self.record_current_playback();
                    self.go_home();
                }
            }
        }
    }
}
