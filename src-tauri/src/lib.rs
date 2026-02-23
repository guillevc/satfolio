use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use app_core::context::Context;
use app_core::errors::CoreError;
use app_core::models::{Asset, BepSnapshot, Candle, PositionSummary, Trade, TradesSummary};
use chrono::NaiveDate;
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
    ctx: Mutex<Context>,
    prices_dir: PathBuf,
}

// -- Commands ------------------------------------------------------------

#[tauri::command]
fn preview_import(state: State<AppState>, path: PathBuf) -> Result<TradesSummary, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::preview_import(ctx.quote(), &path)?)
}

#[tauri::command]
fn confirm_import(state: State<AppState>, path: PathBuf) -> Result<TradesSummary, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::confirm_import(&ctx, &path)?)
}

#[tauri::command]
fn position_summary(state: State<AppState>) -> Result<PositionSummary, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::position_summary(&ctx)?)
}

#[tauri::command]
fn bep_snaps(state: State<AppState>) -> Result<BTreeMap<NaiveDate, BepSnapshot>, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::bep_snaps(&ctx)?)
}

#[tauri::command]
fn trades(state: State<AppState>) -> Result<Vec<Trade>, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::trades(&ctx)?)
}

#[tauri::command]
fn candles(state: State<AppState>) -> Result<Vec<Candle>, AppError> {
    let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
    Ok(app_core::api::candles(&ctx, &state.prices_dir)?)
}

#[tauri::command]
fn load_sample(state: State<AppState>) -> Result<(), AppError> {
    if cfg!(debug_assertions) {
        let ctx = state.ctx.lock().unwrap_or_else(|e| e.into_inner());
        let trades = app_core::api::trades(&ctx)?;
        if trades.is_empty() {
            let fixture = std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../crates/core/fixtures/sample.csv"
            ));
            app_core::api::confirm_import(&ctx, &fixture)?;
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

            let ctx =
                Context::open(&data_dir.join("betc.db"), Asset::Eur).map_err(|e| e.to_string())?;

            let prices_dir = app
                .path()
                .resolve("resources/prices", tauri::path::BaseDirectory::Resource)?;

            app.manage(AppState {
                ctx: Mutex::new(ctx),
                prices_dir,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            preview_import,
            confirm_import,
            position_summary,
            bep_snaps,
            trades,
            candles,
            load_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
