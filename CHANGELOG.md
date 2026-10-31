# Changelog

Semua perubahan pada proyek **Deploy Instant** akan didokumentasikan di file ini.
Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.0.0/).

## [0.4.0] - 2026-10-31
### Ditambahkan
- **Fase 3 — Data Project dan Dashboard (Tanpa Eksekusi)**:
  - Database embedded SQLite menggunakan `rusqlite 0.31` (fitur `bundled`, nol dependensi eksternal sistem host) dengan foreign keys, mode WAL, dan migrasi schema PRD 8.4 (`projects`, `env_vars`, `settings`).
  - Lapisan data `storage::db::Database` dengan validasi keamanan ketat:
    - Nama project: alphanumeric + dash/underscore, anti-shell injection (menolak `;`, `&`, `|`, `..`, spasi).
    - Repo URL: protokol HTTPS/HTTP/Git valid, menolak karakter berbahaya.
    - Branch: format git ref valid tanpa traversal.
    - Port internal: validasi port non-root 1024–65535.
    - Environment variables: key identifier standar (`^[a-zA-Z_][a-zA-Z0-9_]*$`).
  - Perintah IPC Tauri di `commands::project.rs`:
    - `project_create`, `project_list`, `project_get`, `project_update`, `project_delete`.
    - `setting_get`, `setting_set`, `setting_list`.
  - Desain antarmuka modern adaptasi referensi visual "Open Pi":
    - Palet warna light off-white `#f4f5f8`, kartu rounded white `#ffffff` dengan border halus dan soft elevation shadow.
    - Top navigation bar: tombol Settings, judul app, pintasan Diagnostik, dan tombol buat project.
    - Overview card: Engine status pill (`Healthy` / `Siap`), subjudul toolchain, serta 3-kolom metrik ringkas (Status, Total Project Aktif, Ruang Rootfs ~343 MB).
    - Project Card: inisial ikon dinamis, nama project, badge branch, badge port, badge auto-start, baris repo Git monospace, ringkasan perintah build/start, serta aksi Ubah dan Hapus.
    - Empty state elegan: ikon folder, deskripsi edukasi, dan tombol tambah project pertama.
    - Form modal bergaya bottom sheet modern: pengelompokan kartu (Informasi Repository, Konfigurasi Eksekusi, Environment Variables editor) dan footer aksi sticky.
    - Modal dialog konfirmasi hapus berperingatan jelas dengan aksi destruktif aman.
    - Pengaturan aplikasi: selector dwibahasa (Bahasa Indonesia & English) yang tersimpan persisten di SQLite, informasi versi, dan pintasan diagnostik sistem.
  - Verifikasi langsung di HP fisik (`462a07fa0405`): Create, Read, Update, Delete, Persistence saat restart app (`force-stop`), dan Language switch lolos 100%.

## [0.3.0] - 2026-10-31
### Ditambahkan
- **Fase 2 — Rootfs Lengkap dan Setup Pertama**:
  - Pembuatan generator rootfs `scripts/prepare-rootfs.py` berbasis format USTAR dengan normalisasi Debian Merged-USR (`/bin`, `/lib`, `/sbin` -> `usr/*`), menyatukan Debian Bookworm base, Node.js v24.22.1 LTS, PM2, Cloudflared tunnel arm64, Git 2.39.5, Curl 7.88.1, dan CA-Certificates dalam satu arsip terkompresi `rootfs.tar.gz` (168.20 MB, SHA-256 tersimpan di `rootfs/checksums.txt`).
  - Implementasi alur ekstraksi tangguh di Rust `core::rootfs::extract_rootfs_with_progress`:
    - Pengecekan ruang disk host (`MIN_FREE_DISK_MB` ≥ 600 MB via `libc::statvfs`).
    - Verifikasi integritas checksum SHA-256 streaming sebelum ekstraksi.
    - Mekanisme atomic staging (`rootfs.extracting` -> `rootfs`) untuk mencegah korupsi data saat proses terputus di tengah jalan.
    - Penulisan otomatis DNS resolver publik (`1.1.1.1`, `8.8.8.8`) dan penanda versi `.version` (`24.22.1-lts`).
  - Antarmuka First-Run Setup interaktif di UI (`ui/src/main.ts`):
    - Banner edukatif inisialisasi lingkungan server.
    - Indikator ketersediaan ruang disk dan arsip rootfs.
    - Animated progress bar real-time yang memantau tahapan disk check, hash verification, ekstraksi tar, konfigurasi jaringan, hingga toolchain verification via event IPC Tauri `setup-progress`.
    - Error handling mandiri dengan tombol "Coba Lagi Setup".
  - Verifikasi toolchain pasca-setup otomatis dan on-demand:
    - Node.js: `v24.22.1` LTS
    - PM2: `7.1.1` (daemon spawned aktif di `/root/.pm2`)
    - Cloudflared: `2026.10.7`
    - Git & Curl: `git version 2.39.5` dan HTTP/2 200 OK
    - Ukuran terpakai di disk internal: `~343 MB` (target ≤ 400 MB)
    - Durasi eksekusi probe proot container: 882 ms.

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
