# Movie Data Capture - Quick Start Guide

Get up and running in 5 minutes!

---

## For Windows Users (Most Common)

### Step 1: Install Rust (5 minutes)

1. Go to: **https://rustup.rs**
2. Download and run `rustup-init.exe`
3. Follow prompts (accept defaults)
4. Restart terminal/PowerShell

**Verify installation**:
```cmd
cargo --version
```

---

### Step 2: Build MDC (5-10 minutes first time)

1. **Download or clone** this repository
2. **Open PowerShell** in the `rust` folder
3. **Run**:
   ```powershell
   .\build-windows.ps1
   ```

**Wait for build to complete** (shows progress)

---

### Step 3: Process Your First Movie

```cmd
.\target\release\mdc-cli.exe "C:\path\to\movie.mp4"
```

**That's it!** Check the `output` folder for results.

---

## Common Usage Scenarios

### Scenario 1: I have a messy folder of movies

**Goal**: Organize and fetch metadata for all movies

```cmd
mdc-cli.exe "C:\MessyMovies" -s -o "C:\Organized"
```

**Result**:
```
C:\Organized\
├── ABC-001\
│   ├── ABC-001.mp4
│   ├── ABC-001.nfo
│   ├── poster.jpg
│   └── fanart.jpg
├── ABC-002\
│   ├── ABC-002.mp4
│   ├── ABC-002.nfo
│   └── ...
```

---

### Scenario 2: I want to test first

**Goal**: Process without moving files (safe)

```cmd
mdc-cli.exe "C:\Movies" -s -m 3
```

**Result**: NFO files created in same folder, files stay put

---

### Scenario 3: I have 1000+ movies

**Goal**: Fast processing with multiple concurrent jobs

```cmd
mdc-cli.exe "C:\LargeCollection" -s -j 8 -o "C:\Output"
```

**Result**: Processes 8 movies at once (much faster!)

---

### Scenario 4: I want a web interface

**Goal**: Use browser instead of command line

1. **Start server**:
   ```cmd
   mdc-server.exe
   ```

2. **Open browser**: http://localhost:3000

3. **Use the web UI** to:
   - View dashboard
   - Monitor jobs
   - Scan folders
   - Edit config

---

## Understanding the Basics

### Processing Modes

| Mode | What it does | Use when |
|------|--------------|----------|
| **1 (Scraping)** | Fetch metadata + organize | First time, new files |
| **2 (Organizing)** | Just organize, no metadata | Re-organizing existing |
| **3 (Analysis)** | Metadata only, no move | Testing, existing library |

**Examples**:
```cmd
mdc-cli.exe "C:\Movies" -s -m 1  # Full workflow
mdc-cli.exe "C:\Movies" -s -m 2  # Fast organize
mdc-cli.exe "C:\Movies" -s -m 3  # Safe test
```

---

### Link Modes

| Mode | What it does | Disk space | Pros |
|------|--------------|------------|------|
| **0 (Move)** | Move files | Original space freed | Clean |
| **1 (Soft link)** | Create symlinks | Minimal | Keeps originals |
| **2 (Hard link)** | Create hard links | Minimal | No admin needed |

**Examples**:
```cmd
mdc-cli.exe "C:\Movies" -s -l 0  # Move (default)
mdc-cli.exe "C:\Movies" -s -l 1  # Soft link (needs admin)
mdc-cli.exe "C:\Movies" -s -l 2  # Hard link
```

---

### Concurrent Jobs

Control how many movies process at once:

```cmd
mdc-cli.exe "C:\Movies" -s -j 1  # One at a time (safe)
mdc-cli.exe "C:\Movies" -s -j 4  # Four at once (default)
mdc-cli.exe "C:\Movies" -s -j 8  # Eight at once (fast)
```

**Recommendations**:
- **SSD**: Use `-j 8`
- **HDD**: Use `-j 2` or `-j 4`
- **Network**: Use `-j 2`

---

## File Naming Examples

MDC understands these patterns:

```
✅ ABC-001.mp4           → ABC-001
✅ ABC-123.avi           → ABC-123
✅ FC2-PPV-1234567.mp4   → FC2-PPV-1234567
✅ TOKYO-HOT-N0001.mp4   → N0001
✅ [Studio] ABC-001.mp4  → ABC-001
✅ ABC-001-C.mp4         → ABC-001 (Chinese sub)
✅ ABC-001-U.mp4         → ABC-001 (Uncensored)
✅ ABC-001-CD1.mp4       → ABC-001 (Multi-part)
```

**If filename is weird**, override:
```cmd
mdc-cli.exe "weird-name.mp4" -n "ABC-001"
```

---

## Quick Troubleshooting

