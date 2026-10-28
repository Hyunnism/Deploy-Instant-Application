# Rencana Pengujian (Test Plan)

## 1. Fase 0 — Fondasi Proyek dan Toolchain
- [ ] Build Rust test & clippy bebas error (`scripts/check.ps1`).
- [ ] APK debug terkompilasi untuk arsitektur `arm64-v8a`.
- [ ] APK terpasang di HP fisik tanpa kendala signature / manifest.
- [ ] IPC round-trip `system_ping` dari UI WebView ke Rust mengembalikan `pong`.
- [ ] Logging berjalan dengan baik di logcat.

## 2. Fase Selanjutnya
- Fase 1: Uji eksekusi proot dan `node -v` di perangkat fisik.
- Fase 2: Uji integritas rootfs dan setup wizard.
- Fase 3-13: Sesuai checklist di PRD.
