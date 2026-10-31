import './styles/index.css';
import {
  checkSetupStatus,
  onSetupProgress,
  pingBackend,
  runProotProbe,
  startSetup,
  verifySetup,
} from './api';
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
  const probeIndicatorClass = isProbing
    ? 'pinging'
    : isProbeSuccess
    ? 'connected'
    : isProbeFailed
    ? 'error'
    : '';

  const isExtracting = state.setupStatus === 'extracting';
  const isVerifying = state.setupStatus === 'verifying';
  const isSetupReady = state.setupInfo?.is_ready ?? false;
  const setupIndicatorClass = isExtracting || isVerifying
    ? 'pinging'
    : isSetupReady
    ? 'connected'
    : state.setupStatus === 'error'
    ? 'error'
    : '';

  const pRes = state.probeResult;
  const vRes = state.setupVerification;
  const progress = state.setupProgress;

  appEl.innerHTML = `
    <header class="header">
      <div class="brand">
        <div class="logo-badge">⚡</div>
        <div class="brand-text">
          <h1>Deploy Instant</h1>
          <p>Mini VPS di dalam HP</p>
        </div>
      </div>
      <span class="phase-pill">Fase 2: Rootfs Setup</span>
    </header>

    <main>
      ${
        !isSetupReady
          ? `
      <!-- Kartu Penyiapan Lingkungan Server (First-Run Setup) -->
      <section class="card" style="border-color: ${
        state.setupStatus === 'error' ? 'rgba(244, 63, 94, 0.4)' : 'var(--border-active)'
      };">
        <div class="card-title">
          <span>Penyiapan Server (Fase 2)</span>
          <span class="indicator ${setupIndicatorClass}"></span>
        </div>

        <div class="setup-banner">
          <div class="setup-banner-icon">📦</div>
          <div class="setup-banner-content">
            <h3>Inisialisasi Lingkungan Linux</h3>
            <p>Aplikasi akan mengekstrak rootfs Debian lengkap berisi Node.js v24 LTS, PM2, Cloudflared, dan Git ke direktori internal aplikasi.</p>
          </div>
        </div>

        <div class="status-row">
          <span class="status-label">Ruang Disk Tersedia</span>
          <span class="status-value">${
            state.setupInfo ? `${state.setupInfo.free_space_mb} MB` : 'Memeriksa...'
          }</span>
        </div>

        <div class="status-row">
          <span class="status-label">Kebutuhan Minimum</span>
          <span class="status-value">
            ${state.setupInfo ? `${state.setupInfo.required_space_mb} MB` : '600 MB'}
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Versi Target</span>
          <span class="status-value" style="color: var(--accent-cyan); font-family: var(--font-mono);">
            24.22.1-lts
          </span>
        </div>

        ${
          isExtracting || isVerifying
            ? `
        <div class="progress-box">
          <div class="progress-info">
            <span class="progress-message">${progress?.message || 'Memproses...'}</span>
            <span style="font-weight: 600; font-family: var(--font-mono);">${progress?.percent ?? 0}%</span>
          </div>
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: ${progress?.percent ?? 0}%;"></div>
          </div>
        </div>
        `
            : ''
        }

        ${
          state.setupError
            ? `<div class="console-box" style="margin-top: 12px; color: var(--accent-rose);">${state.setupError}</div>`
            : ''
        }

        <div style="margin-top: 16px;">
          <button id="btn-start-setup" class="btn-emerald" style="width: 100%;" ${
            isExtracting || isVerifying ? 'disabled' : ''
          }>
            <span>${
              isExtracting
                ? `⏳ Mengekstrak Rootfs (${progress?.percent ?? 0}%)...`
                : isVerifying
                ? '🔍 Memverifikasi Toolchain...'
                : state.setupStatus === 'error'
                ? '🔄 Coba Lagi Setup'
                : '🚀 Siapkan Lingkungan Server'
            }</span>
          </button>
        </div>
      </section>
      `
          : `
      <!-- Kartu Toolchain Lingkungan Linux (Ready) -->
      <section class="card" style="border-color: rgba(16, 185, 129, 0.4);">
        <div class="card-title">
          <span>Lingkungan Server Siap</span>
          <span class="badge-tag">✅ AKTIF</span>
        </div>

        <div class="toolchain-grid">
          <div class="toolchain-card">
            <span class="tool-name">Node.js</span>
            <span class="tool-version">${
              vRes ? vRes.node_version : state.setupInfo?.version || 'v24.22.1'
            }</span>
          </div>
          <div class="toolchain-card">
            <span class="tool-name">PM2</span>
            <span class="tool-version">${vRes ? vRes.pm2_version : '5.4.3'}</span>
          </div>
          <div class="toolchain-card">
            <span class="tool-name">Cloudflared</span>
            <span class="tool-version">${vRes ? vRes.cloudflared_version : 'Terpasang'}</span>
          </div>
          <div class="toolchain-card">
            <span class="tool-name">Git & Curl</span>
            <span class="tool-version">${vRes ? vRes.git_version : '2.39.5'}</span>
          </div>
        </div>

        <div class="status-row" style="margin-top: 14px;">
          <span class="status-label">Ukuran Direktori Rootfs</span>
          <span class="status-value">${
            vRes?.disk_used_mb ? `~${vRes.disk_used_mb} MB` : 'Siap'
          }</span>
        </div>

        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <button id="btn-verify-toolchain" class="btn" style="flex: 1; padding: 10px; font-size: 13px;" ${
            isVerifying ? 'disabled' : ''
          }>
            <span>${isVerifying ? '🔍 Menguji...' : '🔍 Verifikasi Toolchain'}</span>
          </button>
          <button id="btn-reinstall-setup" class="btn" style="flex: 1; padding: 10px; font-size: 13px; background: rgba(255,255,255,0.05); box-shadow: none;">
            <span>🔄 Re-install</span>
          </button>
        </div>
      </section>

      <!-- Kartu Pembuktian proot (Fase 1 Regression & Validation) -->
      <section class="card" style="border-color: ${
        isProbeSuccess
          ? 'rgba(16, 185, 129, 0.4)'
          : isProbeFailed
          ? 'rgba(244, 63, 94, 0.4)'
          : 'var(--border-color)'
      };">
        <div class="card-title">
          <span>Uji Eksekusi Container proot</span>
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
          <span class="status-label">Arsitektur Kernel</span>
          <span class="status-value">${pRes ? pRes.uname : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Node.js Runtime</span>
          <span class="status-value" style="color: ${
            pRes?.node_version.startsWith('v') ? 'var(--accent-emerald)' : 'inherit'
          }; font-family: var(--font-mono);">
            ${pRes ? pRes.node_version : '-'}
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Git CLI</span>
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
            <input type="checkbox" id="chk-no-seccomp" ${state.noSeccomp ? 'checked' : ''} ${
              isProbing ? 'disabled' : ''
            } />
            <span>Aktifkan PROOT_NO_SECCOMP=1 (Fallback)</span>
          </label>
        </div>

        <div style="margin-top: 14px;">
          <button id="btn-probe" class="btn-emerald" style="width: 100%;" ${
            isProbing ? 'disabled' : ''
          }>
            <span>${isProbing ? '⏳ Mengeksekusi Linux proot...' : '🚀 Jalankan Probe proot'}</span>
          </button>
        </div>

        ${
          state.probeError
            ? `<div class="console-box" style="margin-top: 12px; color: var(--accent-rose);">${state.probeError}</div>`
            : ''
        }
      </section>
      `
      }

      <!-- Kartu IPC Host -->
      <section class="card">
        <div class="card-title">
          <span>Status IPC Core (Host)</span>
          <span class="indicator ${indicatorClass}"></span>
        </div>

        <div class="status-row">
          <span class="status-label">Komunikasi Rust JNI</span>
          <span class="status-value">
            ${
              isPinging
                ? 'Menguji...'
                : isConnected
                ? 'Terhubung (Online)'
                : isError
                ? 'Gagal'
                : 'Siap'
            }
          </span>
        </div>

        <div class="status-row">
          <span class="status-label">Pesan Respons</span>
          <span class="status-value">${state.lastPing ? state.lastPing.message : '-'}</span>
        </div>

        <div class="status-row">
          <span class="status-label">Latensi IPC</span>
          <span class="status-value">${
            state.latencyMs !== null ? `${state.latencyMs} ms` : '-'
          }</span>
        </div>

        <div style="margin-top: 12px;">
          <button id="btn-ping" class="btn" style="width: 100%; padding: 10px;" ${
            isPinging ? 'disabled' : ''
          }>
            <span>${isPinging ? 'Menguji...' : '🔄 Uji Ulang Ping Core'}</span>
          </button>
        </div>
      </section>
    </main>

    <footer class="footer">
      Deploy Instant &bull; Fase 2 Rootfs & Toolchain &bull; Ponytail Clean Architecture
    </footer>
  `;

  // Attach event listeners
  const btnPing = document.getElementById('btn-ping');
  if (btnPing) {
    btnPing.addEventListener('click', handlePing);
  }

  const btnStartSetup = document.getElementById('btn-start-setup');
  if (btnStartSetup) {
    btnStartSetup.addEventListener('click', handleStartSetup);
  }

  const btnVerify = document.getElementById('btn-verify-toolchain');
  if (btnVerify) {
    btnVerify.addEventListener('click', handleVerifyToolchain);
  }

  const btnReinstall = document.getElementById('btn-reinstall-setup');
  if (btnReinstall) {
    btnReinstall.addEventListener('click', () => {
      if (confirm('Apakah Anda yakin ingin mengekstrak ulang rootfs?')) {
        handleStartSetup();
      }
    });
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

async function handleCheckSetup() {
  state.setupStatus = 'checking';
  render();
  try {
    const res = await checkSetupStatus();
    state.setupInfo = res;
    if (res.is_ready) {
      state.setupStatus = 'ready';
      // Automatically run toolchain verification in background
      handleVerifyToolchain();
    } else {
      state.setupStatus = 'not_ready';
    }
  } catch (err: any) {
    state.setupStatus = 'error';
    state.setupError = String(err?.message || err);
  } finally {
    render();
  }
}

async function handleStartSetup() {
  state.setupStatus = 'extracting';
  state.setupError = null;
  state.setupProgress = { step: 'extracting', percent: 0, message: 'Memulai ekstraksi...' };
  render();

  try {
    const verifyRes = await startSetup();
    state.setupVerification = verifyRes;
    state.setupStatus = verifyRes.success ? 'ready' : 'error';
    if (!verifyRes.success) {
      state.setupError = verifyRes.error || 'Verifikasi sebagian toolchain gagal';
    }
    const info = await checkSetupStatus();
    state.setupInfo = info;
  } catch (err: any) {
    state.setupStatus = 'error';
    state.setupError = String(err?.message || err);
  } finally {
    render();
  }
}

async function handleVerifyToolchain() {
  state.setupStatus = state.setupInfo?.is_ready ? 'ready' : 'verifying';
  render();
  try {
    const vRes = await verifySetup();
    state.setupVerification = vRes;
  } catch (err: any) {
    console.error('Verify toolchain error:', err);
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

// Inisialisasi listener event setup progress & status awal
onSetupProgress((progress) => {
  state.setupProgress = progress;
  if (progress.step === 'error') {
    state.setupStatus = 'error';
    state.setupError = progress.message;
  }
  render();
});

render();
handlePing();
handleCheckSetup();
