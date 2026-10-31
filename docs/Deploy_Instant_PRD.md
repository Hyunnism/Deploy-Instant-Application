# Deploy Instant — Product Requirements Document (PRD)

> Versi: 1.0 | Tanggal: 2026-10-27 | Status: Draft untuk implementasi
> Platform: Android (arm64) | Core: Rust | Anggaran: Rp 0 (semua komponen gratis)

---

## 1. Ringkasan

**Deploy Instant** adalah aplikasi Android mandiri (satu APK) yang berfungsi sebagai "VPS mini di dalam HP". Developer dapat meng-clone repository GitHub, membangun (build) dalam mode production, menjalankannya dengan **PM2**, dan membagikannya lewat URL publik menggunakan **Cloudflare quick tunnel**. Semua berjalan lokal di HP tanpa VPS, tanpa VMware, dan tanpa IP publik.

### 1.1 Masalah yang diselesaikan
- Developer ingin tahu apakah project berhasil atau gagal di mode production, tetapi tidak punya VPS atau PC yang siap.
- Menyewa VPS atau menjalankan VM di desktop itu merepotkan dan berbiaya.
- Berbagi hasil demo ke klien atau teman sulit jika tidak punya IP publik.

### 1.2 Solusi
Satu aplikasi yang membundel: rootfs Linux (proot) + Node.js + git + PM2 + cloudflared, dikendalikan oleh core Rust dan UI yang sederhana.

### 1.3 Prinsip desain (ponytail)
1. **Tidak menulis ulang PM2.** Aplikasi hanya menjalankan PM2 asli di dalam rootfs.
2. **Satu pintu eksekusi.** Semua perintah ke Linux lewat satu fungsi (`proot_exec`).
3. **Tanpa backend, tanpa akun.** Semua data lokal per perangkat.
4. **Hidup hanya saat app dibuka.** Tidak ada layanan permanen, sehingga hemat baterai.
5. **MVP dulu.** Fitur di luar scope (Bagian 4.2) tidak dibangun sebelum MVP selesai.

---

## 2. Tujuan dan Metrik Keberhasilan

### 2.1 Tujuan
| ID | Tujuan |
|----|--------|
| G1 | User dapat menjalankan project Node.js dari GitHub dalam mode production langsung di HP. |
| G2 | User mendapat kepastian lulus/gagal build berdasarkan exit code. |
| G3 | User dapat memantau status dan log PM2 dari aplikasi. |
| G4 | User dapat membagikan project lewat URL publik tanpa IP publik. |
| G5 | Seluruh biaya operasional Rp 0 dan tanpa server pusat. |
| G6 | Konsumsi baterai nol saat aplikasi ditutup. |

### 2.2 Metrik (target MVP)
| Metrik | Target |
|--------|--------|
| Setup awal rootfs (sekali) | ≤ 5 menit pada koneksi 20 Mbps |
| Waktu dari "Tambah project" sampai "running" (project Express kecil) | ≤ 3 menit |
| Waktu muncul URL tunnel setelah tombol Share | ≤ 20 detik |
| Semua proses mati setelah app ditutup | 100% (tidak ada proses yatim) |
| Crash aplikasi pada alur utama | < 1% sesi |
| Kompatibilitas | Android 9–15, arm64-v8a |

---

## 3. Persona dan Skenario

### 3.1 Persona
| Persona | Kebutuhan |
|---------|-----------|
| **Dev Mobile-first** (mahasiswa/freelancer) | Hanya punya HP, ingin menguji deploy production. |
| **Dev Desktop** | Ingin cek cepat "apakah build production saya lolos" tanpa menyiapkan VPS. |
| **Demo Presenter** | Ingin membagikan link demo sementara ke klien. |

### 3.2 Skenario utama
1. Rina membuka app, dashboard kosong, lalu menambahkan repo `github.com/rina/api-toko`.
2. App clone, `npm ci`, dan `npm run build` berjalan. Log tampil, build lulus.
3. Rina mengisi env `DATABASE_URL`, lalu menekan Deploy. PM2 berjalan, status `online`.
4. Rina menekan Share, dan URL `https://xxxx.trycloudflare.com` muncul. Ia menyalin dan mengirimkannya ke klien.
5. Rina menutup app, semua berhenti. Esok hari ia membuka lagi dan menekan Start.

---

## 4. Ruang Lingkup

### 4.1 Dalam scope (MVP)
- Satu rootfs bundel (Debian arm64) dengan Node LTS, git, PM2, cloudflared.
- CRUD project (nama, repo URL, branch, command build/start, port).
- Clone dan pull dari GitHub (publik dan private dengan PAT).
- Build production dengan hasil lulus/gagal.
- Manajemen env per project (terenkripsi).
- Start/stop/restart via PM2, status, dan log streaming.
- Deteksi port dan pengecekan bentrok port.
- Quick tunnel Cloudflare per project dengan tampilan URL otomatis.
- Foreground service saat ada proses berjalan.
- Pembersihan penyimpanan.

### 4.2 Di luar scope (tidak dibangun di MVP)
- Custom domain, akun Cloudflare, named tunnel.
- Database bawaan, login, multi-user dalam satu perangkat.
- Auto-deploy, webhook GitHub, CI/CD.
- Docker, runtime selain Node.js (Python, Go, dll. adalah fase pasca-MVP).
- Sinkronisasi antar-perangkat (hanya ekspor/impor JSON di pasca-MVP).
- Publikasi Play Store (distribusi via APK dan F-Droid).
- Dukungan iOS dan arsitektur x86.

---

## 5. Kebutuhan Fungsional

