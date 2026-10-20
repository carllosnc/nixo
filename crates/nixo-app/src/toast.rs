//! Sistema de notificações flutuantes (Toasts) do Nixo.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, Ui, vec2,
};

use crate::app::ACCENT_COLOR;
use crate::components::tokens::{DANGER_TEXT, NEUTRAL_700, NEUTRAL_800, TEXT_PRIMARY, lerp_color};

/// Ícone exibido à esquerda na notificação Toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastIcon {
    Camera,
    Trash,
    Play,
    #[allow(dead_code)]
    Folder,
    Lucide(&'static str),
}

impl From<&'static str> for ToastIcon {
    fn from(s: &'static str) -> Self {
        ToastIcon::Lucide(s)
    }
}

/// Estado de uma notificação temporária no topo da tela.
pub struct ToastNotification {
    pub icon: ToastIcon,
    pub message: String,
    pub path: Option<PathBuf>,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastNotification {
    pub fn new(icon: impl Into<ToastIcon>, message: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            message: message.into(),
            path: None,
            created_at: Instant::now(),
            duration: Duration::from_millis(2800),
        }
    }

    pub fn with_path(
        icon: impl Into<ToastIcon>,
        message: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        Self {
            icon: icon.into(),
            message: message.into(),
            path: Some(path),
            created_at: Instant::now(),
            duration: Duration::from_millis(2800),
        }
    }
}

/// Renderiza o toast flutuante no topo do reprodutor com animações de fade e slide.
pub(crate) fn render_toast(ui: &mut Ui, toast: &mut Option<ToastNotification>, screen_rect: Rect) {
    let Some(t) = toast.as_mut() else { return };

    let elapsed = t.created_at.elapsed();
    if elapsed >= t.duration {
        *toast = None;
        return;
    }

    // Curva de opacidade com fade-in (0.18s) e fade-out (0.3s)
    let total_secs = t.duration.as_secs_f32();
    let elapsed_secs = elapsed.as_secs_f32();
    let in_alpha = (elapsed_secs / 0.18).clamp(0.0, 1.0);
    let out_alpha = ((total_secs - elapsed_secs) / 0.30).clamp(0.0, 1.0);
    let alpha = in_alpha.min(out_alpha);

    // Efeito sutil de deslizamento vindo de cima (slide down)
    let slide_offset = egui::lerp(-12.0..=0.0, in_alpha);
    let center_x = screen_rect.center().x;
    let center_y = screen_rect.top() + 48.0 + slide_offset;

    // Mede a largura necessária para o texto com tamanho de fonte ampliado (14.5pt)
    let font_id = FontId::proportional(14.5);
    let galley = ui.fonts(|f| f.layout_no_wrap(t.message.clone(), font_id.clone(), Color32::WHITE));

    let pad_x = 20.0;
    let pad_y = 10.0;
    let icon_space = 26.0;
    let total_w = galley.size().x + pad_x * 2.0 + icon_space;
    let total_h = (galley.size().y + pad_y * 2.0).max(40.0);

    let toast_rect = Rect::from_center_size(Pos2::new(center_x, center_y), vec2(total_w, total_h));

    // Aloca a área interativa para permitir clique (abrir no explorador)
    let resp = ui.allocate_rect(toast_rect, Sense::click());
    let hovered = resp.hovered();

    // Se o mouse estiver sobre o toast, congela o timer para o usuário conseguir ler ou clicar
    if hovered {
        t.created_at = Instant::now() - Duration::from_millis(1000);
        resp.clone().on_hover_cursor(CursorIcon::PointingHand);
    }

    // Clique abre o arquivo capturado no visualizador/explorador do sistema
    if resp.clicked()
        && let Some(path) = &t.path
    {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(format!("/select,{}", path.display()))
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn();
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            if let Some(parent) = path.parent() {
                let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
            }
        }
    }

    // Fundo da cápsula com tonalidade da paleta neutra oficial (NEUTRAL_800 -> NEUTRAL_700 no hover), sem borda
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("toast_hover"), hovered, 0.15);
    let base_color = lerp_color(NEUTRAL_800, NEUTRAL_700, hover_t);
    let bg_alpha = (250.0 * alpha) as u8;
    ui.painter().rect_filled(
        toast_rect,
        CornerRadius::same(99),
        Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), bg_alpha),
    );

    // Ícone à esquerda configurável por componente
    let icon_c = Pos2::new(toast_rect.left() + pad_x + 6.0, toast_rect.center().y);
    match t.icon {
        ToastIcon::Camera => {
            let icon_color = Color32::from_rgba_unmultiplied(
                ACCENT_COLOR.r(),
                ACCENT_COLOR.g(),
                ACCENT_COLOR.b(),
                (255.0 * alpha) as u8,
            );
            paint_mini_camera(ui, icon_c, icon_color);
        }
        ToastIcon::Trash => {
            let trash_color = Color32::from_rgba_unmultiplied(
                DANGER_TEXT.r(),
                DANGER_TEXT.g(),
                DANGER_TEXT.b(),
                (255.0 * alpha) as u8,
            );
            ui.painter().text(
                icon_c,
                Align2::CENTER_CENTER,
                crate::lucide::TRASH_2,
                crate::lucide::font_id(16.0),
                trash_color,
            );
        }
        ToastIcon::Play => {
            let play_color = Color32::from_rgba_unmultiplied(
                ACCENT_COLOR.r(),
                ACCENT_COLOR.g(),
                ACCENT_COLOR.b(),
                (255.0 * alpha) as u8,
            );
            ui.painter().text(
                icon_c,
                Align2::CENTER_CENTER,
                crate::lucide::PLAY,
                crate::lucide::font_id(15.5),
                play_color,
            );
        }
        ToastIcon::Folder => {
            let folder_color = Color32::from_rgba_unmultiplied(
                ACCENT_COLOR.r(),
                ACCENT_COLOR.g(),
                ACCENT_COLOR.b(),
                (255.0 * alpha) as u8,
            );
            ui.painter().text(
                icon_c,
                Align2::CENTER_CENTER,
                crate::lucide::FOLDER,
                crate::lucide::font_id(16.0),
                folder_color,
            );
        }
        ToastIcon::Lucide(icon_str) => {
            let ic_color = Color32::from_rgba_unmultiplied(
                ACCENT_COLOR.r(),
                ACCENT_COLOR.g(),
                ACCENT_COLOR.b(),
                (255.0 * alpha) as u8,
            );
            ui.painter().text(
                icon_c,
                Align2::CENTER_CENTER,
                icon_str,
                crate::lucide::font_id(16.0),
                ic_color,
            );
        }
    }

    // Texto da notificação
    let text_pos = Pos2::new(
        toast_rect.left() + pad_x + icon_space,
        toast_rect.center().y,
    );
    let text_color = Color32::from_rgba_unmultiplied(
        TEXT_PRIMARY.r(),
        TEXT_PRIMARY.g(),
        TEXT_PRIMARY.b(),
        (255.0 * alpha) as u8,
    );
    ui.painter().text(
        text_pos,
        Align2::LEFT_CENTER,
        &t.message,
        FontId::proportional(14.5),
        text_color,
    );

    // Garante renderização fluida da animação a cada quadro
    ui.ctx().request_repaint();
}

