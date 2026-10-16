use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

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
}
