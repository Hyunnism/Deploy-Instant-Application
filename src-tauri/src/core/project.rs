// core/project.rs
// Operasi Git Clone & Git Pull di dalam rootfs sandbox via proot (PRD Bagian 13 Fase 4)

use crate::core::exec::proot_exec;
use crate::core::rootfs::get_proot_binary;
use crate::error::AppError;
use crate::models::{BuildLogEvent, BuildResult, GitLogEvent, Project, PullResult, RepoStatus};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

pub const GIT_LOG_EVENT: &str = "git:log";
pub const BUILD_LOG_EVENT: &str = "build:log";

/// Manager untuk tracking proses build aktif dan pembatalan (PRD Bagian 10 & 13 Fase 5)
#[derive(Clone, Default)]
pub struct ActiveBuildManager {
    cancellers: Arc<Mutex<HashMap<i64, oneshot::Sender<()>>>>,
}

impl ActiveBuildManager {
    pub fn new() -> Self {
        Self {
            cancellers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, project_id: i64) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        let mut map = self.cancellers.lock().unwrap();
        map.insert(project_id, tx);
        rx
    }

    pub fn unregister(&self, project_id: i64) {
        let mut map = self.cancellers.lock().unwrap();
        map.remove(&project_id);
    }

    pub fn cancel(&self, project_id: i64) -> bool {
        let mut map = self.cancellers.lock().unwrap();
        if let Some(tx) = map.remove(&project_id) {
            let _ = tx.send(());
            true
        } else {
            false
        }
    }
}

/// Memeriksa status repositori Git lokal pada direktori /app/<name>
pub async fn get_project_repo_status(
    rootfs_dir: &Path,
    app_files_dir: &Path,
    project_name: &str,
) -> RepoStatus {
    let project_dir = rootfs_dir.join("app").join(project_name);
    let git_dir = project_dir.join(".git");

    if !project_dir.exists() || !git_dir.exists() {
        return RepoStatus {
            is_cloned: false,
            commit_hash: None,
            commit_message: None,
            branch: None,
            error: None,
        };
    }

    let proot_bin = get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");
    let cwd = format!("/app/{}", project_name);

    // 1. Ambil commit hash pendek
    let mut cmd_hash = proot_exec(
        &proot_bin,
        rootfs_dir,
        &tmp_dir,
        &cwd,
        "git rev-parse --short HEAD",
        &[],
        false,
    );
    let hash_out = cmd_hash.output().await;
    let commit_hash = match hash_out {
        Ok(out) if out.status.success() => {
            let h = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if h.is_empty() { None } else { Some(h) }
        }
        _ => None,
    };

    // 2. Ambil commit message terakhir (baris pertama)
    let mut cmd_msg = proot_exec(
        &proot_bin,
        rootfs_dir,
        &tmp_dir,
        &cwd,
        "git log -1 --pretty=%s",
        &[],
        false,
    );
    let msg_out = cmd_msg.output().await;
    let commit_message = match msg_out {
        Ok(out) if out.status.success() => {
            let m = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if m.is_empty() { None } else { Some(m) }
        }
        _ => None,
    };

    // 3. Ambil branch aktif
    let mut cmd_branch = proot_exec(
        &proot_bin,
        rootfs_dir,
        &tmp_dir,
        &cwd,
        "git branch --show-current",
        &[],
        false,
    );
    let branch_out = cmd_branch.output().await;
    let branch = match branch_out {
        Ok(out) if out.status.success() => {
            let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if b.is_empty() { None } else { Some(b) }
        }
        _ => None,
    };

    RepoStatus {
        is_cloned: true,
        commit_hash,
        commit_message,
        branch,
        error: None,
    }
}

/// Menyiapkan script helper GIT_ASKPASS sementara di rootfs /tmp
/// Mencegah PAT bocor ke command line, process list (ps), dan .git/config
struct AskPassGuard<'a> {
    host_path: std::path::PathBuf,
    container_path: String,
    _pat: &'a str,
}

