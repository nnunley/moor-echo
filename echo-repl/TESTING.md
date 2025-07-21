# Echo Language Testing Guide

This document describes how to run tests for the Echo language implementation.

## Quick Start

```bash
# Run all tests (Rust + Echo)
make test

# Run only Echo language tests
make test-echo

# Run only Rust unit tests
make test-rust

# Run a specific test file
make test-file FILE=mini_test.echo
```

## Test Scripts

### Python Test Runner (`run_echo_tests.py`)

The primary test runner with advanced features:

```bash
# Run all tests (creates test database in ./test-db-<timestamp>-<pid>)
./run_echo_tests.py

# Run specific test files
./run_echo_tests.py mini_test.echo test_batch.echo

# Run tests in a directory
./run_echo_tests.py tests/*.echo

# Keep test database for debugging (don't clean up)
./run_echo_tests.py --no-cleanup

# Use a different database prefix
./run_echo_tests.py --db-prefix /tmp/mytest

# Run tests including Rust unit tests
./run_echo_tests.py --rust

# Use temporary system directory for hermetic testing
./run_echo_tests.py --testing
```

Features:
- Creates isolated test databases in current directory
- Optionally runs Rust unit tests with `--rust` flag
- Strips comments from Echo test files (since grammar doesn't support them yet)
- Handles `.load` commands by inlining files
- Provides colored output and test summaries
- Returns appropriate exit codes for CI/CD
- Cleans up test databases by default (use `--no-cleanup` to preserve)
- Supports temporary system directories with `--testing` flag for hermetic testing

### Shell Test Runner (`run_echo_tests.sh`)

A simpler bash-based test runner:

```bash
# Run all tests
./run_echo_tests.sh

# Run specific tests
./run_echo_tests.sh mini_test.echo
```

### Simple Test Runner (`run_tests_simple.sh`)

Basic test runner that checks for success patterns:

```bash
./run_tests_simple.sh
```

## Writing Echo Tests

### Test File Format

Echo test files should follow this pattern:

```echo
// Test setup
let pass = 0
let fail = 0

// Test function
let test = fn {actual, expected, name}
  if actual == expected
    pass = pass + 1
    "✓ " + name
  else  
    fail = fail + 1
    "✗ " + name + " (got " + actual + ", wanted " + expected + ")"
  endif
endfn

// Run tests
"=== Test Suite Name ==="
test(1 + 1, 2, "Addition")
test(foo(), expected, "Function test")

// Summary
"Passed: " + pass
"Failed: " + fail
if fail == 0
  "✅ All tests passed!"
else
  "❌ Some tests failed!"  
endif
```

### Using .eval Mode

For multi-line constructs, use `.eval` mode:

```echo
.eval
// Multi-line test code here
let complex_fn = fn {x}
  let result = x * 2
  result + 10
endfn

test(complex_fn(5), 20, "Complex function")
.
```

## Test Files

### `mini_test.echo`
Basic test suite covering arithmetic, lambdas, and control flow.

### `test_batch.echo`
Batch-mode test file using `.eval` for multi-line support.

### `echo_test.echo`
Comprehensive test using the assertion framework.

### `test_framework.echo`
Reusable test framework with assertion functions (requires `.load` support).

## Known Limitations

1. **No Comment Support**: The current grammar doesn't support comments, so test files with `//` comments need preprocessing.

2. **No .load Command**: The REPL doesn't have a `.load` command yet, so test framework files must be inlined.

3. **Multi-line Constructs**: Complex multi-line constructs should use `.eval` mode.

## Continuous Integration

For CI/CD pipelines, use:

```bash
# Simple CI test
./test.sh

# Or with make
make test
```

The test runners return:
- Exit code 0: All tests passed
- Exit code 1: Some tests failed

## Future Improvements

- [ ] Add comment support to grammar
- [ ] Implement `.load` command in REPL
- [ ] Support test discovery and automatic test file detection
- [ ] Add code coverage reporting for Echo tests
- [ ] Integrate with cargo test harness