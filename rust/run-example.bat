@echo off
REM Movie Data Capture - Example Usage Script (Windows)
REM Demonstrates common usage patterns

echo ========================================
echo Movie Data Capture - Example Usage
echo ========================================
echo.

REM Check if binaries exist
if not exist "target\release\mdc-cli.exe" (
    echo [ERROR] mdc-cli.exe not found!
    echo.
    echo Please build first using: build-windows.bat
    echo.
    pause
    exit /b 1
)

echo Select an example:
echo.
echo 1. Process a single file
echo 2. Scan a folder (scraping mode)
echo 3. Scan a folder (organizing mode)
echo 4. Scan with custom output
echo 5. Start API server
echo 6. Run all tests
echo 7. Show version
echo 0. Exit
echo.

set /p choice="Enter choice (0-7): "

if "%choice%"=="1" goto single
if "%choice%"=="2" goto scan
if "%choice%"=="3" goto organize
if "%choice%"=="4" goto custom
if "%choice%"=="5" goto server
if "%choice%"=="6" goto tests
if "%choice%"=="7" goto version
if "%choice%"=="0" goto end

echo Invalid choice!
pause
exit /b 1

:single
echo.
echo Example 1: Process a single file
echo ----------------------------------------
echo.
set /p filepath="Enter full path to movie file: "
if "%filepath%"=="" (
    echo No path entered!
    pause
    exit /b 1
)
echo.
echo Running: mdc-cli.exe "%filepath%"
echo.
target\release\mdc-cli.exe "%filepath%"
goto done

:scan
echo.
echo Example 2: Scan a folder (scraping mode)
echo ----------------------------------------
echo.
set /p folderpath="Enter folder path to scan: "
if "%folderpath%"=="" (
    echo No path entered!
    pause
    exit /b 1
)
echo.
echo Running: mdc-cli.exe "%folderpath%" -s -m 1 -g
echo (Mode 1 = Scraping with metadata)
echo.
target\release\mdc-cli.exe "%folderpath%" -s -m 1 -g
goto done

:organize
echo.
echo Example 3: Scan a folder (organizing mode)
echo ----------------------------------------
echo.
set /p folderpath="Enter folder path to organize: "
if "%folderpath%"=="" (
    echo No path entered!
    pause
    exit /b 1
)
echo.
echo Running: mdc-cli.exe "%folderpath%" -s -m 2
echo (Mode 2 = Organizing only, no metadata fetching)
echo.
target\release\mdc-cli.exe "%folderpath%" -s -m 2
goto done

:custom
echo.
echo Example 4: Scan with custom output
echo ----------------------------------------
echo.
set /p inputpath="Enter input folder: "
set /p outputpath="Enter output folder: "
if "%inputpath%"=="" (
    echo No input path entered!
    pause
    exit /b 1
)
if "%outputpath%"=="" (
    echo No output path entered!
    pause
    exit /b 1
)
echo.
echo Running: mdc-cli.exe "%inputpath%" -s -o "%outputpath%" -j 8
echo (8 concurrent jobs for faster processing)
echo.
target\release\mdc-cli.exe "%inputpath%" -s -o "%outputpath%" -j 8
goto done

:server
echo.
echo Example 5: Start API server
echo ----------------------------------------
echo.
echo Starting server on http://localhost:3000
echo.
echo Press Ctrl+C to stop the server
echo.
target\release\mdc-server.exe
goto end

:tests
echo.
echo Example 6: Run all tests
echo ----------------------------------------
echo.
cargo test --workspace --release
goto done

:version
echo.
echo Example 7: Show version
echo ----------------------------------------
echo.
target\release\mdc-cli.exe --version
echo.
target\release\mdc-server.exe --version
goto done

:done
echo.
echo ========================================
echo Example complete!
echo ========================================
echo.
pause
goto end

:end
