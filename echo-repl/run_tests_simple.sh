#!/bin/bash

# Simple Echo test runner - runs tests and checks exit codes

set -e

echo "Running Echo Language Tests"
echo "=========================="

# Function to run a test file
run_test() {
    local test_file="$1"
    local test_name=$(basename "$test_file" .echo)
    
    echo -n "Testing $test_name... "
    
    # Run the test file
    if cargo run --quiet --bin echo-repl -- < "$test_file" > /tmp/echo_test_output.txt 2>&1; then
        # Check if output contains success indicators
        if grep -q "✅ All tests passed!" /tmp/echo_test_output.txt || grep -q "Failed: 0" /tmp/echo_test_output.txt; then
            echo "✅ PASSED"
            return 0
        else
            echo "❌ FAILED"
            echo "Output:"
            cat /tmp/echo_test_output.txt
            return 1
        fi
    else
        echo "❌ ERROR"
        echo "Output:"
        cat /tmp/echo_test_output.txt
        return 1
    fi
}

# Test the files
echo
run_test "test_batch.echo"
run_test "mini_test.echo"

echo
echo "Done!"