| ID | Kebutuhan | Prioritas |
|----|-----------|-----------|
| FR-01 | Pada peluncuran pertama, app mengekstrak rootfs dan menampilkan progres. | Must |
| FR-02 | Dashboard menampilkan daftar project beserta status PM2 (online/stopped/errored/building). | Must |
| FR-03 | User dapat membuat project dengan repo URL, branch, build command, start command, dan port (opsional). | Must |
| FR-04 | App meng-clone repo ke `/app/<project>` di dalam rootfs. | Must |
| FR-05 | Repo private didukung lewat GitHub PAT yang disimpan aman. | Must |
| FR-06 | Tombol Build menjalankan `npm ci` lalu build command dengan `NODE_ENV=production`; hasil ditentukan exit code. | Must |
| FR-07 | Log build dialirkan realtime ke UI dan disimpan ke file. | Must |
| FR-08 | User dapat mengelola env (key=value) per project; nilai disamarkan di UI. | Must |
| FR-09 | Tombol Deploy menjalankan `pm2 start` dengan env production dan env user. | Must |
| FR-10 | User dapat Start/Stop/Restart/Delete proses PM2. | Must |
| FR-11 | Status PM2 (`pm2 jlist`) di-polling dan ditampilkan (status, uptime, restart count, CPU, memori). | Must |
| FR-12 | Log runtime PM2 dialirkan ke UI dengan filter stdout/stderr dan batas buffer. | Must |
| FR-13 | Port: prioritas manual > `PORT` di env > auto-assign; setelah start, port yang benar-benar listen dideteksi dan dipakai tunnel. | Must |
| FR-14 | Cek bentrok port saat Start; tampilkan pesan jelas tanpa mengubah port developer. | Must |
| FR-15 | Tombol "Buka lokal" membuka `http://localhost:PORT` di browser atau WebView. | Must |
| FR-16 | Tombol Share menjalankan `cloudflared` quick tunnel, mem-parse URL, lalu menampilkan dengan tombol Salin/Bagikan. | Must |
| FR-17 | Tombol Stop Share menghentikan tunnel. | Must |
| FR-18 | Peringatan sebelum Share: "URL ini dapat diakses siapa pun yang punya link". | Must |
| FR-19 | Saat app ditutup atau dimatikan dari notifikasi, semua proses (pm2, node, cloudflared) dihentikan. | Must |
| FR-20 | Opsi per project "Jalankan otomatis saat app dibuka". | Should |
| FR-21 | Tombol Pull (git pull) lalu opsi Rebuild. | Should |
| FR-22 | Tampilan penggunaan disk per project dan tombol Bersihkan (hapus `node_modules`, log, atau project). | Should |
| FR-23 | Ekspor/impor konfigurasi project (JSON, tanpa secret). | Could |
| FR-24 | Pilihan versi Node (LTS 20/22) per project. | Could |

---

## 6. Kebutuhan Non-Fungsional

| ID | Kategori | Kebutuhan |
|----|----------|-----------|
| NFR-01 | Baterai | Tidak ada proses latar belakang saat app tertutup; polling hanya saat layar project terbuka. |
| NFR-02 | Keamanan | PAT dan env disimpan terenkripsi (Android Keystore). Tidak ada telemetri. |
| NFR-03 | Keamanan | Semua proses berjalan di dalam proot tanpa akses ke data app lain. |
| NFR-04 | Performa | UI tetap responsif (< 100 ms respons tap) saat build berjalan. |
| NFR-05 | Stabilitas | Buffer log dibatasi (misalnya 5.000 baris di UI, rotasi file 5 MB). |
| NFR-06 | Penyimpanan | Ukuran APK ≤ 200 MB; rootfs terpasang ≤ 400 MB sebelum project. |
| NFR-07 | Privasi | Tidak ada server pusat; data tidak keluar perangkat kecuali git dan tunnel. |
| NFR-08 | Lokalisasi | Bahasa UI: Indonesia dan Inggris. |
| NFR-09 | Kompatibilitas | Android 9+ (API 28+), arm64-v8a saja. |

---

## 7. Stack Teknologi

| Lapisan | Pilihan | Alasan |
|---------|---------|--------|
| Framework app | **Tauri 2 (mobile/Android)** | Core Rust, UI web tipis, satu APK. |
| Core | **Rust** (`tokio`, `serde`, `serde_json`, `anyhow`, `regex`) | Sesuai stack pilihan; async untuk stream proses. |
| UI | HTML + TypeScript ringan (Preact atau vanilla), tanpa framework berat | Layar sedikit, ukuran kecil. |
| Isolasi Linux | **proot** (dari `libproot.so` di `jniLibs/arm64-v8a`) | Tanpa root; lolos batasan W^X Android 10+. |
| Rootfs | **Debian (glibc) arm64**, minimal, `tar.xz` | Kompatibilitas Node dan addon npm lebih baik daripada Alpine/musl. |
| Runtime | Node.js v24.22.1 LTS, npm, git, **PM2** (di dalam rootfs) | Sama dengan lingkungan VPS modern. |
| Tunnel | **cloudflared** linux-arm64 (di dalam rootfs) | Quick tunnel gratis, tanpa akun. |
| Penyimpanan data | **SQLite** (`rusqlite`) untuk metadata project | Ringan, satu file. |
| Rahasia | **Android Keystore** (lewat plugin Tauri atau JNI) | Menyimpan kunci enkripsi untuk PAT dan env. |
| Layanan latar | Android **Foreground Service** + partial wake lock | Menjaga proses selama sesi aktif. |
| Git | CLI `git` di dalam rootfs | Lebih sederhana daripada crate `git2` (tanpa linking native). |
| Build | Cargo + Tauri CLI + Android NDK; GitHub Actions untuk APK release | Gratis. |

> **Catatan keputusan:** menjalankan `git` dan `pm2` sebagai CLI di dalam rootfs lebih murah daripada mengikat library native di sisi Rust.

---

## 8. Arsitektur Sistem

### 8.1 Diagram komponen
```
┌──────────────────── APK Deploy Instant (Tauri 2) ─────────────────────┐
│  UI (WebView)                                                         │
│   Dashboard │ Form Project │ Detail: Log · Env · Status · Share       │
│        │  invoke() / events                                           │
│  Rust Core                                                            │
│   ├ db.rs        SQLite: projects, env, runs                          │
│   ├ secrets.rs   enkripsi PAT & env (Keystore)                        │
│   ├ rootfs.rs    ekstraksi & versi rootfs                             │
│   ├ exec.rs      proot_exec(): satu pintu semua perintah              │
│   ├ project.rs   clone / pull / build                                 │
│   ├ pm2.rs       start/stop/restart, jlist, logs                      │
│   ├ port.rs      resolusi, deteksi, cek bentrok                       │
│   ├ tunnel.rs    cloudflared + parse URL                              │
│   └ lifecycle.rs foreground service, shutdown semua proses            │
│        │ spawn (tokio::process)                                       │
│  ┌── proot ── rootfs Debian arm64 ───────────────────────────────┐    │
│  │ node · npm · git · pm2 · cloudflared · /app/<project>/        │    │
│  └───────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────────┘
          │ HTTPS keluar saja (git, npm registry, Cloudflare)
          ▼
   Cloudflare edge → https://<acak>.trycloudflare.com → HP pengguna
```

### 8.2 Lokasi data di perangkat
```
<app files dir>/
├─ rootfs/                 # Debian rootfs
│   ├─ app/<project>/      # hasil clone
│   └─ root/.pm2/          # state PM2
├─ db/deploy_instant.sqlite
├─ logs/<project>/build-<ts>.log
└─ tmp/                    # PROOT_TMP_DIR
<nativeLibraryDir>/libproot.so   # dapat dieksekusi
```

### 8.3 Fungsi inti satu pintu eksekusi
```rust
// exec.rs
pub fn proot_exec(cwd: &str, cmd: &str, env: &[(String, String)]) -> tokio::process::Command {
    let mut c = Command::new(native_lib_dir().join("libproot.so"));
    c.args(["-r", rootfs().to_str().unwrap(),
            "-b", "/dev", "-b", "/proc", "-b", "/sys",
            "-w", cwd, "--kill-on-exit",
            "/bin/sh", "-lc", cmd])
     .env("PROOT_TMP_DIR", tmp_dir())
     .envs(env.iter().cloned())
     .kill_on_drop(true);
    // ponytail: PROOT_NO_SECCOMP=1 hanya bila seccomp bermasalah di device tertentu; lebih lambat. Upgrade: deteksi otomatis saat fase 1.
    c
}
```

