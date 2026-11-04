//! Dados do mapa com o save e o jogo reais (pula se `fixtures/saves` ou o jogo faltarem).

#![allow(
    clippy::unwrap_used,
    clippy::assert_is_empty,
    reason = "arquivo só de testes"
)]

use std::path::PathBuf;

use cotw_core::language::Language;
use cotw_core::mapview::{build, reserves};
use cotw_formats::archive::GameArchives;

const DEFAULT_GAME_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64";

fn setup() -> Option<(PathBuf, GameArchives)> {
    let saves = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    let game = std::env::var_os("COTW_GAME_DIR")
        .map_or_else(|| PathBuf::from(DEFAULT_GAME_DIR), PathBuf::from);
    if !saves.join("found_need_zones_adf").is_file() || !game.is_dir() {
        eprintln!("save ou jogo ausentes: teste ignorado");
        return None;
    }
    Some((saves, GameArchives::open(&game).unwrap()))
}

#[test]
fn every_reserve_with_data_is_listed_by_name() {
    let Some((saves, archives)) = setup() else {
        return;
    };
    let list = reserves(&saves, &archives).unwrap();
    assert!(list.len() >= 10, "{} reservas", list.len());
    assert!(list.iter().any(|r| r.id == 2 && r.name == "Medved-Taiga"));
    // Toda reserva instalada tem a foto de capa do próprio jogo.
    for reserve in &list {
        let cover = reserve.cover.as_deref();
        assert!(
            cover.is_some_and(|c| c.starts_with("data:image/jpeg;base64,")),
            "reserva {} sem capa",
            reserve.id
        );
    }
}

#[test]
fn a_reserve_map_has_image_zones_groups_and_species() {
    let Some((saves, archives)) = setup() else {
        return;
    };
    let map = build(&saves, &archives, 2, Language::PtBr).unwrap();
    assert_eq!(map.name, "Medved-Taiga");
    assert!(
        map.image
            .as_deref()
            .unwrap()
            .starts_with("data:image/jpeg;base64,")
    );
    assert!((map.span - 16384.0).abs() < f64::EPSILON);
    assert!(map.species.iter().any(|s| s.name == "Alce"));
    assert!(map.zones.len() > 1000);
    assert!(map.zones.iter().any(|z| z.pending));
    assert!(map.zones.iter().any(|z| !z.pending));
    assert!(map.zones.iter().all(|z| (1..=3).contains(&z.kind)));
    assert!(!map.groups.is_empty());
    // Cada parada de cada grupo aponta para uma zona que existe.
    for group in &map.groups {
        assert!(group.species < map.species.len());
        assert!(group.stops.iter().all(|stop| *stop < map.zones.len()));
        assert!(!group.animals.is_empty());
    }
    // A reserva tem animais Great One no save de teste.
    let json = serde_json::to_string(&map).unwrap();
    assert!(json.contains("\"greatOne\""));
}

#[test]
fn a_reserve_without_data_is_an_error_not_a_panic() {
    let Some((saves, archives)) = setup() else {
        return;
    };
    assert!(build(&saves, &archives, 99, Language::PtBr).is_err());
}

#[test]
fn the_inferred_placement_matches_the_known_one_on_reserves_that_have_both() {
    let Some((saves, archives)) = setup() else {
        return;
    };
    let mut worst = 0.0_f64;
    for reserve in reserves(&saves, &archives).unwrap() {
        let map = build(&saves, &archives, reserve.id, Language::PtBr).unwrap();
        let Some(known) = cotw_formats::spawn_map::map_rect(reserve.id) else {
            continue;
        };
        let image = cotw_core::unlock::SpawnSource::map_image(&archives, reserve.id).unwrap();
        let points: Vec<(f32, f32)> = map.zones.iter().map(|z| (z.x, z.z)).collect();
        let inferred = cotw_core::mapview::infer_rect(&image, &points).unwrap();
        let error = (inferred.x0 - known.x0)
            .abs()
            .max((inferred.z0 - known.z0).abs());
        eprintln!("reserva {}: erro {error:.0} m", reserve.id);
        worst = worst.max(error);
    }
    assert!(worst < 1200.0, "pior erro {worst:.0} m");
}

#[test]
fn the_newest_reserves_have_their_map_image_even_without_zones() {
    let Some((_, archives)) = setup() else {
        return;
    };
    for reserve in [10, 18, 19, 20, 21] {
        let image = cotw_core::unlock::SpawnSource::map_image(&archives, reserve);
        let image = image.unwrap_or_else(|| panic!("reserva {reserve} sem imagem"));
        assert_eq!((image.width, image.height), (1024, 1024));
    }
}
