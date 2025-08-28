# Echo REPL Architecture

Comprehensive system architecture documentation for the Echo REPL programming environment, covering runtime design, concurrency model, storage architecture, and system integration patterns.

## Table of Contents

- [Overview](#overview)
- [Core Architecture](#core-architecture)
- [Runtime System](#runtime-system)
- [Concurrency Model](#concurrency-model)
- [Storage Architecture](#storage-architecture)
- [Parser Architecture](#parser-architecture)
- [Event System](#event-system)
- [Security Model](#security-model)
- [Performance Characteristics](#performance-characteristics)
- [Integration Patterns](#integration-patterns)

## Overview

Echo REPL is built on a multi-layer architecture that combines modern systems programming (Rust) with dynamic language runtime capabilities. The system is designed for:

- **Real-time multi-user interaction**
- **Persistent object storage**
- **Cooperative multithreading**
- **Event-driven programming**
- **Web-based user interfaces**

### Key Design Principles

1. **Event-Driven Architecture**: All user interactions and system events flow through a unified event system
2. **Object-Centric Design**: Everything is an object with properties, methods, and event handlers
3. **Persistent-First**: All state persists by default, supporting server restarts and long-running sessions
4. **Cooperative Concurrency**: Non-blocking execution with automatic yielding points
5. **Web-Native**: Built-in support for real-time web UIs and multi-user collaboration

## Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Echo REPL System                     │
├─────────────────────────────────────────────────────────────┤
│  Web Interface Layer                                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │   Browser   │ │  WebSocket  │ │   HTTP API  │            │
│  │   Clients   │ │  Gateway    │ │   Server    │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│  Application Layer                                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │    REPL     │ │  Command    │ │  Web UI     │            │
│  │   Engine    │ │  Processor  │ │  Manager    │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│  Runtime Layer                                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │    Echo     │ │   Event     │ │   Task      │            │
│  │  Evaluator  │ │   System    │ │ Scheduler   │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│  Language Layer                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │   Parser    │ │     AST     │ │    JIT      │            │
│  │  (Tree-     │ │  Evaluator  │ │ Compiler    │            │
│  │  Sitter)    │ │             │ │ (Optional)  │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer                                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │   Object    │ │   Player    │ │   Event     │            │
│  │   Store     │ │  Registry   │ │   Store     │            │
│  │   (Sled)    │ │             │ │             │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

## Runtime System

### Execution Model

Echo uses an **interpreter-first** approach with optional JIT compilation for hot paths:

#### Interpreter Core
- **Tree-walking interpreter** for primary execution
- **Direct AST evaluation** without intermediate bytecode
- **Cooperative yielding** at loop boundaries and async operations
- **Persistent execution state** that survives server restarts

#### Optional JIT Integration
- **Cranelift JIT** for native code compilation (ARM64/x86_64)
- **WebAssembly JIT** for universal bytecode execution
- **Hot path detection** based on execution frequency
- **Feature-flagged** compilation for flexible deployment

### Runtime Components

#### Echo Evaluator
```rust
pub struct EchoEvaluator {
    storage: Arc<dyn ObjectStorage>,
    scheduler: TaskScheduler,
    event_system: EventSystem,
    jit_backend: Option<JitBackend>,
}
```

The evaluator handles:
- **Variable scoping** with per-player environments
- **Object method dispatch** with inheritance resolution  
- **Built-in function execution**
- **Event emission and handling**
- **Error propagation and recovery**

#### Task Scheduler
Implements cooperative multithreading with:
- **Task queuing** with priority levels
- **Automatic yielding** at predefined points
- **Task suspension** with timer-based resumption
- **Inter-task communication** through events

#### Memory Management
- **Reference counting** for objects and values
- **Garbage collection** for circular references
- **Object pooling** for frequently allocated types
- **Continuation persistence** for long-running tasks

## Concurrency Model

### Cooperative Multithreading

Echo implements **cooperative multithreading** where tasks voluntarily yield control:

#### Automatic Yield Points
- **Loop iterations**: End of `for` and `while` loops
- **Function calls**: Before expensive operations
- **I/O operations**: File access, network calls
- **Event processing**: Between event handlers

#### Explicit Yield Controls
```echo
suspend(5.0);        // Suspend for 5 seconds
yield;               // Yield control immediately
sleep(1000);         // Sleep for 1000ms
```

#### Task Management
- **Task IDs**: Unique identifiers for all running tasks
- **Task hierarchy**: Parent-child relationships
- **Task permissions**: Security context inheritance
- **Task cancellation**: Graceful shutdown of running tasks

### Persistence Model

#### Execution State Persistence
All execution state can be serialized and persisted:

```rust
pub struct TaskState {
    pub task_id: TaskId,
    pub player_id: PlayerId, 
    pub call_stack: CallStack,
    pub local_vars: Variables,
    pub program_counter: ProgramCounter,
    pub suspended_until: Option<SystemTime>,
}
```

#### Recovery Process
1. **Serialize state** before suspension/shutdown
2. **Store to disk** in the task database
3. **Restore on startup** with full context
4. **Resume execution** from exact point of suspension

## Storage Architecture

### Multi-Database Architecture

Echo uses specialized databases for different data types:

#### Object Store (Primary Database)
```
echo-db/
├── conf                 # Database configuration
├── db                   # Primary object storage (Sled)
└── snap.*              # Automatic snapshots
```

**Schema Design:**
- **Object IDs**: Numeric identifiers (#1, #2, etc.)
- **Properties**: Key-value storage with type information
- **Inheritance**: Parent-child relationships
- **Permissions**: Access control metadata

#### Player Registry
- **Player profiles**: Name, connection info, preferences
- **Environment variables**: Per-player scoped variables
- **Session state**: Active connections and UI state
- **Authentication**: Login credentials and session tokens

#### Event Store
- **Event history**: Persistent log of all system events
- **Subscriptions**: Object event handler registrations
- **Replay capability**: Reconstruct system state from events
- **Performance metrics**: Event processing statistics

### Data Persistence Strategy

#### Automatic Serialization
Uses `rkyv` for efficient binary serialization:
- **Zero-copy deserialization**
- **Validation** on load
- **Version compatibility** checks
- **Compression** for large objects

#### Snapshot Management
- **Periodic snapshots** of full database state
- **Incremental backups** between snapshots
- **Automatic cleanup** of old snapshots
- **Fast recovery** from snapshots

## Parser Architecture

### Dual-Grammar Design

Echo supports both legacy MOO syntax and modern Echo syntax through a dual-grammar architecture:

#### Tree-Sitter Integration
- **rust-sitter** for compile-time grammar inclusion
- **Performance optimized** parsing with incremental updates
- **Error recovery** with partial parsing support
- **Syntax highlighting** support for editors

#### Grammar Structure
```
Grammar Components:
├── Literals (numbers, strings, booleans)
├── Identifiers (variables, properties)  
├── Expressions (arithmetic, logical, comparison)
├── Statements (assignments, control flow)
├── Objects (definitions, property access)
├── Functions (definitions, calls, lambdas)
├── Control Flow (if/else, loops, try/catch)
└── MOO Compatibility (legacy syntax support)
```

#### AST Design
Unified AST supporting both syntaxes:
- **Statement/Expression separation**
- **Pattern matching** for destructuring
- **Error recovery nodes** for partial parsing
- **Source location** tracking for debugging

### Parser Performance
- **Incremental parsing** for live editing
- **Parallel parsing** for large files
- **Cached parse results** for frequently accessed code
- **Memory efficient** AST representation

## Event System

### Event-Driven Architecture

All interactions in Echo flow through a unified event system:

#### Event Types
1. **User Events**: Keyboard input, mouse clicks, form submissions
2. **System Events**: Server startup, player connections, errors
3. **Custom Events**: Application-defined events between objects
4. **Timer Events**: Scheduled events and task resumption

#### Event Flow
```
Event Source → Event Queue → Event Dispatcher → Event Handlers → Side Effects
     │              │              │               │                │
   User Input    Priority        Router         Object         State Changes
   System       Ordering       Resolution      Methods        UI Updates
   Timer        Buffering      Permission      Functions      Persistence
   Network      Batching       Checking        Queries        Notifications
```

#### Event Processing
- **Asynchronous processing** with cooperative yielding
- **Priority-based queuing** for different event types
- **Event batching** for performance optimization
- **Error isolation** preventing cascading failures

### Real-Time Communication

#### WebSocket Integration
- **Bidirectional** real-time communication
- **Event broadcasting** to multiple clients
- **Selective subscriptions** based on player location/interests
- **Connection management** with automatic reconnection

#### UI State Synchronization
- **Differential updates** only send changes
- **Conflict resolution** for simultaneous edits
- **Optimistic updates** with rollback capability
- **State reconciliation** on reconnection

## Security Model

### Capability-Based Security

Echo implements a capability-based security system:

#### Capability Design
```echo
capability ReadDocument(doc: Object, reader: Player)
capability WriteDocument(doc: Object, writer: Player)  
capability CreateObject(parent: Object, creator: Player)
capability DeleteObject(obj: Object, deleter: Player)
```

#### Permission Checking
```echo
// Query-based permission validation
query can_read(user, document) :-
    has_capability(user, ReadDocument(document, user)).

// Runtime permission enforcement
if !this:can_read(player, document) {
    throw E_PERM;
}
```

#### Security Contexts
- **Task permissions**: Inherited from calling context
- **Player permissions**: Base capability set
- **Object ownership**: Creation and modification rights
- **System permissions**: Administrative capabilities

### Sandboxing
- **Resource limits**: CPU time, memory usage, disk space
- **API restrictions**: Limited access to system functions
- **Network isolation**: Controlled external communication
- **File system isolation**: Restricted file access

## Performance Characteristics

### Benchmarking Results

#### Execution Performance
- **Simple operations**: ~1-10μs (variable access, arithmetic)
- **Function calls**: ~10-100μs (including scope setup)
- **Object method calls**: ~50-200μs (inheritance resolution)
- **Database operations**: ~100μs-1ms (depending on complexity)
- **Event processing**: ~10-50μs per event
- **JIT compilation**: 5-50x speedup for numeric intensive code

#### Memory Usage
- **Base system**: ~10-20MB minimal configuration
- **Per player**: ~1-5MB (depending on active variables)
- **Per object**: ~100-1000 bytes (depending on properties)
- **AST caching**: ~50-500KB per parsed file
- **Event buffers**: ~1-10MB (depending on activity)

#### Scalability Limits
- **Concurrent players**: 50-500 (depending on activity)
- **Objects in database**: Millions (tested with 100K+)
- **Events per second**: 1000-10000 (depending on complexity)
- **Task queue depth**: 10000+ concurrent tasks
- **WebSocket connections**: 100-1000 simultaneous

### Optimization Strategies

#### JIT Compilation
- **Hot path detection**: Track execution frequency
- **Compilation triggers**: >100 executions
- **Optimization levels**: Basic, aggressive, specialized
- **Fallback strategy**: Interpreter for edge cases

#### Caching
- **Parse result caching**: Avoid re-parsing unchanged code  
- **Method resolution caching**: Cache inheritance lookups
- **Property access caching**: Fast property resolution
- **Event handler caching**: Optimize event dispatch

#### Memory Management
- **Object pooling**: Reuse common object types
- **String interning**: Deduplicate string literals
- **Copy-on-write**: Share data until modification
- **Generational collection**: Optimize garbage collection

## Integration Patterns

### Web Framework Integration

#### HTTP API Server
```rust
// RESTful API for external integration
GET    /api/objects/{id}           # Get object state
POST   /api/objects/{id}/methods   # Invoke object method  
GET    /api/players/{id}           # Get player information
POST   /api/events                # Emit custom events
```

#### WebSocket API
```javascript
// Real-time communication protocol
{
    "type": "event",
    "name": "ui_update", 
    "data": {"element": "button1", "property": "text", "value": "Click Me!"}
}

{
    "type": "command",
    "command": "look",
    "args": []
}
```

### Database Integration

#### Import/Export
- **MOO database import**: Parse and convert existing MOO databases
- **JSON export**: Export objects and data in JSON format
- **SQL integration**: Connect to external SQL databases
- **Backup/restore**: Full system backup and restoration

#### Migration Support
- **Version compatibility**: Handle different Echo versions
- **Schema evolution**: Migrate data structures
- **Data validation**: Verify integrity during migration
- **Rollback capability**: Undo problematic migrations

### Development Tools Integration

#### Language Server Protocol
- **Syntax highlighting**: Rich editor support
- **Code completion**: Context-aware suggestions  
- **Error checking**: Real-time error detection
- **Refactoring**: Automated code transformations

#### Testing Framework
- **Unit testing**: Test individual functions and objects
- **Integration testing**: Test multi-object interactions
- **Performance testing**: Benchmark critical operations
- **Regression testing**: Prevent functionality degradation

### Deployment Patterns

#### Container Deployment
```dockerfile
FROM rust:alpine
COPY . /app
RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/echo-repl", "--web", "--port", "8080"]
```

#### Cloud Integration
- **Docker containers**: Containerized deployment
- **Load balancing**: Multiple instance coordination
- **Health monitoring**: Service health endpoints
- **Auto-scaling**: Dynamic resource allocation

## Conclusion

The Echo REPL architecture provides a solid foundation for building interactive, multi-user programming environments. The combination of cooperative multithreading, persistent continuations, event-driven design, and web-native capabilities creates a unique platform for collaborative programming and interactive world-building.

The modular architecture supports both high-level application development and low-level system optimization, making it suitable for educational use, rapid prototyping, and production deployment.