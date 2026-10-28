import type { PingResponse, ProbeResult } from './api';

export interface AppState {
  isInitialized: boolean;
  lastPing: PingResponse | null;
  latencyMs: number | null;
  status: 'idle' | 'pinging' | 'connected' | 'error';
  errorMessage: string | null;

  // Fase 1 Probe State
  probeResult: ProbeResult | null;
  probeStatus: 'idle' | 'running' | 'success' | 'failed';
  probeError: string | null;
  noSeccomp: boolean;
}

export const state: AppState = {
  isInitialized: true,
  lastPing: null,
  latencyMs: null,
  status: 'idle',
  errorMessage: null,

  probeResult: null,
  probeStatus: 'idle',
  probeError: null,
  noSeccomp: false,
};
