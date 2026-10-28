use serde::{Deserialize, Serialize};

/// Metadata model for a Project (PRD Section 8.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub build_cmd: String,
    pub start_cmd: String,
    pub port_manual: Option<u16>,
    pub auto_start: bool,
    pub last_build_ok: Option<bool>,
    pub last_build_at: Option<String>,
    pub created_at: String,
}

/// PM2 Process Status summary (PRD Section 8.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pm2Status {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: u64,
    pub restarts: u32,
    pub cpu: f32,
    pub memory: u64,
}

/// Probe Result model for Fase 1 (PRD Section 13 Fase 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub success: bool,
    pub uname: String,
    pub node_version: String,
    pub git_version: String,
    pub network_check: String,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}
