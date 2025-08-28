# Development Tools and Utilities

This directory contains various development tools, utilities, and configuration files organized by purpose.

## Directory Structure

### `docs/`
AI-generated documentation and tool-specific guides:
- MOO programmer's manual updates
- Tree-sitter parsing documentation
- Grammar writing guides and tutorials

### `web-dev/`
Web development and testing tools:
- `package.json` - Node.js dependencies for web development
- `playwright.config.js` - End-to-end testing configuration
- `e2e-tests/` - Playwright end-to-end test suites
- `node_modules/` - Node.js dependencies (ignored by git)

### `python-dev/`
Python development configuration:
- `pyproject.toml` - Python project configuration
- `requirements.txt` - Python dependencies for tooling

### `legacy-files/`
Deprecated or legacy configuration files:
- `go.mod` - Old tree-sitter Go module configuration

## Usage

### Web Development
```bash
cd tools/web-dev
npm install
npx playwright test
```

### Python Tools
```bash
cd tools/python-dev
pip install -r requirements.txt
```

### Documentation
The `docs/` directory contains reference materials and guides generated during development. These are primarily for historical reference and development context.

## Organization Philosophy

This directory keeps development tools and configuration files organized and separated from the main project structure, while maintaining easy access for developers. Files are grouped by technology stack or purpose to avoid clutter in the root directory.