pub mod subtitle;

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

pub use subtitle::*;

#[derive(Debug, Error)]
pub enum NixoError {
    #[error("Falha ao inicializar o motor de mídia: {0}")]
    InitError(String),

    #[error("Arquivo de mídia não encontrado ou inválido: {0}")]
    InvalidMedia(String),

    #[error("Erro durante reprodução: {0}")]
    PlaybackError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Ended,
}

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub title: String,
    pub path: PathBuf,
    pub duration: Duration,
    pub video_width: u32,
    pub video_height: u32,
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    /// Pixels em formato RGBA8 (4 bytes por pixel).
    pub rgba: Vec<u8>,
    pub frame_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlagCode {
    #[default]
    Generic,
    Brazil,
    Portugal,
    USA,
    UK,
    Japan,
    Spain,
    France,
    Germany,
    Italy,
    SouthKorea,
    China,
    Russia,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInfo {
    pub id: i64,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub selected: bool,
}

impl TrackInfo {
    pub fn display_name(&self) -> String {
        self.localized_display_name("Faixa")
    }

    pub fn localized_display_name(&self, prefix: &str) -> String {
        let lang_upper = self
            .lang
            .as_deref()
            .map(|l| l.trim().to_ascii_uppercase())
            .filter(|l| !l.is_empty());

        let title_clean = self
            .title
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty());

        match (title_clean, lang_upper) {
            (Some(t), Some(l)) => {
                let t_upper = t.to_ascii_uppercase();
                if t_upper == format!("FAIXA {}", self.id)
                    || t_upper == format!("TRACK {}", self.id)
                    || t_upper == format!("PISTA {}", self.id)
                    || t_upper == l
                {
                    format!("{prefix} {} - {l}", self.id)
                } else if t_upper.ends_with(&format!(" - {l}")) {
                    t.to_string()
                } else {
                    format!("{t} - {l}")
                }
            }
            (Some(t), None) => t.to_string(),
            (None, Some(l)) => format!("{prefix} {} - {l}", self.id),
            (None, None) => format!("{prefix} {}", self.id),
        }
    }

    pub fn flag_code(&self) -> FlagCode {
        detect_flag_code(self.lang.as_deref(), self.title.as_deref())
    }
}

