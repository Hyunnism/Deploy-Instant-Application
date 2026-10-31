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

# 3. Download Node.js LTS arm64 (v24.22.1)
$nodeUrl = "https://nodejs.org/dist/v24.22.1/node-v24.22.1-linux-arm64.tar.xz"
$nodeDest = Join-Path $assetsDir "node-v24-linux-arm64.tar.xz"

if (-not (Test-Path $nodeDest)) {
    Write-Host "==> Downloading Node.js v24.22.1 LTS arm64 (30.9 MB)..." -ForegroundColor Cyan
    curl.exe -L -o $nodeDest $nodeUrl
}
Write-Host "==> Node.js v24 LTS arm64 siap di $nodeDest" -ForegroundColor Green

# 4. Download Cloudflared arm64
$cfUrl = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
$cfDest = Join-Path $assetsDir "cloudflared-linux-arm64"

if (-not (Test-Path $cfDest)) {
    Write-Host "==> Downloading Cloudflared Linux arm64 (37.6 MB)..." -ForegroundColor Cyan
    curl.exe -L -o $cfDest $cfUrl
}
Write-Host "==> Cloudflared arm64 siap di $cfDest" -ForegroundColor Green

# 5. Siapkan PM2 untuk bundling rootfs
$pm2Dir = Join-Path $assetsDir "pm2-staging"
if (-not (Test-Path (Join-Path $pm2Dir "node_modules\pm2\bin\pm2"))) {
    Write-Host "==> Menyiapkan PM2 via npm..." -ForegroundColor Cyan
    if (-not (Test-Path $pm2Dir)) { New-Item -ItemType Directory -Force -Path $pm2Dir | Out-Null }
    npm install --prefix $pm2Dir pm2 --omit=dev --no-audit --no-fund
}
Write-Host "==> PM2 staging siap di $pm2Dir" -ForegroundColor Green

Write-Host "==> [SUCCESS] Semua aset Fase 2 berhasil diunduh dan disiapkan! 🎉" -ForegroundColor Green
