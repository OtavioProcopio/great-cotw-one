//! Envelope dos arquivos de save (`animal_population_N`, `*_adf` ...).
//!
//! Layout (ver `.claude/skills/cotw-save-knowledge/SKILL.md`, seção 2.1):
//!
//! ```text
//! 0..32   cabeçalho: "SAVE" @0, "COMP" @16, tamanho descomprimido (u32 LE) @8 e @24
//! 32..    fluxo zlib que, descomprimido, vira: prefixo de 5 bytes + arquivo ADF
//! ```

use std::io::{self, Read, Write};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

const HEADER_LEN: usize = 32;
const PREFIX_LEN: usize = 5;
const SIZE_OFFSETS: [usize; 2] = [8, 24];

/// Erros ao ler ou gravar o envelope de um save.
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    /// O arquivo não tem nem o cabeçalho completo.
    #[error("arquivo pequeno demais ({0} bytes)")]
    TooShort(usize),
    /// O cabeçalho não começa com `SAVE` / não tem `COMP` na posição 16.
    #[error("cabeçalho inválido: não é um save do jogo")]
    BadMagic,
    /// Os dois campos de tamanho do cabeçalho discordam entre si.
    #[error("campos de tamanho do cabeçalho divergentes ({first} e {second})")]
    InconsistentSizes {
        /// Valor no offset 8.
        first: u32,
        /// Valor no offset 24.
        second: u32,
    },
    /// O tamanho declarado no cabeçalho não bate com o que saiu da descompressão.
    #[error("tamanho declarado ({declared}) difere do descomprimido ({actual})")]
    SizeMismatch {
        /// Tamanho escrito no cabeçalho.
        declared: u32,
        /// Tamanho real após descomprimir.
        actual: usize,
    },
    /// O conteúdo descomprimido não comporta o prefixo de 5 bytes.
    #[error("conteúdo descomprimido pequeno demais")]
    PayloadTooShort,
    /// O conteúdo a gravar não cabe num campo u32.
    #[error("conteúdo grande demais para o formato")]
    TooLarge,
    /// Falha do zlib (fluxo corrompido, por exemplo).
    #[error("erro de compressão zlib: {0}")]
    Zlib(#[from] io::Error),
}

/// Um save aberto: cabeçalho e prefixo preservados, mais o ADF descomprimido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEnvelope {
    header: [u8; HEADER_LEN],
    prefix: [u8; PREFIX_LEN],
    adf: Vec<u8>,
}

impl SaveEnvelope {
    /// Lê os bytes crus de um arquivo de save.
    ///
    /// # Errors
    ///
    /// Retorna [`EnvelopeError`] se o cabeçalho, o fluxo zlib ou os tamanhos forem inválidos.
    pub fn parse(raw: &[u8]) -> Result<Self, EnvelopeError> {
        let (header, compressed) = raw
            .split_first_chunk::<HEADER_LEN>()
            .ok_or(EnvelopeError::TooShort(raw.len()))?;
        if &header[0..4] != b"SAVE" || &header[16..20] != b"COMP" {
            return Err(EnvelopeError::BadMagic);
        }
        let declared = declared_size(header)?;

        // Limita a leitura ao tamanho declarado (+1 para detectar excesso): um save
        // malicioso não consegue nos fazer alocar memória sem limite.
        let limit = u64::from(declared) + 1;
        let mut decompressed = Vec::new();
        ZlibDecoder::new(compressed)
            .take(limit)
            .read_to_end(&mut decompressed)?;
        if decompressed.len() != declared as usize {
            return Err(EnvelopeError::SizeMismatch {
                declared,
                actual: decompressed.len(),
            });
        }

        let (prefix, adf) = decompressed
            .split_first_chunk::<PREFIX_LEN>()
            .ok_or(EnvelopeError::PayloadTooShort)?;
        Ok(Self {
            header: *header,
            prefix: *prefix,
            adf: adf.to_vec(),
        })
    }

    /// Prefixo de 5 bytes que vem antes do ADF; deve ser preservado ao regravar.
    #[must_use]
    pub fn prefix(&self) -> &[u8; PREFIX_LEN] {
        &self.prefix
    }

    /// Bytes do ADF (já sem o prefixo de 5 bytes).
    #[must_use]
    pub fn adf(&self) -> &[u8] {
        &self.adf
    }

    /// Substitui o ADF, por exemplo depois de uma edição.
    pub fn set_adf(&mut self, adf: Vec<u8>) {
        self.adf = adf;
    }

    /// Serializa de volta para o formato de arquivo, atualizando os dois campos de tamanho.
    ///
    /// A compressão não é idêntica byte a byte à do jogo, mas o conteúdo descomprimido é.
    ///
    /// # Errors
    ///
    /// Retorna [`EnvelopeError::TooLarge`] se o conteúdo não couber em um u32, ou
    /// [`EnvelopeError::Zlib`] se a compressão falhar.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EnvelopeError> {
        let size = PREFIX_LEN + self.adf.len();
        let size_u32 = u32::try_from(size).map_err(|_| EnvelopeError::TooLarge)?;

