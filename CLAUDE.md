# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Movie Data Capture (MDC) is a Python-based movie metadata scraper designed for local JAV (Japanese Adult Video) management with media servers like Emby, Jellyfin, and Kodi. The project automatically extracts movie numbers from filenames, scrapes metadata from various sources, downloads artwork, and generates NFO files for media library organization.

**Important**: This is an 18+ project for technical, academic, and local media organization purposes only.

## Development Commands

### Installation
```bash
pip3 install -r requirements.txt
```

### Running the Application
```bash
# Single file mode
python3 Movie_Data_Capture.py /path/to/movie.mp4

# Folder mode (uses config.ini settings)
python3 Movie_Data_Capture.py

# With custom number
python3 Movie_Data_Capture.py /path/to/movie.mp4 -n CUSTOM-001

# Debug mode
python3 Movie_Data_Capture.py -g

# Organizing mode (mode 2)
python3 Movie_Data_Capture.py -m 2

# Analysis folder mode (mode 3 - scrapes in place without moving files)
python3 Movie_Data_Capture.py -m 3

# Search for metadata only
python3 Movie_Data_Capture.py -s "MOVIE-001"
```

### Building Executable
```bash
make
# Or manually:
pyinstaller --onefile Movie_Data_Capture.py \
    --hidden-import ADC_function.py \
    --hidden-import core.py \
    --hidden-import "ImageProcessing.cnn" \
    --add-data "cloudscraper_path:cloudscraper" \
    --add-data "opencc_path:opencc" \
    --add-data "face_recognition_models_path:face_recognition_models" \
    --add-data "Img:Img" \
    --add-data "config.ini:."
```

### Docker
```bash
cd docker
docker-compose up -d
```

## Core Architecture

### Main Processing Flow

1. **Entry Point**: `Movie_Data_Capture.py`
   - Parses command-line arguments via `argparse_function()`
   - Loads configuration from `config.ini` (searched in multiple locations)
   - Sets up logging to `~/.mlogs/` by default
   - Manages main execution loop with optional rerun delays

2. **File Discovery**: `movie_lists()` in `Movie_Data_Capture.py`
   - Recursively scans source folder for video files
   - Filters by media type from config (mp4, avi, mkv, etc.)
   - Skips failed files (tracked in `failed_list.txt`)
   - Skips recently modified NFO files based on `nfo_skip_days`
   - Excludes trailers, small files, and escape folders

3. **Number Extraction**: `number_parser.py`
   - Extracts movie numbers from filenames using regex patterns
   - Handles special formats: FC2, Tokyo-Hot, Carib, 1Pondo, Heydouga, etc.
   - Supports custom regex via `config.ini`
   - Strips metadata suffixes: `-C` (Chinese sub), `-U` (uncensored), `-UC`, `-CD1/2`
   - See `G_TAKE_NUM_RULES` dict for site-specific extraction rules

4. **Metadata Scraping**: `scraper.py` + `scrapinglib/`
   - `get_data_from_json()` orchestrates the scraping process
   - `scrapinglib/api.py` iterates through configured sources
   - Each source is a module in `scrapinglib/` (tmdb.py, imdb.py, etc.)
   - Sources implement `Parser` base class with `scrape()` method
   - Returns unified JSON structure with title, actors, tags, cover, etc.
   - Applies special character replacement for filesystem compatibility
   - Handles translation if enabled (Google, Azure, DeepLX)
   - Performs traditional/simplified Chinese conversion via OpenCC

5. **Core Processing**: `core.py`
   - Three main modes:
     - **Mode 1**: Scraping mode - downloads metadata, moves files to organized structure
     - **Mode 2**: Organizing mode - only moves files, no metadata scraping
     - **Mode 3**: Analysis mode - scrapes metadata in place without moving files
   - `core_main()` handles full workflow:
     - Detects multi-part files (CD1, CD2)
     - Detects Chinese subtitles, 4K, ISO, uncensored markers
     - Creates folder structure via `create_folder()` using `location_rule`
     - Downloads cover images and extrafanart (supports parallel download)
     - Downloads actor photos to `.actors/` folder
     - Crops poster from cover using face detection (ImageProcessing module)
     - Adds watermarks for special attributes (sub, 4K, ISO)
     - Moves/links video files and subtitles
     - Generates NFO file via `print_files()`
   - `core_main_no_net_op()` for mode 3 offline operations (reprocessing covers only)

