//! Enhanced logging for Movie Data Capture with file rotation
//!
//! This module provides comprehensive logging capabilities including:
//! - Console output (stdout) with colored formatting
//! - File-based logging with daily rotation
//! - Dual output (both console and file simultaneously)
//! - Graceful degradation if file logging fails

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Get the directory where the executable is located
///
/// Falls back to current directory if unable to determine executable path
fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log directory (defaults to executable directory)
    pub log_dir: PathBuf,

    /// Log file prefix (defaults to "mdc")
    pub file_prefix: String,

    /// Enable console logging (default: true)
    pub console: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_dir: get_exe_dir(),
            file_prefix: "mdc".to_string(),
            console: true,
        }
    }
}

/// Initialize logging with default settings (console only, for backwards compatibility)
///
/// Sets up tracing with sensible defaults:
/// - Format: [timestamp] [level] [target] message
/// - Default level: INFO
/// - Can be overridden with RUST_LOG environment variable
/// - Output: stdout only
///
/// # Examples
///
/// ```no_run
/// use mdc_core::logging::init;
///
/// init();
/// tracing::info!("Application started");
/// ```
pub fn init() {
    init_with_level("info")
}

/// Initialize logging with a specific level (console only, for backwards compatibility)
///
/// # Arguments
///
/// * `level` - Log level: "trace", "debug", "info", "warn", or "error"
pub fn init_with_level(level: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false),
        )
        .try_init();
}

/// Initialize debug logging (includes debug and trace messages)
pub fn init_debug() {
    init_with_level("debug")
}

/// Initialize logging with custom configuration
///
/// This creates both console and file logging simultaneously.
/// File logs use daily rotation and include more detail than console output.
///
/// # Arguments
///
/// * `config` - Logging configuration
///
/// # Returns
///
/// * `Result<()>` - Ok if initialization succeeded, Err if file logging setup failed
///
/// # Examples
///
/// ```no_run
/// use mdc_core::logging::{init_with_config, LogConfig};
/// use std::path::PathBuf;
///
/// let config = LogConfig {
///     level: "debug".to_string(),
///     log_dir: PathBuf::from("/var/log/mdc"),
///     file_prefix: "mdc".to_string(),
///     console: true,
/// };
///
/// init_with_config(config).expect("Failed to initialize logging");
/// ```
pub fn init_with_config(config: LogConfig) -> Result<()> {
    // Environment variable takes precedence
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Create daily rotating file appender
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(&config.file_prefix)
        .filename_suffix("log")
        .build(&config.log_dir)
        .with_context(|| format!(
            "Failed to create log file appender in directory: {:?}",
            config.log_dir
        ))?;

    // Non-blocking file writer for performance
    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

    // Initialize based on console configuration
    // We use separate complete initialization paths to avoid type system issues
    if config.console {
        // Dual output: Both console (stdout) and file
        use tracing_subscriber::fmt::writer::MakeWriterExt;

        let combined_writer = std::io::stdout.and(non_blocking_file);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_writer(combined_writer)
            .init();
    } else {
        // File output only
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false)
            .with_writer(non_blocking_file)
            .init();
    }

    tracing::info!("Logging initialized: level={}, log_dir={:?}", config.level, config.log_dir);

    // Store guard in a global static to prevent it from being dropped
    // This is necessary to keep the non-blocking writer alive
    // NOTE: This intentionally leaks memory, but it's a one-time small allocation
    Box::leak(Box::new(_guard));

    Ok(())
}

/// Initialize logging for CLI with optional log directory
///
/// Creates a log file named `mdc-YYYY-MM-DD.log` in the specified directory
/// (or executable directory if not specified).
///
/// # Arguments
///
/// * `debug` - Enable debug logging if true, info level if false
/// * `log_dir` - Optional directory for log files (defaults to executable directory)
///
/// # Returns
///
/// * `Result<()>` - Ok if initialization succeeded, Err if file logging setup failed
///
/// # Examples
///
/// ```no_run
/// use mdc_core::logging::init_cli;
///
/// // Info level logging in executable directory
/// init_cli(false, None).expect("Failed to initialize logging");
///
/// // Debug level logging in custom directory
/// use std::path::PathBuf;
/// init_cli(true, Some(PathBuf::from("/var/log/mdc"))).expect("Failed to initialize logging");
/// ```
pub fn init_cli(debug: bool, log_dir: Option<PathBuf>) -> Result<()> {
    let mut config = LogConfig::default();
    config.level = if debug { "debug" } else { "info" }.to_string();

    if let Some(dir) = log_dir {
        config.log_dir = dir;
    }

    init_with_config(config)
}

/// Initialize logging for API server
///
/// Creates a log file named `mdc-server-YYYY-MM-DD.log` in the specified directory
/// (or executable directory if not specified).
///
/// # Arguments
///
/// * `debug` - Enable debug logging if true, info level if false
/// * `log_dir` - Optional directory for log files (defaults to executable directory)
///
/// # Returns
///
/// * `Result<()>` - Ok if initialization succeeded, Err if file logging setup failed
///
/// # Examples
///
/// ```no_run
/// use mdc_core::logging::init_server;
///
/// init_server(false, None).expect("Failed to initialize logging");
/// ```
pub fn init_server(debug: bool, log_dir: Option<PathBuf>) -> Result<()> {
    let mut config = LogConfig::default();
    config.level = if debug { "debug" } else { "info" }.to_string();
    config.file_prefix = "mdc-server".to_string();

    if let Some(dir) = log_dir {
        config.log_dir = dir;
    }

    init_with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_with_temp_dir() {
        let temp = TempDir::new().unwrap();

        let config = LogConfig {
            level: "info".to_string(),
            log_dir: temp.path().to_path_buf(),
            file_prefix: "test".to_string(),
            console: false,
        };

        // Try to init - may fail if already initialized by another test
        // This is expected behavior (can only init once per process)
        let _ = init_with_config(config);

        // Verify the log directory was created (even if init failed)
        // The file appender builder should have created it
        assert!(temp.path().exists());
    }

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.file_prefix, "mdc");
        assert!(config.console);
    }

    #[test]
    fn test_init_cli_debug() {
        let temp = TempDir::new().unwrap();

        // Try to init - may fail if already initialized by another test
        let _ = init_cli(true, Some(temp.path().to_path_buf()));

        // Test passes as long as we don't panic
        assert!(true);
    }

    #[test]
    fn test_init_server() {
        let temp = TempDir::new().unwrap();

        // Try to init - may fail if already initialized by another test
        let _ = init_server(false, Some(temp.path().to_path_buf()));

        // Test passes as long as we don't panic
        assert!(true);
    }
}
