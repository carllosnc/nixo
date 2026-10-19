//! Gerenciador de miniaturas da biblioteca de mídia no app.
//!
//! Recebe os quadros processados em background pelo `ThumbnailService` e realiza o
//! upload preguiçoso para a GPU como texturas `egui::TextureHandle`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use nixo_engine::ThumbnailService;

/// Metadados de reprodução (duração e resolução) detectados para o arquivo.
#[derive(Debug, Clone, Default)]
pub struct MediaDetails {
    pub duration: Option<Duration>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl MediaDetails {
    pub fn duration_formatted(&self) -> Option<String> {
        self.duration.map(nixo_core::format_duration)
    }

    pub fn resolution_formatted(&self) -> Option<String> {
        let w = self.width.unwrap_or(0);
        let h = self.height.unwrap_or(0);
        if w == 0 && h == 0 {
            return None;
        }
        let label = match h {
            2160.. => "4K",
            1440..=2159 => "1440p",
            1080..=1439 => "1080p",
            720..=1079 => "720p",
            480..=719 => "480p",
            360..=479 => "360p",
            _ if w >= 3840 => "4K",
            _ if w >= 2560 => "1440p",
            _ if w >= 1920 => "1080p",
            _ if w >= 1280 => "720p",
            _ => return Some(format!("{w}×{h}")),
        };
        Some(label.to_string())
    }
}

/// Identifica rapidamente a resolução a partir do padrão de nomenclatura do arquivo.
pub fn parse_resolution_from_name(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("2160p") || lower.contains("4k") || lower.contains("uhd") {
        Some("4K")
    } else if lower.contains("1440p") || lower.contains("2k") {
        Some("1440p")
    } else if lower.contains("1080p") || lower.contains("fhd") {
        Some("1080p")
    } else if lower.contains("720p") || (lower.contains("hd") && !lower.contains("hdtv")) {
        Some("720p")
    } else if lower.contains("480p") || lower.contains("sd") {
        Some("480p")
    } else {
        None
    }
}

/// Gerenciador de texturas de miniaturas para os cards de mídia.
pub struct ThumbnailManager {
    service: Option<ThumbnailService>,
    textures: HashMap<PathBuf, Vec<TextureHandle>>,
    details: HashMap<PathBuf, MediaDetails>,
    requested: HashSet<PathBuf>,
}

impl Default for ThumbnailManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThumbnailManager {
    pub fn new() -> Self {
        Self {
            service: ThumbnailService::new(),
            textures: HashMap::new(),
            details: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    /// Retorna a textura GPU da miniatura estática padrão (primeiro frame).
    #[allow(dead_code)]
    pub fn get(&self, path: &Path) -> Option<&TextureHandle> {
        self.get_frame(path, 0)
    }

    /// Retorna a textura GPU de uma cena específica no índice especificado.
    pub fn get_frame(&self, path: &Path, index: usize) -> Option<&TextureHandle> {
        let list = self.textures.get(path)?;
        if list.is_empty() {
            return None;
        }
        let idx = index.min(list.len() - 1);
        Some(&list[idx])
    }

    /// Retorna a contagem de frames de cenas disponíveis para o arquivo.
    pub fn frame_count(&self, path: &Path) -> usize {
        self.textures.get(path).map_or(0, |l| l.len())
    }

    /// Retorna os metadados (duração e resolução) detectados para o arquivo.
    pub fn get_details(&self, path: &Path) -> Option<&MediaDetails> {
        self.details.get(path)
    }

    /// Enfileira um arquivo de mídia para extração assíncrona de miniatura e metadados.
    pub fn request(&mut self, path: &Path) {
        if self.requested.insert(path.to_path_buf())
            && let Some(service) = &self.service
        {
            service.request(path.to_path_buf());
        }
    }

    /// Efetua a leitura de novos quadros prontos vindos do worker e faz o upload para GPU.
    pub fn poll(&mut self, ctx: &Context) {
        let Some(service) = &self.service else {
            return;
        };

        let mut loaded_any = false;
        while let Some(res) = service.try_recv() {
            if !res.frames.is_empty() {
                let mut handles = Vec::with_capacity(res.frames.len());
                for (i, frame) in res.frames.into_iter().enumerate() {
                    let size = [frame.width as usize, frame.height as usize];
                    let color_image = ColorImage::from_rgba_unmultiplied(size, &frame.rgba);
                    let tex_name = format!("thumb_{}_{}", self.textures.len(), i);
                    let texture = ctx.load_texture(tex_name, color_image, TextureOptions::LINEAR);
                    handles.push(texture);
                }
                self.textures.insert(res.path.clone(), handles);
                loaded_any = true;
            }
            if res.duration_secs.is_some() || res.width.is_some() || res.height.is_some() {
                self.details.insert(
                    res.path,
                    MediaDetails {
                        duration: res.duration_secs.map(Duration::from_secs_f64),
                        width: res.width,
                        height: res.height,
                    },
                );
                loaded_any = true;
            }
        }

        if loaded_any {
            ctx.request_repaint();
        }
    }
}
