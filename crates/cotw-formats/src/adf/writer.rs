//! Escritor ADF: troca os elementos de arrays reconstruindo offsets, contagens e a cadeia de
//! relocação (ver `layout`).
//!
//! Regras (seção 2.3 da skill `cotw-save-knowledge`): depois de inserir ou remover bytes é
//! preciso deslocar todo offset que aponta para depois da edição, atualizar as contagens,
//! reescrever o `next` de todos os ponteiros, ajustar o tamanho da instância e deslocar as
//! tabelas e instâncias que ficam depois dela. Uma cadeia quebrada trava o jogo.
//!
//! Limites desta versão, verificados e com erro explícito em vez de gravar algo duvidoso:
//! - só a troca dos elementos de arrays que já têm elementos;
//! - elementos sem ponteiros dentro (nada de arrays aninhados nos trechos trocados);
//! - a variação de tamanho deve ser múltiplo de 8 bytes (preserva o alinhamento).

use super::layout::{PointerField, PointerKind};
use super::{Adf, AdfError};

/// Troca todos os elementos de um array por `elements`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayReplacement {
    /// Posição absoluta do cabeçalho do array (campo `position` de [`PointerField`]).
    pub position: usize,
    /// Bytes dos novos elementos, concatenados.
    pub elements: Vec<u8>,
}

/// Erros ao reescrever um ADF.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// Falha ao ler a estrutura original.
    #[error(transparent)]
    Adf(#[from] AdfError),
    /// Não existe a instância pedida.
    #[error("não existe a instância {0}")]
    NoSuchInstance(usize),
    /// A posição não é o cabeçalho de um array da instância.
    #[error("a posição {position} não é o cabeçalho de um array")]
    NotAnArray {
        /// Posição pedida.
        position: usize,
    },
    /// O array está vazio; crescer a partir de vazio não é suportado.
    #[error("o array em {position} está vazio; não é possível crescer a partir de vazio")]
    EmptyArray {
        /// Posição do cabeçalho.
        position: usize,
    },
    /// O tamanho dos novos elementos não é múltiplo do tamanho do elemento.
    #[error("{len} bytes não formam elementos de {element_size} bytes (array em {position})")]
    BadElementLength {
        /// Posição do cabeçalho.
        position: usize,
        /// Tamanho do elemento.
        element_size: u32,
        /// Tamanho recebido.
        len: usize,
    },
    /// A variação de tamanho quebraria o alinhamento de 8 bytes.
    #[error("variação de {delta} bytes no array em {position} não preserva o alinhamento de 8")]
    Misaligned {
        /// Posição do cabeçalho.
        position: usize,
        /// Variação em bytes.
        delta: isize,
    },
    /// Há ponteiros dentro do trecho trocado.
    #[error("o array em {position} contém ou é referenciado por outros ponteiros")]
    NestedPointers {
        /// Posição do cabeçalho.
        position: usize,
    },
    /// O mesmo array foi editado mais de uma vez, ou trechos se sobrepõem.
    #[error("o array em {position} foi editado mais de uma vez")]
    Duplicate {
        /// Posição do cabeçalho.
        position: usize,
    },
    /// O resultado não cabe nos campos de 32 bits do formato.
    #[error("resultado grande demais para o formato")]
    TooLarge,
    /// A conferência do resultado falhou; nada deve ser gravado.
    #[error("o resultado não passou na conferência: {0}")]
    Inconsistent(String),
}

/// Plano de troca de um array.
struct Plan<'a> {
    field: PointerField,
    start: usize,
    old_len: usize,
    new: &'a [u8],
    new_count: u64,
}

impl Plan<'_> {
    fn old_end(&self) -> usize {
        self.start + self.old_len
    }

    fn delta(&self) -> isize {
        // Tamanhos de arquivo cabem em isize.
        self.new.len().cast_signed() - self.old_len.cast_signed()
    }
}

