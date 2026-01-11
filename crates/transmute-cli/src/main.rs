use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use transmute_cli::{Cli, Commands, ConfigCommands, Config, OutputFormatter, ProgressReporter};
use transmute_common::MediaFormat;
use transmute_compress::QualitySettings;
use transmute_core::Converter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        "transmute=debug"
    } else {
        "transmute=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .without_time()
        .init();

    // Load config
    let mut config = Config::load()?;

    // Override config with CLI flags
    if cli.no_color {
        config.colored_output = false;
    }
    if cli.no_progress {
        config.show_progress = false;
    }
    if cli.jobs > 0 {
        config.parallel_jobs = cli.jobs;
    }
    if cli.no_gpu {
        config.use_gpu = false;
    }

    // Create formatter and progress reporter
    let formatter = OutputFormatter::new(config.colored_output);
    let progress = ProgressReporter::new(config.show_progress);

    // Execute command
    match cli.command {
        Commands::Convert {
            input,
            format,
            output,
        } => {
            handle_convert(input, format, output, &config, &formatter, &progress)?;
        }

        Commands::Compress {
            input,
            format,
            quality,
            output,
        } => {
            handle_compress(
                input, format, quality, output, &config, &formatter, &progress,
            )?;
        }

        Commands::Enhance {
            input,
            scale,
            output,
        } => {
            handle_enhance(input, scale, output, &config, &formatter, &progress)?;
        }

        Commands::Batch {
            pattern,
            format,
            output,
        } => {
            handle_batch(pattern, format, output, &config, &formatter, &progress).await?;
        }

        Commands::Natural { command } => {
            handle_natural(command, &config, &formatter, &progress)?;
        }

        Commands::Config { action } => {
            handle_config(action, &formatter)?;
        }
    }

    Ok(())
}

fn handle_convert(
    input: PathBuf,
    format_str: String,
    output: Option<PathBuf>,
    config: &Config,
    formatter: &OutputFormatter,
    progress: &ProgressReporter,
) -> Result<()> {
    let format = MediaFormat::from_extension(&format_str)
        .context(format!("Unsupported format: {}", format_str))?;

    let spinner = progress.create_spinner("Converting...");

    let mut converter = Converter::new()?;
    converter.set_gpu_enabled(config.use_gpu);

    let output_path = converter.convert_image(&input, format, output)?;

    ProgressReporter::finish_bar(&spinner, "Done");
    formatter.print_conversion(&input, &output_path, format);

    Ok(())
}

fn handle_compress(
    input: PathBuf,
    format_str: Option<String>,
    quality_str: String,
    output: Option<PathBuf>,
    config: &Config,
    formatter: &OutputFormatter,
    progress: &ProgressReporter,
) -> Result<()> {
    // Parse quality
    let quality = parse_quality(&quality_str)?;

    // Determine format
    let format = if let Some(fmt) = format_str {
        MediaFormat::from_extension(&fmt).context(format!("Unsupported format: {}", fmt))?
    } else {
        MediaFormat::from_path(&input).unwrap_or(MediaFormat::Jpeg)
    };

    let spinner = progress.create_spinner("Compressing...");

    let mut converter = Converter::new()?;
    converter.set_gpu_enabled(config.use_gpu);

    let (output_path, result) = converter.compress_image(&input, format, quality, output)?;

    ProgressReporter::finish_bar(&spinner, "Done");
    formatter.print_compression(
        &input,
        &output_path,
        result.original_size,
        result.compressed_size,
        result.ratio,
    );

    Ok(())
}

fn handle_enhance(
    input: PathBuf,
    scale: u32,
    output: Option<PathBuf>,
    config: &Config,
    formatter: &OutputFormatter,
    progress: &ProgressReporter,
) -> Result<()> {
    if scale != 2 && scale != 4 {
        anyhow::bail!("Scale factor must be 2 or 4");
    }

    formatter.warn("Enhancement feature requires Phase 4 implementation (ONNX models)");
    formatter.info("For now, use basic upscaling or wait for full AI enhancement support");

    Ok(())
}

