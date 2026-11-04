// commands/project.rs
// Thin IPC layer untuk operasi Project, Git, dan Settings (Fase 3 & 4)

use crate::core::env::parse_dotenv;
use crate::core::project as core_project;
use crate::models::{
    BuildResult, CreateProjectInput, EnvVarItem, PatStatus, Project, PullResult, RepoStatus,
    SettingItem, UpdateProjectInput,
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
    // Fase 12: Cek ruang disk sebelum clone
    crate::core::storage::check_free_disk_space(&app, 150).map_err(|e| e.to_string())?;

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
    // Fase 12: Cek ruang disk sebelum build
    crate::core::storage::check_free_disk_space(&app, 250).map_err(|e| e.to_string())?;

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

#[tauri::command]
pub fn project_env_list(db: State<'_, Database>, id: i64) -> Result<Vec<EnvVarItem>, String> {
    db.get_env_vars(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_env_set(
    db: State<'_, Database>,
    id: i64,
    key: String,
    value: String,
) -> Result<(), String> {
    db.set_env_var(id, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_env_delete(db: State<'_, Database>, id: i64, key: String) -> Result<(), String> {
    db.delete_env_var(id, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_env_import_dotenv(
    db: State<'_, Database>,
    id: i64,
    content: String,
) -> Result<Vec<EnvVarItem>, String> {
    let parsed = parse_dotenv(&content).map_err(|e| e.to_string())?;
    db.set_env_vars_batch(id, &parsed)
        .map_err(|e| e.to_string())?;
    db.get_env_vars(id).map_err(|e| e.to_string())
}

// -----------------------------------------------------------------------------
// EKSPOR & IMPOR KONFIGURASI PROYEK (FR-23 / FASE 12)
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn project_export_config(db: State<'_, Database>, id: i64) -> Result<String, String> {
    let proj = db.get_project(id).map_err(|e| e.to_string())?;
    let config = crate::models::ProjectExportConfig {
        name: proj.name,
        repo_url: proj.repo_url,
        branch: proj.branch,
        port_manual: proj.port_manual,
        build_cmd: proj.build_cmd,
        start_cmd: proj.start_cmd,
        auto_start: proj.auto_start,
    };
    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Gagal mengonversi konfigurasi ke JSON: {e}"))
}

#[tauri::command]
pub fn project_export_all_configs(db: State<'_, Database>) -> Result<String, String> {
    let projects = db.list_projects().map_err(|e| e.to_string())?;
    let configs: Vec<crate::models::ProjectExportConfig> = projects
        .into_iter()
        .map(|p| crate::models::ProjectExportConfig {
            name: p.name,
            repo_url: p.repo_url,
            branch: p.branch,
            port_manual: p.port_manual,
            build_cmd: p.build_cmd,
            start_cmd: p.start_cmd,
            auto_start: p.auto_start,
        })
        .collect();

    serde_json::to_string_pretty(&configs)
        .map_err(|e| format!("Gagal mengonversi seluruh konfigurasi ke JSON: {e}"))
}

#[tauri::command]
pub fn project_import_config(
    db: State<'_, Database>,
    json_content: String,
) -> Result<Project, String> {
    let config: crate::models::ProjectExportConfig = serde_json::from_str(&json_content)
        .map_err(|e| format!("Format JSON konfigurasi tidak valid: {e}"))?;

    let branch = if config.branch.trim().is_empty() {
        "main".to_string()
    } else {
        config.branch
    };

    let input = CreateProjectInput {
        name: config.name,
        repo_url: config.repo_url,
        branch: Some(branch),
        build_cmd: Some(config.build_cmd),
        start_cmd: Some(config.start_cmd),
        port_manual: config.port_manual,
        auto_start: Some(config.auto_start),
        env_vars: None,
    };

    db.create_project(input).map_err(|e| e.to_string())
}