impl Adf {
    /// Reescreve a instância `instance_index` trocando os elementos dos arrays indicados e
    /// devolve os bytes do novo ADF. O resultado é relido e conferido antes de ser devolvido.
    ///
    /// # Errors
    ///
    /// Retorna [`WriteError`] se algum pedido não for suportado ou o resultado for inconsistente.
    pub fn rewrite_arrays(
        &self,
        instance_index: usize,
        edits: &[ArrayReplacement],
    ) -> Result<Vec<u8>, WriteError> {
        let instance = self
            .instances
            .get(instance_index)
            .ok_or(WriteError::NoSuchInstance(instance_index))?;
        let base = instance.offset as usize;
        let old_end = base + instance.size as usize;
        let fields = self.pointer_fields(instance)?;
        let plans = Self::plan(&fields, base, edits)?;
        let total: isize = plans.iter().map(Plan::delta).sum();

        let shift = |position: usize| -> isize {
            plans
                .iter()
                .filter(|p| position >= p.old_end())
                .map(Plan::delta)
                .sum()
        };
        let mapped = |position: usize| position.saturating_add_signed(shift(position));

        // Copia o arquivo trocando os trechos dos arrays.
        let mut out = Vec::with_capacity(self.data.len().saturating_add_signed(total));
        let mut cursor = 0;
        for plan in &plans {
            out.extend_from_slice(&self.data[cursor..plan.start]);
            out.extend_from_slice(plan.new);
            cursor = plan.old_end();
        }
        out.extend_from_slice(&self.data[cursor..]);

        // Offsets, contagens e cadeia dos ponteiros da instância.
        let edited = |f: &PointerField| plans.iter().find(|p| p.field.position == f.position);
        let mut chain: Vec<usize> = Vec::new();
        for field in &fields {
            let position = mapped(field.position);
            let plan = edited(field);
            let becomes_null = plan.is_some_and(|p| p.new_count == 0);
            if let Some(plan) = plan {
                put_u64(&mut out, position + 8, plan.new_count);
            }
            if becomes_null {
                put_u32(&mut out, position, 0);
                put_u32(&mut out, position + 4, 0);
                continue;
            }
            if field.is_null() {
                continue;
            }
            let new_target = mapped(field.target(base));
            put_u32(&mut out, position, to_u32(new_target - base)?);
            chain.push(position);
        }
        // `fields` está em ordem de posição e o mapeamento preserva a ordem.
        for (i, &position) in chain.iter().enumerate() {
            let next = chain.get(i + 1).map_or(0, |&n| n - position);
            put_u32(&mut out, position + 4, to_u32(next)?);
        }

        self.fix_tables(&mut out, instance_index, old_end, total, &mapped)?;
        self.verify(&out, instance_index, &plans)?;
        Ok(out)
    }

