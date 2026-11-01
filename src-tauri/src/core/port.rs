// core/port.rs
// Resolusi port, deteksi port listening, validasi bentrok, dan akses lokal (PRD Bagian 8.7 & 13 Fase 9)

use crate::error::{AppError, Result};
use crate::models::{EnvVarItem, Project};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

/// Socket listening entry yang di-parse dari /proc/net/tcp dan /proc/net/tcp6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningSocket {
    pub port: u16,
    pub uid: u32,
    pub inode: u64,
}

/// Mengecek apakah port dapat di-bind pada host (127.0.0.1)
pub fn is_port_bindable(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Mengecek apakah port sedang listening dan menerima koneksi lokal
pub fn is_port_listening(port: u16) -> bool {
    TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(150),
    )
    .is_ok()
}

/// Mencari port bebas mulai dari `start_port` (default 3001)
pub fn find_free_port(start_port: u16, active_ports: &[u16]) -> Option<u16> {
    (start_port..=65535).find(|&port| !active_ports.contains(&port) && is_port_bindable(port))
}

/// Resolusi port target sebelum menjalankan project sesuai aturan PRD 8.7:
/// 1. Jika `port_manual` terisi, pakai nilai itu, dan jangan menyuntikkan `PORT`.
/// 2. Jika env `PORT` ada, pakai nilai itu.
/// 3. Selain itu, pilih port bebas dari 3001 ke atas dan suntikkan `PORT`.
/// 4. Validasi bentrok: jika port sudah dipakai project online lain, tolak dengan pesan jelas.
///
/// Mengembalikan `(target_port, should_inject_env)`.
pub fn resolve_project_port(
    project: &Project,
    env_vars: &[EnvVarItem],
    active_projects: &[(String, u16)], // (project_name, port) yang sedang online
) -> Result<(u16, bool)> {
    // 1. Port Manual
    if let Some(port) = project.port_manual {
        validate_no_port_conflict(port, &project.name, active_projects)?;
        return Ok((port, false));
    }

    // 2. Env PORT
    if let Some(port) = env_vars
        .iter()
        .find(|e| e.key == "PORT")
        .and_then(|e| e.value.trim().parse::<u16>().ok())
    {
        validate_no_port_conflict(port, &project.name, active_projects)?;
        return Ok((port, false));
    }

    // 3. Auto: cari port bebas >= 3001
    let used_ports: Vec<u16> = active_projects.iter().map(|(_, p)| *p).collect();
    let free_port = find_free_port(3001, &used_ports).ok_or_else(|| {
        AppError::Internal("Tidak menemukan port bebas yang tersedia (rentang 3001-65535)".into())
    })?;

    Ok((free_port, true))
}

/// Memastikan port tidak sedang dipakai oleh project aktif lain
pub fn validate_no_port_conflict(
    port: u16,
    current_project_name: &str,
    active_projects: &[(String, u16)],
) -> Result<()> {
    for (name, active_port) in active_projects {
        if *active_port == port && name != current_project_name {
            return Err(AppError::Validation(format!(
                "Port {port} sedang digunakan oleh project '{name}'."
            )));
        }
    }
    Ok(())
}

/// Parsing isi berkas `/proc/net/tcp` atau `/proc/net/tcp6` untuk mengekstrak socket dengan state 0A (TCP_LISTEN)
pub fn parse_listening_sockets_from_proc(content: &str) -> Vec<ListeningSocket> {
    let mut sockets = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Kolom minimal: sl(0), local_address(1), rem_address(2), st(3), ... uid(7), ... inode(9)
        if parts.len() < 10 {
            continue;
        }

        let state_hex = parts[3];
        // 0A adalah konstanta kernel Linux untuk TCP_LISTEN (10 desimal)
        if state_hex != "0A" && state_hex != "a" {
            continue;
        }

        let local_addr = parts[1];
        let port_hex = match local_addr.split(':').nth(1) {
            Some(p) => p,
            None => continue,
        };

        let port = match u16::from_str_radix(port_hex, 16) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let uid = parts[7].parse::<u32>().unwrap_or(0);
        let inode = parts[9].parse::<u64>().unwrap_or(0);

        sockets.push(ListeningSocket { port, uid, inode });
    }

    sockets
}

