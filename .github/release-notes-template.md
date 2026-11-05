# Deploy Instant $VERSION

> **VPS mini di dalam smartphone Android Anda.** Jalankan proyek Node.js production dari GitHub langsung di HP dengan PM2 dan bagikan via Cloudflare Quick Tunnel.

## 🚀 Apa yang Baru di Versi Ini?
- **Fase 1–2**: Ekstraksi lingkungan Linux Alpine arm64 (PRoot) mandiri tanpa root.
- **Fase 3–4**: Manajemen proyek terisolasi, integrasi Git clone/pull dengan enkripsi PAT.
- **Fase 5–6**: Eksekusi build production mandiri, pelaporan log error, dan manajemen `.env` terenkripsi AES-GCM.
- **Fase 7–8**: PM2 process manager bawaan, deteksi health status, dan live log streaming.
- **Fase 9–10**: Deteksi bentrok port otomatis dan Cloudflare Quick Tunnel publik HTTPS instan.
- **Fase 11**: Android Foreground Service persisten, WakeLock, dan garansi zero-orphan lifecycle.
- **Fase 12**: Manajemen penyimpanan disk, pembersihan cache, tema gelap/terang adaptif, dan ekspor/impor JSON.
- **Fase 13**: Optimasi ukuran APK (R8/ProGuard & Rust opt-level='z'), validasi SHA256 checksums, dan rilis stabil.

## 📦 Aset Rilis
| Berkas | Arsitektur | Keterangan |
|---|---|---|
| `DeployInstant-$VERSION-arm64-v8a.apk` | `arm64-v8a` | APK Rilis Bertanda Tangan |
| `SHA256SUMS.txt` | Semua | Daftar Checksum Integritas |

## 🛡️ Verifikasi Integritas SHA-256
Untuk memverifikasi keaslian file APK yang diunduh:

**Linux / macOS:**
```bash
sha256sum -c SHA256SUMS.txt
```

**Windows PowerShell:**
```powershell
Get-FileHash -Algorithm SHA256 .\DeployInstant-$VERSION-arm64-v8a.apk
```
Bandungkan nilai hash dengan isi berkas `SHA256SUMS.txt`.

