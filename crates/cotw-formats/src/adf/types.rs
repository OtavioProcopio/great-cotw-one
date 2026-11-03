//! Definições de tipos (typedefs) de um ADF.

/// Família de um tipo (campo `meta` do typedef).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// Primitivo nomeado (meta 0).
    Primitive,
    /// Struct com membros (meta 1).
    Struct,
    /// Ponteiro (meta 2).
    Pointer,
    /// Array na heap da instância (meta 3).
    Array,
    /// Array inline de tamanho fixo (meta 4).
    InlineArray,
    /// String (meta 5).
    String,
    /// Campo de bits (meta 7).
    BitField,
    /// Enumeração (meta 8).
    Enum,
    /// Hash de string, resolvido pela tabela de string hashes (meta 9).
    StringHash,
    /// Qualquer outro valor de meta.
    Unknown(u32),
}

impl TypeKind {
    pub(super) fn from_meta(meta: u32) -> Self {
        match meta {
            0 => Self::Primitive,
            1 => Self::Struct,
            2 => Self::Pointer,
            3 => Self::Array,
            4 => Self::InlineArray,
            5 => Self::String,
            7 => Self::BitField,
            8 => Self::Enum,
            9 => Self::StringHash,
            other => Self::Unknown(other),
        }
    }
}

/// Membro de um struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// Nome do membro.
    pub name: String,
    /// Hash do tipo do membro.
    pub type_hash: u32,
    /// Tamanho em bytes.
    pub size: u32,
    /// Deslocamento dentro do struct (24 bits baixos do campo original).
    pub offset: u32,
    /// Posição de bit (8 bits altos do campo original).
    pub bit: u8,
    /// Flags do membro.
    pub flags: u32,
}

/// Valor nomeado de uma enumeração.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValue {
    /// Número do valor.
    pub value: u32,
    /// Nome do valor.
    pub name: String,
}

/// Um typedef completo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    /// Família do tipo.
    pub kind: TypeKind,
    /// Tamanho em bytes.
    pub size: u32,
    /// Alinhamento em bytes.
    pub align: u32,
    /// Hash que identifica o tipo.
    pub hash: u32,
    /// Nome do tipo.
    pub name: String,
    /// Flags do tipo.
    pub flags: u32,
    /// Hash do tipo dos elementos (arrays).
    pub element_hash: u32,
    /// Quantidade de elementos (arrays inline).
    pub element_len: u32,
    /// Membros (structs).
    pub members: Vec<Member>,
    /// Valores (enums).
    pub enum_values: Vec<EnumValue>,
}

/// Tipos primitivos, identificados por hash fixo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Primitive {
    U8,
    S8,
    U16,
    S16,
    U32,
    S32,
    U64,
    S64,
    F32,
    F64,
    String,
}

impl Primitive {
    pub(super) fn from_hash(hash: u32) -> Option<Self> {
        Some(match hash {
            0x0CA2_821D => Self::U8,
            0x580D_0A62 => Self::S8,
            0x86D1_52BD => Self::U16,
            0xD13F_CF93 => Self::S16,
            0x075E_4E4F => Self::U32,
            0x192F_E633 => Self::S32,
            0xA139_E01F => Self::U64,
            0xAF41_354F => Self::S64,
            0x7515_A207 => Self::F32,
            0xC609_F663 => Self::F64,
            0x8955_583E => Self::String,
            _ => return None,
        })
    }

    pub(super) fn size(self) -> u32 {
        match self {
            Self::U8 | Self::S8 => 1,
            Self::U16 | Self::S16 => 2,
            Self::U32 | Self::S32 | Self::F32 => 4,
            Self::U64 | Self::S64 | Self::F64 | Self::String => 8,
        }
    }

    pub(super) fn is_byte(self) -> bool {
        matches!(self, Self::U8 | Self::S8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_values_map_to_kinds() {
        assert_eq!(TypeKind::from_meta(1), TypeKind::Struct);
        assert_eq!(TypeKind::from_meta(8), TypeKind::Enum);
        assert_eq!(TypeKind::from_meta(6), TypeKind::Unknown(6));
        let kinds = [0, 2, 3, 4, 5, 7, 9].map(TypeKind::from_meta);
        assert!(!kinds.contains(&TypeKind::Unknown(0)));
    }

    #[test]
    fn primitive_hashes_and_sizes() {
        assert_eq!(Primitive::from_hash(0x075E_4E4F), Some(Primitive::U32));
        assert_eq!(Primitive::from_hash(1), None);
        assert_eq!(Primitive::U8.size(), 1);
        assert_eq!(Primitive::S16.size(), 2);
        assert_eq!(Primitive::F32.size(), 4);
        assert_eq!(Primitive::String.size(), 8);
        assert!(Primitive::S8.is_byte());
        assert!(!Primitive::U16.is_byte());
    }
}
