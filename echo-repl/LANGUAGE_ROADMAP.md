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
- **List Comprehensions**: MOO-style comprehensions `{expr for var in iterable}`
- **Grammar Restructuring**: Aligning with MOO's CST structure
- **Comprehensive Testing**: Expanding test coverage for edge cases
- **Documentation**: Updating design documents with recent progress

#### Grammar Improvements (Based on MOO Analysis)
- **Statement/Expression Separation**: Restructuring AST for clearer parsing
- **Unified Pattern System**: Consolidating parameter and binding patterns
- **Precedence Table**: Implementing MOO's complete operator precedence
- **Range Syntax**: Adding `[start..end]` for numeric ranges
- **Error Recovery**: Graceful handling of parse errors

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

### 🎯 List Comprehensions (To Implement)

MOO supports list comprehensions using curly brace syntax, which provides a concise way to create lists:

```javascript
// Basic comprehension
let doubles = {x * 2 for x in [1, 2, 3, 4, 5]};  // => [2, 4, 6, 8, 10]

// Comprehension with range
let squares = {x * x for x in [1..10]};  // => [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]

// Nested comprehensions (when implemented with conditions)
let pairs = {[x, y] for x in [1..3] for y in [1..3]};
// => [[1,1], [1,2], [1,3], [2,1], [2,2], [2,3], [3,1], [3,2], [3,3]]

// With string manipulation
let names = ["alice", "bob", "charlie"];
let greetings = {"Hello, " + name + "!" for name in names};
// => ["Hello, alice!", "Hello, bob!", "Hello, charlie!"]
```

#### List Comprehension Features
- **Expression Evaluation**: Any expression can be used as the output
- **Iteration**: Support for lists, ranges, and other iterable objects
- **MOO Syntax**: Uses `{}` instead of `[]` to distinguish from list literals
- **Lazy Evaluation**: Potential for generator-style comprehensions in the future

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

## MOO Compatibility Layer

### Overview
To support existing MOO databases like Cowbell/JHCore, Echo needs to implement a comprehensive compatibility layer that provides all the built-in functions, language features, and system behaviors expected by traditional MOO code.

### Phase 1: Core MOO Built-in Functions

#### Type System Functions
- **`typeof(value)`** - Returns type constant (INT, FLOAT, STR, LIST, OBJ, ERR)
- **`valid(object)`** - Checks if object reference is valid
- **`toint(value)`** - Convert to integer
- **`tofloat(value)`** - Convert to float  
- **`tostr(value...)`** - Convert to string with concatenation
- **`toobj(value)`** - Convert to object reference
- **`tolitter(value)`** - Convert to literal representation

#### Object System Functions
- **`create(parent [, owner])`** - Create new object
- **`parent(object)`** - Get object's parent
- **`children(object)`** - Get list of object's children
- **`chparent(object, new_parent)`** - Change object's parent
- **`max_object()`** - Get highest valid object number
- **`renumber(object)`** - Renumber an object
- **`recycle(object)`** - Recycle an object
- **`move(object, destination)`** - Move object to new location

#### Property Functions  
- **`properties(object)`** - List all properties on object
- **`property_info(object, prop_name)`** - Get property details
- **`add_property(object, prop_name, value, info)`** - Add new property
- **`delete_property(object, prop_name)`** - Remove property
- **`clear_property(object, prop_name)`** - Clear property value
- **`is_clear_property(object, prop_name)`** - Check if property is clear

#### Verb Functions
- **`verbs(object)`** - List all verbs on object
- **`verb_info(object, verb_name)`** - Get verb details  
- **`verb_args(object, verb_name)`** - Get verb argument spec
- **`verb_code(object, verb_name)`** - Get verb source code
- **`add_verb(object, info, args)`** - Add new verb
- **`delete_verb(object, verb_name)`** - Remove verb
- **`set_verb_info(object, verb_name, info)`** - Update verb info
- **`set_verb_args(object, verb_name, args)`** - Update verb args
- **`set_verb_code(object, verb_name, code)`** - Update verb code

#### Player Functions
- **`players()`** - Get list of all player objects
- **`is_player(object)`** - Check if object is a player
- **`set_player_flag(object, value)`** - Set/clear player flag
- **`connected_players()`** - Get list of connected players
- **`connected_seconds(player)`** - Get connection duration
- **`idle_seconds(player)`** - Get idle time
- **`connection_name(player)`** - Get connection info
- **`boot_player(player)`** - Disconnect a player
- **`notify(player, message)`** - Send message to player
- **`force_input(player, command)`** - Force player input

