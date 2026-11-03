//! Montador de ADFs sintéticos para os testes: dispensa saves reais.

pub(super) const U8: u32 = 0x0CA2_821D;
pub(super) const S8: u32 = 0x580D_0A62;
pub(super) const U16: u32 = 0x86D1_52BD;
pub(super) const S16: u32 = 0xD13F_CF93;
pub(super) const U32: u32 = 0x075E_4E4F;
pub(super) const S32: u32 = 0x192F_E633;
pub(super) const U64: u32 = 0xA139_E01F;
pub(super) const S64: u32 = 0xAF41_354F;
pub(super) const F32: u32 = 0x7515_A207;
pub(super) const F64: u32 = 0xC609_F663;
pub(super) const STRING: u32 = 0x8955_583E;

pub(super) const ROOT: u32 = 0x1111;
pub(super) const ARRAY: u32 = 0x2222;
pub(super) const ENUM: u32 = 0x3333;
pub(super) const TAG: u32 = 0x4444;
pub(super) const BYTES4: u32 = 0x5555;
pub(super) const BITS: u32 = 0x6666;
pub(super) const LOOP: u32 = 0x7777;
pub(super) const PRIMS: u32 = 0x8888;

/// Índices na tabela de nomes (ordem de [`NAMES`]).
pub(super) mod name {
    pub(in super::super) const ROOT: u64 = 0;
    pub(in super::super) const COUNT: u64 = 1;
    pub(in super::super) const ITEMS: u64 = 2;
    pub(in super::super) const MODE: u64 = 3;
    pub(in super::super) const FAST: u64 = 4;
    pub(in super::super) const SLOW: u64 = 5;
    pub(in super::super) const TAG: u64 = 6;
    pub(in super::super) const LABEL: u64 = 7;
    pub(in super::super) const BYTES: u64 = 8;
    pub(in super::super) const UNKNOWN: u64 = 9;
    pub(in super::super) const MISC: u64 = 10;
    pub(in super::super) const LOOP: u64 = 11;
}

pub(super) const NAMES: [&str; 12] = [
    "Root", "count", "items", "mode", "Fast", "Slow", "tag", "label", "bytes", "unknown", "misc",
    "loop",
];

fn u32_len(n: usize) -> u32 {
    u32::try_from(n).unwrap()
}

pub(super) fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(super) fn put_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Typedef serializado; `children` já vem codificado e `child_count` é o número de filhos.
#[allow(clippy::too_many_arguments)]
pub(super) fn typedef(
    meta: u32,
    size: u32,
    hash: u32,
    name_idx: u64,
    element_hash: u32,
    element_len: u32,
    child_count: u32,
    children: &[u8],
) -> Vec<u8> {
    let mut b = Vec::new();
    put_u32(&mut b, meta);
    put_u32(&mut b, size);
    put_u32(&mut b, 8); // alinhamento
    put_u32(&mut b, hash);
    put_u64(&mut b, name_idx);
    put_u32(&mut b, 0); // flags
    put_u32(&mut b, element_hash);
    put_u32(&mut b, element_len);
    put_u32(&mut b, child_count);
    b.extend_from_slice(children);
    b
}

/// Membro de struct (32 bytes); `packed_offset` leva o bit nos 8 bits altos.
pub(super) fn member(name_idx: u64, type_hash: u32, size: u32, packed_offset: u32) -> Vec<u8> {
    let mut b = Vec::new();
    put_u64(&mut b, name_idx);
    put_u32(&mut b, type_hash);
    put_u32(&mut b, size);
    put_u32(&mut b, packed_offset);
    put_u32(&mut b, 0); // flags
    put_u64(&mut b, 0); // valor padrão
    b
}

pub(super) fn struct_type(hash: u32, size: u32, name_idx: u64, members: &[Vec<u8>]) -> Vec<u8> {
    typedef(
        1,
        size,
        hash,
        name_idx,
        0,
        0,
        u32_len(members.len()),
        &members.concat(),
    )
}

/// Instância a montar: dados são colocados em sequência no fim do arquivo.
pub(super) struct InstanceSpec {
    pub type_hash: u32,
    pub name_idx: u64,
    pub data: Vec<u8>,
}

/// Resultado da montagem: os bytes e onde cada instância caiu.
pub(super) struct Assembled {
    pub bytes: Vec<u8>,
    pub data_offsets: Vec<usize>,
}

/// Monta um ADF completo: cabeçalho, comentário, nomes, string hashes, typedefs,
/// tabela de instâncias e dados.
pub(super) fn assemble(
    string_hashes: &[(&str, u64)],
    types: &[Vec<u8>],
    instances: &[InstanceSpec],
) -> Assembled {
    let mut buf = vec![0u8; 0x40];
    buf[0..4].copy_from_slice(b" FDA");
    buf.extend_from_slice(b"comentario de teste\0");

    let name_offset = buf.len();
    for n in NAMES {
        buf.push(u8::try_from(n.len()).unwrap());
    }
    for n in NAMES {
        buf.extend_from_slice(n.as_bytes());
        buf.push(0);
    }

    let hash_offset = buf.len();
    if !string_hashes.is_empty() {
        for (text, _) in string_hashes {
            buf.extend_from_slice(text.as_bytes());
            buf.push(0);
        }
        for (_, hash) in string_hashes {
            put_u64(&mut buf, *hash);
        }
    }

    let type_offset = buf.len();
    for t in types {
        buf.extend_from_slice(t);
    }

    let instance_offset = buf.len();
    let mut data_offset = instance_offset + 24 * instances.len();
    let mut data_offsets = Vec::new();
    let mut tail = Vec::new();
    for spec in instances {
        data_offsets.push(data_offset);
        put_u32(&mut buf, 0); // hash do nome (não usado pelo leitor)
        put_u32(&mut buf, spec.type_hash);
        put_u32(&mut buf, u32_len(data_offset));
        put_u32(&mut buf, u32_len(spec.data.len()));
        put_u64(&mut buf, spec.name_idx);
        tail.extend_from_slice(&spec.data);
        data_offset += spec.data.len();
    }
    buf.extend_from_slice(&tail);

    let header = [
        1, // versão
        u32_len(instances.len()),
        u32_len(instance_offset),
        u32_len(types.len()),
        u32_len(type_offset),
        u32_len(string_hashes.len()),
        u32_len(hash_offset),
        u32_len(NAMES.len()),
        u32_len(name_offset),
        u32_len(buf.len()),
    ];
    for (i, v) in header.iter().enumerate() {
        buf[4 + 4 * i..8 + 4 * i].copy_from_slice(&v.to_le_bytes());
    }
    Assembled {
        bytes: buf,
        data_offsets,
    }
}

