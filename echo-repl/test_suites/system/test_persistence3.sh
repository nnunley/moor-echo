#!/bin/bash

echo "Testing object persistence..."
echo

# Check existing objects
echo "=== Checking existing persisted objects ==="
echo '.scope
#0.calculator
#0.test
.quit' | cargo run --quiet --bin echo-repl --

echo
echo "=== Creating new object in this session ==="
echo 'let msg = "Test message"
#0.new_test = msg
#0.new_test
.scope
.quit' | cargo run --quiet --bin echo-repl --

echo
echo "=== Checking if new property persists ==="
echo '.scope
#0.new_test
.quit' | cargo run --quiet --bin echo-repl --