#### String Functions
- **`length(string)`** - Get string length
- **`strsub(subject, what, with [, case])`** - String substitution
- **`index(string, substring [, case])`** - Find substring
- **`rindex(string, substring [, case])`** - Reverse find
- **`strcmp(str1, str2)`** - Compare strings
- **`decode_binary(string)`** - Decode binary string
- **`encode_binary(string)`** - Encode as binary
- **`match(string, pattern [, case])`** - Pattern matching
- **`rmatch(string, pattern [, case])`** - Reverse pattern match
- **`substitute(string, subs)`** - Template substitution

#### List Functions
- **`length(list)`** - Get list length
- **`listappend(list, value [, index])`** - Add to list
- **`listinsert(list, value [, index])`** - Insert in list
- **`listdelete(list, index)`** - Remove from list
- **`listset(list, index, value)`** - Update list element
- **`setadd(list, value)`** - Add to set (unique)
- **`setremove(list, value)`** - Remove from set

#### Time Functions
- **`time()`** - Get current time
- **`ctime([time])`** - Convert time to string
- **`localtime([time])`** - Get local time components
- **`mktime(list)`** - Create time from components

#### System Functions
- **`server_version()`** - Get server version string
- **`server_log(message)`** - Write to server log
- **`shutdown([message])`** - Shutdown server
- **`dump_database()`** - Force database checkpoint
- **`db_disk_size()`** - Get database size
- **`memory_usage()`** - Get memory statistics
- **`queue_info([player])`** - Get task queue info
- **`force_task(player, verb_call)`** - Force task execution

#### Task Functions
- **`task_id()`** - Get current task ID
- **`suspend([seconds])`** - Suspend current task
- **`resume(task_id [, value])`** - Resume suspended task
- **`kill_task(task_id)`** - Kill a task
- **`tasks()`** - List all tasks
- **`task_stack(task_id)`** - Get task call stack
- **`set_task_perms(player)`** - Change task permissions
- **`caller_perms()`** - Get caller permissions

#### Security Functions
- **`caller()`** - Get calling object/player
- **`callers()`** - Get full call stack
- **`task_perms()`** - Get task permissions
- **`set_task_local(value)`** - Set task-local storage
- **`task_local()`** - Get task-local storage

### Phase 2: MOO Language Syntax Extensions

#### Verb Definition Syntax
```moo
@verb object:name this none this
@verb object:name any with/using any
```

#### Preposition Support
Built-in prepositions: with/using, at/to, in front of, in/inside/into, on top of/on/onto/upon, out of/from inside/from, over, through, under/underneath/beneath, behind, beside, for/about, is, as, off/off of

#### Error Constants (Case-Insensitive)
- `E_NONE`, `E_TYPE`, `E_DIV`, `E_PERM`, `E_PROPNF`
- `E_VERBNF`, `E_VARNF`, `E_INVIND`, `E_RECMOVE`, `E_MAXREC`
- `E_RANGE`, `E_ARGS`, `E_NACC`, `E_INVARG`, `E_QUOTA`, `E_FLOAT`

#### Type Constants (Case-Insensitive)  
- `INT`, `FLOAT`, `STR`, `LIST`, `OBJ`, `ERR`
- `NUM` (alias for INT for compatibility)

#### Special Variables in Verbs
- `this` - The object on which the verb is defined
- `caller` - The object that called this verb
- `player` - The player who typed the command
- `args` - List of all arguments
- `argstr` - Unparsed argument string
- `verb` - The name of this verb
- `dobjstr` - Direct object string
- `dobj` - Direct object
- `prepstr` - Preposition string  
- `iobjstr` - Indirect object string
- `iobj` - Indirect object

### Phase 3: System Integration

#### Login and Connection Handling
- **`$do_login_command()`** - Handle login commands
- **`$user_connected(player)`** - Called when player connects
- **`$user_disconnected(player)`** - Called when player disconnects
- **`$user_created(player)`** - Called when new player created
- **`$user_client_disconnected(player)`** - Called on client disconnect

#### Command Processing
- **`$server_started()`** - Called when server starts
- **`$do_command(player, command)`** - Override command processing
- **`$handle_uncaught_error(player, error)`** - Handle errors
- **`$verb_not_found(player, verb)`** - Handle missing verbs

