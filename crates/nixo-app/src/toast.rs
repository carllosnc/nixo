//! Sistema de notificações flutuantes (Toasts) do Nixo.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, Ui, vec2,
};

use crate::app::ACCENT_COLOR;

/// Estado de uma notificação temporária no topo da tela.
pub struct ToastNotification {
    pub message: String,
    pub path: Option<PathBuf>,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastNotification {
    pub fn new(message: String, path: Option<PathBuf>) -> Self {
        Self {
            message,
            path,
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

    // Mede a largura necessária para o texto
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            t.message.clone(),
            FontId::proportional(13.5),
            Color32::WHITE,
        )
    });

    let pad_x = 18.0;
    let pad_y = 9.0;
    let icon_space = 24.0;
    let total_w = galley.size().x + pad_x * 2.0 + icon_space;
    let total_h = galley.size().y + pad_y * 2.0;

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
        #[cfg(not(target_os = "windows"))]
        {
            if let Some(parent) = path.parent() {
                let _ = open::that(parent);
            }
        }
    }

    // Fundo da cápsula translúcida OLED com borda
    let bg_alpha = (235.0 * alpha) as u8;
    let stroke_alpha = (180.0 * alpha) as u8;
    ui.painter().rect_filled(
        toast_rect,
        CornerRadius::same(99),
        Color32::from_rgba_unmultiplied(16, 18, 22, bg_alpha),
    );
    let border_color = if hovered {
        ACCENT_COLOR
    } else {
        Color32::from_rgb(60, 64, 76)
    };
    ui.painter().rect_stroke(
        toast_rect,
        CornerRadius::same(99),
        Stroke::new(
            1.1_f32,
            Color32::from_rgba_unmultiplied(
                border_color.r(),
                border_color.g(),
                border_color.b(),
                stroke_alpha,
            ),
        ),
        egui::StrokeKind::Inside,
    );

    // Ícone de câmera em verde Nixo à esquerda
    let icon_c = Pos2::new(toast_rect.left() + pad_x + 6.0, toast_rect.center().y);
    let icon_color = Color32::from_rgba_unmultiplied(
        ACCENT_COLOR.r(),
        ACCENT_COLOR.g(),
        ACCENT_COLOR.b(),
        (255.0 * alpha) as u8,
    );
    paint_mini_camera(ui, icon_c, icon_color);

    // Texto da notificação
    let text_pos = Pos2::new(
        toast_rect.left() + pad_x + icon_space,
        toast_rect.center().y,
    );
    let text_color = Color32::from_rgba_unmultiplied(235, 235, 240, (255.0 * alpha) as u8);
    ui.painter().text(
        text_pos,
        Align2::LEFT_CENTER,
        &t.message,
        FontId::proportional(13.5),
        text_color,
    );

    // Garante renderização fluida da animação a cada quadro
    ui.ctx().request_repaint();
}

fn paint_mini_camera(ui: &Ui, c: Pos2, color: Color32) {
    let scale = 0.58_f32;
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
