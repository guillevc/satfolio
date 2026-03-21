use std::path::PathBuf;

use app_core::errors::CoreError;
use app_core::models::{
    Asset, DashboardStats, EnrichedTrade, ImportOutcome, ImportPreview, ImportRecord,
};
use serde::Serialize;
use tauri::{Manager, State};

// -- Error ---------------------------------------------------------------

#[derive(Debug, Serialize)]
struct AppError {
    kind: &'static str,
    message: String,
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        let kind = match &e {
            CoreError::Parse(_) => "parse",
            CoreError::Db(_) => "db",
            CoreError::Price(_) => "price",
            CoreError::Engine(_) => "engine",
            CoreError::DuplicateFile | CoreError::AllTradesDuplicate(_) => "import",
        };
        Self {
            kind,
            message: e.to_string(),
        }
    }
}

// -- State ---------------------------------------------------------------

struct AppState {
    db_path: PathBuf,
}

// -- Commands ------------------------------------------------------------

#[tauri::command]
#[allow(clippy::unused_async)]
async fn preview_import(
    state: State<'_, AppState>,
    quote: Asset,
    path: PathBuf,
) -> Result<ImportPreview, AppError> {
    Ok(app_core::api::preview_import(
        &state.db_path,
        &quote,
        &path,
    )?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn confirm_import(
    state: State<'_, AppState>,
    quote: Asset,
    path: PathBuf,
) -> Result<ImportOutcome, AppError> {
    Ok(app_core::api::confirm_import(
        &state.db_path,
        &quote,
        &path,
    )?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn list_imports(state: State<'_, AppState>) -> Result<Vec<ImportRecord>, AppError> {
    Ok(app_core::api::list_imports(&state.db_path)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn remove_import(state: State<'_, AppState>, import_id: i64) -> Result<(), AppError> {
    Ok(app_core::api::remove_import(&state.db_path, import_id)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn dashboard_stats(
    state: State<'_, AppState>,
    quote: Asset,
) -> Result<DashboardStats, AppError> {
    Ok(app_core::api::dashboard_stats(&state.db_path, &quote)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn trades(state: State<'_, AppState>, quote: Asset) -> Result<Vec<EnrichedTrade>, AppError> {
    Ok(app_core::api::trades(&state.db_path, &quote)?)
}

#[tauri::command]
async fn sync_candles(state: State<'_, AppState>) -> Result<(), AppError> {
    Ok(app_core::api::sync_all_candles(&state.db_path).await?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn nuke_all_data(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    app_core::api::nuke_all_data(&state.db_path)?;
    app.restart()
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn load_sample(state: State<'_, AppState>) -> Result<(), AppError> {
    if cfg!(debug_assertions) {
        let trades = app_core::api::trades(&state.db_path, &Asset::Eur)?;
        if trades.is_empty() {
            let fixture = std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../crates/core/fixtures/kraken_sample.csv"
            ));
            let _ = app_core::api::confirm_import(&state.db_path, &Asset::Eur, &fixture)?;
        }
    }
    Ok(())
}

// -- Entrypoint ----------------------------------------------------------

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let log_level = if cfg!(debug_assertions) {
                log::LevelFilter::Info
            } else {
                log::LevelFilter::Warn
            };
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log_level)
                    .build(),
            )?;

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("satfolio.db");

            let prices_dir = app
                .path()
                .resolve("resources/prices", tauri::path::BaseDirectory::Resource)?;

            app_core::api::init_db(&db_path, &prices_dir).map_err(|e| e.to_string())?;

            app.manage(AppState { db_path });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            preview_import,
            confirm_import,
            list_imports,
            remove_import,
            nuke_all_data,
            dashboard_stats,
            trades,
            sync_candles,
            load_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
