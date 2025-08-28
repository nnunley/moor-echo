# Echo REPL Documentation

Comprehensive documentation for the Echo REPL programming environment - a modern implementation of the Echo programming language with real-time web UI capabilities and multi-user collaboration features.

## Documentation Structure

This directory contains consolidated, comprehensive documentation organized for external reviewers and contributors to understand the system architecture, implementation details, and development processes.

### Core Documentation

#### [ARCHITECTURE.md](ARCHITECTURE.md) 
**System Architecture and Design**
- Core system architecture and component design  
- Runtime system and concurrency model
- Storage architecture and persistence strategy
- Event system and real-time communication
- Security model and performance characteristics
- Integration patterns and deployment strategies

#### [MOO_COMPATIBILITY.md](MOO_COMPATIBILITY.md)
**MOO Database Integration and Compatibility**  
- Complete MOO compatibility layer documentation
- Database import system for .db files (LambdaCore, JHCore, etc.)
- Built-in function mapping and implementation status
- Migration tools and validation strategies
- Testing approach for MOO compatibility

#### [PARSER_AND_GRAMMAR.md](PARSER_AND_GRAMMAR.md)
**Parser Architecture and Grammar Implementation**
- Comprehensive rust-sitter integration details
- Dual-grammar system (MOO legacy + modern Echo)
- AST structure and language implementation
- Error recovery and IDE support features
- Performance characteristics and optimization strategies

#### [DEVELOPMENT_NOTES.md](DEVELOPMENT_NOTES.md)  
**Development Process and Workflow**
- Project setup and development environment
- Testing strategies and code quality standards  
- Recent improvements and performance enhancements
- Debugging tools and release processes
- Comprehensive development workflow guide

### Language Documentation

#### [ECHO_LANGUAGE_DESIGN.md](ECHO_LANGUAGE_DESIGN.md)
**Core Language Philosophy and Design**
- Event-Centered Hybrid Objects (ECHO) language design
- Object-centric programming model
- Cooperative multithreading and persistence
- Language philosophy and design principles

#### [LANGUAGE_ROADMAP.md](LANGUAGE_ROADMAP.md)  
**Implementation Roadmap and Feature Planning**
- Current implementation status and completed features
- Phase-by-phase development plan
- Advanced features and future capabilities
- MOO compatibility implementation priorities

### Specialized Topics

#### [TESTING.md](TESTING.md)
Testing framework and validation strategies

#### [ECHO_SYSTEM_TRACER.md](ECHO_SYSTEM_TRACER.md)  
System tracing and monitoring capabilities

#### [ECHO_REFLECTION_DESIGN.md](ECHO_REFLECTION_DESIGN.md)
Reflection and metaprogramming features  

#### [CAPABILITY_BOOTSTRAP.md](CAPABILITY_BOOTSTRAP.md)
Capability-based security system

#### [XMLUI_INTEGRATION.md](XMLUI_INTEGRATION.md)
XML UI integration and web interface

### Reference Materials

#### [PRODUCTION_REFERENCE.md](PRODUCTION_REFERENCE.md)
Production deployment reference

#### [error-constants.md](error-constants.md)
Error code definitions and constants

#### [moo-language-notes.md](moo-language-notes.md) 
MOO language reference notes

#### [rust-sitter.md](rust-sitter.md)
rust-sitter implementation details

## Quick Start Guide

For new users and contributors:

## Key Features

### 🚀 **Core Language Features**

- Dynamic typing with integers, floats, strings, lists, maps, and objects
- Object-oriented programming with properties, methods, and inheritance
- Lambda functions with closure support
- Comprehensive control flow (if/else, loops, try/catch)
- Built-in functions for string manipulation, math, and system operations

### 🌐 **Web UI Capabilities**

- **Real-time Collaboration**: Multiple users can interact simultaneously
- **Dynamic UI Creation**: Build interactive interfaces directly from Echo code
- **Event System**: Reactive programming with custom event handlers
- **Live Updates**: Changes propagate instantly across all connected clients
- **Persistent State**: UI state and application data stored in the database

### 🎯 **Built-in UI Functions**

```echo
ui_clear()                           // Clear the dynamic UI area
ui_add_button(id, label, action)     // Add interactive buttons
ui_add_text(id, text, style)         // Add styled text elements
ui_add_div(id, content, style)       // Add container elements
ui_update(id, properties)            // Update existing elements
emit(event_name, ...args)            // Emit events to web clients
```

## Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **Web browser** (Chrome, Firefox, Safari, or Edge)

### Installation

1. **Clone the repository**:

   ```bash
   git clone <repository-url>
   cd echo-repl
   ```

2. **Build the project**:
   ```bash
   cargo build --features web-ui
   ```

### Running the Service

#### **Web UI Mode (Recommended)**

Start the Echo REPL with web interface:

```bash
./target/debug/echo-repl --web --port 8080
```

Then open your browser to: **http://localhost:8080**

#### **Command Line Mode**

For traditional REPL experience:

```bash
./target/debug/echo-repl
```

#### **Command Line Options**

```bash
./target/debug/echo-repl [OPTIONS]

Options:
  --web                Enable web UI interface
  --port <PORT>        Web server port (default: 8080)
  --db <PATH>          Database directory path (default: ./echo-db)
  <FILE>               Execute Echo script file on startup
```

