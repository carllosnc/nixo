//! Menu de configurações (popover da engrenagem).
//!
//! Organização:
//! - [`style`]: design tokens (cores, tamanhos, fontes) — restilize aqui.
//! - [`components`]: biblioteca de componentes únicos e reutilizáveis.
//! - [`tabs`] / [`subtitle_style`]: telas compostas apenas por componentes.

mod components;
mod style;
mod subtitle_style;
mod tabs;

use egui::{Align2, Context, Id, Key, Pos2, Rect, Ui};
use nixo_core::PlayerCommand;

use self::components::PopoverFrame;
use self::style::menu_style;
use crate::app::{NixoApp, SettingsTab};

/// Ações produzidas por uma tela durante o frame, aplicadas após a renderização.
struct MenuActions {
    next_tab: SettingsTab,
    command: Option<PlayerCommand>,
    close: bool,
}

impl MenuActions {
    fn new(current: SettingsTab) -> Self {
        Self {
            next_tab: current,
            command: None,
            close: false,
        }
    }

    /// Navega para outra tela do menu.
    fn navigate(&mut self, tab: SettingsTab) {
        self.next_tab = tab;
    }

    /// Envia um comando ao player mantendo o menu aberto.
    fn send(&mut self, cmd: PlayerCommand) {
        self.command = Some(cmd);
    }

    /// Envia um comando ao player e fecha o menu.
    fn send_and_close(&mut self, cmd: PlayerCommand) {
        self.send(cmd);
        self.close = true;
    }
}

impl NixoApp {
    pub(crate) fn render_settings_popup(&mut self, ctx: &Context, gear_resp: &egui::Response) {
        let popup_id = Id::new("settings_popup_menu");
        if gear_resp.clicked() {
            let was_open = ctx.memory(|mem| mem.is_popup_open(popup_id));
            if !was_open {
                self.settings_tab = SettingsTab::Main;
            }
            ctx.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        if !ctx.memory(|mem| mem.is_popup_open(popup_id)) {
            return;
        }

        let st = menu_style();
        let screen = ctx.screen_rect();
        let half_w = st.popover_width * 0.5;
        let anchor_x = gear_resp.rect.center().x.clamp(
            screen.left() + half_w + st.popover_margin_left,
            screen.right() - half_w - st.popover_margin_right,
        );
        let anchor_pos = Pos2::new(anchor_x, gear_resp.rect.top() - st.popover_anchor_gap);

        let mut actions = MenuActions::new(self.settings_tab);
        let mut popup_rect = Rect::NOTHING;

        egui::Area::new(popup_id)
            .pivot(Align2::CENTER_BOTTOM)
            .fixed_pos(anchor_pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_width(st.popover_width);
                popup_rect =
                    PopoverFrame::show(ui, |ui| self.render_settings_tab(ui, &mut actions))
                        .response
                        .rect;
            });

        if let Some(cmd) = actions.command.take() {
            self.dispatch(cmd);
        }
        self.settings_tab = actions.next_tab;

        let mut should_close = actions.close || ctx.input(|i| i.key_pressed(Key::Escape));
        if ctx.input(|i| i.pointer.any_pressed())
            && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
            && !popup_rect.contains(pos)
            && !gear_resp.rect.contains(pos)
        {
            should_close = true;
        }

        if should_close {
            self.settings_tab = SettingsTab::Main;
            ctx.memory_mut(|mem| mem.close_popup());
        }
    }

    fn render_settings_tab(&self, ui: &mut Ui, actions: &mut MenuActions) {
        match self.settings_tab {
            SettingsTab::Main => self.render_tab_main(ui, actions),
            SettingsTab::Speed => self.render_tab_speed(ui, actions),
            SettingsTab::Audio => self.render_tab_audio(ui, actions),
            SettingsTab::Subtitles => self.render_tab_subtitles(ui, actions),
            SettingsTab::SubtitleStyle
            | SettingsTab::SubtitleFont
            | SettingsTab::SubtitleWeight
            | SettingsTab::SubtitleSize
            | SettingsTab::SubtitleColor
            | SettingsTab::SubtitleBorder
            | SettingsTab::SubtitleBackground
            | SettingsTab::SubtitleShadow
            | SettingsTab::SubtitlePosition => self.render_subtitle_style_tabs(ui, actions),
        }
    }
}
