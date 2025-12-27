# Movie Data Capture - Troubleshooting Guide

Solutions to common problems.

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Build Problems](#build-problems)
3. [Runtime Errors](#runtime-errors)
4. [Metadata Issues](#metadata-issues)
5. [File Operation Issues](#file-operation-issues)
6. [Performance Problems](#performance-problems)
7. [Windows-Specific Issues](#windows-specific-issues)
8. [Linux/macOS Issues](#linuxmacos-issues)
9. [Docker Issues](#docker-issues)
10. [Getting Help](#getting-help)

---

## Installation Issues

### "cargo: command not found" (All Platforms)

**Problem**: Rust not installed or not in PATH

**Solution**:

1. Install Rust: https://rustup.rs
2. Restart terminal
3. Verify:
   ```bash
   cargo --version
   ```

**Windows**: May need to restart computer

---

### "Visual Studio Build Tools required" (Windows)

**Problem**: Missing C++ build tools

**Solution**:

**Option 1**: Install Visual Studio Build Tools
1. Download: https://visualstudio.microsoft.com/downloads/
2. Select "Build Tools for Visual Studio"
3. Check "C++ build tools"
4. Install

**Option 2**: Use existing Visual Studio
```cmd
rustup default stable-msvc
```

---

### "pkg-config not found" (Linux)

**Problem**: Missing build dependencies

**Solution**:

**Ubuntu/Debian**:
```bash
sudo apt-get install pkg-config libssl-dev
```

**Fedora/RHEL**:
```bash
sudo dnf install pkg-config openssl-devel
```

**Arch**:
```bash
sudo pacman -S pkg-config openssl
```

---

## Build Problems

### "error: linker `link.exe` not found" (Windows)

**Problem**: MSVC linker not available

**Solution**:

1. Install Visual Studio Build Tools
2. Or switch to GNU toolchain:
   ```cmd
   rustup default stable-gnu
   rustup target add x86_64-pc-windows-gnu
   ```

---

### "compilation takes forever"

**Problem**: First build compiles all dependencies

**Expected behavior**: 5-10 minutes first time

**Speedup tips**:
1. Close other programs
2. Use release build: `cargo build --release`
3. Use more CPU cores (automatic)

**Subsequent builds**: Much faster (30s-2min)

---

### "out of memory during build"

**Problem**: Not enough RAM

**Solution**:

1. Close other applications
2. Build without parallelism:
   ```bash
   cargo build -j 1 --release
   ```
3. Add swap space (Linux):
   ```bash
   sudo fallocate -l 4G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   ```

---

### "failed to verify checksum"

**Problem**: Network corruption or caching issue

**Solution**:

```bash
# Clear cargo cache
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git/checkouts

# Retry build
cargo clean
cargo build --release
```

---

## Runtime Errors

### "No such file or directory"

**Problem**: File path incorrect or doesn't exist

**Diagnosis**:
```cmd
# Check file exists
dir "C:\path\to\file.mp4"  # Windows
ls /path/to/file.mp4       # Linux/macOS
```

**Common causes**:
1. **Typo in path**
2. **Using relative path** (use absolute)
3. **File moved/deleted**

**Solution**: Use correct absolute path

---

### "Permission denied"

**Problem**: Insufficient permissions

**Windows solutions**:
1. Run as Administrator
2. Check file is not read-only:
   ```cmd
   attrib -r "C:\Movies\*.*" /s
   ```
3. Close programs using the files
4. Antivirus may be blocking

**Linux/macOS solutions**:
1. Check permissions:
   ```bash
   ls -l /path/to/file
   ```
2. Fix permissions:
   ```bash
   chmod 644 /path/to/file
   ```
3. Use sudo if needed (not recommended)

---

### "Address already in use" (Server)

**Problem**: Port 3000 already in use

**Diagnosis**:

**Windows**:
```cmd
netstat -ano | findstr :3000
```

**Linux/macOS**:
```bash
lsof -i :3000
```

**Solutions**:

1. Kill other process using port
2. Use different port:
   ```cmd
   mdc-server.exe --port 3001
   ```

---

### "Failed to connect to database"

**Problem**: SQLite database locked or corrupted

**Solutions**:

1. Close other MDC instances
2. Delete database and restart:
   ```cmd
   del data\mdc.db
   ```
3. Check disk space

---

## Metadata Issues

### "No metadata found for [movie-number]"

**Possible causes & solutions**:

#### 1. Movie number not recognized

**Diagnosis**: Run with debug:
```cmd
mdc-cli.exe "file.mp4" -g
```

**Solution**: Override number:
```cmd
mdc-cli.exe "file.mp4" -n "CORRECT-001"
```

---

#### 2. Movie doesn't exist in database

**Check manually**:
- Visit javlibrary.com
- Search for the number
- If not found, movie may not exist

**Solution**: Try different scrapers or manual entry

---

#### 3. Network/scraper issues

**Diagnosis**:
```cmd
mdc-cli.exe "file.mp4" -g
# Look for connection errors
```

**Solutions**:
1. Check internet connection
2. Wait and retry (server may be down)
3. Try different scraper in config

---

### "Incorrect metadata retrieved"

**Possible causes**:
1. Wrong movie number detected
2. Database has wrong info
3. Different movie with same number

**Solutions**:

1. Verify number is correct
2. Override if needed: `-n "CORRECT-001"`
3. Try different scraper:
   ```ini
   # In config.ini
   [priority]
   website = javbus,javlibrary  # Try different order
   ```

---

### "Timeout while fetching metadata"

**Problem**: Scraper taking too long

**Solutions**:

1. Increase timeout in config:
   ```ini
   [proxy]
   timeout = 30
   ```

2. Check internet speed
3. Try different scraper
4. Use proxy if site blocked

---

## File Operation Issues

### "Failed to move file"

**Possible causes & solutions**:

#### 1. File in use

**Diagnosis**:
- File open in media player?
- Explorer window showing that folder?

**Solution**: Close all programs using the file

---

#### 2. Destination exists

**Problem**: File already exists in output folder

**Solution**:

1. Enable overwrite (future feature)
2. Delete destination manually
3. Use different output folder

---

#### 3. Cross-device operation

**Problem**: Can't hard link across drives

**Solution**: Use move (`-l 0`) or soft link (`-l 1`)

---

### "Symlink creation failed"

**Problem**: Insufficient permissions (Windows)

**Solution**:

**Windows 10/11**:
1. Enable Developer Mode:
   - Settings → Update & Security → For developers
   - Enable "Developer Mode"

**OR run as Administrator**:
- Right-click PowerShell → "Run as Administrator"

---

### "Hard link failed"

**Possible causes**:
1. **Different drives** (can't hard link across drives)
2. **Network drive** (not supported)
3. **File system** (FAT32 doesn't support)

**Solution**: Use `-l 0` (move) or `-l 1` (soft link)

---

### "Subtitle not moved"

**Check**:
1. Subtitle has matching filename?
   ```
   ✅ movie.mp4 + movie.srt
   ✅ movie.mp4 + movie-en.srt
   ❌ movie.mp4 + other.srt
   ```

2. Supported format?
   - ✅ `.srt`, `.ass`, `.ssa`, `.sub`, `.idx`, `.vtt`
   - ❌ `.txt`, `.doc`

---

## Performance Problems

### "Processing is very slow"

**Diagnosis checklist**:

1. **Check concurrent jobs**:
   ```cmd
   # Increase from 4 to 8
   mdc-cli.exe "C:\Movies" -s -j 8
   ```

2. **Check disk type**:
   - SSD: Fast ✅
   - HDD: Slow ⚠️
   - Network: Very slow ❌

3. **Check antivirus**:
   - Temporarily disable
   - Add MDC to exclusions

4. **Check mode**:
   - Mode 1 (scraping): Slow (network)
   - Mode 2 (organizing): Fast
   - Mode 3 (analysis): Medium

---

### "High CPU usage"

**Normal behavior**: MDC is processing-intensive

**If excessive**:
1. Reduce concurrent jobs: `-j 2`
2. Close other programs
3. Check for infinite loops (bug)

---

### "High memory usage"

**Normal**: MDC should use < 100MB for 10,000 files

**If excessive** (>1GB):
1. Report as bug
2. Restart MDC
3. Process in smaller batches

---

### "Disk usage 100%"

**Causes**:
1. Too many concurrent jobs on HDD
2. Antivirus scanning files
3. Other programs writing

**Solutions**:
1. Reduce concurrent jobs: `-j 2`
2. Add antivirus exclusions
3. Close other disk-intensive programs

---

## Windows-Specific Issues

### "VCRUNTIME140.dll is missing"

**Problem**: Missing Visual C++ Redistributable

**Solution**:

Download and install:
https://aka.ms/vs/17/release/vc_redist.x64.exe

---

### "Windows Defender blocking"

**Problem**: False positive detection

**Solution**:

1. Add exclusions:
   - Settings → Update & Security → Windows Security
   - Virus & threat protection → Manage settings
   - Exclusions → Add exclusion
   - Add `mdc-cli.exe` and `mdc-server.exe`

2. Or submit false positive:
   - https://www.microsoft.com/en-us/wdsi/filesubmission

---

### "Long path errors" (Windows)

**Problem**: Windows path length limit (260 characters)

**Solution**:

**Windows 10 1607+**: Enable long paths:

1. Open Registry Editor (regedit)
2. Navigate to:
   ```
   HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem
   ```
3. Set `LongPathsEnabled` to `1`
4. Restart

**OR** use shorter paths

---

### "Network drive not working"

**Problem**: UNC paths or mapped drives

**Solutions**:

1. Use mapped drive letter:
   ```cmd
   net use Z: \\NAS\Movies
   mdc-cli.exe "Z:\" -s
   ```

2. Copy to local drive first:
   ```cmd
   robocopy "\\NAS\Movies" "C:\Temp\Movies" /E
   mdc-cli.exe "C:\Temp\Movies" -s
   ```

---

## Linux/macOS Issues

### "libssl.so not found" (Linux)

**Problem**: Missing OpenSSL library

**Solution**:

**Ubuntu/Debian**:
```bash
sudo apt-get install libssl-dev
```

**Fedora/RHEL**:
```bash
sudo dnf install openssl-devel
```

**Rebuild**:
```bash
cargo clean
cargo build --release
```

---

### "Permission denied" (Linux/macOS)

**Problem**: File/directory permissions

**Solutions**:

1. Check ownership:
   ```bash
   ls -la /path/to/file
   ```

2. Fix ownership:
   ```bash
   sudo chown -R $USER:$USER /path/to/folder
   ```

3. Fix permissions:
   ```bash
   chmod -R 755 /path/to/folder
   ```

---

### "Too many open files" (Linux/macOS)

**Problem**: System file descriptor limit

**Temporary fix**:
```bash
ulimit -n 4096
```

**Permanent fix**:

**Linux**: Edit `/etc/security/limits.conf`:
```
*  soft  nofile  4096
*  hard  nofile  8192
```

**macOS**: Edit `/etc/launchd.conf`:
```
limit maxfiles 4096 8192
```

---

## Docker Issues

### "docker: command not found"

**Problem**: Docker not installed

**Solution**: Install Docker Desktop:
- Windows/Mac: https://www.docker.com/products/docker-desktop
- Linux: Use package manager

---

### "Cannot connect to Docker daemon"

**Problem**: Docker not running

**Solution**:

**Windows/Mac**: Start Docker Desktop

**Linux**:
```bash
sudo systemctl start docker
sudo systemctl enable docker
```

---

### "Volume mount failed"

**Problem**: Path not shared with Docker

**Solution (Windows)**:

1. Docker Desktop → Settings
2. Resources → File Sharing
3. Add the folder path
4. Apply & Restart

---

### "Container exits immediately"

**Diagnosis**:
```bash
docker logs mdc-server
```

**Common causes**:
1. Port already in use
2. Volume mount failed
3. Configuration error

**Solution**: Check logs and fix issue

---

## Getting Help

### Collect Debug Information

**Run with debug flag**:
```cmd
mdc-cli.exe "C:\Movies" -s -g > debug.log 2>&1
```

**Information to provide**:
1. MDC version: `mdc-cli.exe --version`
2. Operating system & version
3. Rust version: `cargo --version`
4. Command used
5. Error message (full text)
6. Debug log (`debug.log`)

---

### Check Logs

**Windows**:
```
C:\output\logs\mdc.log
%USERPROFILE%\.mlogs\mdc.log
```

**Linux/macOS**:
```
./output/logs/mdc.log
~/.mlogs/mdc.log
```

---

### Search Existing Issues

Before reporting, search:
- GitHub Issues
- This troubleshooting guide
- USER-GUIDE.md

---

### Report a Bug

**Include**:
1. System info (OS, version)
2. MDC version
3. Steps to reproduce
4. Expected vs actual behavior
5. Debug log
6. Sample filename (if relevant)

**GitHub**: https://github.com/yourrepo/Movie_Data_Capture/issues

---

### Get Community Help

- **Discussions**: GitHub Discussions
- **FAQ**: See USER-GUIDE.md
- **Documentation**: README.md, STATUS.md

---

## Common Error Messages

### "Number parsing error: could not parse number from filename"

**Meaning**: Filename format not recognized

**Solution**:
```cmd
mdc-cli.exe "file.mp4" -n "ABC-001"
```

---

### "Metadata fetch error: HTTP 404"

**Meaning**: Movie not found in database

**Solution**: Try different scraper or verify number correct

---

### "Failed to create folder"

**Meaning**: Can't create output directory

**Check**:
1. Permissions
2. Disk space
3. Path valid

---

### "Database is locked"

**Meaning**: Multiple MDC instances or file locked

**Solution**:
1. Close other MDC instances
2. Close any program accessing `data/mdc.db`
3. Restart computer if needed

---

### "WebSocket connection failed"

**Meaning**: Can't connect to server's WebSocket

**Check**:
1. Server running?
2. Correct port?
3. Firewall blocking?

---

## Still Having Issues?

1. **Read documentation**: USER-GUIDE.md
2. **Search issues**: GitHub Issues
3. **Ask community**: GitHub Discussions
4. **Report bug**: GitHub Issues (with debug info)

---

**Troubleshooting Guide Version**: 1.0
**Last Updated**: 2025-12-27

*Most problems have simple solutions!* 🔧
