//! Decodificação de valores a partir dos typedefs.

use super::reader::Reader;
use super::types::Primitive;
use super::{Adf, AdfError, Instance, TypeDef, TypeKind};

/// Profundidade máxima de aninhamento aceita (proteção contra dados corrompidos).
const MAX_DEPTH: usize = 64;

/// Valor decodificado, com os nomes de campos do próprio arquivo.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Inteiro com sinal (s8, s16, s32, s64).
    Signed(i64),
    /// Inteiro sem sinal (u8, u16, u32, u64).
    Unsigned(u64),
    /// Ponto flutuante (f32 ou f64).
    Float(f64),
    /// Texto.
    String(String),
    /// Struct: pares (nome do membro, valor), na ordem de declaração.
    Struct(Vec<(String, Value)>),
    /// Array de valores.
    Array(Vec<Value>),
    /// Array de bytes (u8/s8), mantido cru.
    Bytes(Vec<u8>),
    /// Enum: número e, se conhecido, o nome.
    Enum {
        /// Número armazenado.
        value: u32,
        /// Nome do valor no typedef.
        name: Option<String>,
    },
    /// Hash de string e, se conhecido, o texto original.
    StringHash {
        /// Hash armazenado.
        hash: u64,
        /// Texto resolvido pela tabela de string hashes.
        text: Option<String>,
    },
    /// Tipo desconhecido ou sem decodificação própria: bytes crus.
    Raw {
        /// Hash do tipo.
        type_hash: u32,
        /// Bytes do valor.
        bytes: Vec<u8>,
    },
}

impl Adf {
    /// Decodifica uma instância de nível raiz.
    ///
    /// # Errors
    ///
    /// Retorna [`AdfError`] se os dados da instância estiverem truncados ou corrompidos.
    pub fn read_instance(&self, instance: &Instance) -> Result<Value, AdfError> {
        let base = instance.offset as usize;
        self.read_value(instance.type_hash, base, None, base, 0)
    }

    fn slice(&self, offset: usize, len: usize) -> Result<&[u8], AdfError> {
        Reader::at(&self.data, offset).take(len)
    }

    pub(super) fn element_size(&self, type_hash: u32) -> u32 {
        self.types.get(&type_hash).map_or_else(
            || Primitive::from_hash(type_hash).map_or(4, Primitive::size),
            |t| t.size,
        )
    }

    fn read_value(
        &self,
        type_hash: u32,
        offset: usize,
        size_hint: Option<u32>,
        base: usize,
        depth: usize,
    ) -> Result<Value, AdfError> {
        if depth > MAX_DEPTH {
            return Err(AdfError::TooDeep);
        }
        if let Some(primitive) = Primitive::from_hash(type_hash) {
            return self.read_primitive(primitive, offset, base);
        }
        let Some(def) = self.types.get(&type_hash) else {
            return self.raw(type_hash, offset, size_hint.unwrap_or(4) as usize);
        };
        match def.kind {
            TypeKind::Struct => self.read_struct(def, offset, base, depth),
            TypeKind::Array => self.read_array(def, offset, base, depth),
            TypeKind::InlineArray => self.read_elements(
                def.element_hash,
                offset,
                def.element_len as usize,
                base,
                depth,
            ),
            TypeKind::Enum => {
                let value = Reader::at(&self.data, offset).u32()?;
                let name = def
                    .enum_values
                    .iter()
                    .find(|e| e.value == value)
                    .map(|e| e.name.clone());
                Ok(Value::Enum { value, name })
            }
            TypeKind::StringHash => {
                let hash = Reader::at(&self.data, offset).u64()?;
                let text = self.string_for_hash(hash).map(str::to_owned);
                Ok(Value::StringHash { hash, text })
            }
            _ => self.raw(type_hash, offset, def.size as usize),
        }
    }

    fn raw(&self, type_hash: u32, offset: usize, len: usize) -> Result<Value, AdfError> {
        Ok(Value::Raw {
            type_hash,
            bytes: self.slice(offset, len)?.to_vec(),
        })
    }

    fn read_struct(
        &self,
        def: &TypeDef,
        offset: usize,
        base: usize,
        depth: usize,
    ) -> Result<Value, AdfError> {
        let mut fields = Vec::with_capacity(def.members.len());
        for member in &def.members {
            let value = self.read_value(
                member.type_hash,
                offset + member.offset as usize,
                Some(member.size),
                base,
                depth + 1,
            )?;
            fields.push((member.name.clone(), value));
        }
        Ok(Value::Struct(fields))
    }

    /// Array na heap: cabeçalho `u32 offset (relativo à instância), u32 cadeia, u64 contagem`.
    fn read_array(
        &self,
        def: &TypeDef,
        offset: usize,
        base: usize,
        depth: usize,
    ) -> Result<Value, AdfError> {
        let mut header = Reader::at(&self.data, offset);
        let relative = header.u32()? as usize;
        header.skip(4)?; // elo da cadeia de relocação
        let count = usize::try_from(header.u64()?).map_err(|_| AdfError::OutOfBounds {
            offset,
            wanted: usize::MAX,
        })?;
        self.read_elements(def.element_hash, base + relative, count, base, depth)
    }

    fn read_elements(
        &self,
        element_hash: u32,
        start: usize,
        count: usize,
        base: usize,
        depth: usize,
    ) -> Result<Value, AdfError> {
        if Primitive::from_hash(element_hash).is_some_and(Primitive::is_byte) {
            return Ok(Value::Bytes(self.slice(start, count)?.to_vec()));
        }
        let element_size = self.element_size(element_hash) as usize;
        // Confere o total antes de alocar: uma contagem absurda vira erro, não falta de memória.
        self.slice(start, count.saturating_mul(element_size))?;
        let mut items = Vec::with_capacity(count);
        for i in 0..count {
            items.push(self.read_value(
                element_hash,
                start + i * element_size,
                None,
                base,
                depth + 1,
            )?);
        }
        Ok(Value::Array(items))
    }

    fn read_primitive(
        &self,
        primitive: Primitive,
        offset: usize,
        base: usize,
    ) -> Result<Value, AdfError> {
        let bytes = self.slice(offset, primitive.size() as usize)?;
        let mut buf = [0u8; 8];
        buf[..bytes.len()].copy_from_slice(bytes);
        let unsigned = u64::from_le_bytes(buf);
        Ok(match primitive {
            Primitive::U8 | Primitive::U16 | Primitive::U32 | Primitive::U64 => {
                Value::Unsigned(unsigned)
            }
            Primitive::S8 => Value::Signed(i64::from(i8::from_le_bytes([buf[0]]))),
            Primitive::S16 => Value::Signed(i64::from(i16::from_le_bytes([buf[0], buf[1]]))),
            Primitive::S32 => Value::Signed(i64::from(i32::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3],
            ]))),
            Primitive::S64 => Value::Signed(i64::from_le_bytes(buf)),
            Primitive::F32 => Value::Float(f64::from(f32::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3],
            ]))),
            Primitive::F64 => Value::Float(f64::from_bits(unsigned)),
            Primitive::String => {
                let relative = Reader::at(&self.data, offset).u32()? as usize;
                if relative == 0 {
                    Value::String(String::new())
                } else {
                    let text = Reader::at(&self.data, base + relative).cstr()?;
                    Value::String(String::from_utf8_lossy(text).into_owned())
                }
            }
        })
    }
}
