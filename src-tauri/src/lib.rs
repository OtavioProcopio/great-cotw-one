//! Casca do app: expõe os serviços do `cotw-core` à interface como comandos do Tauri.
//!
//! Aqui não há regra de negócio: cada comando repassa a chamada ao [`AppService`] (testado em
//! `cotw-core`) numa thread de trabalho, para que o acesso a disco não trave a janela.

mod http_store;

use std::path::PathBuf;
use std::sync::Arc;

use cotw_core::animal_editor::SpeciesEditor;
use cotw_core::animals::AnimalReserve;
use cotw_core::career::{ProfileView, ReserveMissions};
use cotw_core::diagnostics::{Diagnostics, SearchRoots};
use cotw_core::equipment::Inventory;
use cotw_core::license::LicenseStatus;
use cotw_core::mapview::{MapReserve, ReserveMap};
use cotw_core::paths::system_appdata;
use cotw_core::process::SystemProcessList;
use cotw_core::purchase::{LICENSE_SERVER, PurchaseStatus};
use cotw_core::service::{
    AppService, EditRequest, EditResult, HistoryItem, RestoreResult, RewriteResult, ServiceError,
    UnlockRequest, UnlockResult, ZonesPreview,
};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

/// Erro enviado à interface: `code` é estável (a interface traduz); `message` é o detalhe técnico.
#[derive(Debug, Serialize)]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl From<ServiceError> for CommandError {
    fn from(error: ServiceError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
        }
    }
}

type Service<'a> = State<'a, Arc<AppService>>;

/// Executa `work` numa thread de trabalho com o serviço.
async fn blocking<T, F>(service: &Service<'_>, work: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce(&AppService) -> Result<T, ServiceError> + Send + 'static,
{
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || work(&service))
        .await
        .map_err(|e| CommandError {
            code: "internal",
            message: e.to_string(),
        })?
        .map_err(CommandError::from)
}

#[tauri::command]
async fn diagnostics(service: Service<'_>) -> Result<Diagnostics, CommandError> {
    blocking(&service, |s| Ok(s.diagnostics())).await
}

#[tauri::command]
async fn select_save_dir(service: Service<'_>, path: String) -> Result<(), CommandError> {
    blocking(&service, move |s| s.select_save_dir(&PathBuf::from(path))).await
}

#[tauri::command]
async fn select_game_dir(service: Service<'_>, path: String) -> Result<(), CommandError> {
    blocking(&service, move |s| s.select_game_dir(&PathBuf::from(path))).await
}

#[tauri::command]
async fn reset_choices(service: Service<'_>) -> Result<(), CommandError> {
    blocking(&service, AppService::reset_choices).await
}

#[tauri::command]
async fn history(service: Service<'_>) -> Result<Vec<HistoryItem>, CommandError> {
    blocking(&service, AppService::history).await
}

#[tauri::command]
async fn rewrite_unchanged(
    service: Service<'_>,
    file: String,
) -> Result<RewriteResult, CommandError> {
    blocking(&service, move |s| s.rewrite_unchanged(&file)).await
}

#[tauri::command]
async fn map_reserves(service: Service<'_>) -> Result<Vec<MapReserve>, CommandError> {
    blocking(&service, AppService::map_reserves).await
}

#[tauri::command]
async fn animals(service: Service<'_>) -> Result<Vec<AnimalReserve>, CommandError> {
    blocking(&service, AppService::animals).await
}

#[tauri::command]
async fn species_editor(
    service: Service<'_>,
    reserve: u32,
    species: String,
) -> Result<SpeciesEditor, CommandError> {
    blocking(&service, move |s| s.species_editor(reserve, &species)).await
}

#[tauri::command]
async fn edit_animals(
    service: Service<'_>,
    request: EditRequest,
) -> Result<EditResult, CommandError> {
    blocking(&service, move |s| s.edit_animals(&request)).await
}

#[tauri::command]
async fn profile(service: Service<'_>) -> Result<ProfileView, CommandError> {
    blocking(&service, AppService::profile).await
}

#[tauri::command]
async fn equipment(service: Service<'_>) -> Result<Inventory, CommandError> {
    blocking(&service, AppService::equipment).await
}

#[tauri::command]
async fn equipment_icons(
    service: Service<'_>,
    ids: Vec<String>,
) -> Result<std::collections::BTreeMap<String, String>, CommandError> {
    blocking(&service, move |s| Ok(s.equipment_icons(&ids))).await
}

#[tauri::command]
async fn missions(service: Service<'_>) -> Result<Vec<ReserveMissions>, CommandError> {
    blocking(&service, AppService::missions).await
}

