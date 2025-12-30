use anyhow::{anyhow, Result};
use configparser::ini::Ini;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Configuration manager for Movie Data Capture
///
/// Reads configuration from INI files, matching Python's config.py behavior
pub struct Config {
    ini: Ini,
    ini_path: PathBuf,
}

impl Config {
    /// Load configuration from file
    ///
    /// Searches for config.ini in the following locations (in order):
    /// 1. Provided path
    /// 2. Current directory/config.ini
    /// 3. ~/mdc.ini
    /// 4. ~/.mdc.ini
    /// 5. ~/.mdc/config.ini
    /// 6. ~/.config/mdc/config.ini
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let search_paths = Self::config_search_paths(path);

        for candidate in &search_paths {
            debug!("Checking for config at: {}", candidate.display());
            if candidate.is_file() {
                info!("Loading config from: {}", candidate.display());
                let mut ini = Ini::new();
                ini.load(candidate).map_err(|e| {
                    anyhow!("Failed to load config from {}: {}", candidate.display(), e)
                })?;

                return Ok(Self {
                    ini,
                    ini_path: candidate.clone(),
                });
            }
        }

        Err(anyhow!(
            "Config file not found in any of the following locations:\n{}",
            search_paths
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }

    /// Get the path where the config was loaded from
    pub fn config_path(&self) -> &Path {
        &self.ini_path
    }

    /// Get config search paths
    fn config_search_paths(custom_path: Option<&Path>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Add custom path if provided
        if let Some(p) = custom_path {
            paths.push(p.to_path_buf());
        }

        // Current directory
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("config.ini"));
        }

        // Home directory variations
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("mdc.ini"));
            paths.push(home.join(".mdc.ini"));
            paths.push(home.join(".mdc").join("config.ini"));
            paths.push(home.join(".config").join("mdc").join("config.ini"));
        }

        paths
    }

    /// Get a string value from config
    fn get_str(&self, section: &str, key: &str) -> Result<String> {
        self.ini
            .get(section, key)
            .ok_or_else(|| anyhow!("[{}] {} not found in config", section, key))
    }

    /// Get an integer value from config
    fn get_int(&self, section: &str, key: &str) -> Result<i32> {
        let value = self
            .ini
            .getint(section, key)
            .map_err(|e| anyhow!("[{}] {} error: {}", section, key, e))?
            .ok_or_else(|| anyhow!("[{}] {} not found in config", section, key))?;
        Ok(value as i32)
    }

    /// Get a boolean value from config
    fn get_bool(&self, section: &str, key: &str) -> Result<bool> {
        self.ini
            .getbool(section, key)
            .map_err(|e| anyhow!("[{}] {} error: {}", section, key, e))?
            .ok_or_else(|| anyhow!("[{}] {} not found in config", section, key))
    }

    /// Get a boolean value with a default
    fn get_bool_or(&self, section: &str, key: &str, default: bool) -> bool {
        self.get_bool(section, key).unwrap_or(default)
    }

    // === Common configuration accessors ===

    /// Get main processing mode (1=Scraping, 2=Organizing, 3=Analysis)
    pub fn main_mode(&self) -> Result<i32> {
        self.get_int("common", "main_mode")
    }

    /// Get source folder path
    pub fn source_folder(&self) -> Result<PathBuf> {
        let path = self.get_str("common", "source_folder")?;
        Ok(PathBuf::from(path.replace("\\\\", "/").replace("\\", "/")))
    }

    /// Get failed output folder path
    pub fn failed_folder(&self) -> Result<PathBuf> {
        let path = self.get_str("common", "failed_output_folder")?;
        Ok(PathBuf::from(path.replace("\\\\", "/").replace("\\", "/")))
    }

    /// Get success output folder path
    pub fn success_folder(&self) -> Result<PathBuf> {
        let path = self.get_str("common", "success_output_folder")?;
        Ok(PathBuf::from(path.replace("\\\\", "/").replace("\\", "/")))
    }

    /// Get link mode (0=move, 1=soft link, 2=hard link)
    pub fn link_mode(&self) -> Result<i32> {
        self.get_int("common", "link_mode")
    }

    /// Should scan hardlinks
    pub fn scan_hardlink(&self) -> bool {
        self.get_bool_or("common", "scan_hardlink", false)
    }

    /// Should move failed files
    pub fn failed_move(&self) -> bool {
        self.get_bool_or("common", "failed_move", true)
    }

    /// Auto exit after completion
    pub fn auto_exit(&self) -> bool {
        self.get_bool_or("common", "auto_exit", false)
    }

    /// Enable debug mode
    pub fn debug(&self) -> bool {
        self.get_bool_or("debug_mode", "switch", false)
    }

    /// Get NFO skip days
    pub fn nfo_skip_days(&self) -> i32 {
        self.get_int("common", "nfo_skip_days").unwrap_or(0)
    }

    /// Get scraper source websites (comma-separated)
    pub fn sources(&self) -> Result<String> {
        self.get_str("priority", "website")
    }

    /// Get media file extensions (comma-separated)
    pub fn media_type(&self) -> String {
        self.get_str("media", "media_type").unwrap_or_else(|_| {
            ".mp4,.avi,.rmvb,.wmv,.mov,.mkv,.flv,.ts,.webm,.iso,.mpg,.m4v".to_string()
        })
    }

    /// Get subtitle file extensions (comma-separated)
    pub fn sub_type(&self) -> String {
        self.get_str("media", "sub_type").unwrap_or_else(|_| {
            ".smi,.srt,.idx,.sub,.sup,.psb,.ssa,.ass,.usf,.xss,.ssf,.rt,.lrc,.sbv,.vtt,.ttml"
                .to_string()
        })
    }

    /// Get cookies configuration for scrapers
    ///
    /// Returns a HashMap where:
    /// - Key: domain (e.g., "javdb.com")
    /// - Value: HashMap of cookie name -> cookie value
    ///
    /// Example config.ini:
    /// ```ini
    /// [cookies]
    /// javdb.com = _jdb_session=abc123,over18=1
    /// javbus.com = cf_clearance=xyz789
    /// ```
    pub fn cookies(&self) -> HashMap<String, HashMap<String, String>> {
        let mut result = HashMap::new();

        // Get all keys in [cookies] section
        if let Some(cookies_section) = self.ini.get_map_ref().get("cookies") {
            for (domain, cookie_str_opt) in cookies_section {
                // Skip if no value
                let cookie_str = match cookie_str_opt {
                    Some(s) => s,
                    None => continue,
                };

                let mut domain_cookies = HashMap::new();

                // Parse cookie_str: "name1=value1,name2=value2"
                for cookie_pair in cookie_str.split(',') {
                    let parts: Vec<&str> = cookie_pair.trim().splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let name = parts[0].trim().to_string();
                        let value = parts[1].trim().to_string();
                        domain_cookies.insert(name, value);
                    }
                }

                if !domain_cookies.is_empty() {
                    result.insert(domain.clone(), domain_cookies);
                }
            }
        }

        result
    }
}

