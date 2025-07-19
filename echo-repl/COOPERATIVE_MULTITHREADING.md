# Cooperative Multithreading and Persistent Continuations

## Overview

Echo language supports cooperative multithreading with persistent continuations that survive server restarts. This enables long-running MOO-style systems where players can have suspended execution that resumes across server lifecycle events.

## Architecture

### Execution Model

Players execute code cooperatively with automatic yield points and explicit suspend mechanisms:

- **Automatic Yield Points**: At the end of each loop iteration
- **Explicit Yield Points**: When calling builtin functions that return futures
- **Persistent Continuations**: Execution state can be serialized to disk and resumed

### Core Components

#### 1. Execution Context
```rust
#[derive(Serialize, Deserialize)]
pub struct ExecutionContext {
    player_id: ObjectId,
    ast_position: AstPosition,
    call_stack: Vec<StackFrame>,
    local_variables: HashMap<String, Value>,
    loop_state: Option<LoopState>,
    yield_reason: YieldReason,
    timestamp: SystemTime,
    ttl: Option<Duration>,
}
```

#### 2. AST Position Tracking
```rust
#[derive(Serialize, Deserialize)]
pub struct AstPosition {
    node_path: Vec<usize>,          // Path through AST tree
    statement_index: usize,         // Current statement in block
    expression_state: ExpressionState,
}

#[derive(Serialize, Deserialize)]
pub enum ExpressionState {
    NotStarted,
    InProgress(Vec<Value>),         // Intermediate results
    Completed(Value),
}
```

#### 3. Stack Frame Management
```rust
#[derive(Serialize, Deserialize)]
pub struct StackFrame {
    function_name: String,
    local_vars: HashMap<String, Value>,
    return_position: AstPosition,
    scope_type: ScopeType,
}

#[derive(Serialize, Deserialize)]
pub enum ScopeType {
    Function,
    Method,
    Loop,
    Block,
}
```

#### 4. Loop State Persistence
```rust
#[derive(Serialize, Deserialize)]
pub struct LoopState {
    loop_type: LoopType,
    iterator_position: IteratorPosition,
    loop_variables: HashMap<String, Value>,
    iteration_count: usize,
}

#[derive(Serialize, Deserialize)]
pub enum LoopType {
    For { collection: Value, variable: String },
    While { condition: Box<EchoAst> },
    ForEach { collection: Value, key_var: Option<String>, value_var: String },
}
```

### Yield Reasons

```rust
#[derive(Serialize, Deserialize)]
pub enum YieldReason {
    LoopYield,                      // End of loop iteration
    BuiltinCall(String, Vec<Value>), // Async builtin function
    TimeSlice,                      // CPU time slice exhausted
    Suspend(Duration),              // Explicit suspend call
    Wait(WaitCondition),            // Waiting for condition
}

#[derive(Serialize, Deserialize)]
pub enum WaitCondition {
    Time(SystemTime),
    PlayerAction(ObjectId),
    ObjectState(ObjectId, String),  // Object property change
    Event(String),
}
```

## Evaluator Integration

### Persistent Evaluator Trait

```rust
pub trait PersistentEvaluator: EvaluatorTrait {
    fn eval_resumable(&mut self, ast: &EchoAst, ctx: Option<ExecutionContext>) -> Result<EvaluationResult>;
    fn create_continuation(&self, player_id: ObjectId) -> Result<ExecutionContext>;
    fn resume_from_continuation(&mut self, ctx: ExecutionContext) -> Result<EvaluationResult>;
    fn should_yield(&self, player_id: ObjectId) -> bool;
}

pub enum EvaluationResult {
    Complete(Value),
    Yielded(ExecutionContext),
    Suspended(ExecutionContext, Duration),
    Waiting(ExecutionContext, WaitCondition),
}
```

### Evaluator Backend Support

- **Interpreter**: Full continuation support with AST position tracking
- **JIT (Cranelift)**: Limited support - falls back to interpreter for continuations
- **WebAssembly JIT**: Experimental support through WASM stack inspection

## Storage Layer

### Continuation Store

```rust
pub struct ContinuationStore {
    db: Arc<Storage>,
    pending_continuations: DashMap<ObjectId, ExecutionContext>,
    continuation_index: DashMap<String, Vec<ObjectId>>, // Index by wait condition
}

impl ContinuationStore {
    pub fn persist_continuation(&self, ctx: ExecutionContext) -> Result<ContinuationId>;
    pub fn resume_continuation(&self, player_id: ObjectId) -> Result<Option<ExecutionContext>>;
    pub fn cleanup_expired(&self) -> Result<Vec<ContinuationId>>;
    pub fn find_waiting_for(&self, condition: &WaitCondition) -> Result<Vec<ObjectId>>;
    pub fn cancel_continuation(&self, player_id: ObjectId) -> Result<()>;
}
```

### Serialization Format

Uses `bincode` for efficient binary serialization of execution contexts:

```rust
// Continuation storage key format
const CONTINUATION_KEY_PREFIX: &str = "cont:";
const CONTINUATION_INDEX_PREFIX: &str = "cont_idx:";

// Storage layout:
// cont:{player_id} -> ExecutionContext
// cont_idx:time:{timestamp} -> Vec<ObjectId>
// cont_idx:wait:{condition_hash} -> Vec<ObjectId>
```

