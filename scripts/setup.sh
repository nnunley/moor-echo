#!/usr/bin/env bash
# Development environment setup script for moor-echo

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if command -v "$1" >/dev/null 2>&1; then
        echo_success "$1 is installed"
        return 0
    else
        echo_error "$1 is not installed"
        return 1
    fi
}

# Main setup
echo_info "Setting up moor-echo development environment..."

# Check for required tools
echo_info "Checking required tools..."

MISSING_TOOLS=()

if ! check_command "rustup"; then
    MISSING_TOOLS+=("rustup")
fi

if ! check_command "node"; then
    MISSING_TOOLS+=("node")
fi

if ! check_command "npm"; then
    MISSING_TOOLS+=("npm")
fi

if ! check_command "python3"; then
    MISSING_TOOLS+=("python3")
fi

if ! check_command "git"; then
    MISSING_TOOLS+=("git")
fi

if [ ${#MISSING_TOOLS[@]} -ne 0 ]; then
    echo_error "The following required tools are missing:"
    for tool in "${MISSING_TOOLS[@]}"; do
        echo "  - $tool"
    done
    echo ""
    echo "Please install the missing tools and run this script again."
    exit 1
fi

# Install Rust components
echo_info "Installing Rust components..."
rustup component add rustfmt clippy rust-analyzer
echo_success "Rust components installed"

# Install cargo tools
echo_info "Installing cargo tools..."
CARGO_TOOLS=(
    "cargo-tarpaulin"
    "cargo-audit"
    "cargo-outdated"
    "cargo-edit"
    "cargo-watch"
    "cargo-release"
)

for tool in "${CARGO_TOOLS[@]}"; do
    if ! cargo install --list | grep -q "^$tool"; then
        echo_info "Installing $tool..."
        cargo install "$tool" --locked
    else
        echo_success "$tool is already installed"
    fi
done

# Install Node.js dependencies
echo_info "Installing Node.js dependencies..."
npm install
echo_success "Node.js dependencies installed"

# Install Python dependencies
echo_info "Installing Python tools..."
python3 -m pip install --user --upgrade pip
python3 -m pip install --user pre-commit black ruff mypy pytest pytest-cov

# Install pre-commit hooks
echo_info "Setting up pre-commit hooks..."
pre-commit install
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push
echo_success "Pre-commit hooks installed"

# Create necessary directories
echo_info "Creating necessary directories..."
mkdir -p .vscode
mkdir -p scripts
mkdir -p docs
echo_success "Directories created"

# Set up git hooks
echo_info "Setting up git configuration..."
git config core.hooksPath .git/hooks
echo_success "Git configuration updated"

# Run initial checks
echo_info "Running initial checks..."
make fmt-check || true
make lint || true

echo ""
echo_success "Development environment setup complete!"
echo ""
echo "You can now run:"
echo "  make help     - Show all available commands"
echo "  make build    - Build the project"
echo "  make test     - Run tests"
echo "  make run-repl - Run the Echo REPL"
echo ""