    fn plan<'a>(
        fields: &[PointerField],
        base: usize,
        edits: &'a [ArrayReplacement],
    ) -> Result<Vec<Plan<'a>>, WriteError> {
        let mut plans: Vec<Plan<'a>> = Vec::new();
        for edit in edits {
            let position = edit.position;
            let field = fields
                .iter()
                .find(|f| f.position == position)
                .copied()
                .ok_or(WriteError::NotAnArray { position })?;
            let PointerKind::Array {
                count,
                element_size,
            } = field.kind
            else {
                return Err(WriteError::NotAnArray { position });
            };
            if plans.iter().any(|p| p.field.position == position) {
                return Err(WriteError::Duplicate { position });
            }
            if count == 0 {
                return Err(WriteError::EmptyArray { position });
            }
            let size = element_size as usize;
            if size == 0 || edit.elements.len() % size != 0 {
                return Err(WriteError::BadElementLength {
                    position,
                    element_size,
                    len: edit.elements.len(),
                });
            }
            let old_len = usize::try_from(count)
                .ok()
                .and_then(|c| c.checked_mul(size))
                .ok_or(WriteError::TooLarge)?;
            let plan = Plan {
                field,
                start: field.target(base),
                old_len,
                new: &edit.elements,
                new_count: (edit.elements.len() / size) as u64,
            };
            if plan.delta() % 8 != 0 {
                return Err(WriteError::Misaligned {
                    position,
                    delta: plan.delta(),
                });
            }
            let range = plan.start..plan.old_end();
            let referenced = fields.iter().filter(|f| f.position != position).any(|f| {
                range.contains(&f.position) || (!f.is_null() && range.contains(&f.target(base)))
            });
            if referenced {
                return Err(WriteError::NestedPointers { position });
            }
            plans.push(plan);
        }
        plans.sort_by_key(|p| p.start);
        if let Some(pair) = plans.windows(2).find(|w| w[0].old_end() > w[1].start) {
            return Err(WriteError::Duplicate {
                position: pair[1].field.position,
            });
        }
        Ok(plans)
    }

    /// Tamanho da instância, instâncias seguintes e tabelas do cabeçalho que ficam depois dela.
    fn fix_tables(
        &self,
        out: &mut [u8],
        instance_index: usize,
        old_end: usize,
        total: isize,
        mapped: &dyn Fn(usize) -> usize,
    ) -> Result<(), WriteError> {
        const TABLE_FIELDS: [usize; 5] = [12, 20, 28, 36, 40];
        let table = crate::bytes::u32_at(&self.data, 12).ok_or(AdfError::OutOfBounds {
            offset: 12,
            wanted: 4,
        })? as usize;
        let shift_u32 = |out: &mut [u8], at: usize| -> Result<(), WriteError> {
            let value = get_u32(out, at)?;
            let moved = (value as usize).saturating_add_signed(total);
            put_u32(out, at, to_u32(moved)?);
            Ok(())
        };

        for (index, instance) in self.instances.iter().enumerate() {
            let entry = mapped(table + 24 * index);
            if index == instance_index {
                shift_u32(out, entry + 12)?; // tamanho dos dados da instância
            } else if instance.offset as usize >= old_end {
                shift_u32(out, entry + 8)?;
            }
        }
        for field in TABLE_FIELDS {
            let value = crate::bytes::u32_at(&self.data, field).ok_or(AdfError::OutOfBounds {
                offset: field,
                wanted: 4,
            })? as usize;
            if value != 0 && value >= old_end {
                shift_u32(out, field)?;
            }
        }
        Ok(())
    }

    /// Relê o resultado e confere cadeia, contagens e que só os arrays editados mudaram.
    fn verify(
        &self,
        out: &[u8],
        instance_index: usize,
        plans: &[Plan<'_>],
    ) -> Result<(), WriteError> {
        let inconsistent = |message: &str| WriteError::Inconsistent(message.to_owned());
        let reread = Adf::parse(out.to_vec())?;
        let instance = reread
            .instances
            .get(instance_index)
            .ok_or_else(|| inconsistent("instância sumiu"))?;
        if !reread.chain_issues(instance)?.is_empty() {
            return Err(inconsistent("cadeia de relocação quebrada"));
        }
        if reread.instances.len() != self.instances.len() || reread.types.len() != self.types.len()
        {
            return Err(inconsistent("tabelas mudaram de tamanho"));
        }
        let before = self.pointer_fields(&self.instances[instance_index])?;
        let after = reread.pointer_fields(instance)?;
        // Cada array editado vira empty (some da lista de elementos) ou continua; os demais
        // ponteiros mantêm tipo e contagem.
        if before.len() != after.len() {
            return Err(inconsistent("quantidade de ponteiros mudou"));
        }
        for (old, new) in before.iter().zip(&after) {
            let planned = plans.iter().find(|p| p.field.position == old.position);
            let same_kind = match (old.kind, new.kind) {
                (PointerKind::Array { count: a, .. }, PointerKind::Array { count: b, .. }) => {
                    planned.map_or(a == b, |p| p.new_count == b)
                }
                (a, b) => a == b,
            };
            if !same_kind {
                return Err(inconsistent("tipo ou contagem de um ponteiro mudou"));
            }
        }
        Ok(())
    }
}

fn to_u32(value: usize) -> Result<u32, WriteError> {
    u32::try_from(value).map_err(|_| WriteError::TooLarge)
}

fn get_u32(data: &[u8], at: usize) -> Result<u32, WriteError> {
    Ok(crate::bytes::u32_at(data, at).ok_or(AdfError::OutOfBounds {
        offset: at,
        wanted: 4,
    })?)
}

fn put_u32(data: &mut [u8], at: usize, value: u32) {
    if let Some(slot) = data.get_mut(at..at + 4) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}

fn put_u64(data: &mut [u8], at: usize, value: u64) {
    if let Some(slot) = data.get_mut(at..at + 8) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}