## Quick Start Examples

### Basic Echo Programming

```echo
// Variables and basic operations
let name = "Echo"
let version = 1.0
print("Welcome to " + name + " v" + str(version))

// Objects and properties
object hello
  property greeting = "Hello"
  property target = "World"
endobject

print(hello.greeting + ", " + hello.target + "!")
```

### Interactive Web UI

```echo
// Load the web UI example
.load web_ui_example.echo

// Create dynamic interface
ui_clear()
ui_add_text("title", "My App", {fontSize: "24px"})
ui_add_button("btn1", "Click Me", "print('Button clicked!')")

// Emit custom events
emit("web:ui:show_message", "Hello from Echo!")
```

### Multi-User Chat

```echo
// Load the chat application
.load chat_app.echo

// Open multiple browser windows to test real-time communication
// Users can join, send messages, and see updates instantly
```

## Example Applications

The repository includes several example applications:

| File                      | Description                                  |
| ------------------------- | -------------------------------------------- |
| `web_ui_example.echo`     | Basic UI manipulation and event handling     |
| `web_ui_advanced.echo`    | Interactive dashboard with tabs and controls |
| `chat_app.echo`           | Real-time multi-user chat application        |
| `dynamic_ui_builder.echo` | Dynamic form builder with live updates       |
| `test_multi_user.echo`    | Simple shared counter for testing sync       |

## Architecture

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

### **Key Components**

- **Echo Evaluator**: Executes Echo code and manages object state
- **Event System**: Routes events between Echo code and web clients
- **WebNotifier**: Manages WebSocket connections and real-time updates
- **Storage Engine**: Persistent database for objects, players, and code
- **Web Interface**: Browser-based UI with dynamic element creation

## Development Commands

### **REPL Commands**

```
.help                  Show available commands
.quit                  Exit the REPL
.eval                  Enter multi-line evaluation mode
.player create <name>  Create a new player
.player switch <name>  Switch to different player
.player list          List all players
.env                   Show current environment variables
.load <file>           Load and execute Echo script
.stats                 Show session statistics
```

### **Player Management**

Echo uses a player-based environment system where each user has their own
variable scope and object context. This enables multi-user programming with
isolated environments.

## Advanced Features

### **Lambda Functions**

```echo
// Function definitions with closures
let counter = 0
let increment = fn(step) {
    counter = counter + (step || 1)
    return counter
}

print(increment(5))  // Output: 5
print(increment())   // Output: 6
```

### **Event Handlers**

```echo
// Custom event handling
on_user_action = fn(action, data) {
    if action == "click" {
        ui_add_text("log", "Button clicked: " + data.button)
        emit("web:ui:user_interaction", action, data)
    }
}
```

### **Real-time Collaboration**

```echo
// Shared state management
let shared_data = {users: [], messages: []}

broadcast_update = fn(type, data) {
    emit("web:shared:" + type, data)
    // Automatically syncs across all connected clients
}
```

## Configuration

### **Environment Variables**

```bash
RUST_LOG=debug          # Enable debug logging
ECHO_DB_PATH=./data     # Custom database path
ECHO_WEB_PORT=3000      # Custom web port
```

### **Database Structure**

```
echo-db/
├── conf                # Database configuration
├── db                  # Object storage
└── snap.*             # Database snapshots
```

## Performance & Limits

- **Concurrent Users**: Supports dozens of simultaneous web clients
- **Object Storage**: Efficient binary serialization with snapshotting
- **Memory Usage**: ~10-50MB baseline, scales with active objects
- **Event Throughput**: 1000+ events/second for real-time applications

## Technical Implementation

### **Core Language Features**

- **Data Types**: Integer (i64), Float (f64), String, Boolean, Objects, Lists,
  Maps, Null
- **Expressions**: Arithmetic, comparison, boolean logic, property access,
  method calls
- **Control Flow**: If/else conditionals, for/while loops, break/continue
  statements
- **Variable Assignment**: Let bindings (mutable), const bindings (immutable)
- **Object System**: Object creation, property assignment, method definitions
- **Lambda Functions**: Closures with optional parameters and rest arguments

### **Parser System**

- Dual-grammar architecture with rust-sitter integration
- Unified AST supporting both MOO and modern Echo features
- Program-level parsing for multi-statement evaluation

### **Storage System**

- Persistent object storage using Sled database
- Automatic serialization with rkyv
- Per-player environments with isolated variable scopes
- Object reference mapping and player registry

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit a pull request

### **Development Build**

```bash
# Debug build with all features
cargo build --features web-ui

# Release build for production
cargo build --release --features web-ui

# Run tests
cargo test
```

## License

This project is open source. See LICENSE file for details.

## Roadmap

- [ ] **Visual Editor**: Drag-and-drop UI builder interface
- [ ] **API Gateway**: REST API for external integration
- [ ] **Plugin System**: Custom extensions and modules
- [ ] **Performance Monitor**: Real-time performance analytics
- [ ] **Cloud Deployment**: Docker containers and cloud templates
- [ ] **Mobile Support**: Progressive web app capabilities

---

**Echo REPL** - Building the future of collaborative programming environments.
