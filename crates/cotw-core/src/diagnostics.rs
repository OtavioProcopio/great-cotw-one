//! Diagnóstico: o que o app encontrou do jogo e do save, e o que reconhece (tarefa 3.3).
//!
//! O resultado é só dados (códigos de estado, caminhos, números); os textos para o usuário
//! ficam na interface, para permitir outros idiomas.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use cotw_formats::adf::Adf;
use cotw_formats::envelope::SaveEnvelope;
use serde::Serialize;

use crate::discovery::files::{SaveFileKind, classify};
use crate::discovery::saves::{self, SaveAccount, SaveLocation};
use crate::discovery::steam::{self, GameInstall};
use crate::discovery::{Reserve, list_reserves};
use crate::settings::Settings;

/// Situação da instalação do jogo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDiagnostic {
    /// O jogo foi encontrado.
    pub found: bool,
    /// Pasta do jogo.
    pub root: Option<String>,
    /// Pasta `archives_win64`.
    pub archives: Option<String>,
    /// Quantidade de arquivos `gameN.tab`.
    pub tab_files: usize,
    /// Número da versão (build) instalada, lido do manifesto da Steam.
    pub build_id: Option<String>,
    /// A pasta veio de uma escolha manual do usuário.
    pub manual: bool,
}

/// Uma conta de save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDiagnostic {
    /// Steam ID.
    pub steam_id: String,
    /// Pasta dos arquivos.
    pub path: String,
    /// Última modificação, em segundos desde 1970.
    pub last_modified: Option<u64>,
}

/// Estado da busca pelo save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SaveStatus {
    /// Uma conta selecionada.
    Found,
    /// Várias contas: o usuário precisa escolher.
    Choose,
    /// Nada encontrado.
    NotFound,
}

/// Situação do save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDiagnostic {
    /// Resultado da busca.
    pub status: SaveStatus,
    /// Conta selecionada (quando `Found`).
    pub account: Option<AccountDiagnostic>,
    /// Contas para escolher (quando `Choose`).
    pub accounts: Vec<AccountDiagnostic>,
    /// Pastas onde se procurou (quando `NotFound`).
    pub searched: Vec<String>,
    /// A pasta veio de uma escolha manual do usuário.
    pub manual: bool,
}

/// Como o app trata um arquivo da pasta de save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileState {
    /// Reconhecido e lido sem erro.
    Ok,
    /// Nome conhecido, mas o conteúdo não pôde ser lido: tratado como não reconhecido.
    Unreadable,
    /// Nome desconhecido: só aparece na lista, nunca é lido nem editado.
    Unrecognized,
}

/// Um arquivo da pasta de save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiagnostic {
    /// Nome do arquivo.
    pub name: String,
    /// Tamanho em bytes.
    pub size: u64,
    /// Tipo de arquivo (`population`, `foundIcons`, `fogOfWar`, `known`, `unrecognized`).
    pub kind: &'static str,
    /// Número da reserva, nos arquivos por reserva.
    pub reserve: Option<u32>,
    /// Estado de reconhecimento.
    pub state: FileState,
}

/// Uma reserva presente no save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveDiagnostic {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
}

impl From<Reserve> for ReserveDiagnostic {
    fn from(r: Reserve) -> Self {
        Self {
            id: r.id,
            name: r.name,
        }
    }
}

/// Resultado completo do diagnóstico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// Jogo.
    pub game: GameDiagnostic,
    /// Save.
    pub save: SaveDiagnostic,
    /// Reservas presentes no save selecionado.
    pub reserves: Vec<ReserveDiagnostic>,
    /// Arquivos do save selecionado.
    pub files: Vec<FileDiagnostic>,
    /// Sem o jogo, as telas que dependem de nomes, mapas e regras dele ficam limitadas.
    pub limited_mode: bool,
}

/// Onde procurar; separado do sistema para ser testável.
#[derive(Debug, Clone, Default)]
pub struct SearchRoots {
    /// Pastas Documentos prováveis.
    pub documents: Vec<PathBuf>,
    /// Raízes da Steam.
    pub steam: Vec<PathBuf>,
}

impl SearchRoots {
    /// Lê as pastas do computador atual.
    #[must_use]
    pub fn from_system() -> Self {
        Self {
            documents: saves::documents_candidates_from_env(),
            steam: steam::steam_roots_from_system(),
        }
    }
}

/// Executa o diagnóstico completo, respeitando as escolhas manuais de `settings`.
#[must_use]
pub fn run(roots: &SearchRoots, settings: &Settings) -> Diagnostics {
    let (game, install) = game_diagnostic(roots, settings);
    let (save, selected) = save_diagnostic(roots, settings);
    let (reserves, files) = selected.map_or_else(
        || (Vec::new(), Vec::new()),
        |dir| {
            (
                list_reserves(&dir)
                    .unwrap_or_default()
                    .into_iter()
                    .map(ReserveDiagnostic::from)
                    .collect(),
                inspect_files(&dir),
            )
        },
    );
    Diagnostics {
        limited_mode: install.is_none(),
        game,
        save,
        reserves,
        files,
    }
}