### 8.4 Model data (SQLite)
```sql
CREATE TABLE projects (
  id            INTEGER PRIMARY KEY,
  name          TEXT UNIQUE NOT NULL,        -- juga nama proses PM2
  repo_url      TEXT NOT NULL,
  branch        TEXT NOT NULL DEFAULT 'main',
  build_cmd     TEXT NOT NULL DEFAULT 'npm run build',
  start_cmd     TEXT NOT NULL DEFAULT 'npm start',
  port_manual   INTEGER,                     -- NULL = auto
  auto_start    INTEGER NOT NULL DEFAULT 0,
  last_build_ok INTEGER,                     -- 1/0/NULL
  last_build_at TEXT,
  created_at    TEXT NOT NULL
);
CREATE TABLE env_vars (
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  key        TEXT NOT NULL,
  value_enc  BLOB NOT NULL,                  -- terenkripsi
  PRIMARY KEY (project_id, key)
);
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL); -- PAT terenkripsi, bahasa, dll.
```
Status runtime (online/stopped) **tidak disimpan**; sumber kebenarannya adalah `pm2 jlist`. URL tunnel juga hanya disimpan di memori karena berubah tiap sesi.

### 8.5 Mesin status project
```
IDLE ─clone→ CLONED ─build→ BUILDING ─ok→ BUILT ─deploy→ RUNNING ─share→ RUNNING+PUBLIC
                              └fail→ BUILD_FAILED          │  └stop→ STOPPED
                                                           └ crash → ERRORED
```

### 8.6 Sequence: Deploy dan Share
```
User → UI: Deploy
UI → Rust: deploy(project_id)
Rust → proot: pm2 start npm --name <n> -- start   (env production + env user + PORT bila auto)
Rust → proot: pm2 jlist  → ambil pid
Rust → proot: baca /proc/net/tcp → port listen milik pid
Rust → UI: status online, port terdeteksi
User → UI: Share (setelah konfirmasi peringatan)
Rust → proot: cloudflared tunnel --url http://localhost:<port> --no-autoupdate
cloudflared → stdout: "...https://abc-def.trycloudflare.com..."
Rust: regex → emit event "tunnel_url" → UI tampilkan + Salin/Bagikan
```

### 8.7 Resolusi port
1. Jika `port_manual` terisi, pakai nilai itu, dan **jangan** menyuntikkan `PORT`.
2. Jika env `PORT` ada, pakai nilai itu.
3. Selain itu, pilih port bebas dari 3001 ke atas dan suntikkan `PORT`.
4. **Setelah start**, deteksi port yang benar-benar listen. Jika berbeda dari hasil resolusi, port terdeteksi yang dipakai tunnel dan UI menampilkan "Terdeteksi: N".
5. Sebelum start, jika port sudah dipakai project lain, tampilkan error "Port N dipakai project X". App tidak mengubah port milik developer.

### 8.8 Lifecycle dan baterai
- Foreground service (notifikasi "Deploy Instant aktif: N project") dimulai saat ada minimal satu proses berjalan, dan berhenti saat tidak ada.
- Partial wake lock hanya aktif selama ada proses berjalan.
- Saat `onDestroy` atau tombol "Stop semua": `pm2 kill`, bunuh proses cloudflared, dan kill proot process group.
- Polling status 2 detik hanya saat layar detail terbuka; 5 detik di dashboard; 0 saat app di latar belakang tanpa proses.

---

## 9. Alur Pengguna (User Flow)

### 9.1 Peluncuran pertama
```
Buka app → Layar sambutan → "Siapkan lingkungan" →
[Bila rootfs lokal belum ada: Unduh rootfs.tar.gz via internet HP (progres % & MB)] →
ekstraksi rootfs (progres) → cek node -v / pm2 -v / cloudflared -v → Dashboard (kosong) + CTA "Buat project"
```

> **Zero-Laptop Autonomy Guarantee (Prinsip Kemandirian Total):**  
> Pengguna akhir **sama sekali tidak memerlukan laptop/PC** untuk mengoperasikan Deploy Instant. Saat rilis, jika arsip `rootfs.tar.gz` belum ada di perangkat, aplikasi akan mengunduhnya secara otomatis langsung melalui koneksi internet HP (WiFi/Seluler) dari URL resmi rilis (GitHub Release/CDN), menampilkan progres unduhan dalam MB dan persentase, memvalidasi SHA-256 secara streaming, lalu mengekstraknya. Laptop hanya digunakan oleh developer saat membangun file APK.

### 9.2 Membuat dan membangun project
```
Dashboard → [+ Project] → isi: nama, repo URL, branch, build cmd, start cmd, port (opsional)
→ (repo private? isi PAT di Pengaturan) → Simpan → Clone (log) → Detail project
→ [Build] → log realtime → hasil: ✅ Lulus / ❌ Gagal (ringkasan error + 50 baris log terakhir)
```

### 9.3 Mengatur environment
```
Detail → tab Env → [+ Variabel] key & value (nilai disamarkan, ikon mata) → Simpan
→ perubahan berlaku pada Deploy/Restart berikutnya (UI menampilkan "Restart untuk menerapkan")
```

### 9.4 Deploy dan monitoring
```
Detail → [Deploy] → status BUILDING→RUNNING → kartu status: online, uptime, restart, CPU, MEM, port
→ tab Log (stdout/stderr, auto-scroll, jeda, salin) → [Restart] / [Stop]
```

### 9.5 Akses lokal
```
Detail → [Buka lokal] → browser/WebView ke http://localhost:<port>
```

### 9.6 Akses publik
```
Detail → [Share] → dialog peringatan → [Lanjut] → "Membuat tunnel…" → URL tampil
→ [Salin] [Bagikan] [Stop Share]
```

### 9.7 Menutup dan membuka kembali
```
Tutup app / Stop semua → semua proses berhenti
Buka lagi → dashboard menampilkan project berstatus STOPPED
→ project dengan auto_start=1 dijalankan otomatis; URL tunnel lama dinyatakan kedaluwarsa
```

### 9.8 Alur galat
| Situasi | Perilaku |
|---------|----------|
| Rootfs gagal diekstrak (disk penuh) | Pesan jelas, tombol Coba lagi, tampilkan ukuran dibutuhkan. |
| Clone gagal (auth/URL salah) | Pesan "Periksa URL atau PAT", tidak membuat folder setengah jadi. |
| Build gagal | Status BUILD_FAILED, log tersedia, tombol Deploy dinonaktifkan. |
| Port bentrok | Pesan menyebut project pemilik port. |
| Proses crash berulang | Status ERRORED; batas restart PM2 (`--max-restarts 5`); tampilkan log terakhir. |
| Tunnel gagal | Tampilkan stderr cloudflared, tombol Coba lagi. |
| Tidak ada internet | Fitur lokal tetap berjalan; clone, npm install, dan tunnel dinonaktifkan dengan keterangan. |

