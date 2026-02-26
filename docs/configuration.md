# Configuration

## Config File Location

| Platform | Path                                              |
| -------- | ------------------------------------------------- |
| Linux    | `~/.config/transmute/config.toml`                 |
| macOS    | `~/Library/Application Support/transmute/config.toml` |
| Windows  | `%APPDATA%\transmute\config.toml`                 |

## Example Configuration

```toml
# Default quality setting for compression
default_quality = "high"

# Enable GPU acceleration
use_gpu = true

# Number of parallel jobs (0 = auto-detect based on CPU cores)
parallel_jobs = 0

# Show progress bars in CLI
show_progress = true

# Enable colored output
colored_output = true
```

## Managing Config via CLI

```bash
# Show current configuration
transmute config show

# Set a value
transmute config set default_quality high
transmute config set use_gpu true
transmute config set parallel_jobs 4

# Reset all values to defaults
transmute config reset

# Print the config file path
transmute config path
```
