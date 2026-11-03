//! Ida e volta completa de cada save real (tarefa 5.1): ler o arquivo, reescrever o ADF sem
//! mudanças, regravar o envelope e reler. O conteúdo descomprimido tem de ser idêntico byte a
//! byte ao original; o cabeçalho de 32 bytes também. (O zlib recomprimido pode diferir do
//! original em bytes, o que a spec `save-file-reading` permite: ela fala do conteúdo
//! descomprimido.)

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

mod common;

use std::fmt::Write as _;

use common::{identity_edits, saves};
use cotw_formats::envelope::SaveEnvelope;
use sha2::{Digest, Sha256};

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data).iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

#[test]
fn every_known_save_round_trips_byte_for_byte() {
    let mut checked = 0;
    for save in saves() {
        // Reescreve cada instância sem mudanças.
        let mut adf_bytes = save.adf.as_bytes().to_vec();
        for idx in 0..save.adf.instances().len() {
            adf_bytes = save
                .adf
                .rewrite_arrays(idx, &identity_edits(&save.adf, idx))
                .unwrap();
            assert!(adf_bytes == save.adf.as_bytes(), "{}: ADF mudou", save.name);
        }

        // Regrava o envelope e relê.
        let mut envelope = save.envelope.clone();
        envelope.set_adf(adf_bytes);
        let rewritten = envelope.to_bytes().unwrap();
        let again = SaveEnvelope::parse(&rewritten).unwrap();

        assert!(again == save.envelope, "{}: conteúdo mudou", save.name);
        assert!(
            rewritten[..32] == save.raw[..32],
            "{}: cabeçalho mudou",
            save.name
        );
        // Mesmo cabeçalho zlib do jogo (nível de compressão).
        assert!(
            rewritten[32..34] == save.raw[32..34],
            "{}: cabeçalho zlib difere do original",
            save.name
        );
        let mut payload = again.prefix().to_vec();
        payload.extend_from_slice(again.adf());
        assert_eq!(
            sha256_hex(&payload),
            save.payload_sha256,
            "{}: descomprimido difere do original (Python)",
            save.name
        );
        checked += 1;
    }
    eprintln!("{checked} saves conferidos");
}
