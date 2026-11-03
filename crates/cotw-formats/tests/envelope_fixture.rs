//! Teste de ida e volta com um save real. Pula se `fixtures/` não existir
//! (a pasta está no `.gitignore`: saves nunca vão para o repositório).

use std::path::PathBuf;

use cotw_formats::envelope::SaveEnvelope;

fn fixture(name: &str) -> Option<Vec<u8>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name);
    std::fs::read(path).ok()
}

#[test]
fn real_save_round_trips_with_identical_content() {
    let Some(raw) = fixture("animal_population_0") else {
        eprintln!("fixtures/animal_population_0 ausente: teste ignorado");
        return;
    };
    let envelope = SaveEnvelope::parse(&raw).unwrap();
    assert_eq!(&envelope.adf()[..4], b" FDA", "o conteúdo deve ser um ADF");

    let rewritten = envelope.to_bytes().unwrap();
    // Cabeçalho idêntico: os tamanhos não mudaram.
    assert_eq!(rewritten[..32], raw[..32]);
    // O zlib pode diferir do original, mas o conteúdo descomprimido não.
    assert_eq!(SaveEnvelope::parse(&rewritten).unwrap(), envelope);
}
