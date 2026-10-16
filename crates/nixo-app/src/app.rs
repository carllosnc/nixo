use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{
    Align, Color32, ColorImage, Context, CornerRadius, FontId, Key, Layout, Margin, Pos2, Rect,
    RichText, Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2, ViewportCommand,
};
use nixo_core::{MediaBackend, PlaybackState, PlayerCommand, PlayerStatus, format_duration};
use tracing::error;

const ACCENT_COLOR: Color32 = Color32::from_rgb(255, 107, 53);
const BG_DARK: Color32 = Color32::from_rgb(12, 13, 18);
const PANEL_BG: Color32 = Color32::from_rgba_premultiplied(20, 22, 30, 235);

pub struct NixoApp {
    backend: Box<dyn MediaBackend>,
    status: PlayerStatus,
    video_texture: Option<TextureHandle>,
    last_frame_id: u64,
    is_fullscreen: bool,
    last_mouse_move: Instant,
    last_pointer_pos: Option<Pos2>,
    error_banner: Option<String>,
}

impl NixoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        Self::configure_theme(&cc.egui_ctx);
        cc.egui_ctx
            .send_viewport_cmd(ViewportCommand::Visible(true));
        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Focus);

        let mut backend = nixo_engine::create_default_backend();
        if let Some(path) = initial_file
            && let Err(err) = backend.execute(PlayerCommand::LoadFile(path))
        {
            error!("Erro ao abrir arquivo inicial: {err}");
        }

        let status = backend.status();

        Self {
            backend,
            status,
            video_texture: None,
            last_frame_id: 0,
            is_fullscreen: false,
            last_mouse_move: Instant::now(),
            last_pointer_pos: None,
            error_banner: None,
        }
    }

    fn configure_theme(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_DARK;
        visuals.window_fill = BG_DARK;
        visuals.selection.bg_fill = ACCENT_COLOR;
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT_COLOR);
        ctx.set_visuals(visuals);
    }

    fn dispatch(&mut self, cmd: PlayerCommand) {
        if let Err(err) = self.backend.execute(cmd) {
            error!("Comando falhou: {err}");
            self.error_banner = Some(err.to_string());
        } else {
            self.error_banner = None;
        }
        self.status = self.backend.status();
    }

    fn open_file_dialog(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Abrir Vídeo no Nixo")
            .add_filter(
                "Arquivos de Vídeo",
                &["mp4", "mkv", "webm", "avi", "mov", "m4v", "ts", "flv"],
            )
            .add_filter("Todos os Arquivos", &["*"])
            .pick_file();

        if let Some(path) = picked {
            self.dispatch(PlayerCommand::LoadFile(path));
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &Context) {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.is_fullscreen));
    }

    fn handle_shortcuts_and_drops(&mut self, ctx: &Context) {
        // Suporte a Drag & Drop de arquivos de vídeo direto na janela
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            if let Some(path) = file.path {
                self.dispatch(PlayerCommand::LoadFile(path));
                break;
            }
        }

        // Detecta movimento do mouse para exibir/ocultar o OSD
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        if pointer_pos != self.last_pointer_pos {
            self.last_pointer_pos = pointer_pos;
            self.last_mouse_move = Instant::now();
        }

        // Atalhos de teclado padrão de players de vídeo
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
        if ctx.input(|i| i.key_pressed(Key::M)) {
            self.dispatch(PlayerCommand::ToggleMute);
        }
        if ctx.input(|i| i.key_pressed(Key::F) || i.key_pressed(Key::F11)) {
            self.toggle_fullscreen(ctx);
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.is_fullscreen {
            self.toggle_fullscreen(ctx);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::O)) {
            self.open_file_dialog();
        }
    }

    fn update_video_texture(&mut self, ctx: &Context, viewport_size: Vec2) {
        let target_w = (viewport_size.x as u32).clamp(320, 1920);
        let target_h = (viewport_size.y as u32).clamp(180, 1080);

        if let Some(frame) = self.backend.poll_video_frame((target_w, target_h)) {
            self.last_frame_id = frame.frame_id;
            let image = ColorImage::from_rgba_premultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );

            match &mut self.video_texture {
                Some(tex) => tex.set(image, TextureOptions::LINEAR),
                None => {
                    let tex = ctx.load_texture("nixo-video-surface", image, TextureOptions::LINEAR);
                    self.video_texture = Some(tex);
                }
            }
        }
    }

    fn render_top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("▶ NIXO")
                    .font(FontId::proportional(17.0))
                    .strong()
                    .color(ACCENT_COLOR),
            );
            ui.separator();

            if let Some(meta) = &self.status.metadata {
                ui.label(
                    RichText::new(&meta.title)
                        .font(FontId::proportional(14.5))
                        .color(Color32::from_gray(230)),
                );
            } else {
                ui.label(
                    RichText::new("Nenhuma mídia carregada")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_gray(140)),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("📂 Abrir Vídeo (Ctrl+O)").clicked() {
                    self.open_file_dialog();
                }

                ui.separator();
                ui.label(
                    RichText::new(format!("Engine: {}", self.status.backend_name))
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_gray(145)),
                );
            });
        });
    }

    fn render_bottom_controls(&mut self, ctx: &Context, ui: &mut Ui) {
        // 1. Barra de Progresso (Seekbar)
        let total_secs = self.status.duration.as_secs_f64().max(0.001);
        let mut current_secs = self.status.position.as_secs_f64().clamp(0.0, total_secs);

        ui.spacing_mut().slider_width = ui.available_width();
        let slider_resp = ui.add(
            egui::Slider::new(&mut current_secs, 0.0..=total_secs)
                .show_value(false)
                .trailing_fill(true),
        );
        if slider_resp.changed() {
            self.dispatch(PlayerCommand::Seek(Duration::from_secs_f64(current_secs)));
        }

        ui.add_space(4.0);

        // 2. Botões de Controle, Tempo, Velocidade e Volume
        ui.horizontal(|ui| {
            let play_label = match self.status.state {
                PlaybackState::Playing => "⏸ Pausar",
                _ => "▶ Reproduzir",
            };

            if ui
                .add_sized(
                    [110.0, 30.0],
                    egui::Button::new(RichText::new(play_label).strong()),
                )
                .clicked()
            {
                self.dispatch(PlayerCommand::TogglePlayPause);
            }

            if ui
                .add_sized([40.0, 30.0], egui::Button::new("⏹"))
                .on_hover_text("Parar")
                .clicked()
            {
                self.dispatch(PlayerCommand::Stop);
                self.video_texture = None;
            }

            if ui
                .add_sized([45.0, 30.0], egui::Button::new("⏪ 5s"))
                .clicked()
            {
                self.dispatch(PlayerCommand::SeekRelative(-5.0));
            }

            if ui
                .add_sized([45.0, 30.0], egui::Button::new("5s ⏩"))
                .clicked()
            {
                self.dispatch(PlayerCommand::SeekRelative(5.0));
            }

            ui.separator();

            let time_text = format!(
                "{} / {}",
                format_duration(self.status.position),
                format_duration(self.status.duration)
            );
            ui.label(
                RichText::new(time_text)
                    .font(FontId::monospace(13.5))
                    .color(Color32::from_gray(220)),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let fs_label = if self.is_fullscreen {
                    "🗗 Sair"
                } else {
                    "⛶ Tela Cheia"
                };
                if ui
                    .button(fs_label)
                    .on_hover_text("Atalho: F ou F11")
                    .clicked()
                {
                    self.toggle_fullscreen(ctx);
                }

                ui.separator();

                // Seletor de Velocidade de Reprodução
                let mut speed = self.status.speed;
                egui::ComboBox::from_id_salt("playback_speed")
                    .selected_text(format!("{speed:.2}x"))
                    .width(68.0)
                    .show_ui(ui, |ui| {
                        for preset in [0.5_f32, 0.75, 1.0, 1.25, 1.5, 2.0] {
                            if ui
                                .selectable_value(&mut speed, preset, format!("{preset:.2}x"))
                                .clicked()
                            {
                                self.dispatch(PlayerCommand::SetSpeed(preset));
                            }
                        }
                    });

                ui.separator();

                // Controle de Volume
                let mut vol = if self.status.muted {
                    0.0
                } else {
                    self.status.volume
                };
                ui.spacing_mut().slider_width = 95.0;
                if ui
                    .add(egui::Slider::new(&mut vol, 0.0..=100.0).show_value(false))
                    .changed()
                {
                    if self.status.muted && vol > 0.0 {
                        self.dispatch(PlayerCommand::ToggleMute);
                    }
                    self.dispatch(PlayerCommand::SetVolume(vol));
                }

                let mute_icon = if self.status.muted || self.status.volume <= 0.1 {
                    "🔇"
                } else if self.status.volume < 50.0 {
                    "🔉"
                } else {
                    "🔊"
                };
                if ui.button(mute_icon).on_hover_text("Mudo (M)").clicked() {
                    self.dispatch(PlayerCommand::ToggleMute);
                }
            });
        });
    }
}

