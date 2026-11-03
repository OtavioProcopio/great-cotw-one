//! Serviço de aplicação: o que a interface chama. Reúne descoberta, histórico e restauração
//! com as pastas do app, para que a camada de interface (Tauri) seja só repasse de chamadas.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use cotw_formats::adf::Adf;
use cotw_formats::archive::GameArchives;
use cotw_formats::envelope::SaveEnvelope;

use crate::animals::{self, AnimalReserve};
use crate::career::{self, CareerError, ProfileView, ReserveMissions};
use crate::diagnostics::{Diagnostics, FileDiagnostic, FileState, SaveStatus, SearchRoots, run};
use crate::discovery::saves::{SaveLocation, resolve_manual};
use crate::discovery::steam::validate_game_dir;
use crate::history::{History, HistoryError, HistoryKind};
use crate::mapview::{self, MapError, MapReserve, ReserveMap};
use crate::paths::{app_data_dir, backups_dir, history_db, migrate_legacy_dir};
use crate::process::ProcessList;
use crate::settings::Settings;
use crate::transaction::{Change, FileChange, SaveTransaction, TransactionError};
use crate::unlock::{ReserveSummary, UnlockError, analyze};

/// Erros que a interface mostra ao usuário; `code()` é estável para tradução.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// A pasta indicada não é uma pasta de save.
    #[error("a pasta escolhida não contém arquivos de save do jogo")]
    InvalidSaveDir,
    /// A pasta indicada não é a instalação do jogo.
    #[error("a pasta escolhida não contém os arquivos do jogo (gameN.tab)")]
    InvalidGameDir,
    /// Ainda não há um save selecionado.
    #[error("nenhum save selecionado")]
    NoSaveSelected,
    /// O backup não está na pasta de backups do app.
    #[error("o backup escolhido não está na pasta de backups do app")]
    BackupOutsideAppFolder,
    /// Falha no histórico.
    #[error(transparent)]
    History(#[from] HistoryError),
    /// Falha ao salvar as configurações.
    #[error("não foi possível salvar as configurações: {0}")]
    Settings(#[from] std::io::Error),
    /// Falha na restauração.
    #[error(transparent)]
    Transaction(#[from] TransactionError),
    /// O arquivo não existe no save selecionado ou não é um save reconhecido.
    #[error("esse arquivo não pode ser regravado pelo app")]
    InvalidFile,
    /// Falha ao ler o arquivo do save.
    #[error("não foi possível ler o arquivo: {0}")]
    Read(String),
    /// A instalação do jogo não foi encontrada (precisa dela para ler os mapas).
    #[error("a instalação do jogo não foi encontrada")]
    GameNotFound,
    /// Não foi possível analisar ou montar a liberação das zonas.
    #[error(transparent)]
    Unlock(#[from] UnlockError),
    /// Não foi possível montar o mapa da reserva.
    #[error(transparent)]
    Map(#[from] MapError),
    /// Não foi possível montar o perfil ou as missões.
    #[error(transparent)]
    Career(#[from] CareerError),
    /// Nenhuma das reservas escolhidas tem zonas a liberar.
    #[error("não há zonas a liberar nas reservas escolhidas")]
    NothingToUnlock,
}

impl ServiceError {
    /// Código estável para a interface traduzir.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidSaveDir => "invalidSaveDir",
            Self::InvalidGameDir => "invalidGameDir",
            Self::NoSaveSelected => "noSaveSelected",
            Self::BackupOutsideAppFolder => "backupOutsideAppFolder",
            Self::InvalidFile => "invalidFile",
            Self::Read(_) => "read",
            Self::GameNotFound => "gameNotFound",
            Self::Unlock(_) => "unlock",
            Self::Map(_) => "map",
            Self::Career(_) => "career",
            Self::NothingToUnlock => "nothingToUnlock",
            Self::History(_) => "history",
            Self::Settings(_) => "settings",
            Self::Transaction(TransactionError::Game(crate::process::GameStateError::Running)) => {
                "gameRunning"
            }
            Self::Transaction(TransactionError::Game(_)) => "gameStateUnknown",
            Self::Transaction(TransactionError::RollbackFailed { .. }) => "rollbackFailed",
            Self::Transaction(_) => "transaction",
        }
    }
}

/// Um registro do histórico para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    /// Número do registro.
    pub id: i64,
    /// Data e hora (`2026-11-02_15-30-00`).
    pub at: String,
    /// `change` ou `restore`.
    pub kind: &'static str,
    /// Função do app.
    pub function: String,
    /// Resumo do que mudou.
    pub summary: String,
    /// Arquivos afetados.
    pub files: Vec<String>,
    /// Pasta do backup feito antes.
    pub backup: String,
    /// O backup não existe mais.
    pub backup_missing: bool,
}

/// Resultado de uma restauração.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    /// Backup de segurança do estado anterior à restauração.
    pub safety_backup: String,
}

/// Resultado de uma regravação sem mudanças.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteResult {
    /// Backup do estado anterior.
    pub backup: String,
    /// Tamanho do arquivo antes.
    pub old_size: u64,
    /// Tamanho do arquivo depois (o zlib pode comprimir diferente).
    pub new_size: u64,
}

