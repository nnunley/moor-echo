# Echo REPL

Echo is an Event-Centered Hybrid Objects language, extending and modernizing MOO (MUD Object-Oriented) with modern programming concepts.

## Current Implementation Status

### Core Features Implemented

#### 1. Basic REPL
- Interactive command-line interface
- Single-line and multi-line input modes
- Execution timing display
- Player management system

#### 2. Data Types
- **Numbers**: Integer (i64) and Float (f64) with full arithmetic
- **Strings**: String literals with concatenation
- **Booleans**: true/false literals
- **Objects**: Object references (#0, #1, etc.)
- **Lists**: List literals and operations
- **Null**: null value

#### 3. Expressions
- **Arithmetic**: +, -, *, /, % operators with proper precedence
- **Comparison**: == operator (more coming soon)
- **Property Access**: object.property syntax
- **System Properties**: $propname syntax (resolves to #0.propname)
- **Method Calls**: object:method() syntax (partial implementation)

#### 4. Object System
- **Lambda Objects**: Named objects created with object...endobject syntax
- **Properties**: Property definitions with initialization expressions
- **Object Binding**: Lambda objects automatically bound as properties on #0
- **Property Assignment**: obj.prop = value syntax

#### 5. Variable Assignment
- **Simple Assignment**: x = 42
- **Let Bindings**: let x = 42 (mutable variables)
- **Const Bindings**: const x = 42 (immutable variables)
- **Property Assignment**: obj.prop = value
- **Const Protection**: Prevents reassignment of const variables

#### 6. REPL Commands
- `.help` - Show help message
- `.quit` - Exit the REPL
- `.eval` - Enter multi-line mode (end with . on its own line)
- `.env` - Show current environment variables
- `.scope` - Show all variables and objects in scope
- `.player create <name>` - Create a new player
- `.player switch <name>` - Switch to a different player
- `.player list` - List all players
- `.player` - Show current player

#### 7. Storage System
- Persistent object storage using RocksDB
- Automatic serialization with rkyv
- System objects (#0 and #1) initialization

### Architecture

#### Parser System
- Dual-grammar architecture prepared for MOO compatibility
- Modern Echo parser using rust-sitter
- Unified AST supporting both MOO and modern Echo features
- Program-level parsing for multi-statement evaluation

#### Evaluator
- Player-based execution environments
- Variable scoping with environment tracking
- Const variable protection
- Object property evaluation
- Method call framework (extensible)

#### LValue/RValue System
- Sophisticated assignment targets
- Support for variable, property, and index assignments
- Destructuring patterns (AST support, parsing pending)
- Binding type tracking (let/const/none)

### Usage Examples

```bash
# Start the REPL
cargo run --bin echo-repl

# Basic arithmetic
echo> 2 + 2
=> 4 [0.123ms]

# Variable assignment
echo> x = 42
=> 42 [0.456ms]
echo> y = x * 2
=> 84 [0.234ms]

# Let and const bindings
echo> let mutable = 100
=> 100 [0.123ms]
echo> const PI = 3.14159
=> 3.14159 [0.234ms]
echo> PI = 3.14  # Error: Cannot reassign const variable

# Object creation (multi-line mode)
echo> .eval
Entering eval mode. End with '.' on a line by itself.
eval> object hello
eval> property greeting = "Hello";
eval> property name = "World"
eval> endobject
eval> .
=> object created [1.234ms]

# Property access
echo> hello.greeting
=> Hello [0.123ms]

# Property assignment
echo> hello.name = "Echo"
=> Echo [0.234ms]

# Environment inspection
echo> .env
=== Current Environment ===
Player: #12345678

Variables:
  PI (const) = 3.14159
  mutable = 100
  x = 42
  y = 84

# Scope inspection
echo> .scope
=== Current Scope ===

Player Variables (player_#12345678):
  PI (const) = 3.14159
  mutable = 100
  x = 42
  y = 84

Objects (bound to #0):
  hello = #87654321

System Properties:
  $system = #0
  $root = #1
```

### Planned Features

#### Control Flow (Next Priority)
- If/else conditionals
- While loops
- For loops
- Break/continue statements

#### Advanced Features
- Events and event handlers
- Green threads and cooperative multitasking
- Pattern matching
- Type system
- Module system
- Full MOO compatibility mode

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run --bin echo-repl

# Run with input file
cargo run --bin echo-repl < test_script.echo
```

### Technical Notes

- Uses rust-sitter for grammar definition and parsing
- RocksDB for persistent storage
- rkyv for high-performance serialization
- Prepared for JIT compilation (Cranelift/WASM backends)
- Extensible architecture supporting multiple parser backends

### Known Limitations

- List destructuring patterns not yet parsed (AST support exists)
- Method execution is simplified (no verb code compilation yet)
- No control flow structures yet
- Limited error messages (line/column info coming soon)
- No import/module system yet

### Contributing

The Echo language is under active development. Key areas for contribution:
- Implementing control flow structures
- Improving error messages with source locations
- Adding more operators and expressions
- Implementing the event system
- Creating comprehensive test suites