use std::time::{Duration, Instant};

use egui::{
    Align, Color32, Context, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke, Ui, vec2,
};
use nixo_core::{PlaybackState, PlayerCommand, format_duration};

use crate::app::{ACCENT_COLOR, NixoApp};
use crate::icons::ControlIcon;

const BTN_HEIGHT: f32 = 44.0;

/// Níveis de densidade e responsividade da barra de controle inferior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OsdTier {
    /// >= 780px: Todos os controles, volume inline e indicador de playlist completo.
    Full,
    /// 580px .. 780px: Oculta atalhos de baixa frequência (Screenshot, Trava de Proporção).
    Medium,
    /// 420px .. 580px: Oculta Repetição e Dashboard; volume vira expand-on-hover.
    Compact,
    /// < 420px: Modo ultra-compacto minimalista (Play, Volume, Tempo resumido, Configurações, Tela Cheia).
    Mini,
}

impl OsdTier {
    pub(crate) fn from_width(width: f32) -> Self {
        if width >= 780.0 {
            Self::Full
        } else if width >= 580.0 {
            Self::Medium
        } else if width >= 420.0 {
            Self::Compact
        } else {
            Self::Mini
        }
    }

    /// Retorna a quantidade de botões exibidos à direita para dimensionamento exato.
    pub(crate) fn right_button_count(self) -> usize {
        match self {
            Self::Full => 7,
            Self::Medium => 5,
            Self::Compact => 3,
            Self::Mini => 2,
        }
    }

    pub(crate) fn show_aspect_and_screenshot(self) -> bool {
        matches!(self, Self::Full)
    }

    pub(crate) fn show_repeat_and_home(self) -> bool {
        matches!(self, Self::Full | Self::Medium)
    }

    pub(crate) fn show_cc(self) -> bool {
        !matches!(self, Self::Mini)
    }

    pub(crate) fn show_track_navigation(self) -> bool {
        !matches!(self, Self::Mini)
    }

    pub(crate) fn show_total_duration(self) -> bool {
        !matches!(self, Self::Mini)
    }

    pub(crate) fn show_playlist_counter(self) -> bool {
        matches!(self, Self::Full)
    }
}