/// Membaca socket inode milik PID dari `/proc/<pid>/fd/`
pub fn get_process_socket_inodes(pid: u32) -> Vec<u64> {
    let mut inodes = Vec::new();
    let fd_dir = format!("/proc/{pid}/fd");
    let path = Path::new(&fd_dir);

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(target) = std::fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy();
                // Format link socket di procfs Linux: "socket:[123456]"
                if let Some(rest) = target_str.strip_prefix("socket:[") {
                    if let Some(inode_str) = rest.strip_suffix(']') {
                        if let Ok(inode) = inode_str.parse::<u64>() {
                            inodes.push(inode);
                        }
                    }
                }
            }
        }
    }

    inodes
}

/// Mendeteksi port listen riil milik proses PM2 (PRD Bagian 8.7 & 13 Fase 9)
/// 1. Inspeksi socket inode dari `/proc/<pid>/fd` dan cocokkan dengan `/proc/net/tcp` serta `/proc/net/tcp6`.
/// 2. Jika pencocokan inode terhalang pembatasan sandbox OEM, cocokkan socket TCP_LISTEN milik UID aplikasi.
/// 3. Fallback active probe koneksi TCP untuk verifikasi keandalan.
pub fn detect_process_listening_port(
    pid: Option<u32>,
    expected_port: Option<u16>,
) -> Option<u16> {
    // 1. Kumpulkan semua listening sockets dari IPv4 & IPv6 dual-stack di kernel
    let mut all_listening = Vec::new();
    if let Ok(tcp_content) = std::fs::read_to_string("/proc/net/tcp") {
        all_listening.extend(parse_listening_sockets_from_proc(&tcp_content));
    }
    if let Ok(tcp6_content) = std::fs::read_to_string("/proc/net/tcp6") {
        all_listening.extend(parse_listening_sockets_from_proc(&tcp6_content));
    }

    // 2. Jika PID tersedia, coba pencocokan langsung via socket inode
    if let Some(p) = pid {
        let inodes = get_process_socket_inodes(p);
        for inode in inodes {
            if let Some(found) = all_listening.iter().find(|s| s.inode == inode && inode > 0) {
                return Some(found.port);
            }
        }
    }

    // 3. Filter socket listening milik UID aplikasi saat ini
    #[cfg(unix)]
    let app_uid = unsafe { libc::getuid() };
    #[cfg(not(unix))]
    let app_uid = 0;

    let app_listening: Vec<&ListeningSocket> = all_listening
        .iter()
        .filter(|s| app_uid == 0 || s.uid == app_uid)
        .collect();

    // Jika expected_port ada dan ditemukan di daftar listening UID aplikasi
    if let Some(exp) = expected_port {
        if app_listening.iter().any(|s| s.port == exp) {
            return Some(exp);
        }
        // Verifikasi langsung dengan active probe
        if is_port_listening(exp) {
            return Some(exp);
        }
    }

    // Jika ada port listening lain milik aplikasi (misal aplikasi hardcoded ke port lain)
    if let Some(first_sock) = app_listening.first() {
        return Some(first_sock.port);
    }

    // Terakhir: bila expected_port listening
    if let Some(exp) = expected_port {
        if is_port_listening(exp) {
            return Some(exp);
        }
    }

    None
}

