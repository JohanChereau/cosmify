use std::sync::{Arc, Mutex};

use cosmify_core::{
    ActivityEntry, AppSettings, BackupInfo, CosmeticPack, Cosmify, CustomPackAnalysis,
    EncryptPackRequest, EncryptPackResult, HostPack, ImportCosmeticPackRequest, ImportPreview,
    ImportPreviewRequest, InstallRequest, MinecraftStatus, OperationResult, RemovePackRequest,
    RestoreBackupRequest, UpdateCosmeticPackRequest,
};
use tauri::{Emitter, State};

struct AppState {
    core: Arc<Cosmify>,
    mutation_lock: Arc<Mutex<()>>,
}

fn core_error(error: cosmify_core::CosmifyError) -> String {
    error.to_string()
}

#[tauri::command]
async fn get_system_status(state: State<'_, AppState>) -> Result<MinecraftStatus, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.system_status().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn list_host_packs(state: State<'_, AppState>) -> Result<Vec<HostPack>, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.list_host_packs().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn analyze_custom_pack(
    path: String,
    state: State<'_, AppState>,
) -> Result<CustomPackAnalysis, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.analyze_custom_pack(path).map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn list_cosmetic_packs(state: State<'_, AppState>) -> Result<Vec<CosmeticPack>, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.list_cosmetic_packs().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn import_cosmetic_pack(
    request: ImportCosmeticPackRequest,
    state: State<'_, AppState>,
) -> Result<CosmeticPack, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.import_cosmetic_pack(request).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn update_cosmetic_pack(
    request: UpdateCosmeticPackRequest,
    state: State<'_, AppState>,
) -> Result<CosmeticPack, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.update_cosmetic_pack(request).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn set_cosmetic_pack_icon(
    id: String,
    source_path: String,
    state: State<'_, AppState>,
) -> Result<CosmeticPack, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.set_cosmetic_pack_icon(&id, std::path::Path::new(&source_path))
            .map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn delete_cosmetic_pack(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.delete_cosmetic_pack(&id).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn preview_import(
    request: ImportPreviewRequest,
    state: State<'_, AppState>,
) -> Result<ImportPreview, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.preview_import(request).map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn install_pack(
    request: InstallRequest,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<OperationResult, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.install(request, |progress| {
            let _ = app.emit("cosmify://progress", progress);
        })
        .map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn encrypt_pack_directory(
    request: EncryptPackRequest,
    state: State<'_, AppState>,
) -> Result<EncryptPackResult, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.encrypt_pack_directory(request).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.list_backups().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn restore_backup(
    request: RestoreBackupRequest,
    state: State<'_, AppState>,
) -> Result<OperationResult, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.restore_backup(request).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn delete_backup(backup_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.delete_backup(&backup_id).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn remove_pack(
    request: RemovePackRequest,
    state: State<'_, AppState>,
) -> Result<OperationResult, String> {
    let core = state.core.clone();
    let lock = state.mutation_lock.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| "The operation lock is poisoned".to_string())?;
        core.remove_pack(request).map_err(core_error)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn get_activity(state: State<'_, AppState>) -> Result<Vec<ActivityEntry>, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.activity().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.settings().map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || core.save_settings(settings).map_err(core_error))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn get_app_data_path(state: State<'_, AppState>) -> String {
    state.core.data_root().to_string_lossy().into_owned()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let core = Cosmify::new().expect("failed to initialize Cosmify core");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            core: Arc::new(core),
            mutation_lock: Arc::new(Mutex::new(())),
        })
        .invoke_handler(tauri::generate_handler![
            get_system_status,
            list_host_packs,
            analyze_custom_pack,
            list_cosmetic_packs,
            import_cosmetic_pack,
            update_cosmetic_pack,
            set_cosmetic_pack_icon,
            delete_cosmetic_pack,
            preview_import,
            install_pack,
            encrypt_pack_directory,
            list_backups,
            restore_backup,
            delete_backup,
            remove_pack,
            get_activity,
            get_settings,
            save_settings,
            get_app_data_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cosmify");
}
