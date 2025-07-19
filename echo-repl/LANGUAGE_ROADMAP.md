# Echo Language Implementation Roadmap

## Current Status

### ✅ Completed Core Features

#### Parser Infrastructure
- **rust-sitter Grammar**: Complete migration from hand-rolled parser
- **Comprehensive Grammar Tests**: 41 tests covering all language constructs
- **AST Structure**: Well-defined AST with proper precedence and associativity

#### Basic Language Constructs
- **Literals**: Numbers, strings, booleans (`true`, `false`)
- **Variables**: Let statements with proper scoping
- **Arithmetic**: Addition with string concatenation support
- **Property Access**: Object property access with dot notation
- **Identifiers**: Variable and property name resolution

#### Control Flow
- **Conditional Statements**: `if (condition) statements endif` and `if-else` constructs
- **Comparison Operators**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Boolean Logic**: Truthiness evaluation (proper falsy values)
- **Loop Constructs**: `for (variable in collection)`, `while (condition)` with `break` and `continue`
- **Exception Handling**: `try...catch(exception_var)...endtry` blocks and `throw value;` statements

#### Object System
- **Object Definitions**: `object Name ... endobject` syntax
- **Property Definitions**: `property name = value;` within objects
- **Verb Definitions**: `verb name() ... endverb` method definitions
- **Method Calls**: `object:method(args)` invocation syntax

#### Execution Environment
- **Player System**: Multi-player execution environments
- **Storage Layer**: Persistent object storage with Sled database
- **Environment Management**: Per-player variable scoping

#### JIT Compilation
- **Cranelift JIT**: Native code compilation for ARM64/x86_64
- **WebAssembly JIT**: Universal bytecode compilation with Wasmtime
- **Feature-Flagged Architecture**: Runtime evaluator selection

### 🔄 In Progress

#### Advanced Language Features
- **Comprehensive Testing**: Expanding test coverage for edge cases
- **Documentation**: Updating design documents with recent progress

## Phase 1: Essential Language Features

### 🎯 Current Priority: Function Definitions and Calls

#### Function Syntax
```javascript
function calculate(a, b) {
    return a + b;
}

// Function calls
let result = calculate(5, 3);
```

#### Function Features
- **Local Scope**: Proper variable scoping within functions
- **Return Values**: Early returns and value passing
- **Parameter Handling**: Named parameters and argument passing
- **Call Stack**: Support for recursion with proper stack management

### 🎯 Next Priority: Lambda/Anonymous Functions

#### Lambda Function Syntax with Scatter Assignment
```javascript
// ✅ Existing scatter assignment patterns in Echo
let {a, b, c} = some_list;
let {x, ?y = 5, @rest} = coordinates;
const {x: posX, y: posY, ?z: posZ = 0} = coordinates;

// Lambda declarations with scatter assignment
let {add} = (a, b) => a + b;
let {multiply, divide} = {
    multiply: (a, b) => a * b,
    divide: (a, b) => a / b
};

// Function parameters with destructuring (existing pattern)
function process_character({name, level, ?class = "warrior", @equipment}) {
    return `${name} (${level} ${class}) has ${equipment.length} items`;
}

// Lambda expressions with destructuring parameters
let damage_calc = {attacker: {strength, ?bonus = 0}, defender: {armor}} => 
    max(1, (strength + bonus) - armor);

// Powerful {} = ... syntax for complex assignments (existing)
let {player, location, inventory} = parse_player_data(data);
let {health, mana, @stats} = player.attributes;
let {x, y, z} = player.position;

// Event handlers with scatter assignment (existing pattern)
event ItemPickedUp({player, item: {type, ?enchantment}}) {
    if (enchantment) {
        player:tell(`You picked up an enchanted ${type}!`);
    }
}

// Multiple function assignment with scatter syntax
let {counter1, counter2} = {
    counter1: makeCounter(),
    counter2: makeCounter()
};
```

#### Lambda Features
- **Arrow Function Syntax**: `(params) => expression` and `(params) => { statements }`
- **Scatter Assignment**: `let {fn1, fn2} = {fn1: () => ..., fn2: () => ...}`
- **Destructuring**: `let {a, b, ...rest} = object` for complex assignments
- **Closures**: Capture variables from enclosing scope
- **First-Class Functions**: Functions as values, stored in variables and data structures
- **Higher-Order Functions**: Functions that take or return other functions
- **Implicit Returns**: Single expression lambdas return automatically

### 🔧 Implementation Tasks

