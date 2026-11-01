// commands/pm2.rs
// Lapisan tipis Tauri Command untuk PM2 Process Management (PRD Bagian 8.3 & 13 Fase 7)

use crate::core::{pm2, rootfs};
use crate::models::Pm2Status;
use crate::storage::db::Database;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Helper untuk mendapatkan direktori dan binary proot
fn get_runtime_env(app: &AppHandle) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    let rootfs_dir = rootfs::ensure_rootfs_ready(&app_files_dir)
        .map_err(|e| format!("Rootfs belum siap: {e}"))?;
    let proot_bin = rootfs::get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");

    if !proot_bin.exists() {
        return Err(format!("Binary proot tidak ditemukan di {:?}", proot_bin));
    }

    Ok((proot_bin, rootfs_dir, tmp_dir))
}

/// Helper untuk mengumpulkan daftar port yang sedang aktif digunakan oleh proses online
async fn get_active_project_ports(
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
) -> Vec<(String, u16)> {
    pm2::list_processes(proot_bin, rootfs_dir, tmp_dir, false)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.status == "online")
        .filter_map(|p| {
            let active_port = p.detected_port.or(p.port);
            active_port.map(|port| (p.name, port))
        })
        .collect()
}

#[tauri::command]
pub async fn pm2_start(app: AppHandle, project_id: i64) -> Result<Pm2Status, String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    let env_vars = db.get_env_vars(project_id).unwrap_or_default();
    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    let active_ports = get_active_project_ports(&proot_bin, &rootfs_dir, &tmp_dir).await;

    pm2::start_project(
        &project,
        &env_vars,
        &active_ports,
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        false,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_stop(app: AppHandle, project_id: i64) -> Result<Pm2Status, String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::stop_project(&project.name, &proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_restart(app: AppHandle, project_id: i64) -> Result<Pm2Status, String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::restart_project(&project.name, &proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_delete(app: AppHandle, project_id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::delete_process(&project.name, &proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_list(app: AppHandle) -> Result<Vec<Pm2Status>, String> {
    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::list_processes(&proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_stop_all(app: AppHandle) -> Result<(), String> {
    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::stop_all_processes(&proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pm2_check_autostart(app: AppHandle) -> Result<Vec<Pm2Status>, String> {
    let db = app.state::<Database>();
    let projects = db.list_projects().map_err(|e| e.to_string())?;

    let (proot_bin, rootfs_dir, tmp_dir) = match get_runtime_env(&app) {
        Ok(res) => res,
        Err(_) => return Ok(vec![]),
    };

    let active_procs = pm2::list_processes(&proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .unwrap_or_default();

    let mut started = Vec::new();

    for proj in projects {
        if proj.auto_start && proj.last_build_ok == Some(true) {
            let is_already_online = active_procs
                .iter()
                .any(|p| p.name == proj.name && p.status == "online");

            if !is_already_online {
                let env_vars = db.get_env_vars(proj.id).unwrap_or_default();
                let active_ports =
                    get_active_project_ports(&proot_bin, &rootfs_dir, &tmp_dir).await;

                if let Ok(status) = pm2::start_project(
                    &proj,
                    &env_vars,
                    &active_ports,
                    &proot_bin,
                    &rootfs_dir,
                    &tmp_dir,
                    false,
                )
                .await
                {
                    started.push(status);
                }
            }
        }
    }

    Ok(started)
}

#[tauri::command]
pub async fn runtime_logs_start(app: AppHandle, project_id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;

    let stream_mgr = app.state::<pm2::ActiveLogStreamManager>();
    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    pm2::start_runtime_log_stream(
        app.clone(),
        project.id,
        project.name,
        stream_mgr.inner().clone(),
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        false,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn runtime_logs_stop(app: AppHandle, project_id: i64) -> Result<(), String> {
    let stream_mgr = app.state::<pm2::ActiveLogStreamManager>();
    pm2::stop_runtime_log_stream(project_id, stream_mgr.inner()).await;
    Ok(())
}

/// Membuka URL lokal di browser eksternal via Android Native Intent (PRD 9.5 & 13 Fase 9)
#[tauri::command]
pub async fn open_local_url(port: u16) -> Result<(), String> {
    if !(1024..=65535).contains(&port) {
        return Err(format!("Port {port} tidak valid (rentang 1024-65535)"));
    }

    let url = format!("http://localhost:{port}");
    crate::core::port::open_in_browser(&url).map_err(|e| e.to_string())
}
