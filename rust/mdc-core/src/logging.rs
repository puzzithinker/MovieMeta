use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging for Movie Data Capture
///
/// Sets up tracing with sensible defaults:
/// - Format: [timestamp] [level] [target] message
/// - Default level: INFO
/// - Can be overridden with RUST_LOG environment variable
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

/// Initialize logging with a specific level
///
/// # Arguments
///
/// * `level` - Log level: "trace", "debug", "info", "warn", or "error"
pub fn init_with_level(level: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();
}

/// Initialize debug logging (includes debug and trace messages)
pub fn init_debug() {
    init_with_level("debug")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging() {
        // Just ensure it doesn't panic
        // Note: Can only init once per test process, so this is a smoke test
        let _ = tracing_subscriber::registry()
            .with(EnvFilter::new("info"))
            .with(fmt::layer())
            .try_init();
    }
}
