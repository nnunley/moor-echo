#!/bin/bash

# Echo Language Test Runner
# Run Echo test files from the command line

set -e

ECHO_REPL="cargo run --quiet --bin echo-repl --"
TEST_DIR="tests"
TEST_DB="./test-db-$$"
TEMP_FILE="./echo_test_temp_$$.echo"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to strip comments and prepare test file
prepare_test_file() {
    local input_file="$1"
    local output_file="$2"
    
    # Remove comment lines and empty lines
    grep -v '^\s*//' "$input_file" | grep -v '^$' > "$output_file" || true
}

# Function to run a single test file
run_test() {
    local test_file="$1"
    local test_name=$(basename "$test_file" .echo)
    
    echo -e "${YELLOW}Running test: ${test_name}${NC}"
    
    # Prepare the test file (remove comments)
    prepare_test_file "$test_file" "$TEMP_FILE"
    
    # Run the test and capture output
    output=$($ECHO_REPL < "$TEMP_FILE" 2>&1)
    exit_code=$?
    
    # Check for success
    if [[ $exit_code -eq 0 && $output == *"✅ All tests passed!"* ]]; then
        echo -e "${GREEN}✓ ${test_name}: PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ ${test_name}: FAILED${NC}"
        echo "Output:"
        echo "$output"
        return 1
    fi
}

# Function to run a single inline test file
run_inline_test() {
    local test_file="$1"
    local test_name=$(basename "$test_file" .echo)
    
    echo -e "${YELLOW}Running test: ${test_name}${NC}"
    
    # For inline test files like mini_test.echo, run directly
    output=$($ECHO_REPL < "$test_file" 2>&1)
    exit_code=$?
    
    # Check for success patterns
    if [[ $exit_code -eq 0 && ($output == *"✅ All tests passed!"* || $output == *"Failed: 0"*) ]]; then
        echo -e "${GREEN}✓ ${test_name}: PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ ${test_name}: FAILED${NC}"
        echo "Output:"
        echo "$output"
        return 1
    fi
}

# Main test runner
main() {
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    
    echo "Echo Language Test Runner"
    echo "========================="
    echo
    
    # Check if specific test files were provided
    if [ $# -gt 0 ]; then
        # Run specific test files
        for test_file in "$@"; do
            if [ -f "$test_file" ]; then
                total_tests=$((total_tests + 1))
                if run_inline_test "$test_file"; then
                    passed_tests=$((passed_tests + 1))
                else
                    failed_tests=$((failed_tests + 1))
                fi
                echo
            else
                echo -e "${RED}Error: Test file not found: $test_file${NC}"
            fi
        done
    else
        # Run all test files
        
        # First run simple inline tests
        for test_file in mini_test.echo echo_test.echo; do
            if [ -f "$test_file" ]; then
                total_tests=$((total_tests + 1))
                if run_inline_test "$test_file"; then
                    passed_tests=$((passed_tests + 1))
                else
                    failed_tests=$((failed_tests + 1))
                fi
                echo
            fi
        done
        
        # Then run test suite files (if they exist and work)
        if [ -d "$TEST_DIR" ]; then
            for test_file in "$TEST_DIR"/*.echo; do
                if [ -f "$test_file" ]; then
                    total_tests=$((total_tests + 1))
                    if run_test "$test_file"; then
                        passed_tests=$((passed_tests + 1))
                    else
                        failed_tests=$((failed_tests + 1))
                    fi
                    echo
                fi
            done
        fi
    fi
    
    # Clean up
    rm -f "$TEMP_FILE"
    
    # Summary
    echo "========================="
    echo "Test Summary:"
    echo "  Total:  $total_tests"
    echo -e "  ${GREEN}Passed: $passed_tests${NC}"
    echo -e "  ${RED}Failed: $failed_tests${NC}"
    echo
    
    if [ $failed_tests -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed!${NC}"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"