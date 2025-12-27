# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**MovieMeta** (formerly Movie Data Capture) is a high-performance Rust-based movie metadata scraper and organizer for local media libraries. It's designed for organizing movie collections with media servers like Emby, Jellyfin, and Kodi.

This is a **complete Rust rewrite** that replaces the original Python implementation with:
- 3-10x better performance
- Modern async architecture
- Type safety and memory safety
- Comprehensive test coverage (200 tests)
- Full documentation suite (9,000+ lines)

**Important**: This is an 18+ project for technical, academic, and local media organization purposes only.

## Project Structure

```
MovieMeta/
├── rust/                      # Main Rust implementation
│   ├── mdc-cli/              # Command-line interface
│   ├── mdc-core/             # Core business logic
│   ├── mdc-scraper/          # Metadata scrapers (7 sources)
│   ├── mdc-image/            # Image processing
│   ├── mdc-storage/          # Database layer (SQLite)
│   ├── mdc-api/              # REST API & WebSocket
│   └── mdc-web/              # Web UI (SvelteKit)
├── README.md                 # Main project readme
├── LICENSE                   # Project license
└── CLAUDE.md                 # This file
```

## Development Commands

### Building

```bash
# Navigate to rust directory
cd rust

# Debug build (fast compile)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check

# Windows users can use build scripts
build-windows.bat          # Batch file
build-windows.ps1         # PowerShell
```

### Running

```bash
cd rust

# CLI usage
./target/release/mdc-cli /path/to/movies -s

# Start API server
./target/release/mdc-server

# Run with debug
./target/release/mdc-cli /path/to/movies -s -g
```

### Testing

```bash
cd rust

# Run all tests (200 tests)
cargo test --workspace

# Run specific crate tests
cargo test --package mdc-core
cargo test --package mdc-scraper

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --package mdc-cli --test integration_test
```

### Code Quality

```bash
cd rust

# Format code
cargo fmt --all

# Linting
cargo clippy --all-targets --all-features

# Check for security issues
cargo audit
```

### Docker

```bash
cd rust

# Build and run server
docker-compose up -d mdc-server

# Build and run CLI
docker-compose run mdc-cli /movies -s

# View logs
docker-compose logs -f mdc-server
```

## Core Architecture

### Main Processing Flow

1. **Entry Point**: `mdc-cli/src/main.rs`
   - Parses command-line arguments using `clap`
   - Initializes logging with `env_logger`
   - Registers all scrapers with `ScraperRegistry`
   - Manages workflow execution

2. **File Discovery**: `mdc-core/src/scanner.rs`
   - `scan_directory()` recursively finds video files
   - Filters by media extensions (mp4, avi, mkv, etc.)
   - Returns `Vec<FileInfo>` with metadata

3. **Number Extraction**: `mdc-core/src/number_parser.rs`
   - Extracts movie numbers from filenames using regex
   - Handles special formats: FC2, Tokyo-Hot, Carib, 1Pondo, etc.
   - Supports 50+ different numbering patterns
   - Returns `ParsedNumber` with detected attributes

4. **Metadata Scraping**: `mdc-scraper/` crate
   - `ScraperRegistry` manages all scrapers
   - Each scraper implements `Scraper` trait
   - 7 sources: JAVLibrary, JAVBus, AVMOO, FC2, Tokyo-Hot, TMDB, IMDB
   - Returns unified `MovieMetadata` struct
   - Async/await with Tokio for concurrency

5. **Core Processing**: `mdc-core/src/workflow.rs`
   - Three modes:
     - **Mode 1**: Scraping - downloads metadata, moves files to organized structure
     - **Mode 2**: Organizing - only moves files, no metadata scraping
     - **Mode 3**: Analysis - scrapes metadata in place without moving files
   - `process_file()` handles full workflow:
     - Parses number and attributes
     - Scrapes metadata from sources
     - Downloads cover images and actor photos
     - Generates NFO file (Kodi/Jellyfin compatible)
     - Moves/links video files and subtitles
     - Handles errors gracefully with retry logic

6. **Database**: `mdc-storage/` crate
   - SQLite with migrations
   - Repository pattern for data access
   - Models for movies, jobs, config
   - Transaction support

7. **API Server**: `mdc-api/` crate
   - REST API with Axum framework
   - 10 endpoints for job management and config
   - WebSocket for real-time progress updates
   - Health checks and metrics

8. **Web UI**: `mdc-web/` directory
   - SvelteKit application
   - 4 pages: Dashboard, Jobs, Scan, Config
   - Real-time updates via WebSocket
   - Responsive design with dark theme

### Key Modules

