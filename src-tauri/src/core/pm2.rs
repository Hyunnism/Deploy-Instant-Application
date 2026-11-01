// core/pm2.rs
// Start, stop, restart, pm2 jlist, auto-start, dan status management (PRD Bagian 8.3 & 13 Fase 7)

use crate::core::exec::proot_exec;
use crate::core::port::resolve_project_port;
use crate::error::{AppError, Result};
use crate::models::{EnvVarItem, Pm2Status, Project};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Mengambil daftar seluruh proses PM2 beserta metrik runtime
pub async fn list_processes(
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<Vec<Pm2Status>> {
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        "/",
        "pm2 jlist",
        &[],
        no_seccomp,
        false, // PM2 daemon harus tetap berjalan
    );

    let output = match tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await
    {
        Ok(res) => {
            res.map_err(|e| AppError::Internal(format!("Gagal mengeksekusi pm2 jlist: {e}")))?
        }
        Err(_) => return Ok(vec![]),
    };

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    parse_pm2_jlist(&stdout_str)
}

/// Parsing output mentah `pm2 jlist` yang tangguh terhadap banner ASCII/log awal PM2
pub fn parse_pm2_jlist(raw_stdout: &str) -> Result<Vec<Pm2Status>> {
    let mut parsed_items: Option<Vec<Value>> = None;

    // Cari kemunculan '[' yang berhasil di-deserialisasi sebagai JSON array
    // Mengabaikan log awalan seperti [PM2] dan log trailing
    for (i, c) in raw_stdout.char_indices() {
        if c == '[' {
            let mut de = serde_json::Deserializer::from_str(&raw_stdout[i..]);
            if let Ok(items) = Vec::<Value>::deserialize(&mut de) {
                parsed_items = Some(items);
                break;
            }
        }
    }

    let items = match parsed_items {
        Some(it) => it,
        None => return Ok(vec![]),
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut result = Vec::new();

    for item in items {
        let name = item["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }

        let pm_id = item["pm_id"].as_u64().map(|v| v as u32);
        let pid = item["pid"]
            .as_u64()
            .and_then(|v| if v > 0 { Some(v as u32) } else { None });

        let pm2_env = &item["pm2_env"];
        let status = pm2_env["status"]
            .as_str()
            .unwrap_or("stopped")
            .to_lowercase();

        let uptime = if status == "online" {
            let start_ms = pm2_env["pm_uptime"].as_u64().unwrap_or(now_ms);
            now_ms.saturating_sub(start_ms) / 1000
        } else {
            0
        };

        let restarts = pm2_env["restart_time"].as_u64().unwrap_or(0) as u32;

        let monit = &item["monit"];
        let cpu = monit["cpu"].as_f64().unwrap_or(0.0) as f32;
        let memory = monit["memory"].as_u64().unwrap_or(0);

        // Ekstrak port dari environment jika tersedia
        let port = pm2_env["env"]["PORT"]
            .as_str()
            .and_then(|s| s.parse::<u16>().ok())
            .or_else(|| {
                pm2_env["env"]["PORT"]
                    .as_u64()
                    .and_then(|p| u16::try_from(p).ok())
            });

        result.push(Pm2Status {
            name,
            status,
            pid,
            pm_id,
            uptime,
            restarts,
            cpu,
            memory,
            port,
        });
    }

    Ok(result)
}

/// Menjalankan project dengan PM2 (PRD Bagian 8.3, 8.6, 13 Fase 7)
pub async fn start_project(
    project: &Project,
    env_vars: &[EnvVarItem],
    active_projects: &[(String, u16)],
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<Pm2Status> {
    // 1. Validasi: Deploy hanya aktif jika build terakhir lulus (PRD Bagian 13 Fase 7)
    if project.last_build_ok != Some(true) {
        return Err(AppError::Validation(
            "Project belum di-build atau build terakhir gagal. Harap jalankan build terlebih dahulu."
                .into(),
        ));
    }

    // 2. Resolusi target port dan validasi bentrok (PRD Bagian 8.7)
    let (target_port, should_inject_port) =
        resolve_project_port(project, env_vars, active_projects)?;

    let app_dir = rootfs_dir.join("app").join(&project.name);
    if !app_dir.exists() {
        return Err(AppError::NotFound(format!(
            "Folder project /app/{} tidak ditemukan di dalam rootfs. Harap clone ulang.",
            project.name
        )));
    }

    // 3. Cek dukungan ecosystem.config.js / ecosystem.config.cjs (PRD Bagian 13 Fase 7)
    let ecosystem_js = app_dir.join("ecosystem.config.js");
    let ecosystem_cjs = app_dir.join("ecosystem.config.cjs");

    let pm2_cmd = if ecosystem_js.exists() {
        "pm2 start ecosystem.config.js --env production --max-restarts 5".to_string()
    } else if ecosystem_cjs.exists() {
        "pm2 start ecosystem.config.cjs --env production --max-restarts 5".to_string()
    } else {
        let trimmed_start = project.start_cmd.trim();
        if trimmed_start.starts_with("npm ") {
            let sub = trimmed_start.strip_prefix("npm ").unwrap_or("start").trim();
            format!(
                "pm2 start npm --name '{}' --max-restarts 5 -- {}",
                project.name, sub
            )
        } else if trimmed_start.starts_with("node ") {
            let script = trimmed_start.strip_prefix("node ").unwrap_or("").trim();
            if script.ends_with(".js") || script.ends_with(".mjs") || script.ends_with(".cjs") {
                format!(
                    "pm2 start {} --name '{}' --max-restarts 5",
                    script, project.name
                )
            } else {
                format!(
                    "pm2 start '{}' --name '{}' --max-restarts 5",
                    trimmed_start.replace('\'', "'\\''"),
                    project.name
                )
            }
        } else {
            format!(
                "pm2 start '{}' --name '{}' --max-restarts 5",
                trimmed_start.replace('\'', "'\\''"),
                project.name
            )
        }
    };

    // 4. Siapkan environment variables
    let mut proot_envs: Vec<(String, String)> = Vec::new();
    proot_envs.push(("NODE_ENV".into(), "production".into()));

    if should_inject_port {
        proot_envs.push(("PORT".into(), target_port.to_string()));
    }

    for env_item in env_vars {
        if env_item.key != "NODE_ENV" && (!should_inject_port || env_item.key != "PORT") {
            proot_envs.push((env_item.key.clone(), env_item.value.clone()));
        }
    }

    // 5. Pastikan direktori log PM2 ada di rootfs & eksekusi pm2 start di direktori project (/app/<name>)
    let pm2_logs = rootfs_dir.join("root").join(".pm2").join("logs");
    let pm2_pids = rootfs_dir.join("root").join(".pm2").join("pids");
    let _ = std::fs::create_dir_all(&pm2_logs);
    let _ = std::fs::create_dir_all(&pm2_pids);

    let full_exec_cmd = format!("mkdir -p /root/.pm2/logs /root/.pm2/pids && {pm2_cmd}");
    let cwd = format!("/app/{}", project.name);
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        &cwd,
        &full_exec_cmd,
        &proot_envs,
        no_seccomp,
        false, // PM2 daemon harus tetap hidup di background
    );

    let output = match tokio::time::timeout(std::time::Duration::from_secs(15), cmd.output()).await
    {
        Ok(res) => {
            res.map_err(|e| AppError::Internal(format!("Gagal menjalankan pm2 start: {e}")))?
        }
        Err(_) => {
            return Err(AppError::Internal(
                "Timeout menjalankan pm2 start (15 detik)".into(),
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::Internal(format!(
            "Gagal start PM2 (exit code {:?}): {stderr} {stdout}",
            output.status.code()
        )));
    }

    // Tunggu sejenak agar daemon PM2 meregister proses
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Ambil status terbaru dari jlist
    let all_procs = list_processes(proot_bin, rootfs_dir, tmp_dir, no_seccomp).await?;
    let proc_status = all_procs
        .into_iter()
        .find(|p| p.name == project.name)
        .unwrap_or(Pm2Status {
            name: project.name.clone(),
            status: "online".into(),
            pid: None,
            pm_id: None,
            uptime: 1,
            restarts: 0,
            cpu: 0.0,
            memory: 0,
            port: Some(target_port),
        });

    Ok(proc_status)
}

/// Menghentikan project di PM2
pub async fn stop_project(
    project_name: &str,
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<Pm2Status> {
    let stop_cmd = format!("pm2 stop '{}'", project_name);
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        "/",
        &stop_cmd,
        &[],
        no_seccomp,
        false,
    );

    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await;

    // Ambil status terbaru
    let all = list_processes(proot_bin, rootfs_dir, tmp_dir, no_seccomp).await?;
    let status = all
        .into_iter()
        .find(|p| p.name == project_name)
        .unwrap_or(Pm2Status {
            name: project_name.to_string(),
            status: "stopped".into(),
            pid: None,
            pm_id: None,
            uptime: 0,
            restarts: 0,
            cpu: 0.0,
            memory: 0,
            port: None,
        });

    Ok(status)
}

/// Restart project di PM2
pub async fn restart_project(
    project_name: &str,
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<Pm2Status> {
    let restart_cmd = format!("pm2 restart '{}'", project_name);
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        "/",
        &restart_cmd,
        &[],
        no_seccomp,
        false,
    );

    let output = match tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await
    {
        Ok(res) => res.map_err(|e| AppError::Internal(format!("Gagal restart PM2: {e}")))?,
        Err(_) => {
            return Err(AppError::Internal(
                "Timeout me-restart PM2 (10 detik)".into(),
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!(
            "Gagal restart project: {stderr}"
        )));
    }

    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let all = list_processes(proot_bin, rootfs_dir, tmp_dir, no_seccomp).await?;
    let status = all
        .into_iter()
        .find(|p| p.name == project_name)
        .unwrap_or(Pm2Status {
            name: project_name.to_string(),
            status: "online".into(),
            pid: None,
            pm_id: None,
            uptime: 1,
            restarts: 0,
            cpu: 0.0,
            memory: 0,
            port: None,
        });

    Ok(status)
}

/// Menghapus project dari daftar PM2
pub async fn delete_process(
    project_name: &str,
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<()> {
    let delete_cmd = format!("pm2 delete '{}'", project_name);
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        "/",
        &delete_cmd,
        &[],
        no_seccomp,
        false,
    );

    // Abaikan jika proses belum ada di PM2
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await;
    Ok(())
}

/// Menghentikan semua proses PM2 ("Stop Semua", PRD Bagian 8.8 & 10.1)
pub async fn stop_all_processes(
    proot_bin: &Path,
    rootfs_dir: &Path,
    tmp_dir: &Path,
    no_seccomp: bool,
) -> Result<()> {
    let mut cmd = proot_exec(
        proot_bin,
        rootfs_dir,
        tmp_dir,
        "/",
        "pm2 stop all",
        &[],
        no_seccomp,
        false,
    );

    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pm2_jlist_clean() {
        let json_input = r#"[
            {
                "name": "express-app",
                "pm_id": 0,
                "pid": 4321,
                "monit": {
                    "memory": 52428800,
                    "cpu": 1.2
                },
                "pm2_env": {
                    "status": "online",
                    "pm_uptime": 1730460000000,
                    "restart_time": 2,
                    "env": {
                        "PORT": "3000"
                    }
                }
            }
        ]"#;

        let res = parse_pm2_jlist(json_input).unwrap();
        assert_eq!(res.len(), 1);
        let item = &res[0];
        assert_eq!(item.name, "express-app");
        assert_eq!(item.status, "online");
        assert_eq!(item.pid, Some(4321));
        assert_eq!(item.pm_id, Some(0));
        assert_eq!(item.restarts, 2);
        assert_eq!(item.port, Some(3000));
        assert_eq!(item.memory, 52428800);
        assert!((item.cpu - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_pm2_jlist_with_ascii_banner() {
        let raw_with_banner = r#"
[PM2] Spawning PM2 daemon with pm2_home=/root/.pm2
[PM2] PM2 Successfully daemonized
[
  {
    "name": "worker",
    "pm_id": 1,
    "pid": 0,
    "monit": {
      "memory": 0,
      "cpu": 0
    },
    "pm2_env": {
      "status": "stopped",
      "pm_uptime": 0,
      "restart_time": 5,
      "env": {}
    }
  }
]
[PM2] Action done.
"#;

        let res = parse_pm2_jlist(raw_with_banner).unwrap();
        assert_eq!(res.len(), 1);
        let item = &res[0];
        assert_eq!(item.name, "worker");
        assert_eq!(item.status, "stopped");
        assert_eq!(item.pid, None); // pid 0 -> None
        assert_eq!(item.restarts, 5);
        assert_eq!(item.port, None);
    }

    #[test]
    fn test_parse_pm2_jlist_empty_or_invalid() {
        assert_eq!(parse_pm2_jlist("").unwrap().len(), 0);
        assert_eq!(parse_pm2_jlist("No process found").unwrap().len(), 0);
        assert_eq!(parse_pm2_jlist("[]").unwrap().len(), 0);
    }
}
