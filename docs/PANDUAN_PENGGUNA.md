# Panduan Pengguna Deploy Instant

Selamat datang di **Deploy Instant** — aplikasi Android mandiri yang mengubah smartphone Android Anda menjadi "VPS mini di saku".

---

## 📋 Daftar Isi
1. [Kebutuhan Perangkat](#1-kebutuhan-perangkat)
2. [Instalasi dan Setup Pertama Kali](#2-instalasi-dan-setup-pertama-kali)
3. [Menambahkan Proyek Baru](#3-menambahkan-proyek-baru)
4. [Membangun dan Menjalankan Proyek](#4-membangun-dan-menjalankan-proyek)
5. [Manajemen Environment Variables (.env)](#5-manajemen-environment-variables-env)
6. [Membagikan Proyek ke Publik (Cloudflare Tunnel)](#6-membagikan-proyek-ke-publik-cloudflare-tunnel)
7. [Layanan Background & Pengaturan Baterai](#7-layanan-background--pengaturan-baterai)
8. [Manajemen Penyimpanan & Ekspor/Impor](#8-manajemen-penyimpanan--eksporimpor)
9. [Tanya Jawab (FAQ) & Batasan](#9-tanya-jawab-faq--batasan)

---

## 1. Kebutuhan Perangkat
* **Sistem Operasi**: Android 9.0 (Pie / API 28) hingga Android 15+.
* **Arsitektur CPU**: 64-bit ARM (`arm64-v8a`).
* **RAM Rekomendasi**: Minimal 3 GB (rekomendasi 4 GB+ untuk build proyek berat).
* **Penyimpanan Bebas**: Minimal 1 GB ruang kosong untuk sistem rootfs Linux dan dependensi `node_modules`.

---

## 2. Instalasi dan Setup Pertama Kali
1. Unduh file APK rilis (`DeployInstant-vX.Y.Z-arm64-v8a.apk`) dari halaman [GitHub Releases](https://github.com/hyunnism/deploy-instant/releases).
2. Pasang APK di perangkat Anda. Berikan izin **Install Unknown Apps** jika diminta.
3. Buka aplikasi Deploy Instant:
   * Pada peluncuran perdana, aplikasi akan mengekstrak lingkungan Linux mandiri (rootfs Alpine arm64).
   * Proses ini hanya memakan waktu 1–2 menit dan membutuhkan ~300 MB ruang penyimpanan awal.
   * Setelah selesai, Anda langsung diarahkan ke Dashboard utama.

---

## 3. Menambahkan Proyek Baru
1. Tekan tombol **+ Tambah Project** di pojok kanan atas Dashboard.
2. Isi formulir informasi proyek:
   * **Nama Project**: Nama bebas pengenal aplikasi Anda (contoh: `API Toko Online`).
   * **URL Repository**: URL Git publik atau privat GitHub (contoh: `https://github.com/username/repo`).
   * **Personal Access Token (PAT)** *(Opsional)*: Masukkan GitHub PAT jika repository Anda bersifat privat. Token disimpan terenkripsi dengan aman di memori lokal.
   * **Branch**: Branch target yang akan di-clone (default: `main`).
   * **Command Build**: Perintah kompilasi production (default: `npm run build` atau `npm ci`).
   * **Command Start**: Perintah menjalankan server (default: `npm start` atau `node index.js`).
   * **Port**: Kosongkan untuk **Auto-detect**, atau isi manual (misal: `3000`).
3. Tekan **Simpan**.

---

## 4. Membangun dan Menjalankan Proyek
* **Clone & Build**:
  * Buka menu proyek, lalu tekan tombol **Build**.
  * Aplikasi akan menjalankan git clone dan eksekusi perintah build di dalam lingkungan terisolasi.
  * Anda dapat memantau **Live Build Logs** secara realtime. Jika build berhasil, status berubah menjadi **Ready**.
* **Deploy (Jalankan via PM2)**:
  * Tekan tombol **Start / Deploy**.
  * Proses Node.js akan dijalankan di latar belakang oleh process manager **PM2**.
  * Status proyek akan berubah menjadi **RUNNING** dengan indikator port aktif.
* **Lihat Live Runtime Logs**:
  * Tekan tombol **Logs** untuk melihat output `stdout` dan `stderr` realtime dari server Node.js Anda.

---

## 5. Manajemen Environment Variables (.env)
1. Pada kartu proyek, pilih opsi **Environment (.env)**.
2. Anda dapat mengisi variabel kunci-nilai:
   ```env
   PORT=3000
   NODE_ENV=production
   DATABASE_URL=https://your-remote-db.supabase.co
   API_SECRET_KEY=my_secure_token_123
   ```
3. Variabel lingkungan dienkripsi secara lokal menggunakan cipher AES-GCM 256-bit dan diinjeksikan secara otomatis ke runtime proyek saat start/restart.

---

## 6. Membagikan Proyek ke Publik (Cloudflare Tunnel)
1. Pastikan proyek dalam status **RUNNING**.
2. Tekan tombol **Share Public**.
3. Sistem akan menginisiasi tunnel terenkripsi Cloudflare (`cloudflared`).
4. Dalam 5–15 detik, sebuah URL publik HTTPS instan akan muncul, contoh:  
   `https://random-subdomain.trycloudflare.com`
5. Anda dapat menyalin URL atau membagikannya ke WhatsApp/Telegram untuk dites oleh klien atau teman tanpa perlu IP publik atau konfigurasi router modem.

---

## 7. Layanan Background & Pengaturan Baterai
Agar server Node.js dan tunnel tidak dimatikan oleh sistem manajemen daya agresif Android (seperti MIUI, HyperOS, ColorOS, OneUI):
1. **Foreground Service**:
   * Deploy Instant secara otomatis menyalakan notifikasi persisten saat ada minimal 1 proyek yang berjalan.
2. **Whitelist Battery Optimization**:
   * Tekan banner atau menu **Settings -> Baterai**.
   * Pilih **Matikan Pembatasan Baterai (Unrestricted / No Restrictions)** agar Android tidak menghentikan proses saat layar HP terkunci.
3. **Zero-Orphan Guarantee**:
   * Ketika Anda menekan Stop atau menghapus proyek dari Recent Apps (saat tidak ada proyek aktif), semua proses background (`pm2`, `cloudflared`, `proot`) dijamin mati bersih tanpa menyisakan beban CPU.

---

## 8. Manajemen Penyimpanan & Ekspor/Impor
* **Pembersihan Ruang**:
  * Buka menu **Penyimpanan (Storage)** untuk melihat rincian pemakaian disk (Rootfs, Direktori Proyek, Cache NPM, dan Log).
  * Anda dapat menekan **Bersihkan Cache NPM** atau **Bersihkan Build Artifacts** untuk membebaskan ruang tanpa merusak file penting.
* **Ekspor & Impor Konfigurasi (JSON)**:
  * Anda dapat mengekspor daftar proyek Anda ke dalam file JSON untuk dipindahkan ke smartphone lain.
  * *Catatan Keamanan*: Demi melindungi privasi Anda, rahasia sensitif (GitHub PAT dan nilai `.env`) sengaja **tidak disertakan** dalam file ekspor publik.

---

## 9. Tanya Jawab (FAQ) & Batasan
* **T: Apakah HP saya harus di-root?**  
  *J:* Sama sekali **TIDAK**. Deploy Instant menggunakan teknologi PRoot yang bekerja sepenuhnya dalam user space standar Android.
* **T: Apakah URL Quick Tunnel permanen?**  
  *J:* URL `trycloudflare.com` adalah quick tunnel gratis sementara yang berubah setiap kali sesi tunnel dimulai ulang. Jika butuh domain permanen, Anda dapat menggunakan Named Tunnel dengan akun Cloudflare Anda.
* **T: Bagaimana menghubungkan Database?**  
  *J:* Gunakan **SQLite** (file lokal di dalam folder proyek seperti Prisma/Drizzle) untuk performa tercepat tanpa makan RAM, atau gunakan database cloud eksternal (Supabase, Neon, PlanetScale) via variabel `.env`.
* **T: Apakah aman membagikan URL tunnel?**  
  *J:* Akses tunnel terbuka untuk publik di internet. Pastikan backend Anda memiliki autentikasi yang aman jika memproses data sensitif.

