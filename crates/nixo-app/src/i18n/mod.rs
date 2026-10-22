//! Sistema de internacionalização (i18n) do Nixo.
//!
//! Fornece suporte tipado em tempo de compilação para múltiplos idiomas
//! (Inglês, Português, Espanhol), com detecção automática pelo sistema operacional
//! e alternância instantânea e reativa na interface gráfica.

mod en;
mod es;
mod pt;

pub use en::EN;
pub use es::ES;
pub use pt::PT;

use nixo_core::FlagCode;
use serde::{Deserialize, Serialize};

/// Idiomas suportados pelo reprodutor Nixo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    English,
    Portuguese,
    Spanish,
}

impl Language {
    /// Lista de todos os idiomas disponíveis.
    pub const ALL: [Self; 3] = [Self::English, Self::Portuguese, Self::Spanish];

    /// Código ISO 639-1 do idioma.
    #[allow(dead_code)]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Portuguese => "pt",
            Self::Spanish => "es",
        }
    }

    /// Nome oficial no próprio idioma.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Portuguese => "Português",
            Self::Spanish => "Español",
        }
    }

    /// Bandeira vetorial associada ao idioma.
    pub const fn flag(self) -> FlagCode {
        match self {
            Self::English => FlagCode::USA,
            Self::Portuguese => FlagCode::Brazil,
            Self::Spanish => FlagCode::Spain,
        }
    }

    /// Retorna o catálogo de traduções estático para este idioma.
    pub const fn t(self) -> &'static Translations {
        match self {
            Self::English => &EN,
            Self::Portuguese => &PT,
            Self::Spanish => &ES,
        }
    }

    /// Detecta o idioma padrão a partir do sistema operacional ou variáveis de ambiente.
    pub fn detect() -> Self {
        #[cfg(windows)]
        {
            // SAFETY: Chamada pura da Win32 API para obter o idioma da interface do usuário atual.
            unsafe extern "system" {
                fn GetUserDefaultUILanguage() -> u16;
            }
            let lang_id = unsafe { GetUserDefaultUILanguage() };
            let primary = lang_id & 0x03FF;
            match primary {
                0x16 => return Self::Portuguese,
                0x0A => return Self::Spanish,
                0x09 => return Self::English,
                _ => {}
            }
        }

        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_ascii_lowercase();
                if lower.starts_with("pt") {
                    return Self::Portuguese;
                } else if lower.starts_with("es") {
                    return Self::Spanish;
                } else if lower.starts_with("en") {
                    return Self::English;
                }
            }
        }

        Self::English
    }
}

/// Catálogo de todas as cadeias de texto traduzíveis da aplicação.
pub struct Translations {
    // Menu raiz / Navegação
    pub settings_title: &'static str,
    pub menu_video: &'static str,
    pub menu_speed: &'static str,
    pub menu_audio: &'static str,
    pub menu_subtitles: &'static str,
    pub menu_language: &'static str,
    pub menu_always_on_top: &'static str,

    // Aba de Vídeo
    pub video_framing: &'static str,
    pub video_fit_screen: &'static str,
    pub video_fill_screen: &'static str,
    pub video_aspect_ratio: &'static str,
    pub video_aspect_auto: &'static str,
    pub video_aspect_16_9: &'static str,
    pub video_aspect_4_3: &'static str,
    pub video_aspect_235_1: &'static str,
    pub video_summary_fit_auto: &'static str,
    pub video_equalizer_title: &'static str,
    pub video_brightness: &'static str,
    pub video_contrast: &'static str,
    pub video_saturation: &'static str,
    pub video_gamma: &'static str,
    pub video_reset_eq: &'static str,

    // Aba de Velocidade
    pub speed_title: &'static str,
    pub speed_normal: &'static str,

    // Aba de Áudio
    pub audio_track_title: &'static str,
    pub audio_no_tracks: &'static str,
    pub audio_disable: &'static str,
    pub audio_sync: &'static str,
    pub audio_none: &'static str,
    pub audio_disabled: &'static str,
    pub audio_track_prefix: &'static str,

    // Aba de Legendas
    pub subtitles_title: &'static str,
    pub subtitles_disabled: &'static str,
    pub subtitles_load_external: &'static str,
    pub subtitles_style: &'static str,
    pub subtitles_dialog_title: &'static str,
    pub filter_subtitles: &'static str,
    pub filter_all_files: &'static str,

