//! "Liberar o mapa" de ponta a ponta com o save e o jogo reais: prévia → gravação segura com
//! backup → conferência → restauração (pula se `fixtures/saves` ou o jogo faltarem).
//! Tudo acontece numa cópia temporária; o save e o jogo do desenvolvedor não são tocados.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use cotw_core::diagnostics::SearchRoots;
use cotw_core::process::ProcessList;
use cotw_core::service::AppService;
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
    let service = service(&setup, Box::new(Closed));
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
    let result = service.unlock_zones(&[ready[0]]).unwrap();
    assert_eq!(result.reserves, 1);
    assert_eq!(result.zones_added, one.new_zones);
    assert_eq!(zone_count(&setup.save), before_zones + one.new_zones);

    // Depois de liberar, essa reserva não tem mais nada a liberar.
    let after = service.zones_preview().unwrap();
    let again = after.reserves.iter().find(|r| r.id == ready[0]).unwrap();
    assert_eq!(again.state, ReserveState::NothingToDo);
    assert_eq!(
        service.unlock_zones(&[ready[0]]).unwrap_err().code(),
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
    let service = service(&setup, Box::new(Open));
    let before = fs::read(setup.save.join(ZONES_FILE)).unwrap();
    let ids: Vec<u32> = service
        .zones_preview()
        .unwrap()
        .reserves
        .iter()
        .filter(|r| r.state == ReserveState::Ready)
        .map(|r| r.id)
        .collect();
    let err = service.unlock_zones(&ids).unwrap_err();
    assert_eq!(err.code(), "gameRunning");
    assert_eq!(fs::read(setup.save.join(ZONES_FILE)).unwrap(), before);
    assert!(!setup.appdata.join("Great COTW One/backups").exists());
}
