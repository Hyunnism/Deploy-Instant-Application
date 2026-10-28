# scripts/fetch-assets.ps1
# Mengunduh binary proot dan rootfs Debian arm64 ke src-tauri/assets dan jniLibs

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$assetsDir = Join-Path $PSScriptRoot "..\src-tauri\assets"
$jniDir = Join-Path $PSScriptRoot "..\src-tauri\gen\android\app\src\main\jniLibs\arm64-v8a"

if (-not (Test-Path $assetsDir)) { New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null }
if (-not (Test-Path $jniDir)) { New-Item -ItemType Directory -Force -Path $jniDir | Out-Null }

# 1. Download proot binary & loader
$prootUrl = "https://github.com/EMERLADD/proot-distro-nolib/releases/download/v1.0.0/libpdn.so"
$loaderUrl = "https://github.com/EMERLADD/proot-distro-nolib/releases/download/v1.0.0/libproot-loader.so"

$prootDest = Join-Path $assetsDir "libproot.so"
$loaderDest = Join-Path $assetsDir "libproot-loader.so"

if (-not (Test-Path $prootDest)) {
    Write-Host "==> Downloading libproot.so..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $prootUrl -OutFile $prootDest -UseBasicParsing
}
if (-not (Test-Path $loaderDest)) {
    Write-Host "==> Downloading libproot-loader.so..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $loaderUrl -OutFile $loaderDest -UseBasicParsing
}

# Salin ke jniLibs
Copy-Item -Path $prootDest -Destination (Join-Path $jniDir "libproot.so") -Force
Copy-Item -Path $loaderDest -Destination (Join-Path $jniDir "libproot-loader.so") -Force
Write-Host "==> libproot.so & libproot-loader.so terpasang di jniLibs/arm64-v8a" -ForegroundColor Green

# 2. Download Debian Bookworm aarch64 rootfs
$rootfsUrl = "https://github.com/termux/proot-distro/releases/download/v4.17.3/debian-bookworm-aarch64-pd-v4.17.3.tar.xz"
$rootfsDest = Join-Path $assetsDir "rootfs-min-arm64.tar.xz"

if (-not (Test-Path $rootfsDest)) {
    Write-Host "==> Downloading Debian Bookworm arm64 rootfs (42.9 MB)..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $rootfsUrl -OutFile $rootfsDest -UseBasicParsing
}
Write-Host "==> Debian rootfs siap di $rootfsDest" -ForegroundColor Green

# 3. Download Node.js LTS arm64 (v20.18.0)
$nodeUrl = "https://nodejs.org/dist/v20.18.0/node-v20.18.0-linux-arm64.tar.xz"
$nodeDest = Join-Path $assetsDir "node-v20-linux-arm64.tar.xz"

if (-not (Test-Path $nodeDest)) {
    Write-Host "==> Downloading Node.js LTS arm64 (21 MB)..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $nodeUrl -OutFile $nodeDest -UseBasicParsing
}
Write-Host "==> Node.js LTS arm64 siap di $nodeDest" -ForegroundColor Green

Write-Host "==> [SUCCESS] Semua aset Fase 1 berhasil diunduh dan disiapkan! 🎉" -ForegroundColor Green
