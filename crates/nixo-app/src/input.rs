use std::path::PathBuf;
use std::time::Instant;

use egui::{Context, Key, ViewportCommand};
use nixo_core::PlayerCommand;

use crate::app::NixoApp;

impl NixoApp {
    pub(crate) fn show_toast(
        &mut self,
        icon: impl Into<crate::toast::ToastIcon>,
        text: impl Into<String>,
    ) {
        self.toast = Some(crate::toast::ToastNotification::new(icon, text));
    }

    pub(crate) fn show_toast_with_path(
        &mut self,
        icon: impl Into<crate::toast::ToastIcon>,
        text: impl Into<String>,
        path: PathBuf,
    ) {
        self.toast = Some(crate::toast::ToastNotification::with_path(icon, text, path));
    }

    pub(crate) fn take_screenshot(&mut self) {
        let path = crate::toast::prepare_screenshot_path();
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        self.dispatch(PlayerCommand::TakeScreenshot(path.clone()));
        crate::toast::copy_screenshot_to_clipboard_async(path.clone());
        let toast_msg = self.t().screenshot_saved(&filename);
        self.show_toast_with_path(crate::toast::ToastIcon::Camera, toast_msg, path);
    }

    pub(crate) fn open_file_dialog(&mut self) {
        let t = self.t();
        let picked = rfd::FileDialog::new()
            .set_title(t.dialog_open_video)
            .add_filter(
                t.filter_video_files,
                &["mp4", "mkv", "webm", "avi", "mov", "m4v", "ts", "flv"],
            )
            .add_filter(t.filter_all_files, &["*"])
            .pick_file();

        if let Some(path) = picked {
            self.dispatch(PlayerCommand::LoadFile(path));
        }
    }

