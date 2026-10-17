//! Persistência das preferências do usuário em `settings.toml`.
//!
//! Local do arquivo (via [`dirs::config_dir`]):
//! - Windows: `%APPDATA%\nixo\settings.toml`
//! - Linux:   `$XDG_CONFIG_HOME/nixo/settings.toml` (ou `~/.config/nixo/settings.toml`)
//! - macOS:   `~/Library/Application Support/nixo/settings.toml`
//!
//! A escrita ocorre numa thread dedicada (nunca bloqueia o render loop), com *debounce*
//! para evitar gravações a cada frame enquanto o usuário arrasta o slider de volume,
//! e é atômica (arquivo temporário + `rename`) para não corromper o arquivo em caso de crash.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use nixo_core::SubtitleStyle;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

const APP_DIR_NAME: &str = "nixo";
const SETTINGS_FILE_NAME: &str = "settings.toml";
/// Tempo sem novas alterações antes de gravar em disco.
const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);

/// Raiz das preferências persistidas. Novas seções devem usar `#[serde(default)]`
/// para que arquivos antigos continuem válidos (compatibilidade futura).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppSettings {
    pub playback: PlaybackSettings,
    pub subtitles: SubtitleSettings,
    pub window: WindowSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackSettings {
    /// Volume entre 0.0 e 100.0
    pub volume: f32,
    pub muted: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            volume: 80.0,
            muted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubtitleSettings {
    pub visible: bool,
    pub style: SubtitleStyle,
}

impl Default for SubtitleSettings {
    fn default() -> Self {
        Self {
            visible: true,
            style: SubtitleStyle::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WindowSettings {
    pub always_on_top: bool,
}

impl AppSettings {
    /// Corrige valores inválidos vindos de um arquivo editado manualmente.
    pub fn sanitized(mut self) -> Self {
        let vol = self.playback.volume;
        self.playback.volume = if vol.is_finite() {
            vol.clamp(0.0, 100.0)
        } else {
            PlaybackSettings::default().volume
        };
        self.subtitles.style = self.subtitles.style.sanitized();
        self
    }

    /// Caminho padrão do arquivo de configurações para a plataforma atual.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join(APP_DIR_NAME).join(SETTINGS_FILE_NAME))
    }

    /// Carrega as configurações de `path`. Nunca falha: arquivos ausentes ou inválidos
    /// resultam nos valores padrão (arquivos corrompidos são preservados como `.bak`).
    pub fn load_from(path: &Path) -> Self {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                info!("Nenhum arquivo de configurações em {}", path.display());
                return Self::default();
            }
            Err(err) => {
                warn!("Falha ao ler {}: {err}", path.display());
                return Self::default();
            }
        };

        match toml::from_str::<Self>(&text) {
            Ok(settings) => {
                info!("Configurações carregadas de {}", path.display());
                settings.sanitized()
            }
            Err(err) => {
                warn!("Configurações inválidas em {}: {err}", path.display());
                let backup = path.with_extension("toml.bak");
                if let Err(err) = std::fs::rename(path, &backup) {
                    warn!("Falha ao criar backup {}: {err}", backup.display());
                }
                Self::default()
            }
        }
    }

    /// Grava de forma atômica: escreve em um arquivo temporário e renomeia sobre o destino.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path)
    }
}

/// Observa mudanças nas preferências e as grava em background com debounce.
pub struct SettingsStore {
    last_saved: AppSettings,
    pending: Option<(AppSettings, Instant)>,
    tx: Option<Sender<AppSettings>>,
    worker: Option<JoinHandle<()>>,
}

impl SettingsStore {
    /// Carrega as configurações do caminho padrão e inicia a thread de escrita.
    pub fn load_default() -> (Self, AppSettings) {
        let path = AppSettings::default_path();
        let settings = match &path {
            Some(p) => AppSettings::load_from(p),
            None => {
                warn!("Diretório de configurações indisponível; preferências não serão salvas");
                AppSettings::default()
            }
        };
        (Self::new(path, settings.clone()), settings)
    }

