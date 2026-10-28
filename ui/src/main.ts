import './styles/index.css';
import { pingBackend, runProotProbe } from './api';
import { state } from './state';

const appEl = document.getElementById('app')!;

function render() {
  const isPinging = state.status === 'pinging';
  const isConnected = state.status === 'connected';
  const isError = state.status === 'error';
  const indicatorClass = isPinging ? 'pinging' : isConnected ? 'connected' : isError ? 'error' : '';

  const isProbing = state.probeStatus === 'running';
  const isProbeSuccess = state.probeStatus === 'success';
  const isProbeFailed = state.probeStatus === 'failed';
  const probeIndicatorClass = isProbing ? 'pinging' : isProbeSuccess ? 'connected' : isProbeFailed ? 'error' : '';

  const pRes = state.probeResult;

  appEl.innerHTML = `
    <header class="header">
      <div class="brand">
        <div class="logo-badge">⚡</div>
        <div class="brand-text">
          <h1>Deploy Instant</h1>
          <p>Mini VPS di dalam HP</p>
        </div>
      </div>
      <span class="phase-pill">Fase 1: Probe proot</span>
    </header>

    <main>
      <!-- Kartu Pembuktian proot (Fase 1) -->
      <section class="card" style="border-color: ${isProbeSuccess ? 'rgba(16, 185, 129, 0.4)' : isProbeFailed ? 'rgba(244, 63, 94, 0.4)' : 'var(--border-color)'};">
        <div class="card-title">
          <span>Pembuktian proot (Fase 1)</span>
          <span class="indicator ${probeIndicatorClass}"></span>
        </div>

        <div class="status-row">
          <span class="status-label">Status Eksekusi</span>
          <span class="status-value">
            ${
              isProbing
                ? 'Menjalankan Probe...'
                : isProbeSuccess
                ? '✅ Berhasil (LULUS)'
                : isProbeFailed
                ? '❌ Gagal'
                : 'Belum Diuji'
            }
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Arsitektur (uname -m)</span>
          <span class="status-value">${pRes ? pRes.uname : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Node.js (node -v)</span>
          <span class="status-value" style="color: ${pRes?.node_version.startsWith('v') ? 'var(--accent-emerald)' : 'inherit'};">
            ${pRes ? pRes.node_version : '-'}
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Git (git --version)</span>
          <span class="status-value">${pRes ? pRes.git_version : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Koneksi Jaringan (curl)</span>
          <span class="status-value">${pRes ? pRes.network_check : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Durasi Eksekusi</span>
          <span class="status-value">${pRes ? `${pRes.elapsed_ms} ms` : '-'}</span>
        </div>

        <div style="margin-top: 14px;">
          <label class="toggle-row">
            <input type="checkbox" id="chk-no-seccomp" ${state.noSeccomp ? 'checked' : ''} ${isProbing ? 'disabled' : ''} />
            <span>Aktifkan PROOT_NO_SECCOMP=1 (Fallback)</span>
          </label>
        </div>

        <div style="margin-top: 14px;">
          <button id="btn-probe" class="btn-emerald" style="width: 100%;" ${isProbing ? 'disabled' : ''}>
            <span>${isProbing ? '⏳ Mengeksekusi Linux proot...' : '🚀 Jalankan Probe proot'}</span>
          </button>
        </div>

        ${state.probeError ? `<div class="console-box" style="margin-top: 12px; color: var(--accent-rose);">${state.probeError}</div>` : ''}
      </section>

      <!-- Kartu IPC Host -->
      <section class="card">
        <div class="card-title">
          <span>Status IPC Core (Host)</span>
          <span class="indicator ${indicatorClass}"></span>
        </div>

        <div class="status-row">
          <span class="status-label">Komunikasi Rust JNI</span>
          <span class="status-value">
            ${isPinging ? 'Menguji...' : isConnected ? 'Terhubung (Online)' : isError ? 'Gagal' : 'Siap'}
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Pesan Respons</span>
          <span class="status-value">${state.lastPing ? state.lastPing.message : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Latensi IPC</span>
          <span class="status-value">${state.latencyMs !== null ? `${state.latencyMs} ms` : '-'}</span>
        </div>

        <div style="margin-top: 12px;">
          <button id="btn-ping" class="btn" style="width: 100%; padding: 10px;" ${isPinging ? 'disabled' : ''}>
            <span>${isPinging ? 'Menguji...' : '🔄 Uji Ulang Ping Core'}</span>
          </button>
        </div>
      </section>
    </main>

    <footer class="footer">
      Deploy Instant &bull; Fase 1 Pembuktian proot &bull; Ponytail Clean Architecture
    </footer>
  `;

  // Attach event listeners
  const btnPing = document.getElementById('btn-ping');
  if (btnPing) {
    btnPing.addEventListener('click', handlePing);
  }

  const btnProbe = document.getElementById('btn-probe');
  if (btnProbe) {
    btnProbe.addEventListener('click', handleProbe);
  }

  const chkNoSeccomp = document.getElementById('chk-no-seccomp') as HTMLInputElement | null;
  if (chkNoSeccomp) {
    chkNoSeccomp.addEventListener('change', (e) => {
      state.noSeccomp = (e.target as HTMLInputElement).checked;
    });
  }
}

async function handlePing() {
  state.status = 'pinging';
  state.errorMessage = null;
  render();

  const startTime = performance.now();
  try {
    const res = await pingBackend();
    const elapsed = Math.round(performance.now() - startTime);
    state.lastPing = res;
    state.latencyMs = elapsed;
    state.status = 'connected';
  } catch (err: any) {
    state.status = 'error';
    state.errorMessage = String(err?.message || err);
  } finally {
    render();
  }
}

async function handleProbe() {
  state.probeStatus = 'running';
  state.probeError = null;
  render();

  try {
    const res = await runProotProbe(state.noSeccomp);
    state.probeResult = res;
    if (res.success) {
      state.probeStatus = 'success';
    } else {
      state.probeStatus = 'failed';
      state.probeError = res.error || 'Probe gagal';
    }
  } catch (err: any) {
    state.probeStatus = 'failed';
    state.probeError = String(err?.message || err);
  } finally {
    render();
  }
}

// Initial render & automatic ping
render();
handlePing();
