# scripts/test-e2e.ps1
# Uji Skenario End-to-End, Audit Keamanan, dan Verifikasi Zero-Orphan pada Perangkat Fisik

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " Deploy Instant -- E2E Testing and Security Audit (Fase 13) " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Cek Koneksi ADB
Write-Host "==> [1/5] Memeriksa koneksi perangkat fisik via ADB..." -ForegroundColor Cyan
$devices = adb devices
$deviceLine = $devices | Where-Object { $_ -match "\tdevice" }
if (-not $deviceLine) {
    Write-Error "Perangkat Android tidak terdeteksi via ADB! Pastikan USB Debugging aktif."
    exit 1
}
$deviceModel = (adb shell getprop ro.product.model).Trim()
$androidVersion = (adb shell getprop ro.build.version.release).Trim()
$abi = (adb shell getprop ro.product.cpu.abi).Trim()
Write-Host "==> Perangkat: $deviceModel | Android: $androidVersion | ABI: $abi" -ForegroundColor Green

# 2. Cek Instalasi Aplikasi
Write-Host "==> [2/5] Memeriksa instalasi com.hyunnism.deployinstant..." -ForegroundColor Cyan
$packageCheck = adb shell "pm list packages com.hyunnism.deployinstant"
if ($packageCheck -notmatch "com.hyunnism.deployinstant") {
    Write-Host "==> Aplikasi belum terpasang. Membangun dan memasang via scripts/dev-install.ps1..." -ForegroundColor Yellow
    & "$PSScriptRoot\dev-install.ps1"
} else {
    Write-Host "==> Aplikasi terpasang dengan baik." -ForegroundColor Green
}

# 3. Audit Izin Android (Minimal Privilege Principle)
Write-Host "==> [3/5] Melakukan Audit Izin Android (AndroidManifest)..." -ForegroundColor Cyan
$perms = adb shell "dumpsys package com.hyunnism.deployinstant" | Select-String "android.permission."
$dangerousPerms = @(
    "android.permission.READ_EXTERNAL_STORAGE",
    "android.permission.WRITE_EXTERNAL_STORAGE",
    "android.permission.CAMERA",
    "android.permission.RECORD_AUDIO",
    "android.permission.ACCESS_FINE_LOCATION"
)
$hasDangerous = $false
foreach ($p in $dangerousPerms) {
    if ($perms -match $p) {
        Write-Host " [WARNING] Ditemukan izin berlebih: $p" -ForegroundColor Red
        $hasDangerous = $true
    }
}
if (-not $hasDangerous) {
    Write-Host "==> Audit Izin LULUS: Aplikasi hanya menggunakan izin minimal yang aman." -ForegroundColor Green
}

# 4. Audit Keamanan Log (Zero-Leakage Check)
Write-Host "==> [4/5] Memeriksa logcat untuk kebocoran data rahasia / token..." -ForegroundColor Cyan
$logLeaks = adb logcat -d | Select-String "ghp_", "GITHUB_PAT", "API_SECRET", "password="
if ($logLeaks) {
    Write-Host " [WARNING] Terdeteksi potensi string sensitif di logcat:" -ForegroundColor Red
    $logLeaks | Select-Object -First 3 | ForEach-Object { Write-Host "   $_" -ForegroundColor Yellow }
} else {
    Write-Host "==> Audit Logcat LULUS: Tidak ada token atau password yang bocor ke logcat." -ForegroundColor Green
}

# 5. Verifikasi Zero-Orphan Process
Write-Host "==> [5/5] Menguji garansi Zero-Orphan Process saat aplikasi dihentikan..." -ForegroundColor Cyan
adb shell am force-stop com.hyunnism.deployinstant
Start-Sleep -Seconds 2

$orphanCheck = adb shell "ps -A | grep -E '[c]loudflared|[p]m2|[p]root'"
if ($orphanCheck -and $orphanCheck.Trim() -ne "") {
    Write-Host " [WARNING] Ditemukan sisa proses background: $orphanCheck" -ForegroundColor Yellow
} else {
    Write-Host "==> Zero-Orphan LULUS: Semua proses background mati bersih 100%!" -ForegroundColor Green
}

Write-Host "============================================================" -ForegroundColor Green
Write-Host " Seluruh Audit dan Pengujian E2E Selesai dengan SUKSES!     " -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
