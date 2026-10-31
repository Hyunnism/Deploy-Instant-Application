// main.ts
// Deploy Instant - Modern Clean UI (Open Pi Reference Adaptation)

import './styles/index.css';
import {
  checkSetupStatus,
  createProject,
  deleteProject,
  getSetting,
  listProjects,
  onSetupProgress,
  runProotProbe,
  setSetting,
  startSetup,
  updateProject,
  verifySetup,
  type Project,
} from './api';
import { translations, type TranslationKey } from './i18n';
import { defaultFormData, state } from './state';

const appEl = document.getElementById('app')!;

function t(key: TranslationKey): string {
  const lang = state.language;
  return translations[lang]?.[key] || translations.id[key] || key;
}

function cleanVer(v: string | undefined | null, fallback: string): string {
  if (!v) return fallback;
  const lines = v.trim().split('\n').map((l) => l.trim()).filter(Boolean);
  const last = lines[lines.length - 1] || fallback;
  return last.length > 20 ? last.slice(0, 18) + '...' : last;
}

// -------------------------------------------------------------
// RENDER CONTROLLER
// -------------------------------------------------------------
function render() {
  const isReady = state.setupInfo?.is_ready ?? false;

  appEl.innerHTML = `
    <div class="app-container">
      <!-- Top Navigation Bar -->
      <nav class="top-nav">
        <button class="top-nav-btn" id="btn-open-settings" title="${t('settingsTab')}">
          ⚙️
        </button>
        <span class="top-nav-title">${t('appName')}</span>
        <div class="top-nav-actions">
          <button class="pill-icon-btn" id="btn-open-diagnostics" title="${t('diagnosticsTab')}">
            <span>💻</span>
          </button>
          <button class="pill-icon-btn primary" id="btn-create-project-top">
            <span>+</span>
          </button>
        </div>
      </nav>

      <!-- Active View -->
      ${
        state.activeTab === 'settings'
          ? renderSettingsView()
          : state.activeTab === 'diagnostics'
          ? renderDiagnosticsView()
          : renderDashboardView(isReady)
      }

      <!-- Modals -->
      ${renderFormModal()}
      ${renderDeleteModal()}
      ${renderEngineModal()}
    </div>
  `;

  attachEventListeners();
}

// -------------------------------------------------------------
// DASHBOARD VIEW
// -------------------------------------------------------------
function renderDashboardView(isReady: boolean): string {
  const projectCount = state.projects.length;

  return `
    <!-- System Status Overview Card (Open Pi Style) -->
    <section class="overview-card" id="card-instant-engine" title="Klik untuk melihat detail atau instal toolchain">
      <div class="overview-header">
        <div class="overview-title-group">
          <h2>${t('activeEngine')}</h2>
          <div class="overview-subtitle">${t('toolchainHeader')}</div>
        </div>
        <div class="engine-status-pill ${isReady ? 'healthy' : 'warning'}">
          <span class="engine-status-dot"></span>
          <span class="engine-status-text">${isReady ? t('statusReady') : t('statusNotReady')}</span>
          <span class="engine-status-sep"></span>
          <span class="engine-status-detail">${isReady ? t('viewDetailPill') : t('installActionPill')}</span>
        </div>
      </div>
      <div class="overview-metrics">
        <div class="metric-col">
          <span class="metric-label">${t('uptime')}</span>
          <span class="metric-val" style="color: ${isReady ? 'var(--success)' : 'var(--warning)'};">
            ${isReady ? t('statusReady') : t('statusNotReady')}
          </span>
        </div>
        <div class="metric-col">
          <span class="metric-label">${t('servicesCount')}</span>
          <span class="metric-val">${projectCount}</span>
        </div>
        <div class="metric-col">
          <span class="metric-label">${t('storageUsed')}</span>
          <span class="metric-val">${isReady ? '~343 MB' : '-'}</span>
        </div>
      </div>
    </section>

    <!-- Project List Section -->
    <section class="section-header">
      <h3 class="section-title">${t('projectListHeader')}</h3>
      <button class="pill-icon-btn primary" id="btn-create-project-header">
        ${t('newProjectBtn')}
      </button>
    </section>

    <div class="projects-container">
      ${
        projectCount === 0
          ? renderEmptyState()
          : state.projects.map((p) => renderProjectCard(p)).join('')
      }
    </div>
  `;
}