fn game_diagnostic(
    roots: &SearchRoots,
    settings: &Settings,
) -> (GameDiagnostic, Option<GameInstall>) {
    let manual = settings
        .game_dir
        .as_deref()
        .and_then(steam::validate_game_dir);
    let is_manual = manual.is_some();
    let install = manual.or_else(|| steam::find_install(&roots.steam));
    let diagnostic = install.as_ref().map_or(
        GameDiagnostic {
            found: false,
            root: None,
            archives: None,
            tab_files: 0,
            build_id: None,
            manual: settings.game_dir.is_some(),
        },
        |install| GameDiagnostic {
            found: true,
            root: Some(install.root.display().to_string()),
            archives: Some(install.archives.display().to_string()),
            tab_files: count_tab_files(&install.archives),
            build_id: steam::build_id_for(&install.root),
            manual: is_manual,
        },
    );
    (diagnostic, install)
}

fn save_diagnostic(roots: &SearchRoots, settings: &Settings) -> (SaveDiagnostic, Option<PathBuf>) {
    let manual_choice = settings.save_dir.as_deref().map(saves::resolve_manual);
    let manual = matches!(manual_choice, Some(SaveLocation::Found(_)));
    let location = match manual_choice {
        Some(found @ SaveLocation::Found(_)) => found,
        _ => saves::locate(&roots.documents),
    };
    match location {
        SaveLocation::Found(account) => {
            let dir = account.path.clone();
            (
                SaveDiagnostic {
                    status: SaveStatus::Found,
                    account: Some(account_dto(&account)),
                    accounts: Vec::new(),
                    searched: Vec::new(),
                    manual,
                },
                Some(dir),
            )
        }
        SaveLocation::Choose(accounts) => (
            SaveDiagnostic {
                status: SaveStatus::Choose,
                account: None,
                accounts: accounts.iter().map(account_dto).collect(),
                searched: Vec::new(),
                manual: false,
            },
            None,
        ),
        SaveLocation::NotFound { searched } => (
            SaveDiagnostic {
                status: SaveStatus::NotFound,
                account: None,
                accounts: Vec::new(),
                searched: searched.iter().map(|p| p.display().to_string()).collect(),
                manual: false,
            },
            None,
        ),
    }
}

fn account_dto(account: &SaveAccount) -> AccountDiagnostic {
    AccountDiagnostic {
        steam_id: account.steam_id.clone(),
        path: account.path.display().to_string(),
        last_modified: account
            .last_modified
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
    }
}

fn count_tab_files(archives: &Path) -> usize {
    fs::read_dir(archives).map_or(0, |entries| {
        entries
            .filter_map(Result::ok)
            .filter(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_prefix("game")
                    .is_some_and(|rest| rest.strip_suffix(".tab").is_some())
            })
            .count()
    })
}

/// Classifica e lê cada arquivo da pasta de save, sem nunca falhar por causa de um deles.
#[must_use]
pub fn inspect_files(dir: &Path) -> Vec<FileDiagnostic> {
    let mut files: Vec<FileDiagnostic> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let size = entry.metadata().map_or(0, |m| m.len());
            let kind = classify(&name);
            let (kind_name, reserve) = match kind {
                SaveFileKind::Population(id) => ("population", Some(id)),
                SaveFileKind::FoundIcons(id) => ("foundIcons", Some(id)),
                SaveFileKind::FogOfWar(id) => ("fogOfWar", Some(id)),
                SaveFileKind::Known => ("known", None),
                SaveFileKind::Unrecognized => ("unrecognized", None),
            };
            let state = if kind == SaveFileKind::Unrecognized {
                FileState::Unrecognized
            } else if readable(&entry.path()) {
                FileState::Ok
            } else {
                FileState::Unreadable
            };
            FileDiagnostic {
                name,
                size,
                kind: kind_name,
                reserve,
                state,
            }
        })
        .collect();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

