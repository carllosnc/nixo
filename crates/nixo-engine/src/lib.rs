pub mod mpv;
pub mod synthetic;

use nixo_core::MediaBackend;
use tracing::{info, warn};

pub use mpv::MpvBackend;
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
}
