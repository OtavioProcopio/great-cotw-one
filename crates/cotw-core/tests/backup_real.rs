//! Backup e restauração com os 74 arquivos de save reais de `fixtures/saves` (pula se ausentes).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::fs;
use std::path::PathBuf;

use cotw_core::backup::{create_backup, dirs_identical, restore_backup};

fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    dir.is_dir().then_some(dir)
}

#[test]
fn backup_and_restore_of_real_saves_are_byte_identical() {
    let Some(original) = fixtures() else {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    };
    // Trabalha numa cópia para nunca mexer nos fixtures.
    let work = tempfile::tempdir().unwrap();
    let save = work.path().join("save");
    let backups = work.path().join("backups");
    fs::create_dir_all(&save).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), save.join(entry.file_name())).unwrap();
        }
    }

    let backup = create_backup(&save, &backups, "teste").unwrap();
    assert!(backup.files >= 70);
    assert!(dirs_identical(&save, &backup.path).unwrap());

    // Estraga o save de várias formas e restaura.
    fs::write(save.join("animal_population_2"), b"corrompido").unwrap();
    fs::remove_file(save.join("settings_adf")).unwrap();
    fs::write(save.join("lixo"), b"x").unwrap();
    restore_backup(&backup.path, &save).unwrap();
    assert!(dirs_identical(&save, &backup.path).unwrap());
}
