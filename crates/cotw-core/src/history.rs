//! Histórico das alterações e restaurações, em SQLite (spec `safe-save-writing`).

use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};

/// Erros do histórico.
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    /// Falha do banco de dados.
    #[error("erro no histórico: {0}")]
    Database(#[from] rusqlite::Error),
    /// Não foi possível criar a pasta do banco.
    #[error("erro de E/S no histórico: {0}")]
    Io(#[from] std::io::Error),
}

/// Tipo de registro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryKind {
    /// Alteração aplicada ao save.
    Change,
    /// Restauração de um backup.
    Restore,
}

impl HistoryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Change => "change",
            Self::Restore => "restore",
        }
    }

    fn from_str(text: &str) -> Self {
        if text == "restore" {
            Self::Restore
        } else {
            Self::Change
        }
    }
}

/// Dados de um novo registro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    /// Data e hora (mesmo formato dos nomes de backup).
    pub at: String,
    /// Alteração ou restauração.
    pub kind: HistoryKind,
    /// Função do app que originou o registro (ex.: "Liberar zonas").
    pub function: String,
    /// Resumo legível do que mudou.
    pub summary: String,
    /// Arquivos afetados.
    pub files: Vec<String>,
    /// Backup feito antes da gravação.
    pub backup: PathBuf,
}

/// Registro lido do banco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Número do registro.
    pub id: i64,
    /// Dados gravados.
    pub data: NewEntry,
    /// O backup referenciado não existe mais (por exemplo, foi apagado à mão).
    pub backup_missing: bool,
}

/// Banco do histórico.
#[derive(Debug)]
pub struct History {
    conn: Connection,
}

