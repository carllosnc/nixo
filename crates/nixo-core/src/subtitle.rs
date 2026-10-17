use serde::{Deserialize, Serialize};

/// Diagnóstico de saúde e compatibilidade da faixa de legenda atualmente selecionada.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SubtitleHealth {
    /// Legenda de texto operando normalmente (ou legendas desativadas).
    #[default]
    Ok,
    /// Faixa de legenda baseada em imagem/bitmap (ex.: DVD VobSub `dvd_subtitle`, Blu-ray PGS `hdmv_pgs_subtitle`).
    /// Por ser pré-renderizada em imagem, não aceita customização de fonte, peso ou cor.
    ImageBased { codec: String },
    /// Faixa selecionada sem emissão de texto decodificado (faixa vazia/Signs, corrompida ou hardsub no vídeo).
    InactiveTrack,
}

impl SubtitleHealth {
    pub fn is_warning(&self) -> bool {
        !matches!(self, Self::Ok)
    }

    pub fn badge_label(&self) -> Option<&'static str> {
        match self {
            Self::Ok => None,
            Self::ImageBased { .. } => Some("LEGENDA FIXA"),
            Self::InactiveTrack => Some("LEGENDA INATIVA"),
        }
    }

    pub fn warning_message(&self) -> Option<String> {
        match self {
            Self::Ok => None,
            Self::ImageBased { codec } => Some(format!(
                "Legenda em imagem ({codec}): não permite alterar fonte ou cor"
            )),
            Self::InactiveTrack => Some(
                "A legenda selecionada não está emitindo texto (faixa vazia ou hardsub no vídeo)"
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleBorderStyle {
    None,
    Thin,
    #[default]
    Medium,
    Thick,
}

impl SubtitleBorderStyle {
    pub const ALL: [Self; 4] = [Self::None, Self::Thin, Self::Medium, Self::Thick];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Nenhum",
            Self::Thin => "Fino",
            Self::Medium => "Médio",
            Self::Thick => "Grosso",
        }
    }

    pub fn border_size_px(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Thin => 1.2,
            Self::Medium => 2.5,
            Self::Thick => 4.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleShadowStyle {
    None,
    #[default]
    Light,
    Strong,
}

impl SubtitleShadowStyle {
    pub const ALL: [Self; 3] = [Self::None, Self::Light, Self::Strong];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Nenhuma",
            Self::Light => "Leve",
            Self::Strong => "Forte",
        }
    }

    pub fn offset_px(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Light => 2.0,
            Self::Strong => 4.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleBackgroundStyle {
    #[default]
    None,
    Box,
    Strip,
}

impl SubtitleBackgroundStyle {
    pub const ALL: [Self; 3] = [Self::None, Self::Box, Self::Strip];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Nenhum",
            Self::Box => "Caixa",
            Self::Strip => "Faixa",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubtitleStyle {
    pub font_family: String,
    /// Escala percentual da fonte (50..=300, padrão 100).
    pub scale_percent: u16,
    /// Cor RGBA do texto da legenda.
    pub text_color: [u8; 4],
    pub bold: bool,
    pub border: SubtitleBorderStyle,
    pub shadow: SubtitleShadowStyle,
    pub background: SubtitleBackgroundStyle,
    /// Opacidade do fundo em porcentagem (10..=100, padrão 60).
    pub background_opacity: u8,
    /// Margem vertical a partir da base em porcentagem (0..=100, padrão 8 -> sub-pos = 92).
    pub bottom_offset_percent: u8,
    /// Se verdadeiro, sobrescreve estilos embutidos em legendas ASS/SSA (`sub-ass-override=strip`).
    pub override_ass: bool,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            scale_percent: 100,
            text_color: [255, 255, 255, 255],
            bold: false,
            border: SubtitleBorderStyle::Medium,
            shadow: SubtitleShadowStyle::Light,
            background: SubtitleBackgroundStyle::None,
            background_opacity: 60,
            bottom_offset_percent: 8,
            override_ass: true,
        }
    }
}

impl SubtitleStyle {
    pub const FONT_CHOICES: [&'static str; 6] = [
        "Inter",
        "Oswald",
        "Montserrat",
        "Roboto",
        "Open Sans",
        "Lato",
    ];

    pub const COLOR_SWATCHES: [(&'static str, [u8; 4]); 6] = [
        ("Branco", [255, 255, 255, 255]),
        ("Amarelo", [255, 225, 77, 255]),
        ("Verde", [125, 255, 92, 255]),
        ("Ciano", [77, 232, 255, 255]),
        ("Magenta", [255, 92, 240, 255]),
        ("Vermelho", [255, 85, 85, 255]),
    ];

    pub fn preset_default() -> Self {
        Self::default()
    }

    pub fn preset_cinema() -> Self {
        Self {
            font_family: "Montserrat".to_string(),
            scale_percent: 110,
            border: SubtitleBorderStyle::Thin,
            shadow: SubtitleShadowStyle::Strong,
            ..Self::default()
        }
    }

    pub fn preset_high_contrast() -> Self {
        Self {
            font_family: "Roboto".to_string(),
            bold: true,
            text_color: [255, 225, 77, 255],
            border: SubtitleBorderStyle::None,
            shadow: SubtitleShadowStyle::None,
            background: SubtitleBackgroundStyle::Box,
            background_opacity: 100,
            ..Self::default()
        }
    }

    pub fn preset_classic_yellow() -> Self {
        Self {
            text_color: [255, 225, 77, 255],
            border: SubtitleBorderStyle::Thick,
            shadow: SubtitleShadowStyle::None,
            ..Self::default()
        }
    }

    pub fn sanitized(mut self) -> Self {
        if !Self::FONT_CHOICES.contains(&self.font_family.as_str()) {
            self.font_family = Self::default().font_family;
        }
        self.scale_percent = self.scale_percent.clamp(50, 300);
        self.background_opacity = self.background_opacity.clamp(10, 100);
        self.bottom_offset_percent = self.bottom_offset_percent.clamp(0, 90);
        self.text_color[3] = 255;
        self
    }

    /// Compara todos os atributos visuais exceto `override_ass` (para destacar presets).
    pub fn matches_preset(&self, preset: &Self) -> bool {
        let mut a = self.clone();
        let mut b = preset.clone();
        a.override_ass = false;
        b.override_ass = false;
        a == b
    }
}
