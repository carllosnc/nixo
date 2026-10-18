//! Visualização completa e interativa do Design System do Nixo.
//!
//! Reúne em uma única tela todos os tokens de superfície, tipografia,
//! botões full-rounded, breadcrumbs, cards de pasta sem wrappers,
//! cards de mídia e estados vazios para validação e referência visual.

use egui::{Align, Align2, Color32, CursorIcon, Layout, Pos2, Rect, RichText, Sense, Ui, vec2};

use super::header::HomeAction;
use crate::components::button::{ButtonVariant, ghost, primary, render_button, secondary};
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;
use crate::theme::font_bold;

/// Renderiza a página completa do Design System Showcase.
pub fn render_design_system_view(ui: &mut Ui) -> HomeAction {
    let mut action = HomeAction::None;

    // Top Bar de ações do showcase
    ui.horizontal(|ui| {
        if secondary(ui, Some(lucide::ARROW_LEFT), "Voltar para Biblioteca").clicked() {
            action = HomeAction::CloseDesignSystem;
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("NIXO DESIGN SYSTEM")
                .font(font_bold(14.0))
                .color(Color32::WHITE),
        );
        ui.label(
            RichText::new("— Visualização Unificada de Componentes")
                .font(font_body())
                .color(TEXT_MUTED),
        );
    });

    ui.add_space(20.0);

    render_surfaces_section(ui);
    ui.add_space(24.0);

    render_typography_section(ui);
    ui.add_space(24.0);

    render_buttons_section(ui);
    ui.add_space(24.0);

    render_navigation_section(ui);
    ui.add_space(24.0);

    render_folder_cards_section(ui);
    ui.add_space(24.0);

    render_media_cards_section(ui);
    ui.add_space(24.0);

    render_empty_state_section(ui);
    ui.add_space(32.0);

    action
}

// ── 01. Superfícies & Elevação Neutra ──
fn render_surfaces_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "01. SUPERFÍCIES & ELEVAÇÃO NEUTRA",
        "Camadas tonais puras sem bordas ou strokes artificiais. Elevação por luminância.",
    );

    let swatches = [
        ("BG_BASE", BG_BASE, "Fundo OLED principal (#0B0B0E)"),
        ("SURFACE_LOW", SURFACE_LOW, "Cards e agrupadores (#14151B)"),
        ("SURFACE_MED", SURFACE_MED, "Botões e chips (#1D1E26)"),
        ("SURFACE_HIGH", SURFACE_HIGH, "Hover de elementos (#242631)"),
        (
            "SURFACE_ACTIVE",
            SURFACE_ACTIVE,
            "Estado ativo/clique (#2E303D)",
        ),
        (
            "ACCENT_FILL",
            ACCENT_NEUTRAL_FILL,
            "Ação primária (#EEEEF2)",
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        for (name, color, desc) in swatches {
            let (rect, _) = ui.allocate_exact_size(vec2(150.0, 80.0), Sense::hover());
            ui.painter().rect_filled(rect, RADIUS_MD, color);

            let is_light = color == ACCENT_NEUTRAL_FILL;
            let text_col = if is_light {
                ACCENT_NEUTRAL_TEXT
            } else {
                TEXT_PRIMARY
            };
            let sub_col = if is_light {
                Color32::from_rgb(80, 80, 88)
            } else {
                TEXT_MUTED
            };

            ui.painter().text(
                Pos2::new(rect.left() + 10.0, rect.top() + 18.0),
                Align2::LEFT_CENTER,
                name,
                font_bold(12.0),
                text_col,
            );
            ui.painter().text(
                Pos2::new(rect.left() + 10.0, rect.top() + 38.0),
                Align2::LEFT_CENTER,
                desc,
                font_xs(),
                sub_col,
            );
            ui.add_space(8.0);
        }
    });
}

// ── 02. Tipografia ──
fn render_typography_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "02. TIPOGRAFIA & HIERARQUIA",
        "Família Inter Proporcional e Lucide Icons integrados com contraste neutro limpo.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 95.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_MD, SURFACE_LOW);

    ui.painter().text(
        Pos2::new(rect.left() + 16.0, rect.top() + 20.0),
        Align2::LEFT_CENTER,
        "Título de Seção / Header",
        font_bold(14.0),
        Color32::WHITE,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 16.0, rect.top() + 42.0),
        Align2::LEFT_CENTER,
        "Corpo de texto padrão para nomes de arquivos, navegação e descrições informativas (13px Inter).",
        font_body(),
        TEXT_SECONDARY,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 16.0, rect.top() + 66.0),
        Align2::LEFT_CENTER,
        "TEXTO MUTED / METADADOS: 1.42 GB • 1080P • H.265 • 24:12",
        font_xs(),
        TEXT_MUTED,
    );
}

