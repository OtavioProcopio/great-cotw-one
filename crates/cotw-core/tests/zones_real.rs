//! "Liberar o mapa" contra o save e o jogo reais (pula se `fixtures/saves` ou o jogo faltarem).
//!
//! A prova de que o cálculo acerta: esconder zonas que o próprio jogo gravou, calcular de novo e
//! comparar posição, altura e horário com o que o jogo tinha.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    reason = "arquivo só de testes"
)]

use std::collections::HashMap;
use std::path::PathBuf;

use cotw_core::zones::{ReserveInput, plan_reserve};
use cotw_formats::archive::GameArchives;
use cotw_formats::envelope::SaveEnvelope;
use cotw_formats::need_zones::{NeedZones, Zone};
use cotw_formats::population::parse_groups;
use cotw_formats::spawn_map::{SpawnMap, load_reserve};

const DEFAULT_GAME_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64";

fn game_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("COTW_GAME_DIR")
        .map_or_else(|| PathBuf::from(DEFAULT_GAME_DIR), PathBuf::from);
    dir.is_dir().then_some(dir)
}

fn saves_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    dir.join("found_need_zones_adf").is_file().then_some(dir)
}

fn adf_of(path: &std::path::Path) -> Vec<u8> {
    SaveEnvelope::parse(&std::fs::read(path).unwrap())
        .unwrap()
        .adf()
        .to_vec()
}

struct Real {
    zones: NeedZones,
    original: Vec<u8>,
    archives: GameArchives,
    saves: PathBuf,
}

fn real() -> Option<Real> {
    let (saves, game) = (saves_dir()?, game_dir()?);
    let original = adf_of(&saves.join("found_need_zones_adf"));
    Some(Real {
        zones: NeedZones::parse(&original).unwrap(),
        original,
        archives: GameArchives::open(&game).unwrap(),
        saves,
    })
}

fn reserve_inputs(
    real: &Real,
    id: u32,
) -> Option<(Vec<cotw_formats::population::Group>, Vec<SpawnMap>)> {
    let population = real.saves.join(format!("animal_population_{id}"));
    if !population.is_file() {
        return None;
    }
    let groups = parse_groups(&adf_of(&population)).unwrap();
    let maps = load_reserve(&real.archives, id).ok()?;
    Some((groups, maps))
}

type IndexedZone<'a> = (usize, &'a Zone);

fn is_game_written(zone: &Zone) -> bool {
    zone.y.to_bits() != 1.0_f32.to_bits()
}

#[test]
fn every_reserve_with_data_gets_a_valid_plan() {
    let Some(real) = real() else {
        eprintln!("save ou jogo ausentes: teste ignorado");
        return;
    };
    let mut total_new = 0;
    for reserve in &real.zones.reserves {
        let id = u32::try_from(reserve.id).unwrap();
        let Some((groups, maps)) = reserve_inputs(&real, id) else {
            continue;
        };
        let plan = plan_reserve(&ReserveInput {
            reserve: id,
            groups: &groups,
            maps: &maps,
            existing: &reserve.zones,
        })
        .unwrap_or_else(|e| panic!("reserva {id}: {e}"));
        eprintln!(
            "reserva {id}: {} novas, {} estimadas, {} sem horário, {} sem altura, {} áreas fora do atlas",
            plan.new_zones.len(),
            plan.estimated,
            plan.no_schedule,
            plan.no_height,
            plan.unknown_areas
        );
        let mut keys: Vec<u32> = plan.new_zones.iter().map(|z| z.key).collect();
        keys.dedup();
        assert_eq!(
            keys.len(),
            plan.new_zones.len(),
            "reserva {id}: chaves repetidas"
        );
        assert!(
            plan.new_zones.iter().all(is_game_written),
            "reserva {id}: altura 1,0"
        );
        total_new += plan.new_zones.len();
    }
    assert!(total_new > 0);
}

