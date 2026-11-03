//! Bundles AAF: blocos comprimidos (deflate cru) que, concatenados, formam um SARC.
//!
//! ```text
//! 0..3    "AAF"
//! @44     u32 quantidade de blocos
//! @48..   por bloco: u32 comprimido, u32 descomprimido, u32 passo até o próximo bloco,
//!         4 bytes de magic, e os dados. Se comprimido == descomprimido o bloco é guardado cru.
//! ```

use std::io::{self, Read};

use flate2::read::DeflateDecoder;

use crate::bytes;

const COUNT_OFFSET: usize = 44;
const FIRST_CHUNK: usize = 48;
const CHUNK_HEADER_LEN: usize = 16;
/// Teto de saída (1 GiB): um AAF corrompido não consegue nos fazer alocar sem limite.
const MAX_DECODED: usize = 1 << 30;

/// Erros ao descompactar um AAF.
#[derive(Debug, thiserror::Error)]
pub enum AafError {
    /// Não começa com `AAF`.
    #[error("assinatura AAF ausente")]
    BadMagic,
    /// Cabeçalho ou bloco além do fim dos dados.
    #[error("AAF truncado (offset {offset})")]
    Truncated {
        /// Posição onde faltou dado.
        offset: usize,
    },
    /// Passo do bloco menor que o cabeçalho do bloco.
    #[error("bloco {chunk} com tamanho de passo inválido ({stride})")]
    BadStride {
        /// Índice do bloco.
        chunk: u32,
        /// Passo declarado.
        stride: u32,
    },
    /// O bloco descomprimido não tem o tamanho declarado.
    #[error("bloco {chunk}: declarado {declared} bytes, obtido {actual}")]
    SizeMismatch {
        /// Índice do bloco.
        chunk: u32,
        /// Tamanho declarado.
        declared: u32,
        /// Tamanho obtido.
        actual: usize,
    },
    /// A soma dos blocos passa do limite aceito.
    #[error("conteúdo descomprimido grande demais")]
    TooLarge,
    /// Falha do deflate.
    #[error("erro de descompressão: {0}")]
    Inflate(#[from] io::Error),
}

/// Descomprime um AAF inteiro.
///
/// # Errors
///
/// Retorna [`AafError`] se o cabeçalho, algum bloco ou os tamanhos forem inválidos.
pub fn decode(data: &[u8]) -> Result<Vec<u8>, AafError> {
    if !data.starts_with(b"AAF") {
        return Err(AafError::BadMagic);
    }
    let count = bytes::u32_at(data, COUNT_OFFSET).ok_or(AafError::Truncated {
        offset: COUNT_OFFSET,
    })?;

    let mut out = Vec::new();
    let mut pos = FIRST_CHUNK;
    for chunk in 0..count {
        let truncated = || AafError::Truncated { offset: pos };
        let compressed = bytes::u32_at(data, pos).ok_or_else(truncated)?;
        let declared = bytes::u32_at(data, pos + 4).ok_or_else(truncated)?;
        let stride = bytes::u32_at(data, pos + 8).ok_or_else(truncated)?;
        let payload = bytes::slice(data, pos + CHUNK_HEADER_LEN, compressed as usize)
            .ok_or_else(truncated)?;
        if (stride as usize) < CHUNK_HEADER_LEN {
            return Err(AafError::BadStride { chunk, stride });
        }
        if out.len().saturating_add(declared as usize) > MAX_DECODED {
            return Err(AafError::TooLarge);
        }

        let before = out.len();
        if compressed == declared {
            out.extend_from_slice(payload);
        } else {
            DeflateDecoder::new(payload)
                .take(u64::from(declared) + 1)
                .read_to_end(&mut out)?;
        }
        let actual = out.len() - before;
        if actual != declared as usize {
            return Err(AafError::SizeMismatch {
                chunk,
                declared,
                actual,
            });
        }
        pos = pos.saturating_add(stride as usize);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::DeflateEncoder;

    use super::*;

    enum Chunk<'a> {
        Stored(&'a [u8]),
        Deflated(&'a [u8]),
    }

    fn build(chunks: &[Chunk]) -> Vec<u8> {
        let mut b = vec![0u8; FIRST_CHUNK];
        b[0..3].copy_from_slice(b"AAF");
        b[COUNT_OFFSET..COUNT_OFFSET + 4]
            .copy_from_slice(&u32::try_from(chunks.len()).unwrap().to_le_bytes());
        for chunk in chunks {
            let (payload, declared) = match chunk {
                Chunk::Stored(raw) => (raw.to_vec(), raw.len()),
                Chunk::Deflated(raw) => {
                    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
                    enc.write_all(raw).unwrap();
                    (enc.finish().unwrap(), raw.len())
                }
            };
            let stride = CHUNK_HEADER_LEN + payload.len();
            for v in [payload.len(), declared, stride] {
                b.extend_from_slice(&u32::try_from(v).unwrap().to_le_bytes());
            }
            b.extend_from_slice(b"MAGC");
            b.extend_from_slice(&payload);
        }
        b
    }

    #[test]
    fn concatenates_stored_and_deflated_chunks() {
        let raw = build(&[
            Chunk::Deflated(&[b'a'; 500]),
            Chunk::Stored(b"cru"),
            Chunk::Deflated(b"fim fim fim fim"),
        ]);
        let mut expected = vec![b'a'; 500];
        expected.extend_from_slice(b"cru");
        expected.extend_from_slice(b"fim fim fim fim");
        assert_eq!(decode(&raw).unwrap(), expected);
    }

    #[test]
    fn empty_bundle_decodes_to_nothing() {
        assert_eq!(decode(&build(&[])).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn rejects_wrong_magic_and_short_header() {
        assert!(matches!(decode(b"XYZ"), Err(AafError::BadMagic)));
        assert!(matches!(
            decode(b"AAF\0"),
            Err(AafError::Truncated { offset: 44 })
        ));
    }

    #[test]
    fn rejects_truncated_chunk() {
        let mut raw = build(&[Chunk::Stored(b"abcdef")]);
        raw.truncate(raw.len() - 2);
        assert!(matches!(decode(&raw), Err(AafError::Truncated { .. })));
    }

    #[test]
    fn rejects_declared_size_that_does_not_match() {
        let mut raw = build(&[Chunk::Deflated(&[b'a'; 100])]);
        raw[FIRST_CHUNK + 4..FIRST_CHUNK + 8].copy_from_slice(&7u32.to_le_bytes());
        assert!(matches!(
            decode(&raw),
            Err(AafError::SizeMismatch { declared: 7, .. })
        ));
    }

    #[test]
    fn rejects_corrupt_deflate_stream() {
        let mut raw = build(&[Chunk::Deflated(&[b'a'; 100])]);
        raw[FIRST_CHUNK + CHUNK_HEADER_LEN] = 0xFF;
        assert!(decode(&raw).is_err());
    }

    #[test]
    fn rejects_stride_smaller_than_chunk_header() {
        let mut raw = build(&[Chunk::Stored(b"abc")]);
        raw[FIRST_CHUNK + 8..FIRST_CHUNK + 12].copy_from_slice(&4u32.to_le_bytes());
        assert!(matches!(
            decode(&raw),
            Err(AafError::BadStride { stride: 4, .. })
        ));
    }

    #[test]
    fn rejects_output_beyond_the_limit() {
        let mut raw = build(&[Chunk::Stored(b"abc")]);
        raw[FIRST_CHUNK + 4..FIRST_CHUNK + 8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(decode(&raw), Err(AafError::TooLarge)));
    }
}