6. **Configuration**: `config.py`
   - Singleton pattern via `getInstance()`
   - Searches for config.ini in multiple locations (cwd, ~/.mdc.ini, ~/.config/mdc/)
   - Supports runtime overrides via `set_override()` and `-C` CLI flag
   - All settings accessed through type-safe methods

### Key Modules

- **ADC_function.py**: Utility functions for downloads, file operations, HTML parsing, translation
- **ImageProcessing/**: Face detection and image cropping (HOG or CNN models)
- **scrapinglib/**: Pluggable scrapers for different metadata sources
- **xlog.py**: Logging utilities

### Data Flow

```
Filename → number_parser.py → Movie Number
    ↓
Movie Number → scrapinglib/api.py → Iterate Sources
    ↓
Source Module → HTTP Request → Parse HTML/JSON → Unified JSON
    ↓
JSON → scraper.py → Translation/Conversion → Normalized JSON
    ↓
Normalized JSON → core.py → Download Assets + Generate NFO + Move Files
```

### File Organization

The `location_rule` and `naming_rule` in config.ini control output structure using Python expressions:
- `location_rule = actor + "/" + number` → `ActorName/MOVIE-001/`
- `naming_rule = number + "-" + title` → `MOVIE-001-Movie Title.mp4`

Available variables: `number`, `title`, `actor`, `studio`, `director`, `series`, `year`

### Modes Explained

- **Mode 1 (Scraping)**: Default. Fetches metadata, downloads images, creates organized folder structure
- **Mode 2 (Organizing)**: Only moves files into folder structure, no metadata fetching
- **Mode 3 (Analysis Folder)**: Scrapes metadata but leaves files in place (for soft-link setups)

### Link Modes

- `link_mode=0`: Move files (default)
- `link_mode=1`: Create soft links
- `link_mode=2`: Create hard links (falls back to soft links if impossible)

### Failed File Handling

- Failed files are tracked in `failed_output_folder/failed_list.txt`
- Skipped on subsequent runs unless `--ignore-failed-list` is used
- Use mode 3 to retry failed files without moving them

### NFO File Structure

Generated NFO files follow Kodi/Jellyfin schema:
- Title, outline, runtime, director, actors with photos
- Studio, series, label, release date, rating
- Tags and genres (with jellyfin-specific options)
- Poster/fanart paths
- Rating from source sites (normalized to 10-point scale)

## Configuration Notes

- Config file searched in order: `config.ini` (cwd) → `~/mdc.ini` → `~/.mdc.ini` → `~/.mdc/config.ini` → `~/.config/mdc/config.ini`
- First instance becomes singleton accessible via `config.getInstance()`
- Override config at runtime: `-C "section:key=value;key2=value2"`
- Section/key names can be abbreviated if unambiguous: `-C "d:s=1"` → `debug_mode:switch=1`

## Common Patterns

### Adding a New Scraper Source

1. Create `scrapinglib/newsource.py`
2. Implement class inheriting from `scrapinglib.parser.Parser`
3. Override `scrape(keyword, scraping)` method
4. Return JSON string with required fields: `number`, `title`, `actor`, `cover`, `source`, etc.
5. Add source name to `[priority]website` in config.ini

### Number Parser Rules

Add custom patterns to `G_TAKE_NUM_RULES` dict in `number_parser.py`:
```python
'site-pattern': lambda x: str(re.search(r'pattern', x, re.I).group()),
```

### Image Processing

- Face detection uses dlib with HOG or CNN models
- `imagecut` values: 1=smart crop, 2=right half, 3=left half, 0=no crop
- Controlled by `[face]` section in config.ini
- Uncensored movies use different crop logic

## Important Constants

- `version = '6.6.7'` in `Movie_Data_Capture.py`
- Default log directory: `~/.mlogs/` (auto-merged by date/month/year)
- Supported media extensions: `.mp4,.avi,.rmvb,.wmv,.mov,.mkv,.flv,.ts,.webm,.iso,.mpg,.m4v`
- Supported subtitle extensions: `.smi,.srt,.idx,.sub,.sup,.psb,.ssa,.ass,.usf,.xss,.ssf,.rt,.lrc,.sbv,.vtt,.ttml`

## Testing & Debugging

- Use `-g` or `--debug` flag for detailed output
- Debug mode shows HTTP requests, JSON data, file operations
- Log files in `~/.mlogs/` with automatic rotation/merging
- Use `-z` or `--zero-operation` to preview without actual operations
- Signal handlers: Ctrl+C to abort, Ctrl+Break (Windows) or SIGWINCH (Linux) to toggle debug mode at runtime
