# Deploy Instant

[![CI](https://github.com/hyunnism/deploy-instant/actions/workflows/ci.yml/badge.svg)](https://github.com/hyunnism/deploy-instant/actions/workflows/ci.yml)
[![Release APK](https://github.com/hyunnism/deploy-instant/actions/workflows/release.yml/badge.svg)](https://github.com/hyunnism/deploy-instant/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: Android arm64](https://img.shields.io/badge/Platform-Android%20arm64-brightgreen.svg)](docs/Deploy_Instant_PRD.md)

> **VPS mini di dalam smartphone Android Anda.** Jalankan proyek Node.js production dari GitHub langsung di HP dengan PM2, dan bagikan akses publik secara instan via Cloudflare Quick Tunnel — tanpa biaya server, tanpa VMware, dan tanpa IP publik.

---

## 📱 Mengapa Deploy Instant?

* **0 Rupiah Biaya Operasional**: Tidak perlu menyewa VPS bulanan atau setup server cloud untuk sekadar menguji build production atau mendemokan hasil kerja ke klien.
* **100% On-Device (Tanpa Root)**: Memanfaatkan lingkungan Linux mandiri (PRoot di atas Alpine arm64) yang berjalan di user space Android.
* **Process Manager PM2 Asli**: Menjalankan server Node.js di latar belakang, menangani auto-restart saat crash, dan memantau penggunaan resource.
* **Instant Public HTTPS (Cloudflare Tunnel)**: Akses aplikasi Anda dari mana saja di internet via subdomain `*.trycloudflare.com` tanpa port-forwarding router modem.
* **Hemat Daya & Zero-Orphan**: Terintegrasi langsung dengan Android Foreground Service dan WakeLock. Ketika aplikasi ditutup dan tidak ada proses berjalan, semua background task dimatikan bersih 100%.

---

## 🏗️ Arsitektur Sistem

```
┌────────────────────────────────────────────────────────┐
│                   Android Device                       │
│  ┌───────────────────────┐   ┌──────────────────────┐  │
│  │      Tauri Web UI     │◄─►│   Rust Core Engine   │  │
│  │ (Vite Vanilla TS GUI) │   │ (Process & Security) │  │
│  └───────────────────────┘   └──────────┬───────────┘  │
│                                         │              │
│                ┌────────────────────────┴────────┐     │
│                ▼                                 ▼     │
│      ┌──────────────────┐              ┌──────────────────┐
│      │ Android Service  │              │  PRoot Container │
│      │ (Wakelock/Notif) │              │ (Alpine Linux)   │
│      └──────────────────┘              └─────────┬────────┘
│                                                  │
│                                 ┌────────────────┴───────────────┐
│                                 ▼                                ▼
│                       ┌───────────────────┐            ┌───────────────────┐
│                       │   PM2 (Node.js)   │◄──────────►│  cloudflared bin  │
│                       │ (Production App)  │            │  (Public Tunnel)  │
│                       └───────────────────┘            └─────────┬─────────┘
└──────────────────────────────────────────────────────────────────┼─────┘
                                                                   ▼
                                                       Internet (Public HTTPS)
```

---

## 🚀 Panduan Memulai Cepat (Pengguna)

1. **Unduh APK Rilis**:
   Unduh versi terbaru `DeployInstant-vX.Y.Z-arm64-v8a.apk` dari [GitHub Releases](https://github.com/hyunnism/deploy-instant/releases).
2. **Setup Perdana**:
   Buka aplikasi. Tunggu 1–2 menit saat sistem mengekstrak lingkungan Linux internal.
3. **Tambah & Jalankan Project**:
   * Tekan **+ Tambah Project**, masukkan URL repository GitHub Anda.
   * Tekan tombol **Build** untuk mengompilasi aplikasi.
   * Tekan **Start** untuk menjalankan aplikasi via PM2.
   * Tekan **Share Public** untuk mendapatkan URL HTTPS publik instan.

> 📖 **Panduan Lengkap**: Baca [Panduan Pengguna Lengkap](docs/PANDUAN_PENGGUNA.md) untuk konfigurasi `.env`, manajemen storage, dan pengaturan baterai.

---

## 🛠️ Panduan Pengembangan (Developer)

### Prasyarat
* **Rust**: `rustc` & `cargo` 1.80+ dengan target `aarch64-linux-android`.
* **Android SDK & NDK**: API 28+ (Android 9.0+) dan NDK r26+.
* **Java**: OpenJDK 17.
* **Node.js**: v20+ & npm.
* **Tauri CLI**: `cargo install tauri-cli --version "^2.0.0"`.

### Perintah Validasi & Build
```powershell
# 1. Jalankan pemeriksaan linter, formatter, dan automated tests
.\scripts\check.ps1

# 2. Build APK Debug dan langsung pasang ke HP fisik via ADB
.\scripts\dev-install.ps1

# 3. Build APK Release bertanda tangan (opt-level='z', LTO, strip)
.\scripts\build-release.ps1 -Install
```

---

## 📁 Struktur Repositori

* `src-tauri/` — Rust core backend, bridge JNI Android, dan manajemen proses.
* `ui/` — Antarmuka web responsif berbasis Vite, Vanilla TypeScript, dan CSS modern (Light/Dark theme).
* `scripts/` — Skrip otomatisasi build, pengujian, dan deployment (`check.ps1`, `build-release.ps1`).
* `docs/` — Dokumen PRD ([Deploy_Instant_PRD.md](docs/Deploy_Instant_PRD.md)), panduan pengguna, dan catatan arsitektur.
* `fastlane/` — Metadata aplikasi untuk publikasi F-Droid (bahasa Indonesia & Inggris).

---

## 🔒 Privasi dan Keamanan

* **Zero External Telemetry**: Tidak ada pelacak, analitik, atau server pengumpul data eksternal. Semua data tersimpan lokal di perangkat Anda.
* **Enkripsi AES-GCM 256-bit**: Kredensial rahasia (GitHub Personal Access Token) dan Environment Variables (`.env`) dienkripsi secara lokal sebelum ditulis ke database internal SQLite.
* **Izin Android Minimal**: Hanya meminta izin esensial untuk konektivitas jaringan (`INTERNET`), layanan latar belakang (`FOREGROUND_SERVICE`), dan pencegahan sleep (`WAKE_LOCK`).

---

## 📄 Lisensi
Didistribusikan di bawah lisensi MIT. Lihat [LICENSE](LICENSE) untuk informasi lebih lanjut.