### Problem: "cargo: command not found"

**Solution**: Install Rust from https://rustup.rs

---

### Problem: Build fails

**Solution**:
1. Update Rust: `rustup update`
2. Try again: `.\build-windows.ps1`
3. If still fails, install Visual Studio Build Tools

---

### Problem: No metadata found

**Possible causes**:
1. **Wrong movie number** → Try: `mdc-cli.exe file.mp4 -n "CORRECT-001"`
2. **Network issue** → Check internet
3. **Movie not in database** → Try different scraper

---

### Problem: Files not moving

**Check**:
1. Using mode 3? → Use mode 1 instead
2. Permission issues? → Run as administrator
3. Different drives? → Use mode 0 (move) not mode 2 (hard link)

---

### Problem: Too slow

**Solutions**:
1. Increase concurrent jobs: `-j 8`
2. Use mode 2 if metadata not needed
3. Check antivirus (add exclusions)
4. Use SSD not HDD

---

## Next Steps

### Learn More

- **Full documentation**: `USER-GUIDE.md`
- **Windows-specific**: `WINDOWS-GUIDE.md`
- **All features**: `README.md`

### Advanced Features

- **Custom folder structure**: `--location-rule "actor/number"`
- **Custom filenames**: `--naming-rule "number-title"`
- **Configuration file**: Create `config.ini`
- **Web UI**: Run `mdc-server.exe`
- **Docker**: Use `docker-compose up`

### Get Help

- **Debug mode**: Add `-g` to any command
- **Check logs**: Look in `logs/` folder
- **GitHub Issues**: Report bugs/ask questions

---

## Cheat Sheet

### Most Common Commands

```cmd
# Single file
mdc-cli.exe "C:\path\to\movie.mp4"

# Scan folder
mdc-cli.exe "C:\Movies" -s

# Custom output
mdc-cli.exe "C:\Movies" -s -o "C:\Output"

# Fast (organize only)
mdc-cli.exe "C:\Movies" -s -m 2

# Safe (test mode)
mdc-cli.exe "C:\Movies" -s -m 3

# Fast processing
mdc-cli.exe "C:\Movies" -s -j 8

# Debug mode
mdc-cli.exe "C:\Movies" -s -g

# Start web server
mdc-server.exe
```

### Useful Options

| Option | Short | Description |
|--------|-------|-------------|
| `--scan` | `-s` | Scan directory |
| `--mode` | `-m` | 1=Scrape, 2=Organize, 3=Analysis |
| `--link-mode` | `-l` | 0=Move, 1=Soft, 2=Hard |
| `--concurrent` | `-j` | Concurrent jobs (1-16) |
| `--output` | `-o` | Output directory |
| `--number` | `-n` | Override movie number |
| `--debug` | `-g` | Enable debug logging |

---

## Examples Gallery

### Example 1: Basic Usage

```cmd
# Input folder
C:\Downloads\
├── movie1.mp4
├── movie2.mp4
└── movie3.mp4

# Command
mdc-cli.exe "C:\Downloads" -s

# Output
C:\output\
├── ABC-001\
│   ├── ABC-001.mp4
│   └── ABC-001.nfo
├── ABC-002\
│   ├── ABC-002.mp4
│   └── ABC-002.nfo
└── ABC-003\
    ├── ABC-003.mp4
    └── ABC-003.nfo
```

---

### Example 2: Organize by Actor

```cmd
# Command
mdc-cli.exe "C:\Downloads" -s --location-rule "actor/number"

# Output
C:\output\
├── Actress One\
│   ├── ABC-001\
│   │   └── ABC-001.mp4
│   └── ABC-002\
│       └── ABC-002.mp4
└── Actress Two\
    └── DEF-001\
        └── DEF-001.mp4
```

---

### Example 3: Keep Original Location

```cmd
# Command (mode 3 = analysis)
mdc-cli.exe "C:\Movies" -s -m 3

# Result: Files stay put, NFO added
C:\Movies\
├── movie1.mp4
├── movie1.nfo  ← Added
├── movie2.mp4
└── movie2.nfo  ← Added
```

---

## Success Checklist

- [ ] Rust installed (`cargo --version` works)
- [ ] MDC built (`.\build-windows.ps1` completed)
- [ ] Binaries exist (`target\release\mdc-cli.exe`)
- [ ] Test run successful (`mdc-cli.exe --version`)
- [ ] Processed first movie successfully
- [ ] Results look good (NFO files created)
- [ ] Ready to process full collection

**If all checked**: You're ready to go! 🎉

---

**Quick Start Version**: 1.0
**Last Updated**: 2025-12-27

*Now go organize your movies!* 🎬