/// O que a liberação do mapa vai fazer, reserva por reserva.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZonesPreview {
    /// Uma entrada por reserva do save.
    pub reserves: Vec<ReserveSummary>,
    /// Total de zonas novas.
    pub total: usize,
}

/// Resultado da liberação do mapa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockResult {
    /// Zonas acrescentadas.
    pub zones_added: usize,
    /// Reservas que ganharam zonas.
    pub reserves: usize,
    /// Backup do estado anterior.
    pub backup: String,
}

/// Serviço da aplicação.
pub struct AppService {
    appdata: PathBuf,
    roots: SearchRoots,
    processes: Box<dyn ProcessList + Send + Sync>,
}

impl AppService {
    /// Cria o serviço. `appdata` é a pasta `%APPDATA%`; `roots` onde procurar save e jogo.
    #[must_use]
    pub fn new(
        appdata: &Path,
        roots: SearchRoots,
        processes: Box<dyn ProcessList + Send + Sync>,
    ) -> Self {
        let service = Self {
            appdata: appdata.to_path_buf(),
            roots,
            processes,
        };
        service.adopt_legacy_data();
        service
    }

    /// Traz a pasta de dados do nome antigo (backups e histórico) para a nova, sem perder nada.
    ///
    /// Se algo falhar, a pasta antiga continua intacta e o app começa com dados novos.
    fn adopt_legacy_data(&self) {
        if let Ok(Some((from, to))) = migrate_legacy_dir(&self.appdata)
            && let Ok(history) = History::open(&history_db(&self.appdata))
        {
            // Sem este ajuste os backups moveriam, mas o histórico apontaria para a pasta antiga.
            let _ = history.relocate_backups(&from, &to);
        }
    }

    fn settings_path(&self) -> PathBuf {
        app_data_dir(&self.appdata).join("settings.json")
    }

    fn settings(&self) -> Settings {
        Settings::load(&self.settings_path())
    }

