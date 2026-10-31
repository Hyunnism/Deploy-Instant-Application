// commands/setup.rs
// IPC commands untuk alur peluncuran pertama & manajemen rootfs (Fase 2)

use crate::core::rootfs;
use crate::models::{SetupStatus, SetupVerificationResult};
use tauri::{Emitter, Manager};

#[tauri::command]
pub fn setup_check_status(app: tauri::AppHandle) -> Result<SetupStatus, String> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    let rootfs_dir = app_files_dir.join("rootfs");
    let is_ready = rootfs::is_rootfs_ready(&rootfs_dir);
    let version = rootfs::get_installed_version(&rootfs_dir);
    let free_space_mb = rootfs::get_disk_free_mb(&app_files_dir);

    Ok(SetupStatus {
        is_ready,
        version,
        free_space_mb,
        required_space_mb: rootfs::MIN_FREE_DISK_MB,
    })
}

#[tauri::command]
pub async fn setup_start(app: tauri::AppHandle) -> Result<SetupVerificationResult, String> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    let app_clone = app.clone();
    let files_dir_clone = app_files_dir.clone();

    // Jalankan ekstraksi dalam blocking thread
    let extract_res = tokio::task::spawn_blocking(move || {
        rootfs::extract_rootfs_with_progress(&files_dir_clone, |p| {
            let _ = app_clone.emit("setup-progress", p);
        })
    })
    .await
    .map_err(|e| format!("Eksekusi background task gagal: {e}"))?;

    let rootfs_dir = extract_res?;

    // Verifikasi toolchain yang telah terpasang
    let _ = app.emit(
        "setup-progress",
        crate::models::SetupProgress {
            step: "verifying".into(),
            percent: 95,
            message: "Memverifikasi toolchain (Node, PM2, Cloudflared, Git)...".into(),
        },
    );

    let verify_res = rootfs::verify_installed_toolchain(&rootfs_dir, &app_files_dir).await;

    let _ = app.emit(
        "setup-progress",
        crate::models::SetupProgress {
            step: "complete".into(),
            percent: 100,
            message: "Setup selesai sepenuhnya!".into(),
        },
    );

    Ok(verify_res)
}

#[tauri::command]
pub async fn setup_verify(app: tauri::AppHandle) -> Result<SetupVerificationResult, String> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    let rootfs_dir = app_files_dir.join("rootfs");
    if !rootfs_dir.exists() {
        return Err("Rootfs belum terpasang".into());
    }

    let res = rootfs::verify_installed_toolchain(&rootfs_dir, &app_files_dir).await;
    Ok(res)
}