async fn handle_batch(
    pattern: String,
    format_str: String,
    output: Option<PathBuf>,
    config: &Config,
    formatter: &OutputFormatter,
    progress: &ProgressReporter,
) -> Result<()> {
    use transmute_nlp::PathResolver;

    let format = MediaFormat::from_extension(&format_str)
        .context(format!("Unsupported format: {}", format_str))?;

    // Resolve pattern
    let resolver = PathResolver::new()?;
    let files = resolver.resolve_pattern(&pattern)?;

    if files.is_empty() {
        anyhow::bail!("No files matched pattern: {}", pattern);
    }

    formatter.info(&format!("Found {} files to process", files.len()));

    let pb = progress.create_bar(files.len() as u64, "Processing batch...");

    let mut converter = Converter::new()?;
    converter.set_gpu_enabled(config.use_gpu);

    let results = converter.convert_batch(files, format, output);

    let mut success = 0;
    let mut failed = 0;

    for result in results {
        if let Some(pb) = &pb {
            pb.inc(1);
        }

        match result {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                formatter.error(&format!("Failed: {}", e));
            }
        }
    }

    ProgressReporter::finish_bar(&pb, "Batch complete");
    formatter.print_batch_summary(success + failed, success, failed);

    Ok(())
}

fn handle_natural(
    command_parts: Vec<String>,
    config: &Config,
    formatter: &OutputFormatter,
    progress: &ProgressReporter,
) -> Result<()> {
    let command = command_parts.join(" ");

    formatter.info(&format!("Executing: {}", command));

    let spinner = progress.create_spinner("Processing...");

    let mut converter = Converter::new()?;
    converter.set_gpu_enabled(config.use_gpu);

    let outputs = converter.execute_command(&command)?;

    ProgressReporter::finish_bar(&spinner, "Done");

    for output in outputs {
        formatter.success(&format!("Created: {}", formatter.format_path(&output)));
    }

    Ok(())
}

fn handle_config(action: ConfigCommands, formatter: &OutputFormatter) -> Result<()> {
    match action {
        ConfigCommands::Show => {
            let config = Config::load()?;
            let toml = toml::to_string_pretty(&config)?;
            println!("{}", toml);
        }

        ConfigCommands::Set { key, value } => {
            let mut config = Config::load()?;

            match key.as_str() {
                "default_quality" => config.default_quality = value.clone(),
                "use_gpu" => config.use_gpu = value.parse()?,
                "parallel_jobs" => config.parallel_jobs = value.parse()?,
                "show_progress" => config.show_progress = value.parse()?,
                "colored_output" => config.colored_output = value.parse()?,
                _ => anyhow::bail!("Unknown config key: {}", key),
            }

            config.save()?;
            formatter.success(&format!("Set {} = {}", key, value));
        }

        ConfigCommands::Reset => {
            Config::reset()?;
            formatter.success("Configuration reset to defaults");
        }

        ConfigCommands::Path => {
            let path = Config::config_path()?;
            println!("{}", path.display());
        }
    }

    Ok(())
}

fn parse_quality(quality_str: &str) -> Result<QualitySettings> {
    // Try parsing as percentage first
    if let Ok(percent) = quality_str.trim_end_matches('%').parse::<u8>() {
        return Ok(QualitySettings::Custom(percent.clamp(1, 100)));
    }

    // Parse as preset
    match quality_str.to_lowercase().as_str() {
        "maximum" | "max" => Ok(QualitySettings::Maximum),
        "high" => Ok(QualitySettings::High),
        "medium" | "balanced" => Ok(QualitySettings::Balanced),
        "low" => Ok(QualitySettings::Low),
        _ => anyhow::bail!(
            "Invalid quality: {}. Use 1-100 or low/medium/high/maximum",
            quality_str
        ),
    }
}
