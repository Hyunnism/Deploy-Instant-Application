# Changelog

Semua perubahan pada proyek **Deploy Instant** akan didokumentasikan di file ini.
Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.0.0/).

## [0.2.0] - 2026-10-28
### Ditambahkan
- **Fase 1 — Pembuktian proot (Risiko Terbesar Lolos)**:
  - Integrasi standalone binary PRoot 5.4.0 (`libproot.so` dan `libproot-loader.so`) ke dalam `jniLibs/arm64-v8a` dengan ekstraksi otomatis ke `nativeLibraryDir` Android (`-rwxr-xr-x`).
  - Implementasi eksekutor proot terpusat di Rust `core::exec::proot_exec()` dengan penanganan lingkungan (`PATH`, `HOME`, `LANG`, `PROOT_TMP_DIR`) dan opsi fallback `PROOT_NO_SECCOMP`.
  - Implementasi deteksi dan ekstraksi rootfs di `core::rootfs::ensure_rootfs_ready()`, konfigurasi DNS resolver `/etc/resolv.conf`, serta penanganan direktori temporer sandbox.
  - Implementasi perintah IPC `system_run_probe` di Rust dan kartu antarmuka UI Probe di WebView dengan status eksekusi real-time.
  - Verifikasi langsung pada perangkat fisik Android arm64:
    - Arsitektur Linux `uname -m`: `aarch64`
    - Node.js runtime `node -v`: `v20.18.0`
    - DNS resolver & konektivitas jaringan outbound: `HTTPS 200 OK (registry.npmjs.org)`
    - Latensi eksekusi proot probe: ~1775 ms
    - Kepatuhan W^X Android 10+ terbukti aman tanpa izin root.

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
