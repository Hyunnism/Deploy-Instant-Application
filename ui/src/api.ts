import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface PingResponse {
  message: string;
  timestamp: number;
  arch: string;
}

export interface ProbeResult {
  success: boolean;
  uname: string;
  node_version: string;
  git_version: string;
  network_check: string;
  elapsed_ms: number;
  error?: string | null;
}

export interface SetupStatusResponse {
  is_ready: boolean;
  version: string | null;
  free_space_mb: number;
  required_space_mb: number;
}

export interface SetupProgress {
  step: string;
  percent: number;
  message: string;
}

export interface SetupVerificationResult {
  success: boolean;
  node_version: string;
  pm2_version: string;
  cloudflared_version: string;
  git_version: string;
  disk_used_mb: number;
  error?: string | null;
}

export interface EnvVarItem {
  key: string;
  value: string;
}

export interface Project {
  id: number;
  name: string;
  repo_url: string;
  branch: string;
  build_cmd: string;
  start_cmd: string;
  port_manual?: number | null;
  auto_start: boolean;
  last_build_ok?: boolean | null;
  last_build_at?: string | null;
  created_at: string;
  updated_at: string;
  env_vars?: EnvVarItem[] | null;
}

export interface CreateProjectInput {
  name: string;
  repo_url: string;
  branch?: string;
  build_cmd?: string;
  start_cmd?: string;
  port_manual?: number | null;
  auto_start?: boolean;
  env_vars?: EnvVarItem[];
}

export interface UpdateProjectInput {
  name?: string;
  repo_url?: string;
  branch?: string;
  build_cmd?: string;
  start_cmd?: string;
  port_manual?: number | null;
  auto_start?: boolean;
  env_vars?: EnvVarItem[];
}

export interface SettingItem {
  key: string;
  value: string;
}

/**
 * Satu-satunya tempat memanggil invoke() sesuai aturan PRD 12A.2.
 */
export async function pingBackend(): Promise<PingResponse> {
  return await invoke<PingResponse>('system_ping');
}

export async function runProotProbe(noSeccomp: boolean): Promise<ProbeResult> {
  return await invoke<ProbeResult>('system_run_probe', { noSeccomp });
}

export async function checkSetupStatus(): Promise<SetupStatusResponse> {
  return await invoke<SetupStatusResponse>('setup_check_status');
}

export async function startSetup(): Promise<SetupVerificationResult> {
  return await invoke<SetupVerificationResult>('setup_start');
}

export async function verifySetup(): Promise<SetupVerificationResult> {
  return await invoke<SetupVerificationResult>('setup_verify');
}

export async function onSetupProgress(
  callback: (progress: SetupProgress) => void
): Promise<UnlistenFn> {
  return await listen<SetupProgress>('setup-progress', (event) => {
    callback(event.payload);
  });
}

// Fase 3 Project CRUD & Settings
export async function listProjects(): Promise<Project[]> {
  return await invoke<Project[]>('project_list');
}

export async function getProject(id: number): Promise<Project> {
  return await invoke<Project>('project_get', { id });
}

export async function createProject(input: CreateProjectInput): Promise<Project> {
  return await invoke<Project>('project_create', { input });
}

export async function updateProject(id: number, input: UpdateProjectInput): Promise<Project> {
  return await invoke<Project>('project_update', { id, input });
}

export async function deleteProject(id: number): Promise<void> {
  return await invoke<void>('project_delete', { id });
}

export async function getSetting(key: string): Promise<string | null> {
  return await invoke<string | null>('setting_get', { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return await invoke<void>('setting_set', { key, value });
}

export async function listSettings(): Promise<SettingItem[]> {
  return await invoke<SettingItem[]>('setting_list');
}