impl History {
    /// Abre (criando, se preciso) o banco em `path`.
    ///
    /// # Errors
    ///
    /// Retorna [`HistoryError`] se o arquivo ou o esquema não puderem ser criados.
    pub fn open(path: &Path) -> Result<Self, HistoryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::init(Connection::open(path)?)
    }

    /// Banco em memória, para testes.
    ///
    /// # Errors
    ///
    /// Retorna [`HistoryError`] se o esquema não puder ser criado.
    pub fn open_in_memory() -> Result<Self, HistoryError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, HistoryError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                at       TEXT NOT NULL,
                kind     TEXT NOT NULL,
                function TEXT NOT NULL,
                summary  TEXT NOT NULL,
                files    TEXT NOT NULL,
                backup   TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    /// Remove a tabela para simular um banco com defeito nos testes de outros módulos.
    #[cfg(test)]
    pub(crate) fn break_for_tests(&self) {
        self.conn.execute_batch("DROP TABLE history;").unwrap();
    }

    /// Acrescenta um registro e devolve o número dele.
    ///
    /// # Errors
    ///
    /// Retorna [`HistoryError`] se a gravação no banco falhar.
    pub fn record(&self, entry: &NewEntry) -> Result<i64, HistoryError> {
        self.conn.execute(
            "INSERT INTO history (at, kind, function, summary, files, backup)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.at,
                entry.kind.as_str(),
                entry.function,
                entry.summary,
                entry.files.join("\n"),
                entry.backup.to_string_lossy(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Troca o início dos caminhos de backup de `from` para `to` (pasta de dados que mudou de lugar).
    ///
    /// # Errors
    ///
    /// Retorna [`HistoryError`] se a atualização falhar.
    pub fn relocate_backups(&self, from: &Path, to: &Path) -> Result<usize, HistoryError> {
        let from = from.to_string_lossy();
        let to = to.to_string_lossy();
        Ok(self.conn.execute(
            "UPDATE history SET backup = ?2 || substr(backup, length(?1) + 1)
             WHERE substr(backup, 1, length(?1)) = ?1",
            params![from, to],
        )?)
    }

    /// Todos os registros, do mais recente para o mais antigo.
    ///
    /// # Errors
    ///
    /// Retorna [`HistoryError`] se a leitura do banco falhar.
    pub fn list(&self) -> Result<Vec<Entry>, HistoryError> {
        let mut statement = self.conn.prepare(
            "SELECT id, at, kind, function, summary, files, backup FROM history ORDER BY id DESC",
        )?;
        let rows = statement.query_map([], |row| {
            let files: String = row.get(5)?;
            let backup = PathBuf::from(row.get::<_, String>(6)?);
            Ok(Entry {
                id: row.get(0)?,
                backup_missing: !backup.is_dir(),
                data: NewEntry {
                    at: row.get(1)?,
                    kind: HistoryKind::from_str(&row.get::<_, String>(2)?),
                    function: row.get(3)?,
                    summary: row.get(4)?,
                    files: files
                        .split('\n')
                        .filter(|f| !f.is_empty())
                        .map(str::to_owned)
                        .collect(),
                    backup,
                },
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(summary: &str, backup: &Path) -> NewEntry {
        NewEntry {
            at: "2026-11-02_15-30-00".into(),
            kind: HistoryKind::Change,
            function: "Liberar zonas".into(),
            summary: summary.into(),
            files: vec!["found_need_zones_adf".into(), "animal_population_2".into()],
            backup: backup.to_path_buf(),
        }
    }

    #[test]
    fn records_come_back_newest_first_with_all_fields() {
        let backup = tempfile::tempdir().unwrap();
        let history = History::open_in_memory().unwrap();
        let first = history.record(&entry("primeira", backup.path())).unwrap();
        let second = history.record(&entry("segunda", backup.path())).unwrap();
        let list = history.list().unwrap();
        assert_eq!(
            list.iter().map(|e| e.id).collect::<Vec<_>>(),
            [second, first]
        );
        assert_eq!(list[0].data, entry("segunda", backup.path()));
        assert!(!list[0].backup_missing);
    }

    #[test]
    fn restore_entries_keep_their_kind() {
        let history = History::open_in_memory().unwrap();
        let mut restore = entry("restaurado", Path::new("x"));
        restore.kind = HistoryKind::Restore;
        history.record(&restore).unwrap();
        assert_eq!(history.list().unwrap()[0].data.kind, HistoryKind::Restore);
    }

    #[test]
    fn warns_when_the_referenced_backup_is_gone() {
        let history = History::open_in_memory().unwrap();
        let gone = tempfile::tempdir().unwrap().path().join("apagado");
        history.record(&entry("sem backup", &gone)).unwrap();
        assert!(history.list().unwrap()[0].backup_missing);
    }

    #[test]
    fn empty_history_is_an_empty_list() {
        assert_eq!(History::open_in_memory().unwrap().list().unwrap(), []);
    }

    #[test]
    fn persists_across_reopening_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dados/history.db");
        History::open(&path)
            .unwrap()
            .record(&entry("persistido", dir.path()))
            .unwrap();
        let list = History::open(&path).unwrap().list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].data.summary, "persistido");
    }

    #[test]
    fn unopenable_path_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        // Um arquivo no lugar da pasta pai impede a criação do banco.
        let blocker = dir.path().join("arquivo");
        std::fs::write(&blocker, b"x").unwrap();
        assert!(History::open(&blocker.join("history.db")).is_err());
    }

    #[test]
    fn relocates_only_backups_under_the_old_folder() {
        let history = History::open_in_memory().unwrap();
        let mut moved = entry("a", Path::new("/velho/backups/1"));
        let other = entry("b", Path::new("/outro/backups/2"));
        history.record(&moved).unwrap();
        history.record(&other).unwrap();

        let changed = history
            .relocate_backups(Path::new("/velho"), Path::new("/novo"))
            .unwrap();

        assert_eq!(changed, 1);
        let listed = history.list().unwrap();
        moved.backup = PathBuf::from("/novo/backups/1");
        assert_eq!(listed[1].data, moved);
        assert_eq!(listed[0].data, other);
    }
}
