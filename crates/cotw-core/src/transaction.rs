//! `SaveTransaction`: o único caminho de escrita no save (spec `safe-save-writing`).
//!
//! Fluxo: [`SaveTransaction::preview`] (só leitura) → o usuário confere e confirma
//! ([`Preview::confirm`]) → [`SaveTransaction::commit`]: jogo fechado → backup completo →
//! gravação atômica de cada arquivo → releitura e conferência → histórico. Qualquer falha
//! depois do backup desfaz os arquivos tocados. Cancelar é simplesmente descartar a prévia:
//! nada é escrito e nenhum backup é criado.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::backup::{self, BackupError, BackupInfo};
use crate::history::{History, HistoryError, HistoryKind, NewEntry};
use crate::process::{GameStateError, ProcessList, ensure_game_closed};

/// Conferência extra de um arquivo gravado: recebe os bytes relidos do disco.
pub type Verifier = Box<dyn Fn(&[u8]) -> Result<(), String> + Send + Sync>;

/// Um item afetado, para a prévia (antes e depois).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemChange {
    /// O que muda (ex.: "Zona de beber do grupo 3").
    pub label: String,
    /// Valor antes.
    pub before: String,
    /// Valor depois.
    pub after: String,
}

/// Novo conteúdo de um arquivo do save.
pub struct FileChange {
    /// Nome do arquivo dentro da pasta do save (sem caminho).
    pub name: String,
    /// Conteúdo completo que o arquivo terá.
    pub new_content: Vec<u8>,
    /// Itens afetados, mostrados na prévia.
    pub items: Vec<ItemChange>,
    /// Conferência adicional depois de gravar (por exemplo, reler com o parser do formato).
    pub verify: Option<Verifier>,
}

/// Uma alteração completa, possivelmente com vários arquivos.
pub struct Change {
    /// Função do app que originou a alteração.
    pub function: String,
    /// Resumo para o histórico.
    pub summary: String,
    /// Arquivos a gravar.
    pub files: Vec<FileChange>,
}

/// O que vai acontecer com um arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePreview {
    /// Nome do arquivo.
    pub name: String,
    /// O arquivo já existe.
    pub exists: bool,
    /// Tamanho atual (0 se não existe).
    pub old_size: u64,
    /// Tamanho depois da gravação.
    pub new_size: u64,
    /// Itens afetados.
    pub items: Vec<ItemChange>,
}

/// Prévia de uma alteração; só vira gravação depois de [`Preview::confirm`].
pub struct Preview {
    /// Um registro por arquivo.
    pub files: Vec<FilePreview>,
    change: Change,
}

impl Preview {
    /// Confirmação explícita do usuário.
    #[must_use]
    pub fn confirm(self) -> Confirmed {
        Confirmed(self.change)
    }
}

/// Alteração confirmada, pronta para [`SaveTransaction::commit`].
pub struct Confirmed(Change);

/// Resultado de uma gravação ou restauração concluída.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Committed {
    /// Backup feito antes de gravar.
    pub backup: BackupInfo,
    /// Número do registro no histórico, se foi possível gravá-lo.
    pub history_id: Option<i64>,
    /// Motivo, se o registro no histórico falhou (a alteração foi aplicada mesmo assim).
    pub history_error: Option<String>,
}

