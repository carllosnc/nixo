//! Cabeçalho superior da Home com navegação, breadcrumbs e botões de ação rápida.

use std::path::{Path, PathBuf};

use egui::{
    Align, Align2, Color32, CursorIcon, Layout, Pos2, Rect, RichText, Sense, Stroke, TextureHandle,
    Ui, vec2,
};

use crate::components::icon_secondary;
use crate::components::tokens::*;
use crate::lucide;

const LOGO_PNG: &[u8] = include_bytes!("../../../../assets/logo.png");
const FOLDER_3D_PNG: &[u8] = include_bytes!("../../../../assets/folder_3d.png");
const VIDEO_CLAPPER_PNG: &[u8] = include_bytes!("../../../../assets/video_clapper.png");

/// Carrega a imagem da logo pequena do Nixo a partir dos bytes embutidos.
pub fn load_logo_image() -> Option<egui::ColorImage> {
    let img = image::load_from_memory_with_format(LOGO_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(size, &rgba))
}

/// Carrega a imagem do ícone 3D de pasta a partir dos bytes embutidos.
pub fn load_folder_3d_image() -> Option<egui::ColorImage> {
    let img = image::load_from_memory_with_format(FOLDER_3D_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(size, &rgba))
}

/// Carrega a imagem da claquete de vídeo a partir dos bytes embutidos.
pub fn load_video_clapper_image() -> Option<egui::ColorImage> {
    let img =
        image::load_from_memory_with_format(VIDEO_CLAPPER_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(size, &rgba))
}

/// Ação disparada por interações do usuário na tela inicial.
#[derive(Debug, Clone)]
pub enum HomeAction {
    None,
    AddFolder,
    RemoveFolder(PathBuf),
    OpenFolder(PathBuf),
    GoBack,
    GoRoot,
    OpenFile,
    PlayFile(PathBuf),
    PlayTestSignal,
    SetLanguage(crate::i18n::Language),
}