#### Regular Functions (Completed)
1. **✅ Grammar Extension**: Add function definitions and calls to rust-sitter grammar
2. **✅ AST Variants**: Define Function and FunctionCall AST nodes
3. **✅ Evaluator Support**: Implement function evaluation with call stack management
4. **✅ Scoping**: Implement function-local variable scoping
5. **✅ Grammar Conflict**: Resolved using assignment syntax `let name = fn(params) body endfn`
6. **✅ Testing**: Comprehensive function testing including recursion

#### Object-Owned Events (Next Priority)
1. **Grammar Extension**: Add event handler syntax within object definitions
2. **AST Variants**: Define Event and EventHandler AST nodes  
3. **Evaluator Support**: Implement event handler registration and execution
4. **Event Emission**: Add syntax for emitting events (`emit`, `fire`, `trigger`)
5. **Event Subscription**: Add syntax for subscribing to events (`on`, `listen`)

#### Object-Owned Queries (Next Priority)
1. **Grammar Extension**: Add Datalog query syntax within object definitions
2. **AST Variants**: Define Query and QueryRule AST nodes
3. **Query Engine**: Implement Datalog evaluation with Horn clauses
4. **Query Execution**: Add syntax for executing queries `object:query(args)`
5. **Capability Integration**: Integrate queries with capability system

#### Lambda Functions (Future)
1. **Grammar Extension**: Add lambda expression syntax `(params) => expression`
2. **✅ Scatter Assignment**: Already implemented `let {a, ?b = default, @rest} = object`
3. **AST Variants**: Define Lambda and ArrowFunction AST nodes (DestructuringAssignment exists)
4. **Closure Support**: Implement variable capture from enclosing scopes
5. **First-Class Functions**: Functions as values in the type system
6. **Higher-Order Functions**: Built-in functions like map, filter, reduce

### 🎯 Advanced Error Handling (Future)

#### Enhanced Exception Features
- **Exception Types**: Custom error types and hierarchies
- **Finally Blocks**: Resource cleanup guarantees
- **Stack Traces**: Detailed error reporting

## Phase 2: Advanced Features

### 🎯 Cooperative Multithreading

#### Yield Points
- **Automatic Yielding**: End of loop iterations
- **Explicit Yielding**: `yield` and `suspend(duration)` statements
- **Builtin Futures**: Async operations that return futures

#### Persistent Continuations
- **Execution Context**: Serializable execution state
- **Disk Persistence**: Survive server restarts
- **Recovery**: Automatic continuation recovery on startup

### 🎯 Event System and Queries

#### The Lobby Object - System Root
```javascript
object $lobby {
    // System-wide properties
    property server_name = "Echo MOO Server";
    property start_time = time();
    property player_count = 0;
    
    // Global system functions
    function find_player(name) {
        return this:query_players().find(p => p.name == name);
    }
    
    function broadcast(message) {
        for (let player of this:query_online_players()) {
            player:tell(message);
        }
    }
    
    // System verbs accessible globally
    verb who() {
        return this:query_online_players().map(p => p.name);
    }
    
    verb help(topic) {
        return this:query_help_topics().find(h => h.topic == topic);
    }
    
    // Global event handlers
    event player_connect(player) {
        this.player_count = this.player_count + 1;
        this:broadcast(`${player.name} has connected.`);
    }
    
    event player_disconnect(player) {
        this.player_count = this.player_count - 1;
        this:broadcast(`${player.name} has disconnected.`);
    }
    
    // System-wide queries - handle $ redirects
    query query_players() :-
        is_player(Player).
    
    query query_online_players() :-
        is_player(Player),
        online(Player).
    
    query query_objects_by_type(type) :-
        has_property(Object, "type", type).
    
    query query_help_topics() :-
        has_property(Object, "help_topic", Topic).
}

// $ syntax redirects through $lobby
$who()              // → $lobby:who()
$find_player("bob") // → $lobby:find_player("bob")
$broadcast("Hello") // → $lobby:broadcast("Hello")
```

#### Object-Owned Everything
```javascript
object Player {
    // Properties (owned by this object)
    property name = "Alice";
    property location = #123;
    property level = 5;
    
    // Functions (owned by this object)
    function calculate_experience(base_xp, multiplier) {
        return base_xp * multiplier * this.level;
    }
    
    // Verbs (owned by this object)
    verb examine() {
        return `You see ${this.name}, a level ${this.level} player.`;
    }
    
    // Event handlers (owned by this object)
    event player_move(from_location, to_location) {
        if (this.location != from_location) return;
        $lobby:broadcast(`${this.name} moved from ${from_location} to ${to_location}`);
        this.location = to_location;
    }
    
    event player_connect() {
        this.last_login = time();
        $lobby:emit("player_connect", this);
    }
    
    // Datalog queries (owned by this object)
    query can_see(other_player) :- 
        same_location(this, other_player),
        not(hidden(other_player)).
    
    query accessible_exits() :-
        exit_from(this.location, Exit),
        has_capability(this, UseExit(Exit)).
}
```

