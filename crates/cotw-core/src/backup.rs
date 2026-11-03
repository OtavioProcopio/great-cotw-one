//! Backup e restauração da pasta de save (spec `safe-save-writing`).
//!
//! O backup é a cópia completa da pasta, feita numa pasta `.partial` que só vira o nome final
//! depois de conferida. Nenhuma função deste módulo apaga um backup existente.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const PARTIAL_SUFFIX: &str = ".partial";

/// Erros de backup e restauração.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// Falha de E/S (sem espaço, sem permissão...).
    #[error("erro de E/S em {}: {source}", path.display())]
    Io {
        /// Caminho envolvido.
        path: PathBuf,
        /// Erro original.
        #[source]
        source: io::Error,
    },
    /// A pasta de origem não existe ou não é uma pasta.
    #[error("{} não é uma pasta", .0.display())]
    NotADirectory(PathBuf),
    /// A pasta de backups ficaria dentro da pasta copiada.
    #[error("a pasta de backups não pode ficar dentro da pasta do save")]
    BackupsInsideSave,
    /// A cópia não bate com a origem.
    #[error("a cópia não confere com a origem: {0}")]
    VerificationFailed(String),
}

fn io_error(path: &Path) -> impl FnOnce(io::Error) -> BackupError + '_ {
    move |source| BackupError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Backup criado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInfo {
    /// Pasta do backup.
    pub path: PathBuf,
    /// Quantidade de arquivos copiados.
    pub files: usize,
    /// Total de bytes copiados.
    pub bytes: u64,
}

/// Data e hora local no formato usado nos nomes de backup (`2026-11-02_15-30-00`).
#[must_use]
pub fn timestamp_now() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}

/// Copia a pasta inteira do save para `<backups_root>/<stamp>` e confere a cópia.
///
/// Se `<stamp>` já existir, usa `<stamp>-2`, `<stamp>-3`... para nunca sobrescrever.
///
/// # Errors
///
/// Retorna [`BackupError`] se não for possível copiar ou se a cópia não conferir; nesse caso a
/// pasta parcial é removida e nenhum backup anterior é tocado.
pub fn create_backup(
    save_dir: &Path,
    backups_root: &Path,
    stamp: &str,
) -> Result<BackupInfo, BackupError> {
    if !save_dir.is_dir() {
        return Err(BackupError::NotADirectory(save_dir.to_path_buf()));
    }
    fs::create_dir_all(backups_root).map_err(io_error(backups_root))?;
    let save_abs = fs::canonicalize(save_dir).map_err(io_error(save_dir))?;
    let root_abs = fs::canonicalize(backups_root).map_err(io_error(backups_root))?;
    if root_abs.starts_with(&save_abs) {
        return Err(BackupError::BackupsInsideSave);
    }

    let final_dir = unique_dir(backups_root, stamp);
    let partial = PathBuf::from(format!("{}{PARTIAL_SUFFIX}", final_dir.display()));
    let result = copy_and_verify(save_dir, &partial).and_then(|info| {
        fs::rename(&partial, &final_dir).map_err(io_error(&final_dir))?;
        Ok(BackupInfo {
            path: final_dir.clone(),
            ..info
        })
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&partial); // só a cópia parcial deste backup
    }
    result
}

fn unique_dir(root: &Path, stamp: &str) -> PathBuf {
    let mut candidate = root.join(stamp);
    let mut n = 2;
    while candidate.exists() {
        candidate = root.join(format!("{stamp}-{n}"));
        n += 1;
    }
    candidate
}

fn copy_and_verify(source: &Path, target: &Path) -> Result<BackupInfo, BackupError> {
    copy_tree(source, target)?;
    let expected = list_tree(source)?;
    let actual = list_tree(target)?;
    if expected != actual {
        return Err(BackupError::VerificationFailed(
            "lista de arquivos ou tamanhos diferentes".into(),
        ));
    }
    Ok(BackupInfo {
        path: target.to_path_buf(),
        files: expected.len(),
        bytes: expected.values().sum(),
    })
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), BackupError> {
    fs::create_dir_all(target).map_err(io_error(target))?;
    for entry in fs::read_dir(source).map_err(io_error(source))? {
        let entry = entry.map_err(io_error(source))?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if entry.file_type().map_err(io_error(&from))?.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(io_error(&from))?;
        }
    }
    Ok(())
}

