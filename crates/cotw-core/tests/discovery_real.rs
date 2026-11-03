//! Descoberta no computador real do desenvolvedor. Cada teste pula se o que procura não existir.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::path::PathBuf;

use cotw_core::discovery::files::{SaveFileKind, classify};
use cotw_core::discovery::saves::{documents_candidates_from_env, locate};
use cotw_core::discovery::steam::{find_install, steam_roots_from_system};
use cotw_core::discovery::{SaveLocation, list_reserves};

#[test]
fn finds_the_real_save_folder_when_present() {
    match locate(&documents_candidates_from_env()) {
        SaveLocation::Found(account) => {
            assert!(account.path.is_dir());
            assert!(account.steam_id.bytes().all(|b| b.is_ascii_digit()));
        }
        SaveLocation::Choose(accounts) => assert!(accounts.iter().all(|a| a.path.is_dir())),
        SaveLocation::NotFound { searched } => {
            eprintln!("nenhum save neste computador (procurado em {searched:?}): teste ignorado");
        }
    }
}

#[test]
fn finds_the_real_game_install_when_present() {
    let Some(install) = find_install(&steam_roots_from_system()) else {
        eprintln!("jogo não encontrado neste computador: teste ignorado");
        return;
    };
    assert!(install.archives.join("game0.tab").is_file());
    assert!(install.archives.ends_with("archives_win64"));
}

#[test]
fn diagnostics_of_this_machine_are_consistent() {
    use cotw_core::diagnostics::{FileState, SaveStatus, SearchRoots, run};
    use cotw_core::settings::Settings;

    let report = run(&SearchRoots::from_system(), &Settings::default());
    eprintln!("{}", serde_json::to_string_pretty(&report.game).unwrap());
    eprintln!(
        "save: {:?}; reservas: {}; arquivos: {}; limitado: {}",
        report.save.status,
        report.reserves.len(),
        report.files.len(),
        report.limited_mode
    );
    assert_eq!(report.limited_mode, !report.game.found);
    if report.save.status == SaveStatus::Found {
        // Todo arquivo de nome conhecido da pasta real deve ser legível.
        assert!(
            report
                .files
                .iter()
                .filter(|f| f.kind != "unrecognized")
                .all(|f| f.state == FileState::Ok),
            "{:?}",
            report
                .files
                .iter()
                .filter(|f| f.state == FileState::Unreadable)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn lists_reserves_and_flags_unrecognized_files_in_the_fixture_saves() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    if !dir.is_dir() {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    }
    let reserves = list_reserves(&dir).unwrap();
    assert!(
        reserves
            .iter()
            .any(|r| r.id == 2 && r.name == "Medved-Taiga")
    );
    assert_eq!(classify("steam_autocloud.vdf"), SaveFileKind::Unrecognized);
    // Ordenadas e sem repetição.
    assert!(reserves.windows(2).all(|w| w[0].id < w[1].id));
}
