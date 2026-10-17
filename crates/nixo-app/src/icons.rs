use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, vec2};

use crate::app::{ACCENT_COLOR, NixoApp};

#[derive(Clone, Copy)]
pub(crate) enum ControlIcon {
    Play,
    Pause,
    NextTrack,
    Volume { level: f32, muted: bool },
    Subtitles { active: bool },
    SettingsGear,
    OpenFile,
    Fullscreen { is_fullscreen: bool },
    AspectFit { active: bool },
}

impl NixoApp {
    /// Renderiza um botão Ghost minimalista (estilo YouTube/IINA) com ícone vetorial desenhado diretamente na GPU.
    pub(crate) fn render_ghost_icon_btn(
        ui: &mut Ui,
        size: Vec2,
        icon: ControlIcon,
        tooltip: &str,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered();
            let pressed = response.is_pointer_button_down_on();

            // Fundo Ghost: invisível em repouso, destaque circular/arredondado translúcido no hover/clique
            if pressed {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 38),
                );
            } else if hovered {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                );
            }

            let fg = if hovered || pressed {
                Color32::WHITE
            } else {
                Color32::from_gray(220)
            };

            let c = rect.center();
            let painter = ui.painter();

            match icon {
                ControlIcon::Play => {
                    // Triângulo de Play opticamente centralizado (+2.0px à direita), maior e nítido
                    let pts = vec![
                        Pos2::new(c.x - 6.5, c.y - 9.5),
                        Pos2::new(c.x + 10.5, c.y),
                        Pos2::new(c.x - 6.5, c.y + 9.5),
                    ];
                    painter.add(egui::Shape::convex_polygon(pts, fg, Stroke::NONE));
                }
                ControlIcon::Pause => {
                    // Duas barras verticais simétricas proporcionais ampliadas
                    let bar_w = 4.8;
                    let bar_h = 19.0;
                    let gap = 4.8;
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x - gap, c.y), vec2(bar_w, bar_h)),
                        CornerRadius::same(1),
                        fg,
                    );
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x + gap, c.y), vec2(bar_w, bar_h)),
                        CornerRadius::same(1),
                        fg,
                    );
                }
                ControlIcon::NextTrack => {
                    // Ícone Next estilo YouTube ampliado: triângulo apontando para direita + barra vertical
                    let pts = vec![
                        Pos2::new(c.x - 7.2, c.y - 9.0),
                        Pos2::new(c.x + 3.0, c.y),
                        Pos2::new(c.x - 7.2, c.y + 9.0),
                    ];
                    painter.add(egui::Shape::convex_polygon(pts, fg, Stroke::NONE));
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(c.x + 6.8, c.y), vec2(3.2, 18.0)),
                        CornerRadius::same(1),
                        fg,
                    );
                }
                ControlIcon::Subtitles { active } => {
                    // Ícone de Legendas proporcional e equilibrado
                    let icon_color = if active {
                        ACCENT_COLOR
                    } else if hovered || pressed {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(185)
                    };

                    let badge_rect = Rect::from_center_size(c, vec2(18.5, 13.5));
                    let stroke = Stroke::new(1.85_f32, icon_color);

                    // Contorno da tela de exibição
                    painter.rect_stroke(
                        badge_rect,
                        CornerRadius::same(2),
                        stroke,
                        egui::StrokeKind::Inside,
                    );

                    // Linhas de texto de diálogo na base
                    let line_stroke = Stroke::new(1.6_f32, icon_color);
                    let y1 = c.y + 0.8;
                    let y2 = c.y + 3.8;

                    // Linha 1: traço curto à esquerda + traço longo à direita
                    painter.line_segment(
                        [Pos2::new(c.x - 5.2, y1), Pos2::new(c.x - 2.0, y1)],
                        line_stroke,
                    );
                    painter.line_segment(
                        [Pos2::new(c.x + 0.8, y1), Pos2::new(c.x + 5.2, y1)],
                        line_stroke,
                    );

                    // Linha 2: traço longo à esquerda + traço curto à direita
                    painter.line_segment(
                        [Pos2::new(c.x - 5.2, y2), Pos2::new(c.x + 0.2, y2)],
                        line_stroke,
                    );
                    painter.line_segment(
                        [Pos2::new(c.x + 2.8, y2), Pos2::new(c.x + 5.2, y2)],
                        line_stroke,
                    );
                }
                ControlIcon::SettingsGear => {
                    // Ícone de engrenagem estilo YouTube proporcional e arejado
                    let r_outer = 8.8;
                    let r_inner = 4.2;
                    let stroke = Stroke::new(1.85_f32, fg);
                    painter.circle_stroke(c, r_inner, stroke);

                    // Dentes da engrenagem distribuídos simetricamente
                    let teeth_stroke = Stroke::new(2.6_f32, fg);
                    for i in 0..6 {
                        let angle = (i as f32) * std::f32::consts::PI / 3.0;
                        let p1 = Pos2::new(
                            c.x + (r_inner + 1.0) * angle.cos(),
                            c.y + (r_inner + 1.0) * angle.sin(),
                        );
                        let p2 =
                            Pos2::new(c.x + r_outer * angle.cos(), c.y + r_outer * angle.sin());
                        painter.line_segment([p1, p2], teeth_stroke);
                    }
                }
                ControlIcon::Volume { level, muted } => {
                    // Corpo do alto-falante ampliado
                    let sx = c.x - 4.2;
                    painter.rect_filled(
                        Rect::from_min_max(
                            Pos2::new(sx - 7.2, c.y - 3.8),
                            Pos2::new(sx - 2.5, c.y + 3.8),
                        ),
                        CornerRadius::same(1),
                        fg,
                    );
                    let cone = vec![
                        Pos2::new(sx - 2.8, c.y - 3.8),
                        Pos2::new(sx + 3.5, c.y - 9.2),
                        Pos2::new(sx + 3.5, c.y + 9.2),
                        Pos2::new(sx - 2.8, c.y + 3.8),
                    ];
                    painter.add(egui::Shape::convex_polygon(cone, fg, Stroke::NONE));

                    let stroke = Stroke::new(2.2_f32, fg);
                    if muted || level <= 0.1 {
                        // Ícone 'X' de mudo ao lado do cone
                        let x0 = sx + 6.2;
                        let x1 = sx + 13.5;
                        painter.line_segment(
                            [Pos2::new(x0, c.y - 4.2), Pos2::new(x1, c.y + 4.2)],
                            stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(x1, c.y - 4.2), Pos2::new(x0, c.y + 4.2)],
                            stroke,
                        );
                    } else {
                        // Onda sonora interna (curva suave)
                        let origin = Pos2::new(sx + 2.0, c.y);
                        let wave1: Vec<Pos2> = (-3..=3)
                            .map(|i| {
                                let angle = (i as f32) * 0.22;
                                Pos2::new(
                                    origin.x + 6.5 * angle.cos(),
                                    origin.y + 6.5 * angle.sin(),
                                )
                            })
                            .collect();
                        painter.add(egui::Shape::line(wave1, stroke));

                        // Onda sonora externa (para volume >= 50%)
                        if level >= 50.0 {
                            let wave2: Vec<Pos2> = (-3..=3)
                                .map(|i| {
                                    let angle = (i as f32) * 0.24;
                                    Pos2::new(
                                        origin.x + 11.0 * angle.cos(),
                                        origin.y + 11.0 * angle.sin(),
                                    )
                                })
                                .collect();
                            painter.add(egui::Shape::line(wave2, stroke));
                        }
                    }
                }
                ControlIcon::OpenFile => {
                    // Ícone de pasta vetorial estilo SF Symbols / Lucide (silhueta contínua elegante)
                    let stroke = Stroke::new(1.85_f32, fg);
                    let x0 = c.x - 8.5;
                    let x1 = c.x + 8.5;
                    let y_tab = c.y - 6.2;
                    let y_top = c.y - 3.4;
                    let y_bot = c.y + 6.2;
                    let tab_w = 5.5;

                    // Contorno contínuo da pasta com aba superior integrada e cantos chanfrados suaves
                    let folder_pts = vec![
                        Pos2::new(x0 + 1.6, y_tab),
                        Pos2::new(x0 + tab_w, y_tab),
                        Pos2::new(x0 + tab_w + 2.0, y_top),
                        Pos2::new(x1 - 1.6, y_top),
                        Pos2::new(x1, y_top + 1.6),
                        Pos2::new(x1, y_bot - 1.6),
                        Pos2::new(x1 - 1.6, y_bot),
                        Pos2::new(x0 + 1.6, y_bot),
                        Pos2::new(x0, y_bot - 1.6),
                        Pos2::new(x0, y_tab + 1.6),
                    ];
                    painter.add(egui::Shape::closed_line(folder_pts, stroke));

                    // Vinco sutil do bolso frontal para dar profundidade estética
                    let fold_y = c.y + 0.8;
                    let pocket_stroke = Stroke::new(
                        1.3_f32,
                        Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), 170),
                    );
                    painter.line_segment(
                        [Pos2::new(x0 + 1.0, fold_y), Pos2::new(x1 - 1.0, fold_y)],
                        pocket_stroke,
                    );
                }
                ControlIcon::Fullscreen { is_fullscreen } => {
                    // Cantoneiras estilo YouTube (4 cantos em L) leves e proporcionais
                    let r = 8.5;
                    let arm = 5.0;
                    let stroke = Stroke::new(1.85_f32, fg);

                    if !is_fullscreen {
                        // Apontando para fora (Expandir)
                        // Sup-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y - r),
                                Pos2::new(c.x - r + arm, c.y - r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y - r),
                                Pos2::new(c.x - r, c.y - r + arm),
                            ],
                            stroke,
                        );
                        // Sup-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y - r),
                                Pos2::new(c.x + r - arm, c.y - r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y - r),
                                Pos2::new(c.x + r, c.y - r + arm),
                            ],
                            stroke,
                        );
                        // Inf-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y + r),
                                Pos2::new(c.x - r + arm, c.y + r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - r, c.y + r),
                                Pos2::new(c.x - r, c.y + r - arm),
                            ],
                            stroke,
                        );
                        // Inf-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y + r),
                                Pos2::new(c.x + r - arm, c.y + r),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + r, c.y + r),
                                Pos2::new(c.x + r, c.y + r - arm),
                            ],
                            stroke,
                        );
                    } else {
                        // Apontando para dentro (Sair da Tela Cheia)
                        let inner = r - arm + 1.0;
                        // Sup-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y - inner),
                                Pos2::new(c.x - r, c.y - inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y - inner),
                                Pos2::new(c.x - inner, c.y - r),
                            ],
                            stroke,
                        );
                        // Sup-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y - inner),
                                Pos2::new(c.x + r, c.y - inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y - inner),
                                Pos2::new(c.x + inner, c.y - r),
                            ],
                            stroke,
                        );
                        // Inf-Esq
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y + inner),
                                Pos2::new(c.x - r, c.y + inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x - inner, c.y + inner),
                                Pos2::new(c.x - inner, c.y + r),
                            ],
                            stroke,
                        );
                        // Inf-Dir
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y + inner),
                                Pos2::new(c.x + r, c.y + inner),
                            ],
                            stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + inner, c.y + inner),
                                Pos2::new(c.x + inner, c.y + r),
                            ],
                            stroke,
                        );
                    }
                }
                ControlIcon::AspectFit { active } => {
                    let icon_color = if active { ACCENT_COLOR } else { fg };
                    let stroke = Stroke::new(1.85_f32, icon_color);

                    let frame = Rect::from_center_size(c, vec2(18.0, 13.5));
                    painter.rect_stroke(frame, CornerRadius::same(2), stroke, StrokeKind::Inside);

                    if active {
                        // Preenchimento translúcido com brilho verde Nixo
                        let inner = frame.shrink(2.0);
                        painter.rect_filled(
                            inner,
                            CornerRadius::same(1),
                            Color32::from_rgba_unmultiplied(47, 229, 11, 40),
                        );
                        // Setas duplas de expansão horizontal (indicando eliminação de barras pretas)
                        let arrow_stroke = Stroke::new(1.6_f32, ACCENT_COLOR);
                        let y = c.y;
                        painter.line_segment(
                            [Pos2::new(c.x - 4.5, y), Pos2::new(c.x + 4.5, y)],
                            arrow_stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(c.x - 4.5, y), Pos2::new(c.x - 2.5, y - 2.0)],
                            arrow_stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(c.x - 4.5, y), Pos2::new(c.x - 2.5, y + 2.0)],
                            arrow_stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(c.x + 4.5, y), Pos2::new(c.x + 2.5, y - 2.0)],
                            arrow_stroke,
                        );
                        painter.line_segment(
                            [Pos2::new(c.x + 4.5, y), Pos2::new(c.x + 2.5, y + 2.0)],
                            arrow_stroke,
                        );
                    } else {
                        // Barras sutis de proporção (letterbox/pillarbox) indicando proporção solta
                        let bar_color = Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), 90);
                        let bar_stroke = Stroke::new(1.3_f32, bar_color);
                        painter.line_segment(
                            [
                                Pos2::new(c.x - 4.5, c.y - 4.2),
                                Pos2::new(c.x - 4.5, c.y + 4.2),
                            ],
                            bar_stroke,
                        );
                        painter.line_segment(
                            [
                                Pos2::new(c.x + 4.5, c.y - 4.2),
                                Pos2::new(c.x + 4.5, c.y + 4.2),
                            ],
                            bar_stroke,
                        );
                    }
                }
            }
        }

        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // Tooltip flutuante centralizado estilo YouTube/IINA: cápsula escura elegante logo acima do botão
        if !tooltip.is_empty() {
            let tt_id = response.id.with("yt_tooltip");
            let is_hovered = response.hovered();
            let tt_alpha = ui.ctx().animate_bool_with_time(tt_id, is_hovered, 0.12);
            Self::render_player_tooltip(ui.ctx(), tt_id, rect, tooltip, tt_alpha);
        }

        response
    }
}
