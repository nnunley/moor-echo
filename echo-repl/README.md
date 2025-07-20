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
- **Comparison**: ==, !=, <, <=, >, >= operators
- **Boolean Logic**: &&, ||, ! operators with short-circuit evaluation
- **Property Access**: object.property syntax
- **System Properties**: $propname syntax (resolves to #0.propname)
- **Method Calls**: object:method() syntax with full verb execution

#### 4. Object System
- **Lambda Objects**: Named objects created with object...endobject syntax
- **Properties**: Property definitions with initialization expressions
- **Verbs**: Method definitions with parameters and return values
- **Object Binding**: Lambda objects automatically bound as properties on #0
- **Property Assignment**: obj.prop = value syntax
- **Object References**: #n syntax with flexible administrator-controlled mapping
- **Player Registry**: Separate player object management with username tracking

#### 5. Variable Assignment
- **Simple Assignment**: x = 42
- **Let Bindings**: let x = 42 (mutable variables)
- **Const Bindings**: const x = 42 (immutable variables)
- **Property Assignment**: obj.prop = value
- **Const Protection**: Prevents reassignment of const variables

#### 6. Functions and Control Flow
- **Lambda Functions**: `fn {params} body endfn` syntax
- **Optional Parameters**: `fn {x, ?y = 10} x + y endfn`
- **Rest Parameters**: `fn {first, @rest} ... endfn`
- **If/Else**: `if condition ... else ... endif`
- **For Loops**: `for item in collection ... endfor`
- **While Loops**: `while condition ... endwhile`
- **Break/Continue**: Loop control statements

#### 7. REPL Commands
- `.help` - Show help message
- `.quit` - Exit the REPL
- `.eval` - Enter multi-line mode (end with . on its own line)
- `.env` - Show current environment variables
- `.scope` - Show all variables and objects in scope
- `.dump` - Dump environment as JSON to stderr
- `.quiet` - Toggle quiet mode (suppress evaluation output)
- `.load <file>` - Load and execute an Echo file
- `.reset` - Reset the current environment
- `.stats` - Show session statistics and memory usage
- `.player create <name>` - Create a new player
- `.player switch <name>` - Switch to a different player
- `.player list` - List all players
- `.player` - Show current player

#### 8. Storage System
- Persistent object storage using Sled database
- Automatic serialization with rkyv
- System objects (#0 and #1) initialization
- Per-player environments with isolated variable scopes
- Player registry with username management
- Object reference mapping via object_map

### Not Yet Implemented

These MOO features are planned but not yet implemented:

#### List Comprehensions
MOO-style comprehensions using curly brace syntax:
```
{x * 2 for x in [1, 2, 3]}  // => [2, 4, 6]
{n * n for n in [1..10]}     // => [1, 4, 9, ..., 100]
```

#### Range Syntax
- `[1..10]` for numeric ranges
- Used in for loops and comprehensions

#### Other MOO Features
- Fork statements for async execution
- Try-catch expressions with `! codes` syntax
- Pass expressions for calling parent verbs
- Maps/dictionaries with `[key -> value]` syntax
- Symbol literals `'symbol`
- Error constants (E_TYPE, E_PERM, etc.)

See GRAMMAR_COMPARISON.md for a full comparison with MOO.

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
- Full verb execution with parameter binding
- Object reference resolution with object_map support
- Player registry with username management

#### LValue/RValue System
- Sophisticated assignment targets
- Support for variable, property, and index assignments
- Destructuring patterns (AST support, parsing pending)
- Binding type tracking (let/const/none)

### Recent Additions

#### Verb System
- Full verb execution from stored AST
- Parameter binding with optional and default values
- `this` and `caller` bindings in verb context
- Proper control flow handling (return statements)
- Properties only accessible via `this.property` syntax within verbs

#### Object Reference System
- Flexible #n object reference resolution
- Administrator-controlled mapping via:
  - `#0.object_map` property (Map type)
  - `#0:object_map(n)` verb method
- Built-in references: #0 (system), #1 (root)
- Helpful error messages for unmapped references

#### Player Management
- Separate player registry from object namespace
- Username-based lookup with `find_player_by_username`
- Support for username changes without breaking references
- Players stored in `#0.players` property

#### Code Introspection
- Complete AST to source code generation
- Source code storage in verb definitions
- Future support for version history

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

# Lambda functions
echo> let add = fn {x, y} x + y endfn
=> <function> [0.123ms]
echo> add(5, 3)
=> 8 [0.234ms]

# Optional and rest parameters
echo> let greet = fn {name, ?greeting = "Hello"} greeting + " " + name endfn
=> <function> [0.123ms]
echo> greet("World")
=> Hello World [0.234ms]

# Control flow
echo> for i in [1, 2, 3, 4, 5]
...   if i % 2 == 0
...     i * i
...   endif
... endfor
=> 4 [0.345ms]
=> 16 [0.345ms]

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

# Control flow
echo> if (x > 40) "x is big"
=> x is big [0.123ms]

echo> if (x < 40) "x is small" else "x is not small"
=> x is not small [0.234ms]

# Lists and iteration
echo> items = {1, 2, 3}
=> [1, 2, 3] [0.123ms]

echo> for (item in items) item
=> null [0.234ms]
```

#### 7. Control Flow ✅ IMPLEMENTED
- **If/Else Conditionals**: Complete with optional else clause
- **While Loops**: Basic while loop implementation
- **For Loops**: for-in iteration over lists
- **Boolean Logic**: AND (&&), OR (||), NOT (!) operators with short-circuit evaluation

### Planned Features

#### Enhanced Control Flow (Next Priority)
- Break/continue statements for loops
- Nested loop control
- Exception handling (try/catch)

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
- Limited error messages (line/column info coming soon)
- No import/module system yet
- Object definition syntax from MOO not yet implemented
- No event system or Datalog queries yet
- Comments not supported in the grammar
- Arrow function syntax not implemented

### Contributing

The Echo language is under active development. Key areas for contribution:
- Implementing object definition syntax (object...endobject)
- Adding event handlers and Datalog queries
- Improving error messages with source locations
- Implementing arrow function syntax
- Adding comment support to the grammar
- Implementing the event system
- Creating comprehensive test suites