function renderEmptyState(): string {
  return `
    <div class="empty-card">
      <div class="empty-icon">📁</div>
      <div class="empty-title">${t('noProjectsTitle')}</div>
      <p class="empty-desc">${t('noProjectsDesc')}</p>
      <button class="empty-btn" id="btn-create-first-project">
        ${t('createFirstProject')}
      </button>
    </div>
  `;
}

function renderProjectCard(p: Project): string {
  const initial = (p.name.charAt(0) || 'P').toUpperCase();
  const portLabel = p.port_manual ? `Port ${p.port_manual}` : 'Port Auto';

  return `
    <div class="project-card" data-id="${p.id}">
      <div class="project-card-header">
        <div class="project-identity">
          <div class="project-icon-box">${initial}</div>
          <div class="project-name">${p.name}</div>
        </div>
        <div class="project-card-actions">
          <button class="proj-action-btn edit btn-edit-project" data-id="${p.id}">${t('editBtn')}</button>
          <button class="proj-action-btn delete btn-delete-project" data-id="${p.id}">${t('deleteBtn')}</button>
        </div>
      </div>

      <div class="project-meta-row">
        <span class="meta-badge">
          <svg class="meta-svg" viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="6" y1="3" x2="6" y2="15"></line>
            <circle cx="18" cy="6" r="3"></circle>
            <circle cx="6" cy="18" r="3"></circle>
            <path d="M18 9a9 9 0 0 1-9 9"></path>
          </svg>
          ${p.branch}
        </span>
        <span class="meta-badge">
          <svg class="meta-svg" viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.5">
            <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
            <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
            <line x1="6" y1="6" x2="6.01" y2="6"></line>
            <line x1="6" y1="18" x2="6.01" y2="18"></line>
          </svg>
          ${portLabel}
        </span>
        ${
          p.auto_start
            ? `<span class="meta-badge autostart">
                 <span class="meta-dot"></span>
                 ${t('autoStartBadge')}
               </span>`
            : ''
        }
      </div>

      <div class="project-repo-line">
        <svg class="meta-svg" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
        </svg>
        <span class="repo-text">${p.repo_url}</span>
      </div>

      <div class="project-commands-grid">
        <div class="command-box">
          <div class="command-box-label">${t('cardBuild')}</div>
          <div class="command-box-cmd">${p.build_cmd}</div>
        </div>
        <div class="command-box">
          <div class="command-box-label">${t('cardStart')}</div>
          <div class="command-box-cmd">${p.start_cmd}</div>
        </div>
      </div>
    </div>
  `;
}

// -------------------------------------------------------------
// SETTINGS VIEW
// -------------------------------------------------------------
function renderSettingsView(): string {
  return `
    <section class="section-header">
      <h3 class="section-title">${t('settingsTitle')}</h3>
      <button class="pill-icon-btn" id="btn-back-from-settings">
        ${t('backToDashboard')}
      </button>
    </section>

    <div class="settings-group">
      <div class="settings-card">
        <div class="overview-title-group">
          <h2>${t('languageSection')}</h2>
        </div>
        <div class="lang-select-group">
          <button class="lang-pill-btn ${state.language === 'id' ? 'active' : ''}" id="btn-set-lang-id">
            ${t('langIndonesian')}
          </button>
          <button class="lang-pill-btn ${state.language === 'en' ? 'active' : ''}" id="btn-set-lang-en">
            ${t('langEnglish')}
          </button>
        </div>
      </div>

      <div class="settings-card">
        <div class="overview-title-group">
          <h2>${t('aboutSection')}</h2>
        </div>
        <div class="settings-row">
          <span class="metric-label">${t('appVersion')}</span>
          <span class="metric-val" style="font-size: 0.88rem;">v0.4.0 (Fase 3)</span>
        </div>
        <div class="settings-row">
          <span class="metric-label">${t('engineRuntime')}</span>
          <span class="metric-val" style="font-size: 0.88rem;">Node v24.22.1 LTS + PM2</span>
        </div>
        <p style="font-size: 0.8rem; color: var(--text-muted); line-height: 1.45; margin-top: 4px;">
          ${t('zeroLaptopNotice')}
        </p>
      </div>

      <div class="settings-card">
        <button class="pill-icon-btn primary" id="btn-diag-shortcut" style="justify-content: center; width: 100%; padding: 10px;">
          ${t('openDiagnostics')}
        </button>
      </div>
    </div>
  `;
}

