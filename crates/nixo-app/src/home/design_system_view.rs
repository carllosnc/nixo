//! Visualização completa e interativa do Design System do Nixo com a paleta Tailwind CSS Neutral.

use egui::{
    Align, Align2, Color32, CursorIcon, FontId, Layout, Pos2, Rect, RichText, Sense, Ui, vec2,
};

use super::header::HomeAction;
use crate::components::button::{ButtonVariant, ghost, primary, render_button, secondary};
use crate::components::empty_state::render_empty_state;
use crate::components::tokens::*;
use crate::lucide;

/// Renderiza a página completa do Design System Showcase.
pub fn render_design_system_view(ui: &mut Ui) -> HomeAction {
    let mut action = HomeAction::None;

    ui.horizontal(|ui| {
        if secondary(ui, Some(lucide::ARROW_LEFT), "Voltar para Biblioteca").clicked() {
            action = HomeAction::CloseDesignSystem;
        }
        ui.add_space(14.0);
        ui.label(
            RichText::new("Design System Showcase")
                .font(font_display())
                .color(Color32::WHITE),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("— Paleta Tailwind Neutral & Componentes")
                .font(font_sm())
                .color(TEXT_MUTED),
        );
    });

    ui.add_space(24.0);
    render_surfaces_section(ui);
    ui.add_space(28.0);
    render_typography_section(ui);
    ui.add_space(28.0);
    render_buttons_section(ui);
    ui.add_space(28.0);
    render_navigation_section(ui);
    ui.add_space(28.0);
    render_folder_cards_section(ui);
    ui.add_space(28.0);
    render_media_cards_section(ui);
    ui.add_space(28.0);
    render_empty_state_section(ui);
    ui.add_space(36.0);

    action
}

// ── 01. Paleta Tailwind Neutral & Superfícies ──
fn render_surfaces_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Paleta Tailwind Neutral & Superfícies",
        "Escala oficial Tailwind Neutral (50 a 950) aplicada sem bordas artificiais.",
    );

    let swatches = [
        ("BG_BASE", BG_BASE, "neutral-950 (#0a0a0a)"),
        ("SURFACE_LOW", SURFACE_LOW, "neutral-900 (#171717)"),
        ("SURFACE_MED", SURFACE_MED, "neutral-800 (#262626)"),
        ("SURFACE_HIGH", SURFACE_HIGH, "neutral-700 (#404040)"),
        ("SURFACE_ACTIVE", SURFACE_ACTIVE, "neutral-600 (#525252)"),
        ("ACCENT_FILL", ACCENT_NEUTRAL_FILL, "neutral-100 (#f5f5f5)"),
    ];

    ui.horizontal_wrapped(|ui| {
        for (name, color, desc) in swatches {
            let (rect, resp) = ui.allocate_exact_size(vec2(180.0, 96.0), Sense::hover());
            let ht = ui
                .ctx()
                .animate_bool_with_time(resp.id.with("swatch"), resp.hovered(), 0.15);
            let bg = if color == ACCENT_NEUTRAL_FILL {
                color
            } else {
                lerp_color(color, SURFACE_HIGH, ht)
            };
            ui.painter().rect_filled(rect, RADIUS_MD, bg);

            let (tc, sc) = if color == ACCENT_NEUTRAL_FILL {
                (ACCENT_NEUTRAL_TEXT, Color32::from_rgb(80, 80, 88))
            } else {
                (TEXT_PRIMARY, TEXT_MUTED)
            };

            ui.painter().text(
                Pos2::new(rect.left() + 14.0, rect.top() + 24.0),
                Align2::LEFT_CENTER,
                name,
                font_card_title(),
                tc,
            );
            ui.painter().text(
                Pos2::new(rect.left() + 14.0, rect.top() + 54.0),
                Align2::LEFT_CENTER,
                desc,
                font_xs(),
                sc,
            );
            ui.add_space(10.0);
        }
    });

    ui.add_space(12.0);

    #[rustfmt::skip]
    let ramp = [
        ("950", NEUTRAL_950, NEUTRAL_400), ("900", NEUTRAL_900, NEUTRAL_400),
        ("800", NEUTRAL_800, NEUTRAL_400), ("700", NEUTRAL_700, NEUTRAL_300),
        ("600", NEUTRAL_600, NEUTRAL_200), ("500", NEUTRAL_500, NEUTRAL_100),
        ("400", NEUTRAL_400, NEUTRAL_950), ("300", NEUTRAL_300, NEUTRAL_950),
        ("200", NEUTRAL_200, NEUTRAL_950), ("100", NEUTRAL_100, NEUTRAL_950),
        ("50", NEUTRAL_50, NEUTRAL_950),
    ];

    ui.horizontal(|ui| {
        let block_w = ((ui.available_width() - 10.0 * 6.0) / 11.0).clamp(40.0, 75.0);
        for (step, color, fg) in ramp {
            let (rect, _) = ui.allocate_exact_size(vec2(block_w, 34.0), Sense::hover());
            ui.painter().rect_filled(rect, RADIUS_SM, color);
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, step, font_xs(), fg);
            ui.add_space(6.0);
        }
    });
}