#[tauri::command]
async fn reserve_map(service: Service<'_>, reserve: u32) -> Result<ReserveMap, CommandError> {
    blocking(&service, move |s| s.reserve_map(reserve)).await
}

#[tauri::command]
async fn zones_preview(service: Service<'_>) -> Result<ZonesPreview, CommandError> {
    blocking(&service, AppService::zones_preview).await
}

#[tauri::command]
async fn unlock_zones(
    service: Service<'_>,
    request: UnlockRequest,
) -> Result<UnlockResult, CommandError> {
    blocking(&service, move |s| s.unlock_zones(&request)).await
}

#[tauri::command]
async fn set_language(service: Service<'_>, tag: String) -> Result<(), CommandError> {
    blocking(&service, move |s| {
        s.set_language(&tag);
        Ok(())
    })
    .await
}

#[tauri::command]
async fn license(service: Service<'_>) -> Result<LicenseStatus, CommandError> {
    blocking(&service, |s| Ok(s.license_status())).await
}

#[tauri::command]
async fn activate_license(
    service: Service<'_>,
    key: String,
) -> Result<LicenseStatus, CommandError> {
    blocking(&service, move |s| s.activate_license(&key)).await
}

#[tauri::command]
async fn remove_license(service: Service<'_>) -> Result<LicenseStatus, CommandError> {
    blocking(&service, AppService::remove_license).await
}

/// Começa a compra: pede o link de pagamento ao servidor e o abre no navegador do jogador.
#[tauri::command]
async fn start_purchase(
    app: AppHandle,
    service: Service<'_>,
    plan: String,
    token: String,
) -> Result<(), CommandError> {
    let url = blocking(&service, move |s| s.start_purchase(&plan, &token)).await?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| CommandError {
            code: "openBrowser",
            message: e.to_string(),
        })
}

/// Confere se o pagamento da compra em andamento foi confirmado e, se foi, liga o Premium.
#[tauri::command]
async fn check_purchase(service: Service<'_>) -> Result<PurchaseStatus, CommandError> {
    blocking(&service, AppService::check_purchase).await
}

/// Desiste da compra em andamento.
#[tauri::command]
async fn cancel_purchase(service: Service<'_>) -> Result<(), CommandError> {
    blocking(&service, |s| {
        s.cancel_purchase();
        Ok(())
    })
    .await
}

/// Endereço do servidor de licenças: o fixo do app; em compilação de depuração, a variável
/// `GREAT_COTW_LICENSE_SERVER` permite apontar para um servidor de teste.
fn license_server() -> String {
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("GREAT_COTW_LICENSE_SERVER") {
        return url;
    }
    LICENSE_SERVER.to_owned()
}

#[tauri::command]
async fn restore_backup(
    service: Service<'_>,
    backup: String,
) -> Result<RestoreResult, CommandError> {
    blocking(&service, move |s| s.restore(&PathBuf::from(backup))).await
}

/// Inicia o app.
///
/// # Panics
///
/// Encerra com mensagem de erro se o Tauri não conseguir iniciar a janela.
#[allow(
    clippy::expect_used,
    reason = "falha ao iniciar a janela não tem recuperação"
)]
pub fn run() {
    let appdata = system_appdata().unwrap_or_else(std::env::temp_dir);
    let mut service = AppService::new(
        &appdata,
        SearchRoots::from_system(),
        Box::new(SystemProcessList),
    );
    let server = license_server();
    if !server.is_empty() {
        service = service.with_store(Box::new(http_store::HttpStore::new(&server)));
    }
    let service = Arc::new(service);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(service)
        .invoke_handler(tauri::generate_handler![
            diagnostics,
            select_save_dir,
            select_game_dir,
            reset_choices,
            history,
            rewrite_unchanged,
            map_reserves,
            reserve_map,
            animals,
            species_editor,
            edit_animals,
            profile,
            equipment,
            equipment_icons,
            missions,
            zones_preview,
            unlock_zones,
            set_language,
            license,
            activate_license,
            remove_license,
            start_purchase,
            check_purchase,
            cancel_purchase,
            restore_backup
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Great COTW One");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_errors_keep_their_stable_code_and_message() {
        let error = CommandError::from(ServiceError::InvalidSaveDir);
        assert_eq!(error.code, "invalidSaveDir");
        assert!(error.message.contains("save"));
    }

    #[test]
    fn command_errors_serialize_for_the_frontend() {
        let json = serde_json::to_value(CommandError::from(ServiceError::NoSaveSelected)).unwrap();
        assert_eq!(json["code"], "noSaveSelected");
        assert!(json["message"].is_string());
    }
}
