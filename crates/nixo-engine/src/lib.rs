pub mod mpv;
pub mod synthetic;

use nixo_core::MediaBackend;
use tracing::{info, warn};

pub use mpv::MpvBackend;
pub use mpv::thumbnail::{ThumbnailResult, ThumbnailService};
pub use synthetic::SyntheticBackend;

/// Cria o motor de reprodução padrão do Nixo.
/// Tenta carregar o `libmpv` nativo do sistema operacional; caso a DLL/SO/DYLIB
/// ainda não esteja presente, ativa automaticamente o motor sintético de 60 FPS
/// para que a interface e o pipeline de texturas GPU funcionem imediatamente.
pub fn create_default_backend() -> Box<dyn MediaBackend> {
    match MpvBackend::try_new() {
        Ok(mpv) => {
            info!("Motor de vídeo inicializado com libmpv nativo.");
            Box::new(mpv)
        }
        Err(err) => {
            warn!(
                "libmpv não detectado ({err}). Usando motor de preview GPU 60 FPS. \
                 Para reproduzir vídeos reais no Windows, coloque 'libmpv-2.dll' no PATH ou na pasta do executável."
            );
            Box::new(SyntheticBackend::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nixo_core::{PlaybackState, PlayerCommand};

    #[test]
    fn test_synthetic_backend_lifecycle() {
        let mut backend = SyntheticBackend::new();
        assert_eq!(backend.status().state, PlaybackState::Stopped);

        backend.execute(PlayerCommand::Play).unwrap();
        assert_eq!(backend.status().state, PlaybackState::Playing);

        let frame = backend
            .poll_video_frame((640, 360))
            .expect("Deveria gerar VideoFrame sintético");
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 360);
    }

    #[test]
    fn test_synthetic_audio_and_subtitles() {
        let mut backend = SyntheticBackend::new();
        let status = backend.status();
        assert_eq!(status.audio_tracks.len(), 2);
        assert_eq!(status.subtitle_tracks.len(), 2);

        // Faixa de áudio: alternar para id 2
        backend
            .execute(PlayerCommand::SetAudioTrack(Some(2)))
            .unwrap();
        assert!(backend.status().audio_tracks[1].selected);
        assert!(!backend.status().audio_tracks[0].selected);

        // Desativar áudio
        backend.execute(PlayerCommand::SetAudioTrack(None)).unwrap();
        assert!(!backend.status().audio_tracks.iter().any(|t| t.selected));

        // Legendas: selecionar id 1
        backend
            .execute(PlayerCommand::SetSubtitleTrack(Some(1)))
            .unwrap();
        assert!(backend.status().subtitle_tracks[0].selected);
        assert!(backend.status().subtitles_visible);

        // Visibilidade de legenda
        backend
            .execute(PlayerCommand::ToggleSubtitleVisibility)
            .unwrap();
        assert!(!backend.status().subtitles_visible);

        // Carregar legenda externa
        backend
            .execute(PlayerCommand::LoadSubtitleFile(std::path::PathBuf::from(
                "pt_br_externa.srt",
            )))
            .unwrap();
        let new_status = backend.status();
        assert_eq!(new_status.subtitle_tracks.len(), 3);
        assert!(new_status.subtitle_tracks[2].selected);
        assert_eq!(
            new_status.subtitle_tracks[2].title.as_deref(),
            Some("pt_br_externa.srt")
        );
        assert!(new_status.subtitles_visible);
    }
}
