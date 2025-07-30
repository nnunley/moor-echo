# Echo Language Grammars

Echo supports two distinct language grammars:

## 1. MOO Extended Language

The MOO extended language is used for:
- Object definitions and persistence
- Verb programming
- System-level operations
- File-based programs

### Features:
- Object-oriented programming with inheritance
- Verb definitions with permissions
- Property declarations
- System property access (`$property`)
- Object references (`#123`)
- Traditional MOO syntax compatibility

### Example:
```moo
object Player extends BaseObject
    property name = "Anonymous"
    property location = #0
    
    verb "greet" (this none this)
        player:tell("Hello, " + dobjstr + "!");
    endverb
endobject
```

## 2. REPL Language

The REPL (Read-Eval-Print Loop) language is used for:
- Interactive shell commands
- Quick expressions and calculations
- Immediate code execution
- Debugging and testing

### Features:
- Expression evaluation
- Variable bindings (`let`, `const`)
- Modern Echo syntax
- Lambda functions
- Event emission
- Async/await support

### Example:
```echo
let player = #100;
player.name = "Alice";
player:move(#42);
emit PlayerJoined { player: player, time: now() };
```

## Unified AST

Both grammars compile to the same unified AST structure (`ast/mod.rs`), ensuring:
- Consistent evaluation semantics
- Code interoperability
- Shared runtime behavior
- Common optimization paths

## Parser Architecture

```
┌─────────────────┐     ┌─────────────────┐
│  MOO Extended   │     │ REPL Language   │
│     Parser      │     │     Parser      │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
              ┌──────▼──────┐
              │ Unified AST │
              └──────┬──────┘
                     │
              ┌──────▼──────┐
              │  Evaluator  │
              └─────────────┘
```

## Implementation Status

- ✅ Unified AST implemented
- ✅ Simple parser for basic expressions
- 🚧 Full MOO parser (rust-sitter based)
- 🚧 Full REPL parser (rust-sitter based)
- ✅ Test coverage for simple parser