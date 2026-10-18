//! Design tokens do Nixo Neutral Design System.
//!
//! Todas as cores são neutras, eliminando bordas duras e criando
//! profundidade através de camadas de elevação sutis (estilo Linear e macOS Dark).

use egui::{Color32, CornerRadius, FontId};

// ── Camadas de Elevação (Superfícies) ──
/// Fundo principal OLED da aplicação.
pub const BG_BASE: Color32 = Color32::from_rgb(11, 11, 14);

/// Superfície 1: Cards inativos, agrupadores, barras e cabeçalhos.
pub const SURFACE_LOW: Color32 = Color32::from_rgb(20, 21, 27);

/// Superfície 2: Botões secundários, chips de pasta e inputs.
pub const SURFACE_MED: Color32 = Color32::from_rgb(29, 30, 38);

/// Superfície 3: Estado hover de botões, chips e cartões.
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(36, 38, 49);

/// Superfície Ativa: Estado pressionado / selecionado.
pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(46, 48, 61);

// ── Destaques Neutros para Ações Primárias ──
/// Fundo do botão de ação primária (alto contraste, limpo).
pub const ACCENT_NEUTRAL_FILL: Color32 = Color32::from_rgb(238, 238, 242);
/// Hover do botão de ação primária.
pub const ACCENT_NEUTRAL_HOVER: Color32 = Color32::WHITE;
/// Texto escuro sobre o botão primário neutro.
pub const ACCENT_NEUTRAL_TEXT: Color32 = Color32::from_rgb(18, 18, 21);

// ── Tipografia & Cores de Texto ──
/// Texto Primário: Títulos principais, itens ativos e texto de alto contraste.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 245, 248);

/// Texto Secundário: Subtítulos, metadados legíveis e breadcrumbs pais.
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(170, 172, 185);

/// Texto Muted: Contadores secundários, separadores e extensões.
pub const TEXT_MUTED: Color32 = Color32::from_rgb(122, 125, 140);

// ── Cores para Ações Destrutivas Sutis ──
pub const DANGER_SURFACE: Color32 = Color32::from_rgb(38, 21, 23);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(56, 27, 31);
pub const DANGER_TEXT: Color32 = Color32::from_rgb(255, 142, 148);

// ── Raios de Curvatura (Corner Radii) ──
pub const RADIUS_SM: CornerRadius = CornerRadius::same(6);
pub const RADIUS_MD: CornerRadius = CornerRadius::same(9);
pub const RADIUS_LG: CornerRadius = CornerRadius::same(14);
pub const RADIUS_FULL: CornerRadius = CornerRadius::same(99);

// ── Escala Tipográfica Padronizada (Hierarquia & Legibilidade) ──
/// Título Principal de Tela / Ecrã (ex.: cabeçalho do showcase ou da biblioteca).
#[inline]
pub fn font_display() -> FontId {
    FontId::new(22.0, egui::FontFamily::Name("bold".into()))
}

/// Título de Seção (hierarquia principal, renderizado em Title Case e branco).
#[inline]
pub fn font_section_title() -> FontId {
    FontId::new(16.5, egui::FontFamily::Name("bold".into()))
}

/// Título de Card / Item em grid ou lista.
#[inline]
pub fn font_card_title() -> FontId {
    FontId::new(14.5, egui::FontFamily::Name("bold".into()))
}

/// Título padrão para compatibilidade de componentes existentes.
#[inline]
pub fn font_title() -> FontId {
    FontId::new(15.5, egui::FontFamily::Name("bold".into()))
}

/// Corpo de texto padrão para nomes, navegação, inputs e botões.
#[inline]
pub fn font_body() -> FontId {
    FontId::proportional(13.5)
}

/// Subtítulo e texto descritivo secundário (confortável e legível).
#[inline]
pub fn font_sm() -> FontId {
    FontId::proportional(12.5)
}

/// Metadados, badges, contadores e timestamps (tamanho mínimo para nitidez).
#[inline]
pub fn font_xs() -> FontId {
    FontId::proportional(11.5)
}
