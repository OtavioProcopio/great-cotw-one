use super::test_support::{self as ts, InstanceSpec, name};
use super::*;

fn parse_sample() -> Adf {
    Adf::parse(ts::sample().bytes).unwrap()
}

fn field<'a>(value: &'a Value, wanted: &str) -> &'a Value {
    let Value::Struct(fields) = value else {
        panic!("esperava struct, veio {value:?}");
    };
    &fields.iter().find(|(n, _)| n == wanted).unwrap().1
}

#[test]
fn parses_header_comment_and_instances() {
    let adf = parse_sample();
    assert_eq!(adf.version(), 1);
    assert_eq!(adf.comment(), "comentario de teste");
    let names: Vec<_> = adf.instances().iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["Root", "misc"]);
    assert_eq!(adf.instances()[0].type_hash, ts::ROOT);
    assert_eq!(adf.instances()[0].size, 85);
    assert_eq!(adf.as_bytes().len(), ts::sample().bytes.len());
}

#[test]
fn parses_struct_members_and_splits_bit_from_offset() {
    let adf = parse_sample();
    let root = adf.type_def(ts::ROOT).unwrap();
    assert_eq!(root.kind, TypeKind::Struct);
    assert_eq!(root.name, "Root");
    assert_eq!(root.size, 56);
    let items = &root.members[1];
    assert_eq!(
        (items.name.as_str(), items.offset, items.bit),
        ("items", 8, 0)
    );

    let prims = adf.type_def(ts::PRIMS).unwrap();
    let last = prims.members.last().unwrap();
    assert_eq!((last.offset, last.bit), (32, 3));
}

#[test]
fn parses_enum_array_and_bitfield_typedefs() {
    let adf = parse_sample();
    let mode = adf.type_def(ts::ENUM).unwrap();
    assert_eq!(mode.kind, TypeKind::Enum);
    assert_eq!(mode.enum_values[1].name, "Slow");
    let array = adf.type_def(ts::ARRAY).unwrap();
    assert_eq!((array.kind, array.element_hash), (TypeKind::Array, ts::U32));
    assert_eq!(adf.type_def(ts::BITS).unwrap().kind, TypeKind::BitField);
    assert_eq!(adf.types().count(), 7);
    assert!(adf.type_def(0x9999).is_none());
}

#[test]
fn decodes_every_composite_kind() {
    let adf = parse_sample();
    let root = adf.read_instance(&adf.instances()[0]).unwrap();

    assert_eq!(field(&root, "count"), &Value::Unsigned(3));
    assert_eq!(
        field(&root, "items"),
        &Value::Array(vec![
            Value::Unsigned(10),
            Value::Unsigned(20),
            Value::Unsigned(30)
        ])
    );
    assert_eq!(
        field(&root, "mode"),
        &Value::Enum {
            value: 2,
            name: Some("Slow".into())
        }
    );
    assert_eq!(
        field(&root, "tag"),
        &Value::StringHash {
            hash: 0xAAAA,
            text: Some("alpha".into())
        }
    );
    assert_eq!(field(&root, "label"), &Value::String("hola".into()));
    assert_eq!(field(&root, "bytes"), &Value::Bytes(b"abcd".to_vec()));
    assert_eq!(
        field(&root, "unknown"),
        &Value::Raw {
            type_hash: 0xDEAD_BEEF,
            bytes: vec![4, 3, 2, 1]
        }
    );
}

#[test]
fn decodes_every_primitive() {
    let adf = parse_sample();
    let Value::Struct(fields) = adf.read_instance(&adf.instances()[1]).unwrap() else {
        panic!("esperava struct");
    };
    let values: Vec<_> = fields.into_iter().map(|(_, v)| v).collect();
    assert_eq!(
        values,
        [
            Value::Signed(-1),
            Value::Signed(-2),
            Value::Signed(-3),
            Value::Signed(-4),
            Value::Float(1.5),
            Value::Float(2.5),
            Value::Unsigned(7),
            Value::Unsigned(9),
            Value::Unsigned(11),
            Value::Unsigned(7),
        ]
    );
}

