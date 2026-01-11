use std::env;
use std::path::PathBuf;
use transmute_common::{Error, Result};

/// Resolves natural language paths to absolute paths
pub struct PathResolver {
    current_dir: PathBuf,
}

impl PathResolver {
    pub fn new() -> Result<Self> {
        let current_dir = env::current_dir()?;
        Ok(Self { current_dir })
    }

    /// Resolve path with shell expansion and natural language
    pub fn resolve(&self, path_str: &str) -> Result<PathBuf> {
        tracing::debug!("Resolving path: {}", path_str);

        // Step 1: Shell expansion (~, $HOME, etc.)
        let expanded =
            shellexpand::full(path_str).map_err(|_| Error::InvalidPath(PathBuf::from(path_str)))?;

        // Step 2: Natural language shortcuts
        let resolved = self.resolve_natural_language(&expanded);

        // Step 3: Convert to absolute path
        let path = PathBuf::from(resolved.as_ref());
        let absolute = if path.is_absolute() {
            path
        } else {
            self.current_dir.join(path)
        };

        tracing::debug!("Resolved '{}' → {:?}", path_str, absolute);
        Ok(absolute)
    }

    /// Resolve natural language path references
    fn resolve_natural_language<'a>(&self, path: &'a str) -> std::borrow::Cow<'a, str> {
        let lower = path.to_lowercase();

        // Common desktop locations
        if lower.contains("desktop") || lower.contains("my desktop") {
            if let Some(user_dirs) = directories::UserDirs::new() {
                if let Some(desktop) = user_dirs.desktop_dir() {
                    return std::borrow::Cow::Owned(
                        path.replace("desktop", desktop.to_str().unwrap())
                            .replace("my desktop", desktop.to_str().unwrap()),
                    );
                }
            }
        }

        // Downloads folder
        if lower.contains("downloads") || lower.contains("my downloads") {
            if let Some(user_dirs) = directories::UserDirs::new() {
                if let Some(downloads) = user_dirs.download_dir() {
                    return std::borrow::Cow::Owned(
                        path.replace("downloads", downloads.to_str().unwrap())
                            .replace("my downloads", downloads.to_str().unwrap()),
                    );
                }
            }
        }

        // Pictures/Photos
        if lower.contains("pictures") || lower.contains("photos") || lower.contains("my photos") {
            if let Some(user_dirs) = directories::UserDirs::new() {
                if let Some(pictures) = user_dirs.picture_dir() {
                    return std::borrow::Cow::Owned(
                        path.replace("pictures", pictures.to_str().unwrap())
                            .replace("photos", pictures.to_str().unwrap())
                            .replace("my photos", pictures.to_str().unwrap()),
                    );
                }
            }
        }

        // Documents
        if lower.contains("documents") || lower.contains("my documents") {
            if let Some(user_dirs) = directories::UserDirs::new() {
                if let Some(documents) = user_dirs.document_dir() {
                    return std::borrow::Cow::Owned(
                        path.replace("documents", documents.to_str().unwrap())
                            .replace("my documents", documents.to_str().unwrap()),
                    );
                }
            }
        }

        // Current directory shortcuts
        if lower == "here" || lower == "this folder" || lower == "current folder" {
            return std::borrow::Cow::Owned(self.current_dir.to_string_lossy().to_string());
        }

        std::borrow::Cow::Borrowed(path)
    }

    /// Resolve glob pattern for batch operations
    pub fn resolve_pattern(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        use glob::glob;

        let resolved_pattern = self.resolve(pattern)?;
        let pattern_str = resolved_pattern.to_string_lossy();

        tracing::debug!("Glob pattern: {}", pattern_str);

        let mut matches = Vec::new();
        for entry in glob(&pattern_str).map_err(|e| Error::InvalidPath(resolved_pattern.clone()))? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        matches.push(path);
                    }
                }
                Err(e) => tracing::warn!("Glob error: {}", e),
            }
        }

        if matches.is_empty() {
            tracing::warn!("No files matched pattern: {}", pattern);
        } else {
            tracing::debug!("Found {} files matching pattern", matches.len());
        }

        Ok(matches)
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new().expect("Failed to get current directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilde_expansion() {
        let resolver = PathResolver::new().unwrap();
        let resolved = resolver.resolve("~/test.png").unwrap();

        // Should expand to home directory
        assert!(!resolved.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_relative_path() {
        let resolver = PathResolver::new().unwrap();
        let resolved = resolver.resolve("./test.png").unwrap();

        // Should be absolute
        assert!(resolved.is_absolute());
    }

    #[test]
    fn test_natural_language_desktop() {
        let resolver = PathResolver::new().unwrap();
        let resolved = resolver.resolve("desktop/image.png");

        // If desktop dir exists, should resolve to it
        if let Some(user_dirs) = directories::UserDirs::new() {
            if user_dirs.desktop_dir().is_some() {
                assert!(resolved.is_ok());
            }
        }
    }
}
