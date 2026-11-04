// commands/storage.rs
// Tauri Commands untuk Manajemen Penyimpanan (PRD Fase 12)

use crate::core::storage;
use crate::models::{ProjectStorageInfo, SystemStorageInfo};
use crate::storage::db::Database;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn storage_get_system_info(app: AppHandle) -> Result<SystemStorageInfo, String> {
    storage::get_system_storage_info(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn storage_get_project_info(
    app: AppHandle,
    db: State<'_, Database>,
    project_id: i64,
) -> Result<ProjectStorageInfo, String> {
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;
    storage::get_project_storage_info(&app, &project).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn storage_clean_project(
    app: AppHandle,
    db: State<'_, Database>,
    project_id: i64,
    target: String,
) -> Result<u64, String> {
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    match target.as_str() {
        "node_modules" => storage::clean_project_node_modules(&app, &project).map_err(|e| e.to_string()),
        "logs" => storage::clean_project_logs(&app, &project).map_err(|e| e.to_string()),
        "all" => {
            let mut freed = storage::clean_project_node_modules(&app, &project).map_err(|e| e.to_string())?;
            freed += storage::clean_project_logs(&app, &project).map_err(|e| e.to_string())?;
            Ok(freed)
        }
        _ => Err(format!("Target pembersihan '{target}' tidak dikenal")),
    }
}

#[tauri::command]
pub async fn storage_check_preflight(app: AppHandle, required_mb: u64) -> Result<bool, String> {
    match storage::check_free_disk_space(&app, required_mb) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

