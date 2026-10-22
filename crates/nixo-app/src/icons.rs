use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use nixo_core::RepeatMode;

use crate::app::{ACCENT_COLOR, NixoApp};

#[derive(Clone, Copy)]
pub(crate) enum ControlIcon {
    Play,
    Pause,
    PreviousTrack,
    NextTrack,
    Repeat { mode: RepeatMode },
    Volume { level: f32, muted: bool },
    Subtitles { active: bool },
    SettingsGear,
    Home,
    Fullscreen { is_fullscreen: bool },
    AspectFit { active: bool },
    Pip { active: bool },
    Screenshot,
}

impl NixoApp {
    /// Renderiza um botão Ghost minimalista com ícone vetorial desenhado na GPU
    /// diretamente alinhado às proporções e geometrias da biblioteca Lucide Icons (grid 24x24).
    pub(crate) fn render_ghost_icon_btn(
        ui: &mut Ui,
        size: Vec2,
        icon: ControlIcon,
        tooltip: &str,
    ) -> egui::Response {
        Self::render_ghost_icon_btn_enabled(ui, size, icon, tooltip, true)
    }

    /// Renderiza um botão Ghost com suporte a estado ativado/desativado (disabled).
    pub(crate) fn render_ghost_icon_btn_enabled(
        ui: &mut Ui,
        size: Vec2,
        icon: ControlIcon,
        tooltip: &str,
        enabled: bool,
    ) -> egui::Response {
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(size, sense);

        if ui.is_rect_visible(rect) {
            let hovered = enabled && response.hovered();
            let pressed = enabled && response.is_pointer_button_down_on();

            let is_active = match icon {
                ControlIcon::Subtitles { active } => active,
                ControlIcon::AspectFit { active } => active,
                ControlIcon::Pip { active } => active,
                ControlIcon::Repeat { mode } => mode != RepeatMode::Off,
                _ => false,
            };

            // Botão full-rounded (círculo proporcional centralizado no slot)
            let btn_size = 40.0_f32.min(rect.width()).min(rect.height());
            let btn_rect = Rect::from_center_size(rect.center(), Vec2::splat(btn_size));
            let radius = CornerRadius::same(99);

            // Botão com estado ativo em alto contraste (verde da marca Nixo) ou ghost sutil em repouso
            let bg_color = if !enabled {
                Color32::TRANSPARENT
            } else if is_active {
                if pressed {
                    Color32::from_rgb(45, 142, 26)
                } else if hovered {
                    Color32::from_rgb(65, 195, 40)
                } else {
                    ACCENT_COLOR
                }
            } else if pressed {
                Color32::from_rgba_unmultiplied(255, 255, 255, 42)
            } else if hovered {
                Color32::from_rgba_unmultiplied(255, 255, 255, 24)
            } else {
                Color32::TRANSPARENT
            };

            if bg_color != Color32::TRANSPARENT {
                ui.painter().rect_filled(btn_rect, radius, bg_color);
            }

            let fg = if !enabled {
                Color32::from_rgba_unmultiplied(255, 255, 255, 60)
            } else if is_active || hovered || pressed {
                Color32::WHITE
            } else {
                Color32::from_gray(220)
            };

            let c = rect.center();
            let painter = ui.painter();

            // Escala e projeção do grid Lucide 24x24 para ~23px na tela
            let scale = 0.96_f32;
            let p = |x: f32, y: f32| Pos2::new(c.x + (x - 12.0) * scale, c.y + (y - 12.0) * scale);
            let stroke = Stroke::new(1.95_f32, fg);

            match icon {
                ControlIcon::Play => {
                    // Lucide 'play': triângulo com cantos arredondados, opticamente centralizado (+0.9px no eixo X)
                    let po = |x: f32, y: f32| {
                        Pos2::new(c.x + (x - 12.0 + 0.9) * scale, c.y + (y - 12.0) * scale)
                    };
                    let pts = vec![po(6.0, 4.8), po(18.5, 12.0), po(6.0, 19.2)];
                    painter.add(Shape::convex_polygon(pts, fg, Stroke::new(1.85_f32, fg)));
                }
                ControlIcon::Pause => {
                    // Lucide 'pause': duas barras verticais com cantos arredondados (x=5..10 e x=14..19)
                    let r1 = Rect::from_min_max(p(5.0, 3.2), p(10.0, 20.8));
                    let r2 = Rect::from_min_max(p(14.0, 3.2), p(19.0, 20.8));
                    painter.rect_filled(r1, CornerRadius::same(1), fg);
                    painter.rect_filled(r2, CornerRadius::same(1), fg);
                }
                ControlIcon::PreviousTrack => {
                    // Lucide 'skip-back': barra vertical de parada + triângulo apontando para a esquerda
                    painter.rect_filled(
                        Rect::from_min_max(p(3.0, 4.0), p(5.5, 20.0)),
                        CornerRadius::same(1),
                        fg,
                    );
                    let pts = vec![p(20.0, 4.8), p(8.0, 12.0), p(20.0, 19.2)];
                    painter.add(Shape::convex_polygon(pts, fg, Stroke::new(1.6_f32, fg)));
                }
                ControlIcon::NextTrack => {
                    // Lucide 'skip-forward': triângulo apontando para a direita + barra vertical de parada
                    let pts = vec![p(4.0, 4.8), p(16.0, 12.0), p(4.0, 19.2)];
                    painter.add(Shape::convex_polygon(pts, fg, Stroke::new(1.6_f32, fg)));
                    painter.rect_filled(
                        Rect::from_min_max(p(18.5, 4.0), p(21.0, 20.0)),
                        CornerRadius::same(1),
                        fg,
                    );
                }
                ControlIcon::Repeat { mode } => {
                    let icon_color = if mode != RepeatMode::Off {
                        Color32::WHITE
                    } else {
                        fg
                    };
                    let rep_stroke = Stroke::new(1.85_f32, icon_color);

                    // Seta superior (indo para a direita)
                    painter.add(Shape::line(
                        vec![p(4.0, 13.0), p(4.0, 8.5), p(5.5, 7.0), p(18.0, 7.0)],
                        rep_stroke,
                    ));
                    painter.add(Shape::line(
                        vec![p(15.0, 4.0), p(18.0, 7.0), p(15.0, 10.0)],
                        rep_stroke,
                    ));

                    // Seta inferior (indo para a esquerda)
                    painter.add(Shape::line(
                        vec![p(20.0, 11.0), p(20.0, 15.5), p(18.5, 17.0), p(6.0, 17.0)],
                        rep_stroke,
                    ));
                    painter.add(Shape::line(
                        vec![p(9.0, 14.0), p(6.0, 17.0), p(9.0, 20.0)],
                        rep_stroke,
                    ));

                    if mode == RepeatMode::One {
                        // Indicador '1' no centro do ícone de repetição
                        let one_st = Stroke::new(2.0_f32, icon_color);
                        painter.line_segment([p(12.0, 9.5), p(12.0, 14.5)], one_st);
                        painter.line_segment([p(10.2, 11.2), p(12.0, 9.5)], one_st);
                    }
                }
                ControlIcon::Volume { level, muted } => {
                    // Lucide 'volume-2' / 'volume-1' / 'volume-x'
                    let body = vec![
                        p(3.0, 8.5),
                        p(6.2, 8.5),
                        p(11.0, 4.5),
                        p(11.0, 19.5),
                        p(6.2, 15.5),
                        p(3.0, 15.5),
                    ];
                    painter.add(Shape::convex_polygon(body, fg, Stroke::new(1.4_f32, fg)));

                    if muted || level <= 0.1 {
                        // Lucide 'volume-x': 'X' à direita do alto-falante
                        painter.line_segment([p(15.5, 8.5), p(21.5, 15.5)], stroke);
                        painter.line_segment([p(15.5, 15.5), p(21.5, 8.5)], stroke);
                    } else {
                        // Lucide 'volume-1': arco de onda interna (raio ~5.2)
                        let arc1: Vec<Pos2> = (-3..=3)
                            .map(|i| {
                                let a = (i as f32 / 3.0) * 0.78_f32;
                                p(11.0 + 5.2 * a.cos(), 12.0 + 5.2 * a.sin())
                            })
                            .collect();
                        painter.add(Shape::line(arc1, stroke));

                        // Lucide 'volume-2': arco de onda externa (raio ~9.2 para volume >= 50%)
                        if level >= 50.0 {
                            let arc2: Vec<Pos2> = (-4..=4)
                                .map(|i| {
                                    let a = (i as f32 / 4.0) * 0.78_f32;
                                    p(11.0 + 9.2 * a.cos(), 12.0 + 9.2 * a.sin())
                                })
                                .collect();
                            painter.add(Shape::line(arc2, stroke));
                        }
                    }
                }
                ControlIcon::Subtitles { active } => {
                    // Lucide 'captions': retângulo arredondado 18x14 com linhas de diálogo estilizadas
                    let icon_color = if active || hovered || pressed {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(185)
                    };
                    let cc_st = Stroke::new(1.85_f32, icon_color);
                    let frame = Rect::from_min_max(p(3.0, 5.0), p(21.0, 19.0));
                    painter.rect_stroke(frame, CornerRadius::same(2), cc_st, StrokeKind::Inside);

                    let line_st = Stroke::new(1.65_f32, icon_color);
                    // Linha superior (y=10.5): curto + longo
                    painter.line_segment([p(6.8, 10.5), p(9.2, 10.5)], line_st);
                    painter.line_segment([p(12.5, 10.5), p(17.2, 10.5)], line_st);
                    // Linha inferior (y=14.5): longo + curto
                    painter.line_segment([p(6.8, 14.5), p(11.5, 14.5)], line_st);
                    painter.line_segment([p(14.8, 14.5), p(17.2, 14.5)], line_st);
                }
                ControlIcon::SettingsGear => {
                    // Lucide 'settings': anel central com engrenagem de 6 dentes arredondados simétricos
                    painter.circle_stroke(c, 3.0 * scale, stroke);

                    let cog_pts: Vec<Pos2> = (0..48)
                        .map(|i| {
                            let angle = (i as f32 / 48.0) * std::f32::consts::TAU;
                            let tooth = (angle * 6.0).cos();
                            let r = 8.2 + 1.6 * (tooth / 0.85).clamp(-1.0, 1.0);
                            Pos2::new(c.x + r * scale * angle.cos(), c.y + r * scale * angle.sin())
                        })
                        .collect();
                    painter.add(Shape::closed_line(cog_pts, stroke));
                }
                ControlIcon::Home => {
                    // Glifo 'home' (\u{e0f5}) renderizado diretamente da fonte Lucide
                    painter.text(
                        c,
                        Align2::CENTER_CENTER,
                        crate::lucide::HOME,
                        crate::lucide::font_id(20.0),
                        fg,
                    );
                }
                ControlIcon::Fullscreen { is_fullscreen } => {
                    if !is_fullscreen {
                        // Lucide 'maximize': 4 cantoneiras apontando para fora
                        painter.add(Shape::line(
                            vec![p(8.0, 3.5), p(5.0, 3.5), p(3.5, 5.0), p(3.5, 8.0)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(16.0, 3.5), p(19.0, 3.5), p(20.5, 5.0), p(20.5, 8.0)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(3.5, 16.0), p(3.5, 19.0), p(5.0, 20.5), p(8.0, 20.5)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(16.0, 20.5), p(19.0, 20.5), p(20.5, 19.0), p(20.5, 16.0)],
                            stroke,
                        ));
                    } else {
                        // Lucide 'minimize': 4 cantoneiras internas apontando para dentro
                        painter.add(Shape::line(
                            vec![p(8.5, 3.5), p(8.5, 6.0), p(6.0, 8.5), p(3.5, 8.5)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(20.5, 8.5), p(18.0, 8.5), p(15.5, 6.0), p(15.5, 3.5)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(3.5, 15.5), p(6.0, 15.5), p(8.5, 18.0), p(8.5, 20.5)],
                            stroke,
                        ));
                        painter.add(Shape::line(
                            vec![p(20.5, 15.5), p(18.0, 15.5), p(15.5, 18.0), p(15.5, 20.5)],
                            stroke,
                        ));
                    }
                }
                ControlIcon::AspectFit { active } => {
                    // Lucide frame + chevrons de expansão ou barras de proporção
                    let icon_color = if active { Color32::WHITE } else { fg };
                    let st = Stroke::new(1.85_f32, icon_color);
                    let frame = Rect::from_min_max(p(2.5, 5.0), p(21.5, 19.0));
                    painter.rect_stroke(frame, CornerRadius::same(2), st, StrokeKind::Inside);

                    if active {
                        // Trava ativa: chevrons de expansão em branco puro de alto contraste sobre o fundo verde da marca
                        let arrow_st = Stroke::new(1.85_f32, Color32::WHITE);
                        painter.line_segment([p(6.0, 12.0), p(18.0, 12.0)], arrow_st);
                        painter.add(Shape::line(
                            vec![p(8.5, 9.5), p(6.0, 12.0), p(8.5, 14.5)],
                            arrow_st,
                        ));
                        painter.add(Shape::line(
                            vec![p(15.5, 9.5), p(18.0, 12.0), p(15.5, 14.5)],
                            arrow_st,
                        ));
                    } else {
                        let bar_color = Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), 90);
                        let bar_st = Stroke::new(1.4_f32, bar_color);
                        painter.line_segment([p(6.5, 7.0), p(6.5, 17.0)], bar_st);
                        painter.line_segment([p(17.5, 7.0), p(17.5, 17.0)], bar_st);
                    }
                }
                ControlIcon::Pip { active } => {
                    let icon_color = if active { Color32::WHITE } else { fg };
                    let st = Stroke::new(1.85_f32, icon_color);
                    let outer = Rect::from_min_max(p(2.5, 4.5), p(21.5, 19.5));
                    painter.rect_stroke(outer, CornerRadius::same(2), st, StrokeKind::Inside);

                    let inner = Rect::from_min_max(p(11.5, 11.0), p(19.5, 17.5));
                    let inner_fill = if active { Color32::WHITE } else { fg };
                    painter.rect_filled(inner, CornerRadius::same(1), inner_fill);
                }
                ControlIcon::Screenshot => {
                    // Lucide 'camera': corpo canônico da câmera com visor superior e lente central
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
                    painter.add(Shape::closed_line(body_pts, stroke));
                    painter.circle_stroke(p(12.0, 13.0), 3.2 * scale, stroke);
                }
            }
        }

        let response = if enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        };

        if !tooltip.is_empty() {
            crate::components::show_tooltip(ui.ctx(), &response, tooltip);
        }

        response
    }
}
