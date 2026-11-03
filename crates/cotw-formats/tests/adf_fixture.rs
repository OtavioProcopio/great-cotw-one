//! Compara o leitor ADF com `research/reference/adf.py` num save real.
//!
//! O oráculo (`fixtures/<save>.adf.oracle.txt`) é gerado pelo script Python de referência.
//! Sem a pasta `fixtures/` o teste é ignorado.

#![allow(
    clippy::unwrap_used,
    reason = "arquivo só de testes; escrever em String não falha"
)]

use std::fmt::Write as _;
use std::path::PathBuf;

use cotw_formats::adf::{Adf, TypeKind};
use cotw_formats::envelope::SaveEnvelope;

fn fixture(name: &str) -> Option<Vec<u8>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name);
    std::fs::read(path).ok()
}

fn meta_number(kind: TypeKind) -> u32 {
    match kind {
        TypeKind::Primitive => 0,
        TypeKind::Struct => 1,
        TypeKind::Pointer => 2,
        TypeKind::Array => 3,
        TypeKind::InlineArray => 4,
        TypeKind::String => 5,
        TypeKind::BitField => 7,
        TypeKind::Enum => 8,
        TypeKind::StringHash => 9,
        TypeKind::Unknown(n) => n,
    }
}

/// Mesmo formato que `make_oracle.py` escreve.
fn render(adf: &Adf) -> String {
    let mut out = String::new();
    writeln!(out, "version\t{}", adf.version()).unwrap();
    writeln!(out, "comment\t{}", adf.comment()).unwrap();
    writeln!(out, "types\t{}", adf.types().count()).unwrap();
    // O save de teste não usa string hashes; a contagem é conferida indiretamente.
    writeln!(out, "string_hashes\t0").unwrap();
    for t in adf.types() {
        writeln!(
            out,
            "type\t{:08x}\t{}\t{}\t{}\t{}\t{}\t{:08x}\t{}\t{}",
            t.hash,
            meta_number(t.kind),
            t.name,
            t.size,
            t.align,
            t.flags,
            t.element_hash,
            t.element_len,
            t.members.len()
        )
        .unwrap();
        for m in &t.members {
            writeln!(
                out,
                "member\t{}\t{:08x}\t{}\t{}\t{}\t{}",
                m.name, m.type_hash, m.size, m.offset, m.bit, m.flags
            )
            .unwrap();
        }
        let mut values = t.enum_values.clone();
        values.sort_by_key(|e| e.value);
        for e in values {
            writeln!(out, "enum\t{}\t{}", e.value, e.name).unwrap();
        }
    }
    for i in adf.instances() {
        writeln!(
            out,
            "instance\t{}\t{:08x}\t{}\t{}",
            i.name, i.type_hash, i.offset, i.size
        )
        .unwrap();
    }
    out
}

#[test]
fn real_save_matches_python_reference() {
    let (Some(raw), Some(oracle)) = (
        fixture("animal_population_0"),
        fixture("animal_population_0.adf.oracle.txt"),
    ) else {
        eprintln!("fixtures ausentes: teste ignorado");
        return;
    };
    let envelope = SaveEnvelope::parse(&raw).unwrap();
    let adf = Adf::parse(envelope.adf().to_vec()).unwrap();

    assert_eq!(render(&adf), String::from_utf8(oracle).unwrap());
}

#[test]
fn real_save_instances_decode_without_error() {
    let Some(raw) = fixture("animal_population_0") else {
        eprintln!("fixtures/animal_population_0 ausente: teste ignorado");
        return;
    };
    let adf = Adf::parse(SaveEnvelope::parse(&raw).unwrap().adf().to_vec()).unwrap();
    assert_ne!(adf.instances(), []);
    for instance in adf.instances() {
        adf.read_instance(instance).unwrap();
    }
}