impl<'a> AskPassGuard<'a> {
    fn create(rootfs_dir: &Path, pat: &'a str) -> Result<Self, AppError> {
        let rootfs_tmp = rootfs_dir.join("tmp");
        if !rootfs_tmp.exists() {
            let _ = fs::create_dir_all(&rootfs_tmp);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let filename = format!(".git_askpass_{}.sh", timestamp);
        let host_path = rootfs_tmp.join(&filename);
        let container_path = format!("/tmp/{}", filename);

        // Script POSIX sh yang membaca prompt git:
        // Jika prompt meminta username, kembalikan 'x-access-token'
        // Jika prompt meminta password, kembalikan PAT dari env GIT_PAT_SECRET
        let script_content = "#!/bin/sh\ncase \"$1\" in\n  *Username*|*username*) echo \"x-access-token\" ;;\n  *) echo \"$GIT_PAT_SECRET\" ;;\nesac\n";

        fs::write(&host_path, script_content)
            .map_err(|e| AppError::Internal(format!("Gagal menulis askpass helper: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&host_path, fs::Permissions::from_mode(0o700));
        }

        Ok(Self {
            host_path,
            container_path,
            _pat: pat,
        })
    }

    fn env_vars(&self) -> Vec<(String, String)> {
        vec![
            ("GIT_ASKPASS".into(), self.container_path.clone()),
            ("GIT_PAT_SECRET".into(), self._pat.to_string()),
            ("GIT_TERMINAL_PROMPT".into(), "0".into()),
        ]
    }
}

impl Drop for AskPassGuard<'_> {
    fn drop(&mut self) {
        if self.host_path.exists() {
            let _ = fs::remove_file(&self.host_path);
        }
    }
}

/// Helper untuk streaming line output dari Child process ke Tauri event
async fn stream_command_output<R: tokio::io::AsyncRead + Unpin>(
    reader: R,
    app: AppHandle,
    project_id: i64,
    stream: &'static str,
    pat_to_redact: Option<String>,
) -> Vec<String> {
    let mut lines = BufReader::new(reader).lines();
    let mut collected = Vec::new();

    while let Ok(Some(raw_line)) = lines.next_line().await {
        let clean_line = if let Some(ref pat) = pat_to_redact {
            raw_line.replace(pat, "******")
        } else {
            raw_line
        };

        let _ = app.emit(
            GIT_LOG_EVENT,
            GitLogEvent {
                project_id,
                line: clean_line.clone(),
                stream: stream.to_string(),
            },
        );

        collected.push(clean_line);
    }

    collected
}

/// Menjalankan git clone ke direktori /app/<project_name>
/// Pembersihan otomatis direktori /app/<project_name> dilakukan bila proses gagal (PRD Bagian 13 Fase 4)
pub async fn git_clone(
    app: &AppHandle,
    rootfs_dir: &Path,
    app_files_dir: &Path,
    project: &Project,
    pat: Option<&str>,
) -> Result<RepoStatus, AppError> {
    let app_dir = rootfs_dir.join("app");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| AppError::Internal(format!("Gagal membuat direktori /app: {e}")))?;
    }

    let target_dir = app_dir.join(&project.name);
    if target_dir.exists() {
        // Jika direktori sudah ada dan memiliki .git, tolak clone agar tidak menimpa data tanpa konfirmasi
        if target_dir.join(".git").exists() {
            return Err(AppError::Validation(format!(
                "Direktori project '{}' sudah ada. Gunakan tombol 'Pull' untuk memperbarui.",
                project.name
            )));
        } else {
            // Jika ada folder sampah kosong / sisa, bersihkan dulu
            let _ = fs::remove_dir_all(&target_dir);
        }
    }

    let _ = app.emit(
        GIT_LOG_EVENT,
        GitLogEvent {
            project_id: project.id,
            line: format!(
                "==> Memulai clone repositori {} (branch: {})...",
                project.repo_url, project.branch
            ),
            stream: "stdout".into(),
        },
    );

    let proot_bin = get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");

    // Persiapkan GIT_ASKPASS jika PAT tersedia
    let (askpass_guard, extra_envs) = if let Some(p) = pat.filter(|s| !s.trim().is_empty()) {
        let guard = AskPassGuard::create(rootfs_dir, p)?;
        let envs = guard.env_vars();
        (Some(guard), envs)
    } else {
        (None, vec![("GIT_TERMINAL_PROMPT".into(), "0".into())])
    };

    let clone_cmd = format!(
        "git clone --depth 1 --branch '{}' '{}' '/app/{}'",
        project.branch, project.repo_url, project.name
    );

