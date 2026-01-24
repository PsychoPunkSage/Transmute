#!/bin/bash

# Transmute Profiling Example Script
# This script demonstrates practical profiling workflows for the Transmute project

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/profiling-results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Transmute Profiling Example${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Check if required tools are installed
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}✗ $1 is not installed${NC}"
        echo -e "${YELLOW}  Install with: $2${NC}"
        return 1
    else
        echo -e "${GREEN}✓ $1 is installed${NC}"
        return 0
    fi
}

echo -e "${BLUE}Checking required tools...${NC}"
check_tool "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_tool "perf" "sudo apt install linux-tools-common linux-tools-generic" || true
check_tool "flamegraph" "cargo install flamegraph" || true

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Step 1: Build with Debug Symbols${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

cd "$PROJECT_ROOT"

# Add the release-with-debug profile if not present
if ! grep -q '\[profile.release-with-debug\]' Cargo.toml; then
    echo -e "${YELLOW}Adding release-with-debug profile to Cargo.toml...${NC}"
    cat >> Cargo.toml << 'EOF'

[profile.release-with-debug]
inherits = "release"
debug = true        # Include debug symbols
strip = false       # Don't strip symbols
EOF
    echo -e "${GREEN}✓ Profile added${NC}"
else
    echo -e "${GREEN}✓ Profile already exists${NC}"
fi

echo ""
echo -e "${YELLOW}Building transmute-cli with profiling symbols...${NC}"
cargo build --profile release-with-debug -p transmute-cli
echo -e "${GREEN}✓ Build complete${NC}"

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Step 2: Running Criterion Benchmarks${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

echo -e "${YELLOW}Running compression benchmarks...${NC}"
echo -e "${YELLOW}This measures CPU vs GPU performance with statistical analysis${NC}"
echo ""

cargo bench -p transmute-compress 2>&1 | tee "$RESULTS_DIR/benchmark_results.txt"

echo ""
echo -e "${GREEN}✓ Benchmark results saved to: $RESULTS_DIR/benchmark_results.txt${NC}"
echo -e "${YELLOW}  Detailed HTML reports: $PROJECT_ROOT/target/criterion/${NC}"

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Step 3: CPU Profiling with Flamegraph${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Check if flamegraph is available
if command -v flamegraph &> /dev/null; then
    echo -e "${YELLOW}Creating test image for profiling...${NC}"

    # Use ImageMagick if available, otherwise create a simple PNG with Rust
    if command -v convert &> /dev/null; then
        convert -size 1920x1080 xc:blue "$RESULTS_DIR/test_1080p.png"
        echo -e "${GREEN}✓ Created test image: $RESULTS_DIR/test_1080p.png${NC}"
    else
        echo -e "${YELLOW}ImageMagick not found, skipping test image creation${NC}"
        echo -e "${YELLOW}You can manually create a test image and place it at:${NC}"
        echo -e "${YELLOW}  $RESULTS_DIR/test_1080p.png${NC}"
    fi

    if [ -f "$RESULTS_DIR/test_1080p.png" ]; then
        echo ""
        echo -e "${YELLOW}Generating flamegraph for image conversion...${NC}"
        echo -e "${YELLOW}This will show where CPU time is spent during PNG→JPEG conversion${NC}"
        echo ""

        cd "$RESULTS_DIR"
        cargo flamegraph --profile release-with-debug -p transmute-cli \
            --output conversion_flamegraph.svg \
            -- convert --input test_1080p.png --output test_1080p.jpg || true

        if [ -f "conversion_flamegraph.svg" ]; then
            echo -e "${GREEN}✓ Flamegraph generated: $RESULTS_DIR/conversion_flamegraph.svg${NC}"
            echo -e "${YELLOW}  Open with: firefox $RESULTS_DIR/conversion_flamegraph.svg${NC}"
        fi
        cd "$PROJECT_ROOT"
    fi
else
    echo -e "${YELLOW}cargo-flamegraph not installed, skipping flamegraph generation${NC}"
    echo -e "${YELLOW}Install with: cargo install flamegraph${NC}"
fi

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Step 4: Memory Profiling with Valgrind${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

if command -v valgrind &> /dev/null && [ -f "$RESULTS_DIR/test_1080p.png" ]; then
    echo -e "${YELLOW}Running Valgrind Massif to analyze heap usage...${NC}"
    echo -e "${YELLOW}This shows memory allocation patterns${NC}"
    echo ""

    valgrind --tool=massif \
        --massif-out-file="$RESULTS_DIR/massif.out" \
        ./target/release-with-debug/transmute-cli \
        convert --input "$RESULTS_DIR/test_1080p.png" --output "$RESULTS_DIR/test_massif.jpg" \
        2>&1 | tee "$RESULTS_DIR/massif_log.txt"

    echo ""
    echo -e "${GREEN}✓ Memory profile saved to: $RESULTS_DIR/massif.out${NC}"
    echo -e "${YELLOW}  View with: ms_print $RESULTS_DIR/massif.out | less${NC}"
    echo ""

    # Generate summary
    echo -e "${YELLOW}Memory usage summary:${NC}"
    ms_print "$RESULTS_DIR/massif.out" | head -50
else
    if ! command -v valgrind &> /dev/null; then
        echo -e "${YELLOW}Valgrind not installed, skipping memory profiling${NC}"
        echo -e "${YELLOW}Install with: sudo apt install valgrind${NC}"
    else
        echo -e "${YELLOW}Test image not found, skipping memory profiling${NC}"
    fi
fi

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Step 5: Advanced perf Analysis${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

if command -v perf &> /dev/null && [ -f "$RESULTS_DIR/test_1080p.png" ]; then
    echo -e "${YELLOW}Recording performance data with perf...${NC}"
    echo -e "${YELLOW}This provides detailed CPU profiling with call graphs${NC}"
    echo ""

    cd "$RESULTS_DIR"
    perf record --call-graph dwarf \
        --output=perf_conversion.data \
        "$PROJECT_ROOT/target/release-with-debug/transmute-cli" \
        convert --input test_1080p.png --output test_perf.jpg \
        2>&1 | tee perf_record_log.txt

    echo ""
    echo -e "${GREEN}✓ Performance data saved to: $RESULTS_DIR/perf_conversion.data${NC}"
    echo ""

    # Generate report
    echo -e "${YELLOW}Top 20 functions by CPU usage:${NC}"
    perf report --stdio --sort=dso,symbol \
        --input=perf_conversion.data 2>/dev/null | head -40 | tee perf_report_summary.txt

    echo ""
    echo -e "${GREEN}✓ Summary saved to: $RESULTS_DIR/perf_report_summary.txt${NC}"
    echo -e "${YELLOW}  Full interactive report: cd $RESULTS_DIR && perf report --input=perf_conversion.data${NC}"

    cd "$PROJECT_ROOT"
else
    if ! command -v perf &> /dev/null; then
        echo -e "${YELLOW}perf not installed, skipping advanced profiling${NC}"
        echo -e "${YELLOW}Install with: sudo apt install linux-tools-generic${NC}"
    else
        echo -e "${YELLOW}Test image not found, skipping perf profiling${NC}"
    fi
fi

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Profiling Complete!${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

echo -e "${GREEN}Results saved to: $RESULTS_DIR/${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review benchmark results to compare CPU vs GPU performance"
echo "  2. Open flamegraph.svg to identify hot functions (widest boxes)"
echo "  3. Check massif.out for memory allocation patterns"
echo "  4. Analyze perf report for detailed CPU profiling"
echo ""
echo -e "${YELLOW}Optimization workflow:${NC}"
echo "  1. Identify bottleneck from profiling data"
echo "  2. Modify code to optimize the bottleneck"
echo "  3. Re-run benchmarks: cargo bench -p transmute-compress"
echo "  4. Verify improvement (look for negative % change)"
echo "  5. Repeat until performance targets are met"
echo ""
echo -e "${BLUE}For more information, see: PROFILING_GUIDE.md${NC}"
echo ""
