use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress reporter for CLI operations
pub struct ProgressReporter {
    multi: MultiProgress,
    show_progress: bool,
}

impl ProgressReporter {
    pub fn new(show_progress: bool) -> Self {
        Self {
            multi: MultiProgress::new(),
            show_progress,
        }
    }

    /// Create progress bar for single operation
    pub fn create_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.show_progress {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Some(pb)
    }

    /// Create spinner for indeterminate operations
    pub fn create_spinner(&self, message: &str) -> Option<ProgressBar> {
        if !self.show_progress {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        Some(pb)
    }

    /// Finish progress bar with success message
    pub fn finish_bar(pb: &Option<ProgressBar>, message: &str) {
        if let Some(pb) = pb {
            pb.finish_with_message(format!("{} {}", style("✓").green(), message));
        }
    }

    /// Finish progress bar with error message
    pub fn finish_bar_error(pb: &Option<ProgressBar>, message: &str) {
        if let Some(pb) = pb {
            pb.finish_with_message(format!("{} {}", style("✗").red(), message));
        }
    }
}