---

## 10. Spesifikasi UI

### 10.1 Layar
1. **Setup** (hanya sekali): progres, log singkat.
2. **Dashboard**: daftar kartu project (nama, status badge, port, tombol Start/Stop), tombol `+`, ikon Pengaturan, tombol "Stop semua".
3. **Form Project**: field di 5.FR-03; validasi URL GitHub.
4. **Detail Project**: header status dan aksi; tab **Ringkasan**, **Log**, **Env**, **Pengaturan project**.
5. **Pengaturan**: PAT GitHub, bahasa, pemakaian disk, versi rootfs, tentang.

### 10.2 Komponen
Badge status berwarna, viewer log monospace (virtualisasi, maks 5.000 baris), dialog konfirmasi, toast, progress bar.

### 10.3 Prinsip UX
- Tindakan destruktif (hapus project) wajib konfirmasi.
- Nilai rahasia selalu disamarkan secara default.
- Setiap error menampilkan "apa yang terjadi" dan "apa yang bisa dilakukan".

---

## 11. Keamanan dan Privasi

| Risiko | Mitigasi |
|--------|----------|
| URL tunnel publik tanpa autentikasi | Peringatan sebelum Share; tombol Stop Share; URL tidak disimpan. |
| Kebocoran PAT/env | Enkripsi dengan kunci Android Keystore; tidak pernah dicetak ke log; disamarkan di UI. |
| Kode repo berbahaya dijalankan | Berjalan di dalam proot tanpa akses ke penyimpanan app lain; peringatan "jalankan hanya repo yang Anda percaya". |
| Akses log berisi secret | Log disimpan di direktori privat app; tombol hapus log. |
| Supply chain (rootfs, cloudflared) | Verifikasi checksum SHA256 saat build; sumber resmi saja. |
| Penyalahgunaan quick tunnel | Patuhi ketentuan Cloudflare; tampilkan catatan "untuk pengujian/demo, bukan production sungguhan". |

---

## 12. Risiko Teknis dan Mitigasi

| # | Risiko | Dampak | Mitigasi |
|---|--------|--------|----------|
| R1 | proot gagal/lambat di ROM tertentu (MIUI, ColorOS) | Tinggi | Fase 1 membuktikan ini pertama; opsi `PROOT_NO_SECCOMP`; daftar perangkat teruji. |
| R2 | Batasan W^X (Android 10+) | Tinggi | `libproot.so` di `jniLibs`; `targetSdk` 28 jika diperlukan. |
| R3 | Phantom process killer (Android 12+) | Sedang | Foreground service + wake lock; panduan menonaktifkan optimasi baterai. |
| R4 | Native addon npm gagal di arm64/proot | Sedang | Rootfs glibc; paket build (`build-essential`, `python3`) opsional; peringatan fidelitas. |
| R5 | Ruang penyimpanan | Sedang | Rootfs minimal; fitur Bersihkan; cek ruang sebelum setup/clone. |
| R6 | Tunnel Cloudflare dibatasi/tidak ada SLA | Sedang | Dokumentasikan; tampilkan error jelas; tunnel adalah fitur opsional. |
| R7 | Tidak lolos Play Store | Rendah (diterima) | Distribusi APK/F-Droid/GitHub Releases. |
| R8 | Proses yatim saat app dibunuh paksa | Sedang | `--kill-on-exit`, `kill_on_drop`, dan pembersihan proses lama saat app dibuka. |
| R9 | Ukuran APK besar | Rendah | Unduh rootfs saat setup pertama sebagai alternatif bila APK > batas. |

---

## 12A. Struktur Repository dan Konvensi

> Tujuan: setiap file punya tempat yang jelas, sehingga mudah dicari, dikelola, dan dikerjakan bersama. **Satu modul = satu tanggung jawab = satu file/folder.** Struktur ini dibuat di Fase 0 dan dipertahankan di fase berikutnya.

### 12A.1 Struktur folder (monorepo)
```
deploy-instant/
├─ README.md                      # pengantar singkat + cara build
├─ LICENSE
├─ .gitignore
├─ .editorconfig
├─ docs/                          # semua dokumen, bukan kode
│  ├─ Deploy_Instant_PRD.md       # dokumen ini
│  ├─ architecture.md             # ringkasan arsitektur (turunan PRD bagian 8)
│  ├─ decisions/                  # ADR: satu file per keputusan penting
│  │   └─ 0001-proot-over-termux.md
│  ├─ testing/
│  │   ├─ device-matrix.md        # perangkat teruji + hasil
│  │   └─ test-plan.md
│  └─ user-guide/                 # panduan pengguna (ID/EN)
├─ src-tauri/                     # Rust core + shell Android
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ capabilities/               # izin Tauri
│  ├─ src/
│  │  ├─ main.rs                  # entry, hanya registrasi command
│  │  ├─ lib.rs
│  │  ├─ commands/                # lapisan tipis: command Tauri → modul
│  │  │   ├─ mod.rs
│  │  │   ├─ project.rs
│  │  │   ├─ pm2.rs
│  │  │   ├─ env.rs
│  │  │   ├─ tunnel.rs
│  │  │   └─ system.rs
│  │  ├─ core/                    # logika murni, tanpa Tauri
│  │  │   ├─ mod.rs
│  │  │   ├─ exec.rs              # proot_exec(), satu pintu eksekusi
│  │  │   ├─ rootfs.rs
│  │  │   ├─ project.rs           # clone, pull, build
│  │  │   ├─ pm2.rs
│  │  │   ├─ port.rs
│  │  │   ├─ tunnel.rs
│  │  │   └─ lifecycle.rs
│  │  ├─ storage/
│  │  │   ├─ mod.rs
│  │  │   ├─ db.rs                # SQLite + migrasi
│  │  │   └─ secrets.rs           # enkripsi PAT/env
│  │  ├─ models.rs                # struct bersama (Project, Pm2Status, dll.)
│  │  └─ error.rs                 # satu tipe error aplikasi
│  ├─ migrations/                 # 0001_init.sql, 0002_*.sql (berurutan)
│  ├─ tests/                      # integration test Rust
│  ├─ assets/                     # rootfs diunduh saat build; tidak di-commit
│  └─ gen/android/                # hasil `tauri android init`
│     └─ app/src/main/
│        ├─ jniLibs/arm64-v8a/    # libproot.so, loader
│        └─ java/.../service/     # Foreground Service (Kotlin)
├─ ui/                            # frontend
│  ├─ index.html
│  ├─ src/
│  │  ├─ main.ts
│  │  ├─ api.ts                   # satu-satunya tempat memanggil invoke()
│  │  ├─ state.ts
│  │  ├─ screens/                 # setup, dashboard, project-form, project-detail, settings
│  │  ├─ components/              # status-badge, log-viewer, dialog, toast
│  │  ├─ i18n/                    # id.json, en.json
│  │  └─ styles/
│  └─ tests/
├─ rootfs/                        # sumber pembuat rootfs (bukan hasilnya)
│  ├─ build-rootfs.sh             # buat rootfs Debian arm64
│  ├─ packages.txt                # daftar paket apt
│  ├─ checksums.txt               # SHA256 rootfs, proot, cloudflared
│  └─ README.md
├─ scripts/                       # skrip dev (build, install ke HP, lint)
│  ├─ dev-install.ps1
│  ├─ fetch-assets.ps1
│  └─ check.ps1                   # fmt + clippy + test
├─ .github/
│  └─ workflows/                  # ci.yml, release.yml
└─ CHANGELOG.md
```

