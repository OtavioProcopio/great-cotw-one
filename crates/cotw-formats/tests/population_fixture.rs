//! Grupos e animais de `animal_population_N` reais (pula se `fixtures/saves` não existir).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

mod common;

use common::saves;
use cotw_formats::population::{Group, parse_groups};

fn populations() -> Vec<(String, Vec<Group>)> {
    saves()
        .into_iter()
        .filter(|s| s.name.starts_with("animal_population_"))
        .map(|s| (s.name.clone(), parse_groups(s.envelope.adf()).unwrap()))
        .collect()
}

#[test]
fn every_real_population_reads_and_the_played_ones_have_routes_and_animals() {
    let all = populations();
    for (name, groups) in &all {
        for animal in groups.iter().flat_map(|g| &g.animals) {
            assert!(animal.weight >= 0.0 && animal.score >= 0.0, "{name}");
        }
    }
    // Reservas que o jogador nunca abriu têm população vazia; as jogadas têm grupos completos.
    let played: Vec<_> = all
        .iter()
        .filter(|(_, groups)| !groups.is_empty())
        .collect();
    assert_eq!(played.is_empty(), all.is_empty());
    for (name, groups) in played {
        assert!(groups.iter().all(|g| !g.route.is_empty()), "{name}");
        assert!(groups.iter().any(|g| !g.animals.is_empty()), "{name}");
    }
}

#[test]
fn some_real_group_has_both_sexes() {
    let all = populations();
    let found = all.iter().any(|(_, groups)| {
        groups
            .iter()
            .any(|g| g.animals.iter().any(|a| a.male) && g.animals.iter().any(|a| !a.male))
    });
    // Sem saves reais o teste não confere nada.
    assert!(found || all.is_empty());
}
