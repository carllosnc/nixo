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