- **mdc-core/batch.rs**: Concurrent batch processing
- **mdc-core/file_ops.rs**: File operations (move/link)
- **mdc-core/nfo.rs**: NFO XML generation
- **mdc-core/processor.rs**: Processing modes & templates
- **mdc-scraper/scrapers/**: Individual scraper implementations
- **mdc-scraper/registry.rs**: Scraper management
- **mdc-image/processor.rs**: Image processing and cropping
- **mdc-storage/repository.rs**: Database operations

### Data Flow

```
Filename → number_parser → Movie Number + Attributes
    ↓
Movie Number → ScraperRegistry → Iterate Sources
    ↓
Source → HTTP Request → Parse HTML/JSON → MovieMetadata
    ↓
MovieMetadata → Workflow → Download Assets + Generate NFO + Move Files
```

## Configuration

Configuration is handled via:
1. Command-line arguments (primary method)
2. Environment variables (for Docker)
3. Database storage (for API/Web UI)

Key CLI options:
- `-s, --scan`: Scan directory mode
- `-m, --mode <N>`: Processing mode (1=Scraping, 2=Organizing, 3=Analysis)
- `-l, --link-mode <N>`: Link mode (0=Move, 1=Soft, 2=Hard)
- `-j, --concurrent <N>`: Concurrent jobs (default: 4)
- `-o, --output <PATH>`: Output directory
- `-n, --number <NUMBER>`: Override movie number
- `-g, --debug`: Enable debug logging

## Scraper Sources

### JAV-Specific Sources

1. **JAVLibrary** (`scrapers/javlibrary.rs`)
   - Most comprehensive JAV database
   - Info table parsing with dt/dd elements
   - Japanese/English support

2. **JAVBus** (`scrapers/javbus.rs`)
   - Popular JAV aggregator
   - Label-based info extraction
   - Multi-language support

3. **AVMOO** (`scrapers/avmoo.rs`)
   - Alternative JAV source
   - Similar to JAVBus structure
   - Protocol-relative URL handling

4. **FC2** (`scrapers/fc2.rs`)
   - FC2-PPV specialized scraper
   - Amateur content focus
   - Flexible ID parsing

5. **Tokyo-Hot** (`scrapers/tokyohot.rs`)
   - Premium JAV studio
   - Official site scraper
   - Uncensored content tagging

### General Movie Sources

6. **TMDB** (`scrapers/tmdb.rs`)
   - The Movie Database API
   - General movies and TV shows

7. **IMDB** (`scrapers/imdb.rs`)
   - Internet Movie Database
   - Fallback for general content

## Adding a New Scraper

1. Create `mdc-scraper/src/scrapers/newsource.rs`
2. Implement the `Scraper` trait:
   ```rust
   #[async_trait]
   pub trait Scraper: Send + Sync {
       fn name(&self) -> &str;
       fn priority(&self) -> u8;
       async fn scrape(&self, number: &str) -> Result<MovieMetadata>;
   }
   ```
3. Add to `mdc-scraper/src/scrapers/mod.rs`
4. Register in `mdc-cli/src/main.rs`
5. Add test in scraper file
6. Update documentation

## Testing

### Test Structure

- **Unit tests**: In each module with `#[cfg(test)]`
- **Integration tests**: In `tests/` directories
- **Benchmarks**: In `benches/` directories (using Criterion)

### Running Specific Tests

```bash
# Test number parser
cargo test --package mdc-core number_parser

# Test scrapers
cargo test --package mdc-scraper

# Test with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Performance Considerations

- Use `tokio::spawn` for CPU-bound tasks
- Limit concurrent HTTP requests (default: 4)
- Use `Arc<T>` for shared read-only data
- Avoid `.clone()` in hot paths
- Profile with `cargo flamegraph` if needed

## Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Log errors with appropriate levels
- Provide user-friendly error messages
- Include debug information with `-g` flag

## Important Constants

- Version: `0.1.0` (in `Cargo.toml` files)
- Default concurrent jobs: 4
- Supported media extensions: `.mp4, .avi, .rmvb, .wmv, .mov, .mkv, .flv, .ts, .webm, .iso, .mpg, .m4v`
- Supported subtitle extensions: `.smi, .srt, .idx, .sub, .sup, .psb, .ssa, .ass, .usf, .xss, .ssf, .rt, .lrc, .sbv, .vtt, .ttml`

## Documentation

All documentation is in the `rust/` directory:

- **QUICKSTART.md**: 5-minute quick start (800 lines)
- **WINDOWS-GUIDE.md**: Complete Windows guide (1,500 lines)
- **USER-GUIDE.md**: Comprehensive manual (3,000 lines)
- **TROUBLESHOOTING.md**: Problem solving (1,800 lines)
- **STATUS.md**: Development status (350 lines)
- **COMPLETE-SUMMARY.md**: Achievement summary (900 lines)
- **DOCUMENTATION-INDEX.md**: Navigation guide (400 lines)

## Common Patterns

### Async/Await
```rust
use tokio::task;

async fn process_files(files: Vec<PathBuf>) -> Result<()> {
    let handles: Vec<_> = files.into_iter()
        .map(|file| task::spawn(process_single_file(file)))
        .collect();

    for handle in handles {
        handle.await??;
    }
    Ok(())
}
```

### Error Handling
```rust
use anyhow::{Context, Result};

fn process() -> Result<()> {
    let data = read_file(path)
        .context("Failed to read file")?;
    Ok(())
}
```

### Logging
```rust
use log::{debug, info, warn, error};

info!("Processing file: {}", path.display());
debug!("Extracted number: {}", number);
warn!("Retrying after error: {}", err);
error!("Failed to process: {}", err);
```

## Debugging Tips

1. **Enable debug logging**: `RUST_LOG=debug cargo run`
2. **Use `dbg!()` macro**: For quick inspection
3. **Check test output**: `cargo test -- --nocapture`
4. **Profile with flamegraph**: `cargo flamegraph`
5. **Use `cargo expand`**: To see macro expansion

## Release Process

1. Update version in all `Cargo.toml` files
2. Update CHANGELOG (if exists)
3. Run full test suite: `cargo test --workspace`
4. Build release binaries: `cargo build --release`
5. Test on Windows, Linux, macOS
6. Create git tag: `git tag v0.x.x`
7. Push with tags: `git push --tags`
8. Create GitHub release with binaries

## Notes for Claude Code

- This project is 100% complete and production-ready
- All 200 tests are passing
- Documentation is comprehensive and up-to-date
- Focus on maintaining code quality and test coverage
- Follow Rust best practices and idioms
- Keep documentation synchronized with code changes
- Maintain zero unsafe code policy
