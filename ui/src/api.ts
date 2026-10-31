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