// -------------------------------------------------------------
// DIAGNOSTICS VIEW (FASE 1 & 2 TOOLCHAIN CHECK)
// -------------------------------------------------------------
function renderDiagnosticsView(): string {
  const pRes = state.probeResult;
  const vRes = state.setupVerification;

  return `
    <section class="section-header">
      <h3 class="section-title">${t('diagnosticsTitle')}</h3>
      <button class="pill-icon-btn" id="btn-back-from-diag">
        ${t('backToDashboard')}
      </button>
    </section>

    <div class="settings-group">
      <div class="settings-card">
        <div class="overview-title-group">
          <h2>Aksi Pengujian Toolchain</h2>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
          <button class="pill-icon-btn primary" id="btn-run-probe" ${state.probeStatus === 'running' ? 'disabled' : ''}>
            ${state.probeStatus === 'running' ? 'Memeriksa...' : t('probeBtn')}
          </button>
          <button class="pill-icon-btn" id="btn-run-verify" ${state.setupStatus === 'verifying' ? 'disabled' : ''}>
            ${state.setupStatus === 'verifying' ? 'Memverifikasi...' : t('verifyBtn')}
          </button>
        </div>
      </div>

      ${
        pRes
          ? `
      <div class="settings-card">
        <div class="overview-title-group">
          <h2>Hasil Probe PRoot</h2>
        </div>
        <div class="diag-log-box">Status: ${pRes.success ? 'BERHASIL' : 'GAGAL'}
Waktu: ${pRes.elapsed_ms} ms
Node: ${pRes.node_version}
Git: ${pRes.git_version}
Network: ${pRes.network_check}
Kernel: ${pRes.uname}</div>
      </div>
      `
          : ''
      }

      ${
        vRes
          ? `
      <div class="settings-card">
        <div class="overview-title-group">
          <h2>Hasil Verifikasi Toolchain</h2>
        </div>
        <div class="diag-log-box">Node.js: ${cleanVer(vRes.node_version, 'N/A')}
PM2: ${cleanVer(vRes.pm2_version, 'N/A')}
Cloudflared: ${cleanVer(vRes.cloudflared_version, 'N/A')}
Git: ${cleanVer(vRes.git_version, 'N/A')}
Disk: ${vRes.disk_used_mb} MB terpasang</div>
      </div>
      `
          : ''
      }
    </div>
  `;
}

