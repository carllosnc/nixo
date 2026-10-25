//! Modelo de domínio e processamento de ajustes de imagem (Equalizador de Vídeo).
//!
//! Executa transformações de cor de alta performance diretamente no espaço de cores RGBA,
//! eliminando qualquer latência de reconstrução de filtros de vídeo (latência zero / 60 FPS).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct VideoEqualizer {
    /// Ajuste de brilho (-100 a 100, padrão: 0).
    pub brightness: i32,
    /// Ajuste de contraste (-100 a 100, padrão: 0).
    pub contrast: i32,
    /// Ajuste de saturação (-100 a 100, padrão: 0).
    pub saturation: i32,
    /// Ajuste de gama (-100 a 100, padrão: 0).
    pub gamma: i32,
}

impl VideoEqualizer {
    #[inline]
    pub fn is_default(&self) -> bool {
        self.brightness == 0 && self.contrast == 0 && self.saturation == 0 && self.gamma == 0
    }

    /// Aplica brilho, contraste, saturação e gama diretamente no buffer RGBA de 32 bits.
    /// Utiliza ponto fixo inteiro e LUT em cache para latência zero e 60 FPS fluídos.
    pub fn apply_to_rgba(&self, rgba: &mut [u8]) {
        if self.is_default() {
            return;
        }

        let b = (self.brightness as f32 / 100.0) * 255.0;
        let c = (1.0 + self.contrast as f32 / 100.0).max(0.0);
        let s = (1.0 + self.saturation as f32 / 100.0).max(0.0);
        let g = (1.0 + self.gamma as f32 / 100.0).clamp(0.1, 10.0);

        let mut gamma_lut = [0u8; 256];
        if (g - 1.0).abs() > 0.001 {
            let inv_g = 1.0 / g;
            for (i, val) in gamma_lut.iter_mut().enumerate() {
                let norm = (i as f32) / 255.0;
                *val = (255.0 * norm.powf(inv_g)).clamp(0.0, 255.0).round() as u8;
            }
        } else {
            for (i, val) in gamma_lut.iter_mut().enumerate() {
                *val = i as u8;
            }
        }

        // Caminho otimizado por LUT único de 256 bytes se a saturação for neutra
        if (s - 1.0).abs() <= 0.001 {
            let mut single_lut = [0u8; 256];
            for (i, val) in single_lut.iter_mut().enumerate() {
                let val_f = ((i as f32 - 128.0) * c + 128.0 + b).clamp(0.0, 255.0);
                *val = gamma_lut[val_f.round() as usize];
            }

            for chunk in rgba.as_chunks_mut::<4>().0 {
                chunk[0] = single_lut[chunk[0] as usize];
                chunk[1] = single_lut[chunk[1] as usize];
                chunk[2] = single_lut[chunk[2] as usize];
            }
            return;
        }

        // Matriz de cor RGB com saturação em ponto fixo 10 bits (x1024)
        let m_r = (c * ((1.0 - s) * 0.299 + s) * 1024.0).round() as i32;
        let m_g = (c * ((1.0 - s) * 0.587) * 1024.0).round() as i32;
        let m_b = (c * ((1.0 - s) * 0.114) * 1024.0).round() as i32;

        let g_r = (c * ((1.0 - s) * 0.299) * 1024.0).round() as i32;
        let g_g = (c * ((1.0 - s) * 0.587 + s) * 1024.0).round() as i32;
        let g_b = (c * ((1.0 - s) * 0.114) * 1024.0).round() as i32;

        let b_r = (c * ((1.0 - s) * 0.299) * 1024.0).round() as i32;
        let b_g = (c * ((1.0 - s) * 0.587) * 1024.0).round() as i32;
        let b_b = (c * ((1.0 - s) * 0.114 + s) * 1024.0).round() as i32;

        let offset = ((128.0 * (1.0 - c) + b) * 1024.0).round() as i32;

        for chunk in rgba.as_chunks_mut::<4>().0 {
            let r = chunk[0] as i32;
            let g = chunk[1] as i32;
            let b = chunk[2] as i32;

            let nr = ((m_r * r + m_g * g + m_b * b + offset) >> 10).clamp(0, 255) as usize;
            let ng = ((g_r * r + g_g * g + g_b * b + offset) >> 10).clamp(0, 255) as usize;
            let nb = ((b_r * r + b_g * g + b_b * b + offset) >> 10).clamp(0, 255) as usize;

            chunk[0] = gamma_lut[nr];
            chunk[1] = gamma_lut[ng];
            chunk[2] = gamma_lut[nb];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equalizer_default_is_noop() {
        let eq = VideoEqualizer::default();
        assert!(eq.is_default());
        let mut pixels = [100u8, 150, 200, 255];
        eq.apply_to_rgba(&mut pixels);
        assert_eq!(pixels, [100, 150, 200, 255]);
    }

    #[test]
    fn test_equalizer_applies_modifications() {
        let eq = VideoEqualizer {
            brightness: 20,
            contrast: 10,
            saturation: -100, // Grayscale
            gamma: 0,
        };
        let mut pixels = [200u8, 50, 50, 255];
        eq.apply_to_rgba(&mut pixels);
        // Em saturação -100 (tons de cinza), R, G e B devem ser quase idênticos
        assert!((pixels[0] as i32 - pixels[1] as i32).abs() <= 2);
        assert!((pixels[1] as i32 - pixels[2] as i32).abs() <= 2);
    }
}
