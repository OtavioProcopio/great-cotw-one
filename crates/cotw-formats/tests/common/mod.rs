//! Ajudantes dos testes com saves reais de `fixtures/saves` (pulam se a pasta não existir).

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    reason = "módulo só de testes; cada teste usa uma parte; contagens dos saves cabem em usize"
)]

use std::path::PathBuf;

use cotw_formats::adf::{Adf, ArrayReplacement, PointerField, PointerKind};
use cotw_formats::envelope::SaveEnvelope;

/// Um save real já aberto.
pub struct Save {
    pub name: String,
    pub raw: Vec<u8>,
    /// SHA-256 do conteúdo descomprimido, conforme o oráculo do Python.
    pub payload_sha256: String,
    pub envelope: SaveEnvelope,
    pub adf: Adf,
}

/// Todos os saves com ADF do índice, ou lista vazia se `fixtures/saves` não existir.
pub fn saves() -> Vec<Save> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    let Ok(index) = std::fs::read_to_string(dir.join("_index.oracle.txt")) else {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return Vec::new();
    };
    index
        .lines()
        .filter(|l| l.contains("\tadf\t"))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let raw = std::fs::read(dir.join(fields[0])).unwrap();
            let envelope = SaveEnvelope::parse(&raw).unwrap();
            let adf = Adf::parse(envelope.adf().to_vec()).unwrap();
            Save {
                name: fields[0].to_owned(),
                payload_sha256: fields[3].to_owned(),
                raw,
                envelope,
                adf,
            }
        })
        .collect()
}

/// (índice na lista, contagem, tamanho do elemento) dos arrays que o escritor aceita trocar:
/// não vazios e sem nenhum outro ponteiro dentro do trecho ou apontando para ele.
pub fn eligible(fields: &[PointerField], base: usize) -> Vec<(usize, u64, u32)> {
    fields
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let PointerKind::Array {
                count,
                element_size,
            } = f.kind
            else {
                return None;
            };
            if count == 0 || element_size == 0 {
                return None;
            }
            let start = f.target(base);
            let end = start + usize::try_from(count).ok()? * element_size as usize;
            let touched = fields.iter().enumerate().any(|(j, o)| {
                j != i
                    && ((start..end).contains(&o.position)
                        || (!o.is_null() && (start..end).contains(&o.target(base))))
            });
            (!touched).then_some((i, count, element_size))
        })
        .collect()
}

/// Bytes dos elementos de um array.
pub fn data_of(adf: &Adf, f: &PointerField, base: usize, count: u64, size: u32) -> Vec<u8> {
    let start = f.target(base);
    adf.as_bytes()[start..start + usize::try_from(count).unwrap() * size as usize].to_vec()
}

/// Troca de cada array elegível da instância pelos próprios elementos.
pub fn identity_edits(adf: &Adf, instance_index: usize) -> Vec<ArrayReplacement> {
    let instance = &adf.instances()[instance_index];
    let base = instance.offset as usize;
    let fields = adf.pointer_fields(instance).unwrap();
    eligible(&fields, base)
        .into_iter()
        .map(|(i, count, size)| ArrayReplacement {
            position: fields[i].position,
            elements: data_of(adf, &fields[i], base, count, size),
        })
        .collect()
}
