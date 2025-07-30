#!/usr/bin/env bash
# Release automation script for moor-echo

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DRY_RUN=false
VERSION_TYPE="patch"
CURRENT_BRANCH=$(git branch --show-current)

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

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -t, --type TYPE     Version bump type: patch, minor, major (default: patch)
    -d, --dry-run       Perform a dry run without making changes
    -h, --help          Show this help message

Examples:
    $0                  # Release a patch version
    $0 -t minor         # Release a minor version
    $0 -t major -d      # Dry run for a major version
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            VERSION_TYPE="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate version type
if [[ ! "$VERSION_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo_error "Invalid version type: $VERSION_TYPE"
    usage
    exit 1
fi

# Pre-release checks
echo_info "Running pre-release checks..."

# Check if we're on main branch
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo_error "Releases must be made from the main branch. Current branch: $CURRENT_BRANCH"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo_error "There are uncommitted changes. Please commit or stash them."
    exit 1
fi

# Pull latest changes
echo_info "Pulling latest changes from remote..."
git pull origin main

# Run tests
echo_info "Running tests..."
make test

# Run linting
echo_info "Running linters..."
make lint

# Run security audit
echo_info "Running security audit..."
make audit

# Check documentation
echo_info "Building documentation..."
make docs-build

# Get current version
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo_info "Current version: $CURRENT_VERSION"

# Calculate new version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR="${VERSION_PARTS[0]}"
MINOR="${VERSION_PARTS[1]}"
PATCH="${VERSION_PARTS[2]}"

case $VERSION_TYPE in
    patch)
        PATCH=$((PATCH + 1))
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo_info "New version will be: $NEW_VERSION"

if [[ "$DRY_RUN" == true ]]; then
    echo_warning "DRY RUN MODE - No changes will be made"
    echo ""
    echo "Would perform the following actions:"
    echo "  1. Update version in Cargo.toml files to $NEW_VERSION"
    echo "  2. Update CHANGELOG.md"
    echo "  3. Commit changes"
    echo "  4. Create tag v$NEW_VERSION"
    echo "  5. Push changes and tag to remote"
    echo "  6. Create GitHub release"
    echo "  7. Publish crates to crates.io"
    exit 0
fi

# Confirm release
echo ""
echo_warning "This will release version $NEW_VERSION"
read -p "Continue? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo_info "Release cancelled"
    exit 0
fi

# Update versions
echo_info "Updating version numbers..."
# Update workspace Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update crate versions
for crate in crates/*/Cargo.toml; do
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$crate"
    rm "${crate}.bak"
done

# Update dependencies
echo_info "Updating internal dependencies..."
cargo update

# Create changelog entry
echo_info "Updating CHANGELOG.md..."
cat > CHANGELOG_NEW.md << EOF
# Changelog

## [v$NEW_VERSION] - $(date +%Y-%m-%d)

### Added
- TODO: Add new features

### Changed
- TODO: Add changes

### Fixed
- TODO: Add fixes

### Security
- TODO: Add security updates

EOF

if [[ -f CHANGELOG.md ]]; then
    tail -n +2 CHANGELOG.md >> CHANGELOG_NEW.md
    mv CHANGELOG_NEW.md CHANGELOG.md
fi

# Commit changes
echo_info "Committing changes..."
git add -A
git commit -m "chore: release v$NEW_VERSION"

# Create tag
echo_info "Creating tag v$NEW_VERSION..."
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

# Push changes
echo_info "Pushing changes to remote..."
git push origin main
git push origin "v$NEW_VERSION"

echo ""
echo_success "Release v$NEW_VERSION completed successfully!"
echo ""
echo "Next steps:"
echo "  1. Wait for CI/CD to complete"
echo "  2. Check the GitHub release page"
echo "  3. Verify crates.io publications"
echo "  4. Update CHANGELOG.md with actual changes"
echo ""