# Movie Data Capture - Windows User Guide

Complete guide for Windows users.

---

## Quick Start for Windows Users

### 1. Install Rust (One-time Setup)

**Download and run**: https://rustup.rs

This installs:
- Rust compiler
- Cargo (package manager)
- Build tools

**Note**: You may need to install Visual Studio Build Tools if prompted.

---

### 2. Build MDC

**Option A: Using Batch File (Easiest)**

1. Open Command Prompt
2. Navigate to project folder:
   ```cmd
   cd C:\path\to\Movie_Data_Capture\rust
   ```
3. Run build script:
   ```cmd
   build-windows.bat
   ```
4. Wait 5-10 minutes for first build
5. Binaries ready in `target\release\`

**Option B: Using PowerShell (Recommended)**

1. Open PowerShell
2. Navigate to project folder:
   ```powershell
   cd C:\path\to\Movie_Data_Capture\rust
   ```
3. Run build script:
   ```powershell
   .\build-windows.ps1
   ```

**Option C: Manual Build**

```cmd
cargo build --release --workspace
```

---

### 3. Use MDC

#### Process a Single Movie

```cmd
target\release\mdc-cli.exe "C:\Movies\ABP-001.mp4"
```

#### Scan a Folder

```cmd
target\release\mdc-cli.exe "C:\Movies" -s
```

#### With Custom Output

```cmd
target\release\mdc-cli.exe "C:\Movies" -s -o "C:\Organized"
```

---

## Installation Options

### Option 1: Portable Installation

1. Build as shown above
2. Copy `mdc-cli.exe` and `mdc-server.exe` to desired location
3. Create shortcut on desktop (optional)

**Advantages**:
- No installation required
- Easy to move/update
- No system changes

---

### Option 2: Add to PATH

Makes `mdc-cli` available from anywhere.

**Steps**:

1. Create folder: `C:\MDC\`
2. Copy binaries:
   ```cmd
   copy target\release\mdc-cli.exe C:\MDC\
   copy target\release\mdc-server.exe C:\MDC\
   ```
3. Add to PATH:
   - Press `Win + X`, select "System"
   - Click "Advanced system settings"
   - Click "Environment Variables"
   - Under "System variables", find "Path"
   - Click "Edit" → "New"
   - Add: `C:\MDC`
   - Click "OK" on all dialogs
4. Restart Command Prompt
5. Test:
   ```cmd
   mdc-cli.exe --version
   ```

**Now you can use MDC from anywhere**:
```cmd
mdc-cli.exe "C:\Any\Path\movie.mp4"
```

---

## Common Windows Workflows

### Workflow 1: First-Time Organization

**Goal**: Organize messy movie collection with metadata

```cmd
REM 1. Backup your files first!
xcopy /E /I "C:\Movies" "C:\Movies_Backup"

REM 2. Test with one file first
mdc-cli.exe "C:\Movies\test.mp4" -g

REM 3. If looks good, process all with debug
mdc-cli.exe "C:\Movies" -s -o "C:\Organized" -g

REM 4. Check results in C:\Organized
```

**Time estimate**: 10-30 seconds per movie

---

### Workflow 2: Re-organize Existing Collection

**Goal**: Change folder structure without fetching metadata

```cmd
REM Use mode 2 (organizing only)
mdc-cli.exe "C:\Movies" -s -m 2 -o "C:\Reorganized"
```

**Time estimate**: 1-5 seconds per movie

---

### Workflow 3: Update Metadata Only

**Goal**: Refresh NFO files without moving files

```cmd
REM Use mode 3 (analysis/in-place)
mdc-cli.exe "C:\Movies" -s -m 3
```

**Time estimate**: 10-20 seconds per movie

---

### Workflow 4: Large Collection (1000+ files)

**Goal**: Process efficiently with concurrency

```cmd
REM Use 8 concurrent jobs
mdc-cli.exe "C:\Large\Collection" -s -j 8 -o "C:\Output"

REM Monitor progress in another window:
type C:\Output\logs\mdc.log
```

**Time estimate**: ~1-2 hours for 1000 movies

---

### Workflow 5: Network Drive Processing

**Goal**: Process files on NAS/Network drive

```cmd
REM Mount network drive
net use Z: \\NAS\Movies

REM Process with lower concurrency
mdc-cli.exe "Z:\" -s -j 2 -o "C:\Local\Output"

