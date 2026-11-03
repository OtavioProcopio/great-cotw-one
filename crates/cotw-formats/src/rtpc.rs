//! RTPC (`.blo`, `settings/player.bin`): árvore de nós com propriedades tipadas.
//!
//! ```text
//! @0   "RTPC"
//! @8   nó raiz: u32 hash, u32 offset das props, u16 nº props, u16 nº filhos (12 bytes)
//!      props (9 bytes cada): u32 hash, 4 bytes brutos, u8 tipo; depois, alinhado a 4,
//!      os filhos (nós de 12 bytes em sequência)
//! ```

use crate::bytes;

const ROOT_OFFSET: usize = 8;
const NODE_LEN: usize = 12;
const PROP_LEN: usize = 9;
const MAX_DEPTH: usize = 64;
/// Teto de nós por arquivo: impede que dados corrompidos criem uma árvore explosiva.
const MAX_NODES: usize = 1_000_000;

/// Erros ao ler um RTPC.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RtpcError {
    /// Não começa com `RTPC`.
    #[error("assinatura RTPC ausente")]
    BadMagic,
    /// Algum campo passa do fim dos dados.
    #[error("RTPC truncado (offset {offset})")]
    Truncated {
        /// Posição onde faltou dado.
        offset: usize,
    },
    /// Árvore profunda demais ou com nós demais.
    #[error("árvore RTPC grande ou funda demais")]
    TooBig,
}

/// Valor de uma propriedade.
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    /// Tipo 1.
    U32(u32),
    /// Tipo 2.
    F32(f32),
    /// Tipo 3: texto UTF-8.
    String(String),
    /// Tipos 4 a 8: vetor ou matriz (2, 3, 4, 9 ou 16 floats).
    Floats(Vec<f32>),
    /// Tipo 9.
    U32Array(Vec<u32>),
    /// Tipo 10.
    F32Array(Vec<f32>),
    /// Tipo 11.
    Bytes(Vec<u8>),
    /// Tipo 13.
    U64(u64),
    /// Tipo 14: pares de ids de objeto.
    ObjectIds(Vec<(u32, u32)>),
    /// Tipo não reconhecido: os 4 bytes brutos.
    Raw {
        /// Número do tipo.
        type_id: u8,
        /// Bytes brutos.
        bytes: [u8; 4],
    },
}

/// Propriedade de um nó.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// Hash do nome da propriedade.
    pub hash: u32,
    /// Valor decodificado.
    pub value: PropValue,
}

/// Nó da árvore.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Hash do nome do nó.
    pub hash: u32,
    /// Propriedades, na ordem do arquivo.
    pub props: Vec<Property>,
    /// Nós filhos.
    pub children: Vec<Node>,
}

/// Lê um RTPC inteiro e devolve o nó raiz.
///
/// # Errors
///
/// Retorna [`RtpcError`] se a assinatura estiver errada, algum campo for truncado ou a
/// árvore ultrapassar os limites.
pub fn parse(data: &[u8]) -> Result<Node, RtpcError> {
    if !data.starts_with(b"RTPC") {
        return Err(RtpcError::BadMagic);
    }
    let mut budget = MAX_NODES;
    read_node(data, ROOT_OFFSET, 0, &mut budget)
}

fn read_node(
    data: &[u8],
    offset: usize,
    depth: usize,
    budget: &mut usize,
) -> Result<Node, RtpcError> {
    if depth > MAX_DEPTH {
        return Err(RtpcError::TooBig);
    }
    *budget = budget.checked_sub(1).ok_or(RtpcError::TooBig)?;
    let truncated = |offset| RtpcError::Truncated { offset };

    let hash = bytes::u32_at(data, offset).ok_or(truncated(offset))?;
    let props_at = bytes::u32_at(data, offset + 4).ok_or(truncated(offset + 4))? as usize;
    let prop_count = usize::from(bytes::u16_at(data, offset + 8).ok_or(truncated(offset + 8))?);
    let child_count = usize::from(bytes::u16_at(data, offset + 10).ok_or(truncated(offset + 10))?);

    let mut props = Vec::with_capacity(prop_count);
    for i in 0..prop_count {
        props.push(read_prop(data, props_at + i * PROP_LEN)?);
    }
    let children_at = (props_at + prop_count * PROP_LEN + 3) & !3;
    let mut children = Vec::with_capacity(child_count);
    for i in 0..child_count {
        children.push(read_node(
            data,
            children_at + i * NODE_LEN,
            depth + 1,
            budget,
        )?);
    }
    Ok(Node {
        hash,
        props,
        children,
    })
}