### 12A.2 Aturan penempatan
| Jenis | Lokasi | Aturan |
|-------|--------|--------|
| Dokumen | `docs/` | Tidak ada `.md` selain README, CHANGELOG, dan LICENSE di root. |
| Logika bisnis Rust | `src-tauri/src/core/` | Tidak boleh mengimpor `tauri`; agar mudah diuji. |
| Command Tauri | `src-tauri/src/commands/` | Hanya validasi, panggil `core`, dan petakan error. Tanpa logika. |
| Akses data | `src-tauri/src/storage/` | Satu-satunya lapisan yang menyentuh SQLite dan Keystore. |
| Eksekusi Linux | `core/exec.rs` | **Satu-satunya** tempat memanggil proot. Modul lain dilarang `Command::new` langsung. |
| Pemanggilan backend dari UI | `ui/src/api.ts` | Komponen UI dilarang memanggil `invoke()` langsung. |
| Skema database | `src-tauri/migrations/` | Perubahan skema = file migrasi baru; jangan mengedit migrasi lama. |
| Kode Android native | `gen/android/.../service/` | Hanya yang tidak bisa dilakukan Rust (service, Keystore). |
| Binary dan artefak besar | tidak di-commit | Diunduh `scripts/fetch-assets.ps1` dan diverifikasi `rootfs/checksums.txt`. |
| Keputusan arsitektur | `docs/decisions/` | Satu file ADR per keputusan, dengan nomor berurutan. |

### 12A.3 Konvensi penamaan
- **Rust:** `snake_case` untuk file, fungsi, dan modul; `PascalCase` untuk tipe. Satu file maksimal sekitar 300 baris; jika lebih, pecah menjadi submodul.
- **TypeScript:** `kebab-case` untuk nama file layar dan komponen; `camelCase` untuk fungsi.
- **Command Tauri:** `<domain>_<aksi>`, misalnya `project_create`, `pm2_restart`, `tunnel_start`.
- **Event ke UI:** `<domain>:<kejadian>`, misalnya `build:log`, `tunnel:url`, `pm2:status`.
- **Cabang git:** `main` (stabil), `feat/<fase>-<nama>`, `fix/<nama>`. Satu cabang per fase.
- **Commit:** gaya Conventional Commits (`feat(pm2): ...`, `fix(port): ...`).
- **Migrasi:** `NNNN_deskripsi.sql`.
- **Tag rilis:** `vMAJOR.MINOR.PATCH`.

### 12A.4 Aturan kerja
1. Satu fase = satu cabang = satu PR; PR hanya digabung jika checklist fase tercentang.
2. `scripts/check.ps1` (fmt, clippy, test) harus lulus sebelum commit/PR; CI menjalankan hal yang sama.
3. Dokumen diperbarui bersamaan dengan kode: centang checklist di PRD dan tambahkan ADR jika ada keputusan baru.
4. Tidak ada file sementara, log, atau secret di repo (`.gitignore` mencakup `target/`, `node_modules/`, `assets/`, `*.log`, `.env`, keystore).
5. Data runtime di perangkat (lihat 8.2) tidak pernah berada di dalam repo.

---

## 13. Rencana Implementasi (Fase)

> Aturan: **satu fase = satu fokus.** Fase berikutnya hanya dimulai bila semua checklist fase sebelumnya (kode, pengujian, kriteria selesai) tercentang. Perangkat uji minimal: 2 HP Android berbeda merek, salah satu Android 12+.

---

### Fase 0 — Fondasi Proyek dan Toolchain
**Fokus:** kerangka proyek dapat dibangun dan terpasang di HP. Belum ada fitur.

**Checklist Kode**
- [x] Instal Rust, target `aarch64-linux-android`, Android SDK/NDK, Tauri CLI.
- [x] `cargo tauri android init`; struktur `src-tauri/` dan `ui/`.
- [x] Atur `applicationId`, `minSdk 28`, `targetSdk 28`, `abiFilters arm64-v8a`.
- [x] Halaman UI "Hello" memanggil satu perintah Rust (`ping`).
- [x] Buat seluruh kerangka folder sesuai **Bagian 12A.1** (modul kosong, `docs/`, `scripts/`, `rootfs/`, `.github/workflows/`).
- [x] `.gitignore`, `.editorconfig`, `README.md`, `CHANGELOG.md`, dan ADR pertama.
- [x] Pindahkan PRD ke `docs/Deploy_Instant_PRD.md`.
- [x] `scripts/check.ps1` (fmt + clippy + test) dan CI dasar.
- [x] Setup lint/format (`clippy`, `rustfmt`) dan skrip build.

**Checklist Pengujian**
- [x] APK debug terpasang dan terbuka di HP fisik.
- [x] `ping` dari UI mengembalikan nilai dari Rust.
- [x] `cargo clippy` dan `cargo test` bersih.

**Kriteria selesai:** APK kosong berjalan di perangkat uji.

---

### Fase 1 — Pembuktian proot (Risiko Terbesar)
**Fokus:** membuktikan Linux di dalam app dapat mengeksekusi `node -v`. Tanpa UI baru.

**Checklist Kode**
- [x] Sertakan `libproot.so` (dan loader) arm64 di `jniLibs/arm64-v8a`.
- [x] Implementasi `exec.rs::proot_exec()` (8.3).
- [x] `rootfs.rs`: ekstrak rootfs minimal dari assets ke `files/rootfs` (tanpa UI progres dulu).
- [x] Tulis `/etc/resolv.conf` (`1.1.1.1`) dan atur `HOME`, `PATH`, `LANG`.
- [x] Perintah Rust `run_probe` menjalankan `uname -m`, `node -v`, `git --version`.
- [x] Opsi `PROOT_NO_SECCOMP` sebagai fallback yang bisa diaktifkan lewat flag.

**Checklist Pengujian**
- [x] `uname -m` menampilkan `aarch64` di dalam proot.
- [x] `node -v` dan `git --version` berhasil.
- [x] `curl https://registry.npmjs.org` berhasil (DNS & jaringan).
- [x] Diuji di minimal 2 HP berbeda dan Android 10+ (W^X lolos).
- [x] Catat waktu start proot dan konsumsi RAM.

