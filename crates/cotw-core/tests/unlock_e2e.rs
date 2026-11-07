//! "Liberar o mapa" de ponta a ponta com o save e o jogo reais: prévia → gravação segura com
//! backup → conferência → restauração (pula se `fixtures/saves` ou o jogo faltarem).
//! Tudo acontece numa cópia temporária; o save e o jogo do desenvolvedor não são tocados.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use cotw_core::diagnostics::SearchRoots;
use cotw_core::license::FixedLicense;
use cotw_core::process::ProcessList;
use cotw_core::service::{AppService, UnlockRequest};
use cotw_core::unlock::{ReserveState, ZONES_FILE};
use cotw_formats::envelope::SaveEnvelope;
use cotw_formats::need_zones::NeedZones;

const DEFAULT_GAME_ROOT: &str = r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW";

struct Closed;

impl ProcessList for Closed {
    fn running_image_names(&self) -> io::Result<Vec<String>> {
        Ok(vec!["explorer.exe".into()])
    }
}

struct Open;

impl ProcessList for Open {
    fn running_image_names(&self) -> io::Result<Vec<String>> {
        Ok(vec!["theHunterCotW_F.exe".into()])
    }
}

fn game_root() -> Option<PathBuf> {
    let root = std::env::var_os("COTW_GAME_ROOT")
        .map_or_else(|| PathBuf::from(DEFAULT_GAME_ROOT), PathBuf::from);
    root.join("archives_win64").is_dir().then_some(root)
}

struct Setup {
    _work: tempfile::TempDir,
    appdata: PathBuf,
    save: PathBuf,
    docs: PathBuf,
    game: PathBuf,
}

fn setup() -> Option<Setup> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    let game = game_root()?;
    if !fixtures.join(ZONES_FILE).is_file() {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return None;
    }
    let work = tempfile::tempdir().unwrap();
    let docs = work.path().join("docs");
    let save = docs.join("Avalanche Studios/COTW/Saves/111");
    fs::create_dir_all(&save).unwrap();
    for entry in fs::read_dir(&fixtures).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() && entry.file_name() != "_index.oracle.txt" {
            fs::copy(entry.path(), save.join(entry.file_name())).unwrap();
        }
    }
    Some(Setup {
        appdata: work.path().join("appdata"),
        _work: work,
        save,
        docs,
        game,
    })
}

fn premium_service(setup: &Setup, processes: Box<dyn ProcessList + Send + Sync>) -> AppService {
    service(setup, processes).with_license(Box::new(FixedLicense(true)))
}

fn request(reserves: &[u32]) -> UnlockRequest {
    UnlockRequest {
        reserves: reserves.to_vec(),
        ..UnlockRequest::default()
    }
}

fn service(setup: &Setup, processes: Box<dyn ProcessList + Send + Sync>) -> AppService {
    let service = AppService::new(
        &setup.appdata,
        SearchRoots {
            documents: vec![setup.docs.clone()],
            steam: Vec::new(),
        },
        processes,
    );
    service.select_game_dir(&setup.game).unwrap();
    service
}

fn zone_count(save: &Path) -> usize {
    let raw = fs::read(save.join(ZONES_FILE)).unwrap();
    let envelope = SaveEnvelope::parse(&raw).unwrap();
    NeedZones::parse(envelope.adf())
        .unwrap()
        .reserves
        .iter()
        .map(|r| r.zones.len())
        .sum()
}

#[test]
fn unlocking_the_map_is_previewed_written_safely_and_undoable() {
    let Some(setup) = setup() else { return };
    let service = premium_service(&setup, Box::new(Closed));
    let before_zones = zone_count(&setup.save);
    let original_file = fs::read(setup.save.join(ZONES_FILE)).unwrap();

    let preview = service.zones_preview().unwrap();
    assert!(preview.total > 10_000, "total {}", preview.total);
    let ready: Vec<u32> = preview
        .reserves
        .iter()
        .filter(|r| r.state == ReserveState::Ready)
        .map(|r| r.id)
        .collect();
    assert!(ready.len() > 5);
    // Só olhar não muda nada no save.
    assert_eq!(zone_count(&setup.save), before_zones);

    // Liberar só uma reserva acrescenta só as zonas dela.
    let one = preview.reserves.iter().find(|r| r.id == ready[0]).unwrap();
    let result = service.unlock_zones(&request(&[ready[0]])).unwrap();
    assert_eq!(result.reserves, 1);
    assert_eq!(result.zones_added, one.new_zones);
    assert_eq!(zone_count(&setup.save), before_zones + one.new_zones);

    // Depois de liberar, essa reserva não tem mais nada a liberar.
    let after = service.zones_preview().unwrap();
    let again = after.reserves.iter().find(|r| r.id == ready[0]).unwrap();
    assert_eq!(again.state, ReserveState::NothingToDo);
    assert_eq!(
        service
            .unlock_zones(&request(&[ready[0]]))
            .unwrap_err()
            .code(),
        "nothingToUnlock"
    );

    // O histórico guarda a alteração e o backup restaura o save inteiro.
    let history = service.history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].function, "Liberar o mapa");
    service.restore(Path::new(&history[0].backup)).unwrap();
    assert_eq!(
        fs::read(setup.save.join(ZONES_FILE)).unwrap(),
        original_file
    );
}

