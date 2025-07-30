# Development Guide for Moor-Echo

This guide provides comprehensive instructions for setting up and working with
the Moor-Echo development environment.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [CI/CD](#cicd)
- [Docker Development](#docker-development)
- [Release Process](#release-process)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.75 or later) - Install via [rustup](https://rustup.rs/)
- **Node.js** (18 or later) and npm - Install from
  [nodejs.org](https://nodejs.org/)
- **Python** (3.8 or later) - For test scripts and tooling
- **Git** - Version control
- **Docker** (optional) - For containerized development

### Recommended Tools

- **VS Code** with recommended extensions (see `.vscode/extensions.json`)
- **pre-commit** - For git hooks
- **Make** - For running common tasks

## Quick Start

1. Clone the repository:

   ```bash
   git clone https://github.com/username/moor-echo.git
   cd moor-echo
   ```

2. Run the setup script:

   ```bash
   ./scripts/setup.sh
   ```

3. Build the project:

   ```bash
   make build
   ```

4. Run tests:

   ```bash
   make test
   ```

5. Start the REPL:
   ```bash
   make run-repl
   ```

## Development Environment Setup

### Manual Setup

If you prefer manual setup or the script fails:

1. **Install Rust components:**

   ```bash
   rustup component add rustfmt clippy rust-analyzer
   ```

2. **Install cargo tools:**

   ```bash
   cargo install cargo-tarpaulin cargo-audit cargo-outdated cargo-edit cargo-watch
   ```

3. **Install Node.js dependencies:**

   ```bash
   npm install
   ```

4. **Install Python tools:**

   ```bash
   pip install pre-commit black ruff mypy pytest pytest-cov
   ```

5. **Set up pre-commit hooks:**
   ```bash
   pre-commit install
   pre-commit install --hook-type commit-msg
   pre-commit install --hook-type pre-push
   ```

### VS Code Setup

1. Open the project in VS Code
2. Install recommended extensions when prompted
3. Reload VS Code to activate all settings

The workspace is configured with:

- Rust analyzer for code intelligence
- Formatters for all languages
- Debugger configurations
- Task runners

## Project Structure

```
moor-echo/
├── crates/              # Rust workspace crates
│   ├── echo-core/       # Core language implementation
│   ├── echo-repl/       # REPL interface
│   └── echo-web/        # Web server interface
├── examples/            # Example Echo programs
├── test_suites/         # Echo language test suites
├── scripts/             # Development scripts
├── .github/workflows/   # CI/CD workflows
└── docs/                # Documentation
```

## Development Workflow

### Common Tasks

All common development tasks are available through the Makefile:

```bash
make help              # Show all available commands
make build             # Build the project
make test              # Run all tests
make fmt               # Format code
make lint              # Run linters
make run-repl          # Start the REPL
make run-web           # Start the web server
```

### Working with Cargo

Direct cargo commands for specific tasks:

```bash
cargo build --package echo-repl    # Build specific crate
cargo test --workspace             # Run all tests
cargo run --package echo-web       # Run web server
cargo watch -x test                # Watch and test
```

### Tree-sitter Development

When modifying the grammar:

```bash
npx tree-sitter generate    # Generate parser
npx tree-sitter test        # Run grammar tests
make tree-sitter            # Run both commands
```

## Testing

### Test Categories

1. **Unit Tests**: Test individual components

   ```bash
   make test-unit
   ```

2. **Integration Tests**: Test component interactions

   ```bash
   make test-integration
   ```

3. **Echo Language Tests**: Test the language implementation

   ```bash
   make test-echo
   ```

4. **Tree-sitter Tests**: Test the grammar
   ```bash
   make test-tree-sitter
   ```

### Running All Tests

```bash
make test              # Quick test run
./scripts/test-all.sh  # Comprehensive test suite
```

### Code Coverage

Generate coverage reports:

```bash
make coverage
# Open tarpaulin-report.html in your browser
```

## Code Quality

### Formatting

The project uses automatic formatting:

```bash
make fmt        # Format all code
make fmt-check  # Check formatting without changes
```

### Linting

Run all linters:

```bash
make lint  # Run all linters
make fix   # Auto-fix issues where possible
```

### Pre-commit Hooks

Pre-commit hooks run automatically on git commit. To run manually:

```bash
make pre-commit  # Run on all files
pre-commit run --files path/to/file  # Run on specific files
```

## CI/CD

### GitHub Actions Workflows

1. **workspace-ci.yml**: Main CI workflow
   - Runs on every push and PR
   - Tests on multiple platforms
   - Includes security scanning and coverage

2. **release.yml**: Release automation
   - Triggered by version tags
   - Builds binaries for all platforms
   - Publishes to crates.io

3. **echo-repl-ci.yml**: Legacy workflow for echo-repl

### Running CI Locally

Test CI checks locally before pushing:

```bash
make ci-local
```

## Docker Development

### Quick Start with Docker

```bash
# Build images
docker-compose build

# Run services
docker-compose up

# Development environment
docker-compose run dev bash
```

### Development Mode

For hot-reloading development:

```bash
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

This provides:

- Auto-reloading on code changes
- Database and Redis services
- Admin interfaces for debugging

## Release Process

### Automated Release

Use the release script:

```bash
./scripts/release.sh           # Patch release
./scripts/release.sh -t minor  # Minor release
./scripts/release.sh -t major  # Major release
./scripts/release.sh -d        # Dry run
```

### Manual Release

1. Update version numbers in `Cargo.toml` files
2. Update CHANGELOG.md
3. Commit changes: `git commit -m "chore: release vX.Y.Z"`
4. Create tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
5. Push: `git push origin main --tags`

## Troubleshooting

### Common Issues

1. **Build fails with "cannot find crate"**
   - Run `cargo update` to update dependencies
   - Check that all workspace members are listed in root `Cargo.toml`

2. **Tree-sitter tests fail**
   - Regenerate parser: `npx tree-sitter generate`
   - Check grammar.js for syntax errors

3. **Pre-commit hooks fail**
   - Run `make fmt` to fix formatting
   - Run `make fix` to auto-fix linting issues

4. **Docker build fails**
   - Ensure Docker daemon is running
   - Try `docker system prune` to free space
   - Check Dockerfile syntax

### Getting Help

- Check existing issues on GitHub
- Run tests with verbose output: `RUST_LOG=debug cargo test`
- Use `make help` to see available commands
- Enable backtrace: `RUST_BACKTRACE=1 cargo run`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass
5. Run `make ci-local` to verify
6. Submit a pull request

### Code Style

- Follow Rust naming conventions
- Write descriptive commit messages
- Add tests for new features
- Update documentation as needed

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/)
- [Echo Language Design](docs/ECHO_LANGUAGE_DESIGN.md)
- [Implementation Guide](IMPLEMENTATION_GUIDE.md)