pub fn detect_flag_code(lang: Option<&str>, title: Option<&str>) -> FlagCode {
    let lang_lower = lang.map(|s| s.to_ascii_lowercase());
    let title_lower = title.map(|s| s.to_ascii_lowercase());

    if let Some(t) = &title_lower {
        if t.contains("portugal") || t.contains("pt-pt") || t.contains("europe") {
            return FlagCode::Portugal;
        }
        if t.contains("brasil") || t.contains("brazil") || t.contains("pt-br") {
            return FlagCode::Brazil;
        }
    }

    if let Some(l) = &lang_lower {
        match l.as_str() {
            "pt-pt" => return FlagCode::Portugal,
            "pt-br" | "pob" | "pt" | "por" => return FlagCode::Brazil,
            "en-gb" => return FlagCode::UK,
            "en" | "eng" | "en-us" => return FlagCode::USA,
            "ja" | "jpn" => return FlagCode::Japan,
            "es" | "spa" => return FlagCode::Spain,
            "fr" | "fra" | "fre" => return FlagCode::France,
            "de" | "deu" | "ger" => return FlagCode::Germany,
            "it" | "ita" => return FlagCode::Italy,
            "ko" | "kor" => return FlagCode::SouthKorea,
            "zh" | "zho" | "chi" => return FlagCode::China,
            "ru" | "rus" => return FlagCode::Russia,
            _ => {}
        }
    }

    if let Some(t) = &title_lower {
        if t.contains("portugu") {
            return FlagCode::Brazil;
        }
        if t.contains("english") || t.contains("inglês") || t.contains("ingles") {
            return FlagCode::USA;
        }
        if t.contains("japan") || t.contains("japon") {
            return FlagCode::Japan;
        }
        if t.contains("span") || t.contains("espan") {
            return FlagCode::Spain;
        }
        if t.contains("french") || t.contains("franc") {
            return FlagCode::France;
        }
        if t.contains("german") || t.contains("alem") {
            return FlagCode::Germany;
        }
        if t.contains("ital") {
            return FlagCode::Italy;
        }
        if t.contains("korean") || t.contains("corean") {
            return FlagCode::SouthKorea;
        }
        if t.contains("chin") {
            return FlagCode::China;
        }
        if t.contains("russ") {
            return FlagCode::Russia;
        }
    }

    FlagCode::Generic
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoAspect {
    #[default]
    Auto,
    Ratio16_9,
    Ratio4_3,
    Ratio235_1,
}

impl VideoAspect {
    pub const ALL: [Self; 4] = [
        Self::Auto,
        Self::Ratio16_9,
        Self::Ratio4_3,
        Self::Ratio235_1,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Original (Auto)",
            Self::Ratio16_9 => "16:9 Widescreen",
            Self::Ratio4_3 => "4:3 Clássico",
            Self::Ratio235_1 => "2.35:1 Cinema",
        }
    }

    pub fn ratio_f64(self) -> Option<f64> {
        match self {
            Self::Auto => None,
            Self::Ratio16_9 => Some(16.0 / 9.0),
            Self::Ratio4_3 => Some(4.0 / 3.0),
            Self::Ratio235_1 => Some(2.35),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFit {
    #[default]
    Fit,
    Fill,
}

impl VideoFit {
    pub const ALL: [Self; 2] = [Self::Fit, Self::Fill];

    pub fn label(self) -> &'static str {
        match self {
            Self::Fit => "Ajustar à tela",
            Self::Fill => "Preencher tela",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    LoadFile(PathBuf),
    Play,
    Pause,
    TogglePlayPause,
    Stop,
    Seek(Duration),
    SeekRelative(f64),
    SetVolume(f32),
    ToggleMute,
    SetSpeed(f32),
    SetAudioTrack(Option<i64>),
    SetSubtitleTrack(Option<i64>),
    ToggleSubtitleVisibility,
    CycleAudioTrack,
    CycleSubtitleTrack,
    LoadSubtitleFile(PathBuf),
    SetSubtitleStyle(SubtitleStyle),
    SetSubDelay(f64),
    SetAudioDelay(f64),
    SetAspectRatio(VideoAspect),
    SetVideoFit(VideoFit),
    TakeScreenshot(PathBuf),
}

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub state: PlaybackState,
    pub position: Duration,
    pub duration: Duration,
    /// Volume entre 0.0 e 100.0
    pub volume: f32,
    pub muted: bool,
    pub speed: f32,
    pub metadata: Option<MediaMetadata>,
    pub backend_name: &'static str,
    pub audio_tracks: Vec<TrackInfo>,
    pub subtitle_tracks: Vec<TrackInfo>,
    pub subtitles_visible: bool,
    pub subtitle_style: SubtitleStyle,
    pub subtitle_health: SubtitleHealth,
    pub sub_delay: f64,
    pub audio_delay: f64,
    pub aspect_ratio: VideoAspect,
    pub video_fit: VideoFit,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 80.0,
            muted: false,
            speed: 1.0,
            metadata: None,
            backend_name: "Nenhum",
            audio_tracks: Vec::new(),
            subtitle_tracks: Vec::new(),
            subtitles_visible: true,
            subtitle_style: SubtitleStyle::default(),
            subtitle_health: SubtitleHealth::default(),
            sub_delay: 0.0,
            audio_delay: 0.0,
            aspect_ratio: VideoAspect::default(),
            video_fit: VideoFit::default(),
        }
    }
}

/// Contrato desacoplado para qualquer motor de mídia no Nixo (libmpv, GStreamer, FFmpeg, Symphonia, etc.).
pub trait MediaBackend: Send {
    /// Nome identificador do motor ativo.
    fn name(&self) -> &'static str;

    /// Executa um comando de controle no player.
    fn execute(&mut self, command: PlayerCommand) -> Result<(), NixoError>;

    /// Retorna o estado atual da reprodução (tempo, volume, metadados).
    fn status(&mut self) -> PlayerStatus;

    /// Solicita o próximo quadro de vídeo decodificado na resolução alvo `(largura, altura)`.
    /// Retorna `Some(VideoFrame)` apenas quando há um novo quadro para atualizar na GPU.
    fn poll_video_frame(&mut self, target_size: (u32, u32)) -> Option<VideoFrame>;

    /// Solicita uma miniatura de pré-visualização (ex: 160x90) para o instante `position` (hover na seekbar).
    /// Opera de forma não-bloqueante: dispara a busca em background e retorna a miniatura mais recente pronta.
    fn poll_preview_frame(&mut self, _position: Duration) -> Option<VideoFrame> {
        None
    }
}

/// Formata um `Duration` como `MM:SS` ou `HH:MM:SS`.
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(65)), "01:05");
        assert_eq!(format_duration(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn test_track_info_display_name() {
        let t1 = TrackInfo {
            id: 1,
            title: Some("Português".to_string()),
            lang: Some("por".to_string()),
            selected: true,
        };
        assert_eq!(t1.display_name(), "Português - POR");

        let t2 = TrackInfo {
            id: 2,
            title: None,
            lang: Some("eng".to_string()),
            selected: false,
        };
        assert_eq!(t2.display_name(), "Faixa 2 - ENG");

        let t3 = TrackInfo {
            id: 3,
            title: None,
            lang: None,
            selected: false,
        };
        assert_eq!(t3.display_name(), "Faixa 3");

        let t4 = TrackInfo {
            id: 1,
            title: Some("Faixa 1".to_string()),
            lang: Some("eng".to_string()),
            selected: true,
        };
        assert_eq!(t4.display_name(), "Faixa 1 - ENG");
    }

    #[test]
    fn test_detect_flag_code() {
        assert_eq!(detect_flag_code(Some("pt"), None), FlagCode::Brazil);
        assert_eq!(detect_flag_code(Some("pt-BR"), None), FlagCode::Brazil);
        assert_eq!(detect_flag_code(Some("pt-PT"), None), FlagCode::Portugal);
        assert_eq!(detect_flag_code(Some("eng"), None), FlagCode::USA);
        assert_eq!(detect_flag_code(Some("en-GB"), None), FlagCode::UK);
        assert_eq!(detect_flag_code(Some("ja"), None), FlagCode::Japan);
        assert_eq!(
            detect_flag_code(None, Some("Áudio em Japonês")),
            FlagCode::Japan
        );
        assert_eq!(detect_flag_code(Some("es"), None), FlagCode::Spain);
        assert_eq!(detect_flag_code(Some("fr"), None), FlagCode::France);
        assert_eq!(detect_flag_code(Some("de"), None), FlagCode::Germany);
        assert_eq!(detect_flag_code(Some("it"), None), FlagCode::Italy);
        assert_eq!(detect_flag_code(Some("ko"), None), FlagCode::SouthKorea);
        assert_eq!(detect_flag_code(Some("zh"), None), FlagCode::China);
        assert_eq!(detect_flag_code(Some("ru"), None), FlagCode::Russia);
        assert_eq!(detect_flag_code(Some("xyz"), None), FlagCode::Generic);
    }
}
