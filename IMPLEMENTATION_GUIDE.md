# Echo Language Implementation Guide for Junior Programmers

## Overview

This document provides a comprehensive guide for implementing the remaining Echo
language features. Echo is an Event-Centered Hybrid Objects language that
extends MOO with modern programming constructs including events, queries,
capabilities, and transparent state persistence.

## Current Implementation Status

### ✅ Completed Features

**Core Language Infrastructure**:

- rust-sitter grammar with comprehensive AST support
- Basic evaluator with variable scoping
- Storage layer with Sled database integration
- Multi-player execution environments
- JIT compilation (Cranelift + WebAssembly)

**Language Constructs**:

- Literals (numbers, strings, booleans)
- Variables and assignments (let, const)
- Arithmetic and property access
- Control flow (if/else, loops, break/continue)
- Exception handling (try/catch/finally, throw)
- Objects (definitions, properties, verbs, method calls)
- Functions (definitions, calls, parameters, local scope, recursion)
- Scatter assignment/destructuring (`let {a, ?b = default, @rest} = object`)

### 🎯 Next Implementation Priorities

1. **Meta-Object Protocol (MOP) & Concurrency** (Critical Priority)
2. **Object-Owned Events** (High Priority)
3. **Object-Owned Queries** (High Priority)
4. **$lobby System Object** (High Priority)
5. **Lambda/Anonymous Functions** (Medium Priority)
6. **State Persistence** (High Priority)
7. **Capabilities/Permissions** (High Priority)

## Implementation Roadmap

### Phase 0: Meta-Object Protocol (MOP) & Concurrency (Estimated: 3-4 weeks)

This phase establishes the foundational reflection system and the concurrency
model, crucial for debugging, persistence, and dynamic optimization.

#### Step 0.1: Implement Core Meta-Object Structure (1 week)

**File**: `src/evaluator/meta_object.rs` (new file)

Define the `MetaObject` structure and ensure every object has a read-only
`$meta` property pointing to it.

```rust
#[derive(Debug, Clone)]
pub struct MetaObject {
    pub object_id: ObjectId,
    // Add fields for properties, verbs, events, queries, capabilities, etc.
    // These will be populated as other features are implemented.
}

impl EchoObject { // Assuming an EchoObject struct exists for runtime objects
    pub fn get_meta(&self) -> &MetaObject {
        // Return a reference to the object's MetaObject
    }
}
```

#### Step 0.2: Design & Implement Concurrency Model (2-3 weeks)

**File**: `src/runtime/scheduler.rs` (new file) **File**:
`src/runtime/supervisor.rs` (new file)

Implement a cooperative multitasking scheduler using green threads for in-game
concurrency. Each player connection will run in its own true thread. Introduce a
Supervisor tree for fault tolerance and process resumption.

```rust
// src/runtime/scheduler.rs
pub struct Scheduler {
    // Manages green threads (tasks/continuations)
}

impl Scheduler {
    pub fn spawn_green_thread<F>(&mut self, f: F) -> GreenThreadId
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        // Spawns a new green thread
    }
    // Add methods for yielding, sleeping, implicit await-on-use
}

// src/runtime/supervisor.rs
pub struct Supervisor {
    // Manages the hierarchy of tasks and their states
    // Handles crashes, restarts, and process resumption
}

impl Supervisor {
    pub fn supervise_task(&mut self, task_id: GreenThreadId, parent_id: Option<GreenThreadId>) {
        // Register a task under supervision
    }
    pub fn handle_crash(&mut self, task_id: GreenThreadId, error: String) {
        // Implement crash recovery logic based on supervisor strategy
    }
    // Add methods for saving/loading task state for persistence
}
```

#### Step 0.3: Integrate MOP with Concurrency (1 week)

**File**: `src/evaluator/mod.rs` **File**: `src/evaluator/meta_object.rs`

Extend the `MetaObject` to expose information about concurrent tasks associated
with an object (e.g., forked tasks, active verbs).

