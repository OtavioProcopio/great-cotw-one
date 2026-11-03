//! Escolhas do usuário que precisam sobreviver ao fechamento do app (pastas escolhidas à mão).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Configurações salvas em `settings.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Pasta de save escolhida (conta Steam ou pasta indicada manualmente).
    #[serde(default)]
    pub save_dir: Option<PathBuf>,
    /// Pasta do jogo (ou `archives_win64`) indicada manualmente.
    #[serde(default)]
    pub game_dir: Option<PathBuf>,
}

impl Settings {
    /// Lê as configurações; arquivo ausente ou ilegível vira o padrão (nunca impede o app de abrir).
    #[must_use]
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Grava as configurações, criando a pasta se preciso.
    ///
    /// # Errors
    ///
    /// Retorna o erro de E/S se não for possível gravar.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        // Grava num temporário e renomeia: um corte de energia não deixa o arquivo pela metade.
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, text)?;
        fs::rename(&temp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dados/settings.json");
        let settings = Settings {
            save_dir: Some(PathBuf::from("C:/saves/1")),
            game_dir: None,
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
        assert!(!dir.path().join("dados/settings.json.tmp").exists());
    }

    #[test]
    fn missing_or_corrupt_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Settings::load(&dir.path().join("nada.json")),
            Settings::default()
        );
        let corrupt = dir.path().join("ruim.json");
        fs::write(&corrupt, b"{ isto nao e json").unwrap();
        assert_eq!(Settings::load(&corrupt), Settings::default());
    }

    #[test]
    fn unknown_and_missing_fields_are_tolerated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        fs::write(&path, br#"{"save_dir":"X","campo_novo":1}"#).unwrap();
        let loaded = Settings::load(&path);
        assert_eq!(loaded.save_dir, Some(PathBuf::from("X")));
        assert_eq!(loaded.game_dir, None);
    }

    #[test]
    fn unwritable_location_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("arquivo");
        fs::write(&blocker, b"x").unwrap();
        assert!(
            Settings::default()
                .save(&blocker.join("settings.json"))
                .is_err()
        );
    }
}
