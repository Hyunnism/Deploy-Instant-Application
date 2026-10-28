# scripts/dev-install.ps1
# Build APK debug untuk aarch64 dan pasang langsung ke HP fisik via ADB

$ErrorActionPreference = "Stop"

if (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
}
if (-not $env:NDK_HOME) {
    $env:NDK_HOME = "$env:LOCALAPPDATA\Android\Sdk\ndk\28.2.13676358"
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

Write-Host "==> Membuka aplikasi di HP..." -ForegroundColor Cyan
adb shell monkey -p com.hyunnism.deployinstant -c android.intent.category.LAUNCHER 1

Write-Host "==> Berhasil terpasang dan dijalankan di HP! 🚀" -ForegroundColor Green