    // Estilo de Legenda
    pub style_title: &'static str,
    pub style_font: &'static str,
    pub style_font_weight: &'static str,
    pub style_size: &'static str,
    pub style_text_color: &'static str,
    pub style_border: &'static str,
    pub style_background: &'static str,
    pub style_shadow: &'static str,
    pub style_position: &'static str,
    pub style_subtitle_height: &'static str,
    pub style_reset_default: &'static str,
    pub style_weight_bold: &'static str,
    pub style_weight_normal: &'static str,
    pub style_border_none: &'static str,
    pub style_border_thin: &'static str,
    pub style_border_medium: &'static str,
    pub style_border_thick: &'static str,
    pub style_shadow_none: &'static str,
    pub style_shadow_light: &'static str,
    pub style_shadow_strong: &'static str,
    pub style_bg_none: &'static str,
    pub style_bg_box_50: &'static str,
    pub style_bg_box_75: &'static str,
    pub style_bg_box_solid: &'static str,
    pub style_pos_bottom: &'static str,
    pub style_pos_raised: &'static str,
    pub style_pos_top: &'static str,

    // Nomes de cores
    pub color_white: &'static str,
    pub color_yellow: &'static str,
    pub color_green: &'static str,
    pub color_cyan: &'static str,
    pub color_magenta: &'static str,
    pub color_red: &'static str,

    // OSD / Controles inferiores
    pub play_tooltip: &'static str,
    pub pause_tooltip: &'static str,
    pub previous_track_tooltip: &'static str,
    pub next_track_tooltip: &'static str,
    pub repeat_off_tooltip: &'static str,
    pub repeat_all_tooltip: &'static str,
    pub repeat_one_tooltip: &'static str,
    #[allow(dead_code)]
    pub forward_5s_tooltip: &'static str,
    pub unmute_tooltip: &'static str,
    pub mute_tooltip: &'static str,
    pub volume_muted_tooltip: &'static str,
    pub fullscreen_enter_tooltip: &'static str,
    pub fullscreen_exit_tooltip: &'static str,
    pub aspect_fit_disable_tooltip: &'static str,
    pub aspect_fit_enable_tooltip: &'static str,
    pub screenshot_tooltip: &'static str,
    pub home_tooltip: &'static str,
    pub playlist_drawer_title: &'static str,
    pub playlist_drawer_tooltip: &'static str,
    pub playlist_close_tooltip: &'static str,
    pub subtitles_disable_tooltip: &'static str,
    pub subtitles_enable_tooltip: &'static str,
    pub mini_player_tooltip: &'static str,
    pub mini_player_restore: &'static str,
    pub subtitle_open_video_first: &'static str,

    // Biblioteca / Home
    pub header_back: &'static str,
    pub header_home: &'static str,
    pub header_home_tooltip: &'static str,
    pub header_folder_fallback: &'static str,
    pub header_language: &'static str,
    pub folders_title: &'static str,
    #[allow(dead_code)]
    pub empty_folders_title: &'static str,
    #[allow(dead_code)]
    pub empty_folders_desc: &'static str,
    pub empty_folders_action: &'static str,
    pub empty_folder_title: &'static str,
    pub empty_folder_desc: &'static str,
    pub loading_folder: &'static str,
    pub calculating: &'static str,
    pub watched_badge: &'static str,
    pub dialog_open_video: &'static str,
    pub filter_video_files: &'static str,

    // Hero Empty State (Redesign da tela inicial limpo e focado)
    pub empty_hero_title: &'static str,
    pub empty_hero_subtitle: &'static str,
    pub toast_folder_removed: &'static str,
    pub toast_history_removed: &'static str,

    // Notificações e Diagnósticos
    #[allow(dead_code)]
    pub health_image_based_badge: &'static str,
    #[allow(dead_code)]
    pub health_inactive_badge: &'static str,
    pub health_inactive_desc: &'static str,
}

impl Translations {
    pub fn volume_level(&self, vol: f32) -> String {
        format!("Volume: {vol:.0}%")
    }

    pub fn subtitle_loaded(&self, filename: &str) -> String {
        match self.menu_language {
            "LANGUAGE" => format!("Subtitle loaded: {filename}"),
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => {
                format!("Legenda carregada: {filename}")
            }
            _ => format!("Subtítulo cargado: {filename}"),
        }
    }

    pub fn settings_speed_tooltip(&self, speed: f32) -> String {
        match self.menu_language {
            "LANGUAGE" => format!("Settings ({speed:.2}x)"),
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => {
                format!("Configurações ({speed:.2}x)")
            }
            _ => format!("Configuración ({speed:.2}x)"),
        }
    }

