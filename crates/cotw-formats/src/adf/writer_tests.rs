use super::test_support::{self as ts, Assembled};
use super::*;

/// Cabeçalho do array `items` do `Root` do exemplo (posição absoluta).
fn items_header(sample: &Assembled) -> usize {
    sample.data_offsets[0] + 8
}

fn u32s(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn edit(position: usize, values: &[u32]) -> ArrayReplacement {
    ArrayReplacement {
        position,
        elements: u32s(values),
    }
}

fn items_of(adf: &Adf) -> Value {
    let root = adf.read_instance(&adf.instances()[0]).unwrap();
    let Value::Struct(fields) = root else {
        panic!("esperava struct");
    };
    fields.into_iter().find(|(n, _)| n == "items").unwrap().1
}

fn unsigned(values: &[u64]) -> Value {
    Value::Array(values.iter().map(|&v| Value::Unsigned(v)).collect())
}

#[test]
fn sample_chain_is_consistent_before_editing() {
    let adf = Adf::parse(ts::sample().bytes).unwrap();
    assert_eq!(
        adf.chain_issues(&adf.instances()[0]).unwrap(),
        Vec::<usize>::new()
    );
}

#[test]
fn identity_edit_reproduces_the_original_bytes() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    let out = adf
        .rewrite_arrays(0, &[edit(items_header(&sample), &[10, 20, 30])])
        .unwrap();
    assert_eq!(out, sample.bytes);
}

#[test]
fn no_edits_reproduces_the_original_bytes() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    assert_eq!(adf.rewrite_arrays(0, &[]).unwrap(), sample.bytes);
}

#[test]
fn growing_an_array_shifts_everything_after_it() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    let out = adf
        .rewrite_arrays(0, &[edit(items_header(&sample), &[10, 20, 30, 40, 50])])
        .unwrap();
    assert_eq!(out.len(), sample.bytes.len() + 8);

    let grown = Adf::parse(out).unwrap();
    assert_eq!(items_of(&grown), unsigned(&[10, 20, 30, 40, 50]));
    // A string que ficava depois do array continua legível, assim como a outra instância.
    let root = grown.read_instance(&grown.instances()[0]).unwrap();
    assert!(
        matches!(&root, Value::Struct(f) if f.iter().any(|(n, v)| n == "label" && *v == Value::String("hola".into())))
    );
    assert_eq!(
        grown.read_instance(&grown.instances()[1]).unwrap(),
        adf.read_instance(&adf.instances()[1]).unwrap()
    );
    assert_eq!(grown.instances()[0].size, adf.instances()[0].size + 8);
    assert_eq!(
        grown.chain_issues(&grown.instances()[0]).unwrap(),
        Vec::<usize>::new()
    );
}

#[test]
fn shrinking_an_array_works_and_reverting_restores_the_original() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    let small = adf
        .rewrite_arrays(0, &[edit(items_header(&sample), &[10])])
        .unwrap();
    assert_eq!(small.len(), sample.bytes.len() - 8);
    let small_adf = Adf::parse(small).unwrap();
    assert_eq!(items_of(&small_adf), unsigned(&[10]));

    let back = small_adf
        .rewrite_arrays(0, &[edit(items_header(&sample), &[10, 20, 30])])
        .unwrap();
    assert_eq!(back, sample.bytes);
}

#[test]
fn header_offsets_after_the_instance_are_shifted() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    let out = adf
        .rewrite_arrays(0, &[edit(items_header(&sample), &[1, 2, 3, 4, 5])])
        .unwrap();
    let total = |b: &[u8]| u32::from_le_bytes([b[40], b[41], b[42], b[43]]);
    assert_eq!(total(&out), total(&sample.bytes) + 8);
    // A segunda instância (Prims) vem depois da primeira e foi empurrada 8 bytes.
    let moved = Adf::parse(out).unwrap();
    assert_eq!(moved.instances()[1].offset, adf.instances()[1].offset + 8);
}

