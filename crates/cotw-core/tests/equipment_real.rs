//! Equipamentos lidos do perfil do save real (pula se `fixtures/saves` não existir).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::path::PathBuf;

use cotw_core::equipment::{Kind, inventory};
use cotw_core::language::Language;

fn saves() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    dir.join("thp_player_profile_adf").is_file().then_some(dir)
}

#[test]
fn the_real_profile_owns_weapons_with_history_and_loadouts() {
    let Some(saves) = saves() else {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    };
    let inv = inventory(&saves, Language::PtBr).unwrap();
    let owned_weapons = inv
        .items
        .iter()
        .filter(|i| i.kind == Kind::Weapon && i.owned)
        .count();
    assert!(owned_weapons >= 5, "armas: {owned_weapons}");
    assert!(
        inv.items
            .iter()
            .any(|i| i.kind == Kind::Weapon && i.stats.as_ref().is_some_and(|s| s.shots_fired > 0))
    );
    assert!(
        inv.items
            .iter()
            .any(|i| i.kind == Kind::Ammo && i.owned && i.amount > 0)
    );
    assert!(inv.loadouts.len() >= 3);
    assert_eq!(inv.loadouts.iter().filter(|l| l.active).count(), 1);
    assert!(inv.weapon_levels.iter().all(|l| *l > 0));
}

const GAME_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64";

#[test]
fn icons_are_read_from_the_installed_game_and_cached() {
    let dir = std::path::Path::new(GAME_DIR);
    if !dir.is_dir() {
        eprintln!("jogo ausente: teste ignorado");
        return;
    }
    let archives = cotw_formats::archive::GameArchives::open(dir).unwrap();
    let cache = tempfile::tempdir().unwrap();
    let ids: Vec<String> = [
        "equipment_weapon_ba_rifle_270_01",
        "equipment_ammo_270_sp_01",
        "equipment_sight_rifle_scope_3_9x44mm_01",
    ]
    .map(str::to_owned)
    .to_vec();
    let found = cotw_core::equipment::icons(Some(&archives), cache.path(), &ids);
    assert_eq!(found.len(), 3);
    assert!(
        found
            .values()
            .all(|url| url.starts_with("data:image/png;base64,"))
    );
    // Segunda vez: vem do cache, sem precisar do jogo.
    let again = cotw_core::equipment::icons(None, cache.path(), &ids);
    assert_eq!(again, found);
}