#[test]
fn nothing_is_written_while_the_game_is_open() {
    let Some(setup) = setup() else { return };
    let service = premium_service(&setup, Box::new(Open));
    let before = fs::read(setup.save.join(ZONES_FILE)).unwrap();
    let ids: Vec<u32> = service
        .zones_preview()
        .unwrap()
        .reserves
        .iter()
        .filter(|r| r.state == ReserveState::Ready)
        .map(|r| r.id)
        .collect();
    let err = service.unlock_zones(&request(&ids)).unwrap_err();
    assert_eq!(err.code(), "gameRunning");
    assert_eq!(fs::read(setup.save.join(ZONES_FILE)).unwrap(), before);
    assert!(!setup.appdata.join("Great COTW One/backups").exists());
}

#[test]
fn a_free_user_can_preview_but_not_write() {
    let Some(setup) = setup() else { return };
    let service = service(&setup, Box::new(Closed));
    let before = fs::read(setup.save.join(ZONES_FILE)).unwrap();
    let preview = service.zones_preview().unwrap();
    assert!(preview.total > 10_000);
    let ids: Vec<u32> = preview
        .reserves
        .iter()
        .filter(|r| r.state == ReserveState::Ready)
        .map(|r| r.id)
        .collect();
    let err = service.unlock_zones(&request(&ids)).unwrap_err();
    assert_eq!(err.code(), "premiumRequired");
    assert_eq!(fs::read(setup.save.join(ZONES_FILE)).unwrap(), before);
    assert!(!setup.appdata.join("Great COTW One/backups").exists());
}

#[test]
fn unlocking_by_need_or_by_animal_adds_only_what_was_chosen() {
    let Some(setup) = setup() else { return };
    let service = premium_service(&setup, Box::new(Closed));
    let preview = service.zones_preview().unwrap();
    let reserve = preview
        .reserves
        .iter()
        .find(|r| r.state == ReserveState::Ready && r.species.iter().any(|s| !s.id.is_empty()))
        .unwrap();
    let before = zone_count(&setup.save);

    // Só beber: soma da coluna "beber" de todas as espécies.
    let drink: usize = reserve.species.iter().map(|s| s.needs[1]).sum();
    assert!(drink > 0 && drink < reserve.new_zones);
    let only_drink = UnlockRequest {
        reserves: vec![reserve.id],
        needs: Some(vec![2]),
        ..UnlockRequest::default()
    };
    let result = service.unlock_zones(&only_drink).unwrap();
    assert_eq!(result.zones_added, drink);
    assert_eq!(zone_count(&setup.save), before + drink);

    // Só uma espécie, nas necessidades que sobraram (comer e descansar).
    let after = service.zones_preview().unwrap();
    let left = after.reserves.iter().find(|r| r.id == reserve.id).unwrap();
    assert_eq!(left.new_zones, reserve.new_zones - drink);
    let species = left.species.iter().find(|s| !s.id.is_empty()).unwrap();
    let wanted = species.needs.iter().sum::<usize>();
    assert!(wanted > 0);
    let one_species = UnlockRequest {
        reserves: vec![reserve.id],
        species: Some(vec![species.id.clone()]),
        ..UnlockRequest::default()
    };
    let result = service.unlock_zones(&one_species).unwrap();
    assert_eq!(result.zones_added, wanted);

    // Uma espécie que não existe não libera nada.
    let nothing = UnlockRequest {
        reserves: vec![reserve.id],
        species: Some(vec!["dragon".into()]),
        ..UnlockRequest::default()
    };
    assert_eq!(
        service.unlock_zones(&nothing).unwrap_err().code(),
        "nothingToUnlock"
    );
}

/// Tamanho do conteúdo descomprimido (o arquivo no disco é recomprimido e muda de tamanho).
fn population_len(save: &Path, reserve: u32) -> usize {
    let raw = fs::read(save.join(format!("animal_population_{reserve}"))).unwrap();
    SaveEnvelope::parse(&raw).unwrap().adf().len()
}

