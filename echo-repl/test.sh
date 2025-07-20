#!/bin/bash
# Simple test runner for CI/CD environments

set -e

echo "Running Echo Language Tests"
echo "=========================="

# Run Rust tests
echo "Running Rust unit tests..."
cargo test --all-features

# Run Echo tests if Python is available
if command -v python3 &> /dev/null; then
    echo
    echo "Running Echo language tests..."
    ./run_echo_tests.py
else
    echo "Python3 not found, using shell script..."
    ./run_echo_tests.sh
fi

echo
echo "All tests completed!"