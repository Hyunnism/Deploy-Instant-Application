# scripts/dev-install.ps1
# Build APK debug untuk aarch64 dan pasang langsung ke HP fisik via ADB

$ErrorActionPreference = "Stop"

if (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
}
if (-not $env:NDK_HOME) {
    $env:NDK_HOME = "$env:LOCALAPPDATA\Android\Sdk\ndk\28.2.13676358"
}

# Pastikan native library proot ada di jniLibs
$jniLib = "src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/libproot.so"
if (-not (Test-Path $jniLib)) {
    Write-Host "==> Menyiapkan asset libproot..." -ForegroundColor Cyan
    & "$PSScriptRoot\fetch-assets.ps1"
}

Write-Host "==> Membangun APK Debug (aarch64)..." -ForegroundColor Cyan
cargo tauri android build --debug --target aarch64 --apk
if ($LASTEXITCODE -ne 0) {
    Write-Error "Gagal membangun APK Android!"
    exit $LASTEXITCODE
}

$apkPath = Get-ChildItem -Path "src-tauri/gen/android/app/build/outputs/apk" -Filter "*.apk" -Recurse | Select-Object -First 1 -ExpandProperty FullName

if (-not $apkPath) {
    Write-Error "APK tidak ditemukan di build outputs!"
    exit 1
}

Write-Host "==> APK ditemukan: $apkPath" -ForegroundColor Green
Write-Host "==> Menginstal ke HP fisik via ADB..." -ForegroundColor Cyan
adb install -r $apkPath

# Pastikan arsip rootfs.tar.gz tersedia di HP untuk pengujian Setup Fase 2
Write-Host "==> Memverifikasi keberadaan arsip rootfs.tar.gz di HP..." -ForegroundColor Cyan
$hasArchive = adb shell "if [ -f /data/local/tmp/rootfs.tar.gz ]; then echo 'YES'; else echo 'NO'; fi"
if ($hasArchive -notmatch "YES") {
    $rootfsArchive = "src-tauri/assets/rootfs.tar.gz"
    if (-not (Test-Path $rootfsArchive)) {
        python "$PSScriptRoot\prepare-rootfs.py"
    }
    Write-Host "==> Mentransfer arsip rootfs ke HP..." -ForegroundColor Cyan
    adb push $rootfsArchive /data/local/tmp/rootfs.tar.gz
}

Write-Host "==> Membuka aplikasi di HP..." -ForegroundColor Cyan
adb shell monkey -p com.hyunnism.deployinstant -c android.intent.category.LAUNCHER 1

Write-Host "==> Berhasil terpasang dan dijalankan di HP! 🚀" -ForegroundColor Green
