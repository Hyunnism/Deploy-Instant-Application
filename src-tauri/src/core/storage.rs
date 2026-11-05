// core/storage.rs
// Manajemen Penyimpanan, Kalkulasi Disk, dan Pembersihan (PRD Fase 12)

use crate::error::{AppError, Result};
use crate::models::{Project, ProjectStorageInfo, SystemStorageInfo};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Menghitung total ukuran file di suatu direktori secara rekursif (dalam bytes)
pub fn calculate_path_size_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total_bytes = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total_bytes += calculate_path_size_bytes(&entry_path);
            } else if let Ok(meta) = entry.metadata() {
                total_bytes += meta.len();
            }
        }
    }
    total_bytes
}

/// Mendapatkan rincian penggunaan penyimpanan untuk suatu project
pub fn get_project_storage_info(app: &AppHandle, project: &Project) -> Result<ProjectStorageInfo> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;

    let app_dir = app_files_dir.join("rootfs").join("app").join(&project.name);
    let node_modules_dir = app_dir.join("node_modules");

    let total_app_bytes = calculate_path_size_bytes(&app_dir);
    let node_modules_bytes = calculate_path_size_bytes(&node_modules_dir);
    let source_bytes = total_app_bytes.saturating_sub(node_modules_bytes);

    // Hitung ukuran log build dan runtime
    let build_log_file = app_files_dir
        .join("logs")
        .join("builds")
        .join(format!("{}.log", project.name));
    let runtime_log_file = app_files_dir
        .join("logs")
        .join("projects")
        .join(format!("{}.log", project.name));

    let build_log_bytes = if build_log_file.exists() {
        build_log_file.metadata().map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let runtime_log_bytes = if runtime_log_file.exists() {
        runtime_log_file.metadata().map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let logs_bytes = build_log_bytes + runtime_log_bytes;

    let has_node_modules = node_modules_dir.exists() && node_modules_bytes > 0;
    let has_logs = logs_bytes > 0;

    Ok(ProjectStorageInfo {
        project_id: project.id,
        project_name: project.name.clone(),
        total_bytes: total_app_bytes + logs_bytes,
        source_bytes,
        node_modules_bytes,
        logs_bytes,
        has_node_modules,
        has_logs,
    })
}

/// Mendapatkan gambaran menyeluruh penggunaan penyimpanan sistem dan aplikasi
pub fn get_system_storage_info(app: &AppHandle) -> Result<SystemStorageInfo> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;

    let rootfs_dir = app_files_dir.join("rootfs");
    let free_disk_mb = crate::core::rootfs::get_disk_free_mb(&app_files_dir);
    let free_disk_bytes = free_disk_mb * 1024 * 1024;

    let rootfs_bytes = calculate_path_size_bytes(&rootfs_dir);
    let rootfs_mb = rootfs_bytes / (1024 * 1024);

    let projects_dir = rootfs_dir.join("app");
    let projects_total_bytes = calculate_path_size_bytes(&projects_dir);
    let projects_total_mb = projects_total_bytes / (1024 * 1024);

    // Sisa penyimpanan dianggap kritis jika di bawah 500 MB
    let is_low_space = free_disk_mb < 500;

    Ok(SystemStorageInfo {
        free_disk_bytes,
        free_disk_mb,
        rootfs_bytes,
        rootfs_mb,
        projects_total_bytes,
        projects_total_mb,
        is_low_space,
    })
}

/// Menghapus direktori node_modules suatu project untuk membebaskan ruang disk
pub fn clean_project_node_modules(app: &AppHandle, project: &Project) -> Result<u64> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;

    let node_modules_dir = app_files_dir
        .join("rootfs")
        .join("app")
        .join(&project.name)
        .join("node_modules");

    if !node_modules_dir.exists() {
        return Ok(0);
    }

    let bytes_freed = calculate_path_size_bytes(&node_modules_dir);
    fs::remove_dir_all(&node_modules_dir)
        .map_err(|e| AppError::Internal(format!("Gagal menghapus node_modules: {e}")))?;

    Ok(bytes_freed)
}

/// Mengosongkan file log build dan runtime suatu project
pub fn clean_project_logs(app: &AppHandle, project: &Project) -> Result<u64> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;

    let build_log_file = app_files_dir
        .join("logs")
        .join("builds")
        .join(format!("{}.log", project.name));
    let runtime_log_file = app_files_dir
        .join("logs")
        .join("projects")
        .join(format!("{}.log", project.name));

    let mut bytes_freed = 0u64;

    if build_log_file.exists() {
        bytes_freed += build_log_file.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = fs::write(&build_log_file, "");
    }

    if runtime_log_file.exists() {
        bytes_freed += runtime_log_file.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = fs::write(&runtime_log_file, "");
    }

    Ok(bytes_freed)
}

/// Memeriksa ketersediaan ruang penyimpanan sebelum clone atau build
pub fn check_free_disk_space(app: &AppHandle, required_mb: u64) -> Result<()> {
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;

    let free_mb = crate::core::rootfs::get_disk_free_mb(&app_files_dir);
    if free_mb < required_mb {
        return Err(AppError::Validation(format!(
            "Ruang penyimpanan tidak mencukupi (tersedia: {} MB, dibutuhkan minimal: {} MB). Silakan bersihkan project atau hapus dependensi yang tidak terpakai.",
            free_mb, required_mb
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_path_size_bytes() {
        let temp_dir = std::env::temp_dir().join("deploy_instant_storage_test");
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test.txt");
        let _ = fs::write(&file_path, "Hello Storage!");

        let size = calculate_path_size_bytes(&temp_dir);
        assert!(size >= 14);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
