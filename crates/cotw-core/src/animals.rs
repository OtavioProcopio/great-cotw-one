//! Wiki dos animais por reserva (spec `animal-wiki`): quais espécies vivem em cada mapa, o que o
//! save mostra de cada uma e os melhores abates do diário de caça.
//!
//! As espécies de cada reserva vêm dos mapas de spawn do próprio jogo, então reservas de qualquer
//! expansão aparecem sozinhas, tenha o jogador jogado nelas ou não.

use std::collections::BTreeMap;
use std::path::Path;

use cotw_formats::adf::{Adf, Value};
use cotw_formats::hash::hashlittle;
use cotw_formats::population::parse_groups;
use cotw_formats::spawn_map::{SpawnMap, reserves_with_atlas};
use serde::Serialize;

use crate::discovery::reserve_name;
use crate::language::Language;
use crate::mapview::{self, MapError, read_adf};
use crate::species::display_name;
use crate::unlock::SpawnSource;
use crate::zones::assign_species;

/// Quantos dos melhores abates cada espécie mostra.
const TOP_HARVESTS: usize = 5;

/// O que o save diz dos animais vivos de uma espécie numa reserva.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Living {
    /// Grupos da espécie.
    pub groups: usize,
    /// Animais no total.
    pub animals: usize,
    /// Machos.
    pub males: usize,
    /// Fêmeas.
    pub females: usize,
    /// Animais Great One.
    pub great_ones: usize,
    /// Maior pontuação de troféu entre os machos.
    pub best_score: f32,
    /// Maior peso, em quilos.
    pub heaviest: f32,
}

/// Um abate do diário de caça.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Harvest {
    /// Pontuação do troféu.
    pub score: f64,
    /// Medalha (0 a 5; 0 é sem medalha).
    pub medal: u64,
    /// Quando foi, em segundos desde 1970.
    pub at: u64,
}

/// Uma espécie numa reserva.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesEntry {
    /// Nome que o jogo usa.
    pub id: String,
    /// Nome para exibir.
    pub name: String,
    /// Quantas áreas de comer, beber e descansar a espécie tem na reserva.
    pub areas: usize,
    /// Animais vivos no save (`None` se a reserva nunca foi jogada).
    pub living: Option<Living>,
    /// Melhores abates da conta, do maior para o menor.
    pub harvests: Vec<Harvest>,
}

/// Uma reserva com as suas espécies.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalReserve {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
    /// Foto de capa pequena, se o jogo a tiver.
    pub cover: Option<String>,
    /// O save tem animais nesta reserva.
    pub played: bool,
    /// Espécies, em ordem alfabética.
    pub species: Vec<SpeciesEntry>,
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn unsigned(value: Option<&Value>) -> u64 {
    match value {
        Some(Value::Unsigned(n)) => *n,
        _ => 0,
    }
}

fn items(value: Option<&Value>) -> &[Value] {
    match value {
        Some(Value::Array(items)) => items,
        _ => &[],
    }
}

/// Melhores abates por espécie (hash do nome → abates), lidos de `hunting_log_adf`.
fn harvest_log(save_dir: &Path) -> BTreeMap<u64, Vec<Harvest>> {
    const FILE: &str = "hunting_log_adf";
    let mut log: BTreeMap<u64, Vec<Harvest>> = BTreeMap::new();
    let Ok(bytes) = read_adf(&save_dir.join(FILE), FILE) else {
        return log;
    };
    let Ok(parsed) = Adf::parse(bytes) else {
        return log;
    };
    let Some(root) = parsed
        .instances()
        .first()
        .and_then(|i| parsed.read_instance(i).ok())
    else {
        return log;
    };
    for entry in items(member(&root, "TopHarvestsBySpecies")) {
        // O nome da espécie do grupo (hash do nome do jogo); o de cada abate é o da variação.
        let species = unsigned(member(entry, "SpeciesName"));
        for harvest in items(member(entry, "TopHarvest")) {
            let score = match member(harvest, "Score") {
                Some(Value::Float(n)) => *n,
                _ => 0.0,
            };
            log.entry(species).or_default().push(Harvest {
                score,
                medal: unsigned(member(harvest, "TrophyScore")),
                at: unsigned(member(harvest, "Timestamp")),
            });
        }
    }
    for list in log.values_mut() {
        list.sort_by(|a, b| b.score.total_cmp(&a.score));
        list.truncate(TOP_HARVESTS);
    }
    log
}

