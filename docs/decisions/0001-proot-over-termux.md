# ADR 0001: Penggunaan Mandiri proot Dibandingkan Ketergantungan pada Termux

- **Status**: Diterima (Accepted)
- **Tanggal**: 2026-10-28
- **Konteks**: Deploy Instant membutuhkan lingkungan Linux arm64 untuk menjalankan Node.js, npm, git, PM2, dan cloudflared di dalam Android.

## Konteks dan Masalah
Ada dua pendekatan umum untuk menjalankan command-line Linux di Android:
1. Meminta pengguna menginstal Termux (atau Termux:API) dan berkomunikasi lewat Intent/Socket.
2. Membundel binary `libproot.so` dan rootfs Debian mandiri langsung di dalam APK Deploy Instant.

## Opsi yang Dipertimbangkan
1. **Opsi A: Integrasi Termux eksternal**
   - Kelebihan: Ukuran APK utama sangat kecil.
   - Kekurangan: Pengalaman pengguna terfragmentasi (harus install app luar, konfigurasi izin repo Termux manual, Termux sering dibatasi oleh battery saver vendor secara terpisah, rentan breaking change pada intent/permission).
2. **Opsi B: Bundel mandiri proot + rootfs di dalam APK (Dipilih)**
   - Kelebihan: Pengalaman "sekali klik langsung jalan", isolasi penuh di direktori privat aplikasi, kontrol versi runtime (Node LTS, PM2) stabil dan konsisten.
   - Kekurangan: Ukuran unduhan/ekstraksi awal lebih besar (~150-300 MB).

## Keputusan
Memilih **Opsi B** (bundel `libproot.so` di `jniLibs` dan ekstraksi rootfs ke storage privat). Hal ini memastikan target MVP tercapai sebagai satu APK mandiri tanpa konfigurasi eksternal oleh pengguna.
