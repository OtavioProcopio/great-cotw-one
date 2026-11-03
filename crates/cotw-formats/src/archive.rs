//! Índice dos arquivos do jogo: pares `gameN.tab` / `gameN.arc` em `archives_win64`.
//!
//! Cada `.tab` tem um cabeçalho de 12 bytes (`TAB\0` + 8 bytes) seguido de entradas
//! `{u32 hash, u32 offset, u32 size}` que apontam para dentro do `.arc` de mesmo número.
//! O `hash` é o `hashlittle` do caminho em minúsculas; como são só 32 bits, colisões
//! existem: quem consome deve validar a assinatura do conteúdo encontrado.
//!
//! Só os `.tab` (pequenos) são carregados; os `.arc` (centenas de MB) são lidos por
//! posição, uma entrada por vez.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::bytes;
use crate::hash::hashlittle;

const TAB_MAGIC: &[u8; 4] = b"TAB\0";
const TAB_HEADER_LEN: usize = 12;
const TAB_ENTRY_LEN: usize = 12;

/// Erros ao abrir o índice ou ler uma entrada.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    /// Falha de leitura de arquivo.
    #[error("erro de E/S em {}: {source}", path.display())]
    Io {
        /// Arquivo envolvido.
        path: PathBuf,
        /// Erro original.
        #[source]
        source: io::Error,
    },
    /// O `.tab` não começa com a assinatura esperada.
    #[error("{} não é um índice .tab válido", .0.display())]
    BadTab(PathBuf),
    /// A pasta não tem nenhum `gameN.tab`.
    #[error("nenhum arquivo gameN.tab em {}", .0.display())]
    NoArchives(PathBuf),
    /// A entrada aponta para fora do `.arc` (arquivo truncado ou índice desatualizado).
    #[error("entrada fora do .arc (offset {offset}, tamanho {size}, arquivo com {file_len} bytes)")]
    OutOfRange {
        /// Offset da entrada.
        offset: u32,
        /// Tamanho da entrada.
        size: u32,
        /// Tamanho real do `.arc`.
        file_len: u64,
    },
}

/// Onde uma entrada está guardada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// Número do arquivo (`gameN`).
    pub archive: u32,
    /// Offset dentro do `.arc`.
    pub offset: u32,
    /// Tamanho em bytes.
    pub size: u32,
}

/// Índice de todos os `.tab` de uma instalação.
#[derive(Debug)]
pub struct GameArchives {
    dir: PathBuf,
    index: HashMap<u32, Vec<Location>>,
    entry_count: usize,
}

impl GameArchives {
    /// Carrega todos os `gameN.tab` de `dir` (normalmente `<jogo>/archives_win64`).
    ///
    /// # Errors
    ///
    /// Retorna [`ArchiveError`] se a pasta não puder ser lida, não tiver nenhum `.tab`
    /// ou algum `.tab` for inválido.
    pub fn open(dir: &Path) -> Result<Self, ArchiveError> {
        let io_error = |path: &Path, source| ArchiveError::Io {
            path: path.to_path_buf(),
            source,
        };
        let mut numbers: Vec<u32> = fs::read_dir(dir)
            .map_err(|e| io_error(dir, e))?
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().into_string().ok()?;
                name.strip_prefix("game")?
                    .strip_suffix(".tab")?
                    .parse()
                    .ok()
            })
            .collect();
        if numbers.is_empty() {
            return Err(ArchiveError::NoArchives(dir.to_path_buf()));
        }
        numbers.sort_unstable();

        let mut index: HashMap<u32, Vec<Location>> = HashMap::new();
        let mut entry_count = 0;
        for archive in numbers {
            let path = dir.join(format!("game{archive}.tab"));
            let bytes = fs::read(&path).map_err(|e| io_error(&path, e))?;
            let entries = parse_tab(&bytes).ok_or_else(|| ArchiveError::BadTab(path.clone()))?;
            entry_count += entries.len();
            for (hash, offset, size) in entries {
                index.entry(hash).or_default().push(Location {
                    archive,
                    offset,
                    size,
                });
            }
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            index,
            entry_count,
        })
    }

    /// Total de entradas nos índices.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Todas as entradas dos índices, como `(hash do caminho, onde está)`.
    pub fn entries(&self) -> impl Iterator<Item = (u32, &Location)> {
        self.index
            .iter()
            .flat_map(|(hash, locations)| locations.iter().map(move |l| (*hash, l)))
    }

    /// Todas as entradas com este hash (mais de uma significa colisão ou arquivo repetido).
    #[must_use]
    pub fn find(&self, hash: u32) -> &[Location] {
        self.index.get(&hash).map_or(&[], Vec::as_slice)
    }

    /// Procura por caminho do jogo, como `settings/hp_settings/player_skills.bin`.
    #[must_use]
    pub fn find_path(&self, path: &str) -> &[Location] {
        self.find(hashlittle(path.to_ascii_lowercase().as_bytes(), 0))
    }

    /// Lê os bytes de uma entrada do `.arc` correspondente.
    ///
    /// # Errors
    ///
    /// Retorna [`ArchiveError`] se o `.arc` não puder ser lido ou a entrada passar do fim dele.
    pub fn read(&self, location: &Location) -> Result<Vec<u8>, ArchiveError> {
        let path = self.dir.join(format!("game{}.arc", location.archive));
        let io_error = |source| ArchiveError::Io {
            path: path.clone(),
            source,
        };
        let mut file = File::open(&path).map_err(io_error)?;
        let file_len = file.metadata().map_err(io_error)?.len();
        let end = u64::from(location.offset) + u64::from(location.size);
        if end > file_len {
            return Err(ArchiveError::OutOfRange {
                offset: location.offset,
                size: location.size,
                file_len,
            });
        }
        file.seek(SeekFrom::Start(u64::from(location.offset)))
            .map_err(io_error)?;
        let mut data = vec![0u8; location.size as usize];
        file.read_exact(&mut data).map_err(io_error)?;
        Ok(data)
    }
}

