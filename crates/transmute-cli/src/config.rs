use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default output directory
    #[serde(default = "default_output_dir")]
    pub default_output_dir: PathBuf,

    /// Default quality for compression
    #[serde(default = "default_quality")]
    pub default_quality: String,

    /// Enable GPU acceleration
    #[serde(default = "default_gpu")]
    pub use_gpu: bool,

    /// Number of parallel jobs for batch operations
    #[serde(default = "default_jobs")]
    pub parallel_jobs: usize,

    /// Show progress bars
    #[serde(default = "default_progress")]
    pub show_progress: bool,

    /// Colored output
    #[serde(default = "default_color")]
    pub colored_output: bool,
}

fn default_output_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|d| d.home_dir().join("Downloads").join("transmute").into())
        .unwrap_or_else(|| PathBuf::from("./output"))
}

fn default_quality() -> String {
    "high".to_string()
}

fn default_gpu() -> bool {
    true
}

fn default_jobs() -> usize {
    num_cpus::get()
}

fn default_progress() -> bool {
    true
}

fn default_color() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_output_dir: default_output_dir(),
            default_quality: default_quality(),
            use_gpu: default_gpu(),
            parallel_jobs: default_jobs(),
            show_progress: default_progress(),
            colored_output: default_color(),
        }
    }
}

impl Config {
    /// Get config file path (XDG-compliant)
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("", "", "transmute")
            .context("Failed to determine config directory")?
            .config_dir()
            .to_path_buf();

        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    /// Load config from file, or create default
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

            tracing::debug!("Loaded config from {:?}", config_path);
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            tracing::info!("Created default config at {:?}", config_path);
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        tracing::debug!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Reset to defaults
    pub fn reset() -> Result<()> {
        let config = Self::default();
        config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.use_gpu);
        assert!(config.show_progress);
        assert_eq!(config.default_quality, "high");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();

        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.default_quality, parsed.default_quality);
    }
}