    pub fn subfolders_count(&self, count: usize) -> String {
        match self.menu_language {
            "LANGUAGE" => format!("Subfolders ({count})"),
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => format!("Subpastas ({count})"),
            _ => format!("Subcarpetas ({count})"),
        }
    }

    pub fn folder_items_count(&self, count: Option<&usize>) -> String {
        match count {
            Some(1) => match self.menu_language {
                "LANGUAGE" => "1 item".to_string(),
                "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => "1 item".to_string(),
                _ => "1 elemento".to_string(),
            },
            Some(c) => match self.menu_language {
                "LANGUAGE" => format!("{c} items"),
                "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => format!("{c} itens"),
                _ => format!("{c} elementos"),
            },
            None => self.calculating.to_string(),
        }
    }

    pub fn resume_badge(&self, time: &str, pct: u32) -> String {
        match self.menu_language {
            "LANGUAGE" => format!("Stopped at {time} ({pct}%)"),
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => {
                format!("Parou em {time} ({pct}%)")
            }
            _ => format!("Pausado en {time} ({pct}%)"),
        }
    }

    pub fn screenshot_saved(&self, filename: &str) -> String {
        match self.menu_language {
            "LANGUAGE" => format!("Screenshot copied and saved: {filename}"),
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => {
                format!("Captura copiada e salva: {filename}")
            }
            _ => format!("Captura copiada y guardada: {filename}"),
        }
    }

    pub fn health_image_based_desc(&self, codec: &str) -> String {
        match self.menu_language {
            "LANGUAGE" => {
                format!("Image-based subtitle ({codec}): cannot change font or text color")
            }
            "IDIOMA" if self.settings_title == "CONFIGURAÇÕES" => {
                format!("Legenda em imagem ({codec}): não permite alterar fonte ou cor")
            }
            _ => format!("Subtítulo en imagen ({codec}): no permite cambiar fuente o color"),
        }
    }

    pub fn color_name(&self, color_pt: &str) -> &'static str {
        match color_pt {
            "Branco" => self.color_white,
            "Amarelo" => self.color_yellow,
            "Verde" => self.color_green,
            "Ciano" => self.color_cyan,
            "Magenta" => self.color_magenta,
            "Vermelho" => self.color_red,
            _ => self.color_white,
        }
    }

    #[allow(dead_code)]
    pub fn subtitle_health_badge(
        &self,
        health: &nixo_core::SubtitleHealth,
    ) -> Option<&'static str> {
        match health {
            nixo_core::SubtitleHealth::Ok => None,
            nixo_core::SubtitleHealth::ImageBased { .. } => Some(self.health_image_based_badge),
            nixo_core::SubtitleHealth::InactiveTrack => Some(self.health_inactive_badge),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_properties() {
        for lang in Language::ALL {
            assert!(!lang.code().is_empty());
            assert!(!lang.display_name().is_empty());
            let t = lang.t();
            assert!(!t.settings_title.is_empty());
            assert!(!t.play_tooltip.is_empty());
            assert!(!t.menu_video.is_empty());
        }
    }

    #[test]
    fn test_translations_coverage() {
        let langs = [Language::English, Language::Portuguese, Language::Spanish];
        for lang in langs {
            let t = lang.t();
            assert!(t.volume_level(80.0).contains("80%"));
            assert!(!t.settings_speed_tooltip(1.5).is_empty());
            assert!(!t.subfolders_count(3).is_empty());
            assert!(!t.folder_items_count(Some(&5)).is_empty());
            assert!(!t.resume_badge("01:23", 45).is_empty());
            assert!(!t.screenshot_saved("shot.png").is_empty());
            assert_eq!(t.color_name("Branco"), t.color_white);
        }
    }

    #[test]
    fn test_subtitle_health_badges() {
        let t_en = Language::English.t();
        let t_pt = Language::Portuguese.t();
        let t_es = Language::Spanish.t();

        let health_img = nixo_core::SubtitleHealth::ImageBased {
            codec: "pgs".to_string(),
        };
        assert_eq!(
            t_en.subtitle_health_badge(&health_img),
            Some("BITMAP SUBTITLE")
        );
        assert_eq!(
            t_pt.subtitle_health_badge(&health_img),
            Some("LEGENDA FIXA")
        );
        assert_eq!(
            t_es.subtitle_health_badge(&health_img),
            Some("SUBTÍTULO BITMAP")
        );

        let health_ok = nixo_core::SubtitleHealth::Ok;
        assert_eq!(t_en.subtitle_health_badge(&health_ok), None);
    }
}
