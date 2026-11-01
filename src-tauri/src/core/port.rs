// core/port.rs
// Resolusi port, deteksi port listening, dan validasi bentrok port (PRD Bagian 8.7)

use crate::error::{AppError, Result};
use crate::models::{EnvVarItem, Project};
use std::net::TcpListener;

/// Mengecek apakah port dapat di-bind pada host (127.0.0.1)
pub fn is_port_bindable(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Mencari port bebas mulai dari `start_port` (default 3001)
pub fn find_free_port(start_port: u16, active_ports: &[u16]) -> Option<u16> {
    (start_port..=65535).find(|&port| !active_ports.contains(&port) && is_port_bindable(port))
}

/// Resolusi port target sebelum menjalankan project sesuai aturan PRD 8.7:
/// 1. Jika `port_manual` terisi, pakai nilai itu, dan jangan menyuntikkan `PORT`.
/// 2. Jika env `PORT` ada, pakai nilai itu.
/// 3. Selain itu, pilih port bebas dari 3001 ke atas dan suntikkan `PORT`.
/// 5. Validasi bentrok: jika port sudah dipakai project online lain, tolak dengan pesan jelas.
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
fn validate_no_port_conflict(
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
}
