# scripts/build-release.ps1
# Membangun APK Release (aarch64), menandatangani APK, menghitung SHA256, dan opsi pasang via ADB.

param(
    [switch]$Install
)

$ErrorActionPreference = "Stop"

if (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
}
if (-not $env:NDK_HOME) {
    $ndkDirs = Get-ChildItem -Path "$env:ANDROID_HOME\ndk" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
    if ($ndkDirs) {
        $env:NDK_HOME = $ndkDirs[0].FullName
    } else {
        $env:NDK_HOME = "$env:ANDROID_HOME\ndk\28.2.13676358"
    }
}

Write-Host "==> [1/5] Membangun Frontend Web UI..." -ForegroundColor Cyan
npm --prefix ui run build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> [2/5] Memastikan library native libproot siap..." -ForegroundColor Cyan
$jniLib = "src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/libproot.so"
if (-not (Test-Path $jniLib)) {
    & "$PSScriptRoot\fetch-assets.ps1"
}

Write-Host "==> [3/5] Mengompilasi APK Release untuk aarch64 (opt-level='z', LTO, strip)..." -ForegroundColor Cyan
cargo tauri android build --target aarch64 --apk
if ($LASTEXITCODE -ne 0) {
    Write-Error "Gagal mengompilasi APK Release!"
    exit $LASTEXITCODE
}

Write-Host "==> [4/5] Mencari APK Release..." -ForegroundColor Cyan
$apkFiles = Get-ChildItem -Path "src-tauri/gen/android/app/build/outputs/apk" -Filter "*.apk" -Recurse | Where-Object { $_.FullName -notmatch "androidTest" }
$apkFile = $apkFiles | Select-Object -First 1

if (-not $apkFile) {
    Write-Error "APK release tidak ditemukan di build outputs!"
    exit 1
}

$apkPath = $apkFile.FullName
$fileSizeMB = [math]::Round($apkFile.Length / 1MB, 2)
Write-Host "==> Ditemukan APK: $apkPath ($fileSizeMB MB)" -ForegroundColor Green

# Periksa apakah apksigner tersedia
$buildToolsDirs = Get-ChildItem -Path "$env:ANDROID_HOME\build-tools" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
$apksigner = $null
if ($buildToolsDirs) {
    $cand = Join-Path $buildToolsDirs[0].FullName "apksigner.bat"
    if (Test-Path $cand) { $apksigner = $cand }
}

if ($apksigner) {
    Write-Host "==> Memverifikasi tanda tangan APK..." -ForegroundColor Cyan
    & $apksigner verify $apkPath 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "==> Menandatangani APK rilis dengan debug/release keystore..." -ForegroundColor Yellow
        $keystore = "$HOME\.android\debug.keystore"
        if (Test-Path $keystore) {
            & $apksigner sign --ks $keystore --ks-pass pass:android --key-pass pass:android --out $apkPath $apkPath
            Write-Host "==> Berhasil ditandatangani!" -ForegroundColor Green
        }
    } else {
        Write-Host "==> APK sudah memiliki tanda tangan yang valid." -ForegroundColor Green
    }
}

Write-Host "==> [5/5] Menghitung SHA256 Checksum..." -ForegroundColor Cyan
$hash = (Get-FileHash -Path $apkPath -Algorithm SHA256).Hash
Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray
Write-Host "APK File : $($apkFile.Name)" -ForegroundColor White
Write-Host "Ukuran   : $fileSizeMB MB" -ForegroundColor White
Write-Host "SHA256   : $hash" -ForegroundColor Yellow
Write-Host "------------------------------------------------------------" -ForegroundColor DarkGray

if ($Install) {
    Write-Host "==> Memasang APK Release ke perangkat fisik via ADB..." -ForegroundColor Cyan
    adb install -r $apkPath
    if ($LASTEXITCODE -eq 0) {
        Write-Host "==> Sukses terpasang di perangkat!" -ForegroundColor Green
        adb shell monkey -p com.hyunnism.deployinstant -c android.intent.category.LAUNCHER 1
    }
}

Write-Host "==> Proses rilis selesai dengan sukses! 🚀" -ForegroundColor Green

