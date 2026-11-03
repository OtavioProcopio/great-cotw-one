//! Ajudantes compartilhados pelos testes unitários do crate.

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;

/// Arquivo de save mínimo e válido: envelope + ADF sem tipos nem instâncias.
pub(crate) fn valid_save_bytes() -> Vec<u8> {
    let mut adf = vec![0u8; 0x40];
    adf[0..4].copy_from_slice(b" FDA");
    adf[4..8].copy_from_slice(&4u32.to_le_bytes());
    let mut payload = vec![1, 1, 0, 0, 0];
    payload.extend(adf);
    let size = u32::try_from(payload.len()).unwrap().to_le_bytes();
    let mut header = [0u8; 32];
    header[0..4].copy_from_slice(b"SAVE");
    header[16..20].copy_from_slice(b"COMP");
    header[8..12].copy_from_slice(&size);
    header[24..28].copy_from_slice(&size);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload).unwrap();
    let mut out = header.to_vec();
    out.extend(encoder.finish().unwrap());
    out
}