impl eframe::App for NixoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.status = self.backend.status();
        self.handle_shortcuts_and_drops(ctx);

        let hide_osd = self.is_fullscreen
            && self.status.state == PlaybackState::Playing
            && self.last_mouse_move.elapsed() > Duration::from_millis(2500);

        if !hide_osd {
            egui::TopBottomPanel::top("nixo_top_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(PANEL_BG)
                        .inner_margin(Margin::symmetric(14, 8)),
                )
                .show(ctx, |ui| {
                    self.render_top_bar(ui);
                });

            egui::TopBottomPanel::bottom("nixo_bottom_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(PANEL_BG)
                        .inner_margin(Margin::symmetric(16, 10)),
                )
                .show(ctx, |ui| {
                    self.render_bottom_controls(ctx, ui);
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG_DARK))
            .show(ctx, |ui| {
                let avail = ui.available_size();
                self.update_video_texture(ctx, avail);

                let (rect, response) = ui.allocate_exact_size(avail, Sense::click());
                if response.double_clicked() {
                    self.toggle_fullscreen(ctx);
                } else if response.clicked() && self.status.state != PlaybackState::Stopped {
                    self.dispatch(PlayerCommand::TogglePlayPause);
                }

                if let Some(tex) = &self.video_texture {
                    // Preserva proporção 16:9 (ou tamanho da textura) com letterbox/pillarbox
                    let tex_size = tex.size_vec2();
                    let aspect = (tex_size.x / tex_size.y.max(1.0)).max(0.1);
                    let mut draw_w = rect.width();
                    let mut draw_h = draw_w / aspect;
                    if draw_h > rect.height() {
                        draw_h = rect.height();
                        draw_w = draw_h * aspect;
                    }

                    let video_rect =
                        Rect::from_center_size(rect.center(), Vec2::new(draw_w, draw_h));
                    ui.painter().image(
                        tex.id(),
                        video_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // Tela de boas-vindas do Nixo quando nenhum vídeo está aberto
                    let card_size = Vec2::new(440.0, 210.0).min(rect.size() * 0.9);
                    let card_rect = Rect::from_center_size(rect.center(), card_size);
                    ui.painter().rect(
                        card_rect,
                        CornerRadius::same(12),
                        Color32::from_rgb(20, 22, 30),
                        Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 65)),
                        egui::StrokeKind::Inside,
                    );

                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(card_rect.shrink(24.0))
                            .layout(Layout::top_down(Align::Center)),
                    );

                    child_ui.label(
                        RichText::new("NIXO MEDIA PLAYER")
                            .font(FontId::proportional(22.0))
                            .strong()
                            .color(ACCENT_COLOR),
                    );
                    child_ui.add_space(8.0);
                    child_ui.label(
                        RichText::new(
                            "Arraste um arquivo de vídeo para esta janela ou clique abaixo",
                        )
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_gray(190)),
                    );
                    child_ui.add_space(18.0);

                    child_ui.horizontal(|ui| {
                        let total_btn_w = 290.0;
                        let pad = ((ui.available_width() - total_btn_w) * 0.5).max(0.0);
                        ui.add_space(pad);

                        if ui
                            .add_sized([145.0, 36.0], egui::Button::new("📂 Abrir Arquivo"))
                            .clicked()
                        {
                            self.open_file_dialog();
                        }
                        if ui
                            .add_sized([135.0, 36.0], egui::Button::new("▶ Testar Sinal GPU"))
                            .clicked()
                        {
                            self.dispatch(PlayerCommand::Play);
                        }
                    });
                }
            });

        // Enquanto o vídeo estiver reproduzindo, sincroniza repintura direto com o VSync do wgpu
        if self.status.state == PlaybackState::Playing {
            ctx.request_repaint();
        }
    }
}
