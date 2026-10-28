use crate::core::{exec::proot_exec, rootfs};
use crate::models::ProbeResult;
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: u64,
    pub arch: String,
}

#[tauri::command]
pub fn system_ping() -> PingResponse {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    PingResponse {
        message: "pong".to_string(),
        timestamp,
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[tauri::command]
pub async fn system_run_probe(
    app: tauri::AppHandle,
    no_seccomp: bool,
) -> Result<ProbeResult, String> {
    let start_time = std::time::Instant::now();

    // Dapatkan direktori data aplikasi
    let app_files_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    let rootfs_dir = rootfs::ensure_rootfs_ready(&app_files_dir)?;
    let proot_bin = rootfs::get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");

    if !proot_bin.exists() {
        return Err(format!("Binary proot tidak ditemukan di {:?}", proot_bin));
    }

    // 1. Probe uname -m
    let mut cmd_uname = proot_exec(
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        "/",
        "uname -m",
        &[],
        no_seccomp,
    );
    let output_uname = cmd_uname
        .output()
        .await
        .map_err(|e| format!("uname gagal: {e}"))?;
    let uname_str = String::from_utf8_lossy(&output_uname.stdout)
        .trim()
        .to_string();

    // 2. Probe node -v
    let mut cmd_node = proot_exec(
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        "/",
        "node -v",
        &[],
        no_seccomp,
    );
    let output_node = cmd_node
        .output()
        .await
        .map_err(|e| format!("node gagal: {e}"))?;
    let node_str = String::from_utf8_lossy(&output_node.stdout)
        .trim()
        .to_string();

    // 3. Probe git --version
    let mut cmd_git = proot_exec(
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        "/",
        "git --version",
        &[],
        no_seccomp,
    );
    let output_git = cmd_git.output().await;
    let git_str = match output_git {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if err.contains("not found") || err.is_empty() {
                    "Belum terinstal (rootfs minimal)".to_string()
                } else {
                    err
                }
            } else {
                s
            }
        }
        Err(e) => format!("git tidak terpasang ({e})"),
    };

    // 4. Probe network: verifikasi konektivitas internet via Node fetch
    let mut cmd_net = proot_exec(
        &proot_bin,
        &rootfs_dir,
        &tmp_dir,
        "/",
        "node -e \"fetch('https://registry.npmjs.org').then(r => console.log('HTTPS ' + r.status + ' OK (registry.npmjs.org)')).catch(e => console.error(e.message))\"",
        &[],
        no_seccomp,
    );
    let net_output = cmd_net.output().await;
    let net_str = match net_output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                String::from_utf8_lossy(&out.stderr).trim().to_string()
            } else {
                s
            }
        }
        Err(e) => format!("Error jaringan ({e})"),
    };

    let elapsed = start_time.elapsed().as_millis() as u64;
    let success = uname_str.contains("aarch64") && node_str.starts_with('v');

    Ok(ProbeResult {
        success,
        uname: uname_str,
        node_version: node_str,
        git_version: git_str,
        network_check: if net_str.is_empty() {
            "Koneksi terverifikasi".to_string()
        } else {
            net_str
        },
        elapsed_ms: elapsed,
        error: if success {
            None
        } else {
            Some("Node.js gagal mengembalikan versi valid".into())
        },
    })
}
