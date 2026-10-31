import type {
  PingResponse,
  ProbeResult,
  Project,
  SetupProgress,
  SetupStatusResponse,
  SetupVerificationResult,
} from './api';
import type { Language } from './i18n';

export interface FormEnvVar {
  key: string;
  value: string;
}

export interface ProjectFormData {
  name: string;
  repo_url: string;
  branch: string;
  port_manual: string;
  build_cmd: string;
  start_cmd: string;
  auto_start: boolean;
  env_vars: FormEnvVar[];
}

export interface AppState {
  activeTab: 'dashboard' | 'settings' | 'diagnostics';
  language: Language;

  // Fase 3 Projects State
  projects: Project[];
  isLoadingProjects: boolean;
  projectsError: string | null;

  // Project Form Modal State
  formModal: {
    isOpen: boolean;
    mode: 'create' | 'edit';
    projectId: number | null;
    data: ProjectFormData;
    isSaving: boolean;
    error: string | null;
  };

  // Delete Confirm Modal State
  deleteModal: {
    isOpen: boolean;
    project: Project | null;
    isDeleting: boolean;
    error: string | null;
  };

  // Fase 1 Probe State
  lastPing: PingResponse | null;
  latencyMs: number | null;
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

export const defaultFormData: ProjectFormData = {
  name: 'my-express-app',
  repo_url: 'https://github.com/expressjs/express.git',
  branch: 'main',
  port_manual: '3000',
  build_cmd: 'npm install',
  start_cmd: 'npm start',
  auto_start: true,
  env_vars: [],
};

export const state: AppState = {
  activeTab: 'dashboard',
  language: 'id',

  projects: [],
  isLoadingProjects: false,
  projectsError: null,

  formModal: {
    isOpen: false,
    mode: 'create',
    projectId: null,
    data: { ...defaultFormData },
    isSaving: false,
    error: null,
  },

  deleteModal: {
    isOpen: false,
    project: null,
    isDeleting: false,
    error: null,
  },

  lastPing: null,
  latencyMs: null,
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