// ── 03. Botões Full-Rounded ──
fn render_buttons_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "03. BOTÕES FULL-ROUNDED (PILL BUTTONS)",
        "Cantos 100% arredondados (RADIUS_FULL), espaçamento exato de 8px entre ícone e texto e hover suave.",
    );

    ui.horizontal(|ui| {
        let _ = primary(ui, Some(lucide::FOLDER_PLUS), "Ação Primária");
        ui.add_space(8.0);
        let _ = secondary(ui, Some(lucide::FOLDER_OPEN), "Secundário");
        ui.add_space(8.0);
        let _ = secondary(ui, Some(lucide::FLASK_CONICAL), "Testar");
        ui.add_space(8.0);
        let _ = ghost(ui, Some(lucide::SETTINGS), "Ghost");
        ui.add_space(8.0);
        let _ = render_button(
            ui,
            ButtonVariant::Danger,
            Some(lucide::TRASH_2),
            "Excluir",
            34.0,
        );

        ui.add_space(16.0);
        // Botões de ícone circular
        let _ = render_button(ui, ButtonVariant::Secondary, Some(lucide::PLAY), "", 34.0);
        ui.add_space(6.0);
        let _ = render_button(ui, ButtonVariant::Secondary, Some(lucide::PAUSE), "", 34.0);
        ui.add_space(6.0);
        let _ = render_button(
            ui,
            ButtonVariant::Secondary,
            Some(lucide::VOLUME_2),
            "",
            34.0,
        );
        ui.add_space(6.0);
        let _ = render_button(
            ui,
            ButtonVariant::Secondary,
            Some(lucide::MAXIMIZE),
            "",
            34.0,
        );
    });
}

// ── 04. Trilha de Navegação ──
fn render_navigation_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "04. TRILHA DE NAVEGAÇÃO (BREADCRUMBS)",
        "Navegação hierárquica fluida com separadores chevrons Lucide e realce no diretório atual.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_MD, SURFACE_LOW);

    let mut cur_x = rect.left() + 16.0;
    let cy = rect.center().y;

    let items = [
        ("Início", TEXT_SECONDARY),
        ("Filmes e Séries", TEXT_SECONDARY),
        ("Animes 2024", TEXT_SECONDARY),
        ("Cyberpunk Edgerunners", TEXT_PRIMARY),
    ];

    for (idx, (text, color)) in items.iter().enumerate() {
        if idx > 0 {
            ui.painter().text(
                Pos2::new(cur_x, cy),
                Align2::LEFT_CENTER,
                lucide::CHEVRON_RIGHT,
                lucide::font_id(12.0),
                TEXT_MUTED,
            );
            cur_x += 18.0;
        }

        let is_last = idx == items.len() - 1;
        let font = if is_last {
            font_bold(13.0)
        } else {
            font_body()
        };
        let galley = ui
            .painter()
            .layout_no_wrap(text.to_string(), font.clone(), *color);
        let w = galley.size().x;

        ui.painter().text(
            Pos2::new(cur_x, cy),
            Align2::LEFT_CENTER,
            *text,
            font,
            *color,
        );
        cur_x += w + 10.0;
    }
}

// ── 05. Cards de Pasta ──
fn render_folder_cards_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "05. CARDS DE PASTA (SEM WRAPPER NO ÍCONE)",
        "Ícones Lucide renderizados diretamente sobre a superfície elevada, sem molduras quadradas.",
    );

    ui.horizontal(|ui| {
        // Card de Pasta Raiz (Grande)
        let (rect, resp) = ui.allocate_exact_size(vec2(280.0, 105.0), Sense::click());
        let hovered = resp.hovered();
        let bg = if hovered { SURFACE_HIGH } else { SURFACE_LOW };
        ui.painter().rect_filled(rect, RADIUS_MD, bg);

        // Ícone Lucide direto sem caixa
        ui.painter().text(
            Pos2::new(rect.left() + 18.0, rect.top() + 27.0),
            Align2::LEFT_CENTER,
            lucide::FOLDER,
            lucide::font_id(20.0),
            if hovered {
                Color32::WHITE
            } else {
                TEXT_SECONDARY
            },
        );

        // Nome da pasta
        ui.painter().text(
            Pos2::new(rect.left() + 48.0, rect.top() + 20.0),
            Align2::LEFT_CENTER,
            "Animes & Séries",
            font_body(),
            if hovered {
                Color32::WHITE
            } else {
                TEXT_PRIMARY
            },
        );

        // Caminho
        ui.painter().text(
            Pos2::new(rect.left() + 48.0, rect.top() + 37.0),
            Align2::LEFT_CENTER,
            "D:\\Media\\Animes & Séries",
            font_xs(),
            TEXT_MUTED,
        );

        // Itens
        ui.painter().text(
            Pos2::new(rect.left() + 14.0, rect.top() + 78.0),
            Align2::LEFT_CENTER,
            "24 itens",
            font_xs(),
            TEXT_SECONDARY,
        );

        ui.add_space(16.0);

        // Chip de Subpasta (Compacto)
        let (chip_rect, chip_resp) = ui.allocate_exact_size(vec2(230.0, 42.0), Sense::click());
        let chip_hovered = chip_resp.hovered();
        let chip_bg = if chip_hovered {
            SURFACE_HIGH
        } else {
            SURFACE_LOW
        };
        ui.painter().rect_filled(chip_rect, RADIUS_MD, chip_bg);

        ui.painter().text(
            Pos2::new(chip_rect.left() + 14.0, chip_rect.center().y),
            Align2::LEFT_CENTER,
            lucide::FOLDER,
            lucide::font_id(16.0),
            if chip_hovered {
                Color32::WHITE
            } else {
                TEXT_SECONDARY
            },
        );

        ui.painter().text(
            Pos2::new(chip_rect.left() + 38.0, chip_rect.center().y),
            Align2::LEFT_CENTER,
            "Season 01",
            font_body(),
            if chip_hovered {
                Color32::WHITE
            } else {
                TEXT_PRIMARY
            },
        );
    });
}

