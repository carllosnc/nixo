//! Gerenciador de miniaturas da biblioteca de mídia no app.
//!
//! Recebe os quadros processados em background pelo `ThumbnailService` e realiza o
//! upload preguiçoso para a GPU como texturas `egui::TextureHandle`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use nixo_engine::ThumbnailService;

/// Gerenciador de texturas de miniaturas para os cards de mídia.
pub struct ThumbnailManager {
    service: Option<ThumbnailService>,
    textures: HashMap<PathBuf, TextureHandle>,
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
            requested: HashSet::new(),
        }
    }

    /// Retorna a textura GPU da miniatura caso já tenha sido processada e carregada.
    pub fn get(&self, path: &Path) -> Option<&TextureHandle> {
        self.textures.get(path)
    }

    /// Enfileira um arquivo de vídeo para extração assíncrona de miniatura.
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
        while let Some((path, frame)) = service.try_recv() {
            let size = [frame.width as usize, frame.height as usize];
            let color_image = ColorImage::from_rgba_unmultiplied(size, &frame.rgba);
            let tex_name = format!("thumb_{}", self.textures.len());
            let texture = ctx.load_texture(tex_name, color_image, TextureOptions::LINEAR);
            self.textures.insert(path, texture);
            loaded_any = true;
        }

        if loaded_any {
            ctx.request_repaint();
        }
    }
}
