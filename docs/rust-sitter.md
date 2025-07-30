# Rust-Sitter Grammar Conflict Resolution Guide

## Overview

Rust-sitter is a Rust library that allows defining Tree-sitter grammars using
Rust type annotations. This document covers conflict resolution mechanisms for
handling grammar ambiguities, specifically addressing the PropertyAccess vs
PropAssignment conflict.

## Current Conflict

**Problem**: Both PropertyAccess and PropAssignment start with the same pattern
`object.property`:

- PropertyAccess: `object.property`
- PropAssignment: `object.property = value`

This creates an LR(1) conflict because the parser cannot determine which rule to
apply until it sees whether there's an `=` token after the property access.

## Conflict Resolution Mechanisms

### 1. Precedence Annotations

Rust-sitter provides three precedence annotations to resolve parsing conflicts:

```rust
#[rust_sitter::prec(n)]        // Non-associative with precedence n
#[rust_sitter::prec_left(n)]   // Left-associative with precedence n
#[rust_sitter::prec_right(n)]  // Right-associative with precedence n
```

**Key Points**:

- Higher precedence numbers bind more tightly
- Default precedence is 0
- Used to resolve conflicts at parser generation time

### 2. Current Grammar Issues

Looking at the current grammar in `/src/parser/grammar.rs`:

```rust
// Property access - precedence 10
#[rust_sitter::prec_left(10)]
PropertyAccess {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
},

// Property assignment - precedence 8
#[rust_sitter::prec(8)]
PropAssignment {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
    #[rust_sitter::leaf(text = "=")]
    _equals: (),
    value: Box<EchoAst>,
},
```

**Issue**: PropertyAccess has higher precedence (10) than PropAssignment (8),
which means the parser will prefer PropertyAccess when it encounters
`object.property`, making PropAssignment unreachable.

## Solution: Restructure Grammar to Eliminate Conflict

### Option 1: Use Dynamic Precedence

Tree-sitter supports dynamic precedence for runtime resolution:

```rust
// In tree-sitter grammar DSL (not directly available in rust-sitter):
// prec.dynamic(1, property_assignment)
// prec.dynamic(0, property_access)
```

**Note**: Rust-sitter may not directly support `prec.dynamic`. Need to check if
this is available.

### Option 2: Restructure as Single Rule with Optional Assignment

Create a unified property interaction rule:

```rust
#[rust_sitter::prec_left(10)]
PropertyInteraction {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
    // Optional assignment part
    assignment: Option<PropertyAssignmentSuffix>,
},

// Separate struct for the assignment suffix
PropertyAssignmentSuffix {
    #[rust_sitter::leaf(text = "=")]
    _equals: (),
    value: Box<EchoAst>,
}
```

### Option 3: Use Tree-sitter's `conflicts` Field

If rust-sitter supports it, declare the conflict explicitly:

```rust
// In grammar configuration (if supported):
// conflicts: [["PropertyAccess", "PropAssignment"]]
```

This tells Tree-sitter to use GLR parsing for this specific conflict.

### Option 4: Lookahead-Based Rule Ordering

Ensure PropAssignment is checked before PropertyAccess by making PropAssignment
more specific:

```rust
// Property assignment (more specific - checked first)
#[rust_sitter::prec(10)]  // Same or higher precedence
PropAssignment {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
    #[rust_sitter::leaf(text = "=")]
    _equals: (),
    value: Box<EchoAst>,
},

// Property access (less specific - checked after)
#[rust_sitter::prec_left(9)]  // Lower precedence
PropertyAccess {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
},
```

## Recommended Solution

**Use Option 4**: Restructure precedence so PropAssignment (more specific) has
higher precedence than PropertyAccess (less specific).

### Implementation

```rust
// Give PropAssignment higher precedence since it's more specific
#[rust_sitter::prec(11)]
PropAssignment {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
    #[rust_sitter::leaf(text = "=")]
    _equals: (),
    value: Box<EchoAst>,
},

// PropertyAccess gets lower precedence
#[rust_sitter::prec_left(10)]
PropertyAccess {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ".")]
    _dot: (),
    property: Box<EchoAst>,
},
```

**Rationale**:

- PropAssignment is more specific (requires `=` token)
- PropertyAccess is more general (just `object.property`)
- Tree-sitter will try the more specific rule first
- If assignment fails (no `=`), it falls back to property access

## Additional Conflict Resolution Techniques

### Associativity for Operators

```rust
// Left-associative addition: (1 + 2) + 3
#[rust_sitter::prec_left(1)]
Add {
    left: Box<EchoAst>,
    #[rust_sitter::leaf(text = "+")]
    _op: (),
    right: Box<EchoAst>,
},

// Right-associative assignment: a = b = c means a = (b = c)
#[rust_sitter::prec_right(1)]
Assignment {
    left: Box<EchoAst>,
    #[rust_sitter::leaf(text = "=")]
    _op: (),
    right: Box<EchoAst>,
},
```

### Method Call vs Property Access

Method calls should have higher precedence than property access:

```rust
// Method calls - higher precedence (12)
#[rust_sitter::prec_left(12)]
MethodCall {
    object: Box<EchoAst>,
    #[rust_sitter::leaf(text = ":")]
    _colon: (),
    method: Box<EchoAst>,
    // ... args
},

// Property access - lower precedence (10)
#[rust_sitter::prec_left(10)]
PropertyAccess {
    // ... property access
},
```

## Testing Conflict Resolution

Create tests for both cases to ensure resolution works:

```rust
#[test]
fn test_property_access_vs_assignment_precedence() {
    // Property access should parse correctly
    let result = parse_echo("obj.prop");
    assert!(matches!(result.unwrap(), EchoAst::PropertyAccess { .. }));

    // Property assignment should parse correctly
    let result = parse_echo("obj.prop = 42");
    assert!(matches!(result.unwrap(), EchoAst::PropAssignment { .. }));
}
```

## References

- [Rust-sitter Documentation](https://github.com/hydro-project/rust-sitter)
- [Tree-sitter Grammar DSL](https://tree-sitter.github.io/tree-sitter/creating-parsers/2-the-grammar-dsl.html)
- [Tree-sitter Conflict Resolution](https://tree-sitter.github.io/tree-sitter/creating-parsers/2-the-grammar-dsl.html#conflicts)

## Conclusion

The PropertyAccess vs PropAssignment conflict can be resolved by adjusting
precedence levels so that the more specific PropAssignment rule has higher
precedence than the more general PropertyAccess rule. This follows the principle
that more specific patterns should be matched before more general ones.