// ── 02. Tipografia & Hierarquia ──
fn render_typography_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Tipografia & Hierarquia Visual",
        "Escala proporcional Inter com pesos dedicados e contraste Tailwind Neutral.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 206.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_MD, SURFACE_LOW);

    #[rustfmt::skip]
    let rows: [(&str, &str, FontId, Color32, &str); 6] = [
        ("Display", "28.0px Bold", font_display(), Color32::WHITE, "Cabeçalhos de tela e títulos principais"),
        ("Seção", "21.0px Bold", font_section_title(), Color32::WHITE, "Títulos de blocos em Title Case"),
        ("Card / Item", "17.0px Bold", font_card_title(), TEXT_PRIMARY, "Nomes de pastas, arquivos e módulos"),
        ("Corpo", "15.5px Regular", font_body(), TEXT_PRIMARY, "Botões, inputs e navegação ativa"),
        ("Subtítulo", "14.0px Regular", font_sm(), TEXT_SECONDARY, "Descrições de apoio e contadores"),
        ("Metadados", "12.5px Regular", font_xs(), TEXT_MUTED, "Extensões, badges e detalhes: 1080P • MKV • 24:12"),
    ];

    for (idx, (role, spec, f, c, s)) in rows.iter().enumerate() {
        let y = rect.top() + 18.0 + (idx as f32) * 31.0;
        ui.painter().text(
            Pos2::new(rect.left() + 20.0, y),
            Align2::LEFT_CENTER,
            format!("{role} ({spec}):"),
            font_xs(),
            TEXT_MUTED,
        );
        ui.painter().text(
            Pos2::new(rect.left() + 240.0, y),
            Align2::LEFT_CENTER,
            *s,
            f.clone(),
            *c,
        );
    }
}

// ── 03. Botões Full-Rounded ──
fn render_buttons_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Botões Full-Rounded (Pill Buttons)",
        "Cantos 100% arredondados (RADIUS_FULL), espaçamento exato de 8px e transição suave de 150ms.",
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
            38.0,
        );

        ui.add_space(16.0);
        for icon in [
            lucide::PLAY,
            lucide::PAUSE,
            lucide::VOLUME_2,
            lucide::MAXIMIZE,
        ] {
            let _ = render_button(ui, ButtonVariant::Secondary, Some(icon), "", 38.0);
            ui.add_space(6.0);
        }
    });
}

// ── 04. Trilha de Navegação ──
fn render_navigation_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Trilha de Navegação (Breadcrumbs)",
        "Navegação hierárquica fluida com separadores chevrons Lucide e realce no diretório atual.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIUS_MD, SURFACE_LOW);

    let mut cur_x = rect.left() + 18.0;
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
                lucide::font_id(14.0),
                TEXT_MUTED,
            );
            cur_x += 22.0;
        }

        let font = if idx == items.len() - 1 {
            font_card_title()
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
        cur_x += w + 12.0;
    }
}