// Helper function to get home directory (using dirs crate in production)
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path) -> PathBuf {
        let config_path = dir.join("config.ini");
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"[common]
main_mode = 1
source_folder = /test/source
failed_output_folder = /test/failed
success_output_folder = /test/success
link_mode = 0
scan_hardlink = false
failed_move = true
auto_exit = false
nfo_skip_days = 7

[debug_mode]
switch = false

[priority]
website = tmdb,imdb

[media]
media_type = .mp4,.mkv
sub_type = .srt,.ass
"#
        )
        .unwrap();
        config_path
    }

    #[test]
    fn test_config_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = Config::load(Some(&config_path)).unwrap();
        assert_eq!(config.config_path(), config_path);
    }

    #[test]
    fn test_config_main_mode() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(config.main_mode().unwrap(), 1);
    }

    #[test]
    fn test_config_folders() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(
            config.source_folder().unwrap(),
            PathBuf::from("/test/source")
        );
        assert_eq!(
            config.failed_folder().unwrap(),
            PathBuf::from("/test/failed")
        );
        assert_eq!(
            config.success_folder().unwrap(),
            PathBuf::from("/test/success")
        );
    }

    #[test]
    fn test_config_link_mode() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(config.link_mode().unwrap(), 0);
    }

    #[test]
    fn test_config_booleans() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert!(!config.scan_hardlink());
        assert!(config.failed_move());
        assert!(!config.auto_exit());
        assert!(!config.debug());
    }

    #[test]
    fn test_config_nfo_skip_days() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(config.nfo_skip_days(), 7);
    }

    #[test]
    fn test_config_sources() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(config.sources().unwrap(), "tmdb,imdb");
    }

    #[test]
    fn test_config_media_types() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let config = Config::load(Some(&config_path)).unwrap();

        assert_eq!(config.media_type(), ".mp4,.mkv");
        assert_eq!(config.sub_type(), ".srt,.ass");
    }

    #[test]
    fn test_config_not_found() {
        let result = Config::load(Some(Path::new("/nonexistent/config.ini")));
        assert!(result.is_err());
    }
}