fn living_by_species(
    save_dir: &Path,
    reserve: u32,
    maps: &[SpawnMap],
) -> Option<BTreeMap<String, Living>> {
    let file = format!("animal_population_{reserve}");
    let groups = parse_groups(&read_adf(&save_dir.join(&file), &file).ok()?).ok()?;
    if groups.is_empty() {
        return None;
    }
    let (species_of_group, _) = assign_species(&groups, maps);
    let mut found: BTreeMap<String, Living> = BTreeMap::new();
    for (group, species) in groups.iter().zip(species_of_group) {
        let Some(species) = species else {
            continue;
        };
        let living = found.entry(species).or_insert(Living {
            groups: 0,
            animals: 0,
            males: 0,
            females: 0,
            great_ones: 0,
            best_score: 0.0,
            heaviest: 0.0,
        });
        living.groups += 1;
        for animal in &group.animals {
            living.animals += 1;
            if animal.male {
                living.males += 1;
                living.best_score = living.best_score.max(animal.score);
            } else {
                living.females += 1;
            }
            living.great_ones += usize::from(animal.great_one);
            living.heaviest = living.heaviest.max(animal.weight);
        }
    }
    Some(found)
}

/// Monta a wiki: todas as reservas que o jogo tem, com as espécies e o que o save sabe delas.
///
/// # Errors
///
/// [`MapError`] só se o arquivo de zonas do save não puder ser lido (ele define quais reservas existem).
pub fn animals(
    save_dir: &Path,
    source: &dyn SpawnSource,
    language: Language,
) -> Result<Vec<AnimalReserve>, MapError> {
    let log = harvest_log(save_dir);
    let mut found = Vec::new();
    for reserve in reserves_with_atlas() {
        let Ok(maps) = source.maps(reserve) else {
            continue;
        };
        if maps.is_empty() {
            continue;
        }
        let living = living_by_species(save_dir, reserve, &maps);
        let mut areas: BTreeMap<&str, usize> = BTreeMap::new();
        for map in &maps {
            *areas.entry(map.species.as_str()).or_default() += map.areas.len();
        }
        let mut species: Vec<SpeciesEntry> = areas
            .into_iter()
            .map(|(id, count)| SpeciesEntry {
                id: id.to_owned(),
                name: display_name(id, language),
                areas: count,
                living: living.as_ref().and_then(|l| l.get(id)).cloned(),
                harvests: log
                    .get(&u64::from(hashlittle(id.as_bytes(), 0)))
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();
        species.sort_by(|a, b| a.name.cmp(&b.name));
        found.push(AnimalReserve {
            id: reserve,
            name: reserve_name(reserve),
            cover: mapview::cover_data_url(source, reserve),
            played: living.is_some(),
            species,
        });
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_helpers_default_to_empty() {
        assert_eq!(unsigned(Some(&Value::Float(1.0))), 0);
        assert_eq!(unsigned(Some(&Value::Unsigned(3))), 3);
        assert_eq!(unsigned(None), 0);
        assert_eq!(items(None).len(), 0);
        assert!(member(&Value::Unsigned(1), "x").is_none());
    }

    #[test]
    fn a_missing_hunting_log_means_no_harvests() {
        assert!(harvest_log(Path::new("não existe")).is_empty());
    }

    #[test]
    fn a_missing_population_means_the_reserve_was_never_played() {
        assert!(living_by_species(Path::new("não existe"), 2, &[]).is_none());
    }
}
