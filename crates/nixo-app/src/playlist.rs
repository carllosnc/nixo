use nixo_core::{PlayerCommand, Playlist, RepeatMode};
use std::path::Path;
use std::time::Duration;

use crate::app::NixoApp;

/// Modo de rolagem ao navegar ou focar faixas na lista de reprodução.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlaylistScrollTarget {
    /// Alinha o item ao topo do drawer (ex.: abertura da lista ou tecla Home).
    Top(usize),
    /// Mantém o item visível na tela: se ultrapassar a borda inferior vai ao bottom, se ultrapassar o topo vai ao topo.
    EnsureVisible(usize),
}

impl PlaylistScrollTarget {
    /// Calcula o deslocamento vertical ideal para o item sem travar no centro.
    #[inline]
    pub(crate) fn compute_offset(
        &self,
        current_scroll: f32,
        avail_h: f32,
        row_total_h: f32,
        max_scroll: f32,
    ) -> f32 {
        match *self {
            Self::Top(idx) => (idx as f32 * row_total_h).clamp(0.0, max_scroll),
            Self::EnsureVisible(idx) => {
                let item_top = idx as f32 * row_total_h;
                let item_bottom = (idx + 1) as f32 * row_total_h;
                if item_top < current_scroll {
                    item_top.clamp(0.0, max_scroll)
                } else if item_bottom > current_scroll + avail_h {
                    (item_bottom - avail_h).clamp(0.0, max_scroll)
                } else {
                    current_scroll.clamp(0.0, max_scroll)
                }
            }
        }
    }
}

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
            if self.playlist_drawer_open {
                let cur = self.playlist.current_index().unwrap_or(0);
                self.playlist_focused_index = cur;
                self.playlist_scroll_target = Some(PlaylistScrollTarget::EnsureVisible(cur));
            }
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
            if self.playlist_drawer_open {
                let cur = self.playlist.current_index().unwrap_or(0);
                self.playlist_focused_index = cur;
                self.playlist_scroll_target = Some(PlaylistScrollTarget::EnsureVisible(cur));
            }
        }
    }

    /// Executa a reprodução direta de um item da playlist pelo seu índice.
    pub(crate) fn play_playlist_index(&mut self, index: usize) {
        if let Some(file) = self.playlist.set_current_by_index(index) {
            let file_name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            self.dispatch(PlayerCommand::LoadFile(file));
            self.show_toast(crate::toast::ToastIcon::Play, file_name);
        }
    }

    /// Alterna a visibilidade do drawer lateral de playlist.
    pub(crate) fn toggle_playlist_drawer(&mut self) {
        self.playlist_drawer_open = !self.playlist_drawer_open;
        if self.playlist_drawer_open {
            let cur = self.playlist.current_index().unwrap_or(0);
            self.playlist_focused_index = cur;
            self.playlist_keyboard_nav_active = true;
            self.playlist_scroll_target = Some(PlaylistScrollTarget::Top(cur));
        } else {
            self.playlist_keyboard_nav_active = false;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_target_top() {
        let target = PlaylistScrollTarget::Top(5);
        let offset = target.compute_offset(100.0, 460.0, 46.0, 1000.0);
        assert_eq!(offset, 230.0); // 5 * 46
    }

    #[test]
    fn test_scroll_target_ensure_visible() {
        let target = PlaylistScrollTarget::EnsureVisible(2);
        // Item 2 (top: 92, bottom: 138) já visível entre 0 e 460 -> mantém scroll 0.0
        assert_eq!(target.compute_offset(0.0, 460.0, 46.0, 1000.0), 0.0);

        // Item 10 (top: 460, bottom: 506) abaixo do viewport [0, 460] -> vai ao bottom (506 - 460 = 46)
        let target_down = PlaylistScrollTarget::EnsureVisible(10);
        assert_eq!(target_down.compute_offset(0.0, 460.0, 46.0, 1000.0), 46.0);

        // Item 1 (top: 46, bottom: 92) acima do viewport [100, 560] -> vai ao topo (46)
        let target_up = PlaylistScrollTarget::EnsureVisible(1);
        assert_eq!(target_up.compute_offset(100.0, 460.0, 46.0, 1000.0), 46.0);
    }
}
