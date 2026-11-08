//! Localização da pasta `Avalanche Studios\COTW\Saves\<steamid>`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::files::looks_like_save_dir;

const SAVES_SUBPATH: [&str; 3] = ["Avalanche Studios", "COTW", "Saves"];
/// Nomes que o Windows dá à pasta Documentos, conforme o idioma.
const DOCUMENTS_NAMES: [&str; 2] = ["Documents", "Documentos"];

/// Uma conta Steam com saves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveAccount {
    /// Steam ID (nome da pasta).
    pub steam_id: String,
    /// Pasta com os arquivos de save.
    pub path: PathBuf,
    /// Data da última modificação de qualquer arquivo da pasta.
    pub last_modified: Option<SystemTime>,
}

/// Resultado da busca.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveLocation {
    /// Exatamente uma conta: pode ser selecionada sem perguntar.
    Found(SaveAccount),
    /// Duas ou mais contas, da mais recente para a mais antiga: o usuário escolhe.
    Choose(Vec<SaveAccount>),
    /// Nada encontrado; `searched` lista onde procuramos para explicar ao usuário.
    NotFound {
        /// Pastas `Saves` consultadas.
        searched: Vec<PathBuf>,
    },
}

/// Pastas Documentos prováveis, inclusive redirecionadas para o `OneDrive`.
///
/// `user_profile` é `%USERPROFILE%`; `onedrive_roots` são as variáveis `OneDrive*`.
#[must_use]
pub fn documents_candidates(
    user_profile: Option<&Path>,
    onedrive_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = user_profile.map(Path::to_path_buf).into_iter().collect();
    bases.extend(user_profile.map(|p| p.join("OneDrive")));
    bases.extend(onedrive_roots.iter().cloned());

    let mut out: Vec<PathBuf> = Vec::new();
    for base in bases {
        for name in DOCUMENTS_NAMES {
            let candidate = base.join(name);
            if !out.contains(&candidate) {
                out.push(candidate);
            }
        }
    }
    out
}

/// Mesmo que [`documents_candidates`], lendo `USERPROFILE` e `OneDrive*` do ambiente.
///
/// A pasta Documentos que o Windows informa vem primeiro: ela vale mesmo quando o usuário a moveu
/// para outro disco ou usa o Windows em outro idioma (Dokumente, Documenti…).
#[must_use]
pub fn documents_candidates_from_env() -> Vec<PathBuf> {
    let profile = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let onedrive: Vec<PathBuf> = ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"]
        .iter()
        .filter_map(|var| std::env::var_os(var).map(PathBuf::from))
        .collect();
    let mut out: Vec<PathBuf> = dirs::document_dir().into_iter().collect();
    for candidate in documents_candidates(profile.as_deref(), &onedrive) {
        if !out.contains(&candidate) {
            out.push(candidate);
        }
    }
    out
}

fn saves_dir_of(documents: &Path) -> PathBuf {
    SAVES_SUBPATH
        .iter()
        .fold(documents.to_path_buf(), |p, s| p.join(s))
}

/// Contas encontradas nas pastas Documentos dadas, da mais recente para a mais antiga.
#[must_use]
pub fn find_accounts(documents: &[PathBuf]) -> Vec<SaveAccount> {
    let mut accounts: Vec<SaveAccount> = Vec::new();
    for docs in documents {
        let Ok(entries) = fs::read_dir(saves_dir_of(docs)) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_account = !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit());
            if !is_account || !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let account = account_at(&name, entry.path());
            // A mesma pasta pode aparecer por dois caminhos (ex.: atalho do OneDrive).
            let duplicate = accounts.iter().any(|a| same_dir(&a.path, &account.path));
            if !duplicate {
                accounts.push(account);
            }
        }
    }
    accounts.sort_by(|a, b| {
        b.last_modified
            .cmp(&a.last_modified)
            .then_with(|| a.steam_id.cmp(&b.steam_id))
    });
    accounts
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn account_at(steam_id: &str, path: PathBuf) -> SaveAccount {
    let newest = fs::read_dir(&path)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max();
    let last_modified = newest.or_else(|| fs::metadata(&path).ok()?.modified().ok());
    SaveAccount {
        steam_id: steam_id.to_owned(),
        path,
        last_modified,
    }
}

fn outcome(mut accounts: Vec<SaveAccount>, searched: Vec<PathBuf>) -> SaveLocation {
    match accounts.len() {
        0 => SaveLocation::NotFound { searched },
        1 => SaveLocation::Found(accounts.remove(0)),
        _ => SaveLocation::Choose(accounts),
    }
}

/// Procura os saves nas pastas Documentos dadas.
#[must_use]
pub fn locate(documents: &[PathBuf]) -> SaveLocation {
    let searched = documents.iter().map(|d| saves_dir_of(d)).collect();
    outcome(find_accounts(documents), searched)
}

