# Development Notes and Process Guide

Comprehensive development guide covering project setup, testing strategies, recent improvements, and development workflows for the Echo REPL project.

## Table of Contents

- [Project Setup](#project-setup)
- [Development Workflow](#development-workflow)
- [Testing Strategy](#testing-strategy)
- [Recent Improvements](#recent-improvements)
- [Code Quality Standards](#code-quality-standards)
- [Performance Optimization](#performance-optimization)
- [Debugging Tools](#debugging-tools)
- [Release Process](#release-process)

## Project Setup

### Prerequisites

Ensure you have the following installed:
- **Rust** (1.75 or later) - Install via [rustup](https://rustup.rs/)
- **Node.js** (18 or later) and npm
- **Python** (3.8 or later) - For test scripts and tooling
- **Git** - Version control
- **Docker** (optional) - For containerized development

### Quick Start

```bash
# Clone and setup
git clone https://github.com/username/moor-echo.git
cd moor-echo
./scripts/setup.sh

# Build and run
cargo build --features web-ui
./target/debug/echo-repl --web --port 8080
```

### Development Environment

#### Recommended VS Code Extensions
- `rust-analyzer` - Rust language support
- `CodeLLDB` - Debugger for Rust
- `Better TOML` - TOML file support
- `markdownlint` - Markdown linting
- `Playwright Test` - E2E testing support

#### Pre-commit Hooks
```bash
# Install pre-commit
pip install pre-commit
pre-commit install

# Run manually
pre-commit run --all-files
```

### Project Structure

```
moor-echo/
├── crates/
│   ├── echo-core/          # Core language implementation
│   │   ├── src/
│   │   │   ├── evaluator/  # Echo language evaluator
│   │   │   ├── parser/     # Grammar and parsing logic
│   │   │   ├── runtime/    # Task scheduler and runtime
│   │   │   ├── storage/    # Object and data persistence
│   │   │   └── tracer/     # System tracing and monitoring
│   │   ├── examples/       # Example programs and tools
│   │   └── tests/          # Integration tests
│   ├── echo-repl/          # REPL interface implementation
│   └── echo-web/           # Web UI and server
├── docs/                   # Documentation
├── examples/               # Example MOO and Echo code
├── scripts/                # Build and utility scripts
└── static/                 # Web UI static files
```

## Development Workflow

### Git Workflow

1. **Feature Development**:
   ```bash
   git checkout -b feature/new-functionality
   # Make changes
   git add .
   git commit -m "Add new functionality"
   git push origin feature/new-functionality
   ```

2. **Code Review**: Create pull request for review

3. **Integration**: Merge after approval and testing

### Build Commands

```bash
# Development build
cargo build

# Release build  
cargo build --release

# Build with web features
cargo build --features web-ui

# Build specific crate
cargo build -p echo-core
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test parser_tests

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

## Testing Strategy

### Test Categories

#### Unit Tests
Located within each module (`src/*/mod.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Test implementation
    }
}
```

#### Integration Tests
Located in `crates/*/tests/`:
- `system_tracer_tests.rs` - System tracing functionality
- `moo_db_browser_tests.rs` - Database browser integration
- `cowbell_integration_tests.rs` - MOO database compatibility

#### End-to-End Tests
Located in `tests/` (Playwright):
- `multiplayer_notify_test.spec.js` - Multi-user interaction testing

#### Echo Test Suite
Located in `crates/echo-core/test_suites/`:
- `minimal_sanity_test.echo` - Basic language functionality
- `simple_sanity_tests.echo` - Core features
- `harness_sanity_tests.echo` - Test framework validation

### Running Specific Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test "*"

# Echo language tests
./run_echo_tests.py crates/echo-core/test_suites/

# E2E tests
npx playwright test

# MOO database tests
cargo test moo_db_browser
```

### Test Data and Fixtures

Test databases in `examples/`:
- `Minimal.db` - 4 objects, basic test case
- `LambdaCore-latest.db` - 97 objects, standard MOO core
- `JHCore-DEV-2.db` - 237 objects, extended functionality

## Recent Improvements

### Performance Enhancements

#### JIT Compilation System
- **Cranelift Integration**: Native code compilation for ARM64/x86_64
- **WebAssembly JIT**: Universal bytecode execution with Wasmtime
- **Hot Path Detection**: Automatic optimization of frequently executed code
- **Performance Gains**: 5-50x speedup for numeric-intensive operations

#### Database Import Improvements
- **Flexible Parsing**: Handles whitespace variations in MOO databases
- **Error Recovery**: Continues parsing after non-critical errors
- **Progress Reporting**: Real-time import progress for large databases
- **Validation**: Comprehensive integrity checking post-import

#### Memory Management
- **Object Pooling**: Reuse common object types
- **String Interning**: Deduplicate string literals
- **Copy-on-Write**: Share data until modification needed
- **Garbage Collection**: Automatic cleanup of unused objects

### Language Feature Additions

#### Enhanced Object System
```echo
object Player extends $thing
    property name = "Anonymous"
    property health = 100
    
    // Modern event handling
    event on_damage(amount) {
        this.health = max(0, this.health - amount)
        if this.health == 0 {
            emit("player_died", this)
        }
    }
    
    // Datalog queries
    query can_see(other) :-
        same_location(this, other),
        not hidden(other).
endobject
```

#### Advanced Pattern Matching
```echo
match player_action {
    case Move{direction, ?speed = 1} => handle_movement(direction, speed),
    case Attack{target, weapon} if weapon.durability > 0 => attack(target, weapon),
    case Say{message} => broadcast_message(player, message),
    case _ => unknown_action()
}
```

#### Cooperative Multithreading
```echo
// Automatic yielding at loop boundaries
for item in large_inventory {
    process_item(item)  // Yields after each iteration
}

// Explicit task control
task long_calculation = fork {
    let result = expensive_computation()
    notify(player, "Calculation complete: " + result)
}
```

### Web UI Improvements

#### Real-time Collaboration
- **WebSocket Integration**: Bi-directional real-time communication
- **State Synchronization**: Automatic UI state sync across clients
- **Event Broadcasting**: Efficient multi-client event distribution
- **Optimistic Updates**: Immediate UI feedback with conflict resolution

#### Dynamic UI Creation
```echo
// Create interactive interfaces from Echo code
ui_clear()
ui_add_button("attack", "Attack Monster", "player:attack_monster()")
ui_add_text("status", "Health: " + player.health)
ui_add_progress("health_bar", player.health, player.max_health)

// Real-time updates
emit("ui_update", {element: "status", text: "Health: " + new_health})
```

### Developer Experience Improvements

#### Enhanced Error Messages
- **Source Location**: Precise error locations in code
- **Context Information**: Relevant code snippets in error output
- **Suggestion System**: Helpful suggestions for common mistakes
- **Stack Traces**: Complete call stack information for debugging

#### Debugging Tools
- **Interactive Debugger**: Step through code execution
- **Variable Inspection**: Examine object state and variables
- **Performance Profiler**: Identify bottlenecks and hot spots
- **Memory Analyzer**: Track memory usage patterns

#### Development Utilities
- **MOO Database Browser**: Visual exploration of imported databases
- **Syntax Validator**: Real-time syntax checking
- **Code Formatter**: Automatic code formatting
- **Documentation Generator**: Generate docs from code comments

## Code Quality Standards

### Formatting and Linting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Run linter with all targets
cargo clippy --all-targets --all-features
```

### Code Style Guidelines

#### Rust Code
- Follow standard Rust naming conventions
- Use `rustfmt` for consistent formatting
- Address all `clippy` warnings
- Prefer explicit error handling over `unwrap()`
- Document public APIs with rustdoc comments

```rust
/// Evaluates an Echo expression in the given environment
/// 
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `env` - The evaluation environment
/// 
/// # Returns
/// The result value or an evaluation error
pub fn evaluate_expression(
    expr: &Expression, 
    env: &mut Environment
) -> Result<Value, EvaluationError> {
    // Implementation
}
```

#### Echo Code Style
- Use clear, descriptive variable names
- Prefer explicit type annotations for clarity
- Structure objects logically with related functionality grouped
- Comment complex algorithms and business logic

```echo
object GameCharacter extends $player
    // Core attributes
    property name = "Unnamed"
    property level = 1
    property experience = 0
    
    // Derived attributes (computed properties)
    property max_health = this.level * 10 + 50
    
    /// Calculate damage for an attack
    function calculate_damage(weapon, target) {
        let base_damage = weapon.damage + this.level
        let armor_reduction = target.armor / 2
        return max(1, base_damage - armor_reduction)
    }
endobject
```

### Testing Standards

#### Unit Test Coverage
- Aim for >90% test coverage on core modules
- Test both success and error paths
- Use descriptive test names that explain the scenario
- Group related tests in modules

```rust
#[cfg(test)]
mod expression_evaluation_tests {
    use super::*;
    
    #[test]
    fn arithmetic_expression_evaluates_correctly() {
        let expr = parse_expression("2 + 3 * 4").unwrap();
        let result = evaluate_expression(&expr, &mut Environment::new()).unwrap();
        assert_eq!(result, Value::Integer(14));
    }
    
    #[test]
    fn division_by_zero_returns_error() {
        let expr = parse_expression("5 / 0").unwrap();
        let result = evaluate_expression(&expr, &mut Environment::new());
        assert!(matches!(result, Err(EvaluationError::DivisionByZero)));
    }
}
```

#### Integration Test Coverage
- Test complete user workflows
- Verify system behavior under realistic conditions
- Test error recovery and edge cases
- Validate performance characteristics

### Documentation Standards

#### Code Documentation
- Public APIs must have rustdoc comments
- Complex algorithms need explanation comments
- Configuration options should be documented
- Examples should be provided for non-trivial usage

#### User Documentation
- Keep README.md up to date with current features
- Provide comprehensive language reference
- Include practical examples and tutorials
- Document migration guides for MOO users

## Performance Optimization

### Profiling and Benchmarking

#### Built-in Benchmarks
```bash
# Run performance benchmarks
cargo bench

# Profile with perf (Linux)
perf record --call-graph dwarf target/release/echo-repl
perf report

# Memory profiling with valgrind
valgrind --tool=massif target/release/echo-repl
```

#### Performance Testing
```rust
// Example benchmark
#[bench]
fn bench_expression_evaluation(b: &mut Bencher) {
    let expr = parse_expression("complex_calculation()").unwrap();
    let mut env = Environment::new();
    
    b.iter(|| {
        evaluate_expression(&expr, &mut env)
    });
}
```

### Optimization Strategies

#### JIT Compilation
- Enable JIT for compute-intensive workloads
- Profile to identify hot paths suitable for compilation  
- Use feature flags for optional JIT backends
- Validate JIT output against interpreter results

#### Memory Optimization
- Use object pooling for frequently allocated types
- Implement copy-on-write for shared data structures
- Profile memory usage patterns
- Implement generational garbage collection for long-lived objects

#### Database Performance
- Use connection pooling for database access
- Implement caching for frequently accessed objects
- Batch database operations where possible
- Profile database query performance

## Debugging Tools

### Built-in Debugging Support

#### REPL Debugging Commands
```echo
.debug on                    # Enable debug mode
.trace object_method         # Trace method execution  
.inspect #123               # Examine object state
.stack                      # Show call stack
.vars                       # List current variables
```

#### Logging Configuration
```bash
# Set log level
RUST_LOG=debug cargo run

# Component-specific logging
RUST_LOG=echo_core::evaluator=debug,echo_core::parser=info cargo run

# Log to file
RUST_LOG=debug cargo run 2> debug.log
```

### External Debugging Tools

#### GDB Integration
```bash
# Build with debug symbols
cargo build --debug

# Run with GDB
rust-gdb target/debug/echo-repl
(gdb) break echo_core::evaluator::evaluate_expression
(gdb) run
```

#### Memory Debugging
```bash
# Address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo run

# Memory leak detection
valgrind --leak-check=full target/debug/echo-repl
```

## Release Process

### Version Management

Follow semantic versioning (SemVer):
- **MAJOR**: Breaking changes to public API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Release Steps

1. **Pre-release Testing**:
   ```bash
   # Run full test suite
   ./scripts/test-all.sh
   
   # Test with real MOO databases
   cargo test --release -- --include-slow-tests
   
   # Performance regression testing
   cargo bench > benchmark-results.txt
   ```

2. **Version Update**:
   ```bash
   # Update version in Cargo.toml files
   # Update CHANGELOG.md
   # Create release commit
   git commit -m "Release version X.Y.Z"
   git tag -a vX.Y.Z -m "Version X.Y.Z"
   ```

3. **Release Build**:
   ```bash
   # Build release artifacts
   ./scripts/release.sh

   # Generate checksums
   sha256sum target/release/echo-repl > checksums.txt
   ```

4. **Publication**:
   ```bash
   # Push tags
   git push origin main --tags
   
   # Create GitHub release
   gh release create vX.Y.Z target/release/echo-repl
   
   # Publish to crates.io (if applicable)
   cargo publish -p echo-core
   ```

### Quality Gates

Before release, ensure:
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation builds (`cargo doc`)
- [ ] MOO database compatibility verified
- [ ] Performance benchmarks within acceptable range
- [ ] Security audit passes (if applicable)
- [ ] CHANGELOG.md updated
- [ ] Version numbers consistent across all crates

This comprehensive development guide should help both new contributors and experienced developers work effectively with the Echo REPL codebase. The emphasis on testing, code quality, and clear documentation helps maintain a robust and reliable system while enabling rapid development of new features.