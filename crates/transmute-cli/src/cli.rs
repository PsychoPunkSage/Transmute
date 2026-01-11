// crates/transmute-cli/src/cli.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Transmute - Privacy-focused media converter with GPU acceleration
#[derive(Parser)]
#[command(name = "transmute")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Disable progress bars
    #[arg(long, global = true)]
    pub no_progress: bool,

    /// Number of parallel jobs (0 = auto-detect)
    #[arg(short, long, global = true, default_value = "0")]
    pub jobs: usize,

    /// Disable GPU acceleration
    #[arg(long, global = true)]
    pub no_gpu: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert image format
    Convert {
        /// Input file path
        input: PathBuf,

        /// Target format (png, jpg, webp, pdf, etc.)
        #[arg(short = 'f', long)]
        format: String,

        /// Output path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compress/optimize image
    Compress {
        /// Input file path
        input: PathBuf,

        /// Target format (optional, keeps original if not specified)
        #[arg(short = 'f', long)]
        format: Option<String>,

        /// Quality (1-100 for percentage, or preset: low/medium/high/maximum)
        #[arg(short, long, default_value = "high")]
        quality: String,

        /// Output path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Enhance/upscale image (requires models)
    Enhance {
        /// Input file path
        input: PathBuf,

        /// Scale factor (2 or 4)
        #[arg(short, long, default_value = "2")]
        scale: u32,

        /// Output path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Batch convert multiple files
    Batch {
        /// File pattern (e.g., *.png, ./photos/*.jpg)
        pattern: String,

        /// Target format
        #[arg(short = 'f', long)]
        format: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Execute natural language command
    Natural {
        /// Natural language command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Reset to defaults
    Reset,

    /// Show config file path
    Path,
}