        // O jogo grava com o nível mais rápido (cabeçalho zlib `78 01`); imitamos para que o
        // arquivo regravado seja o mais parecido possível com os que o jogo gera.
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&self.prefix)?;
        encoder.write_all(&self.adf)?;
        let compressed = encoder.finish()?;

        let mut header = self.header;
        for offset in SIZE_OFFSETS {
            header[offset..offset + 4].copy_from_slice(&size_u32.to_le_bytes());
        }
        let mut out = Vec::with_capacity(HEADER_LEN + compressed.len());
        out.extend_from_slice(&header);
        out.extend_from_slice(&compressed);
        Ok(out)
    }
}

/// Lê os dois campos de tamanho do cabeçalho e exige que sejam iguais.
fn declared_size(header: &[u8; HEADER_LEN]) -> Result<u32, EnvelopeError> {
    let [first, second] = SIZE_OFFSETS
        .map(|o| u32::from_le_bytes([header[o], header[o + 1], header[o + 2], header[o + 3]]));
    if first == second {
        Ok(first)
    } else {
        Err(EnvelopeError::InconsistentSizes { first, second })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monta um arquivo de save sintético válido com o ADF dado.
    fn build_raw(adf: &[u8]) -> Vec<u8> {
        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(b"SAVE");
        header[16..20].copy_from_slice(b"COMP");
        let mut payload = vec![1, 1, 0, 0, 0];
        payload.extend_from_slice(adf);
        let size = u32::try_from(payload.len()).unwrap().to_le_bytes();
        header[8..12].copy_from_slice(&size);
        header[24..28].copy_from_slice(&size);

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&payload).unwrap();
        let mut raw = header.to_vec();
        raw.extend(encoder.finish().unwrap());
        raw
    }

    #[test]
    fn parse_extracts_adf_without_prefix() {
        let envelope = SaveEnvelope::parse(&build_raw(b" FDAdados")).unwrap();
        assert_eq!(envelope.adf(), b" FDAdados");
        assert_eq!(envelope.prefix(), &[1, 1, 0, 0, 0]);
    }

    #[test]
    fn round_trip_preserves_decompressed_content() {
        let envelope = SaveEnvelope::parse(&build_raw(b" FDAdados")).unwrap();
        let again = SaveEnvelope::parse(&envelope.to_bytes().unwrap()).unwrap();
        assert_eq!(envelope, again);
    }

    #[test]
    fn to_bytes_updates_both_size_fields_when_adf_changes() {
        let mut envelope = SaveEnvelope::parse(&build_raw(b" FDA")).unwrap();
        envelope.set_adf(vec![7; 100]);
        let bytes = envelope.to_bytes().unwrap();
        assert_eq!(bytes[8..12], 105u32.to_le_bytes());
        assert_eq!(bytes[24..28], 105u32.to_le_bytes());
        assert_eq!(SaveEnvelope::parse(&bytes).unwrap().adf(), &[7; 100][..]);
    }

    #[test]
    fn rejects_input_shorter_than_header() {
        let err = SaveEnvelope::parse(&[0; 10]).unwrap_err();
        assert!(matches!(err, EnvelopeError::TooShort(10)));
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut raw = build_raw(b" FDA");
        raw[0] = b'X';
        assert!(matches!(
            SaveEnvelope::parse(&raw),
            Err(EnvelopeError::BadMagic)
        ));
    }

    #[test]
    fn rejects_inconsistent_size_fields() {
        let mut raw = build_raw(b" FDA");
        raw[24] ^= 1;
        assert!(matches!(
            SaveEnvelope::parse(&raw),
            Err(EnvelopeError::InconsistentSizes { .. })
        ));
    }

    #[test]
    fn rejects_declared_size_smaller_than_real() {
        let mut raw = build_raw(b" FDAdados");
        for offset in SIZE_OFFSETS {
            raw[offset..offset + 4].copy_from_slice(&6u32.to_le_bytes());
        }
        assert!(matches!(
            SaveEnvelope::parse(&raw),
            Err(EnvelopeError::SizeMismatch { declared: 6, .. })
        ));
    }

    #[test]
    fn rejects_declared_size_larger_than_real() {
        let mut raw = build_raw(b" FDA");
        for offset in SIZE_OFFSETS {
            raw[offset..offset + 4].copy_from_slice(&999u32.to_le_bytes());
        }
        assert!(matches!(
            SaveEnvelope::parse(&raw),
            Err(EnvelopeError::SizeMismatch { declared: 999, .. })
        ));
    }

    #[test]
    fn rejects_corrupt_zlib_stream() {
        let mut raw = build_raw(b" FDA");
        raw.truncate(HEADER_LEN + 4);
        assert!(SaveEnvelope::parse(&raw).is_err());
    }

    #[test]
    fn rejects_payload_without_room_for_prefix() {
        let mut raw = build_raw(b"");
        // Recomprime só 3 bytes e ajusta o cabeçalho para 3.
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&[1, 2, 3]).unwrap();
        raw.truncate(HEADER_LEN);
        raw.extend(encoder.finish().unwrap());
        for offset in SIZE_OFFSETS {
            raw[offset..offset + 4].copy_from_slice(&3u32.to_le_bytes());
        }
        assert!(matches!(
            SaveEnvelope::parse(&raw),
            Err(EnvelopeError::PayloadTooShort)
        ));
    }
}
