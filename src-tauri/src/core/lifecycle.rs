// core/lifecycle.rs
// Lifecycle Android, Foreground Service, WakeLock, dan Pembersihan Proses (PRD Fase 11)

use crate::error::{AppError, Result};
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

static GLOBAL_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Daftarkan global AppHandle saat bootstrap aplikasi
pub fn register_app_handle(app: AppHandle) {
    let _ = GLOBAL_APP_HANDLE.set(app);
}

/// Dapatkan global AppHandle jika tersedia
pub fn get_app_handle() -> Option<AppHandle> {
    GLOBAL_APP_HANDLE.get().cloned()
}

// -----------------------------------------------------------------------------
// JNI BRIDGE NATIVE ANDROID
// -----------------------------------------------------------------------------

#[cfg(target_os = "android")]
pub fn sync_foreground_service_native(active_count: i32, desc: &str) {
    use jni::objects::{JObject, JValue};

    eprintln!("[DeployInstant JNI] sync_foreground_service_native: count={}, desc={}", active_count, desc);

    let vm = match unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) } {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[DeployInstant JNI] Gagal ambil JavaVM: {e}");
            return;
        }
    };
    let mut env = match vm.attach_current_thread() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[DeployInstant JNI] Gagal attach thread: {e}");
            return;
        }
    };

    let context = unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };
    let desc_jstring = match env.new_string(desc) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[DeployInstant JNI] Gagal new_string: {e}");
            return;
        }
    };

    let res = env.call_method(
        &context,
        "startDeployService",
        "(ILjava/lang/String;)V",
        &[JValue::Int(active_count), JValue::Object(&desc_jstring)],
    );
    if let Err(e) = res {
        eprintln!("[DeployInstant JNI] Gagal panggil startDeployService: {e}");
    }
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
}

#[cfg(not(target_os = "android"))]
pub fn sync_foreground_service_native(active_count: i32, desc: &str) {
    println!("[host fallback] sync_foreground_service: {} active, {}", active_count, desc);
}

#[cfg(target_os = "android")]
pub fn stop_foreground_service_native() {
    use jni::objects::JObject;

    eprintln!("[DeployInstant JNI] stop_foreground_service_native dipanggil");

    let vm = match unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) } {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut env = match vm.attach_current_thread() {
        Ok(e) => e,
        Err(_) => return,
    };

    let context = unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };
    let _ = env.call_method(
        &context,
        "stopDeployService",
        "()V",
        &[],
    );
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

#[cfg(not(target_os = "android"))]
pub fn stop_foreground_service_native() {
    println!("[host fallback] stop_foreground_service");
}

#[cfg(target_os = "android")]
pub fn is_ignoring_battery_optimizations_native() -> bool {
    use jni::objects::JObject;

    let vm = match unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) } {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut env = match vm.attach_current_thread() {
        Ok(e) => e,
        Err(_) => return false,
    };

    let context = unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };
    let res = env.call_method(
        &context,
        "isIgnoringBatteryOptimizations",
        "()Z",
        &[],
    );
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
        return false;
    }
    match res {
        Ok(val) => val.z().unwrap_or(false),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "android"))]
pub fn is_ignoring_battery_optimizations_native() -> bool {
    true // Host selalu dianggap tidak dibatasi
}

#[cfg(target_os = "android")]
pub fn open_battery_settings_native() {
    use jni::objects::JObject;

    let vm = match unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) } {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut env = match vm.attach_current_thread() {
        Ok(e) => e,
        Err(_) => return,
    };

    let context = unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };
    let _ = env.call_method(
        &context,
        "openBatterySettings",
        "()V",
        &[],
    );
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

#[cfg(not(target_os = "android"))]
pub fn open_battery_settings_native() {
    println!("[host fallback] open_battery_settings");
}

// -----------------------------------------------------------------------------
// JNI CALLBACKS DARI ANDROID RUNTIME
// -----------------------------------------------------------------------------

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_hyunnism_deployinstant_MainActivity_nativeOnDestroy(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    eprintln!("[DeployInstant Lifecycle] MainActivity.onDestroy() terpanggil!");
    if let Some(app) = get_app_handle() {
        tauri::async_runtime::block_on(async {
            let _ = shutdown_all_cleanly(&app).await;
        });
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_hyunnism_deployinstant_service_ServiceActionReceiver_nativeStopAllFromService(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    eprintln!("[DeployInstant Lifecycle] Stop Semua dari Notifikasi Tray terpanggil!");
    if let Some(app) = get_app_handle() {
        tauri::async_runtime::spawn(async move {
            let _ = shutdown_all_cleanly(&app).await;
        });
    }
}

// -----------------------------------------------------------------------------
// SHUTDOWN & ORPHAN PROCESS CLEANUP
// -----------------------------------------------------------------------------

/// Menghentikan seluruh proses tunnel, PM2, dan melepaskan Foreground Service
pub async fn shutdown_all_cleanly(app: &AppHandle) -> Result<()> {
    eprintln!("[DeployInstant Lifecycle] Memulai shutdown bersih seluruh layanan...");

    // 1. Hentikan seluruh tunnel aktif
    if let Some(tunnel_mgr) = app.try_state::<crate::core::tunnel::TunnelManager>() {
        let _ = tunnel_mgr.stop_all(app).await;
    }

    // 2. Hentikan seluruh project di PM2 & matikan daemon PM2
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Gagal mendapatkan app_data_dir: {e}")))?;
    let proot_bin = crate::core::rootfs::get_proot_binary();
    let rootfs_dir = app_files_dir.join("rootfs");
    let tmp_dir = app_files_dir.join("tmp");

    if rootfs_dir.exists() {
        let _ = crate::core::pm2::stop_all_processes(&proot_bin, &rootfs_dir, &tmp_dir, false).await;
        let _ = crate::core::pm2::kill_pm2_daemon(&proot_bin, &rootfs_dir, &tmp_dir, false).await;
    }

    // 3. Matikan Android Foreground Service & lepaskan WakeLock
    stop_foreground_service_native();

    eprintln!("[DeployInstant Lifecycle] Shutdown bersih selesai.");
    Ok(())
}

/// Membersihkan sisa proses yatim (misal setelah app force-close atau crash)
pub async fn clean_orphan_processes(app_files_dir: &std::path::Path) -> usize {
    let proot_bin = crate::core::rootfs::get_proot_binary();
    let rootfs_dir = app_files_dir.join("rootfs");
    let tmp_dir = app_files_dir.join("tmp");

    if !rootfs_dir.exists() {
        return 0;
    }

    let mut cleaned = 0;

    // Bersihkan sisa proses cloudflared jika ada
    let mut cmd_cf = crate::core::exec::proot_exec(
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        "/",
        "pkill -f 'cloudflared tunnel' 2>/dev/null || true",
        &[],
        false,
        true,
    );
    if let Ok(out) = cmd_cf.output().await {
        if out.status.success() {
            cleaned += 1;
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_host_fallback() {
        sync_foreground_service_native(1, "1 project online");
        assert!(is_ignoring_battery_optimizations_native());
        stop_foreground_service_native();
    }
}