    pub(crate) fn toggle_fullscreen(&mut self, ctx: &Context) {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.is_fullscreen));
        if !self.is_fullscreen {
            Self::apply_windows_titlebar_color();
        }
    }

    pub(crate) fn go_home(&mut self) {
        self.video_texture = None;
        self.aspect_ratio_fit = false;
        self.saved_aspect_ratio_fit = false;
        self.home_state.go_root();
        self.dispatch(PlayerCommand::Stop);
        self.settings_store.observe(self.current_settings());
    }

    pub(crate) fn handle_home_action(&mut self, action: crate::home::HomeAction) {
        use crate::home::HomeAction::*;
        match action {
            None => {}
            AddFolder => {
                if let Some(folders) = rfd::FileDialog::new().pick_folders() {
                    let mut added = false;
                    for folder in folders {
                        if !self.media_folders.contains(&folder) {
                            self.media_folders.push(folder.clone());
                            self.home_state.scanner.invalidate(&folder);
                            added = true;
                        }
                    }
                    if added {
                        self.settings_store.observe(self.current_settings());
                    }
                }
            }
            RemoveFolder(folder) => {
                let folder_name = folder
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Pasta")
                    .to_string();
                self.media_folders.retain(|f| f != &folder);
                self.settings_store.observe(self.current_settings());
                let msg = format!("{}: {}", self.t().toast_folder_removed, folder_name);
                self.show_toast(crate::toast::ToastIcon::Trash, msg);
            }
            OpenFolder(path) => self.home_state.navigate_to(path),
            GoBack => self.home_state.go_back(),
            GoRoot => self.home_state.go_root(),
            OpenFile => self.open_file_dialog(),
            PlayFile(path) => self.dispatch(PlayerCommand::LoadFile(path)),
            PlayTestSignal => self.dispatch(PlayerCommand::Play),
            SetLanguage(lang) => {
                self.language = lang;
                self.settings_store.observe(self.current_settings());
            }
        }
    }

    pub(crate) fn handle_shortcuts_and_drops(&mut self, ctx: &Context) {
        // Amplifica a velocidade de rolagem (scroll vertical mais ágil e veloz na biblioteca e listas)
        ctx.input_mut(|i| {
            i.smooth_scroll_delta *= 1.85;
            i.raw_scroll_delta *= 1.85;
        });

        let scrolled = ctx.input(|i| {
            i.smooth_scroll_delta.length_sq() > 0.0 || i.raw_scroll_delta.length_sq() > 0.0
        });
        if scrolled && self.home_state.keyboard_nav_active {
            self.home_state.keyboard_nav_active = false;
        }

        // Suporte a Drag & Drop de arquivos de vídeo e pastas de mídia direto na janela
        let has_dropped = ctx.input(|i| !i.raw.dropped_files.is_empty());
        if has_dropped {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            let mut added_folders = false;
            for file in dropped_files {
                if let Some(path) = file.path {
                    if path.is_dir() {
                        if !self.media_folders.contains(&path) {
                            self.media_folders.push(path.clone());
                            self.home_state.scanner.invalidate(&path);
                            added_folders = true;
                        }
                    } else {
                        self.dispatch(PlayerCommand::LoadFile(path));
                        break;
                    }
                }
            }
            if added_folders {
                self.settings_store.observe(self.current_settings());
            }
        }

        // Máquina de estados de hover inspirada no YouTube:
        // 1. Mouse fora da janela -> oculta rápido (300ms).
        // 2. Mouse sobre os controles/popups/arrastando -> nunca oculta.
        // 3. Mouse parado sobre o vídeo -> oculta após 2s (e esconde o cursor).
        let (pointer_pos, any_down) = ctx.input(|i| (i.pointer.hover_pos(), i.pointer.any_down()));
        let screen_h = ctx.screen_rect().height();
        let any_popup_open = ctx.memory(|m| m.any_popup_open());

        if let Some(pos) = pointer_pos {
            self.mouse_left_window_at = None;

            // Verifica se o mouse entrou na janela ou se moveu
            let moved = self
                .last_pointer_pos
                .is_none_or(|prev| (pos - prev).length_sq() > 2.0);
            if moved {
                self.last_mouse_move = Instant::now();
                if self.home_state.keyboard_nav_active {
                    self.home_state.keyboard_nav_active = false;
                }
            }
            if any_down && self.home_state.keyboard_nav_active {
                self.home_state.keyboard_nav_active = false;
            }
            self.last_pointer_pos = Some(pos);

            // Se estiver sobre a barra inferior (últimos 90px), topo, arrastando ou com popup aberto, trava visível
            let is_over_controls = pos.y >= (screen_h - 90.0).max(0.0) || pos.y <= 55.0;
            if is_over_controls || any_down || any_popup_open || ctx.is_pointer_over_area() {
                self.last_mouse_move = Instant::now();
            }
        } else {
            self.last_pointer_pos = None;
            if any_down || any_popup_open {
                self.mouse_left_window_at = None;
                self.last_mouse_move = Instant::now();
            } else if self.mouse_left_window_at.is_none() {
                self.mouse_left_window_at = Some(Instant::now());
            }
        }

        let is_home = self.status.state == nixo_core::PlaybackState::Stopped;

        if is_home {
            // Navegação de teclado na biblioteca de pastas e mídias
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                self.home_state.select_prev();
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                let total = self.home_state.total_items(&self.media_folders);
                self.home_state.select_next(total);
            }
            // Seta Direita navega para dentro de pastas/subpastas (não inicia reprodução de vídeo)
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                let action = self.home_state.activate_selected(&self.media_folders);
                if matches!(action, crate::home::HomeAction::OpenFolder(_)) {
                    self.handle_home_action(action);
                }
            }
            // Enter ativa o item selecionado: entra na pasta ou inicia a reprodução da mídia
            if ctx.input(|i| i.key_pressed(Key::Enter)) {
                let action = self.home_state.activate_selected(&self.media_folders);
                self.handle_home_action(action);
            }
            // Tecla Delete remove a pasta selecionada (na raiz) ou limpa o histórico de reprodução (dentro de pastas)
            if ctx.input(|i| i.key_pressed(Key::Delete)) {
                if self.home_state.current_path.is_none() {
                    if let Some(folder) = self
                        .media_folders
                        .get(self.home_state.selected_index)
                        .cloned()
                    {
                        let folder_name = folder
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Pasta")
                            .to_string();
                        self.media_folders.retain(|f| f != &folder);
                        if self.media_folders.is_empty() {
                            self.home_state.selected_index = 0;
                            self.home_state.keyboard_nav_active = false;
                        } else if self.home_state.selected_index >= self.media_folders.len() {
                            self.home_state.selected_index = self.media_folders.len() - 1;
                        }
                        self.settings_store.observe(self.current_settings());
                        let msg = format!("{}: {}", self.t().toast_folder_removed, folder_name);
                        self.show_toast(crate::toast::ToastIcon::Trash, msg);
                    }
                } else if let Some(current) = self.home_state.current_path.clone() {
                    let content = self.home_state.scanner.get_or_request(&current);
                    let subdirs_len = content.subdirs.len();
                    if self.home_state.selected_index >= subdirs_len {
                        let file_idx = self.home_state.selected_index - subdirs_len;
                        if let Some(file) = content.files.get(file_idx)
                            && self.watch_history.remove(&file.path).is_some()
                        {
                            self.settings_store.observe(self.current_settings());
                            self.show_toast(
                                crate::toast::ToastIcon::Trash,
                                self.t().toast_history_removed,
                            );
                        }
                    }
                }
            }
            if (ctx.input(|i| i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::Backspace)))
                && self.home_state.current_path.is_some()
            {
                self.home_state.go_back();
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                if self.is_fullscreen {
                    self.toggle_fullscreen(ctx);
                } else if self.home_state.current_path.is_some() {
                    self.home_state.go_back();
                }
            }
        } else {
            // Controles de reprodução de vídeo quando há mídia em execução
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                self.dispatch(PlayerCommand::TogglePlayPause);
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                self.dispatch(PlayerCommand::SeekRelative(5.0));
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                self.dispatch(PlayerCommand::SeekRelative(-5.0));
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                let next_vol = (self.status.volume + 5.0).min(100.0);
                self.dispatch(PlayerCommand::SetVolume(next_vol));
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                let next_vol = (self.status.volume - 5.0).max(0.0);
                self.dispatch(PlayerCommand::SetVolume(next_vol));
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) && !any_popup_open {
                if self.is_fullscreen {
                    self.toggle_fullscreen(ctx);
                } else {
                    self.home_state.keyboard_nav_active = true;
                    self.dispatch(PlayerCommand::Stop);
                }
            }
        }

        if ctx.input(|i| i.key_pressed(Key::M)) {
            self.dispatch(PlayerCommand::ToggleMute);
        }
        if ctx.input(|i| i.key_pressed(Key::V)) {
            self.dispatch(PlayerCommand::ToggleSubtitleVisibility);
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::S))
            || ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && i.key_pressed(Key::P))
        {
            self.take_screenshot();
        } else if ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && i.key_pressed(Key::S))
        {
            self.dispatch(PlayerCommand::CycleSubtitleTrack);
        }
        if ctx.input(|i| i.key_pressed(Key::B)) {
            self.dispatch(PlayerCommand::CycleAudioTrack);
        }
        if ctx.input(|i| i.key_pressed(Key::F) || i.key_pressed(Key::F11)) {
            self.toggle_fullscreen(ctx);
        }
        if ctx.input(|i| i.key_pressed(Key::T)) {
            self.always_on_top = !self.always_on_top;
            self.settings_dirty = true;
            let level = if self.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
        }
        if ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::A)) {
            self.toggle_aspect_ratio_fit(ctx);
        } else if ctx.input(|i| !i.modifiers.command && i.key_pressed(Key::A)) {
            let next_fit = match self.status.video_fit {
                nixo_core::VideoFit::Fit => nixo_core::VideoFit::Fill,
                nixo_core::VideoFit::Fill => nixo_core::VideoFit::Fit,
            };
            self.dispatch(PlayerCommand::SetVideoFit(next_fit));
        }
        if ctx.input(|i| i.key_pressed(Key::Z)) {
            let next = ((self.status.sub_delay - 0.1) * 10.0).round() / 10.0;
            self.dispatch(PlayerCommand::SetSubDelay(next));
        }
        if ctx.input(|i| i.key_pressed(Key::X)) {
            let next = ((self.status.sub_delay + 0.1) * 10.0).round() / 10.0;
            self.dispatch(PlayerCommand::SetSubDelay(next));
        }
        if ctx.input(|i| i.key_pressed(Key::J)) {
            let next = ((self.status.audio_delay - 0.1) * 10.0).round() / 10.0;
            self.dispatch(PlayerCommand::SetAudioDelay(next));
        }
        if ctx.input(|i| i.key_pressed(Key::K)) {
            let next = ((self.status.audio_delay + 0.1) * 10.0).round() / 10.0;
            self.dispatch(PlayerCommand::SetAudioDelay(next));
        }
        if ctx.input(|i| i.key_pressed(Key::Home)) {
            self.go_home();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::O)) {
            self.open_file_dialog();
        }
    }

    pub(crate) fn active_video_aspect(&self) -> f32 {
        self.status
            .aspect_ratio
            .ratio_f64()
            .map(|r| r as f32)
            .unwrap_or_else(|| {
                if let Some(tex) = &self.video_texture {
                    let sz = tex.size_vec2();
                    (sz.x / sz.y.max(1.0)).max(0.1)
                } else {
                    16.0 / 9.0
                }
            })
            .clamp(0.2, 5.0)
    }

    pub(crate) fn snap_window_to_aspect(&mut self, ctx: &Context) {
        if self.is_fullscreen {
            return;
        }
        let current_size = ctx.screen_rect().size();
        if current_size.x <= 100.0 || current_size.y <= 100.0 {
            return;
        }
        let aspect = self.active_video_aspect();
        let target_h = (current_size.x / aspect).round().max(180.0);
        let target_size = egui::Vec2::new(current_size.x, target_h);
        self.last_window_size = target_size;
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(target_size));
    }

    pub(crate) fn toggle_aspect_ratio_fit(&mut self, ctx: &Context) {
        if self.status.state == nixo_core::PlaybackState::Stopped {
            return;
        }
        self.aspect_ratio_fit = !self.aspect_ratio_fit;
        self.saved_aspect_ratio_fit = self.aspect_ratio_fit;
        if self.aspect_ratio_fit {
            self.snap_window_to_aspect(ctx);
        }
    }
}
