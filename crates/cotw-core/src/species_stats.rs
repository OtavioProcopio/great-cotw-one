//! Faixas de peso e de pontos, medalhas e pelagens de cada espécie.
//!
//! Os números vêm de `data/species_stats.json`, gerado por `tools/gen-species-stats` a partir da
//! instalação do jogo (só fatos numéricos e chaves de nome, nenhum arquivo do jogo). A medalha de um
//! animal depende de onde a sua pontuação cai entre o mínimo e o máximo da espécie.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

const DATA: &str = include_str!("../data/species_stats.json");

/// Faixa `[mínimo, máximo]`.
pub type Range = [f64; 2];

/// Faixas de um grupo (macho, fêmea, Great One).
#[derive(Debug, Clone, Deserialize)]
pub struct Scoring {
    /// Peso em kg.
    pub weight: Range,
    /// Pontos do troféu.
    pub score: Range,
}

/// Uma pelagem possível da espécie.
#[derive(Debug, Clone, Deserialize)]
pub struct Fur {
    /// Número da pelagem na espécie.
    pub index: u32,
    /// Chave do nome (`piebald`, `dark_red`, `albino`…).
    pub key: String,
    /// Raridade: 0 comum, 1 incomum, 2 rara, 3 muito rara.
    pub rarity: u32,
    /// Peso no sorteio (0: só aparece por missão).
    pub odds: u32,
    /// Pelagem exclusiva do Great One.
    pub great_one: bool,
}

/// Tudo o que se sabe de uma espécie.
#[derive(Debug, Clone, Deserialize)]
pub struct SpeciesStats {
    /// Tipo de troféu (`weight`, `skull`, `antlers`, `tusks`, `length`, `none`).
    pub trophy: String,
    /// Classe do animal no jogo (`deer`, `cat`, `wolf`, `duck`…; vazia se o jogo não informa).
    #[serde(default)]
    pub group: String,
    /// Faixas por grupo: `Male_Scoring`, `Female_Scoring`, `Male_GreatOne_Scoring`, `Female_GreatOne_Scoring`.
    pub scoring: BTreeMap<String, Scoring>,
    /// Pelagens.
    pub furs: Vec<Fur>,
}

/// Medalha de um troféu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Medal {
    /// Abaixo de 20% da faixa.
    Bronze,
    /// De 20% a 60%.
    Silver,
    /// De 60% a 90%.
    Gold,
    /// A partir de 90%.
    Diamond,
}

#[derive(Deserialize)]
struct File {
    species: BTreeMap<String, SpeciesStats>,
}

fn table() -> &'static BTreeMap<String, SpeciesStats> {
    static TABLE: OnceLock<BTreeMap<String, SpeciesStats>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str::<File>(DATA)
            .map(|f| f.species)
            .unwrap_or_default()
    })
}

/// Dados da espécie (`id` do jogo, como em `jaguar`), se ela existir.
#[must_use]
pub fn get(species: &str) -> Option<&'static SpeciesStats> {
    table().get(species)
}

/// Nomes que o mapa de aparição usa e que diferem dos da tabela de espécies (apelido, nome na tabela).
const ALIASES: &[(&str, &str)] = &[
    ("bean_goose", "tundra_bean_goose"),
    ("bison_upper", "eu_bison"),
    ("european_bison", "eu_bison"),
    ("blacktail", "blacktail_deer"),
    ("boar", "wild_boar"),
    ("wildhog", "wild_boar"),
    ("brown_bear", "eurasian_brown_bear"),
    ("capercaillie", "western_capercaillie"),
    ("elk", "roosevelt_elk"),
    ("european_hare", "eu_hare"),
    ("fallow", "fallow_deer"),
    ("grey_fox", "gray_fox"),
    ("jackal", "sidestriped_jackal"),
    ("side_striped_jackal", "sidestriped_jackal"),
    ("musk_deer", "siberian_musk_deer"),
    ("peccary", "collared_peccary"),
    ("pronghorn", "prong_horn"),
    ("rocky_mountain_elk", "rockymountain_elk"),
    ("roe", "roe_deer"),
    ("southeastern_spanish_ibex", "southeastern_ibex"),
    ("whitetail", "whitetail_deer"),
];

/// Como [`get`], mas aceita também os apelidos que o mapa de aparição usa para algumas espécies.
#[must_use]
pub fn lookup(species: &str) -> Option<&'static SpeciesStats> {
    get(species).or_else(|| {
        ALIASES
            .iter()
            .find(|(alias, _)| *alias == species)
            .and_then(|(_, id)| get(id))
    })
}

