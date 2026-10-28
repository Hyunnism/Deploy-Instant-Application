// core/rootfs.rs
// Deteksi path binary proot, ekstraksi & verifikasi rootfs Debian arm64

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "android")]
pub fn get_native_lib_dir() -> PathBuf {
    if let Ok(path) = get_native_lib_dir_jni() {
        if path.join("libproot.so").exists() {
            return path;
        }
    }
    // Fallback 1: /data/data/com.hyunnism.deployinstant/lib
    let data_lib = PathBuf::from("/data/data/com.hyunnism.deployinstant/lib");
    if data_lib.join("libproot.so").exists() {
        return data_lib;
    }
    // Fallback 2: /data/local/tmp
    let local_tmp = PathBuf::from("/data/local/tmp");
    if local_tmp.join("libproot.so").exists() {
        return local_tmp;
    }
    if local_tmp.join("libpdn.so").exists() {
        return local_tmp;
    }
    PathBuf::from("/data/local/tmp")
}

#[cfg(target_os = "android")]
fn get_native_lib_dir_jni() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let app_info = env
        .call_method(
            &context,
            "getApplicationInfo",
            "()Landroid/content/pm/ApplicationInfo;",
            &[],
        )?
        .l()?;
    let native_lib_dir_field = env
        .get_field(&app_info, "nativeLibraryDir", "Ljava/lang/String;")?
        .l()?;
    let jstr: jni::objects::JString = native_lib_dir_field.into();
    let dir_str: String = env.get_string(&jstr)?.into();
    Ok(PathBuf::from(dir_str))
}

#[cfg(not(target_os = "android"))]
pub fn get_native_lib_dir() -> PathBuf {
    PathBuf::from("src-tauri/assets")
}

pub fn get_proot_binary() -> PathBuf {
    let dir = get_native_lib_dir();
    let proot_so = dir.join("libproot.so");
    if proot_so.exists() {
        return proot_so;
    }
    let pdn_so = dir.join("libpdn.so");
    if pdn_so.exists() {
        return pdn_so;
    }
    let fallback = PathBuf::from("/data/local/tmp/libpdn.so");
    if fallback.exists() {
        return fallback;
    }
    proot_so
}

/// Memastikan rootfs Debian arm64 siap dieksekusi
pub fn ensure_rootfs_ready(app_files_dir: &Path) -> Result<PathBuf, String> {
    // 1. Cek beberapa kemungkinan lokasi rootfs di dalam sandbox app
    let candidates = [
        app_files_dir.join("rootfs"),
        app_files_dir.join("files/rootfs"),
        PathBuf::from("/data/data/com.hyunnism.deployinstant/files/rootfs"),
        PathBuf::from("/data/user/0/com.hyunnism.deployinstant/files/rootfs"),
    ];

    for rootfs_dir in &candidates {
        let node_bin = rootfs_dir.join("usr/local/bin/node");
        if node_bin.exists() {
            ensure_resolv_conf(rootfs_dir)?;
            let _ = fs::create_dir_all(rootfs_dir.join("tmp"));
            return Ok(rootfs_dir.clone());
        }
    }

    let rootfs_dir = candidates[0].clone();
    let tmp_dir = app_files_dir.join("tmp");
    if !tmp_dir.exists() {
        let _ = fs::create_dir_all(&tmp_dir);
    }

    // Ekstrak rootfs archive jika ada
    fs::create_dir_all(&rootfs_dir).map_err(|e| format!("Gagal membuat rootfs dir: {e}"))?;

    let archive_candidates = [
        app_files_dir.join("rootfs.tar.gz"),
        PathBuf::from("/data/local/tmp/rootfs.tar.gz"),
        PathBuf::from("src-tauri/assets/rootfs.tar.gz"),
    ];

    let mut found_archive = None;
    for cand in &archive_candidates {
        if cand.exists() {
            found_archive = Some(cand.clone());
            break;
        }
    }

    if let Some(archive) = found_archive {
        let _ = std::process::Command::new("/system/bin/tar")
            .args([
                "-xzf",
                archive.to_str().unwrap_or_default(),
                "-C",
                rootfs_dir.to_str().unwrap_or_default(),
            ])
            .status();

        let node_bin = rootfs_dir.join("usr/local/bin/node");
        if !node_bin.exists() {
            return Err(
                "Ekstraksi rootfs gagal: binary node tidak ditemukan di rootfs".to_string(),
            );
        }
    } else {
        return Err("File arsip rootfs.tar.gz tidak ditemukan".to_string());
    }

    ensure_resolv_conf(&rootfs_dir)?;
    let _ = fs::create_dir_all(rootfs_dir.join("tmp"));
    Ok(rootfs_dir)
}

fn ensure_resolv_conf(rootfs_dir: &Path) -> Result<(), String> {
    let etc_dir = rootfs_dir.join("etc");
    if !etc_dir.exists() {
        let _ = fs::create_dir_all(&etc_dir);
    }
    let resolv_path = etc_dir.join("resolv.conf");
    let _ = fs::remove_file(&resolv_path);
    let content = "nameserver 1.1.1.1\nnameserver 8.8.8.8\n";
    let _ = fs::write(resolv_path, content);
    Ok(())
}
