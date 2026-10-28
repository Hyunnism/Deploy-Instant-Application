import { invoke } from '@tauri-apps/api/core';

export interface PingResponse {
  message: string;
  timestamp: number;
  arch: string;
}

/**
 * Satu-satunya tempat memanggil invoke() sesuai aturan PRD 12A.2.
 */
export async function pingBackend(): Promise<PingResponse> {
  return await invoke<PingResponse>('system_ping');
}
