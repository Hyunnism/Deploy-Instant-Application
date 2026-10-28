import './styles/index.css';
import { pingBackend } from './api';
import { state } from './state';

const appEl = document.getElementById('app')!;

function render() {
  const isPinging = state.status === 'pinging';
  const isConnected = state.status === 'connected';
  const isError = state.status === 'error';

  const indicatorClass = isPinging ? 'pinging' : isConnected ? 'connected' : isError ? 'error' : '';

  appEl.innerHTML = `
    <header class="header">
      <div class="brand">
        <div class="logo-badge">⚡</div>
        <div class="brand-text">
          <h1>Deploy Instant</h1>
          <p>Mini VPS di dalam HP</p>
        </div>
      </div>
      <span class="phase-pill">Fase 0: Fondasi</span>
    </header>

    <main>
      <section class="card">
        <div class="card-title">
          <span>Status IPC Core</span>
          <span class="indicator ${indicatorClass}"></span>
        </div>

        <div class="status-row">
          <span class="status-label">Status Koneksi</span>
          <span class="status-value">
            ${isPinging ? 'Menghubungkan...' : isConnected ? 'Terhubung (Online)' : isError ? 'Gagal' : 'Siap'}
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Pesan Respons</span>
          <span class="status-value">${state.lastPing ? state.lastPing.message : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Arsitektur Target</span>
          <span class="status-value">${state.lastPing ? state.lastPing.arch : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Latensi IPC</span>
          <span class="status-value">${state.latencyMs !== null ? `${state.latencyMs} ms` : '-'}</span>
        </div>
      </section>

      <section class="card">
        <div class="card-title">
          <span>Spesifikasi Lingkungan</span>
        </div>

        <div class="status-row">
          <span class="status-label">Framework</span>
          <span class="status-value">Tauri 2 (Mobile)</span>
        </div>

        <div class="status-row">
          <span class="status-label">Package ID</span>
          <span class="status-value">com.hyunnism.deployinstant</span>
        </div>

        <div class="status-row">
          <span class="status-label">Target SDK</span>
          <span class="status-value">28 (Android 9+)</span>
        </div>

        <div class="status-row">
          <span class="status-label">ABI Filter</span>
          <span class="status-value">arm64-v8a</span>
        </div>
      </section>

      <section class="action-container">
        <button id="btn-ping" class="btn" ${isPinging ? 'disabled' : ''}>
          <span>${isPinging ? '⏳ Menguji IPC...' : '🔄 Uji Ping Rust Core'}</span>
        </button>

        ${state.errorMessage ? `<div class="console-box" style="color: var(--accent-rose);">${state.errorMessage}</div>` : ''}
      </section>
    </main>

    <footer class="footer">
      Deploy Instant &bull; Monorepo PRD 12A.1 &bull; Ponytail Clean Architecture
    </footer>
  `;

  const btnPing = document.getElementById('btn-ping');
  if (btnPing) {
    btnPing.addEventListener('click', handlePing);
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

// Initial render & automatic first ping
render();
handlePing();