// ── 05. Cards de Pasta ──
fn render_folder_cards_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Cards de Pasta (Com Transição no Hover)",
        "Transição fluida de 150ms de neutral-900 para neutral-700 e realce suave dos ícones.",
    );

    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(vec2(310.0, 120.0), Sense::click());
        let ht = ui
            .ctx()
            .animate_bool_with_time(resp.id.with("demo_folder"), resp.hovered(), 0.15);
        let bg = lerp_color(SURFACE_LOW, SURFACE_HIGH, ht);
        ui.painter().rect_filled(rect, RADIUS_MD, bg);

        let ic_col = lerp_color(TEXT_SECONDARY, Color32::WHITE, ht);
        ui.painter().text(
            Pos2::new(rect.left() + 20.0, rect.top() + 32.0),
            Align2::LEFT_CENTER,
            lucide::FOLDER,
            lucide::font_id(26.0),
            ic_col,
        );

        let tc = lerp_color(TEXT_PRIMARY, Color32::WHITE, ht);
        ui.painter().text(
            Pos2::new(rect.left() + 56.0, rect.top() + 23.0),
            Align2::LEFT_CENTER,
            "Animes & Séries",
            font_card_title(),
            tc,
        );
        ui.painter().text(
            Pos2::new(rect.left() + 56.0, rect.top() + 44.0),
            Align2::LEFT_CENTER,
            "D:\\Media\\Animes & Séries",
            font_xs(),
            TEXT_MUTED,
        );
        ui.painter().text(
            Pos2::new(rect.left() + 20.0, rect.top() + 92.0),
            Align2::LEFT_CENTER,
            "24 itens",
            font_sm(),
            TEXT_SECONDARY,
        );

        ui.add_space(16.0);

        let (cr, cresp) = ui.allocate_exact_size(vec2(250.0, 50.0), Sense::click());
        let cht =
            ui.ctx()
                .animate_bool_with_time(cresp.id.with("demo_chip"), cresp.hovered(), 0.15);
        let cbg = lerp_color(SURFACE_LOW, SURFACE_HIGH, cht);
        ui.painter().rect_filled(cr, RADIUS_MD, cbg);
        let cic = lerp_color(TEXT_SECONDARY, Color32::WHITE, cht);
        ui.painter().text(
            Pos2::new(cr.left() + 16.0, cr.center().y),
            Align2::LEFT_CENTER,
            lucide::FOLDER,
            lucide::font_id(20.0),
            cic,
        );
        let ctc = lerp_color(TEXT_PRIMARY, Color32::WHITE, cht);
        ui.painter().text(
            Pos2::new(cr.left() + 46.0, cr.center().y),
            Align2::LEFT_CENTER,
            "Season 01",
            font_body(),
            ctc,
        );
    });
}

// ── 06. Cards de Mídia ──
fn render_media_cards_section(ui: &mut Ui) {
    render_section_header(
        ui,
        "Cards de Mídia (Com Transição no Hover)",
        "Thumbnail com transição suave, play circle dinâmico e elevação de luminância.",
    );

    ui.horizontal(|ui| {
        render_sample_media_card(
            ui,
            "Cyberpunk.Edgerunners.S01E01",
            "MKV",
            "1.42 GB",
            lucide::FILM,
        );
        ui.add_space(16.0);
        render_sample_media_card(
            ui,
            "Radiohead - OK Computer",
            "FLAC",
            "320 MB",
            lucide::MUSIC,
        );
    });
}

fn render_sample_media_card(ui: &mut Ui, title: &str, ext: &str, size: &str, icon: &'static str) {
    let (rect, resp) = ui.allocate_exact_size(vec2(230.0, 160.0), Sense::click());
    let ht = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("demo_media"), resp.hovered(), 0.15);
    let bg = lerp_color(SURFACE_LOW, SURFACE_HIGH, ht);
    ui.painter().rect_filled(rect, RADIUS_MD, bg);

    let thumb_h = 96.0;
    let thumb_rect = Rect::from_min_size(rect.min, vec2(230.0, thumb_h));
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
    let cbg = lerp_color(SURFACE_LOW, ACCENT_NEUTRAL_FILL, ht);
    let istr = if ht > 0.4 { lucide::PLAY } else { icon };
    let ifg = lerp_color(TEXT_SECONDARY, ACCENT_NEUTRAL_TEXT, ht);
    ui.painter().circle_filled(center, 22.0, cbg);
    ui.painter().text(
        Pos2::new(center.x + if ht > 0.4 { 1.0 } else { 0.0 }, center.y),
        Align2::CENTER_CENTER,
        istr,
        lucide::font_id(18.0),
        ifg,
    );

    let brect = Rect::from_min_size(
        Pos2::new(thumb_rect.right() - 48.0, thumb_rect.top() + 8.0),
        vec2(40.0, 18.0),
    );
    ui.painter()
        .rect_filled(brect, RADIUS_SM, Color32::from_black_alpha(190));
    ui.painter().text(
        brect.center(),
        Align2::CENTER_CENTER,
        ext,
        font_xs(),
        TEXT_SECONDARY,
    );

    let name = if title.len() > 24 {
        format!("{}...", &title[..22])
    } else {
        title.to_string()
    };
    let ncol = lerp_color(TEXT_SECONDARY, TEXT_PRIMARY, ht);
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.top() + thumb_h + 18.0),
        Align2::LEFT_CENTER,
        name,
        font_body(),
        ncol,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.top() + thumb_h + 38.0),
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
        "Estado Vazio (Empty State)",
        "Espaçamento confortável, ícone Lucide central e botão de ação neutra proeminente.",
    );

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 170.0), Sense::hover());
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
            .font(font_section_title())
            .color(Color32::WHITE),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(subtitle)
            .font(font_sm())
            .color(TEXT_SECONDARY),
    );
    ui.add_space(14.0);
}
