use crate::{Error, Result};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Manages output paths with unique naming and directory creation
pub struct PathManager {
    default_output_dir: PathBuf,
}

impl PathManager {
    /// Create new PathManager with default output directory
    pub fn new() -> Result<Self> {
        let home = directories::UserDirs::new().ok_or_else(|| {
            Error::InvalidPath(PathBuf::from("Could not determine home directory"))
        })?;

        let default_output_dir = home.home_dir().join("Downloads").join("transmute");

        Ok(Self { default_output_dir })
    }

    /// Generate unique output path
    /// Format: YYYYMMDD_original-name_uniqueid.ext
    pub fn generate_unique_path(
        &self,
        original: &Path,
        new_extension: &str,
        custom_output: Option<PathBuf>,
    ) -> Result<PathBuf> {
        let output_dir = if let Some(custom) = custom_output {
            if custom.is_dir() {
                custom
            } else {
                return Ok(custom); // User specified exact file path
            }
        } else {
            self.default_output_dir.clone()
        };

        // Ensure output directory exists
        std::fs::create_dir_all(&output_dir)?;

        let stem = original
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::InvalidPath(original.to_path_buf()))?;

        let timestamp = chrono::Local::now().format("%Y%m%d");
        let unique_id = Uuid::new_v4().simple().to_string();
        let unique_id_short = &unique_id[..8]; // First 8 chars of UUID

        let filename = format!(
            "{}_{}_{}.{}",
            timestamp, stem, unique_id_short, new_extension
        );

        Ok(output_dir.join(filename))
    }

    /// Validate input path exists and is readable
    pub fn validate_input(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_path_buf()));
        }

        if !path.is_file() {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }

        Ok(())
    }

    /// Get default output directory
    pub fn default_output_dir(&self) -> &Path {
        &self.default_output_dir
    }
}

impl Default for PathManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PathManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_unique_path_generation() {
        let manager = PathManager::new().unwrap();
        let original = PathBuf::from("/tmp/test.png");

        let path1 = manager
            .generate_unique_path(&original, "jpg", None)
            .unwrap();
        let path2 = manager
            .generate_unique_path(&original, "jpg", None)
            .unwrap();

        // Should generate different paths
        assert_ne!(path1, path2);

        // Should have correct extension
        assert_eq!(path1.extension().unwrap(), "jpg");
    }
}
