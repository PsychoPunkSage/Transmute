use console::{style, Style};
use std::path::Path;
use transmute_common::MediaFormat;

/// Output formatter with colored messages
pub struct OutputFormatter {
    colored: bool,
}

impl OutputFormatter {
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }

    /// Print success message
    pub fn success(&self, message: &str) {
        if self.colored {
            println!("{} {}", style("✓").green().bold(), message);
        } else {
            println!("[SUCCESS] {}", message);
        }
    }

    /// Print error message
    pub fn error(&self, message: &str) {
        if self.colored {
            eprintln!("{} {}", style("✗").red().bold(), message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }

    /// Print warning message
    pub fn warn(&self, message: &str) {
        if self.colored {
            println!("{} {}", style("⚠").yellow().bold(), message);
        } else {
            println!("[WARN] {}", message);
        }
    }

    /// Print info message
    pub fn info(&self, message: &str) {
        if self.colored {
            println!("{} {}", style("ℹ").cyan(), message);
        } else {
            println!("[INFO] {}", message);
        }
    }

    /// Format file path
    pub fn format_path(&self, path: &Path) -> String {
        if self.colored {
            style(path.display()).cyan().to_string()
        } else {
            path.display().to_string()
        }
    }

    /// Format file size
    pub fn format_size(&self, bytes: usize) -> String {
        let size_str = if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        };

        if self.colored {
            style(size_str).yellow().to_string()
        } else {
            size_str
        }
    }

    /// Format compression ratio
    pub fn format_ratio(&self, ratio: f32) -> String {
        let ratio_str = format!("{:.2}×", ratio);

        if self.colored {
            if ratio > 10.0 {
                style(ratio_str).green().bold().to_string()
            } else if ratio > 5.0 {
                style(ratio_str).green().to_string()
            } else {
                style(ratio_str).yellow().to_string()
            }
        } else {
            ratio_str
        }
    }

    /// Format media format
    pub fn format_format(&self, format: MediaFormat) -> String {
        if self.colored {
            style(format.to_string()).magenta().to_string()
        } else {
            format.to_string()
        }
    }

    /// Print conversion result
    pub fn print_conversion(&self, input: &Path, output: &Path, format: MediaFormat) {
        self.success(&format!(
            "Converted {} → {} ({})",
            self.format_path(input),
            self.format_path(output),
            self.format_format(format)
        ));
    }

    /// Print compression result
    pub fn print_compression(
        &self,
        input: &Path,
        output: &Path,
        original_size: usize,
        compressed_size: usize,
        ratio: f32,
    ) {
        self.success(&format!(
            "Compressed {} → {} ({} → {}, {})",
            self.format_path(input),
            self.format_path(output),
            self.format_size(original_size),
            self.format_size(compressed_size),
            self.format_ratio(ratio)
        ));
    }

    /// Print batch summary
    pub fn print_batch_summary(&self, total: usize, success: usize, failed: usize) {
        println!();
        if self.colored {
            println!(
                "{} Total: {}, {} Success: {}, {} Failed: {}",
                style("Summary:").bold(),
                total,
                style("✓").green(),
                success,
                style("✗").red(),
                failed
            );
        } else {
            println!(
                "Summary: Total: {}, Success: {}, Failed: {}",
                total, success, failed
            );
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(true)
    }
}
