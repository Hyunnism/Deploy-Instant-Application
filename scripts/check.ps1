# scripts/check.ps1
# Menjalankan validasi menyeluruh: format, linter, dan automated test

$ErrorActionPreference = "Stop"

Write-Host "==> [1/4] Checking UI build..." -ForegroundColor Cyan
npm --prefix ui run build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> [2/4] Checking Rust formatting..." -ForegroundColor Cyan
Set-Location "$PSScriptRoot\..\src-tauri"
cargo fmt --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> [3/4] Running Clippy..." -ForegroundColor Cyan
cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> [4/4] Running cargo test..." -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Set-Location "$PSScriptRoot\.."
Write-Host "==> [SUCCESS] All checks passed cleanly! 🎉" -ForegroundColor Green
