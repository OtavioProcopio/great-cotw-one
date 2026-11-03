//! Perfil e missões com o save, o jogo e a Steam reais (pula o que faltar no computador).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::path::{Path, PathBuf};

use cotw_core::career::{missions, profile};
use cotw_formats::archive::GameArchives;

const GAME_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64";
const STATS_DIR: &str = r"C:\Program Files (x86)\Steam\appcache\stats";
const STEAM_ID: &str = "76561198125950350";

fn saves() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    dir.join("thp_player_profile_adf").is_file().then_some(dir)
}

#[test]
fn the_profile_has_level_shots_trophies_and_the_steam_achievements() {
    let Some(saves) = saves() else {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    };
    let stats = Path::new(STATS_DIR);
    let view = profile(&saves, STEAM_ID, stats.is_dir().then_some(stats)).unwrap();
    assert!(view.level >= 1 && view.xp > 0);
    assert!(view.shots_fired >= view.total_kills);
    assert!(view.shots_missed <= view.shots_fired);
    assert!(view.reserves >= 10);
    if stats.is_dir() {
        assert!(view.has_achievements);
        assert_eq!(view.achievements.len(), 160);
        assert!(view.achievements.iter().any(|a| a.unlocked));
        assert!(view.achievements.iter().any(|a| !a.unlocked));
        // Conquistas escondidas só mostram o nome depois de obtidas (a descrição segue a da Steam).
        assert!(view.achievements.iter().all(|a| !a.name.is_empty()));
    }
}

#[test]
fn a_profile_without_steam_files_has_no_achievements() {
    let Some(saves) = saves() else {
        return;
    };
    let view = profile(&saves, STEAM_ID, None).unwrap();
    assert!(!view.has_achievements && view.achievements.is_empty());
}

#[test]
fn every_reserve_of_the_game_gets_its_missions_with_text() {
    let Some(saves) = saves() else {
        return;
    };
    let game = Path::new(GAME_DIR);
    if !game.is_dir() {
        eprintln!("jogo ausente: teste ignorado");
        return;
    }
    let list = missions(&saves, &GameArchives::open(game).unwrap()).unwrap();
    assert!(list.len() >= 10, "{} reservas", list.len());
    let medved = list.iter().find(|r| r.id == 2).unwrap();
    assert_eq!(medved.total, 92);
    assert_eq!(medved.total, medved.missions.len());
    assert!(medved.done > 0 && medved.done < medved.total);
    assert!(medved.main_total > 0 && medved.main_done <= medved.main_total);
    let with_text = medved
        .missions
        .iter()
        .filter(|m| !m.text.is_empty())
        .count();
    assert!(
        with_text * 10 >= medved.total * 8,
        "só {with_text} missões com texto"
    );
    assert!(
        medved
            .missions
            .iter()
            .any(|m| m.state == "active" && m.objectives.iter().any(|o| o.required > 0))
    );
}

#[test]
fn the_animal_wiki_lists_every_reserve_of_the_game_with_species_and_save_data() {
    use cotw_core::animals::animals;
    let Some(saves) = saves() else {
        return;
    };
    let game = Path::new(GAME_DIR);
    if !game.is_dir() {
        return;
    }
    let list = animals(&saves, &GameArchives::open(game).unwrap()).unwrap();
    // Todas as reservas com mapa de spawn no jogo, jogadas ou não.
    assert!(list.len() >= 17, "{} reservas", list.len());
    assert!(
        list.iter().any(|r| !r.played),
        "alguma reserva nunca jogada"
    );
    assert!(list.iter().all(|r| !r.species.is_empty()));
    let medved = list.iter().find(|r| r.id == 2).unwrap();
    assert!(medved.played);
    let moose = medved.species.iter().find(|s| s.id == "moose").unwrap();
    assert_eq!(moose.name, "Alce");
    assert!(moose.areas > 0);
    assert!(
        moose
            .living
            .as_ref()
            .is_some_and(|l| l.animals > 0 && l.males + l.females == l.animals)
    );
    assert!(
        list.iter()
            .flat_map(|r| &r.species)
            .any(|s| !s.harvests.is_empty())
    );
    assert!(list.iter().filter(|r| r.cover.is_some()).count() >= 14);
}
