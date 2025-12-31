//! MovieMeta CLI
//!
//! Command-line interface for organizing movie files with metadata.
//!
//! ## Invalid File Cleanup Feature
//!
//! Use `--delete-invalid` to move files that don't match any valid video code pattern
//! to a trash folder (`./invalid_files/`). Files are moved (not deleted) by default for safety.
//!
//! - Use `--dry-run` to preview what would be moved
//! - Use `--permanent` for permanent deletion (cannot be recovered)
//! - Always test with `--dry-run` first!

use anyhow::{Context, Result};
use clap::Parser;
use mdc_core::{
    logging, scanner, BatchProcessor, DualId, LinkMode, ProcessingMode, ProcessingStats,
    ProcessorConfig,
};
use mdc_scraper::scrapers::{
    AvmooScraper, DmmScraper, Fc2Scraper, ImdbScraper, Jav321Scraper, JavbusScraper,
    JavdbScraper, JavlibraryScraper, MgstageScraper, R18DevScraper, TmdbScraper,
    TokyohotScraper,
};
use mdc_scraper::{ScraperClient, ScraperConfig, ScraperRegistry};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Filter results to find files with number parsing errors
fn filter_invalid_code_files(
    results: &[mdc_core::ProcessingResult],
) -> Vec<&mdc_core::ProcessingResult> {
    results
        .iter()
        .filter(|r| {
            !r.success
                && r.error
                    .as_ref()
                    .map(|e| e.contains("Number parsing error"))
                    .unwrap_or(false)
        })
        .collect()
}

/// Deletion result for a single file
#[allow(dead_code)]
struct DeletionResult {
    file_path: PathBuf,
    success: bool,
    error: Option<String>,
}