/// Arquivos de uma árvore (caminho relativo → tamanho), em ordem.
fn list_tree(root: &Path) -> Result<BTreeMap<PathBuf, u64>, BackupError> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, u64>) -> Result<(), BackupError> {
        for entry in fs::read_dir(dir).map_err(io_error(dir))? {
            let entry = entry.map_err(io_error(dir))?;
            let path = entry.path();
            let meta = entry.metadata().map_err(io_error(&path))?;
            if meta.is_dir() {
                walk(base, &path, out)?;
            } else {
                let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
                out.insert(relative, meta.len());
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files)?;
    Ok(files)
}

/// As duas pastas têm os mesmos arquivos com exatamente o mesmo conteúdo.
///
/// # Errors
///
/// Retorna [`BackupError`] se alguma das pastas não puder ser lida.
pub fn dirs_identical(a: &Path, b: &Path) -> Result<bool, BackupError> {
    let (left, right) = (list_tree(a)?, list_tree(b)?);
    if left != right {
        return Ok(false);
    }
    for relative in left.keys() {
        let x = fs::read(a.join(relative)).map_err(io_error(&a.join(relative)))?;
        let y = fs::read(b.join(relative)).map_err(io_error(&b.join(relative)))?;
        if x != y {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Devolve a pasta do save exatamente ao estado de `backup`.
///
/// Arquivos que não existiam no backup são removidos de `save_dir`; os demais são
/// sobrescritos. O chamador deve ter feito um backup do estado atual antes (a
/// `SaveTransaction` faz isso) e conferido que o jogo está fechado.
///
/// # Errors
///
/// Retorna [`BackupError`] se a cópia falhar ou o resultado não ficar idêntico ao backup.
pub fn restore_backup(backup: &Path, save_dir: &Path) -> Result<(), BackupError> {
    if !backup.is_dir() {
        return Err(BackupError::NotADirectory(backup.to_path_buf()));
    }
    fs::create_dir_all(save_dir).map_err(io_error(save_dir))?;
    let wanted = list_tree(backup)?;
    for relative in list_tree(save_dir)?
        .keys()
        .filter(|r| !wanted.contains_key(*r))
    {
        let path = save_dir.join(relative);
        fs::remove_file(&path).map_err(io_error(&path))?;
    }
    copy_tree(backup, save_dir)?;
    if dirs_identical(backup, save_dir)? {
        Ok(())
    } else {
        Err(BackupError::VerificationFailed(
            "a pasta restaurada difere do backup".into(),
        ))
    }
}

/// Restaura só os arquivos `names` a partir de `backup`; o que não existia no backup é removido.
///
/// Usado para desfazer uma gravação que falhou sem mexer no resto da pasta.
///
/// # Errors
///
/// Retorna [`BackupError`] se alguma cópia ou remoção falhar.
pub fn restore_files(backup: &Path, save_dir: &Path, names: &[&str]) -> Result<(), BackupError> {
    for name in names {
        let (from, to) = (backup.join(name), save_dir.join(name));
        if from.is_file() {
            fs::copy(&from, &to).map_err(io_error(&to))?;
        } else if to.exists() {
            fs::remove_file(&to).map_err(io_error(&to))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_files_touches_only_the_named_files() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        let backup = create_backup(save.path(), backups.path(), "t").unwrap();
        fs::write(save.path().join("settings_adf"), b"alterado").unwrap();
        fs::write(save.path().join("novo"), b"nasceu depois").unwrap();
        fs::write(
            save.path().join("slots/animal_population_0"),
            b"outra mudanca",
        )
        .unwrap();

        restore_files(&backup.path, save.path(), &["settings_adf", "novo"]).unwrap();
        assert_eq!(
            fs::read(save.path().join("settings_adf")).unwrap(),
            b"configuracoes"
        );
        assert!(!save.path().join("novo").exists());
        // Fora da lista: continua como estava.
        assert_eq!(
            fs::read(save.path().join("slots/animal_population_0")).unwrap(),
            b"outra mudanca"
        );
    }

    fn save_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("settings_adf"), b"configuracoes").unwrap();
        fs::create_dir(dir.path().join("slots")).unwrap();
        fs::write(dir.path().join("slots/animal_population_0"), b"animais").unwrap();
        dir
    }

    #[test]
    fn copies_the_whole_folder_and_reports_totals() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        let info = create_backup(save.path(), backups.path(), "2026-11-02_15-30-00").unwrap();
        assert_eq!(info.path, backups.path().join("2026-11-02_15-30-00"));
        assert_eq!((info.files, info.bytes), (2, 20));
        assert!(dirs_identical(save.path(), &info.path).unwrap());
    }

    #[test]
    fn same_stamp_never_overwrites_an_earlier_backup() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        let first = create_backup(save.path(), backups.path(), "t").unwrap();
        fs::write(save.path().join("settings_adf"), b"depois").unwrap();
        let second = create_backup(save.path(), backups.path(), "t").unwrap();
        assert_eq!(second.path, backups.path().join("t-2"));
        assert_eq!(
            fs::read(first.path.join("settings_adf")).unwrap(),
            b"configuracoes"
        );
        assert_eq!(
            fs::read(second.path.join("settings_adf")).unwrap(),
            b"depois"
        );
    }

    #[test]
    fn no_partial_folder_is_left_behind() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        create_backup(save.path(), backups.path(), "t").unwrap();
        let names: Vec<_> = fs::read_dir(backups.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["t"]);
    }

    #[test]
    fn missing_source_is_an_error_and_creates_nothing() {
        let backups = tempfile::tempdir().unwrap();
        let missing = backups.path().join("nao-existe");
        assert!(matches!(
            create_backup(&missing, &backups.path().join("b"), "t"),
            Err(BackupError::NotADirectory(_))
        ));
        assert!(!backups.path().join("b").exists());
    }

    #[test]
    fn backups_folder_inside_the_save_is_refused() {
        let save = save_dir();
        assert!(matches!(
            create_backup(save.path(), &save.path().join("backups"), "t"),
            Err(BackupError::BackupsInsideSave)
        ));
    }

    #[test]
    fn failed_copy_removes_only_its_partial_folder() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        let earlier = create_backup(save.path(), backups.path(), "antigo").unwrap();
        // Um arquivo no lugar da pasta final faz a cópia falhar no meio.
        fs::write(backups.path().join("novo.partial"), b"bloqueio").unwrap();
        let result = create_backup(save.path(), backups.path(), "novo");
        assert!(result.is_err());
        assert!(earlier.path.is_dir(), "backup anterior intacto");
        assert!(!backups.path().join("novo").exists());
    }

    #[test]
    fn restore_returns_the_folder_to_the_exact_backup_state() {
        let (save, backups) = (save_dir(), tempfile::tempdir().unwrap());
        let backup = create_backup(save.path(), backups.path(), "t").unwrap();
        fs::write(save.path().join("settings_adf"), b"alterado").unwrap();
        fs::write(save.path().join("novo_arquivo"), b"extra").unwrap();
        fs::remove_file(save.path().join("slots/animal_population_0")).unwrap();

        restore_backup(&backup.path, save.path()).unwrap();
        assert!(dirs_identical(&backup.path, save.path()).unwrap());
        assert!(!save.path().join("novo_arquivo").exists());
    }

    #[test]
    fn restore_from_a_missing_backup_changes_nothing() {
        let save = save_dir();
        let missing = save.path().join("nao-existe");
        assert!(matches!(
            restore_backup(&missing, save.path()),
            Err(BackupError::NotADirectory(_))
        ));
        assert_eq!(
            fs::read(save.path().join("settings_adf")).unwrap(),
            b"configuracoes"
        );
    }

    #[test]
    fn dirs_identical_detects_content_and_name_differences() {
        let (a, b) = (save_dir(), save_dir());
        assert!(dirs_identical(a.path(), b.path()).unwrap());
        fs::write(b.path().join("settings_adf"), b"configuracoeS").unwrap();
        assert!(!dirs_identical(a.path(), b.path()).unwrap());
        fs::write(b.path().join("settings_adf"), b"configuracoes").unwrap();
        fs::write(b.path().join("extra"), b"x").unwrap();
        assert!(!dirs_identical(a.path(), b.path()).unwrap());
    }

    #[test]
    fn timestamp_has_the_expected_shape() {
        let stamp = timestamp_now();
        assert_eq!(stamp.len(), 19);
        assert_eq!(stamp.as_bytes()[10], b'_');
    }
}