REM Disconnect when done
net use Z: /delete
```

**Tip**: Copy to local drive first for faster processing

---

## Windows-Specific Features

### Using Soft Links (Requires Admin)

**Why**: Keep original files, save disk space

**How**:

1. Open PowerShell as Administrator:
   - Right-click PowerShell → "Run as Administrator"

2. Run with soft link mode:
   ```powershell
   .\target\release\mdc-cli.exe "C:\Movies" -s -l 1 -o "C:\Organized"
   ```

3. Result:
   ```
   C:\Organized\  (symlinks, ~1MB total)
   C:\Movies\     (original files preserved)
   ```

**Check if working**:
```powershell
Get-Item "C:\Organized\MovieName\movie.mp4" | Select-Object -Property LinkType
# Should show: LinkType = SymbolicLink
```

---

### Using Hard Links (No Admin Required)

**Why**: Keep original files, no admin needed, same drive only

**How**:
```cmd
mdc-cli.exe "C:\Movies" -s -l 2 -o "C:\Organized"
```

**Limitations**:
- ✅ Works: `C:\Movies` → `C:\Organized`
- ❌ Fails: `C:\Movies` → `D:\Organized` (different drives)
- ❌ Fails: Network drives

---

## Windows File Paths

### Path Formats Supported

```cmd
REM Absolute paths (recommended)
mdc-cli.exe "C:\Movies\file.mp4"
mdc-cli.exe "D:\Media\Collection" -s

REM Relative paths
mdc-cli.exe ".\movies\file.mp4"
mdc-cli.exe "..\other\folder" -s

REM UNC paths (network)
mdc-cli.exe "\\NAS\Movies\file.mp4"
mdc-cli.exe "\\192.168.1.100\share" -s
```

### Special Characters in Paths

**If path has spaces, use quotes**:
```cmd
REM Correct:
mdc-cli.exe "C:\My Movies\file.mp4"

