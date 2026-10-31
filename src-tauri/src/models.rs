use serde::{Deserialize, Serialize};

/// Project entity stored in SQLite (PRD Section 8.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
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
    pub updated_at: String,
    pub env_vars: Option<Vec<EnvVarItem>>,
}

/// Environment variable key-value pair for a project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvVarItem {
    pub key: String,
    pub value: String,
}

/// Input payload for creating a new Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub repo_url: String,
    pub branch: Option<String>,
    pub build_cmd: Option<String>,
    pub start_cmd: Option<String>,
    pub port_manual: Option<u16>,
    pub auto_start: Option<bool>,
    pub env_vars: Option<Vec<EnvVarItem>>,
}

/// Input payload for updating an existing Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub branch: Option<String>,
    pub build_cmd: Option<String>,
    pub start_cmd: Option<String>,
    pub port_manual: Option<u16>,
    pub auto_start: Option<bool>,
    pub env_vars: Option<Vec<EnvVarItem>>,
}

/// Setting key-value pair
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingItem {
    pub key: String,
    pub value: String,
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

/// Setup Status model for Fase 2 (PRD Section 13 Fase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub is_ready: bool,
    pub version: Option<String>,
    pub free_space_mb: u64,
    pub required_space_mb: u64,
}

/// Setup Progress Event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupProgress {
    pub step: String,
    pub percent: u8,
    pub message: String,
}

/// Setup Verification Result (PRD Section 13 Fase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupVerificationResult {
    pub success: bool,
    pub node_version: String,
    pub pm2_version: String,
    pub cloudflared_version: String,
    pub git_version: String,
    pub disk_used_mb: u64,
    pub error: Option<String>,
}