// -------------------------------------------------------------
// FORM MODAL (CREATE / EDIT PROJECT)
// -------------------------------------------------------------
function renderFormModal(): string {
  const modal = state.formModal;
  if (!modal.isOpen) return '';

  const isEdit = modal.mode === 'edit';
  const d = modal.data;

  return `
    <div class="modal-overlay" id="form-modal-overlay">
      <div class="modal-sheet">
        <div class="sheet-handle-bar"></div>
        <div class="modal-header">
          <h3 class="modal-title">${isEdit ? t('modalEditTitle') : t('modalCreateTitle')}</h3>
          <button class="modal-close-btn" id="btn-close-form-modal">${t('modalClose')}</button>
        </div>

        <div class="modal-body">
          ${modal.error ? `<div class="modal-alert error">${modal.error}</div>` : ''}

          <!-- Repository Card -->
          <div class="form-group-card">
            <div class="form-group-title">${t('fieldBasicInfo')}</div>
            <div class="form-row">
              <label for="input-project-name">${t('fieldName')}</label>
              <input type="text" id="input-project-name" class="form-input" 
                autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off"
                placeholder="${t('fieldNamePlaceholder')}" value="${d.name}" ${isEdit ? 'disabled style="opacity:0.6;"' : ''} />
            </div>
            <div class="form-row">
              <label for="input-repo-url">${t('fieldRepoUrl')}</label>
              <input type="text" id="input-repo-url" class="form-input mono" 
                autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off"
                placeholder="${t('fieldRepoPlaceholder')}" value="${d.repo_url}" />
            </div>
            <div class="form-row">
              <label for="input-branch">${t('fieldBranch')}</label>
              <input type="text" id="input-branch" class="form-input mono" 
                autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off"
                placeholder="${t('fieldBranchPlaceholder')}" value="${d.branch}" />
            </div>
          </div>

          <!-- Execution Card -->
          <div class="form-group-card">
            <div class="form-group-title">${t('fieldExecution')}</div>
            <div class="form-row">
              <label for="input-port">${t('fieldPort')}</label>
              <input type="number" id="input-port" class="form-input mono" 
                placeholder="${t('fieldPortPlaceholder')}" value="${d.port_manual}" />
            </div>
            <div class="form-row">
              <label for="input-build-cmd">${t('fieldBuildCmd')}</label>
              <input type="text" id="input-build-cmd" class="form-input mono" 
                autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off"
                placeholder="${t('fieldBuildCmdPlaceholder')}" value="${d.build_cmd}" />
            </div>
            <div class="form-row">
              <label for="input-start-cmd">${t('fieldStartCmd')}</label>
              <input type="text" id="input-start-cmd" class="form-input mono" 
                autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off"
                placeholder="${t('fieldStartCmdPlaceholder')}" value="${d.start_cmd}" />
            </div>
            <div class="form-toggle-row">
              <label for="input-auto-start">${t('fieldAutoStart')}</label>
              <input type="checkbox" id="input-auto-start" class="switch-input" ${d.auto_start ? 'checked' : ''} />
            </div>
          </div>

          <!-- Env Vars Card -->
          <div class="form-group-card">
            <div class="form-group-title">${t('fieldEnvVars')}</div>
            <div id="env-vars-list">
              ${
                d.env_vars.length === 0
                  ? `<div style="padding: 10px 14px; font-size: 0.8rem; color: var(--text-subtle);">Tidak ada variabel custom.</div>`
                  : d.env_vars
                      .map(
                        (env, idx) => `
                    <div class="env-row-item">
                      <input type="text" class="form-input mono env-key-input" data-index="${idx}" autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off" placeholder="${t('envKeyPlaceholder')}" value="${env.key}" />
                      <input type="text" class="form-input mono env-val-input" data-index="${idx}" autocapitalize="none" autocorrect="off" spellcheck="false" autocomplete="off" placeholder="${t('envValPlaceholder')}" value="${env.value}" />
                      <button class="env-remove-btn" data-index="${idx}" title="Hapus">✕</button>
                    </div>
                  `
                      )
                      .join('')
              }
            </div>
            <div class="add-env-btn-row">
              <button class="subtle-btn" id="btn-add-env-row">${t('addEnvBtn')}</button>
            </div>
          </div>
        </div>

        <!-- Sticky Actions Footer -->
        <div class="modal-footer">
          <button class="modal-btn cancel" id="btn-cancel-form-modal">${t('modalCancel')}</button>
          <button class="modal-btn primary" id="btn-save-project" ${modal.isSaving ? 'disabled' : ''}>
            ${modal.isSaving ? t('modalSaving') : t('modalSave')}
          </button>
        </div>
      </div>
    </div>
  `;
}

