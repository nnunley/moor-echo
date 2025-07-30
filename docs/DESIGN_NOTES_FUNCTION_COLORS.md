# Design Notes: Function Colors and Echo's Future

## Current Situation

We've been adding different execution modes to Echo:

- `emit` for events
- `yield` for generators/coroutines (planned)
- `async`/`await` for async operations (considered)
- Regular imperative calls

This is leading us toward the "function coloring" problem where different types
of functions become incompatible.

## The Function Coloring Problem

```javascript
// Each type of function has a different "color"
async function fetch_data() { ... }
function* generate_items() { ... }
function emit_events() { emit something; }
function regular_function() { ... }

// Colors are viral - they propagate up the call stack
async function caller() {
    await fetch_data();      // Caller must be async
    yield* generate_items(); // Caller must be generator
    // Can't mix easily!
}
```

## Alternative Approach: Tuple Space Architecture

See [TUPLE_SPACE_ARCHITECTURE.md](./TUPLE_SPACE_ARCHITECTURE.md) for a detailed
exploration of an architecture that eliminates function colors by treating all
communication as tuple space operations.

### Key Benefits

- No function colors - everything is just tuple operations
- Natural distribution and RPC
- Capability-based security
- Unified execution model

### Example: Unified Syntax

```javascript
// All of these use the same underlying mechanism
object Example {
    verb process() {
        // Regular call
        result = other:method(args)

        // What would be async
        data = database:query(sql)

        // What would be generator
        for item in (source:items()) {
            process(item)
        }

        // Event emission
        emit thing_happened

        // Coroutine suspension
        yield

        // All just tuple operations under the hood!
    }
}
```

## Immediate Path Forward

For now, we'll continue with the current approach while keeping the tuple space
architecture as a potential future direction:

1. **Current Focus**:
   - Implement object-owned event handlers
   - Add Datalog queries
   - Complete the event system

2. **Design Principles**:
   - Minimize function coloring where possible
   - Keep syntax uniform
   - Don't lock ourselves into colored functions

3. **Future Consideration**:
   - Tuple space could be implemented as a runtime detail
   - Current syntax could map to tuple operations
   - Migration path exists if we decide to pivot

## Lessons for Current Development

Even if we don't implement tuple spaces now, the insights inform our design:

1. **Uniform Syntax**: Keep call syntax consistent regardless of operation type
2. **Avoid Viral Colors**: Don't require callers to know implementation details
3. **RPC Mindset**: Design as if objects could be remote
4. **Capability Thinking**: Consider permissions at communication boundaries

## Questions to Revisit

- Should `emit` require different syntax from method calls?
- Can we make `yield` work in any context without coloring?
- How do we handle async operations without `async`/`await`?
- What's the minimal set of primitives we need?

## Related Documents

- [TUPLE_SPACE_ARCHITECTURE.md](./TUPLE_SPACE_ARCHITECTURE.md) - Full
  exploration
- [LANGUAGE_ROADMAP.md](./LANGUAGE_ROADMAP.md) - Current implementation plan
- [COOPERATIVE_MULTITHREADING.md](./COOPERATIVE_MULTITHREADING.md) - Yield
  design

## Decision

For now, we'll proceed with the current design but remain mindful of function
coloring issues. The tuple space architecture remains a compelling future
direction that could solve multiple problems elegantly.