#### $ Syntax Resolution
```javascript
// All $ syntax resolves through $lobby first
$who()                    // → $lobby:who()
$find_player("alice")     // → $lobby:find_player("alice")
$query_players()          // → $lobby:query_players()
$broadcast("message")     // → $lobby:broadcast("message")

// $ can also refer to other system objects through $lobby
$database:find_by_type("room")  // → $lobby:resolve("database"):find_by_type("room")
$security:check_access(user)    // → $lobby:resolve("security"):check_access(user)
$scheduler:add_task(task)       // → $lobby:resolve("scheduler"):add_task(task)
```

#### System Objects Architecture
```javascript
// Core system objects all registered through $lobby
object $lobby {
    property system_objects = {
        "database" => #101,
        "security" => #102, 
        "scheduler" => #103,
        "network" => #104
    };
    
    function resolve(name) {
        return this.system_objects[name];
    }
    
    query query_system_objects() :-
        system_object(Name, ObjectId).
}

object $database {
    // Database-specific queries
    query find_players_by_location(location) :-
        is_player(Player),
        at_location(Player, location).
    
    query inheritance_chain(object, ancestor) :-
        parent(object, ancestor).
    
    query inheritance_chain(object, ancestor) :-
        parent(object, intermediate),
        inheritance_chain(intermediate, ancestor).
}

object $security {
    // Security-specific queries and functions
    query can_access(subject, object, permission) :-
        has_capability(subject, permission(object)).
    
    function check_permissions(user, action, target) {
        return this:can_access(user, target, action);
    }
}

// Usage examples with $ redirection
let players_here = $database:find_players_by_location(here);
let can_read = $security:can_access(this, document, "read");
let online_users = $query_online_players();  // → $lobby:query_online_players()
```

#### Event System - Object-Owned
```javascript
object EventManager {
    // Event handlers are owned by this object
    event system_startup() {
        log("System starting up");
        emit("welcome_message", "Server is online");
    }
    
    event player_join(player) {
        this:log_player_action(player, "joined");
        emit("player_count_changed", this:count_online_players());
    }
    
    // Functions supporting events (owned by this object)
    function log_player_action(player, action) {
        let timestamp = time();
        log(`${timestamp}: ${player.name} ${action}`);
    }
    
    function count_online_players() {
        return Database:get_online_players().length;
    }
}

// Event emission - always through object methods
player:emit("player_move", old_room, new_room);
EventManager:emit("system_restart", time());
```

### 🎯 Advanced Object System

#### Inheritance
```javascript
object Child extends Parent {
    property child_prop = "value";
    
    verb override_method() {
        super.override_method();
        // Additional behavior
    }
}
```

#### Mixins and Traits
```javascript
trait Debuggable {
    verb debug() {
        return this.toString();
    }
}

object MyObject implements Debuggable {
    // Automatically gets debug() method
}
```

### 🎯 Advanced Control Flow

#### Pattern Matching
```javascript
match (value) {
    case Number(n) if n > 0 => "positive",
    case String(s) => "text: " + s,
    case List(items) => "list with " + items.length + " items",
    case _ => "unknown"
}
```

#### Generators and Iterators
```javascript
function* fibonacci() {
    let a = 0, b = 1;
    while (true) {
        yield a;
        [a, b] = [b, a + b];
    }
}

for (num in fibonacci()) {
    if (num > 100) break;
    print(num);
}
```

## Phase 3: System Integration

### 🎯 Timely Dataflow Integration

#### Stream Processing
```javascript
// Define data streams
let input_stream = stream.from_source(data_source);
let processed = input_stream
    .map(transform_function)
    .filter(predicate)
    .aggregate(aggregation_function);

// Output results
processed.to_sink(output_handler);
```

#### Real-time Processing
- **Event Streams**: Handle real-time events from players
- **Distributed Computation**: Leverage Timely's distributed capabilities
- **Fault Tolerance**: Automatic recovery from node failures

### 🎯 Advanced Storage Features

#### Transactions
```javascript
transaction {
    object1.property = new_value;
    object2.update_state();
    
    if (validation_fails()) {
        rollback;
    }
    
    commit;
}
```

#### Queries
```javascript
// Object queries
let results = query("SELECT * FROM objects WHERE type = ?", ["player"]);

// Property-based queries
let online_players = objects.where(obj => obj.online && obj.location != null);
```

