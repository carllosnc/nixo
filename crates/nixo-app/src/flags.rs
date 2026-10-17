use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, vec2};
use nixo_core::FlagCode;

use crate::app::NixoApp;

impl NixoApp {
    /// Renderiza bandeiras procedurais vetoriais ultra-nítidas diretamente na GPU com egui/wgpu.
    pub(crate) fn render_vector_flag(painter: &egui::Painter, rect: Rect, flag: FlagCode) {
        let border_stroke =
            Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 38));
        let cr = CornerRadius::same(2);

        match flag {
            FlagCode::Brazil => {
                // Fundo verde oficial (#009C3B)
                painter.rect_filled(rect, cr, Color32::from_rgb(0, 156, 59));
                // Losango amarelo (#FFDF00)
                let c = rect.center();
                let diamond = vec![
                    Pos2::new(c.x, rect.top() + 1.2),
                    Pos2::new(rect.right() - 1.8, c.y),
                    Pos2::new(c.x, rect.bottom() - 1.2),
                    Pos2::new(rect.left() + 1.8, c.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    diamond,
                    Color32::from_rgb(255, 223, 0),
                    Stroke::NONE,
                ));
                // Círculo azul central (#002776)
                painter.circle_filled(c, 2.5, Color32::from_rgb(0, 39, 118));
                // Faixa branca central
                painter.line_segment(
                    [
                        Pos2::new(c.x - 1.8, c.y + 0.4),
                        Pos2::new(c.x + 1.8, c.y - 0.4),
                    ],
                    Stroke::new(0.65_f32, Color32::WHITE),
                );
            }
            FlagCode::USA => {
                // Listras vermelhas e brancas
                painter.rect_filled(rect, cr, Color32::WHITE);
                let stripe_h = rect.height() / 5.0;
                let red = Color32::from_rgb(179, 25, 66);
                for i in [0, 2, 4] {
                    let y = rect.top() + (i as f32) * stripe_h;
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(rect.left(), y),
                            vec2(rect.width(), stripe_h),
                        ),
                        CornerRadius::ZERO,
                        red,
                    );
                }
                // Cantão azul superior esquerdo
                let canton =
                    Rect::from_min_size(rect.min, vec2(rect.width() * 0.45, rect.height() * 0.55));
                painter.rect_filled(canton, CornerRadius::ZERO, Color32::from_rgb(10, 49, 97));
                // Constelação de micro-estrelas no cantão azul
                let c = canton.center();
                painter.circle_filled(Pos2::new(c.x - 1.8, c.y - 1.1), 0.55_f32, Color32::WHITE);
                painter.circle_filled(Pos2::new(c.x + 1.8, c.y - 1.1), 0.55_f32, Color32::WHITE);
                painter.circle_filled(Pos2::new(c.x - 1.8, c.y + 1.1), 0.55_f32, Color32::WHITE);
                painter.circle_filled(Pos2::new(c.x + 1.8, c.y + 1.1), 0.55_f32, Color32::WHITE);
            }
            FlagCode::UK => {
                // Fundo azul (#012169)
                painter.rect_filled(rect, cr, Color32::from_rgb(1, 33, 105));
                let red = Color32::from_rgb(200, 16, 46);
                // Cruz diagonal branca
                painter.line_segment(
                    [rect.left_top(), rect.right_bottom()],
                    Stroke::new(1.8_f32, Color32::WHITE),
                );
                painter.line_segment(
                    [rect.right_top(), rect.left_bottom()],
                    Stroke::new(1.8_f32, Color32::WHITE),
                );
                // Cruz diagonal vermelha
                painter.line_segment(
                    [rect.left_top(), rect.right_bottom()],
                    Stroke::new(0.9_f32, red),
                );
                painter.line_segment(
                    [rect.right_top(), rect.left_bottom()],
                    Stroke::new(0.9_f32, red),
                );
                // Cruz de São Jorge branca (larga)
                let c = rect.center();
                painter.line_segment(
                    [Pos2::new(c.x, rect.top()), Pos2::new(c.x, rect.bottom())],
                    Stroke::new(2.8_f32, Color32::WHITE),
                );
                painter.line_segment(
                    [Pos2::new(rect.left(), c.y), Pos2::new(rect.right(), c.y)],
                    Stroke::new(2.8_f32, Color32::WHITE),
                );
                // Cruz de São Jorge vermelha
                painter.line_segment(
                    [Pos2::new(c.x, rect.top()), Pos2::new(c.x, rect.bottom())],
                    Stroke::new(1.5_f32, red),
                );
                painter.line_segment(
                    [Pos2::new(rect.left(), c.y), Pos2::new(rect.right(), c.y)],
                    Stroke::new(1.5_f32, red),
                );
            }
            FlagCode::Japan => {
                painter.rect_filled(rect, cr, Color32::from_rgb(248, 248, 248));
                painter.circle_filled(rect.center(), 3.2, Color32::from_rgb(188, 0, 45));
            }
            FlagCode::Portugal => {
                let w = rect.width();
                let split_x = rect.left() + w * 0.4;
                painter.rect_filled(
                    Rect::from_min_max(rect.min, Pos2::new(split_x, rect.bottom())),
                    CornerRadius::ZERO,
                    Color32::from_rgb(0, 102, 0),
                );
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(split_x, rect.top()), rect.max),
                    CornerRadius::ZERO,
                    Color32::from_rgb(255, 0, 0),
                );
                // Esfera armilar no centro da divisão
                painter.circle_filled(
                    Pos2::new(split_x, rect.center().y),
                    2.2,
                    Color32::from_rgb(255, 223, 0),
                );
                painter.circle_filled(Pos2::new(split_x, rect.center().y), 1.2, Color32::WHITE);
            }
            FlagCode::Spain => {
                let h = rect.height();
                let red = Color32::from_rgb(170, 21, 27);
                let yellow = Color32::from_rgb(241, 191, 0);
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(rect.width(), h * 0.25)),
                    CornerRadius::ZERO,
                    red,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h * 0.25),
                        vec2(rect.width(), h * 0.5),
                    ),
                    CornerRadius::ZERO,
                    yellow,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h * 0.75),
                        vec2(rect.width(), h * 0.25),
                    ),
                    CornerRadius::ZERO,
                    red,
                );
                // Brasão simplificado
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(rect.left() + rect.width() * 0.32, rect.center().y),
                        vec2(2.0, 2.5),
                    ),
                    CornerRadius::same(1),
                    Color32::from_rgb(140, 20, 20),
                );
            }
            FlagCode::France => {
                let w = rect.width() / 3.0;
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(w, rect.height())),
                    CornerRadius::ZERO,
                    Color32::from_rgb(0, 38, 84),
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left() + w, rect.top()),
                        vec2(w, rect.height()),
                    ),
                    CornerRadius::ZERO,
                    Color32::WHITE,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left() + w * 2.0, rect.top()),
                        vec2(w, rect.height()),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(237, 41, 57),
                );
            }
            FlagCode::Germany => {
                let h = rect.height() / 3.0;
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(rect.width(), h)),
                    CornerRadius::ZERO,
                    Color32::from_rgb(20, 20, 20),
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h),
                        vec2(rect.width(), h),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(221, 0, 0),
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h * 2.0),
                        vec2(rect.width(), h),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(255, 206, 0),
                );
            }
            FlagCode::Italy => {
                let w = rect.width() / 3.0;
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(w, rect.height())),
                    CornerRadius::ZERO,
                    Color32::from_rgb(0, 146, 70),
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left() + w, rect.top()),
                        vec2(w, rect.height()),
                    ),
                    CornerRadius::ZERO,
                    Color32::WHITE,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left() + w * 2.0, rect.top()),
                        vec2(w, rect.height()),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(206, 43, 55),
                );
            }
            FlagCode::SouthKorea => {
                painter.rect_filled(rect, cr, Color32::from_rgb(248, 248, 248));
                let c = rect.center();
                // Taegeuk
                painter.circle_filled(c, 2.8, Color32::from_rgb(0, 52, 120));
                let top_half = vec![
                    Pos2::new(c.x - 2.8, c.y),
                    Pos2::new(c.x, c.y - 2.8),
                    Pos2::new(c.x + 2.8, c.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    top_half,
                    Color32::from_rgb(205, 46, 58),
                    Stroke::NONE,
                ));
            }
            FlagCode::China => {
                painter.rect_filled(rect, cr, Color32::from_rgb(222, 41, 16));
                // Estrela amarela grande
                painter.circle_filled(
                    Pos2::new(rect.left() + 4.0, rect.top() + 3.5),
                    1.6,
                    Color32::from_rgb(255, 222, 0),
                );
                // Estrelinha secundária
                painter.circle_filled(
                    Pos2::new(rect.left() + 7.0, rect.top() + 2.2),
                    0.7,
                    Color32::from_rgb(255, 222, 0),
                );
            }
            FlagCode::Russia => {
                let h = rect.height() / 3.0;
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(rect.width(), h)),
                    CornerRadius::ZERO,
                    Color32::WHITE,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h),
                        vec2(rect.width(), h),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(0, 57, 166),
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + h * 2.0),
                        vec2(rect.width(), h),
                    ),
                    CornerRadius::ZERO,
                    Color32::from_rgb(213, 43, 30),
                );
            }
            FlagCode::Generic => {
                // Ícone neutro translúcido em cinza escuro
                painter.rect_filled(rect, cr, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
                let c = rect.center();
                painter.circle_stroke(c, 3.8, Stroke::new(1.1_f32, Color32::from_gray(190)));
                painter.line_segment(
                    [Pos2::new(c.x - 3.8, c.y), Pos2::new(c.x + 3.8, c.y)],
                    Stroke::new(1.0_f32, Color32::from_gray(190)),
                );
            }
        }

        // Borda sutil de contorno para destacar a bandeira em fundos escuros
        painter.rect_stroke(rect, cr, border_stroke, egui::StrokeKind::Inside);
    }
}
