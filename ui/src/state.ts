import type { PingResponse } from './api';

export interface AppState {
  isInitialized: boolean;
  lastPing: PingResponse | null;
  latencyMs: number | null;
  status: 'idle' | 'pinging' | 'connected' | 'error';
  errorMessage: string | null;
}

export const state: AppState = {
  isInitialized: true,
  lastPing: null,
  latencyMs: null,
  status: 'idle',
  errorMessage: null,
};
