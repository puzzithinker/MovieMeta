@echo off
REM Movie Data Capture - Windows Build Script
REM Builds both CLI and Server binaries for Windows

echo ========================================
echo Movie Data Capture - Windows Build
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Rust is not installed!
    echo.
    echo Please install Rust from: https://rustup.rs
    echo.
    pause
    exit /b 1
)

echo [1/4] Checking Rust version...
cargo --version
echo.

echo [2/4] Building release binaries...
echo This may take 5-10 minutes on first build...
echo.
cargo build --release --workspace

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    echo.
    echo Common fixes:
    echo - Ensure Visual Studio 2019+ is installed
    echo - Run: rustup default stable-msvc
    echo - Update Rust: rustup update
    echo.
    pause
    exit /b 1
)

echo.
echo [3/4] Verifying binaries...
if not exist "target\release\mdc-cli.exe" (
    echo [ERROR] mdc-cli.exe not found!
    pause
    exit /b 1
)
if not exist "target\release\mdc-server.exe" (
    echo [ERROR] mdc-server.exe not found!
    pause
    exit /b 1
)

echo [OK] mdc-cli.exe
echo [OK] mdc-server.exe
echo.

echo [4/4] Build complete!
echo.
echo Binaries location:
echo   CLI:    %CD%\target\release\mdc-cli.exe
echo   Server: %CD%\target\release\mdc-server.exe
echo.
echo Binary sizes:
dir target\release\mdc-cli.exe | find "mdc-cli.exe"
dir target\release\mdc-server.exe | find "mdc-server.exe"
echo.

echo ========================================
echo Quick Start:
echo ========================================
echo.
echo Process a movie:
echo   target\release\mdc-cli.exe "C:\path\to\movie.mp4"
echo.
echo Scan a folder:
echo   target\release\mdc-cli.exe "C:\path\to\movies" -s
echo.
echo Start API server:
echo   target\release\mdc-server.exe
echo   (Then visit http://localhost:3000)
echo.
echo See USER-GUIDE.md for complete documentation.
echo.

pause
