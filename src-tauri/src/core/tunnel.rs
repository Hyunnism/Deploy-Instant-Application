// core/tunnel.rs
// Quick tunnel cloudflared & parsing url trycloudflare.com (PRD Bagian 13 Fase 10)

use crate::core::exec::proot_exec;
use crate::error::{AppError, Result};
use crate::models::{ProjectTunnelInfo, TunnelStatus};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

/// Ekstraksi URL Quick Tunnel `https://[subdomain].trycloudflare.com` dari teks/log output
pub fn parse_cloudflare_url(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        // Cari substring https:// dan .trycloudflare.com
        if let Some(start_idx) = trimmed.find("https://") {
            let candidate = &trimmed[start_idx..];
            if let Some(end_domain_idx) = candidate.find(".trycloudflare.com") {
                let end_idx = end_domain_idx + ".trycloudflare.com".len();
                let url = &candidate[..end_idx];
                // Validasi bahwa karakter antara https:// dan .trycloudflare.com hanya a-z, 0-9, -
                let subdomain = &url["https://".len()..end_domain_idx];
                if !subdomain.is_empty()
                    && subdomain
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-')
                {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

/// Instance internal tunnel yang sedang berjalan
struct ActiveTunnel {
    project_id: i64,
    port: u16,
    url: String,
    kill_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Manager Quick Tunnel berbasis state per-project (PRD Bagian 13 Fase 10)
#[derive(Clone)]
pub struct TunnelManager {
    tunnels: Arc<Mutex<HashMap<i64, ActiveTunnel>>>,
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Mendapatkan status tunnel project saat ini
    pub async fn get_tunnel_status(&self, project_id: i64) -> Option<ProjectTunnelInfo> {
        let lock = self.tunnels.lock().await;
        lock.get(&project_id).map(|t| ProjectTunnelInfo {
            project_id: t.project_id,
            port: t.port,
            url: Some(t.url.clone()),
            status: TunnelStatus::Active,
            error: None,
        })
    }

    /// Mendapatkan seluruh daftar tunnel aktif
    pub async fn list_active_tunnels(&self) -> Vec<ProjectTunnelInfo> {
        let lock = self.tunnels.lock().await;
        lock.values()
            .map(|t| ProjectTunnelInfo {
                project_id: t.project_id,
                port: t.port,
                url: Some(t.url.clone()),
                status: TunnelStatus::Active,
                error: None,
            })
            .collect()
    }

    /// Memulai Quick Tunnel untuk project dan port tertentu
    pub async fn start_tunnel(
        &self,
        app: &AppHandle,
        project_id: i64,
        port: u16,
        proot_bin: &Path,
        rootfs_dir: &Path,
        tmp_dir: &Path,
    ) -> Result<ProjectTunnelInfo> {
        // Cek jika sudah aktif
        {
            let mut lock = self.tunnels.lock().await;
            if let Some(existing) = lock.get(&project_id) {
                if existing.port == port {
                    return Ok(ProjectTunnelInfo {
                        project_id,
                        port,
                        url: Some(existing.url.clone()),
                        status: TunnelStatus::Active,
                        error: None,
                    });
                }
                // Port berubah, hentikan yang lama dulu
                if let Some(mut old) = lock.remove(&project_id) {
                    if let Some(tx) = old.kill_tx.take() {
                        let _ = tx.send(());
                    }
                }
            }
        }

        // Emit status Starting
        let _ = app.emit(
            "tunnel:status",
            ProjectTunnelInfo {
                project_id,
                port,
                url: None,
                status: TunnelStatus::Starting,
                error: None,
            },
        );

        let cmd = format!("exec cloudflared tunnel --url http://127.0.0.1:{port} --no-autoupdate --metrics 127.0.0.1:0 --protocol http2");

        // Eksekusi proot command
        let mut child = proot_exec(
            proot_bin,
            rootfs_dir,
            tmp_dir,
            "/",
            &cmd,
            &[],
            false,
            true, // kill_on_exit
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(format!("Gagal menjalankan cloudflared: {e}")))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::Internal("Tidak dapat membuka stderr cloudflared".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::Internal("Tidak dapat membuka stdout cloudflared".into()))?;

        let (url_tx, mut url_rx) = tokio::sync::mpsc::channel::<String>(1);
        let last_lines = Arc::new(Mutex::new(Vec::<String>::new()));

        // Task pembaca stderr (cloudflared menulis log ke stderr)
        let url_tx_err = url_tx.clone();
        let last_lines_err = last_lines.clone();
        let mut err_reader = BufReader::new(stderr).lines();
        let stderr_handle = tokio::spawn(async move {
            let mut captured_url = None;
            while let Ok(Some(line)) = err_reader.next_line().await {
                eprintln!("[cloudflared stderr] {}", line);
                {
                    let mut lines = last_lines_err.lock().await;
                    if lines.len() >= 8 {
                        lines.remove(0);
                    }
                    lines.push(line.clone());
                }
                if captured_url.is_none() {
                    if let Some(url) = parse_cloudflare_url(&line) {
                        captured_url = Some(url.clone());
                        let _ = url_tx_err.send(url).await;
                    }
                }
            }
        });

        // Task pembaca stdout
        let last_lines_out = last_lines.clone();
        let mut out_reader = BufReader::new(stdout).lines();
        let stdout_handle = tokio::spawn(async move {
            let mut captured_url = None;
            while let Ok(Some(line)) = out_reader.next_line().await {
                eprintln!("[cloudflared stdout] {}", line);
                {
                    let mut lines = last_lines_out.lock().await;
                    if lines.len() >= 8 {
                        lines.remove(0);
                    }
                    lines.push(line.clone());
                }
                if captured_url.is_none() {
                    if let Some(url) = parse_cloudflare_url(&line) {
                        captured_url = Some(url.clone());
                        let _ = url_tx.send(url).await;
                    }
                }
            }
        });

        // Tunggu URL keluar dengan timeout 30 detik (PRD Bagian 13 Fase 10)
        let detected_url = tokio::select! {
            url_opt = url_rx.recv() => {
                match url_opt {
                    Some(u) => Ok(u),
                    None => {
                        let lines = last_lines.lock().await;
                        let detail = if lines.is_empty() {
                            "Output cloudflared tertutup tanpa log".to_string()
                        } else {
                            lines.join(" | ")
                        };
                        Err(AppError::Internal(format!("Output cloudflared tertutup sebelum URL ditemukan: {detail}")))
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                let lines = last_lines.lock().await;
                let detail = if lines.is_empty() {
                    "Tidak ada log dari cloudflared".to_string()
                } else {
                    lines.join(" | ")
                };
                Err(AppError::Internal(format!("Batas waktu 30 detik terlampaui saat menunggu Quick Tunnel siap: {detail}")))
            }
        };

        let url = match detected_url {
            Ok(u) => u,
            Err(e) => {
                // Bunuh seluruh process group jika gagal / timeout
                #[cfg(unix)]
                if let Some(pid) = child.id() {
                    unsafe {
                        libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                    }
                }
                let _ = child.kill().await;
                stderr_handle.abort();
                stdout_handle.abort();

                let _ = app.emit(
                    "tunnel:status",
                    ProjectTunnelInfo {
                        project_id,
                        port,
                        url: None,
                        status: TunnelStatus::Error,
                        error: Some(e.to_string()),
                    },
                );
                return Err(e);
            }
        };

        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();

        // Simpan ke daftar tunnel aktif
        {
            let mut lock = self.tunnels.lock().await;
            lock.insert(
                project_id,
                ActiveTunnel {
                    project_id,
                    port,
                    url: url.clone(),
                    kill_tx: Some(kill_tx),
                },
            );
        }

        let info = ProjectTunnelInfo {
            project_id,
            port,
            url: Some(url.clone()),
            status: TunnelStatus::Active,
            error: None,
        };

        // Emit event tunnel:status ke UI
        let _ = app.emit("tunnel:status", info.clone());

        // Spawn watcher task di latar belakang untuk mendeteksi stop atau kematian proses mendadak
        let tunnels_clone = self.tunnels.clone();
        let app_handle = app.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = kill_rx => {
                    // Diminta stop secara eksplisit: bunuh seluruh process group
                    #[cfg(unix)]
                    if let Some(pid) = child.id() {
                        unsafe {
                            libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                        }
                    }
                    let _ = child.kill().await;
                }
                status = child.wait() => {
                    // Proses mati sendiri (koneksi putus / crash)
                    let err_msg = match status {
                        Ok(s) => format!("Proses tunnel keluar dengan status: {s}"),
                        Err(e) => format!("Error pada proses tunnel: {e}"),
                    };
                    let mut lock = tunnels_clone.lock().await;
                    lock.remove(&project_id);

                    let _ = app_handle.emit(
                        "tunnel:status",
                        ProjectTunnelInfo {
                            project_id,
                            port,
                            url: None,
                            status: TunnelStatus::Stopped,
                            error: Some(err_msg),
                        },
                    );
                }
            }
            stderr_handle.abort();
            stdout_handle.abort();
        });

        Ok(info)
    }

    /// Menghentikan Quick Tunnel untuk project tertentu
    pub async fn stop_tunnel(&self, app: &AppHandle, project_id: i64) -> Result<()> {
        let mut lock = self.tunnels.lock().await;
        if let Some(mut active) = lock.remove(&project_id) {
            if let Some(tx) = active.kill_tx.take() {
                let _ = tx.send(());
            }
            let _ = app.emit(
                "tunnel:status",
                ProjectTunnelInfo {
                    project_id,
                    port: active.port,
                    url: None,
                    status: TunnelStatus::Stopped,
                    error: None,
                },
            );
        }
        Ok(())
    }

    /// Menghentikan semua tunnel aktif (misal saat stop semua atau app shutdown)
    pub async fn stop_all(&self, app: &AppHandle) -> Result<()> {
        let mut lock = self.tunnels.lock().await;
        for (project_id, mut active) in lock.drain() {
            if let Some(tx) = active.kill_tx.take() {
                let _ = tx.send(());
            }
            let _ = app.emit(
                "tunnel:status",
                ProjectTunnelInfo {
                    project_id,
                    port: active.port,
                    url: None,
                    status: TunnelStatus::Stopped,
                    error: None,
                },
            );
        }
        Ok(())
    }
}

/// Berbagi teks/URL via Android Native Chooser Share Sheet (PRD Bagian 13 Fase 10)
#[cfg(target_os = "android")]
pub fn share_url_native(url: &str, title: &str) -> Result<()> {
    use jni::objects::{JObject, JValue};

    let vm = unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) }
        .map_err(|e| AppError::Internal(format!("JavaVM error: {e}")))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| AppError::Internal(format!("Attach thread error: {e}")))?;
    let context = unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };

    let url_jstring = env
        .new_string(url)
        .map_err(|e| AppError::Internal(format!("JNI String error: {e}")))?;
    let title_jstring = env
        .new_string(title)
        .map_err(|e| AppError::Internal(format!("JNI String error: {e}")))?;

    // Intent sendIntent = new Intent(Intent.ACTION_SEND);
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|e| AppError::Internal(format!("Find Intent class error: {e}")))?;
    let action_send = env
        .get_static_field(&intent_class, "ACTION_SEND", "Ljava/lang/String;")?
        .l()?;

    let send_intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action_send)],
    )?;

    // sendIntent.setType("text/plain");
    let mime_type = env.new_string("text/plain")?;
    env.call_method(
        &send_intent,
        "setType",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&mime_type)],
    )?;

    // sendIntent.putExtra(Intent.EXTRA_TEXT, url);
    let extra_text = env
        .get_static_field(&intent_class, "EXTRA_TEXT", "Ljava/lang/String;")?
        .l()?;
    env.call_method(
        &send_intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[
            JValue::Object(&extra_text),
            JValue::Object(&url_jstring),
        ],
    )?;

    // Intent chooser = Intent.createChooser(sendIntent, title);
    let chooser = env
        .call_static_method(
            &intent_class,
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[
                JValue::Object(&send_intent),
                JValue::Object(&title_jstring),
            ],
        )?
        .l()?;

    // chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
    let flag_new_task = env.get_static_field(
        &intent_class,
        "FLAG_ACTIVITY_NEW_TASK",
        "I",
    )?.i()?;
    env.call_method(
        &chooser,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(flag_new_task)],
    )?;

    // context.startActivity(chooser);
    env.call_method(
        &context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&chooser)],
    )?;

    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn share_url_native(url: &str, title: &str) -> Result<()> {
    println!("[share_url_native host fallback] Title: {}, URL: {}", title, url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cloudflare_url_standard() {
        let sample = "2026-11-02T09:30:00Z INF |  https://alpha-beta-123.trycloudflare.com  |\n";
        let url = parse_cloudflare_url(sample);
        assert_eq!(
            url,
            Some("https://alpha-beta-123.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn test_parse_cloudflare_url_in_box_drawing() {
        let sample = r#"
+--------------------------------------------------------------------------------------------+
|  Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):  |
|  https://curly-pond-lake-forest.trycloudflare.com                                          |
+--------------------------------------------------------------------------------------------+
"#;
        let url = parse_cloudflare_url(sample);
        assert_eq!(
            url,
            Some("https://curly-pond-lake-forest.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn test_parse_cloudflare_url_invalid() {
        let sample = "Connecting to edge servers... no url here";
        assert_eq!(parse_cloudflare_url(sample), None);

        let invalid_domain = "https://fakecloudflare.com/sub";
        assert_eq!(parse_cloudflare_url(invalid_domain), None);
    }

    #[tokio::test]
    async fn test_tunnel_manager_state_lifecycle() {
        let mgr = TunnelManager::new();
        assert!(mgr.list_active_tunnels().await.is_empty());
        assert!(mgr.get_tunnel_status(1).await.is_none());
    }
}