```rust
// src/evaluator/meta_object.rs (updated)
#[derive(Debug, Clone)]
pub struct MetaObject {
    // ... existing fields ...
    pub active_tasks: Vec<GreenThreadId>, // List of tasks associated with this object
}

// src/evaluator/mod.rs (updates to evaluator logic)
impl EchoEvaluator {
    // Modify fork statement evaluation to register tasks with Supervisor
    // Update verb/function calls to associate with current task
}
```

### Phase 1: Event System & Error Handling (Estimated: 3-4 weeks)

Events are the reactive backbone of Echo, now including a unified error handling
mechanism. All events are owned by objects and can be handled by any object that
subscribes to them. Events will support inheritance.

#### Step 1.1: Add Event Grammar (3-4 days)

**File**: `src/parser/grammar.rs`

Add new AST variants:

```rust
// Event definitions within objects
EventDefinition {
    #[rust_sitter::leaf(text = "event")]
    _event: (),
    name: Box<EchoAst>,
    #[rust_sitter::leaf(text = "extends")]
    _extends: Option<Box<EchoAst>>, // Optional: for event inheritance
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    parameters: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    body: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
},

// Event handlers within objects
EventHandler {
    #[rust_sitter::leaf(text = "on")]
    _on: (),
    event_pattern: Box<EchoAst>, // This will now support event names and their descendants
    #[rust_sitter::leaf(text = "where")]
    _where: (),
    condition: Option<Box<EchoAst>>,
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    body: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
},

// Event emission
EventEmit {
    #[rust_sitter::leaf(text = "emit")]
    _emit: (),
    event_name: Box<EchoAst>,
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    arguments: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
    #[rust_sitter::leaf(text = ";")]
    _semicolon: (),
},
```

**Update ObjectDefinition** to include events:

```rust
ObjectDefinition {
    // ... existing fields ...
    events: Vec<EchoAst>,      // EventDefinition nodes
    handlers: Vec<EchoAst>,    // EventHandler nodes
},
```

#### Step 1.2: Implement Event Evaluation & Error Handling (5-7 days)

**File**: `src/evaluator/mod.rs` **File**: `src/evaluator/events.rs` (new file
for event-specific logic)

Add event system types, including support for event inheritance and a base
`Error` event.

```rust
// src/evaluator/events.rs
#[derive(Debug, Clone, PartialEq)]
pub struct EventDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub owner_object: ObjectId,
    pub parent_event: Option<String>, // For event inheritance
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventHandler {
    pub event_pattern: String, // Can be an event name or a parent event name
    pub condition: Option<EchoAst>,
    pub body: Vec<EchoAst>,
    pub owner_object: ObjectId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventInstance {
    pub event_name: String,
    pub arguments: HashMap<String, Value>,
    pub emitter: ObjectId,
    pub timestamp: u64,
}

// Define a base Error event and its descendants for system errors
pub const ERROR_EVENT_NAME: &str = "Error";
// Other specific error events like "PermissionError" will extend "Error"

// src/evaluator/mod.rs
pub struct EchoEvaluator {
    // ... existing fields ...
    event_definitions: HashMap<String, EventDefinition>,
    event_handlers: Vec<EventHandler>,
    event_queue: VecDeque<EventInstance>,
}

impl EchoEvaluator {
    fn evaluate_event_definition(&mut self, event_def: &EchoAst) -> Result<(), String> {
        // Register event definition with owner object, including parent event
        // Store in event_definitions registry
    }

    fn evaluate_event_handler(&mut self, handler: &EchoAst) -> Result<(), String> {
        // Register event handler with pattern matching, considering event inheritance
        // Store in event_handlers registry
    }

    fn evaluate_event_emit(&mut self, emit: &EchoAst) -> Result<Value, String> {
        // Create EventInstance
        // Add to event_queue for processing
        // Return immediately (non-blocking)
    }

    fn process_event_queue(&mut self) -> Result<(), String> {
        // Process all queued events
        // Match against registered handlers, traversing event inheritance hierarchy
        // Execute handler bodies in new scope
    }

    // New: Handle errors by emitting Error events
    fn handle_error(&mut self, error_type: String, message: String, traceback: Vec<CallFrame>) {
        let error_event = EventInstance {
            event_name: error_type,
            arguments: HashMap::from([
                ("message".to_string(), Value::String(message)),
                ("traceback".to_string(), Value::List(traceback.into_iter().map(Value::from).collect())),
            ]),
            emitter: self.current_object_id(), // Or relevant context
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        self.event_queue.push_back(error_event);
    }
}
```

