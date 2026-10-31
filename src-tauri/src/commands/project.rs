// commands/project.rs
// Thin IPC layer untuk operasi Project, Git, dan Settings (Fase 3 & 4)

use crate::core::project as core_project;
use crate::models::{
    BuildResult, CreateProjectInput, PatStatus, Project, PullResult, RepoStatus, SettingItem,
    UpdateProjectInput,
};
use crate::storage::db::Database;
use crate::storage::secrets;
use tauri::{Manager, State};

#[tauri::command]
pub fn project_create(
    db: State<'_, Database>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    db.create_project(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_list(db: State<'_, Database>) -> Result<Vec<Project>, String> {
    db.list_projects().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_get(db: State<'_, Database>, id: i64) -> Result<Project, String> {
    db.get_project(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_update(
    db: State<'_, Database>,
    id: i64,
    input: UpdateProjectInput,
) -> Result<Project, String> {
    db.update_project(id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_delete(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> Result<(), String> {
    // Ambil project terlebih dahulu untuk mengetahui namanya
    let proj = db.get_project(id).map_err(|e| e.to_string())?;

    // Bersihkan direktori /app/<name> di rootfs jika ada
    if let Ok(app_files_dir) = app.path().app_data_dir() {
        let proj_dir = app_files_dir.join("rootfs").join("app").join(&proj.name);
        if proj_dir.exists() {
            let _ = std::fs::remove_dir_all(proj_dir);
        }
    }

    db.delete_project(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_repo_status(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> Result<RepoStatus, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;
    let rootfs_dir = app_files_dir.join("rootfs");

    Ok(core_project::get_project_repo_status(&rootfs_dir, &app_files_dir, &proj.name).await)
}

#[tauri::command]
pub async fn project_clone(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> Result<RepoStatus, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;
    let rootfs_dir = app_files_dir.join("rootfs");
    let key_dir = app_files_dir.join("db");

    let pat = secrets::get_pat(&db, &key_dir).map_err(|e| e.to_string())?;

    core_project::git_clone(&app, &rootfs_dir, &app_files_dir, &proj, pat.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_pull(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> Result<PullResult, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;
    let rootfs_dir = app_files_dir.join("rootfs");
    let key_dir = app_files_dir.join("db");

    let pat = secrets::get_pat(&db, &key_dir).map_err(|e| e.to_string())?;

    core_project::git_pull(&app, &rootfs_dir, &app_files_dir, &proj, pat.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_save_pat(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    pat: String,
) -> Result<(), String> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal app data dir: {e}"))?;
    let key_dir = app_files_dir.join("db");

    secrets::save_pat(&db, &key_dir, &pat).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_get_pat_status(db: State<'_, Database>) -> Result<PatStatus, String> {
    let has_pat = secrets::has_pat(&db).map_err(|e| e.to_string())?;
    Ok(PatStatus { has_pat })
}

#[tauri::command]
pub fn settings_delete_pat(db: State<'_, Database>) -> Result<(), String> {
    secrets::delete_pat(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_get(db: State<'_, Database>, key: String) -> Result<Option<String>, String> {
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_set(db: State<'_, Database>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_list(db: State<'_, Database>) -> Result<Vec<SettingItem>, String> {
    db.list_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_build(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    build_manager: State<'_, core_project::ActiveBuildManager>,
    id: i64,
) -> Result<BuildResult, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;
    let rootfs_dir = app_files_dir.join("rootfs");

    let result =
        core_project::build_project(&app, &rootfs_dir, &app_files_dir, &proj, &build_manager)
            .await
            .map_err(|e| e.to_string())?;

    let _ = db.update_build_status(id, result.success);

    Ok(result)
}

#[tauri::command]
pub fn project_build_cancel(
    build_manager: State<'_, core_project::ActiveBuildManager>,
    id: i64,
) -> Result<bool, String> {
    Ok(build_manager.cancel(id))
}

#[tauri::command]
pub fn project_build_logs_list(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
) -> Result<Vec<String>, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal app data dir: {e}"))?;
    Ok(core_project::list_build_logs(&app_files_dir, &proj.name))
}

#[tauri::command]
pub fn project_build_log_read(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    id: i64,
    filename: String,
) -> Result<String, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal app data dir: {e}"))?;
    core_project::read_build_log(&app_files_dir, &proj.name, &filename).map_err(|e| e.to_string())
}
