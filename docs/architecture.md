# Deploy Instant — Arsitektur Sistem

Dokumen ini merupakan turunan teknis dari [PRD Bagian 8](Deploy_Instant_PRD.md#8-arsitektur-sistem).

## 1. Ikhtisar Arsitektur
Deploy Instant dibangun sebagai aplikasi satu APK (monorepo) menggunakan **Tauri 2 (Android)** yang mengintegrasikan:
1. **Frontend (WebView)**: UI ultra-ringan berbasis Vanilla TypeScript (Vite) tanpa overhead framework reaktif berat. Semua komunikasi ke backend melalui satu pintu `ui/src/api.ts`.
2. **Backend (Rust Core)**: Berjalan secara native di Android melalui JNI (`app_lib`). Terdiri dari:
   - `commands/`: Handler tipis Tauri IPC yang memvalidasi input dan memanggil `core`.
   - `core/`: Logika bisnis murni (eksekusi proses Linux, PM2, port resolution, cloudflared tunnel, manajemen lifecycle).
   - `storage/`: Akses database SQLite (`rusqlite`) dan enkripsi rahasia (Android Keystore).
3. **Linux Rootfs & Isolasi (proot)**:
   - Rootfs Debian arm64 minimal yang memuat Node.js LTS, npm, git, PM2, dan cloudflared.
   - `libproot.so` (di `jniLibs/arm64-v8a`) dieksekusi melalui satu pintu `core/exec.rs::proot_exec()`.

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
```

## 2. Batasan Desain (Ponytail)
- **YAGNI**: Tidak membangun backend remote atau sistem akun. Seluruh data tetap di perangkat pengguna.
- **Satu Pintu Eksekusi**: Hanya `core/exec.rs` yang boleh menjalankan `libproot.so`. Modul lain tidak memanggil `std::process::Command` atau `tokio::process::Command` langsung.
- **Lifecycle Bersih**: Saat aplikasi dimatikan, semua child process (`pm2`, `node`, `cloudflared`) wajib dihentikan tanpa proses yatim.
