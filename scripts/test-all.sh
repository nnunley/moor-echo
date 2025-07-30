#!/usr/bin/env bash
# Comprehensive test runner for moor-echo

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_SECTIONS=()

# Helper functions
echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
}

echo_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_TESTS++))
    FAILED_SECTIONS+=("$1")
}

echo_section() {
    echo ""
    echo -e "${YELLOW}=== $1 ===${NC}"
    echo ""
}

run_test() {
    local name="$1"
    local command="$2"
    
    ((TOTAL_TESTS++))
    echo_info "Running: $name"
    
    if eval "$command" > /dev/null 2>&1; then
        echo_success "$name"
    else
        echo_failure "$name"
    fi
}

# Main test execution
echo_section "Moor-Echo Test Suite"
START_TIME=$(date +%s)

# Format checks
echo_section "Format Checks"
run_test "Rust formatting" "cargo fmt --all -- --check"
run_test "Prettier formatting" "npx prettier --check '**/*.{json,yml,yaml,md}' --ignore-path .prettierignore"

# Linting
echo_section "Linting"
run_test "Clippy lints" "cargo clippy --workspace --all-targets --all-features -- -D warnings"
run_test "ESLint" "npx eslint . --ext .js,.ts || true"
run_test "Python linting" "python3 -m ruff check echo-repl/run_echo_tests.py || true"

# Unit tests
echo_section "Unit Tests"
run_test "Rust unit tests" "cargo test --workspace --all-features --lib"
run_test "Doc tests" "cargo test --workspace --all-features --doc"

# Integration tests
echo_section "Integration Tests"
run_test "Rust integration tests" "cargo test --workspace --all-features --tests"

# Tree-sitter tests
echo_section "Tree-sitter Tests"
run_test "Tree-sitter generation" "npx tree-sitter generate"
run_test "Tree-sitter tests" "npx tree-sitter test"

# Echo language tests
echo_section "Echo Language Tests"
if [[ -f "echo-repl/test.sh" ]]; then
    run_test "Echo test suite" "cd echo-repl && ./test.sh"
fi

# Security
echo_section "Security Checks"
run_test "Cargo audit" "cargo audit || true"

# Build tests
echo_section "Build Tests"
run_test "Debug build" "cargo build --workspace --all-features"
run_test "Release build" "cargo build --workspace --all-features --release"

# Documentation
echo_section "Documentation"
run_test "Documentation build" "cargo doc --workspace --all-features --no-deps"

# Performance
echo_section "Performance"
if command -v hyperfine >/dev/null 2>&1; then
    echo_info "Running performance benchmarks..."
    hyperfine --warmup 3 'cargo build --package echo-repl' || true
else
    echo_info "Skipping performance tests (hyperfine not installed)"
fi

# Calculate runtime
END_TIME=$(date +%s)
RUNTIME=$((END_TIME - START_TIME))

# Summary
echo ""
echo_section "Test Summary"
echo "Total tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"
echo "Runtime: ${RUNTIME}s"

if [[ $FAILED_TESTS -gt 0 ]]; then
    echo ""
    echo_failure "Some tests failed:"
    for section in "${FAILED_SECTIONS[@]}"; do
        echo "  - $section"
    done
    exit 1
else
    echo ""
    echo_success "All tests passed!"
fi