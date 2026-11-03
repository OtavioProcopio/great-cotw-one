//! O escritor ADF contra os saves reais de `fixtures/saves` (pula se ausentes).
//!
//! - a cadeia de relocação dos originais está íntegra (a regra da skill, aqui verificada);
//! - reescrever todos os arrays com os próprios elementos devolve os bytes originais;
//! - crescer arrays e depois reverter devolve os bytes originais, com a cadeia íntegra no meio.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    reason = "arquivo só de testes; contagens dos saves cabem em usize"
)]

mod common;

use common::{data_of, eligible, identity_edits, saves};
use cotw_formats::adf::{Adf, ArrayReplacement, PointerKind};

/// Quantas cópias do último elemento somar para a variação ser múltipla de 8 bytes.
fn copies_for(size: u32) -> usize {
    let mut k = 1;
    while !(k * size as usize).is_multiple_of(8) {
        k += 1;
    }
    k
}

#[test]
fn original_relocation_chains_are_intact() {
    for save in saves() {
        for instance in save.adf.instances() {
            assert_eq!(
                save.adf.chain_issues(instance).unwrap(),
                Vec::<usize>::new(),
                "{}/{}",
                save.name,
                instance.name
            );
            let fields = save.adf.pointer_fields(instance).unwrap();
            assert!(
                fields.iter().filter(|f| f.is_null()).all(|f| f.next == 0),
                "{}: ponteiro nulo com next != 0",
                save.name
            );
            if let Some(last) = fields.iter().rfind(|f| !f.is_null()) {
                assert_eq!(last.next, 0, "{}: último ponteiro com next != 0", save.name);
            }
        }
    }
}

#[test]
fn rewriting_every_array_with_its_own_elements_is_byte_identical() {
    let mut arrays = 0;
    for save in saves() {
        for (idx, instance) in save.adf.instances().iter().enumerate() {
            let edits = identity_edits(&save.adf, idx);
            arrays += edits.len();
            let out = save.adf.rewrite_arrays(idx, &edits).unwrap();
            assert!(
                out == save.adf.as_bytes(),
                "{}/{}: bytes diferem",
                save.name,
                instance.name
            );
        }
    }
    eprintln!("{arrays} arrays reescritos com identidade");
}

#[test]
fn growing_arrays_and_reverting_restores_the_original_bytes() {
    for save in saves() {
        for (idx, instance) in save.adf.instances().iter().enumerate() {
            let base = instance.offset as usize;
            let fields = save.adf.pointer_fields(instance).unwrap();
            let candidates = eligible(&fields, base);
            // Primeiro, do meio e último, sozinhos e depois todos juntos.
            let mut picks: Vec<_> = [0, candidates.len() / 2, candidates.len().saturating_sub(1)]
                .into_iter()
                .filter_map(|i| candidates.get(i).copied())
                .collect();
            picks.dedup(); // poucos candidatos repetem o mesmo índice
            let mut groups: Vec<Vec<_>> = picks.iter().map(|p| vec![*p]).collect();
            groups.push(picks.clone());
            for group in groups {
                grow_and_revert(&save.name, &save.adf, idx, &group);
            }
        }
    }
}

fn grow_and_revert(name: &str, adf: &Adf, idx: usize, group: &[(usize, u64, u32)]) {
    let base = adf.instances()[idx].offset as usize;
    let fields = adf.pointer_fields(&adf.instances()[idx]).unwrap();

    let mut originals = Vec::new();
    let mut grow = Vec::new();
    for &(i, count, size) in group {
        let old = data_of(adf, &fields[i], base, count, size);
        let mut new = old.clone();
        let last = old[old.len() - size as usize..].to_vec();
        for _ in 0..copies_for(size) {
            new.extend_from_slice(&last);
        }
        grow.push(ArrayReplacement {
            position: fields[i].position,
            elements: new,
        });
        originals.push(old);
    }
    if grow.is_empty() {
        return;
    }

    let grown_bytes = adf.rewrite_arrays(idx, &grow).unwrap();
    let grown = Adf::parse(grown_bytes).unwrap();
    let grown_instance = &grown.instances()[idx];
    assert_eq!(
        grown.chain_issues(grown_instance).unwrap(),
        Vec::<usize>::new(),
        "{name}: cadeia quebrada depois de crescer"
    );
    let grown_fields = grown.pointer_fields(grown_instance).unwrap();
    assert_eq!(
        grown_fields.len(),
        fields.len(),
        "{name}: ponteiros a mais ou a menos"
    );

    // Os arrays editados têm os novos elementos; reverter devolve o arquivo original.
    let mut revert = Vec::new();
    for ((&(i, _, size), edit), old) in group.iter().zip(&grow).zip(&originals) {
        let PointerKind::Array { count, .. } = grown_fields[i].kind else {
            panic!("{name}: deixou de ser array");
        };
        assert_eq!(
            count as usize,
            edit.elements.len() / size as usize,
            "{name}: contagem"
        );
        let now = data_of(&grown, &grown_fields[i], base, count, size);
        assert!(now == edit.elements, "{name}: elementos novos não conferem");
        revert.push(ArrayReplacement {
            position: grown_fields[i].position,
            elements: old.clone(),
        });
    }
    let back = grown.rewrite_arrays(idx, &revert).unwrap();
    assert!(
        back == adf.as_bytes(),
        "{name}: reverter não devolveu os bytes originais"
    );
}
