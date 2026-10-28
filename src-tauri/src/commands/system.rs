use serde::Serialize;

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
