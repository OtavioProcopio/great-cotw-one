//! `animal_population_N`: grupos de animais de uma reserva e as rotas diárias deles.
//!
//! Só o que o app precisa: a rota (sequência cíclica de chaves de áreas) de cada grupo.

use crate::adf::{Adf, AdfError, Member, Value};
use crate::bytes;

/// Erros ao ler a população.
#[derive(Debug, thiserror::Error)]
pub enum PopulationError {
    /// O ADF é inválido.
    #[error("ADF inválido: {0}")]
    Adf(#[from] AdfError),
    /// A estrutura não é a esperada.
    #[error("animal_population com estrutura inesperada")]
    Layout,
    /// O grupo ou o animal da edição não existe.
    #[error("o animal {animal} do grupo {group} não existe")]
    NoSuchAnimal {
        /// Posição do grupo (a partir de 0).
        group: usize,
        /// Posição do animal no grupo (a partir de 0).
        animal: usize,
    },
    /// O valor da edição não é um número válido.
    #[error("valor inválido na edição")]
    InvalidValue,
}

/// Um animal de um grupo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animal {
    /// Macho (o jogo grava 1 para macho e 2 para fêmea).
    pub male: bool,
    /// Peso em quilos.
    pub weight: f32,
    /// Pontuação do troféu.
    pub score: f32,
    /// Marca de Great One.
    pub great_one: bool,
}

/// Um grupo de animais.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Chaves das áreas que o grupo visita ao longo do dia, em ordem e em ciclo.
    pub route: Vec<u32>,
    /// Animais do grupo.
    pub animals: Vec<Animal>,
}

impl Group {
    /// Grupo só com a rota, sem animais (útil em testes).
    #[must_use]
    pub fn new(route: Vec<u32>) -> Self {
        Self {
            route,
            animals: Vec::new(),
        }
    }
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn items(value: &Value) -> &[Value] {
    match value {
        Value::Array(items) => items,
        _ => &[],
    }
}

fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Unsigned(n) => Some(f64::from(u32::try_from(*n).ok()?)),
        Value::Float(x) => Some(*x),
        _ => None,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "peso e pontuação do jogo são f32"
)]
fn read_animal(value: &Value) -> Option<Animal> {
    let gender = number(member(value, "Gender")?)?;
    let flags = member(value, "FeatureModifiers")
        .and_then(|m| member(m, "Flags"))
        .and_then(number)
        .unwrap_or(0.0);
    Some(Animal {
        male: (gender - 1.0).abs() < f64::EPSILON,
        weight: member(value, "Weight").and_then(number).unwrap_or(0.0) as f32,
        score: member(value, "Score").and_then(number).unwrap_or(0.0) as f32,
        great_one: flags.rem_euclid(2.0) >= 1.0,
    })
}

/// Lê todos os grupos de um ADF de `animal_population_N`.
///
/// # Errors
///
/// Retorna [`PopulationError`] se o ADF for inválido ou não tiver as populações e grupos esperados.
pub fn parse_groups(adf: &[u8]) -> Result<Vec<Group>, PopulationError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let instance = parsed.instances().first().ok_or(PopulationError::Layout)?;
    let root = parsed.read_instance(instance)?;
    let mut groups = Vec::new();
    for population in items(member(&root, "Populations").ok_or(PopulationError::Layout)?) {
        for group in items(member(population, "Groups").ok_or(PopulationError::Layout)?) {
            let route = items(member(group, "NeedZonePathGuids").ok_or(PopulationError::Layout)?)
                .iter()
                .map(|key| match key {
                    Value::Unsigned(n) => u32::try_from(*n).map_err(|_| PopulationError::Layout),
                    _ => Err(PopulationError::Layout),
                })
                .collect::<Result<Vec<u32>, _>>()?;
            let animals = member(group, "Animals").map_or_else(Vec::new, |a| {
                items(a).iter().filter_map(read_animal).collect()
            });
            groups.push(Group { route, animals });
        }
    }
    Ok(groups)
}

/// Onde estão, no arquivo, os campos editáveis de um animal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimalSlot {
    /// Posição do grupo (a partir de 0).
    pub group: usize,
    /// Posição do animal no grupo (a partir de 0).
    pub index: usize,
    gender: (usize, u32),
    weight: usize,
    score: usize,
    flags: Option<(usize, u32)>,
}

impl AnimalSlot {
    /// Este arquivo guarda a marca de Great One (saves antigos podem não guardar).
    #[must_use]
    pub fn supports_great_one(&self) -> bool {
        self.flags.is_some()
    }
}

/// O que mudar num animal; o que for `None` fica como está.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AnimalEdit {
    /// Macho (`true`) ou fêmea (`false`).
    pub male: Option<bool>,
    /// Peso em quilos.
    pub weight: Option<f32>,
    /// Pontuação do troféu.
    pub score: Option<f32>,
    /// Marca de Great One.
    pub great_one: Option<bool>,
}

fn member_named<'a>(adf: &'a Adf, type_hash: u32, name: &str) -> Option<&'a Member> {
    adf.type_def(type_hash)?
        .members
        .iter()
        .find(|m| m.name == name)
}

/// Tipo e tamanho dos elementos de um array declarado no membro `member`.
fn element_type(adf: &Adf, member: &Member) -> Option<(u32, usize)> {
    let hash = adf.type_def(member.type_hash)?.element_hash;
    Some((hash, adf.type_def(hash)?.size as usize))
}

