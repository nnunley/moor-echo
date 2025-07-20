#!/bin/bash

echo "Testing object persistence with #0 binding..."
echo

# First session - create and bind object to #0
echo "=== First REPL Session: Creating and binding object ==="
cat << 'EOF' | cargo run --quiet --bin echo-repl -- 2>&1 | tail -20
.eval
object persistent_test
  property message = "I should survive REPL restart"
  property created_at = "Session 1"
  property value = 42
endobject

// Bind to #0
#0.persistent_test = persistent_test
#0.persistent_test.message
.
.scope
.quit
EOF

echo
echo "=== Restarting REPL... ==="
echo

# Second session - access via #0
echo "=== Second REPL Session: Checking object via #0 ==="
cat << 'EOF' | cargo run --quiet --bin echo-repl -- 2>&1 | tail -20
.scope
#0.persistent_test
#0.persistent_test.message
#0.persistent_test.value
.quit
EOF

echo
echo "Test complete!"