/// Erros da transação.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    /// O jogo está aberto ou não foi possível verificar. Nada foi alterado.
    #[error(transparent)]
    Game(#[from] GameStateError),
    /// Nome de arquivo inválido (com caminho ou vazio). Nada foi alterado.
    #[error("nome de arquivo inválido: {0:?}")]
    InvalidFileName(String),
    /// A alteração não tem nenhum arquivo.
    #[error("a alteração não tem arquivos")]
    Empty,
    /// O backup não pôde ser criado. Nada foi alterado.
    #[error("não foi possível criar o backup: {0}")]
    Backup(#[from] BackupError),
    /// Falha ao gravar ou conferir; os arquivos foram restaurados a partir do backup.
    #[error("gravação desfeita, nada foi alterado: {reason}")]
    RolledBack {
        /// O que falhou.
        reason: String,
        /// Backup usado na restauração.
        backup: PathBuf,
    },
    /// Falha ao gravar E ao restaurar: o backup é a única cópia segura.
    #[error("falha ao desfazer a gravação ({reason}); restaure manualmente o backup em {}", backup.display())]
    RollbackFailed {
        /// O que falhou ao desfazer.
        reason: String,
        /// Backup com o estado original.
        backup: PathBuf,
    },
    /// Falha ao ler o estado atual do save.
    #[error("erro de E/S: {0}")]
    Io(#[from] io::Error),
}

/// Gravação segura numa pasta de save.
#[derive(Debug, Clone)]
pub struct SaveTransaction {
    save_dir: PathBuf,
    backups_root: PathBuf,
}

impl SaveTransaction {
    /// Cria a transação para `save_dir`, guardando backups em `backups_root`.
    #[must_use]
    pub fn new(save_dir: &Path, backups_root: &Path) -> Self {
        Self {
            save_dir: save_dir.to_path_buf(),
            backups_root: backups_root.to_path_buf(),
        }
    }

    /// Monta a prévia sem escrever nada.
    ///
    /// # Errors
    ///
    /// Retorna [`TransactionError`] se a alteração for vazia ou tiver nome de arquivo inválido.
    pub fn preview(&self, change: Change) -> Result<Preview, TransactionError> {
        if change.files.is_empty() {
            return Err(TransactionError::Empty);
        }
        let mut files = Vec::new();
        for file in &change.files {
            if !is_plain_file_name(&file.name) {
                return Err(TransactionError::InvalidFileName(file.name.clone()));
            }
            let current = fs::metadata(self.save_dir.join(&file.name)).ok();
            files.push(FilePreview {
                name: file.name.clone(),
                exists: current.is_some(),
                old_size: current.map_or(0, |m| m.len()),
                new_size: file.new_content.len() as u64,
                items: file.items.clone(),
            });
        }
        Ok(Preview { files, change })
    }

    /// Aplica a alteração confirmada.
    ///
    /// # Errors
    ///
    /// Veja [`TransactionError`]: bloqueios e falhas de backup deixam tudo como estava; falhas
    /// posteriores desfazem os arquivos tocados.
    pub fn commit(
        &self,
        confirmed: Confirmed,
        processes: &dyn ProcessList,
        history: &History,
    ) -> Result<Committed, TransactionError> {
        let change = confirmed.0;
        // Revalida: a prévia pode ter sido montada em outro momento.
        for file in &change.files {
            if !is_plain_file_name(&file.name) {
                return Err(TransactionError::InvalidFileName(file.name.clone()));
            }
        }
        ensure_game_closed(processes)?;
        let backup =
            backup::create_backup(&self.save_dir, &self.backups_root, &backup::timestamp_now())?;

        let names: Vec<&str> = change.files.iter().map(|f| f.name.as_str()).collect();
        if let Err(reason) = self.write_and_verify(&change) {
            return match backup::restore_files(&backup.path, &self.save_dir, &names) {
                Ok(()) => Err(TransactionError::RolledBack {
                    reason,
                    backup: backup.path,
                }),
                Err(e) => Err(TransactionError::RollbackFailed {
                    reason: format!("{reason}; ao desfazer: {e}"),
                    backup: backup.path,
                }),
            };
        }

        let entry = NewEntry {
            at: backup::timestamp_now(),
            kind: HistoryKind::Change,
            function: change.function,
            summary: change.summary,
            files: names.iter().map(|n| (*n).to_owned()).collect(),
            backup: backup.path.clone(),
        };
        Ok(record(history, &entry, backup))
    }

    /// Restaura um backup completo (com o jogo fechado) e registra a restauração.
    ///
    /// Antes de restaurar, guarda o estado atual em um novo backup, para que nada se perca.
    ///
    /// # Errors
    ///
    /// Retorna [`TransactionError`] se o jogo estiver aberto, o backup de segurança falhar ou a
    /// restauração falhar.
    pub fn restore(
        &self,
        backup_dir: &Path,
        processes: &dyn ProcessList,
        history: &History,
    ) -> Result<Committed, TransactionError> {
        ensure_game_closed(processes)?;
        let safety =
            backup::create_backup(&self.save_dir, &self.backups_root, &backup::timestamp_now())?;
        backup::restore_backup(backup_dir, &self.save_dir)?;
        let entry = NewEntry {
            at: backup::timestamp_now(),
            kind: HistoryKind::Restore,
            function: "Restaurar backup".into(),
            summary: format!(
                "Pasta do save restaurada a partir de {}",
                backup_dir.display()
            ),
            files: Vec::new(),
            backup: safety.path.clone(),
        };
        Ok(record(history, &entry, safety))
    }

    /// Grava cada arquivo (temporário + renomear) e relê para conferir.
    fn write_and_verify(&self, change: &Change) -> Result<(), String> {
        for file in &change.files {
            let target = self.save_dir.join(&file.name);
            let temp = self.save_dir.join(format!("{}.cotw-tmp", file.name));
            fs::write(&temp, &file.new_content)
                .and_then(|()| fs::rename(&temp, &target))
                .map_err(|e| {
                    let _ = fs::remove_file(&temp);
                    format!("não foi possível gravar {}: {e}", file.name)
                })?;
        }
        for file in &change.files {
            let on_disk = fs::read(self.save_dir.join(&file.name))
                .map_err(|e| format!("não foi possível reler {}: {e}", file.name))?;
            if on_disk != file.new_content {
                return Err(format!("{} gravado difere do esperado", file.name));
            }
            if let Some(verify) = &file.verify {
                verify(&on_disk)
                    .map_err(|e| format!("{} não passou na conferência: {e}", file.name))?;
            }
        }
        Ok(())
    }
}

fn record(history: &History, entry: &NewEntry, backup: BackupInfo) -> Committed {
    match history.record(entry) {
        Ok(id) => Committed {
            backup,
            history_id: Some(id),
            history_error: None,
        },
        Err(e @ (HistoryError::Database(_) | HistoryError::Io(_))) => Committed {
            backup,
            history_id: None,
            history_error: Some(e.to_string()),
        },
    }
}

/// Só um nome de arquivo: sem separadores, sem `..`, sem vazio.
fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':'])
        && !name.ends_with(".cotw-tmp")
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    struct Processes(Vec<&'static str>);

    impl ProcessList for Processes {
        fn running_image_names(&self) -> io::Result<Vec<String>> {
            Ok(self.0.iter().map(|s| (*s).to_owned()).collect())
        }
    }

    struct Broken;

    impl ProcessList for Broken {
        fn running_image_names(&self) -> io::Result<Vec<String>> {
            Err(io::Error::other("sem tasklist"))
        }
    }

    const CLOSED: fn() -> Processes = || Processes(vec!["explorer.exe"]);

    struct Env {
        _root: tempfile::TempDir,
        save: PathBuf,
        backups: PathBuf,
        history: History,
    }

    fn env() -> Env {
        let root = tempfile::tempdir().unwrap();
        let save = root.path().join("save");
        fs::create_dir_all(&save).unwrap();
        fs::write(save.join("a_adf"), b"original A").unwrap();
        fs::write(save.join("b_adf"), b"original B").unwrap();
        Env {
            backups: root.path().join("backups"),
            save,
            history: History::open_in_memory().unwrap(),
            _root: root,
        }
    }

    fn file(name: &str, content: &[u8]) -> FileChange {
        FileChange {
            name: name.into(),
            new_content: content.to_vec(),
            items: vec![ItemChange {
                label: "item".into(),
                before: "x".into(),
                after: "y".into(),
            }],
            verify: None,
        }
    }

    fn change(files: Vec<FileChange>) -> Change {
        Change {
            function: "Teste".into(),
            summary: "alterou coisas".into(),
            files,
        }
    }

    fn read(env: &Env, name: &str) -> Vec<u8> {
        fs::read(env.save.join(name)).unwrap()
    }

    fn tx(env: &Env) -> SaveTransaction {
        SaveTransaction::new(&env.save, &env.backups)
    }

    #[test]
    fn preview_describes_the_change_and_writes_nothing() {
        let env = env();
        let preview = tx(&env)
            .preview(change(vec![
                file("a_adf", b"novo A!"),
                file("novo_adf", b"nasce"),
            ]))
            .unwrap();
        assert_eq!(preview.files[0].old_size, 10);
        assert_eq!(preview.files[0].new_size, 7);
        assert!(preview.files[0].exists);
        assert!(!preview.files[1].exists);
        assert_eq!(preview.files[0].items[0].after, "y");
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert!(!env.backups.exists(), "prévia não cria backup");
    }

    #[test]
    fn cancelling_leaves_files_and_backups_untouched() {
        let env = env();
        drop(
            tx(&env)
                .preview(change(vec![file("a_adf", b"novo")]))
                .unwrap(),
        );
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert!(!env.backups.exists());
        assert_eq!(env.history.list().unwrap(), []);
    }

    #[test]
    fn commit_backs_up_first_then_writes_and_records_history() {
        let env = env();
        let confirmed = tx(&env)
            .preview(change(vec![
                file("a_adf", b"novo A"),
                file("novo_adf", b"nasce"),
            ]))
            .unwrap()
            .confirm();
        let done = tx(&env).commit(confirmed, &CLOSED(), &env.history).unwrap();

        assert_eq!(read(&env, "a_adf"), b"novo A");
        assert_eq!(read(&env, "novo_adf"), b"nasce");
        assert_eq!(read(&env, "b_adf"), b"original B");
        // O backup guarda o estado anterior completo.
        assert_eq!(
            fs::read(done.backup.path.join("a_adf")).unwrap(),
            b"original A"
        );
        assert_eq!(
            fs::read(done.backup.path.join("b_adf")).unwrap(),
            b"original B"
        );
        assert!(!done.backup.path.join("novo_adf").exists());

        let entries = env.history.list().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, done.history_id.unwrap());
        assert_eq!(entries[0].data.files, ["a_adf", "novo_adf"]);
        assert_eq!(entries[0].data.kind, HistoryKind::Change);
        assert_eq!(done.history_error, None);
        // Nenhum temporário sobra.
        assert!(!env.save.join("a_adf.cotw-tmp").exists());
    }

    #[test]
    fn running_game_blocks_everything() {
        let env = env();
        let confirmed = tx(&env)
            .preview(change(vec![file("a_adf", b"novo")]))
            .unwrap()
            .confirm();
        let game = Processes(vec!["theHunterCotW_F.exe"]);
        let result = tx(&env).commit(confirmed, &game, &env.history);
        assert!(matches!(
            result,
            Err(TransactionError::Game(GameStateError::Running))
        ));
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert!(!env.backups.exists());
        assert_eq!(env.history.list().unwrap(), []);
    }

    #[test]
    fn unknown_game_state_blocks_too() {
        let env = env();
        let confirmed = tx(&env)
            .preview(change(vec![file("a_adf", b"novo")]))
            .unwrap()
            .confirm();
        let result = tx(&env).commit(confirmed, &Broken, &env.history);
        assert!(matches!(
            result,
            Err(TransactionError::Game(GameStateError::Unknown(_)))
        ));
        assert_eq!(read(&env, "a_adf"), b"original A");
    }

    #[test]
    fn backup_failure_writes_nothing() {
        let env = env();
        // Um arquivo no lugar da pasta de backups impede o backup.
        fs::write(&env.backups, b"bloqueio").unwrap();
        let confirmed = tx(&env)
            .preview(change(vec![file("a_adf", b"novo")]))
            .unwrap()
            .confirm();
        let result = tx(&env).commit(confirmed, &CLOSED(), &env.history);
        assert!(matches!(result, Err(TransactionError::Backup(_))));
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert_eq!(env.history.list().unwrap(), []);
    }

    #[test]
    fn failed_verification_restores_every_touched_file() {
        let env = env();
        let mut bad = file("b_adf", b"B quebrado");
        bad.verify = Some(Box::new(|_| Err("ponteiro inválido".into())));
        let confirmed = tx(&env)
            .preview(change(vec![
                file("a_adf", b"A novo"),
                bad,
                file("novo_adf", b"nasce"),
            ]))
            .unwrap()
            .confirm();
        let result = tx(&env).commit(confirmed, &CLOSED(), &env.history);

        let Err(TransactionError::RolledBack { reason, backup }) = result else {
            panic!("esperava RolledBack");
        };
        assert!(reason.contains("ponteiro inválido"));
        assert!(backup.is_dir());
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert_eq!(read(&env, "b_adf"), b"original B");
        assert!(!env.save.join("novo_adf").exists(), "arquivo novo removido");
        assert_eq!(env.history.list().unwrap(), [], "nada registrado");
    }

    #[test]
    fn verifier_receives_the_bytes_read_back_from_disk() {
        let env = env();
        let mut checked = file("a_adf", b"conteudo");
        checked.verify = Some(Box::new(|bytes| {
            if bytes == b"conteudo" {
                Ok(())
            } else {
                Err("diferente".into())
            }
        }));
        let confirmed = tx(&env).preview(change(vec![checked])).unwrap().confirm();
        assert!(tx(&env).commit(confirmed, &CLOSED(), &env.history).is_ok());
    }

    #[test]
    fn write_failure_midway_undoes_the_files_already_written() {
        let env = env();
        // Uma pasta no lugar do temporário de `b_adf` faz a segunda gravação falhar.
        fs::create_dir(env.save.join("b_adf.cotw-tmp")).unwrap();
        let confirmed = tx(&env)
            .preview(change(vec![
                file("a_adf", b"A novo"),
                file("b_adf", b"B novo"),
            ]))
            .unwrap()
            .confirm();
        let result = tx(&env).commit(confirmed, &CLOSED(), &env.history);
        assert!(
            matches!(result, Err(TransactionError::RolledBack { .. })),
            "{result:?}"
        );
        assert_eq!(read(&env, "a_adf"), b"original A");
        assert_eq!(read(&env, "b_adf"), b"original B");
        assert_eq!(env.history.list().unwrap(), []);
    }

    #[test]
    fn failing_to_undo_reports_the_backup_that_holds_the_original_state() {
        let env = env();
        // `b_adf` vira uma pasta: a gravação falha e restaurar por cima dela também.
        fs::remove_file(env.save.join("b_adf")).unwrap();
        fs::create_dir(env.save.join("b_adf")).unwrap();
        let confirmed = tx(&env)
            .preview(change(vec![
                file("a_adf", b"A novo"),
                file("b_adf", b"B novo"),
            ]))
            .unwrap()
            .confirm();
        let result = tx(&env).commit(confirmed, &CLOSED(), &env.history);
        let Err(TransactionError::RollbackFailed { backup, .. }) = result else {
            panic!("esperava RollbackFailed, veio {result:?}");
        };
        assert_eq!(fs::read(backup.join("a_adf")).unwrap(), b"original A");
        assert_eq!(env.history.list().unwrap(), []);
    }

    #[test]
    fn invalid_file_names_are_rejected() {
        let env = env();
        for name in [
            "",
            ".",
            "..",
            "../fora",
            "sub/arquivo",
            "sub\\arquivo",
            "C:arq",
            "x.cotw-tmp",
        ] {
            assert!(
                matches!(
                    tx(&env).preview(change(vec![file(name, b"x")])),
                    Err(TransactionError::InvalidFileName(_))
                ),
                "{name:?}"
            );
        }
        assert!(matches!(
            tx(&env).preview(change(vec![])),
            Err(TransactionError::Empty)
        ));
    }

    #[test]
    fn restore_returns_to_a_backup_and_keeps_a_safety_copy() {
        let env = env();
        let confirmed = tx(&env)
            .preview(change(vec![file("a_adf", b"A novo")]))
            .unwrap()
            .confirm();
        let done = tx(&env).commit(confirmed, &CLOSED(), &env.history).unwrap();
        assert_eq!(read(&env, "a_adf"), b"A novo");

        let restored = tx(&env)
            .restore(&done.backup.path, &CLOSED(), &env.history)
            .unwrap();
        assert_eq!(read(&env, "a_adf"), b"original A");
        // A cópia de segurança guarda o estado de antes da restauração.
        assert_eq!(
            fs::read(restored.backup.path.join("a_adf")).unwrap(),
            b"A novo"
        );

        let kinds: Vec<_> = env
            .history
            .list()
            .unwrap()
            .iter()
            .map(|e| e.data.kind)
            .collect();
        assert_eq!(kinds, [HistoryKind::Restore, HistoryKind::Change]);
    }

    #[test]
    fn restore_is_blocked_while_the_game_runs() {
        let env = env();
        let game = Processes(vec!["theHunterCotW_F.exe"]);
        let result = tx(&env).restore(&env.save, &game, &env.history);
        assert!(matches!(result, Err(TransactionError::Game(_))));
        assert!(!env.backups.exists());
    }

    #[test]
    fn history_failure_does_not_undo_a_successful_write() {
        let env = env();
        let bad_history = History::open_in_memory().unwrap();
        bad_history.break_for_tests();
        let confirmed = tx(&env)
            .preview(change(vec![file("a_adf", b"A novo")]))
            .unwrap()
            .confirm();
        let done = tx(&env).commit(confirmed, &CLOSED(), &bad_history).unwrap();
        assert_eq!(read(&env, "a_adf"), b"A novo");
        assert_eq!(done.history_id, None);
        assert!(done.history_error.is_some());
    }
}