impl NixoApp {
    /// Renderiza a barra inferior com seekbar de ponta a ponta e controles responsivos.
    pub(crate) fn render_bottom_controls(&mut self, ctx: &Context, ui: &mut Ui) {
        let total_secs = self.status.duration.as_secs_f64().max(0.001);
        let backend_secs = self.status.position.as_secs_f64().clamp(0.0, total_secs);

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

        let total_w = ui.available_width();
        let tier = OsdTier::from_width(total_w);

        self.render_seekbar_row(ctx, ui, &mut current_secs, total_secs, total_w);
        ui.add_space(2.0);

        let (row_rect, _resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), BTN_HEIGHT), Sense::hover());

        let btn_gap = 6.0;
        let right_count = tier.right_button_count();
        let right_w =
            (right_count as f32) * 40.0 + ((right_count.saturating_sub(1)) as f32) * btn_gap;

        // O bloco esquerdo ocupa apenas o espaço restante seguro, prevenindo qualquer sobreposição.
        let left_w = (row_rect.width() - right_w - 12.0).max(40.0);
        let left_rect = Rect::from_min_size(row_rect.left_top(), vec2(left_w, BTN_HEIGHT));
        let right_rect = Rect::from_min_size(
            Pos2::new(row_rect.right() - right_w, row_rect.top()),
            vec2(right_w, BTN_HEIGHT),
        );

        self.render_left_controls(ctx, ui, left_rect, current_secs, tier);
        self.render_right_controls(ctx, ui, right_rect, tier);
    }

    /// Renderiza a seekbar superior com debounce, throttling e thumbnail de pré-visualização.
    fn render_seekbar_row(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        current_secs: &mut f64,
        total_secs: f64,
        total_w: f32,
    ) {
        let slider_resp =
            Self::render_compact_slider(ui, current_secs, 0.0..=total_secs, total_w, 3.0, 4.0);

        if slider_resp.changed() {
            let target_dur = Duration::from_secs_f64(*current_secs);
            self.seek_override = Some((*current_secs, Instant::now()));
            self.status.position = target_dur;

            let should_dispatch = !slider_resp.dragged()
                || self.last_seek_dispatch.elapsed() >= Duration::from_millis(60);
            if should_dispatch {
                self.last_seek_dispatch = Instant::now();
                self.dispatch(PlayerCommand::Seek(target_dur));
            }
        } else if slider_resp.drag_stopped()
            && let Some((target_secs, _)) = self.seek_override
            && self.last_seek_dispatch.elapsed() >= Duration::from_millis(30)
        {
            self.seek_override = Some((target_secs, Instant::now()));
            self.last_seek_dispatch = Instant::now();
            self.dispatch(PlayerCommand::Seek(Duration::from_secs_f64(target_secs)));
        }

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
    }

    /// Renderiza os controles da esquerda: Navegação, Play/Pause, Volume adaptativo e Timestamps.
    fn render_left_controls(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        rect: Rect,
        current_secs: f64,
        tier: OsdTier,
    ) {
        let mut left_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        left_ui.spacing_mut().item_spacing.x = 6.0;
        left_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        if tier.show_track_navigation() {
            let prev_tooltip = self.playlist.peek_previous().map_or_else(
                || self.t().previous_track_tooltip.to_string(),
                |p| {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                    format!("{}: {name}", self.t().previous_track_tooltip)
                },
            );
            if Self::render_ghost_icon_btn_enabled(
                &mut left_ui,
                vec2(38.0, BTN_HEIGHT),
                ControlIcon::PreviousTrack,
                &prev_tooltip,
                self.playlist.has_previous(),
            )
            .clicked()
            {
                self.play_previous();
            }
        }

        let is_playing = self.status.state == PlaybackState::Playing;
        let (play_icon, play_tooltip) = if is_playing {
            (ControlIcon::Pause, self.t().pause_tooltip)
        } else {
            (ControlIcon::Play, self.t().play_tooltip)
        };
        if Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(44.0, BTN_HEIGHT),
            play_icon,
            play_tooltip,
        )
        .clicked()
        {
            self.dispatch(PlayerCommand::TogglePlayPause);
        }

        if tier.show_track_navigation() {
            let next_tooltip = self.playlist.peek_next().map_or_else(
                || self.t().next_track_tooltip.to_string(),
                |p| {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                    format!("{}: {name}", self.t().next_track_tooltip)
                },
            );
            if Self::render_ghost_icon_btn_enabled(
                &mut left_ui,
                vec2(38.0, BTN_HEIGHT),
                ControlIcon::NextTrack,
                &next_tooltip,
                self.playlist.has_next(),
            )
            .clicked()
            {
                self.play_next();
            }
        }

        let vol_tooltip = if self.status.muted {
            self.t().unmute_tooltip
        } else {
            self.t().mute_tooltip
        };
        let vol_btn_resp = Self::render_ghost_icon_btn(
            &mut left_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::Volume {
                level: self.status.volume,
                muted: self.status.muted,
            },
            vol_tooltip,
        );
        if vol_btn_resp.clicked() {
            self.dispatch(PlayerCommand::ToggleMute);
        }

        self.render_adaptive_volume_slider(ctx, &mut left_ui, &vol_btn_resp, tier);
        left_ui.add_space(6.0);

        let display_pos = Duration::from_secs_f64(current_secs);
        let time_text = if tier.show_total_duration() {
            format!(
                "{} / {}",
                format_duration(display_pos),
                format_duration(self.status.duration)
            )
        } else {
            format_duration(display_pos)
        };

        let time_font_size = if tier == OsdTier::Mini { 12.0 } else { 13.5 };
        left_ui.label(
            RichText::new(time_text)
                .font(FontId::monospace(time_font_size))
                .color(Color32::from_gray(215)),
        );

        if self.playlist.len() > 1 && tier.show_playlist_counter() {
            let cur_idx = self.playlist.current_index().map(|i| i + 1).unwrap_or(1);
            let pill_text = format!("•  {cur_idx} / {}", self.playlist.len());
            let color = if self.playlist_drawer_open {
                ACCENT_COLOR
            } else {
                Color32::from_gray(185)
            };
            let pl_btn = left_ui.add(
                egui::Button::new(
                    RichText::new(pill_text)
                        .font(FontId::monospace(13.0))
                        .color(color),
                )
                .frame(false),
            );
            if pl_btn.clicked() {
                self.toggle_playlist_drawer();
            }
            if pl_btn.hovered() {
                left_ui
                    .ctx()
                    .set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            crate::components::show_tooltip(
                left_ui.ctx(),
                &pl_btn,
                self.t().playlist_drawer_tooltip,
            );
        }
    }

    /// Renderiza o slider de volume com suporte a animação expansível no hover em telas estreitas.
    fn render_adaptive_volume_slider(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        vol_btn_resp: &egui::Response,
        tier: OsdTier,
    ) {
        let vol_anim_id = ui.id().with("osd_vol_expand_anim");
        let vol_hover_id = ui.id().with("osd_vol_was_hovered");
        let prev_vol_hovered = ctx.data(|d| d.get_temp::<bool>(vol_hover_id).unwrap_or(false));

        let max_w = match tier {
            OsdTier::Full => 80.0,
            OsdTier::Medium => 56.0,
            OsdTier::Compact | OsdTier::Mini => 64.0,
        };

        let slider_w = match tier {
            OsdTier::Full | OsdTier::Medium => max_w,
            OsdTier::Compact | OsdTier::Mini => {
                let expand = vol_btn_resp.hovered() || prev_vol_hovered;
                let factor = ctx.animate_bool_with_time(vol_anim_id, expand, 0.16);
                (max_w * factor).round()
            }
        };

        if slider_w <= 4.0 {
            ctx.data_mut(|d| d.insert_temp(vol_hover_id, false));
            return;
        }

        let vol = if self.status.muted {
            0.0
        } else {
            self.status.volume
        };
        let mut vol_f64 = vol as f64;
        let vol_resp =
            Self::render_compact_slider(ui, &mut vol_f64, 0.0..=100.0, slider_w, 3.0, 3.5);
        if vol_resp.changed() {
            let next_vol = vol_f64 as f32;
            if self.status.muted && next_vol > 0.0 {
                self.dispatch(PlayerCommand::ToggleMute);
            }
            self.dispatch(PlayerCommand::SetVolume(next_vol));
        }

        let is_vol_active =
            vol_resp.hovered() || vol_resp.dragged() || vol_resp.is_pointer_button_down_on();
        ctx.data_mut(|d| d.insert_temp(vol_hover_id, is_vol_active));

        let vol_tt_id = vol_resp.id.with("vol_slider_tooltip");
        let vol_alpha = ctx.animate_bool_with_time(vol_tt_id, is_vol_active, 0.12);
        let vol_text = if self.status.muted {
            self.t().volume_muted_tooltip.to_string()
        } else {
            self.t().volume_level(self.status.volume)
        };
        crate::components::render_tooltip(ctx, vol_tt_id, vol_resp.rect, &vol_text, vol_alpha);
    }

    /// Renderiza os controles da direita filtrados pela prioridade do `OsdTier`.
    fn render_right_controls(&mut self, ctx: &Context, ui: &mut Ui, rect: Rect, tier: OsdTier) {
        let mut right_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        right_ui.spacing_mut().item_spacing.x = 6.0;
        right_ui.spacing_mut().interact_size.y = BTN_HEIGHT;

        let fs_tooltip = if self.is_fullscreen {
            self.t().fullscreen_exit_tooltip
        } else {
            self.t().fullscreen_enter_tooltip
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

        if tier.show_aspect_and_screenshot() {
            if Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                ControlIcon::Pip {
                    active: self.is_mini_player,
                },
                self.t().mini_player_tooltip,
            )
            .clicked()
            {
                self.toggle_mini_player(ctx);
            }

            let fit_tooltip = if self.aspect_ratio_fit {
                self.t().aspect_fit_disable_tooltip
            } else {
                self.t().aspect_fit_enable_tooltip
            };
            let fit_icon = ControlIcon::AspectFit {
                active: self.aspect_ratio_fit,
            };
            if Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                fit_icon,
                fit_tooltip,
            )
            .clicked()
            {
                self.toggle_aspect_ratio_fit(ctx);
            }

            if Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                ControlIcon::Screenshot,
                self.t().screenshot_tooltip,
            )
            .clicked()
            {
                self.take_screenshot();
            }
        }

        if tier.show_repeat_and_home() {
            let rep_tooltip = match self.playlist.repeat_mode() {
                nixo_core::RepeatMode::Off => self.t().repeat_off_tooltip,
                nixo_core::RepeatMode::All => self.t().repeat_all_tooltip,
                nixo_core::RepeatMode::One => self.t().repeat_one_tooltip,
            };
            let rep_icon = ControlIcon::Repeat {
                mode: self.playlist.repeat_mode(),
            };
            if Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                rep_icon,
                rep_tooltip,
            )
            .clicked()
            {
                self.cycle_repeat_mode();
            }
        }

        let speed_tooltip = self.t().settings_speed_tooltip(self.status.speed);
        let gear_resp = Self::render_ghost_icon_btn(
            &mut right_ui,
            vec2(40.0, BTN_HEIGHT),
            ControlIcon::SettingsGear,
            &speed_tooltip,
        );

        if tier.show_cc() {
            let subs_active = self.status.subtitles_visible
                && self.status.subtitle_tracks.iter().any(|t| t.selected);
            let cc_tooltip = if let Some(warn_msg) = self.subtitle_health_warning() {
                warn_msg
            } else if subs_active {
                self.t().subtitles_disable_tooltip.to_string()
            } else {
                self.t().subtitles_enable_tooltip.to_string()
            };
            let cc_icon = ControlIcon::Subtitles {
                active: subs_active,
            };
            let cc_resp = Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                cc_icon,
                &cc_tooltip,
            );
            if self.status.subtitle_health.is_warning() {
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
        }

        if tier.show_repeat_and_home()
            && Self::render_ghost_icon_btn(
                &mut right_ui,
                vec2(40.0, BTN_HEIGHT),
                ControlIcon::Home,
                self.t().home_tooltip,
            )
            .clicked()
        {
            self.go_home();
        }

        self.render_settings_popup(ctx, &gear_resp);
    }
}
