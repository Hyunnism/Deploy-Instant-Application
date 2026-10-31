# Changelog

Semua perubahan pada proyek **Deploy Instant** akan didokumentasikan di file ini.
Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.0.0/).

## [0.6.0] - 2026-10-31
### Ditambahkan
- **Fase 5 — Build Production**:
  - Eksekusi build production di dalam rootfs sandbox via `core::exec::proot_exec()` (PRD Bagian 9.2 & 13 Fase 5):
    - Deteksi otomatis jenis dependensi proyek: `if [ -f package-lock.json ]; then npm ci || npm install; elif [ -f package.json ]; then npm install; else echo '==> Tidak ada package.json, melewati npm install.'; fi && <custom_build_cmd>`.
    - Repositori tanpa `package.json` (seperti static site atau script HTML/CSS) dapat melewati fase dependensi dengan mulus tanpa error palsu.
    - Set environment variable produksi otomatis `NODE_ENV=production` beserta seluruh custom environment variables project.
    - Timeout otomatis 15 menit (900 detik) untuk mencegah build tanpa akhir.
  - Streaming stdout dan stderr real-time ke antarmuka aplikasi melalui event Tauri `build:log`.
  - Pembatalan proses build aman & responsif (`ActiveBuildManager`):
    - Pembatalan via oneshot channel terkoordinasi antar thread.
    - Alokasi process group baru (`process_group(0)`) pada proses build sandboxed sehingga seluruh hierarki anak proses (misal subshell dan npm script) dapat dihentikan seketika dengan `SIGKILL` ke process group ID tanpa residu proses zombie.
    - Penghentian instan streaming task tanpa memblokir thread IPC.
  - Persistensi log build dan rotasi otomatis:
    - Log disimpan di storage aplikasi host pada direktori `logs/<project_name>/build-<timestamp>.log`.
    - Mekanisme rotasi otomatis: mempertahankan maksimal 5 file log terbaru dan membersihkan log lama secara otomatis.
    - Proteksi traversal path pada pembacaan file log historis (`read_build_log`).
  - Tracking status build pada SQLite database:
    - Kolom `last_build_ok` (boolean) dan `last_build_at` (timestamp integer) pada tabel `projects`.
    - Badge status dinamis pada kartu project di dashboard: badge hijau `✓ Build Lulus • baru saja` atau badge merah `✕ Build Gagal • baru saja`.
    - Badge build dapat diklik langsung kapan saja untuk membuka riwayat log build terakhir.
  - Antarmuka Modal Build bergaya Open Pi:
    - Desain bottom sheet modal dengan drag pill, hero card project, dan status badge.
    - Tombol aksi kontekstual: "Batalkan Build" saat proses berjalan, otomatis berganti menjadi "Build Ulang" saat proses selesai.
    - Kotak Ringkasan Error (`build-error-summary`) dengan latar belakang soft-red yang mengekstraksi 50 baris terakhir saat build gagal.
    - Kontainer terminal monospace dengan append langsung ke DOM tanpa re-render halaman penuh (flicker-free & memory-safe dengan batas 5.000 baris).
    - Tombol "Salin" untuk menyalin seluruh output log build ke clipboard.
  - Pengujian langsung di perangkat fisik Android (`462a07fa0405`):
    - Build repo sukses 100% dan badge kartu ter-update ke `✓ Build Lulus • baru saja`.
    - Build dengan error terdeteksi akurat dan ringkasan error muncul di kotak merah `build-error-summary`.
    - Fitur pembatalan ("Batalkan Build") terverifikasi menghentikan proses long-running seketika dan mencatat `Build dibatalkan oleh pengguna`.
    - Rotasi log terverifikasi langsung via adb shell: hanya 5 file log terbaru yang dipertahankan.