/// Esconde um décimo das zonas do jogo, recalcula e compara com o que o jogo tinha gravado.
#[test]
fn hidden_game_zones_are_predicted_correctly() {
    let Some(real) = real() else {
        eprintln!("save ou jogo ausentes: teste ignorado");
        return;
    };
    let (mut checked, mut position_ok, mut kind_ok, mut constant_ok, mut slot_ok) = (0, 0, 0, 0, 0);
    let mut height_errors: Vec<f32> = Vec::new();
    for reserve in &real.zones.reserves {
        let id = u32::try_from(reserve.id).unwrap();
        let Some((groups, maps)) = reserve_inputs(&real, id) else {
            continue;
        };
        let used: std::collections::HashSet<u32> = groups
            .iter()
            .flat_map(|g| g.route.iter().copied())
            .collect();
        let (hidden, kept): (Vec<IndexedZone>, Vec<IndexedZone>) = reserve
            .zones
            .iter()
            .enumerate()
            .partition(|(i, z)| is_game_written(z) && i % 10 == 0 && used.contains(&z.key));
        let existing: Vec<Zone> = kept.into_iter().map(|(_, z)| z.clone()).collect();
        let Ok(plan) = plan_reserve(&ReserveInput {
            reserve: id,
            groups: &groups,
            maps: &maps,
            existing: &existing,
        }) else {
            continue;
        };
        let predicted: HashMap<u32, &Zone> = plan.new_zones.iter().map(|z| (z.key, z)).collect();
        for (_, truth) in hidden {
            let Some(zone) = predicted.get(&truth.key) else {
                continue;
            };
            checked += 1;
            position_ok +=
                usize::from((zone.x - truth.x).abs() < 0.01 && (zone.z - truth.z).abs() < 0.01);
            kind_ok += usize::from(zone.kind == truth.kind);
            constant_ok += usize::from(zone.species_name == truth.species_name);
            slot_ok += usize::from(
                zone.start.to_bits() == truth.start.to_bits()
                    && zone.end.to_bits() == truth.end.to_bits(),
            );
            height_errors.push((zone.y - truth.y).abs());
        }
    }
    height_errors.sort_by(f32::total_cmp);
    let percent = |n: usize| 100.0 * n as f64 / checked.max(1) as f64;
    eprintln!(
        "zonas escondidas e previstas: {checked}; posição {:.1}%, tipo {:.1}%, constante {:.1}%, horário {:.1}%; \
         erro de altura: mediana {:.1} m, p90 {:.1} m",
        percent(position_ok),
        percent(kind_ok),
        percent(constant_ok),
        percent(slot_ok),
        height_errors
            .get(height_errors.len() / 2)
            .copied()
            .unwrap_or(0.0),
        height_errors
            .get(height_errors.len() * 9 / 10)
            .copied()
            .unwrap_or(0.0),
    );
    assert!(checked > 100, "poucas zonas conferidas: {checked}");
    assert_eq!(position_ok, checked, "posição tem de ser exata");
    assert_eq!(kind_ok, checked, "tipo tem de ser exato");
    // Áreas compartilhadas por espécies diferentes podem receber a constante da outra espécie.
    assert!(
        percent(constant_ok) > 98.0,
        "constante certa em {:.1}%",
        percent(constant_ok)
    );
    assert!(
        percent(slot_ok) > 60.0,
        "horário certo em {:.1}%",
        percent(slot_ok)
    );
}

#[test]
fn unlocking_every_reserve_writes_a_valid_file() {
    let Some(mut real) = real() else {
        eprintln!("save ou jogo ausentes: teste ignorado");
        return;
    };
    let before: Vec<usize> = real.zones.reserves.iter().map(|r| r.zones.len()).collect();
    let mut added = 0;
    let ids: Vec<u64> = real.zones.reserves.iter().map(|r| r.id).collect();
    for id in ids {
        let id32 = u32::try_from(id).unwrap();
        let Some((groups, maps)) = reserve_inputs(&real, id32) else {
            continue;
        };
        let reserve = real.zones.reserve_mut(id).unwrap();
        let Ok(plan) = plan_reserve(&ReserveInput {
            reserve: id32,
            groups: &groups,
            maps: &maps,
            existing: &reserve.zones,
        }) else {
            continue;
        };
        added += plan.new_zones.len();
        reserve.zones.extend(plan.new_zones);
    }
    let rebuilt = real.zones.to_adf(&real.original).unwrap();
    let reread = NeedZones::parse(&rebuilt).unwrap();
    assert_eq!(reread, real.zones);
    let after: usize = reread.reserves.iter().map(|r| r.zones.len()).sum();
    assert_eq!(after, before.iter().sum::<usize>() + added);
    eprintln!(
        "{added} zonas novas; arquivo foi de {} para {} bytes",
        real.original.len(),
        rebuilt.len()
    );
}