fn read_prop(data: &[u8], offset: usize) -> Result<Property, RtpcError> {
    let truncated = |offset| RtpcError::Truncated { offset };
    let hash = bytes::u32_at(data, offset).ok_or(truncated(offset))?;
    let raw: [u8; 4] = bytes::slice(data, offset + 4, 4)
        .and_then(|b| b.try_into().ok())
        .ok_or(truncated(offset + 4))?;
    let type_id = *data.get(offset + 8).ok_or(truncated(offset + 8))?;
    let pointer = u32::from_le_bytes(raw) as usize;

    let value = match type_id {
        1 => PropValue::U32(u32::from_le_bytes(raw)),
        2 => PropValue::F32(f32::from_le_bytes(raw)),
        3 => {
            let text = bytes::cstr_at(data, pointer).ok_or(truncated(pointer))?;
            PropValue::String(String::from_utf8_lossy(text).into_owned())
        }
        4..=8 => {
            let count = match type_id {
                4 => 2,
                5 => 3,
                6 => 4,
                7 => 9,
                _ => 16,
            };
            PropValue::Floats(read_floats(data, pointer, count)?)
        }
        9 => {
            let count = array_count(data, pointer)?;
            PropValue::U32Array(
                (0..count)
                    .map(|i| bytes::u32_at(data, pointer + 4 + 4 * i).ok_or(truncated(pointer)))
                    .collect::<Result<_, _>>()?,
            )
        }
        10 => {
            let count = array_count(data, pointer)?;
            PropValue::F32Array(read_floats(data, pointer + 4, count)?)
        }
        11 => {
            let count = array_count(data, pointer)?;
            let body = bytes::slice(data, pointer + 4, count).ok_or(truncated(pointer))?;
            PropValue::Bytes(body.to_vec())
        }
        13 => PropValue::U64(bytes::u64_at(data, pointer).ok_or(truncated(pointer))?),
        14 => {
            let count = array_count(data, pointer)?;
            let pair = |i: usize| {
                let at = pointer + 4 + 8 * i;
                Some((bytes::u32_at(data, at)?, bytes::u32_at(data, at + 4)?))
            };
            PropValue::ObjectIds(
                (0..count)
                    .map(|i| pair(i).ok_or(truncated(pointer)))
                    .collect::<Result<_, _>>()?,
            )
        }
        other => PropValue::Raw {
            type_id: other,
            bytes: raw,
        },
    };
    Ok(Property { hash, value })
}

/// Quantidade armazenada no início de um array (`u32`).
fn array_count(data: &[u8], pointer: usize) -> Result<usize, RtpcError> {
    bytes::u32_at(data, pointer)
        .map(|c| c as usize)
        .ok_or(RtpcError::Truncated { offset: pointer })
}

