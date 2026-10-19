//! Cache persistente em disco para miniaturas e metadados de vídeo/áudio.
//!
//! Armazena quadros RGBA pré-renderizados e metadados em formato binário compacto
//! no diretório de cache do sistema operacional, permitindo carregamento em 0ms.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use nixo_core::VideoFrame;

use super::thumbnail::ThumbnailResult;

/// Identifica se o caminho possui extensão comum de arquivo de áudio exclusivo.
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp3" | "flac" | "wav" | "m4a" | "aac" | "opus" | "ogg" | "wma" | "alac" | "aiff"
            )
        })
}

/// Avalia se um buffer RGBA é predominantemente escuro/preto (ex.: vinhetas, fade-ins).
pub fn is_frame_mostly_black(rgba: &[u8]) -> bool {
    let total_pixels = rgba.len() / 4;
    if total_pixels == 0 {
        return true;
    }
    let mut dark_count = 0usize;
    for px in rgba.as_chunks::<4>().0 {
        let r = px[0] as u32;
        let g = px[1] as u32;
        let b = px[2] as u32;
        let lum = (r * 299 + g * 587 + b * 114) / 1000;
        if lum < 22 {
            dark_count += 1;
        }
    }
    dark_count * 100 / total_pixels >= 88
}

/// Calcula ponto temporal inicial para captura evitando aberturas e intros.
pub fn compute_target_sec(dur: f64) -> f64 {
    if dur >= 180.0 {
        (dur * 0.25).clamp(130.0, (dur - 30.0).max(130.0))
    } else if dur >= 135.0 {
        125.0
    } else if dur >= 60.0 {
        (dur * 0.50).clamp(25.0, (dur - 10.0).max(25.0))
    } else if dur >= 15.0 {
        (dur * 0.40).clamp(5.0, (dur - 3.0).max(5.0))
    } else if dur > 0.0 {
        (dur * 0.5).max(1.5)
    } else {
        130.0
    }
}

/// Retorna o diretório de cache persistente padrão para as miniaturas do Nixo.
pub fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nixo")
        .join("thumbnails")
}

/// Calcula o caminho do arquivo de cache persistente baseado em path, tamanho e mtime.
pub fn get_cache_path(cache_dir: &Path, path: &Path) -> Option<PathBuf> {
    let meta = std::fs::metadata(path).ok()?;
    let len = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    let mut hasher = DefaultHasher::new();
    // Sal de versão para isolar e invalidar caches de versões antigas com repetição de cenas
    b"nixo_thumb_v2".hash(&mut hasher);
    path.hash(&mut hasher);
    len.hash(&mut hasher);
    mtime.hash(&mut hasher);
    let hash = hasher.finish();
    Some(cache_dir.join(format!("{hash:016x}.thumb")))
}

/// Carrega miniatura e metadados persistidos em disco em frações de milissegundo.
pub fn load_from_cache(cache_path: &Path, path: &Path) -> Option<ThumbnailResult> {
    let bytes = std::fs::read(cache_path).ok()?;
    if bytes.len() < 24 {
        return None;
    }
    let w = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let h = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let dur_f64 = f64::from_le_bytes(bytes[8..16].try_into().ok()?);
    let orig_w = u32::from_le_bytes(bytes[16..20].try_into().ok()?);
    let orig_h = u32::from_le_bytes(bytes[20..24].try_into().ok()?);
    let expected_rgba = (w as usize) * (h as usize) * 4;
    let frames = if w > 0 && h > 0 && bytes.len() == 24 + expected_rgba {
        vec![VideoFrame {
            width: w,
            height: h,
            rgba: bytes[24..].to_vec(),
            frame_id: 1,
        }]
    } else {
        Vec::new()
    };
    Some(ThumbnailResult {
        path: path.to_path_buf(),
        frames,
        duration_secs: if dur_f64 > 0.0 { Some(dur_f64) } else { None },
        width: if orig_w > 0 { Some(orig_w) } else { None },
        height: if orig_h > 0 { Some(orig_h) } else { None },
    })
}

/// Salva miniatura gerada e metadados no cache em disco para reuso instantâneo.
pub fn save_to_cache(cache_path: &Path, res: &ThumbnailResult) {
    let mut data = Vec::with_capacity(24 + 144 * 81 * 4);
    if let Some(frame) = res.frames.first() {
        data.extend_from_slice(&frame.width.to_le_bytes());
        data.extend_from_slice(&frame.height.to_le_bytes());
        data.extend_from_slice(&res.duration_secs.unwrap_or(0.0).to_le_bytes());
        data.extend_from_slice(&res.width.unwrap_or(0).to_le_bytes());
        data.extend_from_slice(&res.height.unwrap_or(0).to_le_bytes());
        data.extend_from_slice(&frame.rgba);
    } else {
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&res.duration_secs.unwrap_or(0.0).to_le_bytes());
        data.extend_from_slice(&res.width.unwrap_or(0).to_le_bytes());
        data.extend_from_slice(&res.height.unwrap_or(0).to_le_bytes());
    }
    let _ = std::fs::write(cache_path, data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_detection_and_cache_roundtrip() {
        assert!(is_audio_file(Path::new("music.flac")));
        assert!(is_audio_file(Path::new("song.mp3")));
        assert!(!is_audio_file(Path::new("movie.mkv")));
        assert!(!is_audio_file(Path::new("video.mp4")));

        let black = vec![0u8; 144 * 81 * 4];
        assert!(is_frame_mostly_black(&black));

        let bright = vec![255u8; 144 * 81 * 4];
        assert!(!is_frame_mostly_black(&bright));

        let tmp_file = std::env::temp_dir().join("nixo_test_thumb.thumb");
        let fake_res = ThumbnailResult {
            path: PathBuf::from("test.mp4"),
            frames: vec![VideoFrame {
                width: 144,
                height: 81,
                rgba: bright.clone(),
                frame_id: 1,
            }],
            duration_secs: Some(120.5),
            width: Some(1920),
            height: Some(1080),
        };
        save_to_cache(&tmp_file, &fake_res);
        let loaded = load_from_cache(&tmp_file, Path::new("test.mp4")).expect("cached");
        assert_eq!(loaded.duration_secs, Some(120.5));
        assert_eq!(loaded.width, Some(1920));
        assert_eq!(loaded.height, Some(1080));
        assert_eq!(loaded.frames.len(), 1);
        let _ = std::fs::remove_file(tmp_file);
    }
}
