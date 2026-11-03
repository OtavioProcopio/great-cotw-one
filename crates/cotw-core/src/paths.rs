//! Pastas de dados do próprio app, em `%APPDATA%\Great COTW One`.

use std::path::{Path, PathBuf};

const APP_DIR: &str = "Great COTW One";
/// Nome de trabalho antigo; a pasta com esse nome é movida para a nova na primeira execução.
const LEGACY_APP_DIR: &str = "COTW Companion";

/// Pasta de dados do app dentro de `appdata` (normalmente `%APPDATA%`).
#[must_use]
pub fn app_data_dir(appdata: &Path) -> PathBuf {
    appdata.join(APP_DIR)
}

/// Pasta onde ficam os backups com data.
#[must_use]
pub fn backups_dir(appdata: &Path) -> PathBuf {
    app_data_dir(appdata).join("backups")
}

/// Arquivo do banco do histórico.
#[must_use]
pub fn history_db(appdata: &Path) -> PathBuf {
    app_data_dir(appdata).join("history.db")
}

/// Move a pasta de dados do nome antigo para a nova, se só a antiga existir.
///
/// Devolve `(antiga, nova)` quando moveu, para o chamador ajustar os caminhos guardados.
///
/// # Errors
///
/// Retorna o erro de E/S se a pasta não puder ser renomeada; nesse caso nada muda.
pub fn migrate_legacy_dir(appdata: &Path) -> std::io::Result<Option<(PathBuf, PathBuf)>> {
    let legacy = appdata.join(LEGACY_APP_DIR);
    let current = app_data_dir(appdata);
    if !legacy.is_dir() || current.exists() {
        return Ok(None);
    }
    std::fs::rename(&legacy, &current)?;
    Ok(Some((legacy, current)))
}

/// `%APPDATA%` do usuário atual, se definido.
#[must_use]
pub fn system_appdata() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_files_live_under_the_app_folder() {
        let appdata = Path::new("C:/Users/ana/AppData/Roaming");
        assert_eq!(
            backups_dir(appdata),
            appdata.join("Great COTW One").join("backups")
        );
        assert_eq!(
            history_db(appdata),
            appdata.join("Great COTW One").join("history.db")
        );
    }

    #[test]
    fn moves_the_legacy_folder_once() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("COTW Companion");
        std::fs::create_dir_all(legacy.join("backups")).unwrap();
        std::fs::write(legacy.join("history.db"), b"x").unwrap();

        let moved = migrate_legacy_dir(dir.path()).unwrap();
        assert_eq!(moved, Some((legacy.clone(), app_data_dir(dir.path()))));
        assert!(!legacy.exists());
        assert!(history_db(dir.path()).is_file());
        assert_eq!(migrate_legacy_dir(dir.path()).unwrap(), None);
    }

    #[test]
    fn keeps_both_folders_when_the_new_one_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("COTW Companion")).unwrap();
        std::fs::create_dir_all(app_data_dir(dir.path())).unwrap();
        assert_eq!(migrate_legacy_dir(dir.path()).unwrap(), None);
        assert!(dir.path().join("COTW Companion").is_dir());
    }

    #[test]
    fn does_nothing_without_a_legacy_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(migrate_legacy_dir(dir.path()).unwrap(), None);
        assert!(!app_data_dir(dir.path()).exists());
    }
}
