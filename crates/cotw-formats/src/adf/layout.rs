//! Mapa dos ponteiros de uma instância ADF e verificação da cadeia de relocação.
//!
//! Um ponteiro é um campo cujo conteúdo está em outro lugar da área de dados da instância:
//! arrays (cabeçalho de 16 bytes `u32 offset, u32 next, u64 count`) e strings (`u32 offset`
//! seguido de `u32 next`). O offset é relativo ao início da instância. O campo `next` forma a
//! *cadeia de relocação*: guarda a distância até o próximo ponteiro não nulo, em ordem de posição.

use super::reader::Reader;
use super::types::Primitive;
use super::{Adf, AdfError, Instance, TypeKind};

const MAX_DEPTH: usize = 64;

/// O que o ponteiro aponta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerKind {
    /// Array na heap da instância.
    Array {
        /// Quantidade de elementos.
        count: u64,
        /// Tamanho de cada elemento em bytes.
        element_size: u32,
    },
    /// String terminada em NUL.
    String,
    /// Ponteiro tipado (`T*`) para dados cujo tamanho é dado por outro campo.
    Pointer,
}

/// Um campo-ponteiro dentro de uma instância.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerField {
    /// Posição absoluta do campo no arquivo.
    pub position: usize,
    /// Tipo do ponteiro.
    pub kind: PointerKind,
    /// Offset relativo ao início da instância (0 = nulo).
    pub offset: u32,
    /// Valor atual do campo `next` da cadeia.
    pub next: u32,
}

impl PointerField {
    /// Ponteiro sem destino (array vazio ou string vazia).
    #[must_use]
    pub fn is_null(&self) -> bool {
        match self.kind {
            PointerKind::Array { count, .. } => self.offset == 0 && count == 0,
            PointerKind::String | PointerKind::Pointer => self.offset == 0,
        }
    }

    /// Posição absoluta dos dados apontados, dado o início da instância.
    #[must_use]
    pub fn target(&self, base: usize) -> usize {
        base + self.offset as usize
    }
}

impl Adf {
    /// Todos os ponteiros de uma instância, em ordem de posição no arquivo.
    ///
    /// # Errors
    ///
    /// Retorna [`AdfError`] se a estrutura apontar para fora dos dados.
    pub fn pointer_fields(&self, instance: &Instance) -> Result<Vec<PointerField>, AdfError> {
        let base = instance.offset as usize;
        let mut fields = Vec::new();
        self.collect_pointers(instance.type_hash, base, base, 0, &mut fields)?;
        fields.sort_by_key(|f| f.position);
        Ok(fields)
    }

    /// Posições dos ponteiros cujo `next` não bate com a distância até o ponteiro seguinte.
    ///
    /// O último ponteiro da cadeia não é conferido (guarda o valor original, normalmente 0).
    ///
    /// # Errors
    ///
    /// Retorna [`AdfError`] se a estrutura apontar para fora dos dados.
    pub fn chain_issues(&self, instance: &Instance) -> Result<Vec<usize>, AdfError> {
        let members: Vec<PointerField> = self
            .pointer_fields(instance)?
            .into_iter()
            .filter(|f| !f.is_null())
            .collect();
        Ok(members
            .windows(2)
            .filter(|pair| (pair[1].position - pair[0].position) as u64 != u64::from(pair[0].next))
            .map(|pair| pair[0].position)
            .collect())
    }

    /// Se valores deste tipo podem conter ponteiros.
    fn type_has_pointers(&self, type_hash: u32, depth: usize) -> bool {
        if depth > MAX_DEPTH {
            return true;
        }
        if let Some(primitive) = Primitive::from_hash(type_hash) {
            return primitive == Primitive::String;
        }
        let Some(def) = self.types.get(&type_hash) else {
            return false;
        };
        match def.kind {
            TypeKind::Array | TypeKind::Pointer | TypeKind::String => true,
            TypeKind::Struct => def
                .members
                .iter()
                .any(|m| self.type_has_pointers(m.type_hash, depth + 1)),
            TypeKind::InlineArray => self.type_has_pointers(def.element_hash, depth + 1),
            _ => false,
        }
    }

    fn collect_pointers(
        &self,
        type_hash: u32,
        offset: usize,
        base: usize,
        depth: usize,
        out: &mut Vec<PointerField>,
    ) -> Result<(), AdfError> {
        if depth > MAX_DEPTH {
            return Err(AdfError::TooDeep);
        }
        if let Some(primitive) = Primitive::from_hash(type_hash) {
            if primitive == Primitive::String {
                let mut reader = Reader::at(&self.data, offset);
                let relative = reader.u32()?;
                let next = reader.u32()?;
                out.push(PointerField {
                    position: offset,
                    kind: PointerKind::String,
                    offset: relative,
                    next,
                });
            }
            return Ok(());
        }
        let Some(def) = self.types.get(&type_hash) else {
            return Ok(());
        };
        match def.kind {
            TypeKind::Pointer | TypeKind::String => {
                let mut reader = Reader::at(&self.data, offset);
                let relative = reader.u32()?;
                let next = reader.u32()?;
                out.push(PointerField {
                    position: offset,
                    kind: if def.kind == TypeKind::Pointer {
                        PointerKind::Pointer
                    } else {
                        PointerKind::String
                    },
                    offset: relative,
                    next,
                });
            }
            TypeKind::Struct => {
                for member in &def.members {
                    self.collect_pointers(
                        member.type_hash,
                        offset + member.offset as usize,
                        base,
                        depth + 1,
                        out,
                    )?;
                }
            }
            TypeKind::Array => {
                let mut reader = Reader::at(&self.data, offset);
                let relative = reader.u32()?;
                let next = reader.u32()?;
                let count = reader.u64()?;
                let element_size = self.element_size(def.element_hash);
                out.push(PointerField {
                    position: offset,
                    kind: PointerKind::Array {
                        count,
                        element_size,
                    },
                    offset: relative,
                    next,
                });
                if self.type_has_pointers(def.element_hash, 0) {
                    let count = usize::try_from(count).map_err(|_| AdfError::OutOfBounds {
                        offset,
                        wanted: usize::MAX,
                    })?;
                    self.collect_elements(
                        def.element_hash,
                        base + relative as usize,
                        count,
                        base,
                        depth,
                        out,
                    )?;
                }
            }
            TypeKind::InlineArray if self.type_has_pointers(def.element_hash, 0) => {
                self.collect_elements(
                    def.element_hash,
                    offset,
                    def.element_len as usize,
                    base,
                    depth,
                    out,
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    fn collect_elements(
        &self,
        element_hash: u32,
        start: usize,
        count: usize,
        base: usize,
        depth: usize,
        out: &mut Vec<PointerField>,
    ) -> Result<(), AdfError> {
        let element_size = self.element_size(element_hash) as usize;
        // Confere o total antes de percorrer: contagem absurda vira erro.
        Reader::at(&self.data, start).skip(count.saturating_mul(element_size))?;
        for i in 0..count {
            self.collect_pointers(element_hash, start + i * element_size, base, depth + 1, out)?;
        }
        Ok(())
    }
}