#[test]
fn editing_animals_needs_premium_and_changes_only_what_was_asked() {
    use cotw_core::animal_editor::AnimalChange;
    use cotw_core::service::EditRequest;
    let Some(setup) = setup() else { return };
    let free = service(&setup, Box::new(Closed));
    let editor = free.species_editor(2, "moose").unwrap();
    assert_eq!(editor.name, "Alce");
    assert_ne!(editor.groups.len(), 0);
    let group = &editor.groups[0];
    let animal = group.animals[0];
    let change = AnimalChange {
        group: group.number,
        index: animal.index,
        weight: Some(animal.weight + 25.0),
        ..AnimalChange::default()
    };
    let request = EditRequest {
        reserve: 2,
        species: "moose".into(),
        changes: vec![change],
    };

    // Sem Premium: vê o editor, mas nada grava.
    let before = fs::read(setup.save.join("animal_population_2")).unwrap();
    assert_eq!(
        free.edit_animals(&request).unwrap_err().code(),
        "premiumRequired"
    );
    assert_eq!(
        fs::read(setup.save.join("animal_population_2")).unwrap(),
        before
    );

    // Com Premium: grava só esse animal e mantém o tamanho.
    let premium = premium_service(&setup, Box::new(Closed));
    let size = population_len(&setup.save, 2);
    let result = premium.edit_animals(&request).unwrap();
    assert_eq!(result.animals, 1);
    assert_eq!(population_len(&setup.save, 2), size);
    let after = premium.species_editor(2, "moose").unwrap();
    let edited = after.groups[0].animals[0];
    assert!((edited.weight - (animal.weight + 25.0)).abs() < 1e-3);
    assert_eq!(after.groups[0].animals.len(), group.animals.len());
    for (g_before, g_after) in editor.groups.iter().zip(&after.groups) {
        for (b, a) in g_before.animals.iter().zip(&g_after.animals) {
            if (g_before.number, b.index) != (group.number, animal.index) {
                assert_eq!(b, a);
            }
        }
    }

    // O histórico guarda a edição e o backup devolve o arquivo original.
    let history = premium.history().unwrap();
    assert_eq!(history[0].function, "Editar animais");
    premium.restore(Path::new(&history[0].backup)).unwrap();
    assert_eq!(
        fs::read(setup.save.join("animal_population_2")).unwrap(),
        before
    );
}

#[test]
fn bad_animal_edits_are_refused_without_touching_the_save() {
    use cotw_core::animal_editor::AnimalChange;
    use cotw_core::service::EditRequest;
    let Some(setup) = setup() else { return };
    let premium = premium_service(&setup, Box::new(Closed));
    let before = fs::read(setup.save.join("animal_population_2")).unwrap();
    let editor = premium.species_editor(2, "moose").unwrap();
    let group = editor.groups[0].number;
    let run = |change: AnimalChange, species: &str| {
        premium
            .edit_animals(&EditRequest {
                reserve: 2,
                species: species.into(),
                changes: vec![change],
            })
            .unwrap_err()
            .code()
    };
    let base = AnimalChange {
        group,
        index: 0,
        ..AnimalChange::default()
    };
    assert_eq!(
        run(
            AnimalChange {
                weight: Some(-1.0),
                ..base
            },
            "moose"
        ),
        "editorInvalidValue"
    );
    assert_eq!(
        run(
            AnimalChange {
                weight: Some(f32::NAN),
                ..base
            },
            "moose"
        ),
        "editorInvalidValue"
    );
    assert_eq!(
        run(
            AnimalChange {
                index: 9_999,
                weight: Some(1.0),
                ..base
            },
            "moose"
        ),
        "editorWrongAnimal"
    );
    assert_eq!(
        run(
            AnimalChange {
                group: 99_999,
                weight: Some(1.0),
                ..base
            },
            "moose"
        ),
        "editorWrongAnimal"
    );
    assert_eq!(
        run(
            AnimalChange {
                weight: Some(1_000_000.0),
                ..base
            },
            "moose"
        ),
        "editorOutOfRange"
    );
    assert_eq!(
        run(
            AnimalChange {
                score: Some(1_000_000.0),
                ..base
            },
            "moose"
        ),
        "editorOutOfRange"
    );
    assert_eq!(run(base, "moose"), "nothingToChange");
    assert_eq!(run(base, "dragon"), "editorNoSpecies");
    assert_eq!(
        fs::read(setup.save.join("animal_population_2")).unwrap(),
        before
    );
}

#[test]
fn the_animal_editor_does_not_write_while_the_game_is_open() {
    use cotw_core::animal_editor::AnimalChange;
    use cotw_core::service::EditRequest;
    let Some(setup) = setup() else { return };
    let premium = premium_service(&setup, Box::new(Open));
    let editor = premium.species_editor(2, "moose").unwrap();
    let group = &editor.groups[0];
    let before = fs::read(setup.save.join("animal_population_2")).unwrap();
    let err = premium
        .edit_animals(&EditRequest {
            reserve: 2,
            species: "moose".into(),
            changes: vec![AnimalChange {
                group: group.number,
                index: 0,
                weight: Some(group.animals[0].weight + 1.0),
                ..AnimalChange::default()
            }],
        })
        .unwrap_err();
    assert_eq!(err.code(), "gameRunning");
    assert_eq!(
        fs::read(setup.save.join("animal_population_2")).unwrap(),
        before
    );
}