// ── 06. Cards de Mídia ──
fn render_media_cards_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "06. CARDS DE MÍDIA EXECUTÁVEIS",
        "Thumbnail com cantos arredondados, badge translúcido de formato e play circle com hover neutro.",
    );

    ui.horizontal(|ui| {
        // Card Vídeo
        render_sample_media_card(
            ui,
            "Cyberpunk.Edgerunners.S01E01",
            "MKV",
            "1.42 GB",
            lucide::FILM,
        );
        ui.add_space(16.0);

        // Card Áudio
        render_sample_media_card(
            ui,
            "Radiohead - OK Computer (FLAC)",
            "FLAC",
            "320 MB",
            lucide::MUSIC,
        );
    });
}

fn render_sample_media_card(
    ui: &mut Ui,
    title: &str,
    extension: &str,
    size: &str,
    media_icon: &'static str,
) {
    let (rect, resp) = ui.allocate_exact_size(vec2(210.0, 138.0), Sense::click());
    let hovered = resp.hovered();
    let bg = if hovered { SURFACE_HIGH } else { SURFACE_LOW };
    ui.painter().rect_filled(rect, RADIUS_MD, bg);

    let thumb_h = 82.0;
    let thumb_rect = Rect::from_min_size(rect.min, vec2(210.0, thumb_h));
    ui.painter().rect_filled(
        thumb_rect,
        egui::CornerRadius {
            nw: RADIUS_MD.nw,
            ne: RADIUS_MD.ne,
            sw: 0,
            se: 0,
        },
        SURFACE_MED,
    );

    let center = thumb_rect.center();
    let (circle_bg, icon_str, icon_fg) = if hovered {
        (ACCENT_NEUTRAL_FILL, lucide::PLAY, ACCENT_NEUTRAL_TEXT)
    } else {
        (SURFACE_LOW, media_icon, TEXT_SECONDARY)
    };
    ui.painter().circle_filled(center, 18.0, circle_bg);
    ui.painter().text(
        Pos2::new(center.x + if hovered { 1.0 } else { 0.0 }, center.y),
        Align2::CENTER_CENTER,
        icon_str,
        lucide::font_id(16.0),
        icon_fg,
    );

    // Badge de formato
    let badge_rect = Rect::from_min_size(
        Pos2::new(thumb_rect.right() - 44.0, thumb_rect.top() + 6.0),
        vec2(38.0, 16.0),
    );
    ui.painter()
        .rect_filled(badge_rect, RADIUS_SM, Color32::from_black_alpha(190));
    ui.painter().text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        extension,
        font_xs(),
        TEXT_SECONDARY,
    );

    // Título
    let display_title = if title.len() > 22 {
        format!("{}...", &title[..20])
    } else {
        title.to_string()
    };
    ui.painter().text(
        Pos2::new(rect.left() + 10.0, rect.top() + thumb_h + 16.0),
        Align2::LEFT_CENTER,
        display_title,
        font_body(),
        if hovered {
            TEXT_PRIMARY
        } else {
            TEXT_SECONDARY
        },
    );

    // Tamanho
    ui.painter().text(
        Pos2::new(rect.left() + 10.0, rect.top() + thumb_h + 34.0),
        Align2::LEFT_CENTER,
        size,
        font_xs(),
        TEXT_MUTED,
    );

    resp.on_hover_cursor(CursorIcon::PointingHand);
}

// ── 07. Estado Vazio ──
fn render_empty_state_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "07. ESTADO VAZIO (EMPTY STATE)",
        "Espaçamento confortável, ícone Lucide central e botão de ação neutra proeminente.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 160.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_MD, SURFACE_LOW);

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(Align::Center)),
    );

    let _ = render_empty_state(
        &mut child_ui,
        lucide::FOLDER_OPEN,
        "Nenhum arquivo de mídia encontrado",
        "Arraste vídeos para esta janela ou selecione uma pasta de mídia para começar.",
        Some("+ Selecionar Pasta de Mídia"),
    );
}

fn render_section_header(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(title)
            .font(font_bold(13.0))
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(RichText::new(subtitle).font(font_xs()).color(TEXT_MUTED));
    ui.add_space(8.0);
}
