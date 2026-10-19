//! Tela inicial (Home) e biblioteca de pastas de mídia do Nixo.

mod browser_view;
mod design_system_view;
mod folders_view;
mod header;
mod scanner;
mod thumbnail_manager;

use std::path::PathBuf;
use std::time::Instant;

use egui::{Margin, Rect, ScrollArea, Ui};

pub use self::header::HomeAction;
pub use self::scanner::BackgroundScanner;
pub use self::thumbnail_manager::ThumbnailManager;

/// Direção da transição horizontal ao trocar de pasta na biblioteca.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NavDirection {
    Forward,  // Entrando em uma subpasta (conteúdo desliza da direita para esquerda)
    Backward, // Voltando para a pasta pai ou raiz (conteúdo desliza da esquerda para direita)
}

/// Controlador da animação lateral suave com curva cúbica ease-out.
#[derive(Clone, Copy, Debug)]
pub struct NavTransition {
    pub start: Instant,
    pub direction: NavDirection,
    pub duration_secs: f32,
}

impl NavTransition {
    pub fn forward() -> Self {
        Self {
            start: Instant::now(),
            direction: NavDirection::Forward,
            duration_secs: 0.20,
        }
    }

    pub fn backward() -> Self {
        Self {
            start: Instant::now(),
            direction: NavDirection::Backward,
            duration_secs: 0.20,
        }
    }

    /// Calcula o deslocamento horizontal `offset_x`, opacidade e se a transição encerrou.
    pub fn calculate(&self, ctx: &egui::Context) -> (f32, f32, bool) {
        let elapsed = self.start.elapsed().as_secs_f32();
        if elapsed >= self.duration_secs {
            return (0.0, 1.0, true);
        }

        ctx.request_repaint();

        let progress = (elapsed / self.duration_secs).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - progress).powi(3);

        let max_distance = 36.0;
        let sign = match self.direction {
            NavDirection::Forward => 1.0,
            NavDirection::Backward => -1.0,
        };

        let offset_x = (1.0 - ease) * max_distance * sign;
        let opacity = 0.35 + 0.65 * ease;

        (offset_x, opacity, false)
    }
}

