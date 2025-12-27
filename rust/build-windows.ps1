# Movie Data Capture - Windows Build Script (PowerShell)
# Builds both CLI and Server binaries with progress indicators

param(
    [switch]$Release,
    [switch]$Debug,
    [switch]$Test,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Movie Data Capture - Windows Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Rust is not installed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Rust from: https://rustup.rs" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

# Show Rust version
Write-Host "[Step 1/5] Checking Rust version..." -ForegroundColor Green
cargo --version
rustc --version
Write-Host ""

# Clean if requested
if ($Clean) {
    Write-Host "[Step 2/5] Cleaning previous builds..." -ForegroundColor Green
    cargo clean
    Write-Host "Clean complete!" -ForegroundColor Green
    Write-Host ""
}

# Determine build type
$buildType = if ($Debug) { "debug" } else { "release" }
$buildFlag = if ($Release -or !$Debug) { "--release" } else { "" }

Write-Host "[Step 2/5] Building $buildType binaries..." -ForegroundColor Green
Write-Host "This may take 5-10 minutes on first build..." -ForegroundColor Yellow
Write-Host ""

# Build with progress
$startTime = Get-Date
try {
    if ($buildFlag) {
        cargo build --release --workspace
    } else {
        cargo build --workspace
    }
} catch {
    Write-Host ""
    Write-Host "[ERROR] Build failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common fixes:" -ForegroundColor Yellow
    Write-Host "- Ensure Visual Studio 2019+ is installed" -ForegroundColor Yellow
    Write-Host "- Run: rustup default stable-msvc" -ForegroundColor Yellow
    Write-Host "- Update Rust: rustup update" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

$buildTime = (Get-Date) - $startTime
Write-Host ""
Write-Host "Build completed in $($buildTime.TotalSeconds) seconds" -ForegroundColor Green
Write-Host ""

# Run tests if requested
if ($Test) {
    Write-Host "[Step 3/5] Running tests..." -ForegroundColor Green
    cargo test --workspace --release
    Write-Host ""
}

# Verify binaries
Write-Host "[Step 3/5] Verifying binaries..." -ForegroundColor Green
$cliPath = "target\$buildType\mdc-cli.exe"
$serverPath = "target\$buildType\mdc-server.exe"

if (!(Test-Path $cliPath)) {
    Write-Host "[ERROR] mdc-cli.exe not found!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}
if (!(Test-Path $serverPath)) {
    Write-Host "[ERROR] mdc-server.exe not found!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[OK] mdc-cli.exe" -ForegroundColor Green
Write-Host "[OK] mdc-server.exe" -ForegroundColor Green
Write-Host ""

# Show binary info
Write-Host "[Step 4/5] Binary information:" -ForegroundColor Green
$cliSize = (Get-Item $cliPath).Length / 1MB
$serverSize = (Get-Item $serverPath).Length / 1MB
Write-Host "  CLI:    $($cliSize.ToString('0.00')) MB" -ForegroundColor Cyan
Write-Host "  Server: $($serverSize.ToString('0.00')) MB" -ForegroundColor Cyan
Write-Host ""

# Test run
Write-Host "[Step 5/5] Testing binary..." -ForegroundColor Green
& $cliPath --version
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Binaries location:" -ForegroundColor Yellow
Write-Host "  CLI:    $((Get-Item $cliPath).FullName)" -ForegroundColor Cyan
Write-Host "  Server: $((Get-Item $serverPath).FullName)" -ForegroundColor Cyan
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Quick Start:" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Process a movie:" -ForegroundColor Yellow
Write-Host '  .\target\release\mdc-cli.exe "C:\path\to\movie.mp4"' -ForegroundColor Cyan
Write-Host ""
Write-Host "Scan a folder:" -ForegroundColor Yellow
Write-Host '  .\target\release\mdc-cli.exe "C:\path\to\movies" -s' -ForegroundColor Cyan
Write-Host ""
Write-Host "Start API server:" -ForegroundColor Yellow
Write-Host "  .\target\release\mdc-server.exe" -ForegroundColor Cyan
Write-Host "  (Then visit http://localhost:3000)" -ForegroundColor Gray
Write-Host ""
Write-Host "See USER-GUIDE.md for complete documentation." -ForegroundColor Gray
Write-Host ""

Read-Host "Press Enter to exit"