/// O arquivo tem envelope válido e ADF legível.
fn readable(path: &Path) -> bool {
    fs::read(path)
        .ok()
        .and_then(|raw| SaveEnvelope::parse(&raw).ok())
        .is_some_and(|envelope| Adf::parse(envelope.adf().to_vec()).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::valid_save_bytes;

    fn save_account(docs: &Path, id: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = docs.join("Avalanche Studios/COTW/Saves").join(id);
        fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in files {
            fs::write(dir.join(name), bytes).unwrap();
        }
        dir
    }

    fn game_library() -> tempfile::TempDir {
        let lib = tempfile::tempdir().unwrap();
        let archives = lib
            .path()
            .join("steamapps/common/theHunterCotW/archives_win64");
        fs::create_dir_all(&archives).unwrap();
        fs::write(archives.join("game0.tab"), b"TAB").unwrap();
        fs::write(archives.join("game1.tab"), b"TAB").unwrap();
        fs::write(archives.join("game1.arc"), b"x").unwrap();
        fs::write(
            lib.path().join("steamapps/appmanifest_518790.acf"),
            "\"AppState\"\n{\n\t\"installdir\"\t\"theHunterCotW\"\n\t\"buildid\"\t\"12345678\"\n}\n",
        )
        .unwrap();
        lib
    }

    fn roots(docs: &tempfile::TempDir, steam: &tempfile::TempDir) -> SearchRoots {
        SearchRoots {
            documents: vec![docs.path().to_path_buf()],
            steam: vec![steam.path().to_path_buf()],
        }
    }

    #[test]
    fn full_report_for_a_single_account_with_the_game_installed() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        save_account(
            docs.path(),
            "111",
            &[
                ("animal_population_2", &valid_save_bytes()),
                ("animal_population_77", b"corrompido"),
                ("settings_adf", &valid_save_bytes()),
                ("steam_autocloud.vdf", b"x"),
            ],
        );
        let report = run(&roots(&docs, &steam), &Settings::default());

        assert!(report.game.found && !report.limited_mode);
        assert_eq!(report.game.tab_files, 2);
        assert_eq!(report.game.build_id.as_deref(), Some("12345678"));
        assert_eq!(report.save.status, SaveStatus::Found);
        assert_eq!(report.save.account.as_ref().unwrap().steam_id, "111");

        let reserves: Vec<_> = report
            .reserves
            .iter()
            .map(|r| (r.id, r.name.as_str()))
            .collect();
        assert_eq!(reserves, [(2, "Medved-Taiga"), (77, "Reserva 77")]);

        let state = |name: &str| report.files.iter().find(|f| f.name == name).unwrap().state;
        assert_eq!(state("animal_population_2"), FileState::Ok);
        assert_eq!(state("animal_population_77"), FileState::Unreadable);
        assert_eq!(state("settings_adf"), FileState::Ok);
        assert_eq!(state("steam_autocloud.vdf"), FileState::Unrecognized);
    }

    #[test]
    fn missing_game_means_limited_mode() {
        let (docs, empty_steam) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        save_account(docs.path(), "111", &[("settings_adf", &valid_save_bytes())]);
        let report = run(&roots(&docs, &empty_steam), &Settings::default());
        assert!(!report.game.found);
        assert!(report.limited_mode);
        assert_eq!(report.save.status, SaveStatus::Found);
    }

    #[test]
    fn several_accounts_ask_the_user_to_choose_and_list_no_files() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        save_account(docs.path(), "111", &[("settings_adf", b"x")]);
        save_account(docs.path(), "222", &[("settings_adf", b"x")]);
        let report = run(&roots(&docs, &steam), &Settings::default());
        assert_eq!(report.save.status, SaveStatus::Choose);
        assert_eq!(report.save.accounts.len(), 2);
        assert_eq!(report.files, []);
        assert_eq!(report.reserves, []);
    }

    #[test]
    fn manual_save_choice_overrides_the_search() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        save_account(docs.path(), "111", &[("settings_adf", b"x")]);
        let chosen = save_account(
            docs.path(),
            "222",
            &[("animal_population_4", &valid_save_bytes())],
        );
        let settings = Settings {
            save_dir: Some(chosen),
            game_dir: None,
        };
        let report = run(&roots(&docs, &steam), &settings);
        assert_eq!(report.save.status, SaveStatus::Found);
        assert!(report.save.manual);
        assert_eq!(report.save.account.unwrap().steam_id, "222");
        assert_eq!(report.reserves[0].name, "Parque Fernando");
    }

    #[test]
    fn invalid_manual_save_choice_falls_back_to_the_search() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        save_account(docs.path(), "111", &[("settings_adf", b"x")]);
        let settings = Settings {
            save_dir: Some(docs.path().join("nao-existe")),
            game_dir: None,
        };
        let report = run(&roots(&docs, &steam), &settings);
        assert_eq!(report.save.account.unwrap().steam_id, "111");
        assert!(!report.save.manual);
    }

    #[test]
    fn manual_game_choice_is_used_when_valid() {
        let (docs, empty_steam) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let lib = game_library();
        let settings = Settings {
            save_dir: None,
            game_dir: Some(lib.path().join("steamapps/common/theHunterCotW")),
        };
        let report = run(&roots(&docs, &empty_steam), &settings);
        assert!(report.game.found && report.game.manual);
        assert!(!report.limited_mode);
    }

    #[test]
    fn save_not_found_lists_where_it_looked() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        let report = run(&roots(&docs, &steam), &Settings::default());
        assert_eq!(report.save.status, SaveStatus::NotFound);
        assert_eq!(report.save.searched.len(), 1);
    }

    #[test]
    fn report_serializes_with_camel_case_names() {
        let (docs, steam) = (tempfile::tempdir().unwrap(), game_library());
        save_account(docs.path(), "111", &[("settings_adf", &valid_save_bytes())]);
        let json = serde_json::to_value(run(&roots(&docs, &steam), &Settings::default())).unwrap();
        assert_eq!(json["limitedMode"], false);
        assert_eq!(json["save"]["status"], "found");
        assert_eq!(json["game"]["tabFiles"], 2);
        assert_eq!(json["files"][0]["state"], "ok");
    }
}
