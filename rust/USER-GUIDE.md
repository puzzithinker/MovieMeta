# Movie Data Capture - Complete User Guide

**Version**: 0.1.0 (Rust)
**Platform**: Windows, Linux, macOS
**Last Updated**: 2025-12-27

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Command-Line Interface](#command-line-interface)
5. [Processing Modes](#processing-modes)
6. [Configuration](#configuration)
7. [Web Interface](#web-interface)
8. [Docker Deployment](#docker-deployment)
9. [Metadata Scrapers](#metadata-scrapers)
10. [File Organization](#file-organization)
11. [Advanced Usage](#advanced-usage)
12. [Troubleshooting](#troubleshooting)
13. [FAQ](#faq)

---

## Introduction

Movie Data Capture (MDC) is a powerful metadata scraper and organizer for local movie collections. The Rust version provides:

- **7 Metadata Scrapers**: 5 JAV-specific + 2 general movie databases
- **3 Processing Modes**: Scraping, Organizing, Analysis
- **Multiple Interfaces**: CLI, REST API, Web UI
- **High Performance**: 3-10x faster than Python version
- **Cross-Platform**: Windows, Linux, macOS

### What It Does

1. **Scans** your movie folders
2. **Extracts** movie numbers from filenames
3. **Fetches** metadata from online databases
4. **Downloads** cover images and actor photos
5. **Generates** NFO files for media servers
6. **Organizes** files into a structured folder hierarchy

### Supported Media Servers

- ✅ Emby
- ✅ Jellyfin
- ✅ Kodi
- ✅ Plex (with proper NFO plugins)

---

## Installation

### Windows

#### Option 1: Pre-built Binary (Recommended)

1. Download the latest release from GitHub
2. Extract `mdc-cli.exe` and `mdc-server.exe` to a folder
3. Add the folder to your PATH (optional)

#### Option 2: Build from Source

**Prerequisites**:
- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- Visual Studio 2019+ (for MSVC toolchain)
- Git

**Build Steps**:
```cmd
:: Clone repository
git clone https://github.com/yourrepo/Movie_Data_Capture.git
cd Movie_Data_Capture\rust

:: Build release binaries
build-windows.bat

:: Binaries will be in target\release\
```

### Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/yourrepo/Movie_Data_Capture.git
cd Movie_Data_Capture/rust
cargo build --release

# Binaries in target/release/
```

### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
brew install pkg-config openssl

# Clone and build
git clone https://github.com/yourrepo/Movie_Data_Capture.git
cd Movie_Data_Capture/rust
cargo build --release

# Binaries in target/release/
```

---

## Quick Start

### Windows Quick Start

1. **Open PowerShell or Command Prompt**

2. **Process a single file**:
   ```cmd
   mdc-cli.exe "C:\Movies\ABP-001.mp4"
   ```

3. **Scan a folder**:
   ```cmd
   mdc-cli.exe "C:\Movies" -s
   ```

4. **With custom output**:
   ```cmd
   mdc-cli.exe "C:\Movies" -s -o "C:\Organized"
   ```

### Linux/macOS Quick Start

```bash
# Process single file
./mdc-cli /path/to/movie.mp4

# Scan folder
./mdc-cli /path/to/movies -s

# Custom output
./mdc-cli /path/to/movies -s -o /output
```

---

## Command-Line Interface

### Basic Syntax

```
mdc-cli [PATH] [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `PATH` | File or folder to process | Required |
| `-s, --scan` | Scan directory for movies | false |
| `-m, --mode <MODE>` | Processing mode (1-3) | 1 |
| `-n, --number <NUM>` | Override movie number | Auto-detect |
| `-o, --output <DIR>` | Output directory | ./output |
| `-l, --link-mode <MODE>` | Link mode (0-2) | 0 |
| `-j, --concurrent <NUM>` | Concurrent jobs | 4 |
| `-g, --debug` | Enable debug logging | false |
| `--location-rule <RULE>` | Folder naming rule | "number" |
| `--naming-rule <RULE>` | File naming rule | "number" |
| `-C, --config <FILE>` | Config file path | Auto-detect |
| `-v, --version` | Show version | - |
| `-h, --help` | Show help | - |

### Examples

#### Windows

```cmd
:: Process single file
mdc-cli.exe "C:\Movies\ABP-001.mp4"

:: Scan folder with debug
mdc-cli.exe "C:\Movies" -s -g

:: Custom output and mode
mdc-cli.exe "C:\Movies" -s -o "C:\Organized" -m 1

:: Override movie number
mdc-cli.exe "C:\Movies\unknown.mp4" -n "ABP-001"

:: Organize by actor
mdc-cli.exe "C:\Movies" -s --location-rule "actor/number"

:: Use soft links
mdc-cli.exe "C:\Movies" -s -l 1

:: Process with 8 concurrent jobs
mdc-cli.exe "C:\Movies" -s -j 8

:: Analysis mode (in-place, no move)
mdc-cli.exe "C:\Movies" -s -m 3
```

#### Linux/macOS

```bash
# Process single file
./mdc-cli /path/to/movie.mp4

# Scan with options
./mdc-cli /movies -s -o /output -m 1 -j 8

# Debug mode
./mdc-cli /movies -s -g

# Custom rules
./mdc-cli /movies -s --location-rule "studio/number"
```

---

## Processing Modes

### Mode 1: Scraping (Default)

**Full workflow** - Fetch metadata, download images, organize files

```cmd
mdc-cli.exe "C:\Movies" -s -m 1
```

**What it does**:
1. Scans folder for video files
2. Extracts movie numbers from filenames
3. Fetches metadata from scrapers (JAVLibrary, JAVBus, etc.)
4. Downloads cover images
5. Downloads actor photos
6. Generates NFO files
7. Organizes files into folders
8. Moves/links video and subtitle files

**Best for**: First-time organization, new files

---

### Mode 2: Organizing

**File organization only** - No metadata fetching

```cmd
mdc-cli.exe "C:\Movies" -s -m 2
```

**What it does**:
1. Scans folder for video files
2. Uses existing NFO files or filenames
3. Organizes files into folder structure
4. Moves/links files only

**Best for**: Re-organizing existing collection, fast processing

---

### Mode 3: Analysis

**In-place processing** - No file moving

```cmd
mdc-cli.exe "C:\Movies" -s -m 3
```

**What it does**:
1. Scans folder for video files
2. Fetches metadata from scrapers
3. Generates NFO files in current location
4. **Does NOT move files**

**Best for**: Existing organized libraries, soft-linked setups, retrying failed files

---

## Configuration

### Config File Locations (Windows)

MDC looks for config files in this order:

1. `config.ini` (current directory)
2. `%USERPROFILE%\.mdc.ini`
3. `%USERPROFILE%\.mdc\config.ini`
4. `%APPDATA%\mdc\config.ini`

### Config File Locations (Linux/macOS)

1. `config.ini` (current directory)
2. `~/.mdc.ini`
3. `~/.mdc/config.ini`
4. `~/.config/mdc/config.ini`

### Sample Config File

Create `config.ini`:

```ini
[common]
# Processing mode: 1=Scraping, 2=Organizing, 3=Analysis
main_mode = 1

# Link mode: 0=Move, 1=Soft link, 2=Hard link
link_mode = 0

# Output folder
success_folder = ./output

# Failed files folder
failed_folder = ./failed

[Name_Rule]
# Location rule: folder structure
# Variables: number, title, actor, studio, director, series, year
location_rule = number

# Naming rule: filename
naming_rule = number

[media]
# Supported media types (comma-separated)
media_type = .mp4,.avi,.mkv,.wmv,.mov,.flv,.ts,.webm,.iso,.mpg,.m4v

# Skip files with NFO modified within N days
nfo_skip_days = 0

[escape]
# Folders to skip (comma-separated)
folders = failed, already_processed, .actors

[priority]
# Scraper priority (comma-separated)
# Available: javlibrary, javbus, avmoo, fc2, tokyohot, tmdb, imdb
website = javlibrary,javbus,avmoo,fc2,tokyohot,tmdb,imdb

[proxy]
# HTTP proxy (optional)
proxy =
# Timeout in seconds
timeout = 10

[name_rule]
# Maximum title length
max_title_len = 50

[debug]
# Enable debug mode
switch = 0
```

### Configuration via CLI

Override config settings from command line:

```cmd
:: Override mode
mdc-cli.exe "C:\Movies" -s -m 2

:: Override output folder
mdc-cli.exe "C:\Movies" -s -o "C:\Custom"

:: Override location rule
mdc-cli.exe "C:\Movies" -s --location-rule "studio/number"
```

---

## Web Interface

### Starting the Web Server

**Windows**:
```cmd
mdc-server.exe
```

**Linux/macOS**:
```bash
./mdc-server
```

Server starts on: `http://localhost:3000`

### Web UI Features

1. **Dashboard** (`http://localhost:3000`)
   - View statistics (total, completed, failed, pending)
   - See recent jobs
   - Quick access to other pages

2. **Job Queue** (`http://localhost:3000/jobs`)
   - Monitor all processing jobs
   - Real-time progress updates via WebSocket
   - Retry failed jobs
   - Cancel running jobs

3. **Folder Scanner** (`http://localhost:3000/scan`)
   - Scan folders through web interface
   - Configure processing mode
   - Set link mode and concurrency
   - View scan results

4. **Configuration** (`http://localhost:3000/config`)
   - Edit settings through UI
   - Save configuration
   - No need to manually edit config files

### API Endpoints

The server provides a REST API:

- `GET /health` - Health check
- `GET /api/stats` - Get statistics
- `GET /api/jobs` - List all jobs
- `GET /api/jobs/:id` - Get job details
- `POST /api/jobs` - Create new job
- `POST /api/jobs/:id/retry` - Retry failed job
- `POST /api/jobs/:id/cancel` - Cancel job
- `POST /api/scan` - Scan folder
- `GET /api/config` - Get configuration
- `POST /api/config` - Update configuration

### WebSocket

Real-time progress updates:

```
ws://localhost:3000/ws/progress
```

---

## Docker Deployment

### Quick Start (Windows with Docker Desktop)

1. **Install Docker Desktop** from docker.com

2. **Navigate to project folder**:
   ```cmd
   cd Movie_Data_Capture\rust
   ```

3. **Start the server**:
   ```cmd
   docker-compose up -d mdc-server
   ```

4. **Access Web UI**: `http://localhost:3000`

### Docker Compose Services

```yaml
services:
  # API Server + WebSocket
  mdc-server:
    ports: ["3000:3000"]
    volumes:
      - ./movies:/app/input:ro
      - ./output:/app/output
      - ./data:/app/data
      - ./logs:/app/logs

  # CLI (on-demand)
  mdc-cli:
    profiles: ["cli"]
    volumes: [same as above]
```

### Using Docker CLI

```cmd
:: Start server
docker-compose up -d mdc-server

:: Process with CLI
docker-compose run mdc-cli /app/input -s -o /app/output

:: View logs
docker-compose logs -f mdc-server

:: Stop server
docker-compose down
```

### Folder Mapping (Windows)

Place movies in `movies` folder:

```
Movie_Data_Capture/
├── rust/
│   ├── movies/          ← Place your movies here
│   │   ├── movie1.mp4
│   │   └── movie2.mp4
│   ├── output/          ← Organized output
│   ├── data/            ← Database
│   └── logs/            ← Log files
```

---

## Metadata Scrapers

MDC includes 7 metadata scrapers:

### JAV-Specific Scrapers

1. **JAVLibrary** (`javlibrary`)
   - Comprehensive JAV database
   - Japanese and English support
   - Complete metadata (actors, genres, studio, etc.)
   - **Best for**: Comprehensive metadata
   - **URL**: javlibrary.com

2. **JAVBus** (`javbus`)
   - Popular JAV aggregator
   - Chinese/English interface
   - Extensive coverage
   - **Best for**: General JAV content
   - **URL**: javbus.com

3. **AVMOO** (`avmoo`)
   - Multi-language support
   - Good coverage
   - Similar to JAVBus
   - **Best for**: Alternative to JAVBus
   - **URL**: avmoo.com

4. **FC2** (`fc2`)
   - Specialized for FC2-PPV content
   - Amateur content support
   - FC2-specific metadata
   - **Best for**: FC2 movies only
   - **URL**: adult.contents.fc2.com

5. **Tokyo-Hot** (`tokyohot`)
   - Premium JAV studio
   - Official site scraping
   - Uncensored content
   - **Best for**: Tokyo-Hot movies only
   - **URL**: tokyo-hot.com

### General Movie Scrapers

6. **TMDB** (`tmdb`)
   - The Movie Database
   - General movies and TV shows
   - High-quality metadata
   - **Best for**: Western movies/TV
   - **URL**: themoviedb.org

7. **IMDB** (`imdb`)
   - Internet Movie Database
   - Comprehensive movie info
   - Rating data
   - **Best for**: Western movies
   - **URL**: imdb.com

### Scraper Priority

Scrapers are tried in order specified in config:

```ini
[priority]
website = javlibrary,javbus,avmoo,fc2,tokyohot,tmdb,imdb
```

**Default order**: JAV scrapers first, then general scrapers

---

## File Organization

### Location Rules

Control folder structure with `location_rule`:

| Rule | Example Output |
|------|----------------|
| `number` | `ABP-001/` |
| `actor` | `Actress Name/` |
| `actor/number` | `Actress Name/ABP-001/` |
| `studio/number` | `Studio Name/ABP-001/` |
| `studio/actor` | `Studio Name/Actress Name/` |
| `series/number` | `Series Name/ABP-001/` |

**Windows Example**:
```cmd
:: Organize by actor
mdc-cli.exe "C:\Movies" -s --location-rule "actor/number"

Output:
C:\Organized\
├── Actress One\
│   ├── ABP-001\
│   │   ├── ABP-001.mp4
│   │   ├── ABP-001.nfo
│   │   └── poster.jpg
│   └── ABP-002\
└── Actress Two\
```

### Naming Rules

Control filename with `naming_rule`:

| Rule | Example Output |
|------|----------------|
| `number` | `ABP-001.mp4` |
| `number-title` | `ABP-001-Movie Title.mp4` |
| `title` | `Movie Title.mp4` |

### File Attributes

MDC detects special attributes from filenames:

| Suffix | Meaning | NFO Tag |
|--------|---------|---------|
| `-C` | Chinese subtitle | `<tag>Chinese Subtitle</tag>` |
| `-U` | Uncensored | `<tag>Uncensored</tag>` |
| `-UC` | Uncensored + Chinese | Both tags |
| `-CD1`, `-CD2` | Multi-part | Separate entries |
| `4K` | 4K resolution | `<tag>4K</tag>` |
| `.iso` | ISO format | `<tag>ISO</tag>` |

**Examples**:
- `ABP-001-C.mp4` → Chinese subtitle version
- `ABP-001-U.mp4` → Uncensored version
- `ABP-001-CD1.mp4` → Part 1 of 2

---

## Advanced Usage

### Link Modes

#### Mode 0: Move (Default)

```cmd
mdc-cli.exe "C:\Movies" -s -l 0
```

**What it does**: Physically moves files to output folder

**Pros**:
- Clean organization
- Frees source space

**Cons**:
- Original files gone
- Can't undo easily

---

#### Mode 1: Soft Link

```cmd
mdc-cli.exe "C:\Movies" -s -l 1
```

**What it does**: Creates symbolic links to original files

**Pros**:
- Original files preserved
- No disk space duplication
- Reversible

**Cons**:
- Requires admin rights on Windows
- Media server must support symlinks

**Windows**: Run PowerShell as Administrator

---

#### Mode 2: Hard Link

```cmd
mdc-cli.exe "C:\Movies" -s -l 2
```

**What it does**: Creates hard links to original files

**Pros**:
- Original files preserved
- No disk space duplication
- Works on same drive
- No admin required

**Cons**:
- Must be on same drive/partition
- Not available for all file systems

---

### Batch Processing

**Process large collections efficiently**:

```cmd
:: Windows - 8 concurrent jobs
mdc-cli.exe "C:\Large\Collection" -s -j 8 -o "C:\Output"

:: Linux
./mdc-cli /large/collection -s -j 8 -o /output
```

**Recommendations**:
- **SSD**: 4-8 concurrent jobs
- **HDD**: 2-4 concurrent jobs
- **Network**: 2-4 concurrent jobs

---

### Failed File Handling

Files that fail processing are tracked in `failed_list.txt`:

**Windows Location**:
- `C:\Output\failed\failed_list.txt`

**Retry failed files**:

```cmd
:: Method 1: Use mode 3 (in-place)
mdc-cli.exe "C:\Output\failed" -s -m 3

:: Method 2: Re-run with original files
mdc-cli.exe "C:\Movies\failed" -s
```

---

### Custom Number Override

For files with weird names:

```cmd
:: Windows
mdc-cli.exe "C:\Movies\weird-filename.mp4" -n "ABP-001"

:: Linux
./mdc-cli /movies/weird-filename.mp4 -n "ABP-001"
```

---

### Subtitle Handling

MDC automatically handles subtitle files:

**Supported formats**:
- `.srt`, `.ass`, `.ssa`, `.sub`
- `.idx`, `.vtt`, `.smi`

**Matching rules**:
1. Same filename: `movie.mp4` + `movie.srt` ✅
2. Same prefix: `movie-en.srt`, `movie-zh.srt` ✅

**Example**:
```
Input:
├── ABP-001.mp4
├── ABP-001.srt
└── ABP-001-zh.ass

Output:
└── ABP-001/
    ├── ABP-001.mp4
    ├── ABP-001.srt
    └── ABP-001-zh.ass
```

---

## Troubleshooting

### Windows Issues

#### "mdc-cli.exe is not recognized"

**Solution 1**: Use full path
```cmd
C:\Path\To\mdc-cli.exe "C:\Movies" -s
```

**Solution 2**: Add to PATH
1. Copy `mdc-cli.exe` to `C:\MDC\`
2. Add `C:\MDC\` to System PATH
3. Restart terminal
4. Run: `mdc-cli.exe "C:\Movies" -s`

---

#### "Access Denied" when using soft links

**Cause**: Symbolic links require admin rights on Windows

**Solution**:
1. Right-click PowerShell → "Run as Administrator"
2. Run command:
   ```powershell
   .\mdc-cli.exe "C:\Movies" -s -l 1
   ```

**Alternative**: Use hard links (`-l 2`) or move mode (`-l 0`)

---

#### "Permission Denied" on files

**Causes**:
- Files in use by media player
- Antivirus scanning
- Read-only attribute

**Solutions**:
```cmd
:: Close media players
:: Disable antivirus temporarily
:: Remove read-only:
attrib -r "C:\Movies\*.mp4" /s
```

---

### General Issues

#### No metadata found

**Possible causes**:
1. **Filename format not recognized**
   - Try: `mdc-cli.exe file.mp4 -n "CORRECT-NUMBER"`

2. **Network issues**
   - Check internet connection
   - Try with debug: `-g`

3. **Movie doesn't exist in database**
   - Try different scrapers
   - Manual metadata entry needed

---

#### Slow processing

**Solutions**:

1. **Increase concurrent jobs**:
   ```cmd
   mdc-cli.exe "C:\Movies" -s -j 8
   ```

2. **Use mode 2 for re-organization**:
   ```cmd
   mdc-cli.exe "C:\Movies" -s -m 2
   ```

3. **Skip existing NFO files**:
   - Set `nfo_skip_days = 30` in config

---

#### Files not moving

**Check**:
1. **Mode 3 doesn't move files** (by design)
   - Use mode 1 instead: `-m 1`

2. **Link mode failures**:
   - Try move mode: `-l 0`
   - Check permissions

3. **Cross-drive issues with hard links**:
   - Use soft links or move mode

---

## FAQ

### General Questions

**Q: Is this safe for my files?**
A: Yes! Use mode 3 (`-m 3`) for testing first. It processes in-place without moving files. Or use link mode 1/2 to preserve originals.

**Q: What video formats are supported?**
A: `.mp4`, `.avi`, `.mkv`, `.wmv`, `.mov`, `.flv`, `.ts`, `.webm`, `.iso`, `.mpg`, `.m4v`

**Q: Can I undo organization?**
A: Yes, if you used soft links (`-l 1`). Otherwise, you'll need to move files back manually.

**Q: Does it work offline?**
A: No, internet required for metadata scraping. Use mode 2 for offline organization if NFO files exist.

**Q: How long does it take?**
A: Depends on collection size. Roughly 10-30 seconds per file with metadata fetching.

---

### Scraper Questions

**Q: Which scraper should I use?**
A: For JAV: Try JAVLibrary or JAVBus first. For Western movies: TMDB or IMDB.

**Q: Can I use multiple scrapers?**
A: Yes! Configure priority in config file. MDC tries them in order until metadata found.

**Q: Why is metadata wrong?**
A: Some scrapers have incomplete data. Try a different scraper or override the number with `-n`.

**Q: Can I add custom scrapers?**
A: Not currently. Feature planned for future release.

---

### Organization Questions

**Q: Can I customize folder structure?**
A: Yes! Use `--location-rule` with variables: `number`, `title`, `actor`, `studio`, `director`, `series`, `year`

**Q: What if actor name has special characters?**
A: MDC automatically sanitizes filenames, replacing illegal characters.

**Q: Can I organize by multiple criteria?**
A: Yes! Example: `--location-rule "studio/actor/number"`

---

### Performance Questions

**Q: How many concurrent jobs?**
A: **SSD**: 4-8, **HDD**: 2-4, **Network**: 2-4

**Q: Can I speed up processing?**
A:
1. Use mode 2 (no metadata fetching)
2. Increase concurrent jobs
3. Use faster internet
4. Use SSD

**Q: How much disk space needed?**
A:
- **Move mode**: Same as source
- **Link modes**: Minimal (just NFO/images)
- **Images**: ~5MB per movie (covers + actors)

---

### Technical Questions

**Q: Where are databases stored?**
A: `./data/mdc.db` (SQLite)

**Q: Where are logs stored?**
A: `./logs/` or `~/.mlogs/`

**Q: Can I run multiple instances?**
A: Yes for CLI. No for server (port conflict). Use different ports: `mdc-server --port 3001`

**Q: Is it open source?**
A: Yes! Check the GitHub repository.

---

## Support & Resources

### Documentation
- **Main README**: `README.md`
- **Status**: `STATUS.md`
- **User Guide**: This file
- **Complete Summary**: `COMPLETE-SUMMARY.md`

### Community
- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share tips

### Contributing
Contributions welcome! See `README.md` for guidelines.

---

**Guide Version**: 1.0
**Last Updated**: 2025-12-27
**MDC Version**: 0.1.0 (Rust)

---

*Happy organizing! 🎬*
