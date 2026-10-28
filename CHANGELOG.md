# Changelog

Semua perubahan pada proyek **Deploy Instant** akan didokumentasikan di file ini.
Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.0.0/).

## [0.1.0] - 2026-10-28
### Ditambahkan
- **Fase 0 — Fondasi Proyek dan Toolchain**:
  - Inisialisasi struktur monorepo sesuai PRD Bagian 12A.1 (`docs/`, `src-tauri/`, `ui/`, `rootfs/`, `scripts/`, `.github/`).
  - Inisialisasi Tauri 2 Android shell dengan target `aarch64-linux-android`.
  - Konfigurasi Gradle: `applicationId = com.hyunnism.deployinstant`, `minSdk = 28`, `targetSdk = 28`, `ndk.abiFilters = ['arm64-v8a']`.
  - Implementasi perintah IPC `system_ping` di backend Rust dan antarmuka UI di WebView (Vite + TypeScript).
  - Skrip otomatisasi pengujian `scripts/check.ps1` dan instalasi HP `scripts/dev-install.ps1`.
  - Workflow CI GitHub Actions untuk format, clippy, dan automated test.
  - Dokumentasi arsitektur, matriks perangkat, dan ADR 0001 (proot vs Termux).
