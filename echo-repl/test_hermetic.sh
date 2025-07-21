#!/bin/bash

echo "Testing hermetic databases with --db flag..."
echo

# Test 1: Create a test in temporary database
TEST_DB1="/tmp/echo-test-db-$$"
echo "=== Test 1: Creating object in $TEST_DB1 ==="
echo 'let test_var = "Test 1"
#0.test_obj = test_var
.dump
.quit' | cargo run --quiet --bin echo-repl -- --db "$TEST_DB1" 2>&1 | grep -E "(Database:|test_var|test_obj)"

echo
echo "=== Test 2: Verify isolation - different database ==="
TEST_DB2="/tmp/echo-test-db2-$$"
echo '.dump
.quit' | cargo run --quiet --bin echo-repl -- --db "$TEST_DB2" 2>&1 | grep -E "(Database:|test_var|test_obj|objects)"

echo
echo "=== Test 3: Verify persistence - same database ==="
echo '.dump
.quit' | cargo run --quiet --bin echo-repl -- --db "$TEST_DB1" 2>&1 | grep -E "(test_var|test_obj)"

echo
echo "=== Test 4: Default database still works ==="
echo '.scope
.quit' | cargo run --quiet --bin echo-repl -- 2>&1 | grep -E "(Database:|calculator|hello|test)"

# Cleanup
echo
echo "Cleaning up test databases..."
rm -rf "$TEST_DB1" "$TEST_DB2"
echo "Done!"