## [0.5.0] - 2026-10-31
### Ditambahkan
- **Fase 4 — Clone dan Pull dari GitHub**:
  - Eksekusi Git aman di container Linux sandbox sandbox melalui `core::exec::proot_exec()`:
    - `git clone --depth 1 --branch <branch> <repo_url> /app/<project_name>` ke direktori isolasi container `/app/<project_name>`.
    - `git pull origin <branch>` dengan parsing perubahan commit otomatis (*"Already up to date"* vs *"Pembaruan kode berhasil diambil"*).
    - Inspeksi status repositori lokal `get_project_repo_status`: deteksi keberadaan `.git`, commit hash pendek terkini (`git rev-parse --short HEAD`), commit message (`git log -1 --pretty=%s`), dan nama branch aktif.
  - Streaming stdout & stderr real-time ke antarmuka aplikasi melalui event Tauri `git:log`.
  - Mekanisme keamanan kredensial zero-leakage (PRD Bagian 12A.3):
    - Penyimpanan GitHub Personal Access Token (PAT) terenkripsi AES-256-GCM menggunakan master key 256-bit dan 96-bit random nonce di `storage::secrets.rs`.
    - Injeksi kredensial Git aman via helper `GIT_ASKPASS` berumur pendek di `/tmp/.git_askpass_<timestamp>.sh` dengan izin POSIX `0700` dan pembersihan otomatis saat proses selesai (`Drop`).
    - URL repositori di `.git/config`, command string, dan daftar proses `ps` selalu bersih tanpa username/token.
    - Sanitasi otomatis output log: token PAT dan URL berotentikasi disamarkan dengan `******`.
  - Pembersihan otomatis pada kegagalan: folder `/app/<project_name>` langsung dihapus bersih secara rekursif bila proses clone terputus atau gagal, mencegah residu direktori korup.
  - Antarmuka Terminal Git dan Pengaturan Kredensial:
    - Tombol aksi kontekstual pada Kartu Project: Tombol biru "Clone Repo" saat repositori belum diunduh, otomatis berganti menjadi tombol "Pull" saat repositori sudah terkloning.
    - Badge metadata Git: badge commit hash (misal `#d0dd1f6`), branch, dan ringkasan pesan commit terakhir.
    - Modal Log Terminal Git streaming real-time dengan status eksekusi, warna terminal log, tombol "Salin Log", dan opsi tutup.
    - Bagian GitHub PAT di menu Pengaturan: status konfigurasi (Terkonfigurasi Aktif / Belum Diatur), input token tersamar dengan tombol toggle mata (lihat/sembunyikan), tombol Simpan Token dan Hapus Token, serta panduan scope `repo`.
  - Pengujian langsung di perangkat fisik Android (`462a07fa0405`):
    - Clone repo publik `octocat/Spoon-Knife` sukses 100% dengan streaming log real-time.
    - Pembersihan otomatis direktori saat branch salah/gagal clone terverifikasi 100% bersih tanpa residu.
    - Enkripsi PAT di SQLite diperiksa langsung via adb shell: nilai tersimpan berupa hex AES-256-GCM, zero plaintext.
    - Git pull mendeteksi status *"Already up to date"* secara akurat.

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
    - Overview card: Engine status pill (`Healthy` / `Siap`), subjudul toolchain, serta 3-kolom metrik ringkas (Status, Total Project Aktif, Ruang Rootfs ~343 MB). Interaktif dan dapat diklik untuk membuka modal rincian spesifikasi toolchain (Node.js, PM2, Cloudflared, Git, Rootfs disk) dan tombol pasang/periksa ulang toolchain terintegrasi.
    - Project Card: inisial ikon dinamis, nama project, badge branch, badge port, badge auto-start, baris repo Git monospace, ringkasan perintah build/start, serta aksi Ubah dan Hapus.
    - Empty state elegan: ikon folder, deskripsi edukasi, dan tombol tambah project pertama.
    - Form modal bergaya bottom sheet modern: pengelompokan kartu (Informasi Repository, Konfigurasi Eksekusi, Environment Variables editor) dan footer aksi sticky dengan penyesuaian tinggi natural dan padding aman perangkat (`env(safe-area-inset-bottom)`).
    - Modal dialog konfirmasi hapus berperingatan jelas dengan desain pure white `#ffffff` tanpa bayangan abu-abu dan aksi destruktif aman.
    - Safe-area header adaptif: penambahan padding atas `max(32px, env(safe-area-inset-top, 32px))` agar header tidak menempel ke status bar/kamera notch Android.
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
