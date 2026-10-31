import type {
  PingResponse,
  ProbeResult,
  SetupProgress,
  SetupStatusResponse,
  SetupVerificationResult,
} from './api';

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

  // Fase 2 Setup State
  setupStatus: 'idle' | 'checking' | 'not_ready' | 'extracting' | 'verifying' | 'ready' | 'error';
  setupInfo: SetupStatusResponse | null;
  setupProgress: SetupProgress | null;
  setupVerification: SetupVerificationResult | null;
  setupError: string | null;
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

  setupStatus: 'idle',
  setupInfo: null,
  setupProgress: null,
  setupVerification: null,
  setupError: null,
};
