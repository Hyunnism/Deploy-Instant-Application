// commands/tunnel.rs
// Lapisan tipis Tauri Command untuk Cloudflare Quick Tunnel (PRD Bagian 13 Fase 10)

use crate::core::pm2;
use crate::core::port;
use crate::core::rootfs;
use crate::core::tunnel::{self, TunnelManager};
use crate::models::ProjectTunnelInfo;
use crate::storage::db::Database;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

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

#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    project_id: i64,
    tunnel_mgr: State<'_, TunnelManager>,
) -> Result<ProjectTunnelInfo, String> {
    let db = app.state::<Database>();
    let project = db.get_project(project_id).map_err(|e| e.to_string())?;
    let env_vars = db.get_env_vars(project_id).unwrap_or_default();

    let (proot_bin, rootfs_dir, tmp_dir) = get_runtime_env(&app)?;

    // Cari port riil yang sedang aktif listening (Fase 9)
    let active_procs = pm2::list_processes(&proot_bin, &rootfs_dir, &tmp_dir, false)
        .await
        .unwrap_or_default();
    let runtime_port = active_procs
        .iter()
        .find(|p| p.name == project.name)
        .and_then(|p| p.detected_port.or(p.port));

    let port = runtime_port
        .or_else(|| {
            port::resolve_project_port(&project, &env_vars, &[])
                .ok()
                .map(|(p, _)| p)
        })
        .unwrap_or(3001);

    tunnel_mgr
        .start_tunnel(&app, project_id, port, &proot_bin, &rootfs_dir, &tmp_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_tunnel(
    app: AppHandle,
    project_id: i64,
    tunnel_mgr: State<'_, TunnelManager>,
) -> Result<(), String> {
    tunnel_mgr
        .stop_tunnel(&app, project_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tunnel_status(
    project_id: i64,
    tunnel_mgr: State<'_, TunnelManager>,
) -> Result<Option<ProjectTunnelInfo>, String> {
    Ok(tunnel_mgr.get_tunnel_status(project_id).await)
}

#[tauri::command]
pub async fn list_active_tunnels(
    tunnel_mgr: State<'_, TunnelManager>,
) -> Result<Vec<ProjectTunnelInfo>, String> {
    Ok(tunnel_mgr.list_active_tunnels().await)
}

#[tauri::command]
pub fn share_url(url: String, title: String) -> Result<(), String> {
    tunnel::share_url_native(&url, &title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL harus diawali dengan http:// atau https://".into());
    }
    crate::core::port::open_in_browser(&url).map_err(|e| e.to_string())
}
