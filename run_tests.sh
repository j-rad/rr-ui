#!/bin/bash
# Quick Test & Benchmark Runner for RR-UI
# Usage: ./run_tests.sh [test|bench|all|ci]

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   RR-UI Test & Benchmark Suite        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Function to run Rust tests
run_tests() {
    echo -e "${GREEN}► Running Rust Tests...${NC}"
    export RUST_BACKTRACE=1
    cargo test --features server --verbose
    echo ""
}

# Function to run benchmarks
run_benchmarks() {
    echo -e "${GREEN}► Running Benchmarks...${NC}"
    cargo bench --features server
    echo ""
}

# Function to run clippy
run_clippy() {
    echo -e "${GREEN}► Running Clippy (Linter)...${NC}"
    cargo clippy --all-targets --all-features -- -D warnings
    echo ""
}

# Function to check formatting
check_format() {
    echo -e "${GREEN}► Checking Code Formatting...${NC}"
    cargo fmt -- --check
    echo ""
}

# Function to run frontend checks
run_frontend() {
    echo -e "${GREEN}► Running Frontend Checks...${NC}"
    cd web
    echo "  - Type checking..."
    pnpm run check
    echo "  - Building..."
    pnpm run build
    cd ..
    echo ""
}

# Function to run full CI suite
run_ci() {
    echo -e "${YELLOW}Running Full CI Suite...${NC}"
    echo ""
    check_format
    run_clippy
    run_tests
    run_benchmarks
    run_frontend
    echo -e "${GREEN}✓ All CI checks passed!${NC}"
}

# Parse command line argument
case "${1:-all}" in
    test)
        run_tests
        ;;
    bench)
        run_benchmarks
        ;;
    clippy)
        run_clippy
        ;;
    format)
        check_format
        ;;
    frontend)
        run_frontend
        ;;
    ci)
        run_ci
        ;;
    all)
        run_tests
        run_benchmarks
        ;;
    *)
        echo "Usage: $0 [test|bench|clippy|format|frontend|ci|all]"
        echo ""
        echo "Commands:"
        echo "  test      - Run Rust unit and integration tests"
        echo "  bench     - Run performance benchmarks"
        echo "  clippy    - Run Rust linter"
        echo "  format    - Check code formatting"
        echo "  frontend  - Run frontend checks and build"
        echo "  ci        - Run full CI suite (all checks)"
        echo "  all       - Run tests and benchmarks (default)"
        exit 1
        ;;
esac

echo -e "${GREEN}✓ Done!${NC}"