## Language Syntax

### Yield Points

```javascript
// Automatic yield at end of loop iterations
for (item in collection) {
    // ... process item ...
    // <- automatic yield point here
}

while (condition) {
    // ... loop body ...
    // <- automatic yield point here
}

// Explicit yield/suspend
suspend(5000);  // Suspend for 5 seconds
yield;          // Yield to other players immediately

// Builtin functions that return futures
let result = await http_get("https://example.com");
let data = await database_query("SELECT * FROM users");

// Lambda functions for async callbacks
let urls = ["https://api1.com", "https://api2.com", "https://api3.com"];
let promises = urls.map((url) => http_get(url));
let results = await Promise.all(promises);

// Event handling with lambdas
on_player_move((player, old_location, new_location) => {
    broadcast(`${player.name} moved from ${old_location} to ${new_location}`);
});
```

### Wait Conditions

```javascript
// Wait for specific time
wait_until(time("2024-01-01T00:00:00Z"));

// Wait for player action
wait_for_player(player_id, "move");

// Wait for object state change
wait_for_property(object_id, "status", "ready");

// Wait for custom event
wait_for_event("server_restart");
```

## Scheduler Integration

### Cooperative Scheduler

```rust
pub struct CooperativeScheduler {
    active_players: VecDeque<ObjectId>,
    continuation_store: ContinuationStore,
    time_slice_duration: Duration,
    max_iterations_per_slice: usize,
}

impl CooperativeScheduler {
    pub fn schedule_player(&mut self, player_id: ObjectId, priority: Priority);
    pub fn yield_player(&mut self, player_id: ObjectId, reason: YieldReason);
    pub fn resume_waiting_continuations(&mut self, condition: &WaitCondition);
    pub fn cleanup_expired_continuations(&mut self);
    pub fn run_scheduler_tick(&mut self) -> Result<SchedulerStats>;
}
```

### Time Slice Management

- **Default Time Slice**: 10ms per player
- **Max Iterations**: 1000 operations per slice
- **Yield Frequency**: Every loop iteration, function call, or builtin operation
- **Priority Scheduling**: Player priority affects scheduling frequency

## Server Restart Recovery

### Continuation Recovery Process

1. **Server Startup**: Load all persisted continuations from storage
2. **Validation**: Verify continuation integrity and TTL
3. **Scheduling**: Add valid continuations to scheduler queue
4. **Cleanup**: Remove expired or invalid continuations

### Recovery Implementation

```rust
impl CooperativeScheduler {
    pub fn recover_from_restart(&mut self) -> Result<RecoveryStats> {
        let continuations = self.continuation_store.load_all_continuations()?;
        let mut stats = RecoveryStats::new();
        
        for ctx in continuations {
            if self.is_valid_continuation(&ctx) {
                self.schedule_continuation(ctx)?;
                stats.recovered += 1;
            } else {
                self.continuation_store.cleanup_continuation(ctx.player_id)?;
                stats.expired += 1;
            }
        }
        
        Ok(stats)
    }
}
```

## Performance Considerations

### Memory Management

- **Continuation Pooling**: Reuse ExecutionContext objects
- **Incremental GC**: Clean up expired continuations periodically
- **Memory Limits**: Limit total continuation memory usage
- **Compression**: Compress large continuation states

### Optimization Strategies

- **AST Caching**: Cache compiled AST nodes for faster resume
- **Position Optimization**: Optimize AST position representation
- **Lazy Serialization**: Only serialize when necessary for persistence
- **Batch Operations**: Group continuation operations for efficiency

## Future Enhancements

### Distributed Continuations

- **Cross-Server Migration**: Move continuations between server instances
- **Load Balancing**: Balance continuation load across servers
- **Replication**: Replicate critical continuations for fault tolerance

### Advanced Scheduling

- **Priority Scheduling**: Advanced priority algorithms
- **Fair Scheduling**: Ensure all players get fair execution time
- **Adaptive Time Slicing**: Dynamic time slice adjustment
- **Resource Awareness**: Schedule based on resource usage

### Integration Features

- **Debugger Support**: Debug suspended continuations
- **Metrics Collection**: Track continuation performance
- **Admin Tools**: Manage continuations through admin interface
- **Monitoring**: Real-time continuation monitoring and alerting

## Implementation Phases

### Phase 1: Basic Cooperative Multithreading
- **✅ Partial**: Implement basic yield points in loops (TODO comments in evaluator)
- **Pending**: Add simple scheduler with time slicing
- **Pending**: Support for basic suspend/resume
- **Prerequisites**: 
  - Function definitions and calls must be implemented first
  - Lambda/anonymous functions recommended for async callbacks

### Phase 2: Persistent Continuations
- Add ExecutionContext serialization
- Implement continuation storage layer
- Support server restart recovery

### Phase 3: Advanced Features
- Add wait conditions and event-driven resumption
- Implement advanced scheduling algorithms
- Add distributed continuation support

### Phase 4: Production Readiness
- Performance optimization and tuning
- Comprehensive testing and validation
- Documentation and tooling
- Integration with existing MOO systems