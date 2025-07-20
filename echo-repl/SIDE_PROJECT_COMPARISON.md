# Side Project: Current vs Tuple Space Approach

## Quick Comparison

### Event Emission

**Current Approach:**
```javascript
// Special syntax for events
emit player_moved
emit damage_taken(amount)  // Not yet implemented

// In evaluator: special handling
case Emit { event_name, args } => {
    // Direct event system call
    event_system.emit(event_name, args)
}
```

**Tuple Space Approach:**
```javascript
// Same as any operation
emit player_moved
// Internally: put("*", "player_moved", [], broadcast=true)

// No special handling - just tuples
case any_operation => {
    put(target, operation, args)
}
```

### Method Calls

**Current Approach:**
```javascript
// Direct object method invocation
result = player:take_damage(10)

// In evaluator: lookup and call
case MethodCall { object, method, args } => {
    obj = lookup(object)
    method = obj.find_verb(method)
    method.call(args)
}
```

**Tuple Space Approach:**
```javascript
// Looks the same to programmer
result = player:take_damage(10)

// But internally: RPC through tuple space
put(player, "take_damage", [10], cont_123)
result = get(cont_123)
```

### Future: Generators

**Current Approach (Would Require Colors):**
```javascript
function* generate_items() {
    for i in (1..10) {
        yield i * 2
    }
}

// Caller must handle generator
for item in (generate_items()) {
    process(item)
}
```

**Tuple Space Approach (No Colors):**
```javascript
function generate_items() {
    for i in (1..10) {
        yield i * 2  // Just: put(caller, "yield", i * 2)
    }
}

// Caller doesn't know it's a generator
for item in (generate_items()) {
    process(item)
}
```

### Future: Async Operations

**Current Approach (Would Require Colors):**
```javascript
async function fetch_data() {
    return await database:query(sql)
}

// Caller must be async
async function use_data() {
    data = await fetch_data()
}
```

**Tuple Space Approach (No Colors):**
```javascript
function fetch_data() {
    return database:query(sql)  // Just tuples
}

// Caller doesn't know it's async
function use_data() {
    data = fetch_data()  // Runtime handles waiting
}
```

## Key Insight

The tuple space approach treats **everything** as message passing:
- No special syntax for different operation types
- No function coloring
- Uniform execution model
- Natural distribution

But it requires:
- Sophisticated runtime
- Performance optimization
- Good debugging tools
- Mental model shift

## Why This Matters

Even if we don't implement tuple spaces, thinking about communication this way helps us:
1. Avoid introducing unnecessary function colors
2. Design APIs that could work distributed
3. Keep syntax uniform
4. Think about capabilities and security

## Current Decision

Continue with the current approach but:
- Minimize special syntax
- Avoid viral function colors
- Design with future flexibility
- Keep tuple space as possible evolution

## Code We Should Avoid

```javascript
// BAD: Viral coloring
async emit player_moved  // Don't combine colors!

// BAD: Caller must know implementation
let data = await object:method()  // Forces caller to be async

// BAD: Incompatible worlds  
async function* stream_events() {  // Async generator nightmare
    while (true) {
        yield await get_event()
    }
}
```

## Code We Should Write

```javascript
// GOOD: Uniform syntax
emit player_moved
result = object:method()
for item in (source:items()) { }

// GOOD: Implementation details hidden
function process() {
    data = fetch_data()      // Might be async
    emit processing_done     // Might be queued
    yield                    // Might suspend
    // Caller doesn't need to know!
}
```

## Future Migration Path

If we decide to adopt tuple spaces:

1. Current syntax remains the same
2. Runtime implementation changes
3. Add tuple space primitives
4. Gradual migration of internals
5. Enable distribution features

The key is: **our current syntax shouldn't lock us into function colors**.