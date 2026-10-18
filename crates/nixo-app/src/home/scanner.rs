//! Varredura assíncrona de pastas e identificação de arquivos de mídia executáveis.
//!
//! A leitura do sistema de arquivos é executada numa thread desacoplada do render loop
//! para garantir 60/120 FPS cravados na interface sem qualquer travamento.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// Formatos de vídeo suportados pelo motor libmpv/nixo-engine.
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts", "mts", "m2ts", "vob", "ogv",
    "3gp", "rmvb",
];

/// Formatos de áudio suportados.
const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "m4a", "aac", "opus", "ogg", "wma", "alac", "aiff",
];

/// Categoria do item de mídia encontrado no disco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Audio,
    Directory,
}

/// Representação de um item listado na pasta de mídia.
#[derive(Debug, Clone)]
pub struct MediaItem {
    pub name: String,
    pub path: PathBuf,
    pub kind: MediaKind,
    pub size_formatted: String,
    pub extension: String,
}

/// Conteúdo consolidado de uma pasta (subpastas e arquivos executáveis).
#[derive(Debug, Clone, Default)]
pub struct FolderContent {
    pub _path: PathBuf,
    pub subdirs: Vec<MediaItem>,
    pub files: Vec<MediaItem>,
    pub is_loading: bool,
}

/// Mensagem enviada da thread de varredura para a thread principal da UI.
enum ScanResponse {
    Folder {
        path: PathBuf,
        subdirs: Vec<MediaItem>,
        files: Vec<MediaItem>,
    },
    Count {
        path: PathBuf,
        count: usize,
    },
}

/// Solicitação enviada da UI para o worker em background.
enum ScanRequest {
    Folder(PathBuf),
    Count(PathBuf),
}

/// Gerenciador de varredura em background com cache em memória.
pub struct BackgroundScanner {
    tx: Sender<ScanRequest>,
    rx: Receiver<ScanResponse>,
    cache: HashMap<PathBuf, FolderContent>,
    folder_counts: HashMap<PathBuf, usize>,
    pending: HashSet<PathBuf>,
}

impl Default for BackgroundScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundScanner {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<ScanRequest>();
        let (res_tx, res_rx) = channel::<ScanResponse>();

        thread::Builder::new()
            .name("nixo-media-scanner".into())
            .spawn(move || {
                while let Ok(req) = req_rx.recv() {
                    match req {
                        ScanRequest::Folder(path) => {
                            let (subdirs, files) = Self::scan_directory(&path);
                            let _ = res_tx.send(ScanResponse::Folder {
                                path,
                                subdirs,
                                files,
                            });
                        }
                        ScanRequest::Count(path) => {
                            let count = Self::count_playable_files(&path);
                            let _ = res_tx.send(ScanResponse::Count { path, count });
                        }
                    }
                }
            })
            .expect("falha ao iniciar thread nixo-media-scanner");

        Self {
            tx: req_tx,
            rx: res_rx,
            cache: HashMap::new(),
            folder_counts: HashMap::new(),
            pending: HashSet::new(),
        }
    }

    /// Processa respostas assíncronas do worker sem bloquear a UI.
    pub fn poll(&mut self) {
        while let Ok(res) = self.rx.try_recv() {
            match res {
                ScanResponse::Folder {
                    path,
                    subdirs,
                    files,
                } => {
                    self.pending.remove(&path);
                    self.folder_counts.insert(path.clone(), files.len());
                    self.cache.insert(
                        path.clone(),
                        FolderContent {
                            _path: path,
                            subdirs,
                            files,
                            is_loading: false,
                        },
                    );
                }
                ScanResponse::Count { path, count } => {
                    self.pending.remove(&path);
                    self.folder_counts.insert(path, count);
                }
            }
        }
    }

    /// Obtém o conteúdo de uma pasta ou dispara sua varredura se ainda não estiver em cache.
    pub fn get_or_request(&mut self, path: &Path) -> FolderContent {
        if let Some(content) = self.cache.get(path) {
            return content.clone();
        }

        if !self.pending.contains(path) {
            self.pending.insert(path.to_path_buf());
            let _ = self.tx.send(ScanRequest::Folder(path.to_path_buf()));
        }

        FolderContent {
            _path: path.to_path_buf(),
            subdirs: Vec::new(),
            files: Vec::new(),
            is_loading: true,
        }
    }

    /// Força uma nova varredura de um diretório (ex: após adicionar novo arquivo).
    pub fn invalidate(&mut self, path: &Path) {
        self.cache.remove(path);
        self.pending.remove(path);
        let _ = self.tx.send(ScanRequest::Folder(path.to_path_buf()));
    }

    /// Retorna a quantidade de itens executáveis numa pasta raiz salva.
    pub fn get_folder_count(&mut self, path: &Path) -> Option<usize> {
        if let Some(&count) = self.folder_counts.get(path) {
            Some(count)
        } else {
            if !self.pending.contains(path) {
                self.pending.insert(path.to_path_buf());
                let _ = self.tx.send(ScanRequest::Count(path.to_path_buf()));
            }
            None
        }
    }

    /// Realiza a leitura síncrona do diretório no worker em background.
    fn scan_directory(path: &Path) -> (Vec<MediaItem>, Vec<MediaItem>) {
        let mut subdirs = Vec::new();
        let mut files = Vec::new();

        let Ok(entries) = std::fs::read_dir(path) else {
            return (subdirs, files);
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            // Ignora arquivos/pastas ocultas do sistema operacional
            if file_name.starts_with('.') || file_name.starts_with('$') {
                continue;
            }

            let entry_path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };

            if meta.is_dir() {
                subdirs.push(MediaItem {
                    name: file_name,
                    path: entry_path,
                    kind: MediaKind::Directory,
                    size_formatted: String::new(),
                    extension: String::new(),
                });
            } else if meta.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();

                let is_video = VIDEO_EXTENSIONS.contains(&ext.as_str());
                let is_audio = !is_video && AUDIO_EXTENSIONS.contains(&ext.as_str());

                if is_video || is_audio {
                    let kind = if is_video {
                        MediaKind::Video
                    } else {
                        MediaKind::Audio
                    };
                    files.push(MediaItem {
                        name: file_name,
                        path: entry_path,
                        kind,
                        size_formatted: format_bytes(meta.len()),
                        extension: ext.to_uppercase(),
                    });
                }
            }
        }

        subdirs.sort_by_key(|a| a.name.to_lowercase());
        files.sort_by_key(|a| a.name.to_lowercase());

        (subdirs, files)
    }

    /// Contagem rápida de itens na pasta para exibição nos badges da tela inicial.
    fn count_playable_files(path: &Path) -> usize {
        let Ok(entries) = std::fs::read_dir(path) else {
            return 0;
        };

        let mut count = 0;
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with('.') || file_name.starts_with('$') {
                continue;
            }
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();

                if VIDEO_EXTENSIONS.contains(&ext.as_str())
                    || AUDIO_EXTENSIONS.contains(&ext.as_str())
                {
                    count += 1;
                }
            }
        }
        count
    }
}

/// Formata quantidade de bytes em unidade legível (B, KB, MB, GB).
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
