# Moor Echo Project

A comprehensive implementation of the Echo programming language with modern
features, real-time collaboration, and web-based interfaces.

## Project Overview

This repository contains a complete ecosystem for the Echo programming language,
including:

- **Echo REPL**: Interactive programming environment with web UI
- **Language Implementation**: Modern parser, evaluator, and runtime
- **Documentation**: Comprehensive guides and specifications
- **Examples**: Sample applications and test suites
- **Core Libraries**: Standard Echo object hierarchy

## Repository Structure

```
moor-echo/
├── echo-repl/              # Main Echo REPL implementation
│   ├── src/                # Rust source code
│   ├── static/             # Web UI assets
│   ├── tests/              # Integration tests
│   ├── examples/           # Echo code examples
│   └── README.md           # Detailed Echo REPL documentation
├── docs/                   # Project documentation
├── examples/               # Language examples and demos
├── echo_jhcore/            # Core Echo object library
└── test_suites/            # Comprehensive test suites
```

## Quick Start

### 🚀 **Echo REPL with Web UI**

The main implementation is in the `echo-repl/` directory:

```bash
cd echo-repl
cargo build --features web-ui
./target/debug/echo-repl --web --port 8080
```

Then open your browser to: **http://localhost:8080**

### 📖 **Documentation**

- **[Echo REPL README](echo-repl/README.md)** - Comprehensive guide for the main
  implementation
- **[Language Design](docs/ECHO_LANGUAGE_DESIGN.md)** - Echo language
  specification
- **[Implementation Guide](IMPLEMENTATION_GUIDE.md)** - Technical implementation
  details

## Key Features

### 🎯 **Echo Language**

- Dynamic object-oriented programming
- Lambda functions with closures
- Real-time event system
- Persistent object storage
- Multi-user environments

### 🌐 **Web Interface**

- Real-time collaboration between multiple users
- Dynamic UI creation from Echo code
- WebSocket-based event propagation
- Interactive programming environment

### ⚡ **Performance**

- Rust-based implementation for speed and safety
- Efficient binary serialization
- Event-driven architecture
- Optional JIT compilation support

## Example Applications

The repository includes several example applications showcasing different
capabilities:

| Directory                           | Description                                       |
| ----------------------------------- | ------------------------------------------------- |
| `echo-repl/chat_app.echo`           | Real-time multi-user chat application             |
| `echo-repl/web_ui_advanced.echo`    | Interactive dashboard with tabs and controls      |
| `echo-repl/dynamic_ui_builder.echo` | Dynamic form builder with live updates            |
| `examples/`                         | Language feature demonstrations                   |
| `echo_jhcore/`                      | Core object library (player, room, thing classes) |

## Architecture

### **Core Components**

1. **Parser System** (`echo-repl/src/parser/`)
   - Rust-sitter based grammar
   - Support for both MOO and modern Echo syntax
   - AST generation and source code reconstruction

2. **Evaluator** (`echo-repl/src/evaluator/`)
   - Dynamic type system
   - Object-oriented execution model
   - Event system integration
   - Player environment management

3. **Storage Engine** (`echo-repl/src/storage/`)
   - Persistent object database
   - Binary serialization with rkyv
   - Snapshot-based persistence

4. **Web Interface** (`echo-repl/src/web/`, `echo-repl/static/`)
   - Real-time WebSocket communication
   - Dynamic UI element creation
   - Multi-client event synchronization

### **Event-Driven Design**

```
┌─────────────────┐    Events    ┌─────────────────┐
│   Echo Code     │◄────────────►│   Web Clients   │
│   (Database)    │   WebSocket  │   (Browsers)    │
└─────────────────┘              └─────────────────┘
         │                                │
         │          ┌─────────────────┐   │
         └─────────►│  Event System   │◄──┘
                    │  (Rust Core)    │
                    └─────────────────┘
```

## Development

### **Prerequisites**

- Rust (latest stable)
- Node.js (for any JavaScript tooling)
- Web browser for testing UI features

### **Building**

```bash
# Main implementation
cd echo-repl
cargo build --features web-ui

# Run tests
cargo test

# Development build with all features
cargo build --features "web-ui jit"
```

### **Testing**

```bash
# Run Rust tests
cargo test

# Run Echo language tests
cd echo-repl
./run_echo_tests.sh

# Test specific features
cargo test --test repl_tests
```

## Documentation

### **Language Documentation**

- **[Echo Language Design](docs/ECHO_LANGUAGE_DESIGN.md)** - Language
  specification and design philosophy
- **[MOO Comparison](GRAMMAR_COMPARISON.md)** - Comparison with original MOO
  language
- **[Implementation Guide](IMPLEMENTATION_GUIDE.md)** - Technical implementation
  details

### **Development Guides**

- **[CST Alignment Plan](CST_ALIGNMENT_PLAN.md)** - Concrete syntax tree
  implementation
- **[Lambda Implementation](LAMBDA_IMPLEMENTATION.md)** - Function system design
- **[Event System](echo-repl/src/evaluator/event_system.rs)** - Event-driven
  architecture

### **Testing Documentation**

- **[Testing Guide](TESTING.md)** - Test suite organization and usage
- **[Test Suites](test_suites/)** - Organized test collections

## Project History

This project represents a modernization of the MOO (MUD Object-Oriented)
programming language, originally developed for multi-user virtual environments.
Echo extends MOO with:

- Modern syntax and language features
- Web-based interfaces
- Real-time collaboration
- Performance optimizations
- Type safety through Rust implementation

## Contributing

1. **Fork the repository**
2. **Create a feature branch** from the main branch
3. **Implement changes** with appropriate tests
4. **Update documentation** as needed
5. **Submit a pull request**

### **Areas for Contribution**

- Language feature implementation
- Performance optimizations
- Web UI enhancements
- Documentation improvements
- Test coverage expansion
- Example applications

## License

This project is open source. See individual files for specific license
information.

## Roadmap

### **Short Term**

- [ ] Complete MOO language compatibility
- [ ] Enhanced error messages with source locations
- [ ] Performance profiling and optimization
- [ ] Extended standard library

### **Medium Term**

- [ ] Visual programming interface
- [ ] Mobile web app support
- [ ] Plugin system for extensions
- [ ] Cloud deployment options

### **Long Term**

- [ ] Distributed object system
- [ ] Real-time collaborative editing
- [ ] Advanced debugging tools
- [ ] Machine learning integration

---

## Getting Help

- **Main Documentation**: See [echo-repl/README.md](echo-repl/README.md) for
  detailed usage instructions
- **Language Questions**: Check [docs/](docs/) for language design and
  specification
- **Development Issues**: Review
  [IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md) for technical details
- **Examples**: Browse [examples/](examples/) and [echo_jhcore/](echo_jhcore/)
  for code samples

**Echo Project** - Bringing modern collaborative programming to virtual worlds.