/// Estado da tela Home: navegação atual, histórico de pastas e scanner assíncrono.
pub struct HomeState {
    pub current_path: Option<PathBuf>,
    pub breadcrumbs: Vec<PathBuf>,
    pub scanner: BackgroundScanner,
    pub is_design_system: bool,
    pub logo_texture: Option<egui::TextureHandle>,
    pub folder_texture: Option<egui::TextureHandle>,
    pub video_clapper_texture: Option<egui::TextureHandle>,
    pub thumbnails: ThumbnailManager,
    pub selected_index: usize,
    pub transition: Option<NavTransition>,
    pub keyboard_nav_active: bool,
    pub pending_selection_path: Option<PathBuf>,
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
            folder_texture: None,
            video_clapper_texture: None,
            thumbnails: ThumbnailManager::new(),
            selected_index: 0,
            transition: None,
            keyboard_nav_active: false,
            pending_selection_path: None,
        }
    }

    /// Navega para dentro de uma pasta e atualiza a trilha de navegação (breadcrumbs).
    pub fn navigate_to(&mut self, target: PathBuf) {
        self.is_design_system = false;
        if self.current_path.as_deref() == Some(&target) {
            return;
        }

        if let Some(pos) = self.breadcrumbs.iter().position(|b| b == &target) {
            if pos + 1 < self.breadcrumbs.len() {
                self.pending_selection_path = Some(self.breadcrumbs[pos + 1].clone());
            } else if let Some(curr) = &self.current_path {
                self.pending_selection_path = Some(curr.clone());
            }
            self.breadcrumbs.truncate(pos + 1);
            self.transition = Some(NavTransition::backward());
            self.keyboard_nav_active = true;
        } else if let Some(curr) = &self.current_path {
            if target.starts_with(curr) && target != *curr {
                self.breadcrumbs.push(target.clone());
            } else {
                self.breadcrumbs = vec![target.clone()];
            }
            self.selected_index = 0;
            self.transition = Some(NavTransition::forward());
        } else {
            self.breadcrumbs = vec![target.clone()];
            self.selected_index = 0;
            self.transition = Some(NavTransition::forward());
        }

        self.current_path = Some(target);
    }

    /// Retorna um nível na árvore de diretórios ou à raiz das pastas salvas,
    /// mantendo selecionada a pasta que acabou de ser fechada.
    pub fn go_back(&mut self) {
        self.is_design_system = false;
        self.transition = Some(NavTransition::backward());
        self.keyboard_nav_active = true;
        self.selected_index = 0;

        let exited = self.current_path.take();
        self.pending_selection_path = exited;

        if self.breadcrumbs.len() > 1 {
            self.breadcrumbs.pop();
            self.current_path = self.breadcrumbs.last().cloned();
        } else {
            self.breadcrumbs.clear();
            self.current_path = None;
        }
    }

    /// Retorna para a visualização principal com a lista de todas as pastas salvas.
    pub fn go_root(&mut self) {
        if let Some(first) = self.breadcrumbs.first() {
            self.pending_selection_path = Some(first.clone());
        } else if let Some(curr) = self.current_path.take() {
            self.pending_selection_path = Some(curr);
        }
        self.breadcrumbs.clear();
        self.current_path = None;
        self.is_design_system = false;
        self.selected_index = 0;
        self.transition = Some(NavTransition::backward());
        self.keyboard_nav_active = true;
    }

    /// Seleciona o item anterior na lista de navegação via teclado.
    pub fn select_prev(&mut self) {
        self.keyboard_nav_active = true;
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Seleciona o próximo item na lista de navegação via teclado.
    pub fn select_next(&mut self, total_items: usize) {
        self.keyboard_nav_active = true;
        if total_items > 0 && self.selected_index + 1 < total_items {
            self.selected_index += 1;
        }
    }

    /// Calcula o número total de itens navegáveis na visualização atual.
    pub fn total_items(&mut self, folders: &[PathBuf]) -> usize {
        if let Some(current) = &self.current_path {
            let content = self.scanner.get_or_request(current);
            content.subdirs.len() + content.files.len()
        } else {
            folders.len()
        }
    }

    /// Retorna a ação correspondente ao item selecionado via teclado (Enter para abrir/reproduzir, Seta Direita apenas para entrar em pasta).
    pub fn activate_selected(&mut self, folders: &[PathBuf]) -> HomeAction {
        if let Some(current) = &self.current_path {
            let content = self.scanner.get_or_request(current);
            let subdirs_len = content.subdirs.len();
            if self.selected_index < subdirs_len {
                if let Some(dir) = content.subdirs.get(self.selected_index) {
                    return HomeAction::OpenFolder(dir.path.clone());
                }
            } else {
                let file_idx = self.selected_index.saturating_sub(subdirs_len);
                if let Some(file) = content.files.get(file_idx) {
                    return HomeAction::PlayFile(file.path.clone());
                }
            }
        } else if let Some(folder) = folders.get(self.selected_index) {
            return HomeAction::OpenFolder(folder.clone());
        }
        HomeAction::None
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

        if self.is_design_system {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(Margin {
                            left: 28,
                            right: 28,
                            top: 20,
                            bottom: 24,
                        })
                        .show(ui, |ui| {
                            let ds_act = design_system_view::render_design_system_view(ui);
                            if !matches!(ds_act, HomeAction::None) {
                                action = ds_act;
                            }
                        });
                });
        } else {
            // Garante que a textura da logo e o ícone 3D de pasta estejam carregados na GPU
            if self.logo_texture.is_none()
                && let Some(color_img) = header::load_logo_image()
            {
                self.logo_texture = Some(ui.ctx().load_texture(
                    "nixo_logo",
                    color_img,
                    egui::TextureOptions::LINEAR,
                ));
            }

            if self.folder_texture.is_none()
                && let Some(color_img) = header::load_folder_3d_image()
            {
                self.folder_texture = Some(ui.ctx().load_texture(
                    "nixo_folder_3d",
                    color_img,
                    egui::TextureOptions::LINEAR,
                ));
            }

            if self.video_clapper_texture.is_none()
                && let Some(color_img) = header::load_video_clapper_image()
            {
                self.video_clapper_texture = Some(ui.ctx().load_texture(
                    "nixo_video_clapper",
                    color_img,
                    egui::TextureOptions::LINEAR,
                ));
            }

            // 1. Cabeçalho fixo no topo com padding interno
            egui::Frame::NONE
                .inner_margin(Margin {
                    left: 28,
                    right: 28,
                    top: 20,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    let header_action = header::render_header(
                        ui,
                        self.current_path.as_deref(),
                        &self.breadcrumbs,
                        self.logo_texture.as_ref(),
                    );
                    if !matches!(header_action, HomeAction::None) {
                        action = header_action;
                    }
                });

            // 2. Área de Conteúdo rolável com scrollbar colada na borda direita e paddings internos
            let (offset_x, opacity, is_finished) = self
                .transition
                .map(|t| t.calculate(ui.ctx()))
                .unwrap_or((0.0, 1.0, true));

            if is_finished && self.transition.is_some() {
                self.transition = None;
            }

            let left_margin = (28.0 + offset_x).round().clamp(-64.0, 120.0) as i8;
            let right_margin = (28.0 - offset_x).round().clamp(-64.0, 120.0) as i8;

            let scroll_id = self
                .current_path
                .as_deref()
                .unwrap_or(std::path::Path::new("__nixo_home_root__"));

            let is_navigating_or_restoring =
                self.transition.is_some() || self.pending_selection_path.is_some();

            ScrollArea::vertical()
                .id_salt(scroll_id)
                .animated(!is_navigating_or_restoring)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_opacity(opacity);
                    egui::Frame::NONE
                        .inner_margin(Margin {
                            left: left_margin,
                            right: right_margin,
                            top: 14,
                            bottom: 24,
                        })
                        .show(ui, |ui| {
                            if let Some(current) = &self.current_path {
                                let content = self.scanner.get_or_request(current);

                                if let Some(ref target) = self.pending_selection_path {
                                    if let Some(idx) =
                                        content.subdirs.iter().position(|d| &d.path == target)
                                    {
                                        self.selected_index = idx;
                                        self.pending_selection_path = None;
                                    } else if let Some(idx) =
                                        content.files.iter().position(|f| &f.path == target)
                                    {
                                        self.selected_index = content.subdirs.len() + idx;
                                        self.pending_selection_path = None;
                                    }
                                }

                                let browser_act = browser_view::render_folder_browser(
                                    ui,
                                    &content,
                                    &mut self.thumbnails,
                                    self.folder_texture.as_ref(),
                                    self.video_clapper_texture.as_ref(),
                                    &mut self.selected_index,
                                    &mut self.keyboard_nav_active,
                                );
                                if !matches!(browser_act, HomeAction::None) {
                                    action = browser_act;
                                }
                            } else {
                                if let Some(target) = self.pending_selection_path.take()
                                    && let Some(idx) = folders.iter().position(|f| f == &target)
                                {
                                    self.selected_index = idx;
                                }

                                // Visão das pastas de mídia configuradas
                                let folders_act = folders_view::render_folders_view(
                                    ui,
                                    folders,
                                    &self.get_counts_map(folders),
                                    self.folder_texture.as_ref(),
                                    &mut self.selected_index,
                                    &mut self.keyboard_nav_active,
                                );
                                if !matches!(folders_act, HomeAction::None) {
                                    action = folders_act;
                                }
                            }
                        });
                });
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_back_preserves_exited_folder_in_pending_selection() {
        let mut state = HomeState::new();
        let folder_a = PathBuf::from("D:/Media/A");
        let folder_b = PathBuf::from("D:/Media/A/B");

        state.navigate_to(folder_a.clone());
        state.navigate_to(folder_b.clone());
        assert_eq!(state.current_path.as_ref(), Some(&folder_b));

        state.go_back();
        assert_eq!(state.current_path.as_ref(), Some(&folder_a));
        assert_eq!(state.pending_selection_path.as_ref(), Some(&folder_b));

        state.go_back();
        assert_eq!(state.current_path, None);
        assert_eq!(state.pending_selection_path.as_ref(), Some(&folder_a));
    }

    #[test]
    fn test_go_root_preserves_root_folder_in_pending_selection() {
        let mut state = HomeState::new();
        let folder_a = PathBuf::from("D:/Media/A");
        let folder_b = PathBuf::from("D:/Media/A/B");

        state.navigate_to(folder_a.clone());
        state.navigate_to(folder_b);
        state.go_root();

        assert_eq!(state.current_path, None);
        assert_eq!(state.pending_selection_path.as_ref(), Some(&folder_a));
    }

    #[test]
    fn test_navigate_to_ancestor_preserves_child_in_pending_selection() {
        let mut state = HomeState::new();
        let folder_a = PathBuf::from("D:/Media/A");
        let folder_b = PathBuf::from("D:/Media/A/B");
        let folder_c = PathBuf::from("D:/Media/A/B/C");

        state.navigate_to(folder_a.clone());
        state.navigate_to(folder_b.clone());
        state.navigate_to(folder_c);

        // Clica no breadcrumb de 'A': deve selecionar 'B'
        state.navigate_to(folder_a.clone());
        assert_eq!(state.current_path.as_ref(), Some(&folder_a));
        assert_eq!(state.pending_selection_path.as_ref(), Some(&folder_b));
    }
}