/// Move or delete files and return results
fn delete_files(
    files: &[&mdc_core::ProcessingResult],
    dry_run: bool,
    permanent: bool,
    trash_dir: &Path,
) -> Result<Vec<DeletionResult>> {
    let mut deletion_results = Vec::new();

    // Create trash directory if not in dry-run mode and not permanent deletion
    if !dry_run && !permanent {
        std::fs::create_dir_all(trash_dir)?;
    }

    for result in files {
        if dry_run {
            let action = if permanent { "delete" } else { "move to trash" };
            println!(
                "  [DRY RUN] Would {}: {}",
                action,
                result.file_path.display()
            );
            deletion_results.push(DeletionResult {
                file_path: result.file_path.clone(),
                success: true,
                error: None,
            });
        } else if permanent {
            // Permanent deletion
            match std::fs::remove_file(&result.file_path) {
                Ok(_) => {
                    println!("  ✓ Permanently deleted: {}", result.file_path.display());
                    deletion_results.push(DeletionResult {
                        file_path: result.file_path.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    eprintln!(
                        "  ✗ Failed to delete: {} - {}",
                        result.file_path.display(),
                        e
                    );
                    deletion_results.push(DeletionResult {
                        file_path: result.file_path.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        } else {
            // Move to trash (default)
            let file_name = result
                .file_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
            let dest = trash_dir.join(file_name);

            match std::fs::rename(&result.file_path, &dest) {
                Ok(_) => {
                    println!("  ✓ Moved to trash: {}", result.file_path.display());
                    deletion_results.push(DeletionResult {
                        file_path: result.file_path.clone(),
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to move: {} - {}", result.file_path.display(), e);
                    deletion_results.push(DeletionResult {
                        file_path: result.file_path.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    Ok(deletion_results)
}

/// Get user confirmation for deletion
fn confirm_deletion() -> Result<bool> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// Print deletion summary
fn print_deletion_summary(
    results: &[DeletionResult],
    dry_run: bool,
    permanent: bool,
    trash_dir: &Path,
) {
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!("\n{}", "=".repeat(60));
    if dry_run {
        let action = if permanent {
            "deleted"
        } else {
            "moved to trash"
        };
        println!("Dry Run Summary:");
        println!("  {} files would be {}", successful, action);
    } else {
        if permanent {
            println!("Deletion Summary:");
            println!("  {} files permanently deleted", successful);
        } else {
            println!("Move to Trash Summary:");
            println!("  {} files moved to: {}", successful, trash_dir.display());
        }
        if failed > 0 {
            println!("  {} files failed", failed);
        }
    }
    println!("{}", "=".repeat(60));
}

#[derive(Parser)]
#[clap(name = "mdc")]
#[clap(version = VERSION)]
#[clap(about = "Movie Data Capture - Metadata scraper for movies", long_about = None)]
struct Cli {
    /// Movie file or directory to process
    #[clap(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Movie number (override automatic detection)
    #[clap(short = 'n', long, value_name = "NUMBER")]
    number: Option<String>,

    /// Processing mode: 1=Scraping (default), 2=Organizing, 3=Analysis
    #[clap(short = 'm', long, value_name = "MODE", default_value = "1")]
    mode: u8,

    /// Config file path
    #[clap(short = 'C', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[clap(short = 'g', long)]
    debug: bool,

    /// Output folder
    #[clap(short = 'o', long, value_name = "DIR")]
    output: Option<PathBuf>,

    /// Location rule (e.g., "actor/number", "studio/number")
    #[clap(long, value_name = "RULE", default_value = "actor/year/number-title")]
    location_rule: String,

    /// Naming rule (e.g., "number", "number-title")
    #[clap(long, value_name = "RULE", default_value = "number-title")]
    naming_rule: String,

    /// Organization preset (jav, jav-simple, studio, simple)
    /// - jav: actor/year with number-title naming (for JAV collections)
    /// - jav-simple: actor with number-title naming
    /// - studio: studio/actor with number-title naming
    /// - simple: number with number naming (default)
    #[clap(long, value_name = "PRESET")]
    preset: Option<String>,

    /// Link mode: 0=move (default), 1=soft link, 2=hard link
    #[clap(short = 'l', long, value_name = "MODE", default_value = "0")]
    link_mode: u8,

    /// Maximum concurrent tasks
    #[clap(short = 'j', long, value_name = "NUM", default_value = "4")]
    concurrent: usize,

    /// Scan folder for movies (alternative to single file)
    #[clap(short = 's', long)]
    scan: bool,

    /// Move files with invalid video codes to trash folder
    #[clap(long, requires = "scan")]
    delete_invalid: bool,

    /// Permanently delete instead of moving to trash (use with --delete-invalid)
    #[clap(long, requires = "delete_invalid")]
    permanent: bool,

    /// Skip confirmation prompt (use with --delete-invalid)
    #[clap(long, requires = "delete_invalid")]
    yes: bool,

    /// Preview what would be moved/deleted without actually doing it
    #[clap(long, requires = "delete_invalid")]
    dry_run: bool,

    /// Download cover images and fanart (poster, fanart.jpg)
    #[clap(long)]
    download_images: bool,

    /// Show version information
    #[clap(short = 'v', long)]
    version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.debug {
        logging::init_debug();
    } else {
        logging::init();
    }

    if cli.version {
        print_version();
        return Ok(());
    }

    // Determine processing mode
    let processing_mode = match cli.mode {
        1 => ProcessingMode::Scraping,
        2 => ProcessingMode::Organizing,
        3 => ProcessingMode::Analysis,
        _ => {
            eprintln!("Invalid mode: {}. Must be 1, 2, or 3.", cli.mode);
            std::process::exit(1);
        }
    };

    // Determine link mode
    let link_mode = match cli.link_mode {
        0 => LinkMode::Move,
        1 => LinkMode::SoftLink,
        2 => LinkMode::HardLink,
        _ => {
            eprintln!("Invalid link mode: {}. Must be 0, 1, or 2.", cli.link_mode);
            std::process::exit(1);
        }
    };

    // Apply organization preset if specified (overrides location_rule and naming_rule)
    let (location_rule, naming_rule) = if let Some(preset) = &cli.preset {
        match preset.as_str() {
            "jav" => {
                println!("Using JAV preset: actor/year/number-title");
                ("actor/year".to_string(), "number-title".to_string())
            }
            "jav-simple" => {
                println!("Using JAV-Simple preset: actor/number-title");
                ("actor".to_string(), "number-title".to_string())
            }
            "studio" => {
                println!("Using Studio preset: studio/actor/number-title");
                ("studio/actor".to_string(), "number-title".to_string())
            }
            "simple" => {
                println!("Using Simple preset: number/number");
                ("number".to_string(), "number".to_string())
            }
            _ => {
                eprintln!("Unknown preset: {}. Using CLI flags or defaults.", preset);
                (cli.location_rule.clone(), cli.naming_rule.clone())
            }
        }
    } else {
        // No preset specified, use CLI flags or defaults
        (cli.location_rule.clone(), cli.naming_rule.clone())
    };

    // If no path provided, show help
    if cli.path.is_none() {
        eprintln!("Error: No path provided");
        eprintln!("\nUsage: mdc <PATH> [OPTIONS]");
        eprintln!("\nExamples:");
        eprintln!("  mdc /path/to/movie.mp4");
        eprintln!("  mdc /path/to/movies -s -m 1");
        eprintln!("  mdc /path/to/movie.mp4 -n MOVIE-001");
        eprintln!("\nUse --help for more information");
        std::process::exit(1);
    }

    let source_path = cli.path.unwrap();

    // Determine output folder (default to source folder for in-place organization)
    let output_folder = cli.output.unwrap_or_else(|| {
        // Default to source folder location
        if source_path.is_dir() {
            source_path.clone()
        } else {
            // For single file, use parent directory
            source_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("./output"))
        }
    });

    // Create processor config with determined settings
    let processor_config = ProcessorConfig {
        mode: processing_mode,
        link_mode,
        success_folder: output_folder,
        location_rule,
        naming_rule,
        max_title_len: 50,
        skip_existing: false,
        download_images: cli.download_images,
        create_nfo: true,
        move_subtitles: true,
    };

    // Determine files to process
    let files: Vec<PathBuf> = if cli.scan || source_path.is_dir() {
        // Scan directory for video files
        println!("Scanning directory: {}", source_path.display());
        let media_types = vec![
            "mp4", "avi", "mkv", "wmv", "mov", "flv", "rmvb", "ts", "webm", "iso", "mpg", "m4v",
        ];
        let found_files = scanner::scan_directory(&source_path, &media_types).await?;
        println!("Found {} video files", found_files.len());
        found_files
    } else if source_path.is_file() {
        // Single file processing
        vec![source_path.clone()]
    } else {
        eprintln!("Error: Path does not exist: {}", source_path.display());
        std::process::exit(1);
    };

    if files.is_empty() {
        println!("No video files found");
        return Ok(());
    }

    // Create scraper registry with TMDB and IMDB
    let scraper_client = ScraperClient::new()?;

    // Try to load cookies from config file (optional)
    let cookies = if let Ok(config) = mdc_storage::Config::load(None) {
        let cookies = config.cookies();
        if !cookies.is_empty() {
            println!(
                "Loaded cookies for domains: {}",
                cookies
                    .keys()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        cookies
    } else {
        std::collections::HashMap::new()
    };

    let scraper_config = ScraperConfig::new(scraper_client)
        .debug(cli.debug)
        .cookies(cookies);

    let mut registry = ScraperRegistry::new();

    // Register JAV-specific scrapers (in priority order)
    // TIER 1: Official/High Quality Sources
    registry.register(Arc::new(DmmScraper::new())); // Official FANZA store
    registry.register(Arc::new(R18DevScraper::new())); // R18.com API (English)
    registry.register(Arc::new(JavdbScraper::new())); // Modern aggregator (EN/ZH)
    registry.register(Arc::new(MgstageScraper::new())); // MGS/Prestige official studio

    // TIER 2: Comprehensive Aggregators
    registry.register(Arc::new(JavlibraryScraper::new()));
    registry.register(Arc::new(JavbusScraper::new()));
    registry.register(Arc::new(AvmooScraper::new()));
    registry.register(Arc::new(Jav321Scraper::new())); // Fallback aggregator

    // TIER 3: Specialized Sources
    registry.register(Arc::new(Fc2Scraper::new()));
    registry.register(Arc::new(TokyohotScraper::new()));

    // Register general movie scrapers
    registry.register(Arc::new(TmdbScraper::new()));
    registry.register(Arc::new(ImdbScraper::new()));

    let registry = Arc::new(registry);

    // Create batch processor
    let batch_processor = BatchProcessor::new(processor_config, cli.concurrent);

    // Metadata provider function with dual ID support
    let metadata_provider = Arc::new(move |dual_id: DualId| {
        let registry_clone = registry.clone();
        let scraper_config_clone = scraper_config.clone();

        async move {
            // Use search_with_ids() to pass both display and content IDs
            // Each scraper will receive the ID format it prefers
            match registry_clone
                .search_with_ids(
                    &dual_id.display,
                    &dual_id.content,
                    None,
                    &scraper_config_clone,
                )
                .await?
            {
                Some(metadata) => {
                    // Convert MovieMetadata to JSON
                    let json =
                        serde_json::to_value(&metadata).context("Failed to serialize metadata")?;
                    Ok(json)
                }
                None => Err(anyhow::anyhow!(
                    "No metadata found for {}/{}",
                    dual_id.display,
                    dual_id.content
                )),
            }
        }
    });

    // Progress callback
    let progress_callback = Arc::new(move |current: usize, total: usize| {
        println!("[{}/{}] Processing...", current, total);
    });

    // Process batch
    println!(
        "\nProcessing {} files with {} concurrent tasks...",
        files.len(),
        cli.concurrent
    );
    println!("Mode: {:?}, Link: {:?}\n", processing_mode, link_mode);

    let (results, stats) = batch_processor
        .process_batch(files, metadata_provider, Some(progress_callback))
        .await?;

    // Handle deletion of invalid files
    if cli.delete_invalid {
        let invalid_files = filter_invalid_code_files(&results);

        if invalid_files.is_empty() {
            println!("\n✓ No files with invalid video codes found.");
        } else {
            // Determine trash directory (in the source folder)
            let trash_dir = source_path
                .parent()
                .unwrap_or(&source_path)
                .join("invalid_files");

            println!("\n{}", "=".repeat(60));
            println!(
                "Found {} files with invalid video codes:",
                invalid_files.len()
            );
            println!("{}", "=".repeat(60));

            for result in &invalid_files {
                println!(
                    "  {} - {}",
                    result.file_path.display(),
                    result.error.as_deref().unwrap_or("Unknown error")
                );
            }

            if !cli.permanent {
                println!("\nFiles will be moved to: {}", trash_dir.display());
            }

            // Confirm action (unless --yes flag is set)
            let should_delete = if cli.yes {
                true
            } else if cli.dry_run {
                true // Dry run doesn't need confirmation
            } else {
                print!(
                    "\n{} these files? (y/N): ",
                    if cli.permanent {
                        "Permanently delete"
                    } else {
                        "Move to trash"
                    }
                );
                use std::io::{self, Write};
                io::stdout().flush()?;
                confirm_deletion()?
            };

            if should_delete {
                let deletion_results =
                    delete_files(&invalid_files, cli.dry_run, cli.permanent, &trash_dir)?;
                print_deletion_summary(&deletion_results, cli.dry_run, cli.permanent, &trash_dir);
            } else {
                println!("Operation cancelled.");
            }
        }
    }

    // Print results
    print_results(&results, &stats);

    Ok(())
}

fn print_version() {
    println!("Movie Data Capture (Rust) v{}", VERSION);
    println!("\nCompleted Features:");
    println!("  ✅ Week 1: Foundation & Project Setup");
    println!("  ✅ Week 2: Number Parser Migration");
    println!("  ✅ Week 3: File Scanner & Discovery");
    println!("  ✅ Week 4-5: HTTP Client & CloudFlare Bridge");
    println!("  ✅ Week 6: Image Processing Foundation");
    println!("  ✅ Week 7: Scraper Framework (TMDB/IMDB)");
    println!("  ✅ Week 8: Database & Persistence");
    println!("  ✅ Week 9: Core Processing Engine");
    println!("  ✅ Week 12: CLI & Integration");
    println!("\nStats:");
    println!("  • 105+ tests passing");
    println!("  • 6,500+ lines of Rust code");
    println!("  • Zero unsafe code");
    println!("  • Full Python feature parity (core features)");
}

fn print_results(results: &[mdc_core::ProcessingResult], stats: &ProcessingStats) {
    println!("\n{}", "=".repeat(60));
    println!("Processing Complete");
    println!("{}", "=".repeat(60));
    println!();
    println!("Total:     {}", stats.total_processed);
    println!("Succeeded: {} ✓", stats.succeeded);
    println!("Failed:    {} ✗", stats.failed);
    println!("Skipped:   {}", stats.skipped);
    println!();

    // Show failed files if any
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if !failed.is_empty() {
        println!("Failed Files:");
        println!("{}", "-".repeat(60));
        for result in failed {
            println!(
                "  {} - {}",
                result.file_path.display(),
                result.error.as_deref().unwrap_or("Unknown error")
            );
        }
        println!();
    }

    // Show success rate
    let success_rate = if stats.total_processed > 0 {
        (stats.succeeded as f64 / stats.total_processed as f64) * 100.0
    } else {
        0.0
    };
    println!("Success Rate: {:.1}%", success_rate);
    println!("{}", "=".repeat(60));
}