/// Membuka URL lokal di browser eksternal via Android Native Intent (ACTION_VIEW) atau platform shell (PRD 9.5 & 13 Fase 9)
#[cfg(target_os = "android")]
pub fn open_in_browser(url: &str) -> Result<()> {
    use jni::objects::{JObject, JValue};

    let vm = unsafe { jni::JavaVM::from_raw(ndk_context::android_context().vm().cast()) }
        .map_err(|e| AppError::Internal(format!("JavaVM error: {e}")))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| AppError::Internal(format!("Attach thread error: {e}")))?;
    let context =
        unsafe { JObject::from_raw(ndk_context::android_context().context().cast()) };

    let url_jstring = env
        .new_string(url)
        .map_err(|e| AppError::Internal(format!("JNI String error: {e}")))?;

    // Uri.parse(url)
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|e| AppError::Internal(format!("Find Uri class error: {e}")))?;
    let uri = env
        .call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&url_jstring)],
        )?
        .l()?;

    // Intent intent = new Intent(Intent.ACTION_VIEW, uri);
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|e| AppError::Internal(format!("Find Intent class error: {e}")))?;
    let action_view = env
        .get_static_field(&intent_class, "ACTION_VIEW", "Ljava/lang/String;")?
        .l()?;

    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action_view), JValue::Object(&uri)],
    )?;

    // intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK); (0x10000000)
    let _ = env.call_method(
        &intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(0x1000_0000)],
    )?;

    // context.startActivity(intent);
    env.call_method(
        &context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )?;

    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn open_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_project(name: &str, port_manual: Option<u16>) -> Project {
        Project {
            id: 1,
            name: name.to_string(),
            repo_url: "https://github.com/example/repo".into(),
            branch: "main".into(),
            build_cmd: "npm run build".into(),
            start_cmd: "npm start".into(),
            port_manual,
            auto_start: false,
            last_build_ok: Some(true),
            last_build_at: None,
            created_at: "2026-10-31T00:00:00Z".into(),
            updated_at: "2026-10-31T00:00:00Z".into(),
            env_vars: None,
        }
    }

    #[test]
    fn test_resolve_port_manual_priority() {
        let proj = dummy_project("app1", Some(8080));
        let envs = vec![EnvVarItem {
            key: "PORT".into(),
            value: "9000".into(),
        }];
        let active = vec![];

        let (port, should_inject) = resolve_project_port(&proj, &envs, &active).unwrap();
        assert_eq!(port, 8080);
        assert!(!should_inject); // Tidak menyuntikkan PORT jika manual terisi
    }

    #[test]
    fn test_resolve_port_env_priority_over_auto() {
        let proj = dummy_project("app2", None);
        let envs = vec![EnvVarItem {
            key: "PORT".into(),
            value: "5000".into(),
        }];
        let active = vec![];

        let (port, should_inject) = resolve_project_port(&proj, &envs, &active).unwrap();
        assert_eq!(port, 5000);
        assert!(!should_inject); // Sudah ada di env vars
    }

    #[test]
    fn test_resolve_port_auto_assigns_free_port() {
        let proj = dummy_project("app3", None);
        let envs = vec![];
        let active = vec![("other".to_string(), 3001)];

        let (port, should_inject) = resolve_project_port(&proj, &envs, &active).unwrap();
        assert!(port >= 3001);
        assert_ne!(port, 3001); // 3001 sudah terpakai
        assert!(should_inject); // Harus menyuntikkan PORT ke env
    }

    #[test]
    fn test_port_conflict_validation() {
        let proj = dummy_project("app4", Some(3000));
        let envs = vec![];
        let active = vec![("running-service".to_string(), 3000)];

        let err = resolve_project_port(&proj, &envs, &active).unwrap_err();
        assert!(err.to_string().contains("running-service"));
        assert!(err.to_string().contains("Port 3000 sedang digunakan"));
    }

    #[test]
    fn test_parse_proc_net_tcp() {
        let sample = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n   0: 0100007F:0BB9 00000000:0000 0A 00000000:00000000 00:00000000 00000000 10438        0 18738483 1 0000000000000000 99 0 0 10 0\n   1: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000 10438        0 19999999 1 0000000000000000 99 0 0 10 0\n   2: 0364A8C0:DCE2 22C32617:01BB 04 00000001:00000000 00:00000000 00000000     0        0 0 1 0000000000000000 21 3 0 10 -1";

        let sockets = parse_listening_sockets_from_proc(sample);
        assert_eq!(sockets.len(), 2);
        assert_eq!(sockets[0].port, 3001); // 0x0BB9 = 3001
        assert_eq!(sockets[0].uid, 10438);
        assert_eq!(sockets[0].inode, 18738483);
        assert_eq!(sockets[1].port, 8080); // 0x1F90 = 8080
    }

    #[test]
    fn test_parse_proc_net_tcp6() {
        let sample = "  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n   0: 00000000000000000000000000000000:0BB9 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000 10438        0 18738483 1 0000000000000000 99 0 0 10 0";

        let sockets = parse_listening_sockets_from_proc(sample);
        assert_eq!(sockets.len(), 1);
        assert_eq!(sockets[0].port, 3001);
        assert_eq!(sockets[0].inode, 18738483);
    }
}
