# Parser Architecture

## Overview

The Echo parser uses rust-sitter, a Rust procedural macro system that generates parsers from annotated AST definitions. This document explains the architecture and design decisions.

## Key Concepts

### rust-sitter Integration

rust-sitter works by **annotating AST definitions** with parsing instructions. The AST and grammar are not separate - they are defined together:

```rust
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[derive(Debug, PartialEq)]
    #[rust_sitter::language]
    pub enum EchoAst {
        // The AST node IS the grammar rule
        #[rust_sitter::leaf(text = "true")]
        True,
        
        // Pattern matching with transformation
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i64),
        
        // Precedence and associativity
        #[rust_sitter::prec_left(7)]
        Add {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "+")]
            _op: (),
            right: Box<EchoAst>,
        },
    }
}
```

### Two-AST Architecture

The codebase has two AST representations for good reasons:

1. **Parser AST** (`parser/echo/grammar.rs`)
   - Tightly coupled to rust-sitter
   - Contains parsing annotations
   - Defines grammar rules through structure
   - Parser-specific concerns (precedence, associativity)

2. **Unified AST** (`ast/mod.rs`)
   - Clean AST without parser annotations
   - Used by evaluator and other components
   - Parser-agnostic representation
   - Can be targeted by multiple parsers

### Architecture Flow

```
┌─────────────────────┐
│   Source Code       │
└──────────┬──────────┘
           │
    ┌──────▼──────────┐
    │  rust-sitter    │
    │  Parser         │
    │  (grammar.rs)   │
    └──────┬──────────┘
           │
    ┌──────▼──────────┐
    │  rust-sitter    │
    │  AST Instances  │
    └──────┬──────────┘
           │
    ┌──────▼──────────┐
    │  AST Converter  │
    │  (echo/mod.rs)  │
    └──────┬──────────┘
           │
    ┌──────▼──────────┐
    │  Unified AST    │
    │  (ast/mod.rs)   │
    └──────┬──────────┘
           │
    ┌──────▼──────────┐
    │   Evaluator     │
    └─────────────────┘
```

## Design Benefits

### Separation of Concerns
- **Parser AST**: Focused on parsing concerns (grammar, precedence, tokens)
- **Unified AST**: Focused on semantic representation
- **Converter**: Handles the mapping between them

### Multiple Parser Support
The unified AST can be targeted by:
- rust-sitter Echo parser (main implementation)
- MOO compatibility parser (future)
- Simple parser (temporary/testing)
- REPL-specific parser extensions

### Clean Interfaces
- Evaluator doesn't need to know about parsing details
- Parser doesn't need to know about evaluation
- AST transformations are explicit in the converter

## Implementation Details

### Parser AST (grammar.rs)
```rust
// Tightly coupled to parsing
#[rust_sitter::prec_left(7)]
Add {
    left: Box<EchoAst>,
    #[rust_sitter::leaf(text = "+")]
    _op: (),  // Parser needs this token
    right: Box<EchoAst>,
}
```

### Unified AST (ast/mod.rs)
```rust
// Clean semantic representation
Add {
    left: Box<EchoAst>,
    right: Box<EchoAst>,
    // No parser-specific fields
}
```

### Converter (echo/mod.rs)
```rust
// Explicit conversion
match grammar_ast {
    G::Add { left, right, .. } => A::Add {
        left: Box::new(convert_grammar_to_ast(*left)?),
        right: Box::new(convert_grammar_to_ast(*right)?),
    },
    // ...
}
```

## Why Not a Single AST?

1. **rust-sitter constraints**: The parser AST must include parsing-specific annotations and fields
2. **Evaluator simplicity**: The evaluator shouldn't deal with parser tokens or annotations
3. **Multiple parsers**: Different parsers (MOO, REPL) may have different parsing needs but target the same semantic AST
4. **Evolution**: Parser and evaluator can evolve independently

## Best Practices

### When to Modify Parser AST
- Adding new syntax elements
- Changing parsing precedence/associativity
- Fixing parsing ambiguities
- Adding parser-specific optimizations

### When to Modify Unified AST
- Adding new language features
- Changing semantic representation
- Optimizing evaluation
- Adding type information

### When to Modify Converter
- After any change to either AST
- Adding validation during conversion
- Implementing syntactic sugar expansion
- Error recovery and reporting

## Common Patterns

### Optional Syntax Elements
Parser AST includes optional tokens that don't affect semantics:
```rust
// Parser AST - needs the semicolon token
ExpressionStatement {
    expr: Box<EchoAst>,
    #[rust_sitter::leaf(text = ";", optional = true)]
    _semicolon: Option<()>,
}

// Unified AST - just the expression
ExpressionStatement(Box<EchoAst>)
```

### Syntactic Sugar
Parser accepts multiple syntaxes that map to same semantic meaning:
```rust
// Parser AST - both `=` and `:=` for assignment
// Unified AST - single Assignment variant
```

### Error Recovery
Converter can provide better error messages:
```rust
match grammar_ast {
    G::Assignment { left, .. } => {
        // Validate left is a valid LValue
        let lvalue = validate_lvalue(left)?;
        // ...
    }
}
```

## Future Considerations

### Performance
- Parser AST is optimized for parsing speed
- Unified AST is optimized for evaluation speed
- Converter runs once per parse, so conversion cost is acceptable

### Extensibility
- New parsers can target the unified AST
- Parser-specific features don't pollute the core AST
- Language extensions can be prototyped in one parser first

### Tooling
- Pretty-printing uses unified AST
- Analysis tools use unified AST
- Only parser generator needs parser AST