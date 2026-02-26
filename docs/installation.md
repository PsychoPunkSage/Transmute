# Installation

## Platform Support

- **Linux**: Full CLI and GUI support (primary development platform)
- **macOS**: Full CLI and GUI support
- **Windows**: Full CLI and GUI support
- **Android**: JNI bridge — see [Android](android.md)

## Build Requirements

<details>
<summary><b>Linux</b></summary>

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# For GUI support, install GTK3
sudo apt install libgtk-3-dev

# For GPU acceleration (optional)
# Vulkan is recommended on Linux
sudo apt install libvulkan-dev vulkan-tools
```

</details>

<details>
<summary><b>macOS</b></summary>

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# GPU acceleration uses Metal (built into macOS)
```

</details>

<details>
<summary><b>Windows</b></summary>

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ support
2. Install Rust from [rustup.rs](https://rustup.rs/)
3. GPU acceleration uses DirectX 12 (Windows 10+)

</details>

## Install from Source

```bash
# Clone the repository
git clone https://github.com/PsychoPunkSage/transmute.git
cd transmute

# Build and install CLI (transmute binary)
cargo install --path crates/transmute-cli

# Build and install GUI (transmute-gui binary)
cargo install --path crates/transmute-gui

# Or build both in release mode
cargo build --release
```

Binaries will be available at:

- CLI: `target/release/transmute`
- GUI: `target/release/transmute-gui`