    /// Diagnóstico completo do jogo, do save e dos arquivos.
    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        run(&self.roots, &self.settings())
    }

    /// Seleciona a pasta de save de uma conta (escolhida na lista ou indicada à mão).
    ///
    /// # Errors
    ///
    /// [`ServiceError::InvalidSaveDir`] se a pasta não tiver arquivos de save.
    pub fn select_save_dir(&self, path: &Path) -> Result<(), ServiceError> {
        let SaveLocation::Found(account) = resolve_manual(path) else {
            return Err(ServiceError::InvalidSaveDir);
        };
        let mut settings = self.settings();
        settings.save_dir = Some(account.path);
        settings.save(&self.settings_path())?;
        Ok(())
    }

    /// Seleciona a pasta do jogo (a instalação ou a `archives_win64`).
    ///
    /// # Errors
    ///
    /// [`ServiceError::InvalidGameDir`] se a pasta não tiver os `gameN.tab`.
    pub fn select_game_dir(&self, path: &Path) -> Result<(), ServiceError> {
        let install = validate_game_dir(path).ok_or(ServiceError::InvalidGameDir)?;
        let mut settings = self.settings();
        settings.game_dir = Some(install.root);
        settings.save(&self.settings_path())?;
        Ok(())
    }

    /// Esquece as escolhas manuais e volta à busca automática.
    ///
    /// # Errors
    ///
    /// Retorna o erro de E/S se não for possível salvar.
    pub fn reset_choices(&self) -> Result<(), ServiceError> {
        Settings::default().save(&self.settings_path())?;
        Ok(())
    }

    /// Histórico de alterações e restaurações, do mais recente ao mais antigo.
    ///
    /// # Errors
    ///
    /// [`ServiceError::History`] se o banco não puder ser lido.
    pub fn history(&self) -> Result<Vec<HistoryItem>, ServiceError> {
        let history = History::open(&history_db(&self.appdata))?;
        Ok(history
            .list()?
            .into_iter()
            .map(|entry| HistoryItem {
                id: entry.id,
                at: entry.data.at,
                kind: match entry.data.kind {
                    HistoryKind::Change => "change",
                    HistoryKind::Restore => "restore",
                },
                function: entry.data.function,
                summary: entry.data.summary,
                files: entry.data.files,
                backup: entry.data.backup.display().to_string(),
                backup_missing: entry.backup_missing,
            })
            .collect())
    }

    /// Pasta do save selecionado, se houver exatamente uma.
    fn selected_save_dir(&self) -> Result<(PathBuf, Vec<FileDiagnostic>), ServiceError> {
        let report = self.diagnostics();
        match (report.save.status, report.save.account) {
            (SaveStatus::Found, Some(account)) => Ok((PathBuf::from(account.path), report.files)),
            _ => Err(ServiceError::NoSaveSelected),
        }
    }

    /// Pasta do save e arquivos do jogo, necessários para calcular a liberação do mapa.
    fn unlock_context(&self) -> Result<(PathBuf, GameArchives), ServiceError> {
        let report = self.diagnostics();
        let archives = report.game.archives.ok_or(ServiceError::GameNotFound)?;
        let (save_dir, _) = match (report.save.status, report.save.account) {
            (SaveStatus::Found, Some(account)) => (PathBuf::from(account.path), report.files),
            _ => return Err(ServiceError::NoSaveSelected),
        };
        let archives =
            GameArchives::open(Path::new(&archives)).map_err(|_| ServiceError::GameNotFound)?;
        Ok((save_dir, archives))
    }

    /// Wiki dos animais: espécies de cada reserva do jogo e o que o save sabe delas.
    ///
    /// # Errors
    ///
    /// [`ServiceError::GameNotFound`] sem o jogo, [`ServiceError::NoSaveSelected`] sem save e
    /// [`ServiceError::Map`] se o arquivo de zonas não puder ser lido.
    pub fn animals(&self) -> Result<Vec<AnimalReserve>, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        Ok(animals::animals(&save_dir, &archives)?)
    }

    /// Perfil do jogador: nível, dinheiro, tiros, troféus e conquistas da Steam.
    ///
    /// # Errors
    ///
    /// [`ServiceError::NoSaveSelected`] sem save e [`ServiceError::Career`] se o perfil não puder ser lido.
    pub fn profile(&self) -> Result<ProfileView, ServiceError> {
        let report = self.diagnostics();
        let (SaveStatus::Found, Some(account)) = (report.save.status, report.save.account) else {
            return Err(ServiceError::NoSaveSelected);
        };
        let stats_dir = self
            .roots
            .steam
            .iter()
            .map(|root| root.join("appcache").join("stats"))
            .find(|dir| dir.is_dir());
        Ok(career::profile(
            Path::new(&account.path),
            &account.steam_id,
            stats_dir.as_deref(),
        )?)
    }

    /// Progresso das missões de cada reserva, com os textos do jogo.
    ///
    /// # Errors
    ///
    /// [`ServiceError::GameNotFound`] sem o jogo, [`ServiceError::NoSaveSelected`] sem save e
    /// [`ServiceError::Career`] se o progresso não puder ser lido.
    pub fn missions(&self) -> Result<Vec<ReserveMissions>, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        Ok(career::missions(&save_dir, &archives)?)
    }

    /// Reservas que o mapa consegue mostrar (têm população no save e mapas no jogo).
    ///
    /// # Errors
    ///
    /// [`ServiceError::GameNotFound`] sem o jogo, [`ServiceError::NoSaveSelected`] sem save e
    /// [`ServiceError::Map`] se o arquivo de zonas não puder ser lido.
    pub fn map_reserves(&self) -> Result<Vec<MapReserve>, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        Ok(mapview::reserves(&save_dir, &archives)?)
    }

    /// Dados do mapa de uma reserva: zonas, grupos, espécies e imagem de fundo.
    ///
    /// # Errors
    ///
    /// Como [`Self::map_reserves`], mais [`ServiceError::Map`] se a reserva não tiver dados.
    pub fn reserve_map(&self, reserve: u32) -> Result<ReserveMap, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        Ok(mapview::build(&save_dir, &archives, reserve)?)
    }

    /// Calcula, sem gravar nada, as zonas que cada reserva ganharia ao liberar o mapa.
    ///
    /// # Errors
    ///
    /// [`ServiceError::GameNotFound`] sem a instalação do jogo, [`ServiceError::NoSaveSelected`]
    /// sem save e [`ServiceError::Unlock`] se o arquivo de zonas não puder ser lido.
    pub fn zones_preview(&self) -> Result<ZonesPreview, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        let analysis = analyze(&save_dir, &archives)?;
        Ok(ZonesPreview {
            total: analysis.total_new_zones(),
            reserves: analysis.reserves,
        })
    }

    /// Libera o mapa nas reservas escolhidas, pelo caminho seguro completo (jogo fechado,
    /// backup, conferência, histórico).
    ///
    /// # Errors
    ///
    /// [`ServiceError::NothingToUnlock`] se não houver zonas a liberar nas reservas escolhidas;
    /// [`ServiceError::Transaction`] se o jogo estiver aberto ou algo falhar (os arquivos são
    /// restaurados); demais erros como em [`Self::zones_preview`].
    pub fn unlock_zones(&self, reserves: &[u32]) -> Result<UnlockResult, ServiceError> {
        let (save_dir, archives) = self.unlock_context()?;
        let analysis = analyze(&save_dir, &archives)?;
        let unlock = analysis
            .build(reserves)?
            .ok_or(ServiceError::NothingToUnlock)?;
        let (zones_added, reserves_changed) = (unlock.zones_added, unlock.reserves);

        let transaction = SaveTransaction::new(&save_dir, &backups_dir(&self.appdata));
        let preview = transaction.preview(Change {
            function: "Liberar o mapa".into(),
            summary: format!("{zones_added} zonas liberadas em {reserves_changed} reservas"),
            files: vec![unlock.file],
        })?;
        let history = History::open(&history_db(&self.appdata))?;
        let done = transaction.commit(preview.confirm(), self.processes.as_ref(), &history)?;
        Ok(UnlockResult {
            zones_added,
            reserves: reserves_changed,
            backup: done.backup.path.display().to_string(),
        })
    }

    /// Regrava um arquivo do save sem mudar nada, pelo caminho seguro completo (jogo fechado,
    /// backup, conferência, histórico). Serve para confirmar que o jogo aceita arquivos gravados
    /// pelo app antes de qualquer edição real.
    ///
    /// # Errors
    ///
    /// [`ServiceError::InvalidFile`] se o arquivo não for um save reconhecido do save
    /// selecionado; [`ServiceError::Transaction`] se o jogo estiver aberto ou algo falhar (os
    /// arquivos são restaurados).
    pub fn rewrite_unchanged(&self, file: &str) -> Result<RewriteResult, ServiceError> {
        let (save_dir, files) = self.selected_save_dir()?;
        // Só arquivos que o app lê com sucesso, listados na pasta (isso também barra `..\x`).
        let known = files
            .iter()
            .any(|f| f.name == file && f.state == FileState::Ok);
        if !known {
            return Err(ServiceError::InvalidFile);
        }
        let raw = fs::read(save_dir.join(file)).map_err(|e| ServiceError::Read(e.to_string()))?;
        let envelope = SaveEnvelope::parse(&raw).map_err(|_| ServiceError::InvalidFile)?;
        let new_content = envelope.to_bytes().map_err(|_| ServiceError::InvalidFile)?;

        let transaction = SaveTransaction::new(&save_dir, &backups_dir(&self.appdata));
        let preview = transaction.preview(Change {
            function: "Teste de gravação".into(),
            summary: format!("{file} regravado sem mudanças"),
            files: vec![FileChange {
                name: file.to_owned(),
                new_content,
                items: Vec::new(),
                // Relê o arquivo gravado: mesmo conteúdo descomprimido e ADF legível.
                verify: Some(Box::new(move |bytes| {
                    let again = SaveEnvelope::parse(bytes).map_err(|e| e.to_string())?;
                    if again != envelope {
                        return Err("o conteúdo descomprimido mudou".into());
                    }
                    Adf::parse(again.adf().to_vec()).map_err(|e| e.to_string())?;
                    Ok(())
                })),
            }],
        })?;
        let sizes = preview.files[0].clone();
        let history = History::open(&history_db(&self.appdata))?;
        let done = transaction.commit(preview.confirm(), self.processes.as_ref(), &history)?;
        Ok(RewriteResult {
            backup: done.backup.path.display().to_string(),
            old_size: sizes.old_size,
            new_size: sizes.new_size,
        })
    }

    /// Restaura um backup do app no save selecionado (com o jogo fechado).
    ///
    /// # Errors
    ///
    /// Veja [`ServiceError`]: nenhum save selecionado, backup fora da pasta do app, jogo aberto
    /// ou falha na restauração.
    pub fn restore(&self, backup: &Path) -> Result<RestoreResult, ServiceError> {
        let (save_dir, _) = self.selected_save_dir()?;
        let backups_root = backups_dir(&self.appdata);
        let inside = match (fs::canonicalize(backup), fs::canonicalize(&backups_root)) {
            (Ok(backup), Ok(root)) => backup.starts_with(root),
            _ => false,
        };
        if !inside {
            return Err(ServiceError::BackupOutsideAppFolder);
        }
        let history = History::open(&history_db(&self.appdata))?;
        let done = SaveTransaction::new(&save_dir, &backups_root).restore(
            backup,
            self.processes.as_ref(),
            &history,
        )?;
        Ok(RestoreResult {
            safety_backup: done.backup.path.display().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use crate::backup::create_backup;
    use crate::history::NewEntry;

    struct Fixed(Vec<&'static str>);

    impl ProcessList for Fixed {
        fn running_image_names(&self) -> io::Result<Vec<String>> {
            Ok(self.0.iter().map(|s| (*s).to_owned()).collect())
        }
    }

    struct Env {
        _root: tempfile::TempDir,
        appdata: PathBuf,
        docs: PathBuf,
        save: PathBuf,
    }

    fn env() -> Env {
        let root = tempfile::tempdir().unwrap();
        let appdata = root.path().join("appdata");
        let docs = root.path().join("docs");
        let save = docs.join("Avalanche Studios/COTW/Saves/111");
        fs::create_dir_all(&save).unwrap();
        fs::write(save.join("settings_adf"), b"original").unwrap();
        Env {
            appdata,
            docs,
            save,
            _root: root,
        }
    }

    fn service(env: &Env, processes: Vec<&'static str>) -> AppService {
        AppService::new(
            &env.appdata,
            SearchRoots {
                documents: vec![env.docs.clone()],
                steam: Vec::new(),
            },
            Box::new(Fixed(processes)),
        )
    }

    #[test]
    fn diagnostics_reflect_the_search_and_the_missing_game() {
        let env = env();
        let report = service(&env, vec![]).diagnostics();
        assert_eq!(report.save.status, SaveStatus::Found);
        assert!(report.limited_mode);
    }

    #[test]
    fn selecting_a_save_folder_is_remembered_and_validated() {
        let env = env();
        let service = service(&env, vec![]);
        let other = env.docs.join("Avalanche Studios/COTW/Saves/222");
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("settings_adf"), b"x").unwrap();

        service.select_save_dir(&other).unwrap();
        let report = service.diagnostics();
        assert_eq!(report.save.account.unwrap().steam_id, "222");
        assert!(report.save.manual);

        let err = service
            .select_save_dir(&env.docs.join("vazia"))
            .unwrap_err();
        assert_eq!(err.code(), "invalidSaveDir");
        service.reset_choices().unwrap();
        assert!(!service.diagnostics().save.manual || service.diagnostics().save.account.is_some());
    }

    #[test]
    fn selecting_a_non_game_folder_is_rejected() {
        let env = env();
        let err = service(&env, vec![])
            .select_game_dir(&env.docs)
            .unwrap_err();
        assert_eq!(err.code(), "invalidGameDir");
    }

    #[test]
    fn history_is_empty_at_first_and_lists_recorded_entries() {
        let env = env();
        let service = service(&env, vec![]);
        assert_eq!(service.history().unwrap(), []);

        let history = History::open(&history_db(&env.appdata)).unwrap();
        history
            .record(&NewEntry {
                at: "2026-11-02_10-00-00".into(),
                kind: HistoryKind::Change,
                function: "Liberar zonas".into(),
                summary: "+3 zonas".into(),
                files: vec!["found_need_zones_adf".into()],
                backup: env.appdata.join("sumiu"),
            })
            .unwrap();
        let items = service.history().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "change");
        assert!(items[0].backup_missing);
        assert_eq!(items[0].files, ["found_need_zones_adf"]);
    }

    #[test]
    fn restore_brings_back_the_backup_and_is_logged() {
        let env = env();
        let service = service(&env, vec!["explorer.exe"]);
        let backup = create_backup(&env.save, &backups_dir(&env.appdata), "antes").unwrap();
        fs::write(env.save.join("settings_adf"), b"alterado").unwrap();

        let result = service.restore(&backup.path).unwrap();
        assert_eq!(
            fs::read(env.save.join("settings_adf")).unwrap(),
            b"original"
        );
        assert_eq!(
            fs::read(Path::new(&result.safety_backup).join("settings_adf")).unwrap(),
            b"alterado"
        );
        let items = service.history().unwrap();
        assert_eq!(items[0].kind, "restore");
    }

    #[test]
    fn restore_is_blocked_while_the_game_runs() {
        let env = env();
        let backup = create_backup(&env.save, &backups_dir(&env.appdata), "antes").unwrap();
        fs::write(env.save.join("settings_adf"), b"alterado").unwrap();
        let err = service(&env, vec!["theHunterCotW_F.exe"])
            .restore(&backup.path)
            .unwrap_err();
        assert_eq!(err.code(), "gameRunning");
        assert_eq!(
            fs::read(env.save.join("settings_adf")).unwrap(),
            b"alterado"
        );
    }

    #[test]
    fn restore_only_accepts_backups_inside_the_app_folder() {
        let env = env();
        let outside = env.docs.join("backup-de-fora");
        fs::create_dir_all(&outside).unwrap();
        let err = service(&env, vec![]).restore(&outside).unwrap_err();
        assert_eq!(err.code(), "backupOutsideAppFolder");
        // Tentativa de escapar com `..` também falha.
        fs::create_dir_all(backups_dir(&env.appdata)).unwrap();
        let sneaky = backups_dir(&env.appdata).join("..").join("..").join("docs");
        assert_eq!(
            service(&env, vec![]).restore(&sneaky).unwrap_err().code(),
            "backupOutsideAppFolder"
        );
    }

    #[test]
    fn restore_without_a_selected_save_fails_cleanly() {
        let env = env();
        fs::remove_dir_all(&env.docs).unwrap();
        let err = service(&env, vec![]).restore(&env.appdata).unwrap_err();
        assert_eq!(err.code(), "noSaveSelected");
    }

    fn env_with_valid_save() -> Env {
        let env = env();
        fs::write(
            env.save.join("animal_population_2"),
            crate::test_util::valid_save_bytes(),
        )
        .unwrap();
        env
    }

    #[test]
    fn rewrite_unchanged_backs_up_writes_and_logs() {
        let env = env_with_valid_save();
        let before = fs::read(env.save.join("animal_population_2")).unwrap();
        let result = service(&env, vec!["explorer.exe"])
            .rewrite_unchanged("animal_population_2")
            .unwrap();

        // O backup guarda o arquivo como estava; o conteúdo descomprimido não mudou.
        let backed_up = fs::read(Path::new(&result.backup).join("animal_population_2")).unwrap();
        assert_eq!(backed_up, before);
        let after = fs::read(env.save.join("animal_population_2")).unwrap();
        assert_eq!(
            SaveEnvelope::parse(&after).unwrap(),
            SaveEnvelope::parse(&before).unwrap()
        );
        assert_eq!(result.new_size, after.len() as u64);
        assert_eq!(result.old_size, before.len() as u64);
        let items = service(&env, vec![]).history().unwrap();
        assert_eq!(items[0].files, ["animal_population_2"]);
        assert_eq!(items[0].function, "Teste de gravação");
    }

    #[test]
    fn rewrite_unchanged_is_blocked_while_the_game_runs() {
        let env = env_with_valid_save();
        let before = fs::read(env.save.join("animal_population_2")).unwrap();
        let err = service(&env, vec!["theHunterCotW_F.exe"])
            .rewrite_unchanged("animal_population_2")
            .unwrap_err();
        assert_eq!(err.code(), "gameRunning");
        assert_eq!(
            fs::read(env.save.join("animal_population_2")).unwrap(),
            before
        );
        assert!(!backups_dir(&env.appdata).exists());
    }

    #[test]
    fn rewrite_unchanged_only_accepts_recognized_readable_files() {
        let env = env_with_valid_save();
        fs::write(env.save.join("animal_population_3"), b"corrompido").unwrap();
        fs::write(env.save.join("notas.txt"), b"x").unwrap();
        let service = service(&env, vec![]);
        for name in [
            "animal_population_3",
            "notas.txt",
            "nao_existe",
            "..\\x",
            "../x",
            "",
        ] {
            assert_eq!(
                service.rewrite_unchanged(name).unwrap_err().code(),
                "invalidFile",
                "{name:?}"
            );
        }
        assert!(!backups_dir(&env.appdata).exists(), "nada foi tocado");
    }

    #[test]
    fn rewrite_unchanged_needs_a_selected_save() {
        let env = env();
        fs::remove_dir_all(&env.docs).unwrap();
        let err = service(&env, vec![]).rewrite_unchanged("x").unwrap_err();
        assert_eq!(err.code(), "noSaveSelected");
    }

    #[test]
    fn items_serialize_with_camel_case_names() {
        let item = HistoryItem {
            id: 1,
            at: "t".into(),
            kind: "restore",
            function: "f".into(),
            summary: "s".into(),
            files: vec![],
            backup: "b".into(),
            backup_missing: true,
        };
        let json = serde_json::to_value(item).unwrap();
        assert_eq!(json["backupMissing"], true);
    }

    #[test]
    fn adopts_the_data_folder_of_the_old_name_keeping_the_history_valid() {
        let env = env();
        let legacy = env.appdata.join("COTW Companion");
        let old_backup = create_backup(&env.save, &legacy.join("backups"), "antes").unwrap();
        History::open(&legacy.join("history.db"))
            .unwrap()
            .record(&NewEntry {
                at: "2026-11-02_10-00-00".into(),
                kind: HistoryKind::Change,
                function: "Teste".into(),
                summary: "sem mudanças".into(),
                files: vec![],
                backup: old_backup.path,
            })
            .unwrap();

        let items = service(&env, vec![]).history().unwrap();

        assert!(!legacy.exists());
        assert_eq!(items.len(), 1);
        assert!(!items[0].backup_missing);
        assert!(Path::new(&items[0].backup).starts_with(app_data_dir(&env.appdata)));
    }
}
