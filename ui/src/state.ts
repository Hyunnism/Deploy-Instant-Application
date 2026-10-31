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
    revealedEnvs: Record<number, boolean>;
  };

  // Fase 6 Dotenv Import Modal State
  dotenvModal: {
    isOpen: boolean;
    text: string;
    error: string | null;
  };

  // Delete Confirm Modal State
  deleteModal: {
    isOpen: boolean;
    project: Project | null;
    isDeleting: boolean;
    error: string | null;
  };

  // Engine Detail Modal State
  engineModal: {
    isOpen: boolean;
  };
  engineLogs: string[];

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

  // Fase 4 Git & PAT State
  repoStatuses: Record<
    number,
    {
      is_cloned: boolean;
      commit_hash?: string | null;
      commit_message?: string | null;
      branch?: string | null;
      isCloning?: boolean;
      isPulling?: boolean;
    }
  >;
  gitLogModal: {
    isOpen: boolean;
    projectId: number | null;
    projectName: string;
    logs: string[];
    isRunning: boolean;
    statusMessage?: string | null;
  };
  buildModal: {
    isOpen: boolean;
    projectId: number | null;
    projectName: string;
    isBuilding: boolean;
    isCancelling: boolean;
    logs: string[];
    result: {
      success: boolean;
      exitCode?: number | null;
      elapsedSeconds: number;
      errorSummary?: string | null;
      logFile?: string;
    } | null;
    statusMessage?: string | null;
  };
  patSettings: {
    hasPat: boolean;
    inputPat: string;
    showPat: boolean;
    isSaving: boolean;
    error: string | null;
    successMessage: string | null;
  };
}

export const defaultFormData: ProjectFormData = {
  name: '',
  repo_url: '',
  branch: 'main',
  port_manual: '',
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
    revealedEnvs: {},
  },

  dotenvModal: {
    isOpen: false,
    text: '',
    error: null,
  },

  deleteModal: {
    isOpen: false,
    project: null,
    isDeleting: false,
    error: null,
  },

  engineModal: {
    isOpen: false,
  },
  engineLogs: [],

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

  // Fase 4
  repoStatuses: {},
  gitLogModal: {
    isOpen: false,
    projectId: null,
    projectName: '',
    logs: [],
    isRunning: false,
    statusMessage: null,
  },
  buildModal: {
    isOpen: false,
    projectId: null,
    projectName: '',
    isBuilding: false,
    isCancelling: false,
    logs: [],
    result: null,
    statusMessage: null,
  },
  patSettings: {
    hasPat: false,
    inputPat: '',
    showPat: false,
    isSaving: false,
    error: null,
    successMessage: null,
  },
};