    let mut command = proot_exec(
        &proot_bin,
        rootfs_dir,
        &tmp_dir,
        "/app",
        &clone_cmd,
        &extra_envs,
        false,
    );

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| AppError::Internal(format!("Gagal menjalankan proses git clone: {e}")))?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let pat_clone1 = pat.map(|s| s.to_string());
    let pat_clone2 = pat_clone1.clone();
    let app1 = app.clone();
    let app2 = app.clone();
    let pid = project.id;

    let stdout_handle = tokio::spawn(async move {
        stream_command_output(stdout, app1, pid, "stdout", pat_clone1).await
    });
    let stderr_handle = tokio::spawn(async move {
        stream_command_output(stderr, app2, pid, "stderr", pat_clone2).await
    });

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::Internal(format!("Error saat menunggu proses git clone: {e}")))?;

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;
    drop(askpass_guard);

    if !status.success() {
        // Otomatis bersihkan direktori project yang setengah ter-clone agar tidak meninggalkan folder korup
        if target_dir.exists() {
            let _ = fs::remove_dir_all(&target_dir);
            let _ = app.emit(
                GIT_LOG_EVENT,
                GitLogEvent {
                    project_id: project.id,
                    line: format!(
                        "==> [CLEANUP] Direktori /app/{} dibersihkan karena proses clone gagal.",
                        project.name
                    ),
                    stream: "stderr".into(),
                },
            );
        }

        return Err(AppError::Internal(format!(
            "Git clone gagal dengan kode keluar {:?}",
            status.code()
        )));
    }

    let _ = app.emit(
        GIT_LOG_EVENT,
        GitLogEvent {
            project_id: project.id,
            line: format!(
                "==> Git clone untuk '{}' selesai dengan sukses! 🎉",
                project.name
            ),
            stream: "stdout".into(),
        },
    );

    // Dapatkan status terkini setelah berhasil di-clone
    let status_after = get_project_repo_status(rootfs_dir, app_files_dir, &project.name).await;
    Ok(status_after)
}

/// Menjalankan git pull di dalam direktori /app/<project_name>
pub async fn git_pull(
    app: &AppHandle,
    rootfs_dir: &Path,
    app_files_dir: &Path,
    project: &Project,
    pat: Option<&str>,
) -> Result<PullResult, AppError> {
    let project_dir = rootfs_dir.join("app").join(&project.name);
    if !project_dir.exists() || !project_dir.join(".git").exists() {
        return Err(AppError::NotFound(format!(
            "Repositori project '{}' belum di-clone. Lakukan clone terlebih dahulu.",
            project.name
        )));
    }

    let _ = app.emit(
        GIT_LOG_EVENT,
        GitLogEvent {
            project_id: project.id,
            line: format!(
                "==> Memulai git pull origin {} pada /app/{}...",
                project.branch, project.name
            ),
            stream: "stdout".into(),
        },
    );

    let proot_bin = get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");
    let cwd = format!("/app/{}", project.name);

    // Persiapkan GIT_ASKPASS jika PAT tersedia
    let (askpass_guard, extra_envs) = if let Some(p) = pat.filter(|s| !s.trim().is_empty()) {
        let guard = AskPassGuard::create(rootfs_dir, p)?;
        let envs = guard.env_vars();
        (Some(guard), envs)
    } else {
        (None, vec![("GIT_TERMINAL_PROMPT".into(), "0".into())])
    };

    let pull_cmd = format!("git pull origin '{}'", project.branch);

    let mut command = proot_exec(
        &proot_bin,
        rootfs_dir,
        &tmp_dir,
        &cwd,
        &pull_cmd,
        &extra_envs,
        false,
    );

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| AppError::Internal(format!("Gagal menjalankan proses git pull: {e}")))?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let pat_clone1 = pat.map(|s| s.to_string());
    let pat_clone2 = pat_clone1.clone();
    let app1 = app.clone();
    let app2 = app.clone();
    let pid = project.id;

    let stdout_handle = tokio::spawn(async move {
        stream_command_output(stdout, app1, pid, "stdout", pat_clone1).await
    });
    let stderr_handle = tokio::spawn(async move {
        stream_command_output(stderr, app2, pid, "stderr", pat_clone2).await
    });

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::Internal(format!("Error saat menunggu proses git pull: {e}")))?;

    let stdout_lines = stdout_handle.await.unwrap_or_default();
    let _ = stderr_handle.await;
    drop(askpass_guard);

    if !status.success() {
        return Err(AppError::Internal(format!(
            "Git pull gagal dengan status {:?}",
            status.code()
        )));
    }

    let all_stdout = stdout_lines.join(" ");
    let updated =
        !all_stdout.contains("Already up to date") && !all_stdout.contains("Already up-to-date");

    let summary = if updated {
        "Pembaruan kode berhasil diambil dari GitHub.".to_string()
    } else {
        "Repositori sudah berada pada versi terbaru.".to_string()
    };

    let _ = app.emit(
        GIT_LOG_EVENT,
        GitLogEvent {
            project_id: project.id,
            line: format!("==> {}", summary),
            stream: "stdout".into(),
        },
    );

    Ok(PullResult { updated, summary })
}