#[test]
fn unresolved_enum_and_string_hash_keep_raw_numbers() {
    let mut sample = ts::sample();
    let base = sample.data_offsets[0];
    sample.bytes[base + 24..base + 28].copy_from_slice(&99u32.to_le_bytes());
    sample.bytes[base + 32..base + 40].copy_from_slice(&0xBBBBu64.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    let root = adf.read_instance(&adf.instances()[0]).unwrap();
    assert_eq!(
        field(&root, "mode"),
        &Value::Enum {
            value: 99,
            name: None
        }
    );
    assert_eq!(
        field(&root, "tag"),
        &Value::StringHash {
            hash: 0xBBBB,
            text: None
        }
    );
    assert_eq!(adf.string_for_hash(0xAAAA), Some("alpha"));
}

#[test]
fn null_string_pointer_is_empty_string() {
    let mut sample = ts::sample();
    let base = sample.data_offsets[0];
    sample.bytes[base + 40..base + 44].copy_from_slice(&0u32.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    let root = adf.read_instance(&adf.instances()[0]).unwrap();
    assert_eq!(field(&root, "label"), &Value::String(String::new()));
}

#[test]
fn file_without_string_hash_table_parses() {
    let types = [ts::struct_type(
        ts::ROOT,
        4,
        name::ROOT,
        &[ts::member(name::COUNT, ts::U32, 4, 0)],
    )];
    let assembled = ts::assemble(
        &[],
        &types,
        &[InstanceSpec {
            type_hash: ts::ROOT,
            name_idx: name::ROOT,
            data: 5u32.to_le_bytes().to_vec(),
        }],
    );
    let adf = Adf::parse(assembled.bytes).unwrap();
    let root = adf.read_instance(&adf.instances()[0]).unwrap();
    assert_eq!(field(&root, "count"), &Value::Unsigned(5));
}

#[test]
fn rejects_missing_signature() {
    let mut bytes = ts::sample().bytes;
    bytes[0] = b'X';
    assert_eq!(Adf::parse(bytes).unwrap_err(), AdfError::BadMagic);
}

#[test]
fn rejects_out_of_range_name_index() {
    let mut sample = ts::sample();
    // O nome da primeira instância é o u64 nos últimos 8 bytes da entrada de 24 bytes.
    let entry = sample.data_offsets[0] - 48;
    sample.bytes[entry + 16..entry + 24].copy_from_slice(&999u64.to_le_bytes());
    assert_eq!(
        Adf::parse(sample.bytes).unwrap_err(),
        AdfError::BadNameIndex(999)
    );
}

#[test]
fn absurd_array_count_is_an_error_not_an_allocation() {
    let mut sample = ts::sample();
    let base = sample.data_offsets[0];
    sample.bytes[base + 16..base + 24].copy_from_slice(&u64::MAX.to_le_bytes());
    let adf = Adf::parse(sample.bytes).unwrap();
    assert!(matches!(
        adf.read_instance(&adf.instances()[0]),
        Err(AdfError::OutOfBounds { .. })
    ));
}

#[test]
fn self_referencing_struct_stops_at_depth_limit() {
    let types = [ts::struct_type(
        ts::LOOP,
        8,
        name::LOOP,
        &[ts::member(name::LOOP, ts::LOOP, 8, 0)],
    )];
    let assembled = ts::assemble(
        &[],
        &types,
        &[InstanceSpec {
            type_hash: ts::LOOP,
            name_idx: name::LOOP,
            data: vec![0; 8],
        }],
    );
    let adf = Adf::parse(assembled.bytes).unwrap();
    assert_eq!(
        adf.read_instance(&adf.instances()[0]).unwrap_err(),
        AdfError::TooDeep
    );
}

#[test]
fn truncated_files_never_panic() {
    let bytes = ts::sample().bytes;
    for len in 0..bytes.len() {
        let Ok(adf) = Adf::parse(bytes[..len].to_vec()) else {
            continue;
        };
        for instance in adf.instances() {
            let _ = adf.read_instance(instance);
        }
    }
}