    fn new(path: Option<PathBuf>, initial: AppSettings) -> Self {
        let (tx, worker) = match path {
            Some(path) => {
                let (tx, rx) = mpsc::channel::<AppSettings>();
                let spawn = std::thread::Builder::new()
                    .name("nixo-settings-writer".into())
                    .spawn(move || {
                        while let Ok(mut settings) = rx.recv() {
                            // Coalesce: grava apenas o estado mais recente da fila.
                            while let Ok(newer) = rx.try_recv() {
                                settings = newer;
                            }
                            match settings.save_to(&path) {
                                Ok(()) => debug!("Configurações salvas em {}", path.display()),
                                Err(err) => {
                                    error!("Falha ao salvar {}: {err}", path.display());
                                }
                            }
                        }
                    });
                match spawn {
                    Ok(handle) => (Some(tx), Some(handle)),
                    Err(err) => {
                        error!("Falha ao iniciar thread de configurações: {err}");
                        (None, None)
                    }
                }
            }
            None => (None, None),
        };

        Self {
            last_saved: initial,
            pending: None,
            tx,
            worker,
        }
    }

    /// Deve ser chamado a cada frame com o estado atual. Barato quando nada mudou.
    pub fn observe(&mut self, current: AppSettings) {
        let baseline = self
            .pending
            .as_ref()
            .map_or(&self.last_saved, |(pending, _)| pending);
        if *baseline != current {
            self.pending = Some((current, Instant::now()));
        }

        if let Some((_, changed_at)) = &self.pending
            && changed_at.elapsed() >= SAVE_DEBOUNCE
        {
            self.commit();
        }
    }

    /// Tempo restante até a próxima gravação pendente (para agendar um repaint).
    pub fn time_until_save(&self) -> Option<Duration> {
        self.pending
            .as_ref()
            .map(|(_, at)| SAVE_DEBOUNCE.saturating_sub(at.elapsed()))
    }

    fn commit(&mut self) {
        let Some((settings, _)) = self.pending.take() else {
            return;
        };
        if settings == self.last_saved {
            return;
        }
        if let Some(tx) = &self.tx
            && tx.send(settings.clone()).is_err()
        {
            error!("Thread de configurações encerrada; alteração descartada");
        }
        self.last_saved = settings;
    }

    /// Grava alterações pendentes e aguarda a thread de escrita terminar (usado ao sair).
    pub fn flush(&mut self) {
        self.commit();
        self.tx = None; // fecha o canal → a thread termina após esvaziar a fila
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            error!("Thread de configurações terminou com pânico");
        }
    }
}

impl Drop for SettingsStore {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("nixo-settings-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join(SETTINGS_FILE_NAME)
    }

    #[test]
    fn roundtrip_save_and_load() {
        let path = temp_path("roundtrip");
        let settings = AppSettings {
            playback: PlaybackSettings {
                volume: 42.5,
                muted: true,
            },
            subtitles: SubtitleSettings {
                visible: false,
                style: SubtitleStyle::preset_cinema(),
            },
            window: WindowSettings {
                always_on_top: true,
            },
        };
        settings.save_to(&path).expect("save");
        assert_eq!(AppSettings::load_from(&path), settings);
    }

    #[test]
    fn missing_file_yields_defaults() {
        let path = temp_path("missing");
        assert_eq!(AppSettings::load_from(&path), AppSettings::default());
    }

    #[test]
    fn partial_file_fills_defaults() {
        let parsed: AppSettings = toml::from_str("[playback]\nvolume = 10.0\n").expect("parse");
        assert_eq!(parsed.playback.volume, 10.0);
        assert!(!parsed.playback.muted);
        assert!(parsed.subtitles.visible);
    }

    #[test]
    fn invalid_file_is_backed_up() {
        let path = temp_path("invalid");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "isto não é toml = = =").expect("write");
        assert_eq!(AppSettings::load_from(&path), AppSettings::default());
        assert!(path.with_extension("toml.bak").exists());
    }

    #[test]
    fn sanitize_clamps_volume() {
        let mut s = AppSettings::default();
        s.playback.volume = 250.0;
        assert_eq!(s.sanitized().playback.volume, 100.0);
        let mut s = AppSettings::default();
        s.playback.volume = f32::NAN;
        assert_eq!(s.sanitized().playback.volume, 80.0);
    }

    #[test]
    fn store_flush_writes_pending_changes() {
        let path = temp_path("store");
        let mut store = SettingsStore::new(Some(path.clone()), AppSettings::default());
        let mut changed = AppSettings::default();
        changed.playback.volume = 33.0;
        store.observe(changed.clone());
        assert!(store.time_until_save().is_some());
        store.flush();
        assert_eq!(AppSettings::load_from(&path), changed);
    }
}
