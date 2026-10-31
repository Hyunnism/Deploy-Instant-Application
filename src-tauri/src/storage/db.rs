// storage/db.rs
// SQLite database + migrasi + CRUD Project & Settings (Fase 3 & Fase 6)

use crate::error::AppError;
use crate::models::{CreateProjectInput, EnvVarItem, Project, SettingItem, UpdateProjectInput};
use crate::storage::secrets::{decrypt_secret, encrypt_secret, get_or_create_master_key};
use aes_gcm::{
    Aes256Gcm, Key,
    aead::{KeyInit, OsRng},
};
use rusqlite::{Connection, OptionalExtension, params};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
    master_key: Key<Aes256Gcm>,
}

impl Database {
    /// Inisialisasi database SQLite pada path yang diberikan dan jalankan migrasi schema
    pub fn open(db_path: &Path, key_dir: &Path) -> Result<Self, AppError> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Gagal membuat direktori db: {e}")))?;
        }

        let master_key = get_or_create_master_key(key_dir)?;

        let conn = Connection::open(db_path)
            .map_err(|e| AppError::Internal(format!("Gagal membuka koneksi SQLite: {e}")))?;

        let db = Self {
            conn: Mutex::new(conn),
            master_key,
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Inisialisasi database SQLite in-memory (berguna untuk unit test)
    pub fn open_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::Internal(format!("Gagal membuka SQLite in-memory: {e}")))?;

        let master_key = Aes256Gcm::generate_key(OsRng);

        let db = Self {
            conn: Mutex::new(conn),
            master_key,
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Enkripsi nilai environment variable dengan AES-256-GCM
    pub fn encrypt_env_value(&self, val: &str) -> Result<String, AppError> {
        encrypt_secret(&self.master_key, val)
    }

    /// Dekripsi nilai environment variable terenkripsi.
    /// Bila data lama belum terenkripsi, fallback ke nilai plaintext asli.
    pub fn decrypt_env_value(&self, val: &str) -> String {
        decrypt_secret(&self.master_key, val).unwrap_or_else(|_| val.to_string())
    }

    /// Eksekusi migrasi schema tabel PRD Section 8.4
    pub fn init_schema(&self) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock poisoning: {e}")))?;

        // Optimasi SQLite untuk embedded Android
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(|e| AppError::Internal(format!("Gagal set PRAGMA SQLite: {e}")))?;

        // Tabel Projects
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                repo_url TEXT NOT NULL,
                branch TEXT NOT NULL DEFAULT 'main',
                build_cmd TEXT NOT NULL DEFAULT 'npm install',
                start_cmd TEXT NOT NULL DEFAULT 'npm start',
                port_manual INTEGER,
                auto_start BOOLEAN NOT NULL DEFAULT 1,
                last_build_ok BOOLEAN,
                last_build_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
            [],
        )
        .map_err(|e| AppError::Internal(format!("Gagal membuat tabel projects: {e}")))?;

        // Tabel Environment Variables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS env_vars (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                UNIQUE(project_id, key)
            );",
            [],
        )
        .map_err(|e| AppError::Internal(format!("Gagal membuat tabel env_vars: {e}")))?;

        // Tabel Settings (key-value)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| AppError::Internal(format!("Gagal membuat tabel settings: {e}")))?;

        // Nilai default untuk pengaturan jika belum ada
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'id');",
            [],
        )
        .map_err(|e| AppError::Internal(format!("Gagal inisialisasi default settings: {e}")))?;

        Ok(())
    }

    /// Validasi input nama project (PRD 12A.4: No Shell Injection, Alphanumeric + dash/underscore)
    pub fn validate_project_name(name: &str) -> Result<(), AppError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "Nama project tidak boleh kosong".into(),
            ));
        }
        if trimmed.len() > 50 {
            return Err(AppError::Validation(
                "Nama project maksimal 50 karakter".into(),
            ));
        }
        // Validasi karakter aman: a-z, A-Z, 0-9, -, _
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::Validation(
                "Nama project hanya boleh berisi huruf, angka, strip (-), atau underscore (_)"
                    .into(),
            ));
        }
        // Tolak reserved names atau traversal
        if trimmed == "." || trimmed == ".." {
            return Err(AppError::Validation(
                "Nama project tidak boleh berupa '.' atau '..'".into(),
            ));
        }
        Ok(())
    }

    /// Validasi repository URL (Git HTTPS / SSH)
    pub fn validate_repo_url(url: &str) -> Result<(), AppError> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "Repository URL tidak boleh kosong".into(),
            ));
        }
        if trimmed.len() > 500 {
            return Err(AppError::Validation(
                "Repository URL terlalu panjang (maks 500 karakter)".into(),
            ));
        }
        // Tolak karakter spasi atau shell operator berbahaya
        if trimmed
            .chars()
            .any(|c| c.is_whitespace() || c == ';' || c == '&' || c == '|' || c == '`' || c == '$')
        {
            return Err(AppError::Validation(
                "Repository URL mengandung karakter tidak valid".into(),
            ));
        }
        let valid_prefix = trimmed.starts_with("https://")
            || trimmed.starts_with("http://")
            || trimmed.starts_with("git@");
        if !valid_prefix {
            return Err(AppError::Validation(
                "Repository URL harus diawali dengan https://, http://, atau git@".into(),
            ));
        }
        Ok(())
    }

    /// Validasi nama git branch
    pub fn validate_branch(branch: &str) -> Result<(), AppError> {
        let trimmed = branch.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("Branch tidak boleh kosong".into()));
        }
        if trimmed.len() > 100 {
            return Err(AppError::Validation("Branch maksimal 100 karakter".into()));
        }
        if trimmed.contains("..") || trimmed.starts_with('/') || trimmed.ends_with('/') {
            return Err(AppError::Validation("Format branch tidak valid".into()));
        }
        if !trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || c == '-'
                || c == '_'
                || c == '.'
                || c == '/'
                || c == '@'
                || c == '#'
        }) {
            return Err(AppError::Validation(
                "Branch mengandung karakter tidak diizinkan".into(),
            ));
        }
        Ok(())
    }

    /// Validasi port manual (1024 - 65535 untuk non-root port)
    pub fn validate_port(port: u16) -> Result<(), AppError> {
        if port < 1024 {
            return Err(AppError::Validation(
                "Port harus berada pada rentang 1024 hingga 65535 (port non-root)".into(),
            ));
        }
        Ok(())
    }

    /// Validasi key env var
    pub fn validate_env_key(key: &str) -> Result<(), AppError> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "Key environment variable tidak boleh kosong".into(),
            ));
        }
        let first = trimmed.chars().next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(AppError::Validation(
                "Key env var harus diawali dengan huruf atau underscore".into(),
            ));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::Validation(
                "Key env var hanya boleh berisi huruf, angka, dan underscore".into(),
            ));
        }
        Ok(())
    }

    /// Buat project baru
    pub fn create_project(&self, input: CreateProjectInput) -> Result<Project, AppError> {
        let name = input.name.trim().to_string();
        Self::validate_project_name(&name)?;

        let repo_url = input.repo_url.trim().to_string();
        Self::validate_repo_url(&repo_url)?;

        let branch = input.branch.unwrap_or_else(|| "main".to_string());
        let branch = branch.trim().to_string();
        Self::validate_branch(&branch)?;

        let build_cmd = input
            .build_cmd
            .unwrap_or_else(|| "npm install".to_string())
            .trim()
            .to_string();
        let start_cmd = input
            .start_cmd
            .unwrap_or_else(|| "npm start".to_string())
            .trim()
            .to_string();

        if let Some(port) = input.port_manual {
            Self::validate_port(port)?;
        }

        if let Some(ref envs) = input.env_vars {
            for item in envs {
                Self::validate_env_key(&item.key)?;
            }
        }

        let auto_start = input.auto_start.unwrap_or(true);

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let tx = conn
            .transaction()
            .map_err(|e| AppError::Internal(format!("Gagal memulai transaksi: {e}")))?;

        let insert_res = tx.execute(
            "INSERT INTO projects (name, repo_url, branch, build_cmd, start_cmd, port_manual, auto_start)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                name,
                repo_url,
                branch,
                build_cmd,
                start_cmd,
                input.port_manual,
                auto_start,
            ],
        );

        match insert_res {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(AppError::Validation(format!(
                    "Project dengan nama '{name}' sudah ada"
                )));
            }
            Err(e) => return Err(AppError::Internal(format!("Gagal menyimpan project: {e}"))),
        }

        let project_id = tx.last_insert_rowid();

        if let Some(envs) = &input.env_vars {
            for item in envs {
                let enc_val = self.encrypt_env_value(&item.value)?;
                tx.execute(
                    "INSERT INTO env_vars (project_id, key, value) VALUES (?1, ?2, ?3);",
                    params![project_id, item.key.trim(), enc_val],
                )
                .map_err(|e| AppError::Internal(format!("Gagal menyimpan env var: {e}")))?;
            }
        }

        tx.commit()
            .map_err(|e| AppError::Internal(format!("Gagal commit transaksi: {e}")))?;

        drop(conn);
        self.get_project(project_id)
    }

    /// Ambil daftar semua project
    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, repo_url, branch, build_cmd, start_cmd, port_manual,
                        auto_start, last_build_ok, last_build_at, created_at, updated_at
                 FROM projects
                 ORDER BY id DESC;",
            )
            .map_err(|e| AppError::Internal(format!("Prepare query projects error: {e}")))?;

        let project_rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<u16>>(6)?,
                    row.get::<_, bool>(7)?,
                    row.get::<_, Option<bool>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                ))
            })
            .map_err(|e| AppError::Internal(format!("Query projects error: {e}")))?;

        let mut projects = Vec::new();
        for res in project_rows {
            let (
                id,
                name,
                repo_url,
                branch,
                build_cmd,
                start_cmd,
                port_manual,
                auto_start,
                last_build_ok,
                last_build_at,
                created_at,
                updated_at,
            ) = res.map_err(|e| AppError::Internal(format!("Row error: {e}")))?;

            // Ambil env_vars untuk project ini
            let mut env_stmt = conn
                .prepare("SELECT key, value FROM env_vars WHERE project_id = ?1 ORDER BY key ASC;")
                .map_err(|e| AppError::Internal(format!("Prepare env_vars error: {e}")))?;

            let env_iter = env_stmt
                .query_map(params![id], |r| {
                    Ok(EnvVarItem {
                        key: r.get(0)?,
                        value: r.get(1)?,
                    })
                })
                .map_err(|e| AppError::Internal(format!("Query env_vars error: {e}")))?;

            let mut env_vars = Vec::new();
            for item in env_iter {
                let raw_item =
                    item.map_err(|e| AppError::Internal(format!("Env row error: {e}")))?;
                let decrypted_val = self.decrypt_env_value(&raw_item.value);
                env_vars.push(EnvVarItem {
                    key: raw_item.key,
                    value: decrypted_val,
                });
            }

            projects.push(Project {
                id,
                name,
                repo_url,
                branch,
                build_cmd,
                start_cmd,
                port_manual,
                auto_start,
                last_build_ok,
                last_build_at,
                created_at,
                updated_at,
                env_vars: if env_vars.is_empty() {
                    None
                } else {
                    Some(env_vars)
                },
            });
        }

        Ok(projects)
    }

    /// Ambil satu project berdasarkan ID
    pub fn get_project(&self, id: i64) -> Result<Project, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, repo_url, branch, build_cmd, start_cmd, port_manual,
                        auto_start, last_build_ok, last_build_at, created_at, updated_at
                 FROM projects
                 WHERE id = ?1;",
            )
            .map_err(|e| AppError::Internal(format!("Prepare query project error: {e}")))?;

        let row = stmt
            .query_row(params![id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<u16>>(6)?,
                    row.get::<_, bool>(7)?,
                    row.get::<_, Option<bool>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                ))
            })
            .optional()
            .map_err(|e| AppError::Internal(format!("Query project error: {e}")))?;

        let (
            id,
            name,
            repo_url,
            branch,
            build_cmd,
            start_cmd,
            port_manual,
            auto_start,
            last_build_ok,
            last_build_at,
            created_at,
            updated_at,
        ) = match row {
            Some(data) => data,
            None => {
                return Err(AppError::NotFound(format!(
                    "Project dengan ID {id} tidak ditemukan"
                )));
            }
        };

        // Query env_vars
        let mut env_stmt = conn
            .prepare("SELECT key, value FROM env_vars WHERE project_id = ?1 ORDER BY key ASC;")
            .map_err(|e| AppError::Internal(format!("Prepare env_vars error: {e}")))?;

        let env_iter = env_stmt
            .query_map(params![id], |r| {
                Ok(EnvVarItem {
                    key: r.get(0)?,
                    value: r.get(1)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Query env_vars error: {e}")))?;

        let mut env_vars = Vec::new();
        for item in env_iter {
            let raw_item = item.map_err(|e| AppError::Internal(format!("Env row error: {e}")))?;
            let decrypted_val = self.decrypt_env_value(&raw_item.value);
            env_vars.push(EnvVarItem {
                key: raw_item.key,
                value: decrypted_val,
            });
        }

        Ok(Project {
            id,
            name,
            repo_url,
            branch,
            build_cmd,
            start_cmd,
            port_manual,
            auto_start,
            last_build_ok,
            last_build_at,
            created_at,
            updated_at,
            env_vars: if env_vars.is_empty() {
                None
            } else {
                Some(env_vars)
            },
        })
    }

    /// Update project
    pub fn update_project(&self, id: i64, input: UpdateProjectInput) -> Result<Project, AppError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        // Pastikan project ada
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1);",
                params![id],
                |r| r.get(0),
            )
            .map_err(|e| AppError::Internal(format!("Check exists error: {e}")))?;

        if !exists {
            return Err(AppError::NotFound(format!(
                "Project dengan ID {id} tidak ditemukan"
            )));
        }

        if let Some(ref name) = input.name {
            Self::validate_project_name(name)?;
        }
        if let Some(ref url) = input.repo_url {
            Self::validate_repo_url(url)?;
        }
        if let Some(ref branch) = input.branch {
            Self::validate_branch(branch)?;
        }
        if let Some(port) = input.port_manual {
            Self::validate_port(port)?;
        }
        if let Some(ref envs) = input.env_vars {
            for item in envs {
                Self::validate_env_key(&item.key)?;
            }
        }

        let tx = conn
            .transaction()
            .map_err(|e| AppError::Internal(format!("Gagal mulai transaksi update: {e}")))?;

        // Update fields yang diberikan
        if let Some(name) = &input.name {
            let res = tx.execute(
                "UPDATE projects SET name = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![name.trim(), id],
            );
            match res {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    return Err(AppError::Validation(format!(
                        "Project dengan nama '{name}' sudah ada"
                    )));
                }
                Err(e) => {
                    return Err(AppError::Internal(format!(
                        "Gagal update nama project: {e}"
                    )));
                }
            }
        }

        if let Some(repo_url) = &input.repo_url {
            tx.execute(
                "UPDATE projects SET repo_url = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![repo_url.trim(), id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update repo_url: {e}")))?;
        }

        if let Some(branch) = &input.branch {
            tx.execute(
                "UPDATE projects SET branch = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![branch.trim(), id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update branch: {e}")))?;
        }

        if let Some(build_cmd) = &input.build_cmd {
            tx.execute(
                "UPDATE projects SET build_cmd = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![build_cmd.trim(), id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update build_cmd: {e}")))?;
        }

        if let Some(start_cmd) = &input.start_cmd {
            tx.execute(
                "UPDATE projects SET start_cmd = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![start_cmd.trim(), id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update start_cmd: {e}")))?;
        }

        if let Some(port_manual) = input.port_manual {
            tx.execute(
                "UPDATE projects SET port_manual = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![port_manual, id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update port_manual: {e}")))?;
        }

        if let Some(auto_start) = input.auto_start {
            tx.execute(
                "UPDATE projects SET auto_start = ?1, updated_at = datetime('now') WHERE id = ?2;",
                params![auto_start, id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update auto_start: {e}")))?;
        }

        // Jika env_vars disediakan, update total untuk project tersebut
        if let Some(envs) = &input.env_vars {
            tx.execute("DELETE FROM env_vars WHERE project_id = ?1;", params![id])
                .map_err(|e| AppError::Internal(format!("Gagal hapus env_vars lama: {e}")))?;

            for item in envs {
                let enc_val = self.encrypt_env_value(&item.value)?;
                tx.execute(
                    "INSERT INTO env_vars (project_id, key, value) VALUES (?1, ?2, ?3);",
                    params![id, item.key.trim(), enc_val],
                )
                .map_err(|e| AppError::Internal(format!("Gagal simpan env_var baru: {e}")))?;
            }
        }

        tx.commit()
            .map_err(|e| AppError::Internal(format!("Gagal commit update: {e}")))?;

        drop(conn);
        self.get_project(id)
    }

    /// Hapus project berdasarkan ID (Cascade menghapus env_vars)
    pub fn delete_project(&self, id: i64) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let rows_affected = conn
            .execute("DELETE FROM projects WHERE id = ?1;", params![id])
            .map_err(|e| AppError::Internal(format!("Gagal menghapus project: {e}")))?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Project dengan ID {id} tidak ditemukan"
            )));
        }

        Ok(())
    }

    /// Update status build terakhir project (Fase 5)
    pub fn update_build_status(&self, id: i64, success: bool) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let rows_affected = conn
            .execute(
                "UPDATE projects SET last_build_ok = ?1, last_build_at = datetime('now'), updated_at = datetime('now') WHERE id = ?2;",
                params![success, id],
            )
            .map_err(|e| AppError::Internal(format!("Gagal update status build: {e}")))?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Project dengan ID {id} tidak ditemukan"
            )));
        }

        Ok(())
    }

    /// Ambil daftar environment variables untuk suatu project (nilai terdekripsi)
    pub fn get_env_vars(&self, project_id: i64) -> Result<Vec<EnvVarItem>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let mut stmt = conn
            .prepare("SELECT key, value FROM env_vars WHERE project_id = ?1 ORDER BY key ASC;")
            .map_err(|e| AppError::Internal(format!("Prepare env_vars error: {e}")))?;

        let rows = stmt
            .query_map(params![project_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::Internal(format!("Query env_vars error: {e}")))?;

        let mut result = Vec::new();
        for item in rows {
            let (key, raw_val) =
                item.map_err(|e| AppError::Internal(format!("Env row error: {e}")))?;
            let value = self.decrypt_env_value(&raw_val);
            result.push(EnvVarItem { key, value });
        }

        Ok(result)
    }

    /// Set satu environment variable (otomatis terenkripsi AES-256-GCM)
    pub fn set_env_var(&self, project_id: i64, key: &str, value: &str) -> Result<(), AppError> {
        let clean_key = key.trim();
        Self::validate_env_key(clean_key)?;
        let enc_val = self.encrypt_env_value(value)?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        conn.execute(
            "INSERT INTO env_vars (project_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(project_id, key) DO UPDATE SET value = excluded.value;",
            params![project_id, clean_key, enc_val],
        )
        .map_err(|e| AppError::Internal(format!("Gagal upsert env var: {e}")))?;

        Ok(())
    }

    /// Hapus satu environment variable berdasarkan key
    pub fn delete_env_var(&self, project_id: i64, key: &str) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        conn.execute(
            "DELETE FROM env_vars WHERE project_id = ?1 AND key = ?2;",
            params![project_id, key.trim()],
        )
        .map_err(|e| AppError::Internal(format!("Gagal hapus env var: {e}")))?;

        Ok(())
    }

    /// Set daftar environment variables sekaligus (batch upsert/replace)
    pub fn set_env_vars_batch(&self, project_id: i64, vars: &[EnvVarItem]) -> Result<(), AppError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let tx = conn
            .transaction()
            .map_err(|e| AppError::Internal(format!("Gagal transaksi batch env: {e}")))?;

        for item in vars {
            let key = item.key.trim();
            Self::validate_env_key(key)?;
            let enc_val = self.encrypt_env_value(&item.value)?;
            tx.execute(
                "INSERT INTO env_vars (project_id, key, value) VALUES (?1, ?2, ?3)
                 ON CONFLICT(project_id, key) DO UPDATE SET value = excluded.value;",
                params![project_id, key, enc_val],
            )
            .map_err(|e| AppError::Internal(format!("Gagal upsert batch env var: {e}")))?;
        }

        tx.commit()
            .map_err(|e| AppError::Internal(format!("Gagal commit batch env: {e}")))?;

        Ok(())
    }

    /// Ambil nilai setting berdasarkan key
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1;")
            .map_err(|e| AppError::Internal(format!("Prepare setting error: {e}")))?;

        let val = stmt
            .query_row(params![key], |r| r.get::<_, String>(0))
            .optional()
            .map_err(|e| AppError::Internal(format!("Query setting error: {e}")))?;

        Ok(val)
    }

    /// Simpan atau perbarui nilai setting
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            params![key, value],
        )
        .map_err(|e| AppError::Internal(format!("Gagal menyimpan setting: {e}")))?;

        Ok(())
    }

    /// Hapus nilai setting berdasarkan key
    pub fn delete_setting(&self, key: &str) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        conn.execute("DELETE FROM settings WHERE key = ?1;", params![key])
            .map_err(|e| AppError::Internal(format!("Gagal menghapus setting: {e}")))?;

        Ok(())
    }

    /// Ambil semua setting
    pub fn list_settings(&self) -> Result<Vec<SettingItem>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(format!("Lock error: {e}")))?;

        let mut stmt = conn
            .prepare("SELECT key, value FROM settings ORDER BY key ASC;")
            .map_err(|e| AppError::Internal(format!("Prepare list settings error: {e}")))?;

        let iter = stmt
            .query_map([], |r| {
                Ok(SettingItem {
                    key: r.get(0)?,
                    value: r.get(1)?,
                })
            })
            .map_err(|e| AppError::Internal(format!("Query list settings error: {e}")))?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item.map_err(|e| AppError::Internal(format!("Row setting error: {e}")))?);
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_crud_and_validation() {
        let db = Database::open_in_memory().expect("open in-memory db");

        // 1. Validation tests
        assert!(Database::validate_project_name("valid-name_123").is_ok());
        assert!(Database::validate_project_name("bad name with spaces").is_err());
        assert!(Database::validate_project_name("bad;name").is_err());
        assert!(Database::validate_project_name("../traversal").is_err());

        assert!(Database::validate_repo_url("https://github.com/expressjs/express.git").is_ok());
        assert!(Database::validate_repo_url("ftp://invalid.com").is_err());
        assert!(Database::validate_repo_url("https://github.com/app;rm -rf").is_err());

        assert!(Database::validate_branch("main").is_ok());
        assert!(Database::validate_branch("feature/new-login").is_ok());
        assert!(Database::validate_branch("../bad").is_err());

        assert!(Database::validate_port(3000).is_ok());
        assert!(Database::validate_port(80).is_err()); // non-root port enforcement

        // 2. Create project
        let created = db
            .create_project(CreateProjectInput {
                name: "express-sample".into(),
                repo_url: "https://github.com/expressjs/express.git".into(),
                branch: Some("main".into()),
                build_cmd: Some("npm install".into()),
                start_cmd: Some("node index.js".into()),
                port_manual: Some(3001),
                auto_start: Some(true),
                env_vars: Some(vec![EnvVarItem {
                    key: "PORT".into(),
                    value: "3001".into(),
                }]),
            })
            .expect("create project");

        assert_eq!(created.name, "express-sample");
        assert_eq!(created.port_manual, Some(3001));
        assert!(created.env_vars.is_some());
        assert_eq!(created.env_vars.as_ref().unwrap().len(), 1);

        // 3. Duplicate name rejected
        let dup = db.create_project(CreateProjectInput {
            name: "express-sample".into(),
            repo_url: "https://github.com/test/test.git".into(),
            branch: None,
            build_cmd: None,
            start_cmd: None,
            port_manual: None,
            auto_start: None,
            env_vars: None,
        });
        assert!(dup.is_err());

        // 4. List projects
        let list = db.list_projects().expect("list projects");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        // 5. Update project
        let updated = db
            .update_project(
                created.id,
                UpdateProjectInput {
                    name: Some("express-updated".into()),
                    repo_url: None,
                    branch: Some("develop".into()),
                    build_cmd: None,
                    start_cmd: None,
                    port_manual: Some(8080),
                    auto_start: None,
                    env_vars: Some(vec![
                        EnvVarItem {
                            key: "PORT".into(),
                            value: "8080".into(),
                        },
                        EnvVarItem {
                            key: "NODE_ENV".into(),
                            value: "production".into(),
                        },
                    ]),
                },
            )
            .expect("update project");

        assert_eq!(updated.name, "express-updated");
        assert_eq!(updated.branch, "develop");
        assert_eq!(updated.port_manual, Some(8080));
        assert_eq!(updated.env_vars.as_ref().unwrap().len(), 2);

        // 5b. Update build status (Fase 5)
        db.update_build_status(created.id, true)
            .expect("update build status ok");
        let proj_built = db.get_project(created.id).expect("get project after build");
        assert_eq!(proj_built.last_build_ok, Some(true));
        assert!(proj_built.last_build_at.is_some());

        // 5b. Update build status (Fase 5)
        db.update_build_status(created.id, true)
            .expect("update build status ok");
        let proj_built = db.get_project(created.id).expect("get project after build");
        assert_eq!(proj_built.last_build_ok, Some(true));
        assert!(proj_built.last_build_at.is_some());

        // 6. Settings get/set
        let default_lang = db.get_setting("language").expect("get setting");
        assert_eq!(default_lang, Some("id".into()));

        db.set_setting("language", "en").expect("set setting");
        let updated_lang = db.get_setting("language").expect("get setting");
        assert_eq!(updated_lang, Some("en".into()));

        // 7. Delete project (cascade removes env_vars)
        db.delete_project(created.id).expect("delete project");
        let list_after = db.list_projects().expect("list after delete");
        assert_eq!(list_after.len(), 0);

        let get_deleted = db.get_project(created.id);
        assert!(get_deleted.is_err());
    }

    #[test]
    fn test_env_vars_encryption_and_crud() {
        let db = Database::open_in_memory().expect("open in-memory db");

        let project = db
            .create_project(CreateProjectInput {
                name: "secure-app".into(),
                repo_url: "https://github.com/secure/app.git".into(),
                branch: Some("main".into()),
                build_cmd: None,
                start_cmd: None,
                port_manual: Some(3000),
                auto_start: Some(true),
                env_vars: Some(vec![EnvVarItem {
                    key: "DATABASE_PASSWORD".into(),
                    value: "SuperSecretPass123!".into(),
                }]),
            })
            .expect("create project");

        // 1. Periksa penyimpanan raw di SQLite: nilainya terenkripsi dan BUKAN plaintext
        {
            let conn = db.conn.lock().unwrap();
            let raw_stored: String = conn
                .query_row(
                    "SELECT value FROM env_vars WHERE project_id = ?1 AND key = 'DATABASE_PASSWORD';",
                    params![project.id],
                    |r| r.get(0),
                )
                .expect("query raw env");

            // Format terenkripsi AES-256-GCM adalah <nonce_hex>:<ciphertext_hex>
            assert!(raw_stored.contains(':'));
            // Plaintext tidak boleh bocor sama sekali di database!
            assert!(!raw_stored.contains("SuperSecretPass123!"));
        }

        // 2. get_env_vars mendekripsi nilai dengan sempurna
        let envs = db.get_env_vars(project.id).expect("get_env_vars");
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].key, "DATABASE_PASSWORD");
        assert_eq!(envs[0].value, "SuperSecretPass123!");

        // 3. set_env_var menambah/mengupdate variabel tunggal
        db.set_env_var(project.id, "API_KEY", "key-999-xyz")
            .expect("set_env_var");
        let envs2 = db.get_env_vars(project.id).expect("get_env_vars after set");
        assert_eq!(envs2.len(), 2);

        // 4. delete_env_var menghapus variabel tertentu
        db.delete_env_var(project.id, "API_KEY")
            .expect("delete_env_var");
        let envs3 = db
            .get_env_vars(project.id)
            .expect("get_env_vars after delete");
        assert_eq!(envs3.len(), 1);
        assert_eq!(envs3[0].key, "DATABASE_PASSWORD");

        // 5. set_env_vars_batch
        db.set_env_vars_batch(
            project.id,
            &[
                EnvVarItem {
                    key: "VAR_A".into(),
                    value: "apple".into(),
                },
                EnvVarItem {
                    key: "VAR_B".into(),
                    value: "banana".into(),
                },
            ],
        )
        .expect("batch set envs");

        let envs4 = db.get_env_vars(project.id).expect("get envs after batch");
        assert_eq!(envs4.len(), 3);
    }
}
