//! Tela inicial (Home) e biblioteca de pastas de mídia do Nixo.

mod browser_view;
mod folders_view;
mod header;
mod scanner;

use std::path::PathBuf;

use egui::{Margin, Rect, ScrollArea, Ui};

pub use self::header::HomeAction;
pub use self::scanner::BackgroundScanner;

/// Estado da tela Home: navegação atual, histórico de pastas e scanner assíncrono.
pub struct HomeState {
    pub current_path: Option<PathBuf>,
    pub breadcrumbs: Vec<PathBuf>,
    pub scanner: BackgroundScanner,
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
        }
    }

    /// Navega para dentro de uma pasta e atualiza a trilha de navegação (breadcrumbs).
    pub fn navigate_to(&mut self, target: PathBuf) {
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
    }

    /// Renderiza a tela Home completa dentro do espaço disponível.
    pub fn render(&mut self, ui: &mut Ui, folders: &[PathBuf], _rect: Rect) -> HomeAction {
        self.scanner.poll();

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
                // 1. Cabeçalho com breadcrumbs e botões rápidos
                let header_action =
                    header::render_header(ui, self.current_path.as_deref(), &self.breadcrumbs);
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
                            let browser_act = browser_view::render_folder_browser(ui, &content);
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
            });

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
