//! Tela inicial (Home) e biblioteca de pastas de mídia do Nixo.

mod browser_view;
mod design_system_view;
mod folders_view;
mod header;
mod scanner;
mod thumbnail_manager;

use std::path::PathBuf;

use egui::{Margin, Rect, ScrollArea, Ui};

pub use self::header::HomeAction;
pub use self::scanner::BackgroundScanner;
pub use self::thumbnail_manager::ThumbnailManager;

/// Estado da tela Home: navegação atual, histórico de pastas e scanner assíncrono.
pub struct HomeState {
    pub current_path: Option<PathBuf>,
    pub breadcrumbs: Vec<PathBuf>,
    pub scanner: BackgroundScanner,
    pub is_design_system: bool,
    pub logo_texture: Option<egui::TextureHandle>,
    pub thumbnails: ThumbnailManager,
}

impl Default for HomeState {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeState {
    pub fn new() -> Self {
        Self {
            current_path: None,
            breadcrumbs: Vec::new(),
            scanner: BackgroundScanner::new(),
            is_design_system: false,
            logo_texture: None,
            thumbnails: ThumbnailManager::new(),
        }
    }

    /// Navega para dentro de uma pasta e atualiza a trilha de navegação (breadcrumbs).
    pub fn navigate_to(&mut self, target: PathBuf) {
        self.is_design_system = false;
        if self.current_path.as_deref() == Some(&target) {
            return;
        }

        // Se a pasta for subdiretório do atual, adiciona ao histórico; caso contrário, reseta a raiz
        if let Some(curr) = &self.current_path {
            if target.starts_with(curr) && target != *curr {
                self.breadcrumbs.push(target.clone());
            } else {
                self.breadcrumbs = vec![target.clone()];
            }
        } else {
            self.breadcrumbs = vec![target.clone()];
        }

        self.current_path = Some(target);
    }

    /// Retorna um nível na árvore de diretórios ou à raiz das pastas salvas.
    pub fn go_back(&mut self) {
        self.is_design_system = false;
        if self.breadcrumbs.len() > 1 {
            self.breadcrumbs.pop();
            self.current_path = self.breadcrumbs.last().cloned();
        } else {
            self.go_root();
        }
    }

    /// Retorna para a visualização principal com a lista de todas as pastas salvas.
    pub fn go_root(&mut self) {
        self.breadcrumbs.clear();
        self.current_path = None;
        self.is_design_system = false;
    }

    /// Renderiza a tela Home completa dentro do espaço disponível.
    pub fn render(&mut self, ui: &mut Ui, folders: &[PathBuf], _rect: Rect) -> HomeAction {
        self.scanner.poll();
        self.thumbnails.poll(ui.ctx());

        // Garante contagem de itens para as pastas salvas exibidas na raiz
        for folder in folders {
            self.scanner.get_folder_count(folder);
        }

        let mut action = HomeAction::None;

        egui::Frame::NONE
            .inner_margin(Margin {
                left: 28,
                right: 28,
                top: 20,
                bottom: 24,
            })
            .show(ui, |ui| {
                if self.is_design_system {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let ds_act = design_system_view::render_design_system_view(ui);
                            if !matches!(ds_act, HomeAction::None) {
                                action = ds_act;
                            }
                        });
                } else {
                    // Garante que a textura da logo esteja carregada na GPU
                    if self.logo_texture.is_none()
                        && let Some(color_img) = header::load_logo_image()
                    {
                        self.logo_texture = Some(ui.ctx().load_texture(
                            "nixo_logo",
                            color_img,
                            egui::TextureOptions::LINEAR,
                        ));
                    }

                    // 1. Cabeçalho com breadcrumbs, logo e botões rápidos
                    let header_action = header::render_header(
                        ui,
                        self.current_path.as_deref(),
                        &self.breadcrumbs,
                        self.logo_texture.as_ref(),
                    );
                    if !matches!(header_action, HomeAction::None) {
                        action = header_action;
                    }

                    ui.add_space(16.0);

                    // 2. Área de Conteúdo rolável
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if let Some(current) = &self.current_path {
                                let content = self.scanner.get_or_request(current);
                                let browser_act = browser_view::render_folder_browser(
                                    ui,
                                    &content,
                                    &mut self.thumbnails,
                                );
                                if !matches!(browser_act, HomeAction::None) {
                                    action = browser_act;
                                }
                            } else {
                                // Visão das pastas de mídia configuradas
                                let folders_act = folders_view::render_folders_view(
                                    ui,
                                    folders,
                                    &self.get_counts_map(folders),
                                );
                                if !matches!(folders_act, HomeAction::None) {
                                    action = folders_act;
                                }
                            }
                        });
                }
            });

        match &action {
            HomeAction::OpenDesignSystem => self.is_design_system = true,
            HomeAction::CloseDesignSystem => self.is_design_system = false,
            _ => {}
        }

        action
    }

    fn get_counts_map(&mut self, folders: &[PathBuf]) -> std::collections::HashMap<PathBuf, usize> {
        let mut map = std::collections::HashMap::new();
        for f in folders {
            if let Some(count) = self.scanner.get_folder_count(f) {
                map.insert(f.clone(), count);
            }
        }
        map
    }
}
