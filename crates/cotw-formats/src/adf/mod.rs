//! Leitor genérico de ADF (Avalanche Data Format), somente leitura.
//!
//! Layout (ver `.claude/skills/cotw-save-knowledge/SKILL.md`, seção 2.2):
//!
//! ```text
//! @0    magic " FDA"
//! @4    10 × u32: versão, nº instâncias, off instâncias, nº typedefs, off typedefs,
//!       nº string hashes, off string hashes, nº nomes, off nomes, tamanho total
//! @0x40 comentário (texto terminado em NUL)
//! ```
//!
//! Porta de `research/reference/adf.py`, que serve de oráculo nos testes.

mod layout;
mod reader;
mod types;
mod value;
mod writer;

use std::collections::BTreeMap;

pub use layout::{PointerField, PointerKind};
use reader::{Reader, latin1};
pub use types::{EnumValue, Member, TypeDef, TypeKind};
pub use value::Value;
pub use writer::{ArrayReplacement, WriteError};

const MAGIC: &[u8; 4] = b" FDA";
const COMMENT_OFFSET: usize = 0x40;

/// Erros ao ler um ADF.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdfError {
    /// O arquivo não começa com a assinatura ` FDA`.
    #[error("assinatura ADF ausente: arquivo não reconhecido")]
    BadMagic,
    /// Uma leitura passou do fim dos dados.
    #[error("leitura fora dos limites (offset {offset}, {wanted} bytes)")]
    OutOfBounds {
        /// Posição da leitura.
        offset: usize,
        /// Quantidade de bytes pedida.
        wanted: usize,
    },
    /// Texto sem o NUL final.
    #[error("texto sem terminador NUL (offset {offset})")]
    UnterminatedString {
        /// Posição onde o texto começa.
        offset: usize,
    },
    /// Índice inexistente na tabela de nomes.
    #[error("índice de nome inválido: {0}")]
    BadNameIndex(u64),
    /// Estrutura aninhada fundo demais (dados corrompidos ou tipo cíclico).
    #[error("estrutura aninhada fundo demais")]
    TooDeep,
}

/// Instância de nível raiz do arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// Hash do nome da instância.
    pub name_hash: u32,
    /// Hash do tipo da instância.
    pub type_hash: u32,
    /// Offset dos dados da instância no arquivo.
    pub offset: u32,
    /// Tamanho dos dados da instância.
    pub size: u32,
    /// Nome da instância.
    pub name: String,
}

/// Um arquivo ADF decodificado: tabelas de tipos, instâncias e os bytes originais.
#[derive(Debug, Clone)]
pub struct Adf {
    data: Vec<u8>,
    version: u32,
    comment: String,
    string_hashes: BTreeMap<u64, String>,
    types: BTreeMap<u32, TypeDef>,
    instances: Vec<Instance>,
}

impl Adf {
    /// Lê um ADF (por exemplo, o resultado de `SaveEnvelope::adf`).
    ///
    /// # Errors
    ///
    /// Retorna [`AdfError`] se a assinatura for inválida ou alguma tabela estiver truncada.
    pub fn parse(data: Vec<u8>) -> Result<Self, AdfError> {
        let mut header = Reader::at(&data, 0);
        if header.take(4)? != MAGIC {
            return Err(AdfError::BadMagic);
        }
        let version = header.u32()?;
        let instance_count = header.u32()?;
        let instance_offset = header.u32()?;
        let type_count = header.u32()?;
        let type_offset = header.u32()?;
        let hash_count = header.u32()?;
        let hash_offset = header.u32()?;
        let name_count = header.u32()?;
        let name_offset = header.u32()?;

        let names = parse_names(&data, name_offset as usize, name_count as usize)?;
        let string_hashes = parse_string_hashes(&data, hash_offset as usize, hash_count)?;
        let types = parse_types(&data, type_offset as usize, type_count, &names)?;
        let instances = parse_instances(&data, instance_offset as usize, instance_count, &names)?;

        Ok(Self {
            comment: read_comment(&data),
            data,
            version,
            string_hashes,
            types,
            instances,
        })
    }

    /// Versão do formato.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Comentário do cabeçalho (geralmente identifica a ferramenta que gerou o arquivo).
    #[must_use]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    /// Bytes originais do arquivo.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Todos os typedefs, ordenados pelo hash.
    pub fn types(&self) -> impl Iterator<Item = &TypeDef> {
        self.types.values()
    }

