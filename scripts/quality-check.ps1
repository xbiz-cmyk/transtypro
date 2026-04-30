# quality-check.ps1
# Run all safe quality checks from the repo root.
# Usage: pwsh scripts/quality-check.ps1
# Exit code: 0 if all checks pass, 1 if any check fails.
#
# Requires: node/npm, Rust/cargo. cargo must be on PATH or in $env:USERPROFILE\.cargo\bin.

$ErrorActionPreference = 'Continue'
$failed = @()

# Extend PATH so cargo is reachable on Windows installs that don't add it globally.
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $env:PATH = "$cargoBin;$env:PATH"
}

function Run-Check {
    param(
        [string]$Label,
        [scriptblock]$Block
    )
    Write-Host ""
    Write-Host "==> $Label" -ForegroundColor Cyan
    & $Block
    $exitCode = $LASTEXITCODE
    if ($null -eq $exitCode) { $exitCode = 0 }
    if ($exitCode -eq 0) {
        Write-Host "    PASS: $Label" -ForegroundColor Green
        return $true
    } else {
        Write-Host "    FAIL: $Label (exit $exitCode)" -ForegroundColor Red
        return $false
    }
}

Write-Host ""
Write-Host "======================================" -ForegroundColor White
Write-Host "  transtypro quality-check.ps1" -ForegroundColor White
Write-Host "======================================" -ForegroundColor White

# 1. Frontend build
$result = Run-Check -Label "npm run build" -Block { npm run build }
if (-not $result) { $failed += "npm run build" }

# 2. Frontend lint (TypeScript type check without emit)
$result = Run-Check -Label "npm run lint" -Block { npm run lint }
if (-not $result) { $failed += "npm run lint" }

# 3. Rust format check
$result = Run-Check -Label "cargo fmt --check" -Block {
    cargo fmt --check --manifest-path src-tauri/Cargo.toml
}
if (-not $result) { $failed += "cargo fmt --check" }

# 4. Cargo clippy
$result = Run-Check -Label "cargo clippy" -Block {
    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
}
if (-not $result) { $failed += "cargo clippy" }

# 5. Cargo test
$result = Run-Check -Label "cargo test" -Block {
    cargo test --manifest-path src-tauri/Cargo.toml
}
if (-not $result) { $failed += "cargo test" }

# 5. Git status (informational — always passes)
Write-Host ""
Write-Host "==> git status" -ForegroundColor Cyan
git status --short
Write-Host "    PASS: git status" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "======================================" -ForegroundColor White
Write-Host "  SUMMARY" -ForegroundColor White
Write-Host "======================================" -ForegroundColor White

if ($failed.Count -eq 0) {
    Write-Host "  All checks passed." -ForegroundColor Green
    exit 0
} else {
    Write-Host "  Failed checks:" -ForegroundColor Red
    foreach ($f in $failed) {
        Write-Host "    - $f" -ForegroundColor Red
    }
    exit 1
}