#### Step 1.3: Add Event & Error Handling Tests (2-3 days)

**File**: `tests/event_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_definition() {
        let code = r#"
        object TestObject {
            event PlayerMoved(player, from_room, to_room) {
                // Event definition within object
            }
        }
        "#;

        let mut evaluator = EchoEvaluator::new();
        let result = evaluator.evaluate_string(code);
        assert!(result.is_ok());

        // Verify event is registered
        assert!(evaluator.event_definitions.contains_key("PlayerMoved"));
    }

    #[test]
    fn test_event_inheritance() {
        let code = r#"
        object TestObject {
            event CustomError extends Error(message) {}
            on CustomError(msg) {
                this:log("Custom error occurred: " + msg);
            }
            on Error(msg) {
                this:log("Generic error caught: " + msg);
            }
            function cause_error() {
                throw CustomError("Something went wrong!");
            }
        }
        "#;

        let mut evaluator = EchoEvaluator::new();
        let result = evaluator.evaluate_string(code);
        assert!(result.is_ok());
        // Test that CustomError handler is triggered, and potentially Error handler too
    }

    #[test]
    fn test_event_handler() {
        let code = r#"
        object TestObject {
            on PlayerMoved(player, from, to) where to == this.location {
                this:announce(player.name + " arrives.");
            }
        }
        "#;

        let mut evaluator = EchoEvaluator::new();
        let result = evaluator.evaluate_string(code);
        assert!(result.is_ok());

        // Verify handler is registered
        assert_eq!(evaluator.event_handlers.len(), 1);
    }

    #[test]
    fn test_event_emission() {
        let code = r#"
        object TestObject {
            function test_emit() {
                emit PlayerMoved(this, #123, #456);
            }
        }
        "#;

        let mut evaluator = EchoEvaluator::new();
        let result = evaluator.evaluate_string(code);
        assert!(result.is_ok());

        // Test that event is queued
        // Test that handlers are triggered
    }
}
```

### Phase 2: Datalog Query System (Estimated: 3-4 weeks)

Queries provide declarative logic capabilities using Datalog-style syntax with
Horn clauses.

#### Step 2.1: Add Query Grammar (4-5 days)

**File**: `src/parser/grammar.rs`

Add query AST variants:

```rust
// Query definitions within objects
QueryDefinition {
    #[rust_sitter::leaf(text = "query")]
    _query: (),
    head: Box<EchoAst>,
    #[rust_sitter::leaf(text = ":-")]
    _rule: (),
    body: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ";")]
    _semicolon: (),
},

// Query head (predicate with arguments)
QueryHead {
    name: Box<EchoAst>,
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    arguments: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
},

// Query body clauses
QueryClause {
    predicate: Box<EchoAst>,
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    arguments: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
},

// Query execution
QueryExecution {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ":")]
    _colon: (),
    query: Box<EchoAst>,
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    arguments: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
},
```

#### Step 2.2: Implement Query Engine (7-10 days)

**File**: `src/evaluator/query_engine.rs`

Create a basic Datalog engine:

```rust
#[derive(Debug, Clone)]
pub struct QueryEngine {
    facts: HashMap<String, Vec<Vec<Value>>>,
    rules: HashMap<String, Vec<QueryRule>>,
}

#[derive(Debug, Clone)]
pub struct QueryRule {
    pub head: QueryHead,
    pub body: Vec<QueryClause>,
    pub owner_object: ObjectId,
}

impl QueryEngine {
    pub fn new() -> Self {
        QueryEngine {
            facts: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    pub fn add_fact(&mut self, predicate: &str, args: Vec<Value>) {
        self.facts.entry(predicate.to_string())
            .or_insert_with(Vec::new)
            .push(args);
    }

    pub fn add_rule(&mut self, rule: QueryRule) {
        self.rules.entry(rule.head.name.clone())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    pub fn query(&self, predicate: &str, args: &[Value]) -> Vec<HashMap<String, Value>> {
        // Implement basic SLD resolution
        // 1. Try to match facts directly
        // 2. Try to match rules and recurse
        // 3. Return all successful bindings
    }
}
```

#### Step 2.3: Integrate with Evaluator (3-4 days)

**File**: `src/evaluator/mod.rs`

Add query support to evaluator:

```rust
impl EchoEvaluator {
    fn evaluate_query_definition(&mut self, query: &EchoAst) -> Result<(), String> {
        // Parse query rule
        // Register with query engine
        // Associate with owner object
    }

    fn evaluate_query_execution(&mut self, query: &EchoAst) -> Result<Value, String> {
        // Execute query through engine
        // Return results as list of bindings
    }
}
```

### Phase 3: $lobby System Object (Estimated: 2-3 weeks)

The $lobby object serves as the system root, handling global operations and
acting as a discovery path for key root objects.

#### Step 3.1: Implement $lobby Object (5-7 days)

**File**: `src/evaluator/system_objects.rs`

Create system object infrastructure:

```rust
#[derive(Debug, Clone)]
pub struct SystemObject {
    pub id: ObjectId,
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub functions: HashMap<String, FunctionValue>,
    pub verbs: HashMap<String, VerbValue>,
    pub queries: HashMap<String, QueryRule>,
}

impl SystemObject {
    pub fn create_lobby() -> Self {
        let mut lobby = SystemObject {
            id: ObjectId::new(0), // Special ID for $lobby
            name: "$lobby".to_string(),
            properties: HashMap::new(),
            functions: HashMap::new(),
            verbs: HashMap::new(),
            queries: HashMap::new(),
        };

        // Add system properties
        lobby.properties.insert("server_name".to_string(),
            Value::String("Echo MOO Server".to_string()));
        lobby.properties.insert("start_time".to_string(),
            Value::Number(chrono::Utc::now().timestamp() as f64));

        // Add system functions
        lobby.functions.insert("find_player".to_string(),
            FunctionValue::builtin("find_player"));
        lobby.functions.insert("broadcast".to_string(),
            FunctionValue::builtin("broadcast"));

        lobby
    }
}
```

### Phase 4: Lambda Functions (Estimated: 2-3 weeks)

Lambda functions provide first-class function support with closure capabilities.

Lambda functions provide first-class function support with closure capabilities.

#### Step 4.1: Add Lambda Grammar (3-4 days)

**File**: `src/parser/grammar.rs`

Add lambda expression support:

```rust
LambdaExpression {
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    parameters: Vec<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
    #[rust_sitter::leaf(text = "=>")]
    _arrow: (),
    body: Box<EchoAst>,
},

// Support scatter assignment for lambda declarations
LambdaAssignment {
    #[rust_sitter::leaf(text = "let")]
    _let: (),
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    names: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
    #[rust_sitter::leaf(text = "=")]
    _equals: (),
    #[rust_sitter::leaf(text = "{")]
    _lbrace2: (),
    lambdas: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace2: (),
    #[rust_sitter::leaf(text = ";")]
    _semicolon: (),
},
```

#### Step 4.2: Implement Lambda Evaluation (4-5 days)

**File**: `src/evaluator/mod.rs`

Add lambda support:

```rust
#[derive(Debug, Clone)]
pub struct LambdaValue {
    pub parameters: Vec<String>,
    pub body: EchoAst,
    pub closure: HashMap<String, Value>, // Captured variables
}

impl EchoEvaluator {
    fn evaluate_lambda_expression(&mut self, lambda: &EchoAst) -> Result<Value, String> {
        // Create LambdaValue with current scope as closure
        // Return as first-class value
    }

    fn call_lambda(&mut self, lambda: &LambdaValue, args: &[Value]) -> Result<Value, String> {
        // Create new scope with closure + parameters
        // Evaluate body in new scope
        // Return result
    }
}
```

