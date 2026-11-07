//! Editor de equipamentos sobre o perfil real (pula se `fixtures/saves` não existir): mesmo tamanho,
//! item entregue aparece na leitura e o conferidor da gravação aceita o arquivo.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::path::PathBuf;

use cotw_core::equipment::{Kind, inventory};
use cotw_core::equipment_editor::{EquipmentChange, build_edit};
use cotw_core::language::Language;
use cotw_formats::envelope::SaveEnvelope;

const PROFILE: &str = "thp_player_profile_adf";

fn fixture() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    dir.join(PROFILE).is_file().then_some(dir)
}

#[test]
fn giving_items_and_cash_keeps_the_size_and_shows_up_when_read_back() {
    let Some(saves) = fixture() else {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    };
    let work = tempfile::tempdir().unwrap();
    std::fs::copy(saves.join(PROFILE), work.path().join(PROFILE)).unwrap();
    let before = inventory(work.path(), Language::PtBr).unwrap();
    let missing_weapon = before
        .items
        .iter()
        .find(|i| i.kind == Kind::Weapon && !i.owned && i.class.is_some())
        .unwrap()
        .id
        .clone();
    let ammo = before
        .items
        .iter()
        .find(|i| i.kind == Kind::Ammo && i.owned)
        .unwrap();
    let (ammo_id, ammo_was) = (ammo.id.clone(), ammo.amount);

    let plan = build_edit(
        work.path(),
        &[
            EquipmentChange::Give {
                id: missing_weapon.clone(),
                amount: None,
            },
            EquipmentChange::SetAmount {
                id: ammo_id.clone(),
                amount: 321,
            },
            EquipmentChange::SetCash { cash: 123_456 },
        ],
    )
    .unwrap();
    assert_eq!(plan.changes, 3);
    let old = SaveEnvelope::parse(&std::fs::read(work.path().join(PROFILE)).unwrap()).unwrap();
    let new = SaveEnvelope::parse(&plan.file.new_content).unwrap();
    assert_eq!(new.adf().len(), old.adf().len());
    let differing = new
        .adf()
        .iter()
        .zip(old.adf())
        .filter(|(a, b)| a != b)
        .count();
    assert!(differing < 80, "bytes diferentes: {differing}");
    plan.file.verify.as_ref().unwrap()(&plan.file.new_content).unwrap();

    std::fs::write(work.path().join(PROFILE), &plan.file.new_content).unwrap();
    let after = inventory(work.path(), Language::PtBr).unwrap();
    assert_eq!(after.cash, 123_456);
    let weapon = after.items.iter().find(|i| i.id == missing_weapon).unwrap();
    assert!(weapon.owned && weapon.unlocked && weapon.stats.is_some());
    let ammo = after.items.iter().find(|i| i.id == ammo_id).unwrap();
    assert_eq!(ammo.amount, 321);
    assert_ne!(ammo_was, 321);
    // O resto do inventário não muda.
    let owned =
        |inv: &cotw_core::equipment::Inventory| inv.items.iter().filter(|i| i.owned).count();
    assert_eq!(owned(&after), owned(&before) + 1);
    assert_eq!(after.loadouts, before.loadouts);
}
