use std::time::{Duration, Instant};

use egui::{
    Align, Color32, Context, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke, Ui, vec2,
};
use nixo_core::{PlaybackState, PlayerCommand, format_duration};

use crate::app::NixoApp;
use crate::icons::ControlIcon;

impl NixoApp {
    /// Renderiza a barra inferior de ponta a ponta com seekbar, timestamps e controles de mídia.
    pub(crate) fn render_bottom_controls(&mut self, ctx: &Context, ui: &mut Ui) {
        let total_secs = self.status.duration.as_secs_f64().max(0.001);
        let backend_secs = self.status.position.as_secs_f64().clamp(0.0, total_secs);

        // Mantém a posição clicada/arrastada enquanto o demuxer do vídeo conclui o seek assíncrono,
        // impedindo que a barra volte para o tempo antigo por alguns quadros.
        let mut current_secs = if let Some((target_secs, seek_instant)) = self.seek_override {
            if seek_instant.elapsed() < Duration::from_millis(400)
                && (backend_secs - target_secs).abs() > 0.5
            {
                target_secs.clamp(0.0, total_secs)
            } else {
                self.seek_override = None;
                backend_secs
            }
        } else {
            backend_secs
        };

        const BTN_HEIGHT: f32 = 40.0;

        // 1. Linha Superior: Seekbar 100% de ponta a ponta (estilo YouTube)
        let total_w = ui.available_width();
        let slider_resp =
            Self::render_compact_slider(ui, &mut current_secs, 0.0..=total_secs, total_w, 2.5, 3.5);
        let is_dragging = slider_resp.dragged();
        let drag_ended = slider_resp.drag_stopped();

        if slider_resp.changed() {
            let target_dur = Duration::from_secs_f64(current_secs);
            self.seek_override = Some((current_secs, Instant::now()));
            self.status.position = target_dur;

            // Se for um clique direto (não arrasto contínuo), dispara o seek imediatamente na hora.
            // Se estiver arrastando (scrubbing), aplica throttling de 60ms para fluidez sem sobrecarregar o demuxer.
            let should_dispatch = if is_dragging {
                self.last_seek_dispatch.elapsed() >= Duration::from_millis(60)
            } else {
                true
            };

            if should_dispatch {
                self.last_seek_dispatch = Instant::now();
                self.dispatch(PlayerCommand::Seek(target_dur));
            }
        } else if drag_ended && let Some((target_secs, _)) = self.seek_override {
            // Ao soltar o mouse após arrastar, garante que a posição final exata seja enviada
            if self.last_seek_dispatch.elapsed() >= Duration::from_millis(30) {
                self.seek_override = Some((target_secs, Instant::now()));
                self.last_seek_dispatch = Instant::now();
                self.dispatch(PlayerCommand::Seek(Duration::from_secs_f64(target_secs)));
            }
        }

        // Pré-visualização de frame (160x90 + timestamp) estilo YouTube ao passar o mouse na seekbar
        if (slider_resp.hovered() || slider_resp.is_pointer_button_down_on())
            && (self.status.state != PlaybackState::Stopped || self.status.metadata.is_some())
            && let Some(pointer_pos) =
                ctx.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()))
        {
            let s_rect = slider_resp.rect;
            let norm = ((pointer_pos.x - s_rect.left()) / s_rect.width()).clamp(0.0, 1.0);
            let hover_secs = (norm as f64) * total_secs;
            self.render_seek_preview_tooltip(ctx, s_rect, pointer_pos.x, hover_secs);
        }

        ui.add_space(2.0);

        // 2. Linha Inferior: Controles organizados exatamente como no YouTube
        // Esquerda: Play/Pause | Próximo | Mute + Volume Slider | 14:28 / 42:15
        // Direita: Legendas CC | Configurações (⚙ Engrenagem) | Abrir Arquivo | Tela Cheia
        let (row_rect, _resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), BTN_HEIGHT), Sense::hover());

        // A. Bloco da Esquerda (Play, Próximo, Volume, Tempo)
        let left_w = (row_rect.width() - 155.0).max(280.0);
        let left_rect = Rect::from_min_size(row_rect.left_top(), vec2(left_w, BTN_HEIGHT));
        let mut left_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(left_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        left_ui.spacing_mut().item_spacing.x = 6.0;
        left_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        // 1. Play / Pause
        let play_icon = match self.status.state {
            PlaybackState::Playing => ControlIcon::Pause,
            _ => ControlIcon::Play,
        };
        let play_tooltip = match self.status.state {
            PlaybackState::Playing => "Pausar (Espaço)",
            _ => "Reproduzir (Espaço)",
        };
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(42.0, BTN_HEIGHT),
            play_icon,
            play_tooltip,
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::TogglePlayPause);
        }

        // 2. Próximo (+5s)
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(38.0, BTN_HEIGHT),
            ControlIcon::NextTrack,
            "Avançar 5s (Seta Direita)",
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::SeekRelative(5.0));
        }

        // 3. Ícone de Volume / Mute
        let vol_tooltip = if self.status.muted {
            "Ativar som (M)"
        } else {
            "Desativar som (M)"
        };
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(38.0, BTN_HEIGHT),
            ControlIcon::Volume {
                level: self.status.volume,
                muted: self.status.muted,
            },
            vol_tooltip,
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::ToggleMute);
        }

        // 4. Slider de Volume inline (estilo YouTube: diretamente ao lado do alto-falante)
        let vol = if self.status.muted {
            0.0
        } else {
            self.status.volume
        };
        let mut vol_f64 = vol as f64;
        let vol_resp =
            Self::render_compact_slider(&mut left_ui, &mut vol_f64, 0.0..=100.0, 85.0, 2.5, 3.0);
        if vol_resp.changed() {
            let next_vol = vol_f64 as f32;
            if self.status.muted && next_vol > 0.0 {
                self.dispatch(PlayerCommand::ToggleMute);
            }
            self.dispatch(PlayerCommand::SetVolume(next_vol));
        }

        // Tooltip flutuante do slider de volume ("Volume: 80%" / "Volume: Mudo")
        let is_vol_active =
            vol_resp.hovered() || vol_resp.dragged() || vol_resp.is_pointer_button_down_on();
        let vol_tt_id = vol_resp.id.with("vol_slider_tooltip");
        let vol_alpha = ctx.animate_bool_with_time(vol_tt_id, is_vol_active, 0.12);
        let vol_text = if self.status.muted {
            "Volume: Mudo".to_string()
        } else {
            format!("Volume: {:.0}%", self.status.volume)
        };
        Self::render_player_tooltip(ctx, vol_tt_id, vol_resp.rect, &vol_text, vol_alpha);

        left_ui.add_space(8.0);

        // 5. Timestamps no estilo YouTube: "14:28 / 42:15" alinhado à esquerda após o volume
        let display_pos = Duration::from_secs_f64(current_secs);
        let time_text = format!(
            "{} / {}",
            format_duration(display_pos),
            format_duration(self.status.duration)
        );
        left_ui.label(
            RichText::new(time_text)
                .font(FontId::monospace(13.5))
                .color(Color32::from_gray(215)),
        );

        // B. Bloco da Direita (CC, Configurações ⚙, Abrir Arquivo, Capturar Tela, Proporção, Tela Cheia)
        let has_sub_warning = self.status.subtitle_health.is_warning();
        let right_w = 285.0;
        let left_max_w = (row_rect.width() - right_w - 8.0).max(260.0);
        let _ = left_max_w;
        let right_rect = Rect::from_min_size(
            Pos2::new(row_rect.right() - right_w, row_rect.top()),
            vec2(right_w, BTN_HEIGHT),
        );
        let mut right_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(right_rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        right_ui.spacing_mut().item_spacing.x = 6.0;
        right_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        // 1. Tela Cheia (na extremidade direita)
        let fs_tooltip = if self.is_fullscreen {
            "Sair da tela cheia (F)"
        } else {
            "Tela cheia (F)"
        };
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::Fullscreen {
                is_fullscreen: self.is_fullscreen,
            },
            fs_tooltip,
        )
        .clicked()
        {
            self.toggle_fullscreen(ctx);
        }

        // 2. Trava de Proporção da Janela (ao lado de Tela Cheia)
        let fit_tooltip = if self.aspect_ratio_fit {
            "Desativar ajuste de proporção da janela (Shift+A)"
        } else {
            "Ajustar janela na proporção do vídeo sem bordas pretas (Shift+A)"
        };
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::AspectFit {
                active: self.aspect_ratio_fit,
            },
            fit_tooltip,
        )
        .clicked()
        {
            self.toggle_aspect_ratio_fit(ctx);
        }

        // 3. Captura de Tela (Ctrl+S / P)
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::Screenshot,
            "Capturar tela (Ctrl+S / P)",
        )
        .clicked()
        {
            self.take_screenshot();
        }

        // 4. Abrir Arquivo (Ctrl+O)
        if Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::OpenFile,
            "Abrir arquivo (Ctrl+O)",
        )
        .clicked()
        {
            self.open_file_dialog();
        }

        // 3. Configurações (Ícone de Engrenagem ⚙ estilo YouTube)
        let speed = self.status.speed;
        let speed_tooltip = format!("Configurações ({speed:.2}x)");
        let gear_resp = Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::SettingsGear,
            &speed_tooltip,
        );

        // 4. Legendas CC (Atalho rápido estilo YouTube + Badge vermelho de alerta quando inativa/fixa)
        let subs_active =
            self.status.subtitles_visible && self.status.subtitle_tracks.iter().any(|t| t.selected);
        let cc_tooltip = if let Some(warn_msg) = self.status.subtitle_health.warning_message() {
            warn_msg
        } else if subs_active {
            "Desativar legendas (V)".to_string()
        } else {
            "Ativar legendas (V)".to_string()
        };
        let cc_resp = Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::Subtitles {
                active: subs_active,
            },
            &cc_tooltip,
        );
        if has_sub_warning {
            let dot_center = Pos2::new(cc_resp.rect.right() - 8.0, cc_resp.rect.top() + 9.0);
            right_ui
                .painter()
                .circle_filled(dot_center, 4.0, Color32::from_rgb(235, 45, 45));
            right_ui.painter().circle_stroke(
                dot_center,
                4.0,
                Stroke::new(1.2_f32, Color32::from_rgb(18, 18, 22)),
            );
        }
        if cc_resp.clicked() {
            self.dispatch(PlayerCommand::ToggleSubtitleVisibility);
        }

        self.render_settings_popup(ctx, &gear_resp);
    }
}
