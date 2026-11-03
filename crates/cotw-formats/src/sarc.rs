//! SARC: lista de arquivos (nome, offset, tamanho) que o AAF descomprime.
//!
//! ```text
//! @4   "SARC"
//! @12  u32 tamanho da área de entradas
//! @16  entradas: u32 tamanho do nome, nome (com NULs de preenchimento), u32 offset, u32 tamanho
//! ```
//!
//! Offset 0 significa arquivo externo (não está neste SARC).

use crate::bytes;

const HEADER_LEN: usize = 16;

/// Erros ao ler um SARC.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SarcError {
    /// Não tem `SARC` na posição 4.
    #[error("assinatura SARC ausente")]
    BadMagic,
    /// A tabela de entradas passa do fim dos dados.
    #[error("SARC truncado (offset {offset})")]
    Truncated {
        /// Posição onde faltou dado.
        offset: usize,
    },
}

/// Uma entrada da tabela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SarcEntry {
    /// Caminho do arquivo dentro do jogo.
    pub name: String,
    /// Offset dos dados neste SARC; 0 = arquivo externo.
    pub offset: u32,
    /// Tamanho dos dados.
    pub size: u32,
}

/// SARC lido, com os bytes originais para extrair o conteúdo das entradas.
#[derive(Debug, Clone)]
pub struct Sarc {
    data: Vec<u8>,
    entries: Vec<SarcEntry>,
}

impl Sarc {
    /// Lê a tabela de entradas.
    ///
    /// # Errors
    ///
    /// Retorna [`SarcError`] se a assinatura estiver errada ou a tabela for truncada.
    pub fn parse(data: Vec<u8>) -> Result<Self, SarcError> {
        if bytes::slice(&data, 4, 4) != Some(b"SARC") {
            return Err(SarcError::BadMagic);
        }
        let area = bytes::u32_at(&data, 12).ok_or(SarcError::Truncated { offset: 12 })?;
        let end = HEADER_LEN.saturating_add(area as usize);

        let mut entries = Vec::new();
        let mut pos = HEADER_LEN;
        while pos < end {
            let truncated = || SarcError::Truncated { offset: pos };
            let name_len = bytes::u32_at(&data, pos).ok_or_else(truncated)? as usize;
            if name_len == 0 {
                break;
            }
            let raw = bytes::slice(&data, pos + 4, name_len).ok_or_else(truncated)?;
            let trimmed = raw
                .iter()
                .rposition(|&b| b != 0)
                .map_or(&raw[..0], |i| &raw[..=i]);
            let fields = pos + 4 + name_len;
            let offset = bytes::u32_at(&data, fields).ok_or_else(truncated)?;
            let size = bytes::u32_at(&data, fields + 4).ok_or_else(truncated)?;
            entries.push(SarcEntry {
                name: String::from_utf8_lossy(trimmed).into_owned(),
                offset,
                size,
            });
            pos = fields + 8;
        }
        Ok(Self { data, entries })
    }

    /// Entradas na ordem do arquivo.
    #[must_use]
    pub fn entries(&self) -> &[SarcEntry] {
        &self.entries
    }

    /// Busca uma entrada pelo nome exato.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&SarcEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Bytes de uma entrada, ou `None` se for externa (offset 0) ou passar do fim.
    #[must_use]
    pub fn data_of(&self, entry: &SarcEntry) -> Option<&[u8]> {
        if entry.offset == 0 {
            return None;
        }
        bytes::slice(&self.data, entry.offset as usize, entry.size as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(nome, offset, tamanho)`; nomes recebem `pad` NULs de preenchimento.
    fn build(entries: &[(&str, u32, u32)], pad: usize, payload: &[u8]) -> Vec<u8> {
        let mut table = Vec::new();
        for (name, offset, size) in entries {
            let len = u32::try_from(name.len() + pad).unwrap();
            table.extend_from_slice(&len.to_le_bytes());
            table.extend_from_slice(name.as_bytes());
            table.extend(std::iter::repeat_n(0u8, pad));
            table.extend_from_slice(&offset.to_le_bytes());
            table.extend_from_slice(&size.to_le_bytes());
        }
        let mut b = vec![0u8; HEADER_LEN];
        b[4..8].copy_from_slice(b"SARC");
        b[12..16].copy_from_slice(&u32::try_from(table.len()).unwrap().to_le_bytes());
        b.extend(table);
        b.extend_from_slice(payload);
        b
    }

    #[test]
    fn reads_entries_and_strips_name_padding() {
        let raw = build(&[("a/b.bin", 0, 10), ("c.bin", 100, 4)], 3, &[]);
        let sarc = Sarc::parse(raw).unwrap();
        let names: Vec<_> = sarc.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a/b.bin", "c.bin"]);
        assert_eq!(sarc.find("c.bin").unwrap().offset, 100);
        assert!(sarc.find("nada").is_none());
    }

    #[test]
    fn extracts_internal_entries_and_skips_external_ones() {
        let mut raw = build(&[("ext.bin", 0, 5), ("in.bin", 0, 3)], 1, b"");
        // Dados no fim; aponta a segunda entrada para eles.
        let data_at = u32::try_from(raw.len()).unwrap();
        raw.extend_from_slice(b"xyz");
        let sarc = Sarc::parse(raw.clone()).unwrap();
        assert!(sarc.data_of(&sarc.entries()[0]).is_none());

        let entry = SarcEntry {
            name: "in.bin".into(),
            offset: data_at,
            size: 3,
        };
        assert_eq!(sarc.data_of(&entry), Some(&b"xyz"[..]));
        let beyond = SarcEntry {
            offset: data_at,
            size: 99,
            ..entry
        };
        assert!(sarc.data_of(&beyond).is_none());
    }

    #[test]
    fn zero_length_name_ends_the_table() {
        let mut raw = build(&[("a", 1, 1)], 0, &[]);
        raw.extend_from_slice(&[0; 4]);
        let area = u32::try_from(raw.len() - HEADER_LEN).unwrap();
        raw[12..16].copy_from_slice(&area.to_le_bytes());
        assert_eq!(Sarc::parse(raw).unwrap().entries().len(), 1);
    }

    #[test]
    fn rejects_wrong_magic_and_truncation() {
        assert_eq!(Sarc::parse(vec![0; 20]).unwrap_err(), SarcError::BadMagic);
        let mut raw = build(&[("a.bin", 1, 1)], 0, &[]);
        raw.truncate(raw.len() - 3);
        assert!(matches!(Sarc::parse(raw), Err(SarcError::Truncated { .. })));
        let mut short = vec![0u8; 8];
        short[4..8].copy_from_slice(b"SARC");
        assert_eq!(
            Sarc::parse(short).unwrap_err(),
            SarcError::Truncated { offset: 12 }
        );
    }
}
