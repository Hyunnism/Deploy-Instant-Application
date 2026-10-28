# Deploy Instant

> **VPS mini di dalam HP Android.** Jalankan proyek Node.js production dari GitHub langsung di HP dengan PM2 dan bagikan via Cloudflare Quick Tunnel.

## 🚀 Fitur Utama
- **100% Lokal di HP**: Menggunakan proot + rootfs Debian arm64 tanpa perlu root.
- **PM2 Asli**: Proses Node.js dikelola langsung oleh PM2 di dalam lingkungan proot.
- **Quick Tunnel**: Bagikan akses lokal ke publik via URL gratis Cloudflare `trycloudflare.com`.
- **Hemat Daya & Bersih**: Bebas proses yatim saat aplikasi ditutup (prinsip lifecycle ketat).

## 🛠️ Persyaratan Lingkungan
- Rust toolchain (`cargo`, `rustc` 1.80+) dengan target `aarch64-linux-android`
- Android SDK (API 28+) & NDK (r25+)
- Java OpenJDK 17+
- Node.js 20+ & npm
- Tauri CLI 2+

## 📁 Struktur Monorepo (PRD 12A.1)
- `docs/` — Dokumen PRD, arsitektur, dan Architecture Decision Records (ADR).
- `src-tauri/` — Rust Core dan Android shell project (Tauri 2).
- `ui/` — Frontend WebView ringan (Vite Vanilla TypeScript).
- `rootfs/` — Konfigurasi dan skrip pembangun rootfs Debian arm64.
- `scripts/` — Skrip dev (`check.ps1`, `dev-install.ps1`).

## 🧪 Validasi Cepat
```powershell
# Jalankan linter, formatter, dan test
.\scripts\check.ps1

# Bangun dan pasang ke HP via ADB
.\scripts\dev-install.ps1
```

## 📄 Lisensi
[MIT License](LICENSE)