/// Todas as espécies conhecidas, em ordem alfabética do `id`.
pub fn all() -> impl Iterator<Item = (&'static str, &'static SpeciesStats)> {
    table().iter().map(|(id, s)| (id.as_str(), s))
}

impl SpeciesStats {
    /// Faixa do macho (o grupo que define as medalhas).
    #[must_use]
    pub fn male(&self) -> Option<&Scoring> {
        self.scoring.get("Male_Scoring")
    }

    /// Pontuação mínima para a medalha, como fração da faixa do macho (`0.9` = diamante).
    fn threshold(&self, fraction: f64) -> Option<f64> {
        let [lo, hi] = self.male()?.score;
        (hi > lo).then_some(lo + fraction * (hi - lo))
    }

    /// Pontuação a partir da qual o macho é diamante.
    #[must_use]
    pub fn diamond_min(&self) -> Option<f64> {
        self.threshold(0.9)
    }

    /// Medalha de uma pontuação de macho; `None` se a espécie não tem faixa de pontos.
    #[must_use]
    pub fn medal(&self, score: f64) -> Option<Medal> {
        let [lo, hi] = self.male()?.score;
        if hi <= lo {
            return None;
        }
        let fraction = (score - lo) / (hi - lo);
        Some(if fraction >= 0.9 {
            Medal::Diamond
        } else if fraction >= 0.6 {
            Medal::Gold
        } else if fraction >= 0.2 {
            Medal::Silver
        } else {
            Medal::Bronze
        })
    }

    /// Chance (0 a 1) de cada pelagem comum da espécie, pelo peso de sorteio; as de Great One e as
    /// só de missão ficam de fora.
    #[must_use]
    pub fn fur_chances(&self) -> Vec<(&Fur, f64)> {
        let regular: Vec<&Fur> = self
            .furs
            .iter()
            .filter(|f| !f.great_one && f.odds > 0)
            .collect();
        let total: f64 = regular.iter().map(|f| f64::from(f.odds)).sum();
        regular
            .into_iter()
            .map(|f| {
                (
                    f,
                    if total > 0.0 {
                        f64::from(f.odds) / total
                    } else {
                        0.0
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) -> bool {
        // Os guias arredondam (177,5 para 177,58), então 0,1 de folga basta.
        (a - b).abs() < 0.1
    }

    #[test]
    fn the_table_loads_and_has_every_species() {
        assert!(all().count() >= 130);
        assert!(get("jaguar").is_some());
        assert!(get("not_an_animal").is_none());
    }

    #[test]
    fn diamond_thresholds_match_the_values_confirmed_by_the_community() {
        for (id, expected) in [
            ("moose", 274.99),
            ("reindeer", 430.23),
            ("wild_boar", 144.25),
            ("whitetail_deer", 255.09),
            ("blacktail_deer", 177.5),
        ] {
            let got = get(id).and_then(SpeciesStats::diamond_min).unwrap();
            assert!(near(got, expected), "{id}: {got} != {expected}");
        }
    }

    #[test]
    fn the_jaguar_has_its_own_great_one_furs_and_a_mission_only_one() {
        let jaguar = get("jaguar").unwrap();
        assert!(near(jaguar.diamond_min().unwrap(), 44.4));
        assert_eq!(jaguar.furs.iter().filter(|f| f.great_one).count(), 7);
        let old_scar = jaguar.furs.iter().find(|f| f.key == "oldscar").unwrap();
        assert_eq!(old_scar.odds, 0);
        assert!(jaguar.fur_chances().iter().all(|(f, _)| f.key != "oldscar"));
    }

    #[test]
    fn every_alias_points_to_a_species_in_the_table() {
        for (alias, id) in ALIASES {
            assert!(get(id).is_some(), "{alias} -> {id}");
            assert!(lookup(alias).is_some());
        }
        assert!(lookup("boar").is_some() && get("boar").is_none());
    }

    #[test]
    fn medals_follow_the_percentage_of_the_range() {
        let moose = get("moose").unwrap();
        assert_eq!(moose.medal(40.0), Some(Medal::Bronze));
        assert_eq!(moose.medal(150.0), Some(Medal::Silver));
        assert_eq!(moose.medal(250.0), Some(Medal::Gold));
        assert_eq!(moose.medal(280.0), Some(Medal::Diamond));
    }

    #[test]
    fn fur_chances_add_up_to_one() {
        for (id, stats) in all() {
            let total: f64 = stats.fur_chances().iter().map(|(_, c)| c).sum();
            assert!(total == 0.0 || near(total, 1.0), "{id}: {total}");
        }
    }
}
