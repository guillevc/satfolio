use std::path::PathBuf;

use app_core::errors::CoreError;
use app_core::models::{AppConfig, Asset, DashboardStats, EnrichedTrade, TradesSummary};
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
        };
        Self {
            kind,
            message: e.to_string(),
        }
    }
}

// -- State ---------------------------------------------------------------

struct AppState {
    cfg: AppConfig,
}

// -- Commands ------------------------------------------------------------

#[tauri::command]
#[allow(clippy::unused_async)]
async fn preview_import(
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<TradesSummary, AppError> {
    Ok(app_core::api::preview_import(&state.cfg.quote, &path)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn confirm_import(
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<TradesSummary, AppError> {
    Ok(app_core::api::confirm_import(&state.cfg, &path)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, AppError> {
    Ok(app_core::api::dashboard_stats(&state.cfg)?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn trades(state: State<'_, AppState>) -> Result<Vec<EnrichedTrade>, AppError> {
    Ok(app_core::api::trades(&state.cfg)?)
}

#[tauri::command]
async fn sync_candles(state: State<'_, AppState>) -> Result<(), AppError> {
    Ok(app_core::api::sync_candles(&state.cfg).await?)
}

#[tauri::command]
#[allow(clippy::unused_async)]
async fn load_sample(state: State<'_, AppState>) -> Result<(), AppError> {
    if cfg!(debug_assertions) {
        let trades = app_core::api::trades(&state.cfg)?;
        if trades.is_empty() {
            let fixture = std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../crates/core/fixtures/sample.csv"
            ));
            app_core::api::confirm_import(&state.cfg, &fixture)?;
        }
    }
    Ok(())
}

// -- Entrypoint ----------------------------------------------------------

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let cfg = AppConfig {
                db_path: data_dir.join("betc.db"),
                quote: Asset::Eur,
            };

            let prices_dir = app
                .path()
                .resolve("resources/prices", tauri::path::BaseDirectory::Resource)?;

            app_core::api::init_db(&cfg, &prices_dir).map_err(|e| e.to_string())?;

            app.manage(AppState { cfg });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            preview_import,
            confirm_import,
            dashboard_stats,
            trades,
            sync_candles,
            load_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
