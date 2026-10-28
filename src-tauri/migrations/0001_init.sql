-- 0001_init.sql
-- Skema database awal Deploy Instant (PRD Bagian 8.4)

CREATE TABLE IF NOT EXISTS projects (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  name          TEXT UNIQUE NOT NULL,
  repo_url      TEXT NOT NULL,
  branch        TEXT NOT NULL DEFAULT 'main',
  build_cmd     TEXT NOT NULL DEFAULT 'npm run build',
  start_cmd     TEXT NOT NULL DEFAULT 'npm start',
  port_manual   INTEGER,
  auto_start    INTEGER NOT NULL DEFAULT 0,
  last_build_ok INTEGER,
  last_build_at TEXT,
  created_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS env_vars (
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  key        TEXT NOT NULL,
  value_enc  BLOB NOT NULL,
  PRIMARY KEY (project_id, key)
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