/// Interpreta uma pasta escolhida manualmente pelo usuário.
///
/// Aceita a própria pasta da conta (com arquivos de save) ou a pasta `Saves` que contém as
/// contas. Qualquer outra coisa é `NotFound`.
#[must_use]
pub fn resolve_manual(chosen: &Path) -> SaveLocation {
    if looks_like_save_dir(chosen) {
        let steam_id = chosen
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return SaveLocation::Found(account_at(&steam_id, chosen.to_path_buf()));
    }
    let searched = vec![chosen.to_path_buf()];
    let Ok(entries) = fs::read_dir(chosen) else {
        return SaveLocation::NotFound { searched };
    };
    let mut accounts: Vec<SaveAccount> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()) && looks_like_save_dir(&e.path()))
        .map(|e| account_at(&e.file_name().to_string_lossy(), e.path()))
        .collect();
    accounts.sort_by_key(|a| std::cmp::Reverse(a.last_modified));
    outcome(accounts, searched)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::time::Duration;

    use super::*;

    /// Cria `<docs>/Avalanche Studios/COTW/Saves/<id>/settings_adf` e devolve a pasta da conta.
    fn account(docs: &Path, id: &str) -> PathBuf {
        let dir = saves_dir_of(docs).join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings_adf"), b"x").unwrap();
        dir
    }

    fn set_mtime(path: &Path, secs_after_epoch: u64) {
        let file = File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs_after_epoch))
            .unwrap();
    }

    #[test]
    fn candidates_cover_documents_and_onedrive_in_both_languages() {
        let profile = PathBuf::from("C:/Users/ana");
        let onedrive = vec![PathBuf::from("D:/OneDrive")];
        let found = documents_candidates(Some(&profile), &onedrive);
        for expected in [
            "C:/Users/ana/Documents",
            "C:/Users/ana/Documentos",
            "C:/Users/ana/OneDrive/Documents",
            "C:/Users/ana/OneDrive/Documentos",
            "D:/OneDrive/Documents",
            "D:/OneDrive/Documentos",
        ] {
            assert!(found.contains(&PathBuf::from(expected)), "{expected}");
        }
    }

    #[test]
    fn candidates_have_no_duplicates_and_work_without_profile() {
        let onedrive = vec![PathBuf::from("X:/OD"), PathBuf::from("X:/OD")];
        let found = documents_candidates(None, &onedrive);
        assert_eq!(found.len(), 2);
        assert_eq!(documents_candidates(None, &[]), Vec::<PathBuf>::new());
    }

    #[test]
    fn single_account_is_selected_automatically() {
        let docs = tempfile::tempdir().unwrap();
        let dir = account(docs.path(), "76561198000000001");
        let SaveLocation::Found(found) = locate(&[docs.path().to_path_buf()]) else {
            panic!("esperava Found");
        };
        assert_eq!(found.steam_id, "76561198000000001");
        assert_eq!(found.path, dir);
        assert!(found.last_modified.is_some());
    }

    #[test]
    fn several_accounts_are_listed_most_recent_first() {
        let docs = tempfile::tempdir().unwrap();
        let old = account(docs.path(), "111");
        let new = account(docs.path(), "222");
        set_mtime(&old.join("settings_adf"), 1_000);
        set_mtime(&new.join("settings_adf"), 9_000);
        let SaveLocation::Choose(list) = locate(&[docs.path().to_path_buf()]) else {
            panic!("esperava Choose");
        };
        let ids: Vec<_> = list.iter().map(|a| a.steam_id.as_str()).collect();
        assert_eq!(ids, ["222", "111"]);
    }

    #[test]
    fn same_account_under_two_documents_paths_counts_once() {
        let docs = tempfile::tempdir().unwrap();
        account(docs.path(), "111");
        let again = docs.path().to_path_buf();
        assert!(matches!(
            locate(&[docs.path().to_path_buf(), again]),
            SaveLocation::Found(_)
        ));
    }

    #[test]
    fn non_numeric_folders_and_files_are_not_accounts() {
        let docs = tempfile::tempdir().unwrap();
        account(docs.path(), "111");
        let saves = saves_dir_of(docs.path());
        fs::create_dir(saves.join("backup")).unwrap();
        fs::write(saves.join("222"), b"arquivo, nao pasta").unwrap();
        let SaveLocation::Found(found) = locate(&[docs.path().to_path_buf()]) else {
            panic!("esperava Found");
        };
        assert_eq!(found.steam_id, "111");
    }

    #[test]
    fn not_found_lists_where_it_looked() {
        let docs = tempfile::tempdir().unwrap();
        let SaveLocation::NotFound { searched } = locate(&[docs.path().to_path_buf()]) else {
            panic!("esperava NotFound");
        };
        assert_eq!(searched, [saves_dir_of(docs.path())]);
    }

    #[test]
    fn manual_choice_accepts_an_account_folder() {
        let docs = tempfile::tempdir().unwrap();
        let dir = account(docs.path(), "333");
        let SaveLocation::Found(found) = resolve_manual(&dir) else {
            panic!("esperava Found");
        };
        assert_eq!((found.steam_id.as_str(), found.path), ("333", dir));
    }

    #[test]
    fn manual_choice_accepts_the_saves_folder_with_accounts() {
        let docs = tempfile::tempdir().unwrap();
        account(docs.path(), "1");
        account(docs.path(), "2");
        let saves = saves_dir_of(docs.path());
        assert!(matches!(resolve_manual(&saves), SaveLocation::Choose(v) if v.len() == 2));
    }

    #[test]
    fn manual_choice_of_unrelated_or_missing_folder_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            resolve_manual(dir.path()),
            SaveLocation::NotFound { .. }
        ));
        assert!(matches!(
            resolve_manual(&dir.path().join("nada")),
            SaveLocation::NotFound { .. }
        ));
    }
}
