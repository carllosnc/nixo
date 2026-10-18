//! Design tokens do Nixo Neutral Design System baseados na paleta Tailwind CSS Neutral.
//!
//! Todas as cores são neutras puras (Tailwind Neutral 50 a 950), eliminando bordas duras e criando
//! profundidade através de camadas de elevação sutis (estilo Linear, shadcn e macOS Dark).

use std::sync::Arc;

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Align2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Ui};

// ── Paleta Tailwind CSS Neutral Oficial ──
/// Tailwind neutral-50 (#fafafa) - Destaque máximo / hover primário.
pub const NEUTRAL_50: Color32 = Color32::from_rgb(250, 250, 250);

/// Tailwind neutral-100 (#f5f5f5) - Ação primária e texto de alto contraste.
pub const NEUTRAL_100: Color32 = Color32::from_rgb(245, 245, 245);

/// Tailwind neutral-200 (#e5e5e5) - Ícones e realces claros.
pub const NEUTRAL_200: Color32 = Color32::from_rgb(229, 229, 229);

/// Tailwind neutral-300 (#d4d4d4) - Texto secundário em botões ghost.
pub const NEUTRAL_300: Color32 = Color32::from_rgb(212, 212, 212);

/// Tailwind neutral-400 (#a3a3a3) - Texto secundário, subtítulos e breadcrumbs.
pub const NEUTRAL_400: Color32 = Color32::from_rgb(163, 163, 163);

/// Tailwind neutral-500 (#737373) - Texto muted, separadores e metadados.
pub const NEUTRAL_500: Color32 = Color32::from_rgb(115, 115, 115);

/// Tailwind neutral-600 (#525252) - Superfície ativa / pressionada.
pub const NEUTRAL_600: Color32 = Color32::from_rgb(82, 82, 82);

/// Tailwind neutral-700 (#404040) - Superfície hover de botões e cards.
pub const NEUTRAL_700: Color32 = Color32::from_rgb(64, 64, 64);

/// Tailwind neutral-800 (#262626) - Botões secundários, chips e caixas de mídia.
pub const NEUTRAL_800: Color32 = Color32::from_rgb(38, 38, 38);

/// Tailwind neutral-900 (#171717) - Cards inativos e agrupadores principais.
pub const NEUTRAL_900: Color32 = Color32::from_rgb(23, 23, 23);

/// Tailwind neutral-950 (#0a0a0a) - Fundo base OLED da aplicação.
pub const NEUTRAL_950: Color32 = Color32::from_rgb(10, 10, 10);

// ── Camadas Semânticas de Elevação (Superfícies) ──
/// Fundo principal OLED da aplicação (Tailwind neutral-950).
pub const BG_BASE: Color32 = NEUTRAL_950;

/// Superfície 1: Cards inativos, agrupadores, barras e cabeçalhos (Tailwind neutral-900).
pub const SURFACE_LOW: Color32 = NEUTRAL_900;

/// Superfície 2: Botões secundários, chips de pasta e inputs (Tailwind neutral-800).
pub const SURFACE_MED: Color32 = NEUTRAL_800;

/// Superfície 3: Estado hover de botões, chips e cartões (Tailwind neutral-700).
pub const SURFACE_HIGH: Color32 = NEUTRAL_700;

/// Superfície Ativa: Estado pressionado / selecionado (Tailwind neutral-600).
pub const SURFACE_ACTIVE: Color32 = NEUTRAL_600;

// ── Destaques Neutros para Ações Primárias ──
/// Fundo do botão de ação primária (Tailwind neutral-100).
pub const ACCENT_NEUTRAL_FILL: Color32 = NEUTRAL_100;
/// Hover do botão de ação primária (Tailwind neutral-50).
pub const ACCENT_NEUTRAL_HOVER: Color32 = NEUTRAL_50;
/// Texto escuro sobre o botão primário neutro (Tailwind neutral-950).
pub const ACCENT_NEUTRAL_TEXT: Color32 = NEUTRAL_950;

// ── Tipografia & Cores de Texto ──
/// Texto Primário: Títulos principais, itens ativos e texto de alto contraste (Tailwind neutral-100).
pub const TEXT_PRIMARY: Color32 = NEUTRAL_100;

/// Texto Secundário: Subtítulos, metadados legíveis e breadcrumbs pais (Tailwind neutral-400).
pub const TEXT_SECONDARY: Color32 = NEUTRAL_400;

/// Texto Muted: Contadores secundários, separadores e extensões (Tailwind neutral-500).
pub const TEXT_MUTED: Color32 = NEUTRAL_500;

// ── Cores para Ações Destrutivas Sutis ──
pub const DANGER_SURFACE: Color32 = Color32::from_rgb(43, 20, 22);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(61, 26, 30);
pub const DANGER_TEXT: Color32 = Color32::from_rgb(248, 113, 113); // Tailwind red-400

// ── Raios de Curvatura (Corner Radii) ──
pub const RADIUS_SM: CornerRadius = CornerRadius::same(6);
pub const RADIUS_MD: CornerRadius = CornerRadius::same(9);
pub const RADIUS_LG: CornerRadius = CornerRadius::same(14);
pub const RADIUS_FULL: CornerRadius = CornerRadius::same(99);

// ── Escala Tipográfica Padronizada (Hierarquia & Legibilidade Desktop) ──
/// Título Principal de Tela / Ecrã (ex.: cabeçalho do showcase ou da biblioteca).
#[inline]
pub fn font_display() -> FontId {
    FontId::new(28.0, egui::FontFamily::Name("bold".into()))
}

/// Título de Seção (hierarquia principal, renderizado em Title Case e branco).
#[inline]
pub fn font_section_title() -> FontId {
    FontId::new(21.0, egui::FontFamily::Name("bold".into()))
}

/// Título de Card / Item em grid ou lista.
#[inline]
pub fn font_card_title() -> FontId {
    FontId::new(17.0, egui::FontFamily::Name("bold".into()))
}

/// Título padrão para compatibilidade de componentes existentes.
#[inline]
pub fn font_title() -> FontId {
    FontId::new(18.5, egui::FontFamily::Name("bold".into()))
}

/// Corpo de texto padrão para nomes, navegação, inputs e botões.
#[inline]
pub fn font_body() -> FontId {
    FontId::proportional(15.5)
}

/// Subtítulo e texto descritivo secundário (confortável e legível).
#[inline]
pub fn font_sm() -> FontId {
    FontId::proportional(14.0)
}

/// Metadados, badges, contadores e timestamps (tamanho mínimo para nitidez).
#[inline]
pub fn font_xs() -> FontId {
    FontId::proportional(12.5)
}

/// Interpola linearmente entre duas cores (`Color32`) com base em um fator `t` de 0.0 a 1.0 para transições suaves de hover.
#[inline]
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t).round() as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t).round() as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t).round() as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t).round() as u8,
    )
}

/// Faz o layout de uma linha única de texto, truncando com `…` pela largura máxima em pixels.
pub fn layout_single_line(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), TextFormat::simple(font, color));
    job.wrap = TextWrapping {
        max_width: max_width.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.fonts(|f| f.layout_job(job))
}

/// Desenha uma linha única de texto truncada com `…` dentro da largura máxima informada.
pub fn paint_truncated_text(
    ui: &Ui,
    anchor: Pos2,
    align: Align2,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Rect {
    let galley = layout_single_line(ui, text, font, color, max_width);
    let rect = align.anchor_size(anchor, galley.size());
    ui.painter().galley(rect.min, galley, Color32::PLACEHOLDER);
    rect
}