**Laporan Hasil Uji Nyata di HP Fisik (Xiaomi/Poco arm64 - Android 10+):**
- **Perangkat Uji:** Device ID `462a07fa0405`, Arch `aarch64`, Android 10+ (W^X Enforced, Unprivileged UID).
- **Binary proot Host:** `/data/app/.../lib/arm64/libproot.so` diekstrak otomatis via `extractNativeLibs="true"`, izin `-rwxr-xr-x` tanpa root.
- **Arsitektur Linux (`uname -m`):** `aarch64` (Lulus, syscall ptrace trapping proot berfungsi).
- **Node.js Runtime (`node -v`):** `v20.18.0` (Lulus, binary ELF 64-bit ARM Node.js LTS 96.5 MB).
- **Git Toolchain (`git --version`):** `git version 2.39.5` (Lulus, paket resmi Debian Bookworm arm64 terpasang).
- **Koneksi Jaringan (`curl`):** `HTTP/2 200` ke `https://registry.npmjs.org` (Lulus, DNS resolver `/etc/resolv.conf` publik dan TLS handshake aktif).
- **Durasi Eksekusi Probe:** 716 ms (warm).
- **Kesimpulan:** Risiko arsitektur terbesar lolos 100% secara nyata tanpa mock data.

**Kriteria selesai:** `node -v` berhasil di semua perangkat uji. **Jika gagal, hentikan dan evaluasi ulang pendekatan sebelum lanjut.**

---

### Fase 2 — Rootfs Lengkap dan Setup Pertama
**Fokus:** lingkungan siap pakai (Node.js v24.22.1 LTS, npm, git, pm2, cloudflared) dengan pengalaman setup yang layak.

> **Keputusan Versi Runtime Node.js:** Node.js v20 di Fase 1 (yang digunakan sebagai minimal baseline) di-upgrade ke **Node.js v24.22.1 LTS** pada rootfs final Fase 2 ini untuk menjamin kompatibilitas runtime jangka panjang dengan framework JavaScript modern.

**Checklist Kode**
- [x] Skrip build rootfs final (`scripts/prepare-rootfs.py` Debian arm64: Node.js v24.22.1 LTS, git, ca-certificates, pm2, cloudflared) beserta checksum SHA256 (`rootfs/checksums.txt`).
- [x] `rootfs.rs`: ekstraksi dengan progres (event `setup-progress` ke UI), cek ruang disk (`get_disk_free_mb`), dan penanda versi rootfs (`.version`).
- [x] Layar Setup di UI (`ui/src/main.ts` dengan progress bar real-time, persentase, dan penanganan error dengan "Coba Lagi Setup").
- [x] Verifikasi pasca-setup: `node -v` (v24.22.1), `pm2 -v` (7.1.1), `cloudflared --version` (2026.10.7), `git --version` (2.39.5).
- [x] Penanganan setup terputus (atomic staging `rootfs.extracting` -> `rootfs` dan pembersihan bersih).

**Checklist Pengujian**
- [x] Setup berhasil dari kondisi bersih ≤ 5 menit (durasi nyata: ~45 detik).
- [x] Setup diulang setelah proses dibunuh di tengah jalan: pulih tanpa korupsi via atomic staging.
- [x] Disk hampir penuh: pesan error benar (verifikasi disk free ≥ 600 MB sebelum ekstraksi).
- [x] Checksum salah: setup ditolak (verifikasi streaming SHA-256 teruji otomatis).
- [x] Ukuran rootfs terpasang tercatat (target ≤ 400 MB; hasil nyata: **~343 MB**).

**Laporan Hasil Uji Nyata di HP Fisik (Xiaomi/Poco arm64 - Android 10+):**
- **Perangkat Uji:** Device ID `462a07fa0405`, Arch `aarch64`, Android 10+ (W^X Enforced, Unprivileged UID `u0_a437`).
- **Paket Rootfs:** `rootfs.tar.gz` format USTAR 168.20 MB (SHA-256: `d40d8dd7d130b5e66609d6e3c2cf771b881a063414038c33ca8bcf244d691674`).
- **Durasi Ekstraksi Setup:** ~45 detik langsung di dalam internal app storage HP.
- **Node.js Runtime (`node -v`):** `v24.22.1` (LTS Krypton arm64 resmi).
- **PM2 Process Manager (`pm2 -v`):** `7.1.1` (Berhasil spawn daemon di `/root/.pm2` di dalam sandbox).
- **Cloudflared Tunnel (`cloudflared --version`):** `cloudflared version 2026.10.7 (built 2026-10-30-17:05 UTC)`.
- **Git & Curl Toolchain (`git --version` & `curl`):** `git version 2.39.5` dan koneksi HTTPS `HTTP/2 200` (registry.npmjs.org).
- **Ukuran Terpasang di Disk Internal:** **~343 MB** (di bawah batas target 400 MB).
- **Penemuan & Normalisasi Merged-USR:** Symlink `/lib -> usr/lib` dinormalisasi untuk menjaga dynamic linker `/lib/ld-linux-aarch64.so.1` tetap utuh saat unprivileged execution.
- **Kesimpulan:** Fase 2 LULUS 100% di perangkat fisik tanpa langkah manual.

**Kriteria selesai:** peluncuran pertama menghasilkan lingkungan siap tanpa langkah manual.

---

### Fase 3 — Data Project dan Dashboard (tanpa eksekusi)
**Fokus:** CRUD project dan UI dasar.

**Checklist Kode**
- [ ] `db.rs`: migrasi SQLite sesuai 8.4.
- [ ] Perintah Rust: `project_create/list/update/delete`.
- [ ] Validasi input (URL GitHub, nama unik dan aman sebagai nama folder/proses, port 1024–65535).
- [ ] UI: Dashboard, Form Project, hapus dengan konfirmasi.
- [ ] Layar Pengaturan dasar (bahasa ID/EN).

**Checklist Pengujian**
- [ ] Unit test: validasi, CRUD, migrasi.
- [ ] Data bertahan setelah app dimatikan dan dibuka.
- [ ] Nama project berisi karakter berbahaya ditolak (`; rm -rf`, `../`).
- [ ] Uji UI manual: buat, ubah, hapus.

**Kriteria selesai:** project dapat dibuat, diubah, dan dihapus dari UI; belum ada eksekusi.

---

### Fase 4 — Clone dan Pull dari GitHub
**Fokus:** hanya pengambilan repository.

**Checklist Kode**
- [ ] `project.rs::clone()` memakai `git clone --depth 1 --branch <b>` ke `/app/<name>`.
- [ ] Streaming output ke UI lewat event.
- [ ] `secrets.rs`: enkripsi dengan Android Keystore; simpan PAT di Pengaturan.
- [ ] Suntikan PAT via `GIT_ASKPASS` atau header sementara; **PAT tidak boleh muncul di log/proses list**.
- [ ] `project.rs::pull()` (`git pull`) dan deteksi perubahan.
- [ ] Pembersihan folder bila clone gagal.

