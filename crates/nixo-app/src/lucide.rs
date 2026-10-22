//! Constantes e utilitários da biblioteca Lucide Icons para a interface do Nixo.
#![allow(unused)]

use egui::{Color32, FontId, RichText};

pub const HOME: &str = "\u{e0f5}";
pub const LIST_VIDEO: &str = "\u{e2e2}";
pub const REPEAT: &str = "\u{e146}";
pub const REPEAT_1: &str = "\u{e1fd}";
pub const FOLDER: &str = "\u{e0d7}";
pub const FOLDER_PLUS: &str = "\u{e0d9}";
pub const FOLDER_OPEN: &str = "\u{e247}";
pub const FILM: &str = "\u{e0d0}";
pub const FILE_VIDEO: &str = "\u{e321}";
pub const FILE_AUDIO: &str = "\u{e31a}";
pub const FILE: &str = "\u{e0c0}";
pub const PLAY: &str = "\u{e13c}";
pub const PAUSE: &str = "\u{e12e}";
pub const ARROW_LEFT: &str = "\u{e048}";
pub const CHEVRON_RIGHT: &str = "\u{e06f}";
pub const CHEVRON_LEFT: &str = "\u{e06e}";
pub const SPARKLES: &str = "\u{e412}";
pub const FLASK_CONICAL: &str = "\u{e0d5}";
pub const X: &str = "\u{e1b2}";
pub const TRASH_2: &str = "\u{e18e}";
pub const PLUS: &str = "\u{e13d}";
pub const SETTINGS: &str = "\u{e154}";
pub const SLIDERS: &str = "\u{e162}";
pub const MAXIMIZE: &str = "\u{e112}";
pub const MINIMIZE: &str = "\u{e11a}";
pub const VOLUME_2: &str = "\u{e1ab}";
pub const MUSIC: &str = "\u{e122}";
pub const GLOBE: &str = "\u{e0e8}";
pub const LANGUAGES: &str = "\u{e0fe}";

/// Retorna a `FontId` dedicada para a fonte Lucide Icons.
#[inline]
pub fn font_id(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("lucide".into()))
}

/// Cria um `RichText` configurado com a fonte Lucide e a cor especificada.
#[inline]
pub fn icon(name: &'static str, size: f32, color: Color32) -> RichText {
    RichText::new(name).font(font_id(size)).color(color)
}