// -------------------------------------------------------------
// DELETE CONFIRMATION MODAL
// -------------------------------------------------------------
function renderDeleteModal(): string {
  const modal = state.deleteModal;
  if (!modal.isOpen || !modal.project) return '';

  return `
    <div class="modal-overlay dialog" id="delete-modal-overlay">
      <div class="delete-confirm-box" style="width: 90%; max-width: 400px; margin: auto;">
        <div class="overview-title-group">
          <h2>${t('modalDeleteTitle')}</h2>
          <div class="overview-subtitle" style="font-weight: 600; color: var(--text-main); margin-top: 4px;">
            ${modal.project.name}
          </div>
        </div>
        <p style="font-size: 0.85rem; color: var(--text-muted); line-height: 1.45;">
          ${t('modalDeleteMsg')}
        </p>

        ${modal.error ? `<div class="modal-alert error">${modal.error}</div>` : ''}

        <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 10px;">
          <button class="modal-btn cancel" id="btn-cancel-delete">${t('modalCancel')}</button>
          <button class="modal-btn danger" id="btn-confirm-delete" ${modal.isDeleting ? 'disabled' : ''}>
            ${modal.isDeleting ? 'Menghapus...' : t('confirmDeleteBtn')}
          </button>
        </div>
      </div>
    </div>
  `;
}

// -------------------------------------------------------------
// ENGINE DETAIL & INSTALL MODAL (Open Pi Service Style)
// -------------------------------------------------------------
function renderEngineModal(): string {
  if (!state.engineModal.isOpen) return '';

  const isReady = state.setupInfo?.is_ready ?? false;
  const isExtracting = state.setupStatus === 'extracting';
  const isVerifying = state.setupStatus === 'verifying';
  const progress = state.setupProgress;
  const vRes = state.setupVerification;

  const nodeVer = cleanVer(vRes?.node_version, isReady ? 'v24.22.1 LTS' : 'Belum Terpasang');
  const pm2Ver = cleanVer(vRes?.pm2_version, isReady ? '7.1.1' : 'Belum Terpasang');
  const cfVer = cleanVer(vRes?.cloudflared_version, isReady ? '2026.10.7' : 'Belum Terpasang');
  const gitVer = cleanVer(vRes?.git_version, isReady ? '2.39.5' : 'Belum Terpasang');
  const diskUsed = vRes?.disk_used_mb ? `${vRes.disk_used_mb} MB` : isReady ? '~343 MB' : '-';

  return `
    <div class="modal-overlay" id="engine-modal-overlay">
      <div class="modal-sheet">
        <div class="sheet-handle-bar"></div>
        <div class="modal-header">
          <h3 class="modal-title">${t('engineModalTitle')}</h3>
          <button class="modal-close-btn" id="btn-close-engine-modal">${t('modalClose')}</button>
        </div>

        <div class="modal-body">
          <!-- Hero Card -->
          <div class="engine-hero-card">
            <div class="engine-hero-icon">⚡</div>
            <div class="engine-hero-info">
              <span class="engine-hero-title">${t('activeEngine')}</span>
              <span class="engine-hero-sub">${t('engineInfoSubtitle')}</span>
            </div>
          </div>

          <!-- Specs List (Ref 2 Service Monitoring Style) -->
          <div class="engine-spec-list">
            <div class="form-group-title">${t('monitoringSection')}</div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">Status Lingkungan</span>
              <span class="engine-spec-value ${isReady ? 'healthy' : 'warning'}">
                ${isReady ? t('statusReady') : t('statusNotReady')}
              </span>
            </div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">Node.js Runtime</span>
              <span class="engine-spec-value">${nodeVer}</span>
            </div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">PM2 Process Manager</span>
              <span class="engine-spec-value">${pm2Ver}</span>
            </div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">Cloudflared Tunnel</span>
              <span class="engine-spec-value">${cfVer}</span>
            </div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">Git SCM</span>
              <span class="engine-spec-value">${gitVer}</span>
            </div>
            <div class="engine-spec-row">
              <span class="engine-spec-label">Ruang Rootfs Terpakai</span>
              <span class="engine-spec-value">${diskUsed}</span>
            </div>
          </div>

          <!-- If extracting / installing -->
          ${
            isExtracting && progress
              ? `
          <div class="setup-progress-wrap">
            <div style="display: flex; justify-content: space-between; font-size: 0.82rem; font-weight: 600;">
              <span>${progress.message}</span>
              <span>${progress.percent}%</span>
            </div>
            <div class="setup-progress-bar-bg">
              <div class="setup-progress-bar-fill" style="width: ${progress.percent}%;"></div>
            </div>
          </div>
          `
              : ''
          }

          <!-- Info text -->
          <p style="font-size: 0.82rem; color: var(--text-muted); line-height: 1.45; padding: 2px 4px;">
            ${isReady ? t('engineInstalledDesc') : t('engineNotInstalled')}
          </p>
        </div>

        <!-- Sticky Footer -->
        <div class="modal-footer">
          <button class="modal-btn cancel" id="btn-cancel-engine-modal">${t('modalClose')}</button>
          ${
            !isReady
              ? `
            <button class="modal-btn primary" id="btn-start-engine-setup" ${isExtracting || isVerifying ? 'disabled' : ''}>
              ${isExtracting ? t('statusInstalling') : t('installEngineBtn')}
            </button>
          `
              : `
            <button class="modal-btn primary" id="btn-reverify-engine" ${isVerifying ? 'disabled' : ''}>
              ${isVerifying ? 'Memverifikasi...' : t('reverifyBtn')}
            </button>
          `
          }
        </div>
      </div>
    </div>
  `;
}

