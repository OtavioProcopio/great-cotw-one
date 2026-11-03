//! `animal_population_N`: grupos de animais de uma reserva e as rotas diárias deles.
//!
//! Só o que o app precisa: a rota (sequência cíclica de chaves de áreas) de cada grupo.

use crate::adf::{Adf, AdfError, Value};

/// Erros ao ler a população.
#[derive(Debug, thiserror::Error)]
pub enum PopulationError {
    /// O ADF é inválido.
    #[error("ADF inválido: {0}")]
    Adf(#[from] AdfError),
    /// A estrutura não é a esperada.
    #[error("animal_population com estrutura inesperada")]
    Layout,
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