fn paint_mini_camera(ui: &Ui, c: Pos2, color: Color32) {
    let scale = 0.62_f32;
    let p = |x: f32, y: f32| Pos2::new(c.x + (x - 12.0) * scale, c.y + (y - 12.0) * scale);
    let stroke = Stroke::new(1.4_f32, color);
    let body_pts = vec![
        p(9.8, 4.0),
        p(14.2, 4.0),
        p(16.0, 7.0),
        p(20.0, 7.0),
        p(21.5, 8.5),
        p(21.5, 18.5),
        p(20.0, 20.0),
        p(4.0, 20.0),
        p(2.5, 18.5),
        p(2.5, 8.5),
        p(4.0, 7.0),
        p(8.0, 7.0),
    ];
    ui.painter().add(egui::Shape::closed_line(body_pts, stroke));
    ui.painter()
        .circle_stroke(p(12.0, 13.0), 3.0 * scale, stroke);
}

/// Copia a imagem salva no disco para a área de transferência (clipboard) em background.
pub(crate) fn copy_screenshot_to_clipboard_async(path: PathBuf) {
    std::thread::spawn(move || {
        // Aguarda a gravação assíncrona do libmpv ser concluída no disco (até 2s)
        for _ in 0..40 {
            if let Ok(bytes) = std::fs::read(&path)
                && !bytes.is_empty()
                && let Ok(img) = image::load_from_memory(&bytes)
            {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let img_data = arboard::ImageData {
                        width: w as usize,
                        height: h as usize,
                        bytes: std::borrow::Cow::Borrowed(rgba.as_raw()),
                    };
                    if let Err(err) = cb.set_image(img_data) {
                        tracing::warn!("Falha ao copiar imagem para clipboard: {err}");
                    } else {
                        tracing::debug!("Captura copiada para a área de transferência com sucesso");
                    }
                }
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        tracing::warn!("Timeout aguardando gravação do arquivo de captura para o clipboard");
    });
}

/// Cria o diretório de capturas e gera o caminho com timestamp único.
pub(crate) fn prepare_screenshot_path() -> PathBuf {
    let base_dir = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Nixo");
    let _ = std::fs::create_dir_all(&base_dir);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    base_dir.join(format!("Nixo_{timestamp}.png"))
}