/// Instância `Root` com um exemplo de cada tipo composto.
fn root_data() -> Vec<u8> {
    let mut d = Vec::new();
    put_u32(&mut d, 3); // @0 count
    put_u32(&mut d, 0); // @4 (livre)
    put_u32(&mut d, 64); // @8 items: offset relativo
    put_u32(&mut d, 32); // @12 elo da cadeia: distância até o ponteiro seguinte (label, @40)
    put_u64(&mut d, 3); // @16 contagem
    put_u64(&mut d, 0); // @24 -> enum ocupa os 4 primeiros bytes
    d[24..28].copy_from_slice(&2u32.to_le_bytes()); // mode = Slow
    put_u64(&mut d, 0xAAAA); // @32 tag
    put_u32(&mut d, 80); // @40 label: offset relativo
    put_u32(&mut d, 0);
    d.extend_from_slice(b"abcd"); // @48 bytes inline
    put_u32(&mut d, 0x0102_0304); // @52 tipo desconhecido
    d.resize(64, 0);
    for v in [10u32, 20, 30] {
        put_u32(&mut d, v); // @64 elementos do array
    }
    d.resize(80, 0);
    d.extend_from_slice(b"hola\0"); // @80 texto
    d
}

/// Instância `Prims` com um de cada primitivo.
fn prims_data() -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&(-1i8).to_le_bytes()); // @0 s8
    d.push(0);
    d.extend_from_slice(&(-2i16).to_le_bytes()); // @2 s16
    d.extend_from_slice(&(-3i32).to_le_bytes()); // @4 s32
    d.extend_from_slice(&(-4i64).to_le_bytes()); // @8 s64
    d.extend_from_slice(&1.5f32.to_le_bytes()); // @16 f32
    d.extend_from_slice(&[0; 4]);
    d.extend_from_slice(&2.5f64.to_le_bytes()); // @24 f64
    d.push(7); // @32 u8
    d.push(0);
    d.extend_from_slice(&9u16.to_le_bytes()); // @34 u16
    d.extend_from_slice(&[0; 4]);
    d.extend_from_slice(&11u64.to_le_bytes()); // @40 u64
    d
}

/// ADF de exemplo com duas instâncias (`Root` e `Prims`).
pub(super) fn sample() -> Assembled {
    let mut enum_values = Vec::new();
    put_u64(&mut enum_values, name::FAST);
    put_u32(&mut enum_values, 1);
    put_u64(&mut enum_values, name::SLOW);
    put_u32(&mut enum_values, 2);

    let mut bits = Vec::new();
    bits.extend_from_slice(&[0; 12]);

    let types = [
        struct_type(
            ROOT,
            56,
            name::ROOT,
            &[
                member(name::COUNT, U32, 4, 0),
                member(name::ITEMS, ARRAY, 16, 8),
                member(name::MODE, ENUM, 4, 24),
                member(name::TAG, TAG, 8, 32),
                member(name::LABEL, STRING, 8, 40),
                member(name::BYTES, BYTES4, 4, 48),
                member(name::UNKNOWN, 0xDEAD_BEEF, 4, 52),
            ],
        ),
        typedef(3, 16, ARRAY, name::ITEMS, U32, 0, 0, &[]),
        typedef(8, 4, ENUM, name::MODE, 0, 0, 2, &enum_values),
        typedef(9, 8, TAG, name::TAG, 0, 0, 0, &[]),
        typedef(4, 4, BYTES4, name::BYTES, U8, 4, 0, &[]),
        typedef(7, 4, BITS, name::MISC, 0, 0, 1, &bits),
        struct_type(
            PRIMS,
            48,
            name::MISC,
            &[
                member(name::MISC, S8, 1, 0),
                member(name::MISC, S16, 2, 2),
                member(name::MISC, S32, 4, 4),
                member(name::MISC, S64, 8, 8),
                member(name::MISC, F32, 4, 16),
                member(name::MISC, F64, 8, 24),
                member(name::MISC, U8, 1, 32),
                member(name::MISC, U16, 2, 34),
                member(name::MISC, U64, 8, 40),
                // bit 3 no byte alto do offset: o leitor deve separá-lo do deslocamento
                member(name::MISC, U8, 1, (3 << 24) | 32),
            ],
        ),
    ];
    assemble(
        &[("alpha", 0xAAAA)],
        &types,
        &[
            InstanceSpec {
                type_hash: ROOT,
                name_idx: name::ROOT,
                data: root_data(),
            },
            InstanceSpec {
                type_hash: PRIMS,
                name_idx: name::MISC,
                data: prims_data(),
            },
        ],
    )
}