REM Wrong:
mdc-cli.exe C:\My Movies\file.mp4  ❌
```

**Chinese/Japanese characters work**:
```cmd
mdc-cli.exe "C:\電影\ムービー\file.mp4"  ✅
```

---

## Windows Performance Tips

### 1. Antivirus Exclusions

**Problem**: Antivirus slows down file operations

**Solution**: Add exclusions for:
- `mdc-cli.exe`
- `mdc-server.exe`
- Output folder (`C:\Organized`)

**Windows Defender**:
1. Settings → Update & Security → Windows Security
2. Virus & threat protection → Manage settings
3. Exclusions → Add exclusion
4. Add files and folders above

---

### 2. Disk Performance

**SSD vs HDD**:
- **SSD**: Use `-j 8` (8 concurrent jobs)
- **HDD**: Use `-j 2` or `-j 4`

**Check disk type**:
```powershell
Get-PhysicalDisk | Select-Object FriendlyName, MediaType
```

---

### 3. Network Drive Optimization

**Slow network drives**:

1. Copy to local first:
   ```cmd
   robocopy "Z:\Movies" "C:\Temp\Movies" /E /MT:8
   mdc-cli.exe "C:\Temp\Movies" -s -o "C:\Organized"
   ```

2. Or process directly with low concurrency:
   ```cmd
   mdc-cli.exe "Z:\Movies" -s -j 1
   ```

---

## Scheduled Tasks (Automation)

### Auto-Process New Movies Daily

**Setup**:

1. Create batch file `C:\MDC\daily-scan.bat`:
   ```batch
   @echo off
   C:\MDC\mdc-cli.exe "C:\Downloads\Movies" -s -o "C:\Organized" >> C:\MDC\daily.log 2>&1
   ```

2. Create scheduled task:
   - Open Task Scheduler
   - Create Basic Task
   - Name: "MDC Daily Scan"
   - Trigger: Daily at 2:00 AM
   - Action: Start Program
   - Program: `C:\MDC\daily-scan.bat`
   - Finish

**Result**: Automatically processes new movies every night

---

## Windows Troubleshooting

### "VCRUNTIME140.dll not found"

**Cause**: Missing Visual C++ Redistributable

**Solution**:
Download and install from Microsoft:
https://aka.ms/vs/17/release/vc_redist.x64.exe

---

### "OpenSSL not found"

**Cause**: OpenSSL missing (rare on Windows)

**Solution**:
1. Download from: https://slproweb.com/products/Win32OpenSSL.html
2. Install "Win64 OpenSSL v3.x.x Light"
3. Restart terminal
4. Rebuild: `build-windows.bat`

---

### "Permission Denied" Errors

**Causes & Solutions**:

1. **File in use**:
   - Close media players
   - Close Explorer windows showing that folder

2. **Read-only files**:
   ```cmd
   attrib -r "C:\Movies\*.*" /s
   ```

3. **Administrator needed** (for symlinks):
   - Right-click PowerShell → Run as Administrator

4. **Antivirus blocking**:
   - Add MDC to exclusions
   - Temporarily disable antivirus

---

### Slow Scanning

**Check**:

1. **Too many concurrent jobs**:
   ```cmd
   REM Reduce from 8 to 4
   mdc-cli.exe "C:\Movies" -s -j 4
   ```

2. **Antivirus scanning files**:
   - Add exclusions

3. **Network drive**:
   - Copy to local drive first

4. **Large folders**:
   - Process in batches:
   ```cmd
   mdc-cli.exe "C:\Movies\A-D" -s
   mdc-cli.exe "C:\Movies\E-H" -s
   ```

---

## Windows Batch Scripts

### Create Your Own Scripts

**Example: Quick Scan Script**

Create `quick-scan.bat`:
```batch
@echo off
echo Quick Movie Scanner
echo.
set /p input="Enter folder to scan: "
echo.
echo Scanning with 4 concurrent jobs...
C:\MDC\mdc-cli.exe "%input%" -s -j 4 -o "C:\Organized"
echo.
echo Done!
pause
```

Double-click to run!

---

### Example: Organize by Actor

Create `organize-by-actor.bat`:
```batch
@echo off
C:\MDC\mdc-cli.exe "C:\Movies" -s --location-rule "actor/number" -o "C:\Organized"
pause
```

---

## Integration with Windows Explorer

### Context Menu (Right-Click)

**Add "Process with MDC" to context menu**:

1. Create `process-with-mdc.reg`:
   ```reg
   Windows Registry Editor Version 5.00

   [HKEY_CLASSES_ROOT\*\shell\MDC]
   @="Process with MDC"

   [HKEY_CLASSES_ROOT\*\shell\MDC\command]
   @="\"C:\\MDC\\mdc-cli.exe\" \"%1\""
   ```

2. Double-click to add to registry
3. Right-click any video file → "Process with MDC"

---

## Web UI on Windows

### Start Web Server

```cmd
target\release\mdc-server.exe
```

**Access**: http://localhost:3000

### Run as Windows Service

**Using NSSM** (Non-Sucking Service Manager):

1. Download NSSM: https://nssm.cc/download
2. Install service:
   ```cmd
   nssm install MDC C:\MDC\mdc-server.exe
   nssm start MDC
   ```
3. Configure auto-start:
   ```cmd
   nssm set MDC Start SERVICE_AUTO_START
   ```

**Result**: MDC server runs automatically on Windows startup

---

## Docker on Windows

### Prerequisites

- Docker Desktop for Windows
- WSL 2 (Windows Subsystem for Linux)

### Setup

1. Install Docker Desktop: https://www.docker.com/products/docker-desktop
2. Enable WSL 2 integration
3. Navigate to project:
   ```powershell
   cd C:\path\to\Movie_Data_Capture\rust
   ```
4. Start server:
   ```powershell
   docker-compose up -d mdc-server
   ```
5. Access UI: http://localhost:3000

### Windows File Paths in Docker

Place movies in `movies` folder:
```
C:\Movie_Data_Capture\rust\movies\  ← Your movies here
```

Organized output appears in:
```
C:\Movie_Data_Capture\rust\output\  ← Results here
```

---

## Best Practices for Windows

### 1. Use Full Paths

```cmd
REM Good:
mdc-cli.exe "C:\Movies\file.mp4"

REM Avoid:
mdc-cli.exe ".\file.mp4"
```

### 2. Regular Backups

Before first run:
```cmd
robocopy "C:\Movies" "C:\Movies_Backup" /E /MT:8
```

### 3. Test First

```cmd
REM Test one file with debug
mdc-cli.exe "C:\Movies\test.mp4" -g

REM If good, process all
mdc-cli.exe "C:\Movies" -s
```

### 4. Use Analysis Mode First

```cmd
REM Try mode 3 first (no moving)
mdc-cli.exe "C:\Movies" -s -m 3

REM Check NFO files, then use mode 1
mdc-cli.exe "C:\Movies" -s -m 1
```

### 5. Monitor Disk Space

Check before processing:
```powershell
Get-PSDrive C | Select-Object Used,Free
```

---

## Support

### Getting Help

1. **Check logs**: `C:\Organized\logs\mdc.log`
2. **Run with debug**: `mdc-cli.exe ... -g`
3. **Read USER-GUIDE.md**
4. **GitHub Issues**: Report bugs

### Collecting Debug Info

```cmd
REM Run with debug and save output
mdc-cli.exe "C:\Movies" -s -g > debug.log 2>&1
```

Send `debug.log` when reporting issues.

---

**Windows Guide Version**: 1.0
**Last Updated**: 2025-12-27
**MDC Version**: 0.1.0 (Rust)

*Happy organizing on Windows! 🪟*
