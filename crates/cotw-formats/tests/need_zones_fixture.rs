//! `found_need_zones_adf` real: reconstruir sem mudanças devolve os mesmos bytes e acrescentar
//! zonas deixa as antigas intactas (pula se `fixtures/saves` não existir).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

mod common;

use common::saves;
use cotw_formats::need_zones::{NeedZones, Zone};

fn zones_adf() -> Option<Vec<u8>> {
    saves()
        .into_iter()
        .find(|s| s.name == "found_need_zones_adf")
        .map(|s| s.envelope.adf().to_vec())
}

#[test]
fn rebuilding_without_changes_gives_back_the_same_bytes() {
    let Some(original) = zones_adf() else { return };
    let zones = NeedZones::parse(&original).unwrap();
    assert_ne!(zones.reserves, []);
    assert_eq!(zones.to_adf(&original).unwrap(), original);
}

#[test]
fn adding_zones_keeps_the_old_ones_and_a_valid_chain() {
    let Some(original) = zones_adf() else { return };
    let before = NeedZones::parse(&original).unwrap();
    let mut after = before.clone();
    let reserve = after
        .reserves
        .iter_mut()
        .find(|r| !r.zones.is_empty())
        .unwrap();
    let id = reserve.id;
    let old_count = reserve.zones.len();
    for k in 0..5u16 {
        reserve.zones.push(Zone::new(
            (1000.0 + 32.0 * f32::from(k), 250.5, -2000.0),
            0xFFFF_0000 + u32::from(k),
            1,
            0x0102_0304_0506_0000 + u64::from(k),
            (4.0, 8.0),
            7,
            1,
        ));
    }

    let rebuilt = after.to_adf(&original).unwrap();
    let reread = NeedZones::parse(&rebuilt).unwrap();

    assert_eq!(reread, after);
    let reserve = reread.reserve(id).unwrap();
    assert_eq!(reserve.zones.len(), old_count + 5);
    assert_eq!(
        &reserve.zones[..old_count],
        &before.reserve(id).unwrap().zones[..]
    );
    assert!(rebuilt.len() > original.len());
}