### Phase 5: State Persistence (Estimated: 3-4 weeks)

Implement transparent state persistence for all game objects, functions, events,
and continuations.

#### Step 5.1: Design Persistent Storage Schema (3-4 days)

**File**: `src/storage/schema.rs`

Design comprehensive storage schema:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentGameState {
    pub objects: HashMap<ObjectId, PersistentObject>,
    pub functions: HashMap<String, PersistentFunction>,
    pub events: HashMap<String, PersistentEvent>,
    pub queries: HashMap<String, PersistentQuery>,
    pub continuations: HashMap<ContinuationId, PersistentContinuation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentObject {
    pub id: ObjectId,
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub functions: HashMap<String, FunctionValue>,
    pub verbs: HashMap<String, VerbValue>,
    pub events: HashMap<String, EventDefinition>,
    pub handlers: Vec<EventHandler>,
    pub queries: HashMap<String, QueryRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentContinuation {
    pub id: ContinuationId,
    pub call_stack: Vec<CallFrame>,
    pub local_variables: HashMap<String, Value>,
    pub program_counter: usize,
    pub suspended_at: u64,
}
```

#### Step 5.2: Implement Transparent Persistence (7-10 days)

**File**: `src/storage/persistent_storage.rs`

Implement automatic state persistence:

```rust
pub struct PersistentStorage {
    db: sled::Db,
    auto_save_interval: Duration,
    last_save: Instant,
}

impl PersistentStorage {
    pub fn save_game_state(&self, state: &PersistentGameState) -> Result<(), String> {
        // Serialize entire game state
        // Save to Sled database
        // Handle errors gracefully
    }

    pub fn load_game_state(&self) -> Result<PersistentGameState, String> {
        // Load from Sled database
        // Deserialize game state
        // Handle missing or corrupted data
    }

    pub fn auto_save_if_needed(&mut self, state: &PersistentGameState) -> Result<(), String> {
        // Check if auto-save interval has passed
        // Save state if needed
        // Update last_save timestamp
    }
}
```

### Phase 6: Capabilities/Permissions (Estimated: 2-3 weeks)

Implement fine-grained capability-based security system.

#### Step 6.1: Design Capability System (3-4 days)

**File**: `src/security/capabilities.rs`

Design capability-based security:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    ReadProperty(ObjectId, String),
    WriteProperty(ObjectId, String),
    CallFunction(ObjectId, String),
    CallVerb(ObjectId, String),
    EmitEvent(String),
    ExecuteQuery(ObjectId, String),
    AccessRoom(ObjectId),
    ModifyHealth(ObjectId),
    SystemAccess(String),
}

pub struct CapabilityManager {
    grants: HashMap<ObjectId, HashSet<Capability>>,
    denials: HashMap<ObjectId, HashSet<Capability>>,
}

impl CapabilityManager {
    pub fn check_capability(&self, subject: ObjectId, capability: &Capability) -> bool {
        // Check if explicitly denied
        if let Some(denials) = self.denials.get(&subject) {
            if denials.contains(capability) {
                return false;
            }
        }

        // Check if explicitly granted
        if let Some(grants) = self.grants.get(&subject) {
            if grants.contains(capability) {
                return true;
            }
        }

        // Default deny
        false
    }
}
```

#### Step 6.2: Integrate Security Checks (4-5 days)

**File**: `src/evaluator/mod.rs`

Add security checks to all operations:

```rust
impl EchoEvaluator {
    fn evaluate_property_access(&mut self, access: &EchoAst) -> Result<Value, String> {
        // Extract object and property
        // Check ReadProperty capability
        // Proceed with access if authorized
    }

    fn evaluate_function_call(&mut self, call: &EchoAst) -> Result<Value, String> {
        // Check CallFunction capability
        // Proceed with call if authorized
    }

    fn evaluate_verb_call(&mut self, call: &EchoAst) -> Result<Value, String> {
        // Check CallVerb capability
        // Proceed with call if authorized
    }
}
```

## Testing Strategy

### Unit Tests

- Test each component in isolation
- Verify grammar parsing for all new constructs
- Test evaluation logic for all new features
- Validate error handling and edge cases

### Integration Tests

- Test complete workflows end-to-end
- Verify object-event-query interactions
- Test persistence and recovery scenarios
- Validate security policy enforcement

### Performance Tests

- Benchmark event processing throughput
- Test query performance with large datasets
- Validate memory usage under load
- Test persistence performance

## Development Environment Setup

### Prerequisites

- Rust 1.70+ with Cargo
- rust-sitter dependencies
- Sled database
- Test frameworks (tokio-test, criterion)

### Build Process

1. `cargo build` - Build all components
2. `cargo test` - Run all tests
3. `cargo bench` - Run performance benchmarks
4. `cargo check` - Check for compilation errors

### Debugging

- Use `RUST_LOG=debug` for detailed logging
- Enable `trace` level for evaluator debugging
- Use `cargo test -- --nocapture` for test debugging

## Implementation Tips for Junior Programmers

### 1. Start Small

- Implement each feature incrementally
- Write tests before implementation
- Get basic functionality working before optimization

### 2. Follow Existing Patterns

- Study how functions are implemented for reference
- Use similar error handling patterns
- Follow the same naming conventions

### 3. Test Thoroughly

- Write unit tests for each new function
- Test error conditions and edge cases
- Use integration tests for complex workflows

### 4. Document as You Go

- Add comments explaining complex logic
- Update this guide with new discoveries
- Document any design decisions

### 5. Ask for Help

- Don't hesitate to ask questions
- Use existing code as examples
- Leverage the Rust community for language-specific help

## Common Pitfalls to Avoid

### 1. Grammar Conflicts

- Be careful with rust-sitter precedence
- Test grammar changes thoroughly
- Use `conflicts_with` attribute when needed

### 2. Memory Management

- Be mindful of circular references
- Use `Rc<RefCell<>>` appropriately
- Consider performance implications

### 3. Error Handling

- Always handle `Result` types properly
- Provide meaningful error messages
- Don't panic in production code

### 4. Threading Issues

- Be careful with shared state
- Use appropriate synchronization primitives
- Test concurrent scenarios

## Success Metrics

### Phase 1 (Events)

- [ ] All event syntax parses correctly
- [ ] Event handlers execute properly
- [ ] Event emission works end-to-end
- [ ] 95%+ test coverage

### Phase 2 (Queries)

- [ ] Basic Datalog queries work
- [ ] Query results are accurate
- [ ] Performance is acceptable
- [ ] Integration with objects complete

### Phase 3 ($lobby)

- [ ] $ syntax resolves correctly
- [ ] System objects are accessible
- [ ] Global operations work
- [ ] Backward compatibility maintained

### Phase 4 (Lambdas)

- [ ] Lambda expressions evaluate correctly
- [ ] Closures capture variables properly
- [ ] First-class function support works
- [ ] Scatter assignment integration complete

### Phase 5 (Persistence)

- [ ] All state persists correctly
- [ ] Recovery from disk works
- [ ] Performance impact is minimal
- [ ] Data integrity is maintained

### Phase 6 (Security)

- [ ] All operations are secured
- [ ] Capability checks work correctly
- [ ] Policy enforcement is consistent
- [ ] No security vulnerabilities exist

## Conclusion

This implementation guide provides a comprehensive roadmap for completing the
Echo language. By following these phases sequentially and maintaining high code
quality standards, you'll create a robust, secure, and performant language
implementation.

Remember: Echo is designed to be a cohesive evolution of MOO that maintains
backward compatibility while adding powerful modern features. Each
implementation decision should support both beginner-friendly learning and
advanced programming capabilities.

The key to success is incremental progress with thorough testing at each step.
Don't rush - build solid foundations that will support the sophisticated
features that make Echo unique.