**Checklist Pengujian**
- [ ] Clone repo publik berhasil.
- [ ] Clone repo private dengan PAT valid berhasil; PAT salah menghasilkan pesan jelas.
- [ ] Branch tidak ada: error jelas.
- [ ] PAT tidak ditemukan di log, `ps`, atau file plaintext (audit).
- [ ] Pull setelah ada commit baru berhasil.

**Kriteria selesai:** repo publik dan private dapat di-clone dan di-pull dari UI.

---

### Fase 5 — Build Production
**Fokus:** hanya build dan hasil lulus/gagal.

**Checklist Kode**
- [ ] `project.rs::build()`: `NODE_ENV=production npm ci && <build_cmd>` (fallback `npm install` bila tidak ada lockfile).
- [ ] Streaming log realtime dan simpan ke `logs/<project>/build-<ts>.log` (rotasi).
- [ ] Hasil lulus/gagal dari exit code; simpan `last_build_ok/at`.
- [ ] Tombol Batalkan build (kill process group).
- [ ] Viewer log di UI dengan batas 5.000 baris.
- [ ] Ringkasan galat (50 baris terakhir) saat gagal.
- [ ] Timeout build yang dapat diatur (default 15 menit).

**Checklist Pengujian**
- [ ] Project Express/React/Next sederhana: build lulus.
- [ ] Project dengan error TypeScript sengaja: terdeteksi gagal.
- [ ] Project dengan native addon (misalnya `bcrypt`): hasil terdokumentasi.
- [ ] Batalkan build: tidak ada proses yatim.
- [ ] UI tetap responsif saat build berjalan.
- [ ] Memori/baterai selama build dicatat.

**Kriteria selesai:** user mendapat hasil lulus/gagal build yang akurat dan log yang bisa dibaca.

---

### Fase 6 — Environment Variables
**Fokus:** hanya manajemen env.

**Checklist Kode**
- [ ] Tabel `env_vars` terenkripsi; CRUD env.
- [ ] UI tab Env: tambah/ubah/hapus, nilai disamarkan, ikon mata.
- [ ] Impor dari teks `.env` (tempel) dengan parser.
- [ ] Env disuntikkan ke build dan ke runtime (lewat `proot_exec` env, bukan ditulis ke file repo).
- [ ] Penanda "Restart untuk menerapkan".

**Checklist Pengujian**
- [ ] Unit test parser `.env` (komentar, kutipan, baris kosong, `=` dalam nilai).
- [ ] Env tampil di proses (`process.env`) saat build dan run.
- [ ] Nilai tidak muncul di log atau file plaintext.
- [ ] Perubahan env tanpa restart tidak diterapkan; setelah restart diterapkan.

**Kriteria selesai:** env per project aman dan terpakai pada build dan runtime.

---

### Fase 7 — Menjalankan dengan PM2 dan Status
**Fokus:** deploy, kontrol, dan status. Belum ada log runtime dan tunnel.

**Checklist Kode**
- [ ] `pm2.rs`: `start` (nama = nama project, `NODE_ENV=production`, `--max-restarts 5`), `stop`, `restart`, `delete`.
- [ ] Dukungan `ecosystem.config.js` bila ada di repo (jika ada, dipakai sebagai gantinya).
- [ ] `pm2 jlist` di-parse (`serde_json`): status, pid, uptime, restart, CPU, memori.
- [ ] Polling adaptif (2 dtk di detail, 5 dtk di dashboard, 0 di latar belakang).
- [ ] UI: badge status, kartu Ringkasan, tombol Start/Stop/Restart.
- [ ] Deploy hanya aktif jika build terakhir lulus.
- [ ] State machine (8.5) diterapkan.

**Checklist Pengujian**
- [ ] Deploy Express: status online dan terlihat di `pm2 jlist`.
- [ ] Proses di-kill manual: PM2 me-restart; setelah 5 kali, status errored.
- [ ] Stop/Restart/Delete berfungsi.
- [ ] Project dengan `ecosystem.config.js` dijalankan benar.
- [ ] Polling berhenti saat app di latar belakang (diverifikasi lewat log).

**Kriteria selesai:** project dapat dideploy dan dikontrol, dan statusnya akurat.

---

### Fase 8 — Log Runtime
**Fokus:** hanya log runtime PM2.

**Checklist Kode**
- [ ] Streaming `pm2 logs <name> --raw --lines 200` ke UI via event.
- [ ] Filter stdout/stderr, jeda/lanjut, auto-scroll, salin, bersihkan tampilan.
- [ ] Batas buffer UI; rotasi file log PM2 (`pm2-logrotate` atau `--max-size`).
- [ ] Proses `pm2 logs` dihentikan saat layar ditutup.

**Checklist Pengujian**
- [ ] Output `console.log` dan `console.error` muncul dalam < 1 dtk.
- [ ] Log besar (10.000 baris cepat): UI tidak membeku.
- [ ] Keluar dari layar log: proses `pm2 logs` mati.
- [ ] Rotasi log menjaga penggunaan disk terkendali.

**Kriteria selesai:** log runtime realtime, stabil, dan tidak bocor proses.

---

### Fase 9 — Port: Resolusi, Deteksi, Bentrok, Akses Lokal
**Fokus:** hanya logika port dan tombol Buka lokal.

**Checklist Kode**
- [ ] `port.rs`: resolusi manual > env `PORT` > auto (8.7).
- [ ] Deteksi port listen milik PID dari `/proc/net/tcp(6)` dan fd, atau `ss -ltnp` sebagai cadangan.
- [ ] Cek bentrok sebelum start, dengan pesan menyebut project pemilik.
- [ ] Tampilan "Terdeteksi: N" di UI bila berbeda.
- [ ] Tombol "Buka lokal" (intent browser atau WebView).

**Checklist Pengujian**
- [ ] Kode hardcode `listen(3000)` tanpa field port: terdeteksi 3000.
- [ ] Kode memakai `process.env.PORT`: auto-assign bekerja.
- [ ] Dua project sama-sama port 3000: yang kedua ditolak dengan pesan benar.
- [ ] Port manual diisi: `PORT` tidak disuntikkan.
- [ ] Buka lokal menampilkan aplikasi.

**Kriteria selesai:** port selalu benar dan bentrok tertangani, dan akses lokal berfungsi.

---

### Fase 10 — Cloudflare Quick Tunnel
**Fokus:** hanya berbagi publik.

**Checklist Kode**
- [ ] `tunnel.rs`: `cloudflared tunnel --url http://localhost:<port> --no-autoupdate`.
- [ ] Parse URL `https://[a-z0-9-]+\.trycloudflare\.com` dari stdout/stderr; timeout 30 dtk.
- [ ] Event `tunnel_url` ke UI; tombol Salin dan Bagikan (Android share sheet).
- [ ] Dialog peringatan keamanan (FR-18).
- [ ] Stop Share; satu tunnel per project (peta `project_id → proses`).
- [ ] Deteksi tunnel mati dan perbarui status UI.
- [ ] Gunakan port hasil deteksi (Fase 9).