/// Renderiza o cabeçalho superior da Home no padrão Lucide e tons neutros sem bordas.
pub fn render_header(
    ui: &mut Ui,
    current_path: Option<&Path>,
    breadcrumbs: &[PathBuf],
    logo_texture: Option<&TextureHandle>,
    t: &crate::i18n::Translations,
    current_lang: crate::i18n::Language,
) -> HomeAction {
    let mut action = HomeAction::None;

    ui.horizontal(|ui| {
        if let Some(_path) = current_path {
            if icon_secondary(ui, lucide::ARROW_LEFT, t.header_back).clicked() {
                action = HomeAction::GoBack;
            }

            ui.add_space(10.0);

            // Breadcrumb limpo
            let root_link = ui
                .link(
                    RichText::new(t.header_home)
                        .font(font_body())
                        .color(TEXT_SECONDARY),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(t.header_home_tooltip);
            if root_link.clicked() {
                action = HomeAction::GoRoot;
            }

            for crumb in breadcrumbs {
                ui.label(
                    RichText::new(lucide::CHEVRON_RIGHT)
                        .font(lucide::font_id(12.0))
                        .color(TEXT_MUTED),
                );
                let crumb_name = crumb
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(t.header_folder_fallback);

                let is_current = current_path.is_some_and(|p| p == crumb);
                let display_name = truncate_breadcrumb(crumb_name, 22);

                let crumb_text =
                    RichText::new(display_name)
                        .font(font_body())
                        .color(if is_current {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        });

                let link_resp = ui
                    .link(crumb_text)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text(crumb_name);
                if link_resp.clicked() && !is_current {
                    action = HomeAction::OpenFolder(crumb.clone());
                }
            }
        } else if let Some(tex) = logo_texture {
            let header_h = 38.0_f32;
            let logo_h = 18.0_f32;
            let logo_w = (logo_h * (1024.0_f32 / 270.0_f32)).round();
            let (rect, resp) = ui.allocate_exact_size(vec2(logo_w, header_h), Sense::click());
            let logo_rect = Rect::from_center_size(rect.center(), vec2(logo_w, logo_h));
            ui.painter().image(
                tex.id(),
                logo_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            if resp.clicked() {
                action = HomeAction::GoRoot;
            }
            resp.on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Nixo Media Player");
        } else {
            let header_h = 38.0_f32;
            let (rect, _) = ui.allocate_exact_size(vec2(60.0, header_h), Sense::hover());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Nixo",
                font_card_title(),
                Color32::WHITE,
            );
        }

        // Botões de ação rápida no cabeçalho
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if current_path.is_none() {
                if icon_secondary(
                    ui,
                    lucide::FLASK_CONICAL,
                    "Sinal de teste sintético (60 FPS)",
                )
                .clicked()
                {
                    action = HomeAction::PlayTestSignal;
                }

                ui.add_space(6.0);

                if icon_secondary(ui, lucide::FILE, "Abrir arquivo de mídia").clicked() {
                    action = HomeAction::OpenFile;
                }

                ui.add_space(6.0);

                if icon_secondary(ui, lucide::FOLDER_PLUS, "Adicionar pasta de mídia").clicked() {
                    action = HomeAction::AddFolder;
                }

                ui.add_space(6.0);
            }

            // Botão de seleção de idioma com dropdown
            let lang_popup_id = ui.id().with("home_language_dropdown");
            let lang_btn_resp = icon_secondary(ui, lucide::LANGUAGES, t.header_language);
            if lang_btn_resp.clicked() {
                ui.ctx().memory_mut(|mem| mem.toggle_popup(lang_popup_id));
            }

            if ui.ctx().memory(|mem| mem.is_popup_open(lang_popup_id)) {
                let dropdown_w = 164.0_f32;
                let dropdown_pos = Pos2::new(
                    lang_btn_resp.rect.right() - dropdown_w,
                    lang_btn_resp.rect.bottom() + 6.0,
                );
                let mut dropdown_rect = Rect::NOTHING;

                egui::Area::new(lang_popup_id)
                    .fixed_pos(dropdown_pos)
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        let frame = egui::Frame::NONE
                            .fill(SURFACE_LOW)
                            .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                            .corner_radius(RADIUS_MD)
                            .shadow(egui::Shadow {
                                offset: [0, 8],
                                blur: 24,
                                spread: 0,
                                color: Color32::from_black_alpha(200),
                            })
                            .inner_margin(egui::Margin::same(4));

                        dropdown_rect = frame
                            .show(ui, |ui| {
                                ui.set_width(dropdown_w - 8.0);
                                ui.spacing_mut().item_spacing.y = 2.0;

                                for &lang in &crate::i18n::Language::ALL {
                                    let is_selected = lang == current_lang;
                                    let (row_rect, resp) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), 34.0),
                                        Sense::click(),
                                    );

                                    let hover_t = ui
                                        .ctx()
                                        .animate_bool(resp.id.with("hover"), resp.hovered());
                                    let bg = if resp.is_pointer_button_down_on() {
                                        SURFACE_ACTIVE
                                    } else if is_selected {
                                        lerp_color(SURFACE_MED, SURFACE_HIGH, hover_t)
                                    } else {
                                        lerp_color(Color32::TRANSPARENT, SURFACE_MED, hover_t)
                                    };

                                    ui.painter().rect_filled(row_rect, RADIUS_SM, bg);

                                    // Bandeira vetorial
                                    let flag_rect = Rect::from_center_size(
                                        Pos2::new(row_rect.left() + 18.0, row_rect.center().y),
                                        vec2(20.0, 14.0),
                                    );
                                    crate::app::NixoApp::render_vector_flag(
                                        ui.painter(),
                                        flag_rect,
                                        lang.flag(),
                                    );

                                    // Nome do idioma
                                    let text_color = if is_selected || resp.hovered() {
                                        TEXT_PRIMARY
                                    } else {
                                        TEXT_SECONDARY
                                    };
                                    ui.painter().text(
                                        Pos2::new(row_rect.left() + 38.0, row_rect.center().y),
                                        Align2::LEFT_CENTER,
                                        lang.display_name(),
                                        font_body(),
                                        text_color,
                                    );

                                    // Marcador de seleção ativo
                                    if is_selected {
                                        let check_x = row_rect.right() - 14.0;
                                        let check_y = row_rect.center().y;
                                        ui.painter().add(egui::Shape::line(
                                            vec![
                                                Pos2::new(check_x - 4.5, check_y + 0.2),
                                                Pos2::new(check_x - 1.2, check_y + 3.5),
                                                Pos2::new(check_x + 5.0, check_y - 3.5),
                                            ],
                                            Stroke::new(1.8_f32, crate::app::ACCENT_COLOR),
                                        ));
                                    }

                                    if resp.clicked() {
                                        action = HomeAction::SetLanguage(lang);
                                        ui.ctx().memory_mut(|mem| mem.close_popup());
                                    }
                                }
                            })
                            .response
                            .rect;
                    });

                let mut should_close = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if ui.input(|i| i.pointer.any_pressed())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                    && !dropdown_rect.contains(pos)
                    && !lang_btn_resp.rect.contains(pos)
                {
                    should_close = true;
                }
                if should_close {
                    ui.ctx().memory_mut(|mem| mem.close_popup());
                }
            }
        });
    });

    ui.add_space(14.0);

    // Linha divisória horizontal sutil do cabeçalho
    let (sep_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().line_segment(
        [sep_rect.left_center(), sep_rect.right_center()],
        Stroke::new(1.0_f32, SURFACE_MED),
    );

    action
}

/// Trunca o texto do breadcrumb com reticências se exceder a contagem máxima de caracteres.
fn truncate_breadcrumb(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        let s: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        let trimmed = s.trim_end_matches([' ', '-', '_', '.']);
        format!("{trimmed}…")
    } else {
        text.to_string()
    }
}