/// Menjalankan build production di dalam rootfs sandbox via proot (PRD Bagian 9.2 & 13 Fase 5)
pub async fn build_project(
    app: &AppHandle,
    rootfs_dir: &Path,
    app_files_dir: &Path,
    project: &Project,
    build_manager: &ActiveBuildManager,
) -> Result<BuildResult, AppError> {
    let start_time = Instant::now();
    let project_dir = rootfs_dir.join("app").join(&project.name);

    if !project_dir.exists() {
        return Err(AppError::NotFound(format!(
            "Direktori project '{}' tidak ditemukan pada /app/{}. Silakan clone repositori terlebih dahulu.",
            project.name, project.name
        )));
    }

    // Persiapkan direktori log: <app_files_dir>/logs/<project_name>/
    let logs_dir = app_files_dir.join("logs").join(&project.name);
    fs::create_dir_all(&logs_dir)
        .map_err(|e| AppError::Internal(format!("Gagal membuat direktori logs: {e}")))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let log_filename = format!("build-{}.log", timestamp);
    let log_filepath = logs_dir.join(&log_filename);

    let log_file = fs::File::create(&log_filepath)
        .map_err(|e| AppError::Internal(format!("Gagal membuat file log {}: {e}", log_filename)))?;

    // Bangun shell command build yang kompatibel dengan POSIX sh (dash) di rootfs Debian
    let custom_cmd = project.build_cmd.trim();
    let install_logic = "if [ -f package-lock.json ]; then npm ci || npm install; elif [ -f package.json ]; then npm install; else echo '==> Tidak ada package.json, melewati npm install.'; fi";

    let full_cmd =
        if !custom_cmd.is_empty() && custom_cmd != "npm install" && custom_cmd != "npm ci" {
            format!("export NODE_ENV=production; {install_logic} && {custom_cmd}")
        } else {
            format!("export NODE_ENV=production; {install_logic}")
        };

    let start_msg = format!("==> Memulai build untuk project '{}'...", project.name);
    let cmd_msg = format!("==> Perintah: {}", full_cmd);
    let _ = app.emit(
        BUILD_LOG_EVENT,
        BuildLogEvent {
            project_id: project.id,
            line: start_msg.clone(),
            stream: "stdout".into(),
        },
    );
    let _ = app.emit(
        BUILD_LOG_EVENT,
        BuildLogEvent {
            project_id: project.id,
            line: cmd_msg.clone(),
            stream: "stdout".into(),
        },
    );

    let log_file = Arc::new(Mutex::new(log_file));
    let all_lines = Arc::new(Mutex::new(Vec::<String>::new()));

    {
        let mut f = log_file.lock().unwrap();
        let _ = writeln!(f, "{start_msg}");
        let _ = writeln!(f, "{cmd_msg}");
    }
    {
        let mut l = all_lines.lock().unwrap();
        l.push(start_msg);
        l.push(cmd_msg);
    }

    // Persiapkan env vars
    let mut envs = vec![("NODE_ENV".to_string(), "production".to_string())];
    if let Some(ref proj_envs) = project.env_vars {
        for ev in proj_envs {
            envs.push((ev.key.clone(), ev.value.clone()));
        }
    }

    let proot_bin = get_proot_binary();
    let tmp_dir = app_files_dir.join("tmp");
    let cwd = format!("/app/{}", project.name);

    let mut command = proot_exec(
        &proot_bin, rootfs_dir, &tmp_dir, &cwd, &full_cmd, &envs, false,
    );

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| AppError::Internal(format!("Gagal menjalankan proses build: {e}")))?;

    let mut cancel_rx = build_manager.register(project.id);

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let pid = project.id;
    let app_out = app.clone();
    let file_out = Arc::clone(&log_file);
    let lines_out = Arc::clone(&all_lines);
    let stdout_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            {
                let mut f = file_out.lock().unwrap();
                let _ = writeln!(f, "{line}");
            }
            {
                let mut l = lines_out.lock().unwrap();
                l.push(line.clone());
            }
            let _ = app_out.emit(
                BUILD_LOG_EVENT,
                BuildLogEvent {
                    project_id: pid,
                    line,
                    stream: "stdout".into(),
                },
            );
        }
    });

    let app_err = app.clone();
    let file_err = Arc::clone(&log_file);
    let lines_err = Arc::clone(&all_lines);
    let stderr_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            {
                let mut f = file_err.lock().unwrap();
                let _ = writeln!(f, "{line}");
            }
            {
                let mut l = lines_err.lock().unwrap();
                l.push(line.clone());
            }
            let _ = app_err.emit(
                BUILD_LOG_EVENT,
                BuildLogEvent {
                    project_id: pid,
                    line,
                    stream: "stderr".into(),
                },
            );
        }
    });

    // Timeout: 15 menit (900 detik)
    let timeout_duration = std::time::Duration::from_secs(900);
    let sleep_timeout = tokio::time::sleep(timeout_duration);
    tokio::pin!(sleep_timeout);

    let mut was_cancelled = false;
    let mut was_timeout = false;

    let exit_status = tokio::select! {
        res = child.wait() => {
            match res {
                Ok(status) => Some(status),
                Err(e) => {
                    let _ = child.start_kill();
                    return Err(AppError::Internal(format!("Error child build wait: {e}")));
                }
            }
        }
        _ = &mut cancel_rx => {
            was_cancelled = true;
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                }
            }
            let _ = child.start_kill();
            stdout_handle.abort();
            stderr_handle.abort();
            let msg = "==> Build dibatalkan oleh pengguna.".to_string();
            {
                let mut f = log_file.lock().unwrap();
                let _ = writeln!(f, "{msg}");
            }
            {
                let mut l = all_lines.lock().unwrap();
                l.push(msg.clone());
            }
            let _ = app.emit(
                BUILD_LOG_EVENT,
                BuildLogEvent {
                    project_id: project.id,
                    line: msg,
                    stream: "stderr".into(),
                },
            );
            None
        }
        _ = &mut sleep_timeout => {
            was_timeout = true;
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                }
            }
            let _ = child.start_kill();
            stdout_handle.abort();
            stderr_handle.abort();
            let msg = "==> Build dibatalkan karena timeout melebihi 15 menit.".to_string();
            {
                let mut f = log_file.lock().unwrap();
                let _ = writeln!(f, "{msg}");
            }
            {
                let mut l = all_lines.lock().unwrap();
                l.push(msg.clone());
            }
            let _ = app.emit(
                BUILD_LOG_EVENT,
                BuildLogEvent {
                    project_id: project.id,
                    line: msg,
                    stream: "stderr".into(),
                },
            );
            None
        }
    };

    build_manager.unregister(project.id);

    // Tunggu streaming stdout & stderr selesai
    let _ = tokio::join!(stdout_handle, stderr_handle);

    {
        let mut f = log_file.lock().unwrap();
        let _ = f.flush();
    }

    // Rotasi log: simpan maksimal 5 log terbaru
    rotate_build_logs(&logs_dir, 5);

    let elapsed_seconds = start_time.elapsed().as_secs();
    let lines_snapshot = all_lines.lock().unwrap().clone();

    let (success, exit_code, error_summary) = if was_cancelled {
        (false, None, Some("Build dibatalkan oleh pengguna".into()))
    } else if was_timeout {
        (false, None, Some("Build timeout melebihi 15 menit".into()))
    } else if let Some(status) = exit_status {
        let code = status.code();
        let ok = status.success();
        let err_sum = if !ok {
            let start_idx = if lines_snapshot.len() > 50 {
                lines_snapshot.len() - 50
            } else {
                0
            };
            Some(lines_snapshot[start_idx..].join("\n"))
        } else {
            None
        };
        (ok, code, err_sum)
    } else {
        (
            false,
            None,
            Some("Proses build berakhir tanpa status exit".into()),
        )
    };

    let finish_msg = if success {
        format!("==> Build BERHASIL dalam {}s.", elapsed_seconds)
    } else {
        format!("==> Build GAGAL setelah {}s.", elapsed_seconds)
    };

    let _ = app.emit(
        BUILD_LOG_EVENT,
        BuildLogEvent {
            project_id: project.id,
            line: finish_msg,
            stream: if success {
                "stdout".into()
            } else {
                "stderr".into()
            },
        },
    );

    Ok(BuildResult {
        success,
        exit_code,
        log_file: log_filename,
        elapsed_seconds,
        error_summary,
    })
}