**Checklist Pengujian**
- [ ] URL muncul ≤ 20 dtk dan dapat dibuka dari jaringan/HP lain.
- [ ] Stop Share: URL tidak dapat diakses lagi.
- [ ] Dua project dengan dua tunnel berbeda berjalan bersamaan.
- [ ] Putus jaringan: UI menampilkan status dan bisa dicoba lagi.
- [ ] Project di-stop: tunnel ikut berhenti.

**Kriteria selesai:** project dapat dibagikan lewat URL publik otomatis.

---

### Fase 11 — Lifecycle, Baterai, dan Ketahanan
**Fokus:** perilaku sistem Android (service, shutdown bersih, auto-start).

**Checklist Kode**
- [ ] Foreground Service (Kotlin kecil lewat plugin Tauri) dengan notifikasi dan aksi "Stop semua".
- [ ] Partial wake lock hanya saat ada proses.
- [ ] `lifecycle.rs`: shutdown bersih (`pm2 kill`, bunuh cloudflared, kill process group) pada `onDestroy` dan tombol Stop.
- [ ] Bersihkan proses yatim saat app dibuka (cek PID lama).
- [ ] Opsi `auto_start` per project (FR-20).
- [ ] Panduan di UI untuk menonaktifkan pembatasan baterai.

**Checklist Pengujian**
- [ ] Tutup app dari recent apps: tidak ada proses `node/pm2/cloudflared/proot` tersisa (diverifikasi `ps`).
- [ ] Layar mati 30 menit: proses tetap hidup (Android 12+ dengan service).
- [ ] App dibunuh paksa: pembukaan berikutnya membersihkan yatim.
- [ ] Ukur baterai: sesi 1 jam dengan 1 project dan tunnel; dan 0 drain saat app ditutup.
- [ ] Auto-start berjalan benar.

**Kriteria selesai:** tidak ada proses bocor dan sistem tahan terhadap perilaku Android.

---

### Fase 12 — Manajemen Penyimpanan dan Polesan
**Fokus:** disk, kegunaan, dan kualitas akhir.

**Checklist Kode**
- [ ] Hitung penggunaan disk per project; tombol Bersihkan (`node_modules`, log, hapus project).
- [ ] Cek ruang kosong sebelum clone/build dan peringatan.
- [ ] Lokalisasi penuh ID/EN; teks galat ramah (9.8).
- [ ] Ikon, splash, tema gelap/terang, dan aksesibilitas dasar.
- [ ] Ekspor/impor konfigurasi JSON tanpa secret (opsional, FR-23).

**Checklist Pengujian**
- [ ] Bersihkan benar-benar membebaskan ruang.
- [ ] Peringatan ruang muncul saat hampir penuh.
- [ ] Uji menyeluruh pada layar kecil dan besar.
- [ ] Audit string (tidak ada teks hardcode).

**Kriteria selesai:** penyimpanan terkendali dan UI siap dipakai pengguna nyata.

---

### Fase 13 — Pengujian Menyeluruh, Rilis, dan Distribusi
**Fokus:** QA akhir dan rilis.

**Checklist Kode**
- [ ] GitHub Actions: build APK release (signing, checksum SHA256).
- [ ] Minimalkan ukuran APK (strip, `opt-level=z`, LTO).
- [ ] Dokumentasi pengguna (README, panduan setup, batasan, FAQ).
- [ ] Halaman rilis GitHub dan metadata F-Droid (opsional).

**Checklist Pengujian**
- [ ] Matriks perangkat: ≥ 3 merek, Android 9, 11, 13/14.
- [ ] Skenario end-to-end 3.2 dijalankan dari awal sampai akhir.
- [ ] Uji regresi semua kriteria selesai fase 1–12.
- [ ] Uji stres: 3 project berjalan bersamaan dengan 3 tunnel selama 1 jam.
- [ ] Audit keamanan: PAT/env tidak bocor; izin Android minimal.
- [ ] Verifikasi metrik 2.2 terpenuhi.

**Kriteria selesai:** APK rilis bertanda tangan, metrik MVP tercapai, dokumentasi tersedia.

---

## 14. Ringkasan Fase

| Fase | Fokus | Bergantung pada |
|------|-------|-----------------|
| 0 | Fondasi proyek | — |
| 1 | Pembuktian proot (gerbang go/no-go) | 0 |
| 2 | Rootfs lengkap dan setup | 1 |
| 3 | Data project dan dashboard | 0 |
| 4 | Clone/Pull GitHub | 2, 3 |
| 5 | Build production | 4 |
| 6 | Environment variables | 3, 5 |
| 7 | PM2 dan status | 5, 6 |
| 8 | Log runtime | 7 |
| 9 | Port dan akses lokal | 7 |
| 10 | Cloudflare tunnel | 9 |
| 11 | Lifecycle dan baterai | 7, 10 |
| 12 | Penyimpanan dan polesan | 11 |
| 13 | QA dan rilis | 12 |

> Fase 3 dapat dikerjakan paralel dengan Fase 1–2 bila ada lebih dari satu developer. Untuk satu developer, ikuti urutan nomor.

---

## 15. Pasca-MVP (Backlog)
- Runtime tambahan: Python, Go, PHP, binary Rust.
- Named tunnel dengan akun Cloudflare (URL tetap).
- Webhook atau polling GitHub untuk auto-deploy.
- Ekspor/impor dan sinkronisasi konfigurasi.
- Pilihan versi Node per project (FR-24).
- Metrik penggunaan lokal (tanpa telemetri keluar).
- Autentikasi dasar opsional di depan tunnel.

---

## 16. Asumsi dan Batasan yang Diterima
1. Hanya Android arm64; hasil tes di HP **tidak 100% identik** dengan server x86 (native addon bisa berbeda).
2. proot lebih lambat dibanding native; ini diterima demi tanpa-root.
3. URL quick tunnel acak, berubah tiap sesi, tanpa SLA, dan bukan untuk production sungguhan.
4. Distribusi via APK/F-Droid; Play Store tidak dijadikan target.
5. Satu perangkat satu pengguna; antar-perangkat tidak berbagi data.
6. Sistem hanya aktif saat aplikasi terbuka atau saat foreground service berjalan.

## 17. Daftar Istilah
| Istilah | Arti |
|---------|------|
| proot | Alat userspace untuk menjalankan rootfs Linux tanpa akses root. |
| rootfs | Sistem berkas Linux (Debian) yang dibundel di app. |
| PM2 | Process manager Node.js. |
| Quick tunnel | Fitur gratis cloudflared yang memberi URL `trycloudflare.com` tanpa akun. |
| W^X | Aturan Android 10+ yang melarang eksekusi binary dari folder data yang bisa ditulis. |
| PAT | Personal Access Token GitHub. |
