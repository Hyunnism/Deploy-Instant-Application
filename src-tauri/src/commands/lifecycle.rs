// commands/lifecycle.rs
// Command Tauri untuk Lifecycle, Battery Optimization, & Cleanup (PRD Fase 11)

use crate::core::lifecycle::{
    clean_orphan_processes, is_ignoring_battery_optimizations_native, open_battery_settings_native,
    shutdown_all_cleanly,
};
use crate::error::{AppError, Result};
use tauri::{AppHandle, Manager, command};

#[command]
pub async fn get_battery_status() -> Result<bool> {
    Ok(is_ignoring_battery_optimizations_native())
}

#[command]
pub async fn open_battery_settings() -> Result<()> {
    open_battery_settings_native();
    Ok(())
}

#[command]
pub async fn clean_orphans(app: AppHandle) -> Result<usize> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal app_data_dir: {e}")))?;
    Ok(clean_orphan_processes(&app_files_dir).await)
}

#[command]
pub async fn shutdown_cleanly(app: AppHandle) -> Result<()> {
    shutdown_all_cleanly(&app).await
}