// -------------------------------------------------------------
// EVENT LISTENERS & USER ACTIONS
// -------------------------------------------------------------
function attachEventListeners() {
  // Navigation
  document.getElementById('btn-open-settings')?.addEventListener('click', () => {
    state.activeTab = state.activeTab === 'settings' ? 'dashboard' : 'settings';
    render();
  });

  document.getElementById('btn-open-diagnostics')?.addEventListener('click', () => {
    state.activeTab = state.activeTab === 'diagnostics' ? 'dashboard' : 'diagnostics';
    render();
  });

  document.getElementById('btn-back-from-settings')?.addEventListener('click', () => {
    state.activeTab = 'dashboard';
    render();
  });

  document.getElementById('btn-back-from-diag')?.addEventListener('click', () => {
    state.activeTab = 'dashboard';
    render();
  });

  document.getElementById('btn-diag-shortcut')?.addEventListener('click', () => {
    state.activeTab = 'diagnostics';
    render();
  });

  // Language buttons
  document.getElementById('btn-set-lang-id')?.addEventListener('click', async () => {
    state.language = 'id';
    render();
    try {
      await setSetting('language', 'id');
    } catch (e) {
      console.error(e);
    }
  });

  document.getElementById('btn-set-lang-en')?.addEventListener('click', async () => {
    state.language = 'en';
    render();
    try {
      await setSetting('language', 'en');
    } catch (e) {
      console.error(e);
    }
  });

  // Diagnostics actions
  document.getElementById('btn-run-probe')?.addEventListener('click', async () => {
    state.probeStatus = 'running';
    render();
    try {
      const res = await runProotProbe(state.noSeccomp);
      state.probeResult = res;
      state.probeStatus = res.success ? 'success' : 'failed';
    } catch (err: any) {
      state.probeStatus = 'failed';
    }
    render();
  });

  document.getElementById('btn-run-verify')?.addEventListener('click', async () => {
    state.setupStatus = 'verifying';
    render();
    try {
      const res = await verifySetup();
      state.setupVerification = res;
      state.setupStatus = res.success ? 'ready' : 'error';
    } catch (err: any) {
      state.setupStatus = 'error';
    }
    render();
  });

  // Engine Modal Handlers
  document.getElementById('card-instant-engine')?.addEventListener('click', () => {
    state.engineModal.isOpen = true;
    render();
  });

  document.getElementById('btn-close-engine-modal')?.addEventListener('click', () => {
    state.engineModal.isOpen = false;
    render();
  });

  document.getElementById('btn-cancel-engine-modal')?.addEventListener('click', () => {
    state.engineModal.isOpen = false;
    render();
  });

  document.getElementById('btn-start-engine-setup')?.addEventListener('click', async () => {
    state.setupStatus = 'extracting';
    render();

    try {
      const unlisten = await onSetupProgress((p) => {
        state.setupProgress = p;
        render();
      });

      const res = await startSetup();
      unlisten();
      state.setupVerification = res;
      state.setupStatus = res.success ? 'ready' : 'error';
      state.setupInfo = await checkSetupStatus();
    } catch (err: any) {
      state.setupStatus = 'error';
      state.setupError = String(err);
    }
    render();
  });

  document.getElementById('btn-reverify-engine')?.addEventListener('click', async () => {
    state.setupStatus = 'verifying';
    render();

    try {
      const res = await verifySetup();
      state.setupVerification = res;
      state.setupStatus = res.success ? 'ready' : 'error';
      state.setupInfo = await checkSetupStatus();
    } catch (err: any) {
      state.setupStatus = 'error';
    }
    render();
  });

  // Project Create buttons
  const openCreateModal = () => {
    state.formModal = {
      isOpen: true,
      mode: 'create',
      projectId: null,
      data: { ...defaultFormData, env_vars: [] },
      isSaving: false,
      error: null,
    };
    render();
  };

  document.getElementById('btn-create-project-top')?.addEventListener('click', openCreateModal);
  document.getElementById('btn-create-project-header')?.addEventListener('click', openCreateModal);
  document.getElementById('btn-create-first-project')?.addEventListener('click', openCreateModal);

  // Close Form Modal
  document.getElementById('btn-close-form-modal')?.addEventListener('click', () => {
    state.formModal.isOpen = false;
    render();
  });
  document.getElementById('btn-cancel-form-modal')?.addEventListener('click', () => {
    state.formModal.isOpen = false;
    render();
  });

  // Project Edit buttons
  document.querySelectorAll('.btn-edit-project').forEach((el) => {
    el.addEventListener('click', () => {
      const id = Number(el.getAttribute('data-id'));
      const p = state.projects.find((proj) => proj.id === id);
      if (!p) return;

      state.formModal = {
        isOpen: true,
        mode: 'edit',
        projectId: p.id,
        data: {
          name: p.name,
          repo_url: p.repo_url,
          branch: p.branch,
          port_manual: p.port_manual ? String(p.port_manual) : '',
          build_cmd: p.build_cmd,
          start_cmd: p.start_cmd,
          auto_start: p.auto_start,
          env_vars: p.env_vars ? p.env_vars.map((ev) => ({ key: ev.key, value: ev.value })) : [],
        },
        isSaving: false,
        error: null,
      };
      render();
    });
  });

  // Project Delete buttons
  document.querySelectorAll('.btn-delete-project').forEach((el) => {
    el.addEventListener('click', () => {
      const id = Number(el.getAttribute('data-id'));
      const p = state.projects.find((proj) => proj.id === id);
      if (!p) return;

      state.deleteModal = {
        isOpen: true,
        project: p,
        isDeleting: false,
        error: null,
      };
      render();
    });
  });

  // Delete modal cancel & confirm
  document.getElementById('btn-cancel-delete')?.addEventListener('click', () => {
    state.deleteModal.isOpen = false;
    render();
  });

  document.getElementById('btn-confirm-delete')?.addEventListener('click', async () => {
    if (!state.deleteModal.project) return;
    state.deleteModal.isDeleting = true;
    render();

    try {
      await deleteProject(state.deleteModal.project.id);
      state.deleteModal.isOpen = false;
      await loadProjects();
    } catch (err: any) {
      state.deleteModal.isDeleting = false;
      state.deleteModal.error = String(err);
      render();
    }
  });

  // Form Modal inputs change sync
  const inputName = document.getElementById('input-project-name') as HTMLInputElement | null;
  const inputRepo = document.getElementById('input-repo-url') as HTMLInputElement | null;
  const inputBranch = document.getElementById('input-branch') as HTMLInputElement | null;
  const inputPort = document.getElementById('input-port') as HTMLInputElement | null;
  const inputBuild = document.getElementById('input-build-cmd') as HTMLInputElement | null;
  const inputStart = document.getElementById('input-start-cmd') as HTMLInputElement | null;
  const inputAutoStart = document.getElementById('input-auto-start') as HTMLInputElement | null;

  if (inputName) inputName.oninput = (e) => (state.formModal.data.name = (e.target as HTMLInputElement).value);
  if (inputRepo) inputRepo.oninput = (e) => (state.formModal.data.repo_url = (e.target as HTMLInputElement).value);
  if (inputBranch) inputBranch.oninput = (e) => (state.formModal.data.branch = (e.target as HTMLInputElement).value);
  if (inputPort) inputPort.oninput = (e) => (state.formModal.data.port_manual = (e.target as HTMLInputElement).value);
  if (inputBuild) inputBuild.oninput = (e) => (state.formModal.data.build_cmd = (e.target as HTMLInputElement).value);
  if (inputStart) inputStart.oninput = (e) => (state.formModal.data.start_cmd = (e.target as HTMLInputElement).value);
  if (inputAutoStart) inputAutoStart.onchange = (e) => (state.formModal.data.auto_start = (e.target as HTMLInputElement).checked);

  // Env inputs sync
  document.querySelectorAll('.env-key-input').forEach((el) => {
    const idx = Number(el.getAttribute('data-index'));
    (el as HTMLInputElement).oninput = (e) => {
      state.formModal.data.env_vars[idx].key = (e.target as HTMLInputElement).value;
    };
  });
  document.querySelectorAll('.env-val-input').forEach((el) => {
    const idx = Number(el.getAttribute('data-index'));
    (el as HTMLInputElement).oninput = (e) => {
      state.formModal.data.env_vars[idx].value = (e.target as HTMLInputElement).value;
    };
  });

  // Add Env Row
  document.getElementById('btn-add-env-row')?.addEventListener('click', () => {
    state.formModal.data.env_vars.push({ key: '', value: '' });
    render();
  });

  // Remove Env Row
  document.querySelectorAll('.env-remove-btn').forEach((el) => {
    el.addEventListener('click', () => {
      const idx = Number(el.getAttribute('data-index'));
      state.formModal.data.env_vars.splice(idx, 1);
      render();
    });
  });

  // Save Project
  document.getElementById('btn-save-project')?.addEventListener('click', async () => {
    const d = state.formModal.data;
    const name = d.name.trim();
    const repo_url = d.repo_url.trim();

    if (!name) {
      state.formModal.error = 'Nama project tidak boleh kosong';
      render();
      return;
    }

    if (!repo_url) {
      state.formModal.error = 'Repository URL tidak boleh kosong';
      render();
      return;
    }

    state.formModal.isSaving = true;
    state.formModal.error = null;
    render();

    try {
      const portNum = d.port_manual.trim() ? Number(d.port_manual.trim()) : null;
      const validEnvs = d.env_vars.filter((ev) => ev.key.trim().length > 0);

      if (state.formModal.mode === 'create') {
        await createProject({
          name,
          repo_url,
          branch: d.branch.trim() || 'main',
          port_manual: portNum,
          build_cmd: d.build_cmd.trim() || 'npm install',
          start_cmd: d.start_cmd.trim() || 'npm start',
          auto_start: d.auto_start,
          env_vars: validEnvs,
        });
      } else if (state.formModal.projectId) {
        await updateProject(state.formModal.projectId, {
          repo_url,
          branch: d.branch.trim() || 'main',
          port_manual: portNum,
          build_cmd: d.build_cmd.trim() || 'npm install',
          start_cmd: d.start_cmd.trim() || 'npm start',
          auto_start: d.auto_start,
          env_vars: validEnvs,
        });
      }

      state.formModal.isOpen = false;
      await loadProjects();
    } catch (err: any) {
      state.formModal.isSaving = false;
      state.formModal.error = String(err);
      render();
    }
  });
}

// -------------------------------------------------------------
// DATA LOADERS & INITIALIZATION
// -------------------------------------------------------------
async function loadProjects() {
  state.isLoadingProjects = true;
  try {
    const list = await listProjects();
    state.projects = list;
    state.projectsError = null;
  } catch (err: any) {
    state.projectsError = String(err);
  } finally {
    state.isLoadingProjects = false;
    render();
  }
}

async function init() {
  render();

  // Load language setting
  try {
    const savedLang = await getSetting('language');
    if (savedLang === 'id' || savedLang === 'en') {
      state.language = savedLang;
    }
  } catch (e) {
    console.error('Failed to load language setting:', e);
  }

  // Check setup status
  try {
    const status = await checkSetupStatus();
    state.setupInfo = status;
  } catch (e) {
    console.error('Failed to check setup status:', e);
  }

  // Load projects
  await loadProjects();
}

init();
