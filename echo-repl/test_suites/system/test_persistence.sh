#!/bin/bash

echo "Testing object persistence between REPL sessions..."
echo

# First session - create an object
echo "=== First REPL Session: Creating object ==="
echo '.eval
object persistent_test
  property message = "I should survive REPL restart"
  property created_at = "Session 1"
  property value = 42
endobject

persistent_test.message
.
.dump
.quit' | cargo run --quiet --bin echo-repl -- 2>&1 | grep -E "(Created|message|objects|persistent_test)"

echo
echo "=== Restarting REPL... ==="
echo

# Second session - check if object exists
echo "=== Second REPL Session: Checking object ==="
echo '.scope
persistent_test.message
persistent_test.value
.dump
.quit' | cargo run --quiet --bin echo-repl -- 2>&1 | grep -E "(persistent_test|message|value|objects|Error)"

echo
echo "Test complete!"