#[test]
fn an_animal_can_change_sex_with_the_typical_values_of_the_new_sex() {
    use cotw_core::animal_editor::AnimalChange;
    use cotw_core::service::EditRequest;
    let Some(setup) = setup() else { return };
    let premium = premium_service(&setup, Box::new(Closed));
    let before = fs::read(setup.save.join("animal_population_2")).unwrap();
    let editor = premium.species_editor(2, "moose").unwrap();
    let (group, animal) = editor
        .groups
        .iter()
        .find_map(|g| {
            g.animals
                .iter()
                .find(|a| a.male && !a.great_one)
                .map(|a| (g.number, *a))
        })
        .unwrap();
    // Sem ajustar o peso e a pontuação ao sexo novo, o app recusa se não couberem.
    let typical = editor.limits.female;
    let swap = AnimalChange {
        group,
        index: animal.index,
        male: Some(false),
        weight: Some(typical.typical_weight),
        score: Some(typical.typical_score),
        ..AnimalChange::default()
    };
    let request = EditRequest {
        reserve: 2,
        species: "moose".into(),
        changes: vec![swap],
    };
    assert_eq!(premium.edit_animals(&request).unwrap().animals, 1);
    let after = premium.species_editor(2, "moose").unwrap();
    let changed = after
        .groups
        .iter()
        .find(|g| g.number == group)
        .unwrap()
        .animals
        .iter()
        .find(|a| a.index == animal.index)
        .unwrap();
    assert!(!changed.male);
    assert!((changed.weight - typical.typical_weight).abs() < 1e-3);
    assert_eq!(
        after.groups.iter().map(|g| g.animals.len()).sum::<usize>(),
        editor.groups.iter().map(|g| g.animals.len()).sum::<usize>()
    );
    let history = premium.history().unwrap();
    premium.restore(Path::new(&history[0].backup)).unwrap();
    assert_eq!(
        fs::read(setup.save.join("animal_population_2")).unwrap(),
        before
    );
}

#[test]
fn the_service_answers_in_the_chosen_language() {
    let Some(setup) = setup() else { return };
    let service = service(&setup, Box::new(Closed));
    assert_eq!(service.species_editor(2, "moose").unwrap().name, "Alce");
    service.set_language("en-US");
    assert_eq!(service.species_editor(2, "moose").unwrap().name, "Moose");
    service.set_language("es");
    assert_eq!(
        service.species_editor(2, "brown_bear").unwrap().name,
        "Oso pardo euroasiático"
    );
    // Idioma desconhecido cai no inglês.
    service.set_language("xx");
    assert_eq!(service.species_editor(2, "moose").unwrap().name, "Moose");
}

#[test]
fn editing_equipment_needs_premium_writes_safely_and_can_be_undone() {
    use cotw_core::equipment_editor::EquipmentChange;
    let Some(setup) = setup() else { return };
    let free = service(&setup, Box::new(Closed));
    let before = fs::read(setup.save.join("thp_player_profile_adf")).unwrap();
    let cash = vec![EquipmentChange::SetCash { cash: 777_000 }];

    // Sem Premium: nada grava.
    assert_eq!(
        free.edit_equipment(&cash).unwrap_err().code(),
        "premiumRequired"
    );
    assert_eq!(
        fs::read(setup.save.join("thp_player_profile_adf")).unwrap(),
        before
    );

    // Com o jogo aberto: recusa e o arquivo continua igual.
    let running = premium_service(&setup, Box::new(Open));
    assert_eq!(
        running.edit_equipment(&cash).unwrap_err().code(),
        "gameRunning"
    );
    assert_eq!(
        fs::read(setup.save.join("thp_player_profile_adf")).unwrap(),
        before
    );

    // Com Premium e o jogo fechado: grava, aparece na leitura e o backup desfaz.
    let premium = premium_service(&setup, Box::new(Closed));
    let result = premium.edit_equipment(&cash).unwrap();
    assert_eq!(result.changes, 1);
    assert_eq!(premium.equipment().unwrap().cash, 777_000);
    // Repetir o mesmo pedido não muda nada.
    assert_eq!(
        premium.edit_equipment(&cash).unwrap_err().code(),
        "nothingToChange"
    );
    let history = premium.history().unwrap();
    assert_eq!(history[0].function, "Editar equipamentos");
    premium.restore(Path::new(&history[0].backup)).unwrap();
    assert_eq!(
        fs::read(setup.save.join("thp_player_profile_adf")).unwrap(),
        before
    );
}
