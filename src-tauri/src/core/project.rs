// core/project.rs
// Operasi Git Clone & Git Pull di dalam rootfs sandbox via proot (PRD Bagian 13 Fase 4)

use crate::core::exec::proot_exec;
use crate::core::rootfs::get_proot_binary;
use crate::error::AppError;
use crate::models::{GitLogEvent, Project, PullResult, RepoStatus};
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

pub const GIT_LOG_EVENT: &str = "git:log";

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