#[test]
fn rejects_unsupported_requests_without_writing() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes.clone()).unwrap();
    let header = items_header(&sample);

    assert!(matches!(
        adf.rewrite_arrays(9, &[]),
        Err(WriteError::NoSuchInstance(9))
    ));
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(sample.data_offsets[0], &[1, 2])]),
        Err(WriteError::NotAnArray { .. })
    ));
    assert!(matches!(
        adf.rewrite_arrays(
            0,
            &[ArrayReplacement {
                position: header,
                elements: vec![1, 2, 3]
            }]
        ),
        Err(WriteError::BadElementLength { len: 3, .. })
    ));
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(header, &[1, 2, 3, 4])]),
        Err(WriteError::Misaligned { delta: 4, .. })
    ));
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(header, &[1]), edit(header, &[2])]),
        Err(WriteError::Duplicate { .. })
    ));
}

#[test]
fn rejects_emptying_an_array_that_would_misalign() {
    let sample = ts::sample();
    let header = items_header(&sample);
    let adf = Adf::parse(sample.bytes).unwrap();
    // 12 bytes -> 0: variação de -12, não múltipla de 8.
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(header, &[])]),
        Err(WriteError::Misaligned { delta: -12, .. })
    ));
}

#[test]
fn rejects_growing_from_empty() {
    let mut sample = ts::sample();
    let header = items_header(&sample);
    sample.bytes[header + 8..header + 16].copy_from_slice(&0u64.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(header, &[1, 2])]),
        Err(WriteError::EmptyArray { .. })
    ));
}

#[test]
fn rejects_edit_when_another_pointer_targets_the_replaced_range() {
    let mut sample = ts::sample();
    // Faz o `label` apontar para dentro dos elementos de `items`.
    let label = sample.data_offsets[0] + 40;
    sample.bytes[label..label + 4].copy_from_slice(&66u32.to_le_bytes());
    let header = items_header(&sample);
    let adf = Adf::parse(sample.bytes).unwrap();
    assert!(matches!(
        adf.rewrite_arrays(0, &[edit(header, &[1, 2, 3, 4, 5])]),
        Err(WriteError::NestedPointers { .. })
    ));
}

#[test]
fn pointer_map_lists_the_array_and_the_string_in_file_order() {
    let sample = ts::sample();
    let adf = Adf::parse(sample.bytes).unwrap();
    let fields = adf.pointer_fields(&adf.instances()[0]).unwrap();
    let base = adf.instances()[0].offset as usize;
    let summary: Vec<_> = fields
        .iter()
        .map(|f| (f.position - base, f.offset, f.next))
        .collect();
    assert_eq!(summary, [(8, 64, 32), (40, 80, 0)]);
    assert!(matches!(
        fields[0].kind,
        PointerKind::Array {
            count: 3,
            element_size: 4
        }
    ));
    assert_eq!(fields[1].kind, PointerKind::String);
    assert_eq!(fields[0].target(base), base + 64);
}

#[test]
fn broken_chain_is_reported() {
    let mut sample = ts::sample();
    let header = items_header(&sample);
    sample.bytes[header + 4..header + 8].copy_from_slice(&7u32.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    assert_eq!(adf.chain_issues(&adf.instances()[0]).unwrap(), [header]);
}

#[test]
fn rewriting_repairs_a_broken_chain_and_stale_next_of_the_last_pointer() {
    let mut sample = ts::sample();
    let header = items_header(&sample);
    sample.bytes[header + 4..header + 8].copy_from_slice(&7u32.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    let fixed = adf
        .rewrite_arrays(0, &[edit(header, &[10, 20, 30])])
        .unwrap();
    let fixed = Adf::parse(fixed).unwrap();
    assert_eq!(
        fixed.chain_issues(&fixed.instances()[0]).unwrap(),
        Vec::<usize>::new()
    );
}