/// Rotasi file build log: simpan maksimal `max_keep` file terbaru
pub fn rotate_build_logs(logs_dir: &Path, max_keep: usize) {
    if let Ok(entries) = fs::read_dir(logs_dir) {
        let mut log_files: Vec<std::path::PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("build-") && s.ends_with(".log"))
                        .unwrap_or(false)
            })
            .collect();

        // Urutkan menurun berdasarkan nama file (terbaru di atas)
        log_files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        if log_files.len() > max_keep {
            for old_file in &log_files[max_keep..] {
                let _ = fs::remove_file(old_file);
            }
        }
    }
}

/// Ambil daftar nama file log build untuk suatu project (terbaru di atas)
pub fn list_build_logs(app_files_dir: &Path, project_name: &str) -> Vec<String> {
    let logs_dir = app_files_dir.join("logs").join(project_name);
    if !logs_dir.exists() {
        return Vec::new();
    }
    if let Ok(entries) = fs::read_dir(logs_dir) {
        let mut files: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("build-") && s.ends_with(".log"))
                        .unwrap_or(false)
            })
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();

        files.sort_by(|a, b| b.cmp(a));
        files
    } else {
        Vec::new()
    }
}

/// Baca isi file log build tertentu
pub fn read_build_log(
    app_files_dir: &Path,
    project_name: &str,
    filename: &str,
) -> Result<String, AppError> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::Validation("Nama file log tidak valid".into()));
    }

    let log_path = app_files_dir.join("logs").join(project_name).join(filename);
    if !log_path.exists() {
        return Err(AppError::NotFound(format!(
            "File log '{filename}' tidak ditemukan"
        )));
    }

    fs::read_to_string(&log_path)
        .map_err(|e| AppError::Internal(format!("Gagal membaca file log: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_manager_registration_and_cancellation() {
        let manager = ActiveBuildManager::new();
        let rx = manager.register(42);
        assert!(manager.cancel(42));
        assert!(!manager.cancel(42)); // already cancelled
        drop(rx);
    }

    #[test]
    fn test_log_rotation_and_reading() {
        let temp_dir = std::env::temp_dir().join(format!("test_logs_{}", std::process::id()));
        let project_logs = temp_dir.join("logs").join("sample-app");
        let _ = fs::create_dir_all(&project_logs);

        // Create 8 dummy build log files
        for i in 1..=8 {
            let filename = format!("build-100{}.log", i);
            fs::write(project_logs.join(&filename), format!("log content {i}")).unwrap();
        }

        // Test listing before rotation
        let before = list_build_logs(&temp_dir, "sample-app");
        assert_eq!(before.len(), 8);
        assert_eq!(before[0], "build-1008.log");

        // Rotate to keep only 5 newest
        rotate_build_logs(&project_logs, 5);

        let after = list_build_logs(&temp_dir, "sample-app");
        assert_eq!(after.len(), 5);
        assert_eq!(after[0], "build-1008.log");
        assert_eq!(after[4], "build-1004.log");

        // Test reading valid file
        let content = read_build_log(&temp_dir, "sample-app", "build-1008.log").unwrap();
        assert_eq!(content, "log content 8");

        // Test security: path traversal rejected
        let traversal = read_build_log(&temp_dir, "sample-app", "../evil.log");
        assert!(traversal.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