#### Server Options ($server_options)
- Connection timeouts
- Task limits (ticks, seconds)
- Resource limits (stack depth, etc.)
- Security settings

### Phase 4: Advanced Features

#### Pattern Matching
- MOO-style pattern matching with wildcards
- `*` matches any text, `?` matches single word
- Pattern templates for verb argument parsing

#### Scattering Assignment
```moo
{first, second, @rest} = some_list;
{?name = "Anonymous", ?age = 0} = player_data;
```

#### Try-Finally Blocks
```moo
try
    risky_operation();
finally  
    cleanup();
endtry
```

#### Fork and Task Management
```moo
fork (5)
    player:tell("5 seconds have passed!");
endfork

fork task_id (0)
    background_processing();
endfork
```

### Implementation Priority

1. **Critical** (Required for basic MOO operation):
   - Type system functions (`typeof`, `valid`, type conversions)
   - Basic object functions (`create`, `parent`, `children`)
   - Property access functions
   - String and list manipulation
   - Player notification (`notify`, `tell`)

2. **Important** (Common MOO patterns):
   - Verb management functions
   - Player functions (`players`, `is_player`)
   - Time functions
   - Error handling
   - Task suspension

3. **Advanced** (Full compatibility):
   - Pattern matching
   - Fork/task management
   - Server administration
   - Advanced security features

### Testing Strategy
- Port LambdaCore test suite
- Test against Cowbell/JHCore database
- Verify verb execution and inheritance
- Test all built-in functions
- Validate error handling

## Grammar Restructuring Plan

Based on MOO grammar analysis, Echo's rust-sitter grammar should be restructured to:

### Phase 1: Core Structural Changes
1. **Separate Statement and Expression Enums**
   - Create distinct `Statement` and `Expression` types
   - Add `ExpressionStatement` wrapper for expressions used as statements
   - Improves error messages and parsing clarity

2. **Implement Precedence Constants**
   - Define MOO's precedence table as constants
   - Apply consistently across all operators
   - Fix current precedence inconsistencies

3. **Unify Pattern System**
   - Merge `ParamPattern` and `BindingPattern` into single `Pattern` type
   - Use same pattern structure for all contexts (parameters, bindings, destructuring)
   - Reduces code duplication and complexity

### Phase 2: Syntax Alignment
1. **Fix List Literal Syntax**
   - Ensure `{}` is used consistently for lists (not `[]`)
   - Aligns with MOO conventions

2. **Add Range Syntax**
   - Implement `[start..end]` for numeric ranges
   - Essential for list comprehensions and iterations

3. **List Comprehensions**
   - Add `{expr for var in iterable}` syntax
   - Frequently requested MOO feature

### Phase 3: Missing Features
1. **Control Flow Enhancements**
   - Add `elseif` clauses to if statements
   - Implement labeled loops for break/continue
   - Add fork statements for async execution

2. **Advanced Expressions**
   - Try expressions with `! codes` syntax
   - Pass expressions for parent verb calls
   - Map literals with `[key -> value]` syntax

3. **Operators**
   - Bitwise operators (`|.`, `&.`, `^.`, `<<`, `>>`)
   - `in` operator for membership testing
   - Complete MOO operator set

### Phase 4: Error Recovery
1. **Error Recovery Nodes**
   - Add graceful error recovery to grammar
   - Continue parsing after errors
   - Better error messages with context

2. **Field Names**
   - Add field names to all AST nodes
   - Provides context in error messages
   - Improves debugging experience

See RUST_SITTER_PATTERNS.md for detailed implementation patterns and CST_ALIGNMENT_PLAN.md for the complete migration strategy.

## Next Steps

1. **Implement Grammar Restructuring**: Start with Statement/Expression separation
2. **Add Missing Operators**: Complete MOO operator compatibility
3. **Implement MOO Built-ins**: Start with critical type and object functions
4. **Add Verb System**: Implement verb storage, lookup, and execution
5. **Login System**: Handle player connections and authentication  
6. **Import Existing MOO**: Test with Cowbell/JHCore database
7. **Performance Testing**: Benchmark against LambdaMOO

This roadmap provides a clear path from the current state to a fully-featured, production-ready MOO programming language with unique cooperative multithreading and persistent continuation capabilities.