fn read_floats(data: &[u8], offset: usize, count: usize) -> Result<Vec<f32>, RtpcError> {
    // Confere o total antes de alocar: contagem absurda vira erro.
    let body = bytes::slice(data, offset, count.saturating_mul(4))
        .ok_or(RtpcError::Truncated { offset })?;
    let (words, _) = body.as_chunks::<4>();
    Ok(words.iter().map(|w| f32::from_le_bytes(*w)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monta um RTPC com a raiz em 8, props logo depois do nó e dados no fim.
    struct Builder {
        data: Vec<u8>,
    }

    impl Builder {
        fn new() -> Self {
            let mut data = b"RTPC".to_vec();
            data.extend_from_slice(&[2, 0, 0, 0]);
            Self { data }
        }

        fn u32(&mut self, v: u32) {
            self.data.extend_from_slice(&v.to_le_bytes());
        }

        fn node(&mut self, hash: u32, props_at: usize, props: u16, kids: u16) {
            self.u32(hash);
            self.u32(u32::try_from(props_at).unwrap());
            self.data.extend_from_slice(&props.to_le_bytes());
            self.data.extend_from_slice(&kids.to_le_bytes());
        }

        fn prop(&mut self, hash: u32, raw: [u8; 4], type_id: u8) {
            self.u32(hash);
            self.data.extend_from_slice(&raw);
            self.data.push(type_id);
        }

        fn here(&self) -> u32 {
            u32::try_from(self.data.len()).unwrap()
        }
    }

    /// Raiz com uma propriedade de cada tipo de ponteiro e um filho.
    fn sample() -> Vec<u8> {
        // Layout: raiz (8..20), props em 20 (11 × 9 = 99 bytes -> 119), filhos em 120.
        let mut b = Builder::new();
        b.node(0xAA, 20, 11, 1);
        let data_at = 20 + 99 + 1 + 12 + 12; // depois de alinhamento (120), filho (12) e folga
        let mut at = u32::try_from(data_at).unwrap();
        let mut blobs: Vec<Vec<u8>> = Vec::new();
        let mut ptr = |blob: Vec<u8>| {
            let p = at;
            at += u32::try_from(blob.len()).unwrap();
            blobs.push(blob);
            p.to_le_bytes()
        };
        let le = |vals: &[f32]| {
            vals.iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let counted = |n: u32, body: Vec<u8>| [n.to_le_bytes().to_vec(), body].concat();

        b.prop(1, 7u32.to_le_bytes(), 1);
        b.prop(2, 1.5f32.to_le_bytes(), 2);
        b.prop(3, ptr(b"texto\0".to_vec()), 3);
        b.prop(4, ptr(le(&[1.0, 2.0])), 4);
        b.prop(8, ptr(le(&[0.5; 16])), 8);
        b.prop(
            9,
            ptr(counted(
                2,
                [5u32.to_le_bytes(), 6u32.to_le_bytes()].concat(),
            )),
            9,
        );
        b.prop(10, ptr(counted(1, le(&[2.5]))), 10);
        b.prop(11, ptr(counted(3, vec![9, 8, 7])), 11);
        b.prop(13, ptr(u64::MAX.to_le_bytes().to_vec()), 13);
        b.prop(
            14,
            ptr(counted(
                1,
                [1u32.to_le_bytes(), 2u32.to_le_bytes()].concat(),
            )),
            14,
        );
        b.prop(99, [1, 2, 3, 4], 77);
        assert_eq!(b.data.len(), 20 + 99);
        b.data.push(0); // alinhamento até 120
        b.node(0xBB, 0, 0, 0); // filho em 120..132
        b.data.resize(data_at, 0);
        assert_eq!(b.here(), u32::try_from(data_at).unwrap());
        for blob in blobs {
            b.data.extend(blob);
        }
        b.data
    }

    #[test]
    fn decodes_every_property_type_and_the_child_node() {
        let root = parse(&sample()).unwrap();
        assert_eq!(root.hash, 0xAA);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].hash, 0xBB);

        let values: Vec<_> = root.props.iter().map(|p| p.value.clone()).collect();
        assert_eq!(values[0], PropValue::U32(7));
        assert_eq!(values[1], PropValue::F32(1.5));
        assert_eq!(values[2], PropValue::String("texto".into()));
        assert_eq!(values[3], PropValue::Floats(vec![1.0, 2.0]));
        assert_eq!(values[4], PropValue::Floats(vec![0.5; 16]));
        assert_eq!(values[5], PropValue::U32Array(vec![5, 6]));
        assert_eq!(values[6], PropValue::F32Array(vec![2.5]));
        assert_eq!(values[7], PropValue::Bytes(vec![9, 8, 7]));
        assert_eq!(values[8], PropValue::U64(u64::MAX));
        assert_eq!(values[9], PropValue::ObjectIds(vec![(1, 2)]));
        assert_eq!(
            values[10],
            PropValue::Raw {
                type_id: 77,
                bytes: [1, 2, 3, 4]
            }
        );
    }

    #[test]
    fn rejects_wrong_magic() {
        assert_eq!(parse(b"NOPE").unwrap_err(), RtpcError::BadMagic);
    }

    #[test]
    fn every_truncation_is_an_error_not_a_panic() {
        let full = sample();
        for len in 4..full.len() {
            let _ = parse(&full[..len]);
        }
        assert!(matches!(
            parse(&full[..full.len() - 1]),
            Err(RtpcError::Truncated { .. })
        ));
    }

    #[test]
    fn child_that_points_to_itself_hits_the_depth_limit() {
        // Raiz sem props, 1 filho em 20; o filho aponta props em 20 e tem 1 filho: ele mesmo.
        let mut b = Builder::new();
        b.node(1, 20, 0, 1);
        b.node(2, 20, 0, 1);
        assert_eq!(parse(&b.data).unwrap_err(), RtpcError::TooBig);
    }

    #[test]
    fn huge_float_array_count_is_an_error_not_an_allocation() {
        let mut b = Builder::new();
        b.node(1, 20, 1, 0);
        b.prop(1, 40u32.to_le_bytes(), 10);
        b.data.resize(40, 0);
        b.u32(u32::MAX);
        assert!(matches!(parse(&b.data), Err(RtpcError::Truncated { .. })));
    }
}
