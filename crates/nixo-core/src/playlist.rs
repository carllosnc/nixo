use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// Extensões comuns de vídeo suportadas pelo reprodutor Nixo.
pub const DEFAULT_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts", "mts", "m2ts", "vob", "ogv",
    "3gp", "rmvb",
];

/// Modo de repetição da reprodução de mídia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::All, Self::One];

    /// Alterna sequencialmente entre os modos de repetição: Off -> All -> One -> Off.
    pub fn cycle(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }
}

/// Token para ordenação natural alfanumérica humana.
#[derive(Debug, PartialEq, Eq)]
enum NaturalChunk<'a> {
    Str(&'a str),
    Num(u64),
}

/// Compara duas strings usando ordenação natural (ex: "Ep 2" vem antes de "Ep 10").
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut chunks_a = split_natural(a);
    let mut chunks_b = split_natural(b);

    loop {
        match (chunks_a.next(), chunks_b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(ca), Some(cb)) => {
                let ord = match (ca, cb) {
                    (NaturalChunk::Num(na), NaturalChunk::Num(nb)) => na.cmp(&nb),
                    (NaturalChunk::Str(sa), NaturalChunk::Str(sb)) => {
                        sa.to_lowercase().cmp(&sb.to_lowercase())
                    }
                    (NaturalChunk::Num(_), NaturalChunk::Str(_)) => Ordering::Less,
                    (NaturalChunk::Str(_), NaturalChunk::Num(_)) => Ordering::Greater,
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}

/// Divide uma string em partes numéricas e não-numéricas.
fn split_natural(s: &str) -> impl Iterator<Item = NaturalChunk<'_>> {
    let mut chars = s.char_indices().peekable();
    let mut result = Vec::new();

    while let Some(&(start, ch)) = chars.peek() {
        if ch.is_ascii_digit() {
            let mut end = start;
            while let Some(&(idx, d)) = chars.peek() {
                if d.is_ascii_digit() {
                    end = idx + d.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            let slice = &s[start..end];
            let num = slice.parse::<u64>().unwrap_or(u64::MAX);
            result.push(NaturalChunk::Num(num));
        } else {
            let mut end = start;
            while let Some(&(idx, nd)) = chars.peek() {
                if !nd.is_ascii_digit() {
                    end = idx + nd.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            result.push(NaturalChunk::Str(&s[start..end]));
        }
    }

    result.into_iter()
}

/// Gerenciador desacoplado de playlist de arquivos de vídeo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Playlist {
    items: Vec<PathBuf>,
    current_index: Option<usize>,
    repeat_mode: RepeatMode,
}

impl Playlist {
    /// Cria uma nova playlist vazia.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria uma playlist a partir do diretório do arquivo ativo, ordenando os arquivos irmãos
    /// de forma natural e definindo o arquivo ativo como o item atual.
    pub fn from_folder_with_active(folder: &Path, active_file: &Path) -> Self {
        let mut items = Vec::new();

        if let Ok(entries) = std::fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if DEFAULT_VIDEO_EXTENSIONS.contains(&ext.as_str()) {
                        items.push(path);
                    }
                }
            }
        }

        // Ordenação natural de arquivos por nome
        items.sort_by(|a, b| {
            let name_a = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let name_b = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
            natural_cmp(name_a, name_b)
        });

        // Se o arquivo ativo não estiver na lista (ex: extensão atípica), insere-o
        let current_index = items.iter().position(|p| p == active_file).or_else(|| {
            if active_file.is_file() {
                items.push(active_file.to_path_buf());
                items.sort_by(|a, b| {
                    let name_a = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let name_b = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    natural_cmp(name_a, name_b)
                });
                items.iter().position(|p| p == active_file)
            } else {
                None
            }
        });

        Self {
            items,
            current_index,
            repeat_mode: RepeatMode::Off,
        }
    }

    /// Retorna a lista imutável de itens da playlist.
    pub fn items(&self) -> &[PathBuf] {
        &self.items
    }

    /// Retorna o índice do item ativo atualmente.
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Retorna a referência ao caminho do item atual.
    pub fn current(&self) -> Option<&PathBuf> {
        self.current_index.and_then(|idx| self.items.get(idx))
    }

    /// Quantidade total de arquivos na playlist.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Indica se a playlist está vazia.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Modo de repetição ativo.
    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    /// Altera o modo de repetição.
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    /// Alterna sequencialmente o modo de repetição e retorna o novo valor.
    pub fn cycle_repeat_mode(&mut self) -> RepeatMode {
        self.repeat_mode = self.repeat_mode.cycle();
        self.repeat_mode
    }

    /// Indica se há um próximo item disponível para reprodução.
    pub fn has_next(&self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        match self.repeat_mode {
            RepeatMode::All => true,
            RepeatMode::One => self.current_index.is_some(),
            RepeatMode::Off => {
                if let Some(idx) = self.current_index {
                    idx + 1 < self.items.len()
                } else {
                    !self.items.is_empty()
                }
            }
        }
    }

    /// Indica se há um item anterior disponível.
    pub fn has_previous(&self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        match self.repeat_mode {
            RepeatMode::All => true,
            RepeatMode::One => self.current_index.is_some(),
            RepeatMode::Off => {
                if let Some(idx) = self.current_index {
                    idx > 0
                } else {
                    false
                }
            }
        }
    }

    /// Consulta o próximo item sem avançar o cursor da playlist.
    pub fn peek_next(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            return None;
        }
        match self.repeat_mode {
            RepeatMode::One => self.current(),
            RepeatMode::All => {
                let next_idx = match self.current_index {
                    Some(idx) if idx + 1 < self.items.len() => idx + 1,
                    _ => 0,
                };
                self.items.get(next_idx)
            }
            RepeatMode::Off => {
                let next_idx = match self.current_index {
                    Some(idx) => idx + 1,
                    None => 0,
                };
                self.items.get(next_idx)
            }
        }
    }

    /// Consulta o item anterior sem recuar o cursor da playlist.
    pub fn peek_previous(&self) -> Option<&PathBuf> {
        if self.items.is_empty() {
            return None;
        }
        match self.repeat_mode {
            RepeatMode::One => self.current(),
            RepeatMode::All => {
                let prev_idx = match self.current_index {
                    Some(0) | None => self.items.len().saturating_sub(1),
                    Some(idx) => idx - 1,
                };
                self.items.get(prev_idx)
            }
            RepeatMode::Off => {
                let prev_idx = match self.current_index {
                    Some(0) | None => return None,
                    Some(idx) => idx - 1,
                };
                self.items.get(prev_idx)
            }
        }
    }

    /// Avança o cursor para o próximo item e retorna o novo caminho.
    pub fn advance_next(&mut self) -> Option<PathBuf> {
        let next_path = self.peek_next().cloned()?;
        if let Some(pos) = self.items.iter().position(|p| p == &next_path) {
            self.current_index = Some(pos);
        }
        Some(next_path)
    }

    /// Recua o cursor para o item anterior e retorna o novo caminho.
    pub fn advance_previous(&mut self) -> Option<PathBuf> {
        let prev_path = self.peek_previous().cloned()?;
        if let Some(pos) = self.items.iter().position(|p| p == &prev_path) {
            self.current_index = Some(pos);
        }
        Some(prev_path)
    }

    /// Define o item ativo buscando pelo caminho no vetor.
    pub fn set_current_by_path(&mut self, path: &Path) {
        if let Some(pos) = self.items.iter().position(|p| p == path) {
            self.current_index = Some(pos);
        } else {
            self.items.push(path.to_path_buf());
            self.current_index = Some(self.items.len() - 1);
        }
    }

    /// Define o item ativo diretamente pelo seu índice ordinal na lista.
    /// Retorna o caminho do item se o índice for válido, ou `None` caso contrário.
    pub fn set_current_by_index(&mut self, index: usize) -> Option<PathBuf> {
        if index < self.items.len() {
            self.current_index = Some(index);
            self.items.get(index).cloned()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_cmp() {
        assert_eq!(natural_cmp("Ep 1.mp4", "Ep 2.mp4"), Ordering::Less);
        assert_eq!(natural_cmp("Ep 2.mp4", "Ep 10.mp4"), Ordering::Less);
        assert_eq!(natural_cmp("Ep 10.mp4", "Ep 2.mp4"), Ordering::Greater);
        assert_eq!(natural_cmp("s01e09.mkv", "s01e10.mkv"), Ordering::Less);
        assert_eq!(natural_cmp("Video 1.avi", "video 1.avi"), Ordering::Equal);
    }

    #[test]
    fn test_repeat_mode_cycle() {
        assert_eq!(RepeatMode::Off.cycle(), RepeatMode::All);
        assert_eq!(RepeatMode::All.cycle(), RepeatMode::One);
        assert_eq!(RepeatMode::One.cycle(), RepeatMode::Off);
    }

    #[test]
    fn test_playlist_navigation() {
        let mut pl = Playlist {
            items: vec![
                PathBuf::from("ep01.mp4"),
                PathBuf::from("ep02.mp4"),
                PathBuf::from("ep03.mp4"),
            ],
            current_index: Some(0),
            repeat_mode: RepeatMode::Off,
        };

        assert!(pl.has_next());
        assert!(!pl.has_previous());

        let next = pl.advance_next();
        assert_eq!(next, Some(PathBuf::from("ep02.mp4")));
        assert_eq!(pl.current_index(), Some(1));
        assert!(pl.has_next());
        assert!(pl.has_previous());

        let next2 = pl.advance_next();
        assert_eq!(next2, Some(PathBuf::from("ep03.mp4")));
        assert_eq!(pl.current_index(), Some(2));
        assert!(!pl.has_next());
        assert!(pl.has_previous());

        let prev = pl.advance_previous();
        assert_eq!(prev, Some(PathBuf::from("ep02.mp4")));
        assert_eq!(pl.current_index(), Some(1));
    }

    #[test]
    fn test_playlist_repeat_all() {
        let mut pl = Playlist {
            items: vec![PathBuf::from("a.mp4"), PathBuf::from("b.mp4")],
            current_index: Some(1),
            repeat_mode: RepeatMode::All,
        };

        assert!(pl.has_next());
        let wrapped = pl.advance_next();
        assert_eq!(wrapped, Some(PathBuf::from("a.mp4")));
        assert_eq!(pl.current_index(), Some(0));
    }

    #[test]
    fn test_playlist_set_current_by_index() {
        let mut pl = Playlist {
            items: vec![
                PathBuf::from("ep01.mp4"),
                PathBuf::from("ep02.mp4"),
                PathBuf::from("ep03.mp4"),
            ],
            current_index: Some(0),
            repeat_mode: RepeatMode::Off,
        };

        assert_eq!(pl.set_current_by_index(2), Some(PathBuf::from("ep03.mp4")));
        assert_eq!(pl.current_index(), Some(2));

        assert_eq!(pl.set_current_by_index(0), Some(PathBuf::from("ep01.mp4")));
        assert_eq!(pl.current_index(), Some(0));

        assert_eq!(pl.set_current_by_index(99), None);
        assert_eq!(pl.current_index(), Some(0));
    }
}