### 🎯 Network and Communication

#### HTTP Integration
```javascript
// Async HTTP requests
let response = await http.get("https://api.example.com/data");
let data = await response.json();

// WebSocket support
let ws = websocket.connect("wss://chat.example.com");
ws.on_message(handle_message);
```

#### Inter-Player Communication
```javascript
// Send message to player
player.send_message("Hello from the system");

// Broadcast to all players
broadcast("Server announcement");

// Channel-based communication
let channel = channels.create("global_chat");
channel.subscribe(player_id, message_handler);
```

## Phase 4: Performance and Optimization

### 🎯 JIT Optimization

#### Hot Path Detection
- **Execution Profiling**: Track frequently executed code paths
- **Adaptive Compilation**: Compile hot paths with higher optimization
- **Inline Caching**: Cache property access and method calls

#### Memory Management
- **Garbage Collection**: Automatic memory management
- **Object Pooling**: Reuse objects to reduce allocation pressure
- **Continuation Pooling**: Reuse continuation objects

### 🎯 Distributed Execution

#### Load Balancing
- **Player Distribution**: Balance players across server instances
- **Continuation Migration**: Move long-running tasks between servers
- **Resource Awareness**: Schedule based on CPU/memory usage

#### Fault Tolerance
- **Replication**: Replicate critical continuations
- **Automatic Recovery**: Recover from server failures
- **Graceful Degradation**: Maintain service during partial failures

## Phase 5: Development Tools and Ecosystem

### 🎯 Debugging and Profiling

#### Interactive Debugger
```javascript
// Debugging support
debugger;  // Breakpoint
inspect(object);  // Object inspection
trace(function_name);  // Function call tracing
```

#### Performance Profiling
- **Execution Profiling**: Track time spent in functions
- **Memory Profiling**: Monitor memory usage patterns
- **Continuation Analysis**: Analyze continuation performance

### 🎯 Development Environment

#### Language Server Protocol
- **Syntax Highlighting**: Rich syntax highlighting for editors
- **Auto-completion**: Intelligent code completion
- **Error Checking**: Real-time error detection
- **Refactoring**: Automated code refactoring tools

#### Testing Framework
```javascript
// Unit testing
test("arithmetic operations", () => {
    assert_equals(2 + 2, 4);
    assert_true(5 > 3);
});

// Integration testing
test("player interactions", async () => {
    let player = create_test_player();
    let result = await player.execute_command("look");
    assert_contains(result, "You are in");
});
```

## Implementation Priority Matrix

### High Priority (Essential)
1. **Function Definitions** - Core programming construct (COMPLETED)
2. **Event Handler Syntax** - Object-owned event system (NEXT)
3. **Datalog Query Syntax** - Object-owned declarative queries (NEXT)
4. **Lambda/Anonymous Functions** - Modern functional programming features
5. **Cooperative Multithreading** - Key differentiator for MOO systems
6. **Advanced Object System** - Enhanced OOP capabilities

### Medium Priority (Important)
1. **Pattern Matching** - Modern language feature
2. **Generators/Iterators** - Advanced iteration patterns
3. **Timely Dataflow** - Distributed processing capabilities
4. **Advanced Networking** - Extended connectivity features

### Low Priority (Enhancement)
1. **Performance Optimizations** - After core features are stable
2. **Development Tools** - Support productivity improvements
3. **Advanced Networking** - Extended connectivity features
4. **Ecosystem Integration** - Third-party tool integration

## Success Metrics

### Language Completeness
- **Core Features**: 100% of essential language constructs implemented
- **Test Coverage**: >90% code coverage with comprehensive tests
- **Documentation**: Complete language reference and tutorials

### Performance Targets
- **Execution Speed**: Competitive with Python/JavaScript for typical workloads
- **Memory Usage**: <100MB for typical MOO server with 50 concurrent players
- **Continuation Overhead**: <10ms additional latency for yielding operations

### Reliability Goals
- **Server Uptime**: 99.9% uptime with continuation persistence
- **Data Integrity**: Zero data loss during server restarts
- **Error Recovery**: Graceful handling of all error conditions

## Next Steps

1. **Implement Functions**: Function definitions and calls with proper scoping (HIGH PRIORITY)
2. **Begin Multithreading**: Start with basic cooperative scheduling and yield points
3. **Advanced Object Features**: Inheritance and enhanced OOP capabilities
4. **Performance Testing**: Benchmark against other MOO implementations
5. **Timely Dataflow**: Distributed processing integration

This roadmap provides a clear path from the current state to a fully-featured, production-ready MOO programming language with unique cooperative multithreading and persistent continuation capabilities.