/// Início dos elementos e quantidade de um array cujo cabeçalho está em `field`.
fn array_at(data: &[u8], base: usize, field: usize) -> Option<(usize, usize)> {
    let offset = usize::try_from(bytes::u32_at(data, field)?).ok()?;
    let count = usize::try_from(bytes::u64_at(data, field + 8)?).ok()?;
    Some((base.checked_add(offset)?, count))
}

/// Acha, em ordem de grupo e de animal, os bytes editáveis de cada animal do ADF.
///
/// # Errors
///
/// [`PopulationError`] se o ADF for inválido ou não tiver a estrutura de populações esperada.
pub fn locate_animals(adf: &Adf) -> Result<Vec<AnimalSlot>, PopulationError> {
    let data = adf.as_bytes();
    let instance = adf.instances().first().ok_or(PopulationError::Layout)?;
    let base = instance.offset as usize;
    let layout = || PopulationError::Layout;

    let populations = member_named(adf, instance.type_hash, "Populations").ok_or_else(layout)?;
    let (population_type, population_size) = element_type(adf, populations).ok_or_else(layout)?;
    let groups = member_named(adf, population_type, "Groups").ok_or_else(layout)?;
    let (group_type, group_size) = element_type(adf, groups).ok_or_else(layout)?;
    let animals = member_named(adf, group_type, "Animals").ok_or_else(layout)?;
    let (animal_type, animal_size) = element_type(adf, animals).ok_or_else(layout)?;
    let gender = member_named(adf, animal_type, "Gender").ok_or_else(layout)?;
    let weight = member_named(adf, animal_type, "Weight").ok_or_else(layout)?;
    let score = member_named(adf, animal_type, "Score").ok_or_else(layout)?;
    // Saves antigos podem não ter o campo de bits (sem Great One).
    let modifiers = member_named(adf, animal_type, "FeatureModifiers");

    let (pops_at, pops) =
        array_at(data, base, base + populations.offset as usize).ok_or_else(layout)?;
    let mut slots = Vec::new();
    let mut group_number = 0;
    for p in 0..pops {
        let population_at = pops_at + p * population_size;
        let (groups_at, group_count) =
            array_at(data, base, population_at + groups.offset as usize).ok_or_else(layout)?;
        for g in 0..group_count {
            let this_group = groups_at + g * group_size;
            let (animals_at, animal_count) =
                array_at(data, base, this_group + animals.offset as usize).ok_or_else(layout)?;
            for a in 0..animal_count {
                let at = animals_at + a * animal_size;
                if at + animal_size > data.len() {
                    return Err(PopulationError::Layout);
                }
                slots.push(AnimalSlot {
                    group: group_number,
                    index: a,
                    gender: (at + gender.offset as usize, gender.size),
                    weight: at + weight.offset as usize,
                    score: at + score.offset as usize,
                    // `FeatureModifiers` é um campo de bits de 1 byte; o bit 0 é o Great One.
                    flags: modifiers.map(|m| (at + m.offset as usize, m.size)),
                });
            }
            group_number += 1;
        }
    }
    Ok(slots)
}

fn write_int(data: &mut [u8], at: usize, size: u32, value: u32) -> Result<(), PopulationError> {
    let length = size as usize;
    let target = data
        .get_mut(at..at + length)
        .ok_or(PopulationError::Layout)?;
    match length {
        1 => target[0] = u8::try_from(value).map_err(|_| PopulationError::InvalidValue)?,
        4 => target.copy_from_slice(&value.to_le_bytes()),
        _ => return Err(PopulationError::Layout),
    }
    Ok(())
}

fn read_int(data: &[u8], at: usize, size: u32) -> Result<u32, PopulationError> {
    match size {
        1 => data
            .get(at)
            .copied()
            .map(u32::from)
            .ok_or(PopulationError::Layout),
        4 => bytes::u32_at(data, at).ok_or(PopulationError::Layout),
        _ => Err(PopulationError::Layout),
    }
}

/// Aplica edições a animais que já existem, sem mudar o tamanho do arquivo nem o resto dos bytes.
///
/// Cada edição é `(grupo, animal, mudanças)`, com as posições a partir de 0.
///
/// # Errors
///
/// [`PopulationError::NoSuchAnimal`] para um animal que não existe, [`PopulationError::InvalidValue`]
/// para peso ou pontuação que não seja um número finito e não negativo, e os demais erros de
/// leitura do ADF.
pub fn edit_animals(
    adf: &[u8],
    edits: &[(usize, usize, AnimalEdit)],
) -> Result<Vec<u8>, PopulationError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let slots = locate_animals(&parsed)?;
    let mut out = adf.to_vec();
    for (group, animal, edit) in edits {
        let slot = slots
            .iter()
            .find(|s| s.group == *group && s.index == *animal)
            .ok_or(PopulationError::NoSuchAnimal {
                group: *group,
                animal: *animal,
            })?;
        if let Some(male) = edit.male {
            write_int(
                &mut out,
                slot.gender.0,
                slot.gender.1,
                if male { 1 } else { 2 },
            )?;
        }
        for (value, at) in [(edit.weight, slot.weight), (edit.score, slot.score)] {
            if let Some(number) = value {
                if !number.is_finite() || number < 0.0 {
                    return Err(PopulationError::InvalidValue);
                }
                out.get_mut(at..at + 4)
                    .ok_or(PopulationError::Layout)?
                    .copy_from_slice(&number.to_le_bytes());
            }
        }
        if let Some(great_one) = edit.great_one {
            let (at, size) = slot.flags.ok_or(PopulationError::Layout)?;
            let current = read_int(&out, at, size)?;
            let updated = if great_one { current | 1 } else { current & !1 };
            write_int(&mut out, at, size, updated)?;
        }
    }
    Ok(out)
}