/// Entradas `(hash, offset, size)` de um `.tab`; `None` se a assinatura estiver errada.
fn parse_tab(data: &[u8]) -> Option<Vec<(u32, u32, u32)>> {
    if data.get(..4)? != TAB_MAGIC || data.len() < TAB_HEADER_LEN {
        return None;
    }
    let count = (data.len() - TAB_HEADER_LEN) / TAB_ENTRY_LEN;
    (0..count)
        .map(|i| {
            let at = TAB_HEADER_LEN + i * TAB_ENTRY_LEN;
            Some((
                bytes::u32_at(data, at)?,
                bytes::u32_at(data, at + 4)?,
                bytes::u32_at(data, at + 8)?,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(entries: &[(u32, u32, u32)]) -> Vec<u8> {
        let mut b = TAB_MAGIC.to_vec();
        b.extend_from_slice(&[2, 0, 1, 0, 0, 0x10, 0, 0]);
        for (hash, offset, size) in entries {
            b.extend_from_slice(&hash.to_le_bytes());
            b.extend_from_slice(&offset.to_le_bytes());
            b.extend_from_slice(&size.to_le_bytes());
        }
        b
    }

    fn game_hash(path: &str) -> u32 {
        hashlittle(path.as_bytes(), 0)
    }

    /// Pasta com `game0` (duas entradas) e `game3` (uma, mesmo hash de `game0`).
    fn sample_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let a = game_hash("missions/a.bin");
        let b = game_hash("settings/b.bin");
        fs::write(dir.path().join("game0.tab"), tab(&[(a, 4, 5), (b, 0, 4)])).unwrap();
        fs::write(dir.path().join("game0.arc"), b"abcdHELLOxyz").unwrap();
        fs::write(dir.path().join("game3.tab"), tab(&[(a, 0, 3)])).unwrap();
        fs::write(dir.path().join("game3.arc"), b"ZZZ").unwrap();
        dir
    }

    #[test]
    fn indexes_every_tab_in_numeric_order() {
        let dir = sample_dir();
        let archives = GameArchives::open(dir.path()).unwrap();
        assert_eq!(archives.entry_count(), 3);
        let hits = archives.find_path("missions/a.bin");
        assert_eq!(hits.len(), 2);
        assert_eq!((hits[0].archive, hits[1].archive), (0, 3));
    }

    #[test]
    fn find_path_is_case_insensitive_and_unknown_is_empty() {
        let dir = sample_dir();
        let archives = GameArchives::open(dir.path()).unwrap();
        assert_eq!(archives.find_path("Settings/B.BIN").len(), 1);
        assert_eq!(archives.find_path("nao/existe"), []);
    }

    #[test]
    fn reads_entry_bytes_from_the_matching_arc() {
        let dir = sample_dir();
        let archives = GameArchives::open(dir.path()).unwrap();
        let hits = archives.find_path("missions/a.bin");
        assert_eq!(archives.read(&hits[0]).unwrap(), b"HELLO");
        assert_eq!(archives.read(&hits[1]).unwrap(), b"ZZZ");
    }

    #[test]
    fn ignores_files_that_are_not_game_tabs() {
        let dir = sample_dir();
        fs::write(dir.path().join("gamex.tab"), b"lixo").unwrap();
        fs::write(dir.path().join("game9.txt"), b"lixo").unwrap();
        fs::write(dir.path().join("readme.tab"), b"lixo").unwrap();
        assert_eq!(GameArchives::open(dir.path()).unwrap().entry_count(), 3);
    }

    #[test]
    fn folder_without_tabs_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            GameArchives::open(dir.path()),
            Err(ArchiveError::NoArchives(_))
        ));
    }

    #[test]
    fn missing_folder_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nada");
        assert!(matches!(
            GameArchives::open(&missing),
            Err(ArchiveError::Io { .. })
        ));
    }

    #[test]
    fn tab_with_wrong_signature_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("game0.tab"), b"XXXX00000000").unwrap();
        assert!(matches!(
            GameArchives::open(dir.path()),
            Err(ArchiveError::BadTab(_))
        ));
        fs::write(dir.path().join("game0.tab"), b"TAB").unwrap();
        assert!(matches!(
            GameArchives::open(dir.path()),
            Err(ArchiveError::BadTab(_))
        ));
    }

    #[test]
    fn entry_past_end_of_arc_is_out_of_range() {
        let dir = sample_dir();
        let archives = GameArchives::open(dir.path()).unwrap();
        let bad = Location {
            archive: 0,
            offset: 10,
            size: 50,
        };
        assert!(matches!(
            archives.read(&bad),
            Err(ArchiveError::OutOfRange { file_len: 12, .. })
        ));
    }

    #[test]
    fn missing_arc_is_an_io_error() {
        let dir = sample_dir();
        let archives = GameArchives::open(dir.path()).unwrap();
        let gone = Location {
            archive: 77,
            offset: 0,
            size: 1,
        };
        assert!(matches!(archives.read(&gone), Err(ArchiveError::Io { .. })));
    }
}