    /// Busca um typedef pelo hash.
    #[must_use]
    pub fn type_def(&self, hash: u32) -> Option<&TypeDef> {
        self.types.get(&hash)
    }

    /// Instâncias de nível raiz, na ordem do arquivo.
    #[must_use]
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    /// Texto associado a um hash de string (tipos de meta 9).
    #[must_use]
    pub fn string_for_hash(&self, hash: u64) -> Option<&str> {
        self.string_hashes.get(&hash).map(String::as_str)
    }
}

fn read_comment(data: &[u8]) -> String {
    let rest = data.get(COMMENT_OFFSET..).unwrap_or_default();
    let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    latin1(&rest[..end])
}

fn parse_names(data: &[u8], offset: usize, count: usize) -> Result<Vec<String>, AdfError> {
    let mut reader = Reader::at(data, offset);
    let lengths = reader.take(count)?;
    let mut names = Vec::with_capacity(count);
    for &len in lengths {
        names.push(latin1(reader.take(usize::from(len))?));
        reader.skip(1)?; // NUL final
    }
    Ok(names)
}

fn parse_string_hashes(
    data: &[u8],
    offset: usize,
    count: u32,
) -> Result<BTreeMap<u64, String>, AdfError> {
    let mut map = BTreeMap::new();
    if count == 0 {
        return Ok(map);
    }
    let mut reader = Reader::at(data, offset);
    let mut texts = Vec::new();
    for _ in 0..count {
        texts.push(latin1(reader.cstr()?));
    }
    for text in texts {
        map.insert(reader.u64()?, text);
    }
    Ok(map)
}

fn name_at(names: &[String], index: u64) -> Result<String, AdfError> {
    usize::try_from(index)
        .ok()
        .and_then(|i| names.get(i))
        .cloned()
        .ok_or(AdfError::BadNameIndex(index))
}

fn parse_types(
    data: &[u8],
    offset: usize,
    count: u32,
    names: &[String],
) -> Result<BTreeMap<u32, TypeDef>, AdfError> {
    let mut reader = Reader::at(data, offset);
    let mut types = BTreeMap::new();
    for _ in 0..count {
        let kind = TypeKind::from_meta(reader.u32()?);
        let size = reader.u32()?;
        let align = reader.u32()?;
        let hash = reader.u32()?;
        let name = name_at(names, reader.u64()?)?;
        let flags = reader.u32()?;
        let element_hash = reader.u32()?;
        let element_len = reader.u32()?;
        let child_count = reader.u32()?;

        let mut members = Vec::new();
        let mut enum_values = Vec::new();
        match kind {
            TypeKind::Struct => {
                for _ in 0..child_count {
                    let member_name = name_at(names, reader.u64()?)?;
                    let type_hash = reader.u32()?;
                    let member_size = reader.u32()?;
                    let packed_offset = reader.u32()?;
                    let member_flags = reader.u32()?;
                    reader.skip(8)?; // valor padrão, não usado
                    members.push(Member {
                        name: member_name,
                        type_hash,
                        size: member_size,
                        offset: packed_offset & 0x00ff_ffff,
                        bit: (packed_offset >> 24) as u8,
                        flags: member_flags,
                    });
                }
            }
            TypeKind::Enum => {
                for _ in 0..child_count {
                    let value_name = name_at(names, reader.u64()?)?;
                    enum_values.push(EnumValue {
                        value: reader.u32()?,
                        name: value_name,
                    });
                }
            }
            TypeKind::BitField => reader.skip((child_count as usize).saturating_mul(12))?,
            _ => {}
        }
        types.insert(
            hash,
            TypeDef {
                kind,
                size,
                align,
                hash,
                name,
                flags,
                element_hash,
                element_len,
                members,
                enum_values,
            },
        );
    }
    Ok(types)
}

fn parse_instances(
    data: &[u8],
    offset: usize,
    count: u32,
    names: &[String],
) -> Result<Vec<Instance>, AdfError> {
    let mut reader = Reader::at(data, offset);
    let mut instances = Vec::new();
    for _ in 0..count {
        let name_hash = reader.u32()?;
        let type_hash = reader.u32()?;
        let offset = reader.u32()?;
        let size = reader.u32()?;
        let name = name_at(names, reader.u64()?)?;
        instances.push(Instance {
            name_hash,
            type_hash,
            offset,
            size,
            name,
        });
    }
    Ok(instances)
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod writer_tests;
