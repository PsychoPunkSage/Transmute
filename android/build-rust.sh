#!/usr/bin/env bash
# build-rust.sh — Cross-compile transmute-jni for all Android ABIs and copy
# the produced .so files into the correct jniLibs directories.
#
# Usage:
#   ./android/build-rust.sh
#
# Prerequisites:
#   1. Android NDK installed.  Point NDK_HOME at it:
#        export NDK_HOME=$HOME/Android/Sdk/ndk/27.2.12479018
#      (or set it below as a fallback)
#   2. Rust Android targets installed:
#        rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#   3. .cargo/config.toml already maps the targets to the NDK clang wrappers.

set -euo pipefail

# ── NDK location ────────────────────────────────────────────────────────────
: "${NDK_HOME:=/opt/android-ndk}"
TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/windows-x86_64"

if [[ ! -d "$TOOLCHAIN" ]]; then
    echo "ERROR: NDK toolchain not found at $TOOLCHAIN"
    echo "  Set NDK_HOME to your NDK installation, e.g.:"
    echo "  export NDK_HOME=\$HOME/Android/Sdk/ndk/27.2.12479018"
    exit 1
fi

# ── Output directory (relative to workspace root) ───────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "$SCRIPT_DIR")"
JNILIBS_DIR="$SCRIPT_DIR/app/src/main/jniLibs"

# ── Targets to build ────────────────────────────────────────────────────────
declare -A ABI_MAP=(
    ["aarch64-linux-android"]="arm64-v8a"
    ["armv7-linux-androideabi"]="armeabi-v7a"
    ["x86_64-linux-android"]="x86_64"
)

# ── Build each target ───────────────────────────────────────────────────────
for TARGET in "${!ABI_MAP[@]}"; do
    ABI="${ABI_MAP[$TARGET]}"
    echo ""
    echo "══════════════════════════════════════════════"
    echo "  Building: $TARGET  →  $ABI"
    echo "══════════════════════════════════════════════"

    # C compiler for this target (needed by mozjpeg and webp C FFI builds)
    case "$TARGET" in
        aarch64-linux-android)
            CC="$TOOLCHAIN/bin/aarch64-linux-android26-clang"
            ;;
        armv7-linux-androideabi)
            CC="$TOOLCHAIN/bin/armv7a-linux-androideabi26-clang"
            ;;
        x86_64-linux-android)
            CC="$TOOLCHAIN/bin/x86_64-linux-android26-clang"
            ;;
    esac

    AR="$TOOLCHAIN/bin/llvm-ar"

    CC="$CC" AR="$AR" \
    cargo build --release \
        --target "$TARGET" \
        --no-default-features \
        -p transmute-jni \
        --manifest-path "$WORKSPACE_ROOT/Cargo.toml"

    # ── Copy .so into jniLibs/<abi>/ ─────────────────────────────────────
    DEST="$JNILIBS_DIR/$ABI"
    mkdir -p "$DEST"
    cp "$WORKSPACE_ROOT/target/$TARGET/release/libtransmute_jni.so" \
       "$DEST/libtransmute_jni.so"

    echo "  Copied to $DEST/libtransmute_jni.so"
done

echo ""
echo "Done! All ABIs built and copied to $JNILIBS_DIR"
