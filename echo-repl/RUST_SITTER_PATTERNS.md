# Rust-Sitter Grammar Patterns

This document contains patterns and solutions for common rust-sitter grammar issues discovered during Echo language development, including MOO-inspired patterns that could improve Echo's grammar structure.

## Optional Fields with `Option<T>`

Rust-sitter supports optional fields using Rust's `Option<T>` type. This is useful for implementing optional syntax elements like else clauses, labels, etc.

### Basic Optional Pattern

```rust
#[derive(Debug, PartialEq)]
pub struct IfStatement {
    #[rust_sitter::leaf(text = "if")]
    _if: (),
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    condition: Box<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
    then_body: Vec<EchoAst>,
    else_clause: Option<ElseClause>,  // Optional field
    #[rust_sitter::leaf(text = "endif")]
    _endif: (),
}

#[derive(Debug, PartialEq)]
pub struct ElseClause {
    #[rust_sitter::leaf(text = "else")]
    _else: (),
    body: Vec<EchoAst>,
}
```

### Optional Labels in Control Flow

MOO-style labeled loops can be implemented with optional labels:

```rust
While {
    #[rust_sitter::leaf(text = "while")]
    _while: (),
    label: Option<Box<EchoAst>>, // Optional label: while [label] (condition)
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    condition: Box<EchoAst>,
    #[rust_sitter::leaf(text = ")")]
    _rparen: (),
    body: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "endwhile")]
    _endwhile: (),
}
```

### Optional vs Alternative Patterns

**Use Optional when:**
- The element is truly optional (may or may not appear)
- No ambiguity in parsing (clear when the optional part starts/ends)
- Following established patterns (like MOO's `while [label] (condition)`)

**Use Separate Variants when:**
- Ambiguous parsing (conflicts between with/without optional)
- Complex precedence rules needed
- Different semantic meaning

## Conflict Resolution

### Precedence for Optional Elements

When optionals create conflicts, use precedence to resolve ambiguity:

```rust
// Higher precedence for if without else (prefer this when no else follows)
#[rust_sitter::prec_right(6)]
If {
    // ... without else
}

// Lower precedence for if with else  
#[rust_sitter::prec_right(5)]
IfElse {
    // ... with else
}
```

### Conflict Markers

Use `add_conflict = true` when the parser suggests it:

```rust
Break {
    #[rust_sitter::leaf(text = "break", add_conflict = true)]
    _break: (),
    label: Option<Box<EchoAst>>,
}
```

### The Dangling Else Problem

Classic grammar issue with nested if statements:
```
if (a) if (b) stmt1 else stmt2  // else belongs to inner or outer if?
```

**Solution:** Use precedence to make else bind to nearest if:

```rust
#[rust_sitter::prec_right(3)]  // Right associative for proper binding
If {
    condition: Box<EchoAst>,
    then_body: Vec<EchoAst>,
    else_clause: Option<ElseClause>,
}
```

## Repetition Patterns

### Vec<T> for Statement Bodies

Use `#[rust_sitter::repeat(non_empty = false)]` for statement collections:

```rust
If {
    condition: Box<EchoAst>,
    #[rust_sitter::repeat(non_empty = false)]
    then_body: Vec<EchoAst>,  // Zero or more statements
    else_clause: Option<ElseClause>,
}
```

### Delimited Repetition

For comma-separated lists:

```rust
List {
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    #[rust_sitter::repeat(non_empty = false)]
    #[rust_sitter::delimited(
        #[rust_sitter::leaf(text = ",")]
        ()
    )]
    elements: Vec<EchoAst>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
}
```

## Common Patterns and Solutions

### 1. Optional Else Clause

**Problem:** Making else optional in if statements
**Solution:** Use `Option<ElseClause>` with separate struct

```rust
If {
    // ...
    else_clause: Option<ElseClause>,
}

#[derive(Debug, PartialEq)]
pub struct ElseClause {
    #[rust_sitter::leaf(text = "else")]
    _else: (),
    #[rust_sitter::repeat(non_empty = false)]
    body: Vec<EchoAst>,
}
```

### 2. Optional Labels for Break/Continue

**Problem:** Supporting `break` and `break label`
**Solution:** Optional field with high precedence

```rust
#[rust_sitter::prec_right(9)]
Break {
    #[rust_sitter::leaf(text = "break")]
    _break: (),
    label: Option<Box<EchoAst>>,
}
```

### 3. MOO-Style Control Flow

**Pattern:** `while [label] (condition) ... endwhile`
**Solution:** Optional label between keyword and parentheses

```rust
While {
    #[rust_sitter::leaf(text = "while")]
    _while: (),
    label: Option<Box<EchoAst>>,  // Goes here, not after condition
    #[rust_sitter::leaf(text = "(")]
    _lparen: (),
    condition: Box<EchoAst>,
    // ...
}
```

## Debugging Grammar Conflicts

### Understanding Error Messages

Conflict errors show:
- `symbol_sequence`: What was parsed so far
- `conflicting_lookahead`: What token caused ambiguity  
- `possible_interpretations`: Different ways to parse
- `possible_resolutions`: Suggested fixes

Example:
```
ConflictError { 
  symbol_sequence: ["'if'"], 
  conflicting_lookahead: "'('", 
  possible_interpretations: [If, IfElse],
  possible_resolutions: [Precedence, AddConflict]
}
```

### Resolution Strategies

1. **Precedence:** Use `#[rust_sitter::prec_left/right(N)]`
2. **Conflicts:** Add `add_conflict = true` to leaf nodes
3. **Restructure:** Split into separate variants
4. **Associativity:** Choose left/right associativity

### Testing Patterns

Use rust-sitter examples from https://github.com/hydro-project/rust-sitter/tree/main/example/src:

- `optionals.rs` - Optional field patterns
- `arithmetic.rs` - Precedence and associativity
- `repetitions.rs` - Vec and delimited patterns

## Best Practices

1. **Start Simple:** Begin with required fields, add optionals later
2. **Follow Conventions:** Use established patterns (MOO grammar as reference)
3. **Test Early:** Check grammar compilation frequently
4. **Document Conflicts:** Note resolution strategies for future reference
5. **Study Examples:** Learn from working grammars in rust-sitter repo

## Parser Integration

When using optionals in parser conversion:

```rust
// Handle optional label
let label_str = if let Some(label_expr) = label {
    match label_expr.as_ref() {
        G::Identifier(s) => Some(s.clone()),
        _ => anyhow::bail!("Label must be identifier"),
    }
} else {
    None
};

// Handle optional else clause
let else_vec = if let Some(else_clause) = else_clause {
    Some(else_clause.body.into_iter()
        .map(convert_grammar_to_ast)
        .collect::<Result<Vec<_>>>()?)
} else {
    None
};
```

## Common Pitfalls

1. **Optional Conflicts:** Optional fields can create parsing ambiguity
2. **Precedence Order:** Higher numbers bind tighter (multiplication > addition)  
3. **Field Visibility:** Remember to make struct fields `pub` when needed
4. **AST Conversion:** Handle `Option<T>` in conversion functions
5. **Error Recovery:** Optional fields may mask parsing errors
6. **Parentheses Conflicts:** Optional elements before `(` can conflict with `Paren` expressions
7. **Lookahead Issues:** Optional fields can create complex lookahead conflicts

## Advanced Conflict Resolution

### Parentheses Conflicts

**Problem:** `while [label] (condition)` conflicts with `while (condition)` when label is optional

**Attempted Solutions:**
- `add_conflict = true` on `(` token 
- Precedence rules
- Separate variants (While vs LabeledWhile)

**Working Solution:** Implement labels as a separate feature or use different syntax

### Complex Optional Patterns

When optionals create irresolvable conflicts, consider:

1. **Separate Variants:** Create distinct AST nodes
2. **Different Syntax:** Use unambiguous delimiters  
3. **Post-Processing:** Parse permissively, validate semantically
4. **Lexical Hints:** Use keywords to disambiguate

### Debugging Strategy

1. Start with minimal grammar
2. Add complexity incrementally  
3. Test each addition
4. Document conflicts and resolutions
5. Consider alternative syntax when stuck

## MOO-Inspired Grammar Improvements

Based on analysis of the MOO grammar, here are patterns that could significantly improve Echo's rust-sitter structure:

### 1. Statement/Expression Separation

**MOO Pattern:** Clear separation between statements (control flow) and expressions (values)

```rust
// Instead of everything in one enum
#[rust_sitter::language]
pub enum Program {
    Statements(Vec<Statement>),
    ObjectDefinition(ObjectDefinition),
}

pub enum Statement {
    Expression(ExpressionStatement),
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(ReturnStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
}

pub struct ExpressionStatement {
    expression: Expression,
    #[rust_sitter::leaf(text = ";", optional = true)]
    semicolon: Option<()>,
}

pub enum Expression {
    // Only value-producing constructs
    Number(i64),
    String(String),
    Identifier(String),
    Binary(BinaryExpression),
    Call(CallExpression),
    // etc.
}
```

**Benefits:**
- Better error messages ("expected statement" vs "expected expression")
- Clearer parsing rules
- Easier to add statement-only or expression-only features

### 2. Unified Pattern System

**MOO Pattern:** Same pattern structure for parameters, bindings, and destructuring

```rust
// Single Pattern type used everywhere
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Identifier(Identifier),
    List(ListPattern),
    Rest(RestPattern),
    Ignore,
}

// Use in all contexts
pub struct LetStatement {
    #[rust_sitter::leaf(text = "let")]
    _let: (),
    pattern: Pattern,  // Not a separate BindingPattern
    #[rust_sitter::leaf(text = "=")]
    _eq: (),
    expression: Expression,
}

pub struct Function {
    #[rust_sitter::leaf(text = "fn")]
    _fn: (),
    parameters: Pattern,  // Same Pattern type
    body: Vec<Statement>,
    #[rust_sitter::leaf(text = "endfn")]
    _endfn: (),
}
```

### 3. Centralized Precedence

**MOO Pattern:** Define precedence constants for clarity

```rust
// Precedence module
mod prec {
    pub const ASSIGNMENT: i32 = 1;      // right associative
    pub const CONDITIONAL: i32 = 2;     // non-associative
    pub const LOGICAL_OR: i32 = 3;      // left associative
    pub const LOGICAL_AND: i32 = 4;     // left associative
    pub const EQUALITY: i32 = 5;        // left associative
    pub const COMPARISON: i32 = 6;      // left associative
    pub const ADDITIVE: i32 = 7;        // left associative
    pub const MULTIPLICATIVE: i32 = 8;  // left associative
    pub const POWER: i32 = 9;           // right associative
    pub const UNARY: i32 = 10;          // left associative
    pub const ACCESS: i32 = 11;         // non-associative
}

// Use in grammar
#[rust_sitter::prec_right(prec::ASSIGNMENT)]
Assignment {
    left: Box<Expression>,
    #[rust_sitter::leaf(text = "=")]
    _op: (),
    right: Box<Expression>,
}

#[rust_sitter::prec_left(prec::LOGICAL_OR)]
Or {
    left: Box<Expression>,
    #[rust_sitter::leaf(text = "||")]
    _op: (),
    right: Box<Expression>,
}
```

### 4. Field Names for Better Error Messages

**MOO Pattern:** Named fields provide context in error messages

```rust
If {
    #[rust_sitter::field("condition")]
    condition: Box<Expression>,
    #[rust_sitter::field("then_body")]
    then_body: Vec<Statement>,
    #[rust_sitter::field("else_clause")]
    else_clause: Option<ElseClause>,
}

// Error message can say "expected expression in 'condition' field"
// instead of just "expected expression"
```

### 5. List Comprehension Implementation

**MOO Pattern:** Elegant comprehension syntax

```rust
#[derive(Debug, PartialEq, Clone)]
pub struct ListComprehension {
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    #[rust_sitter::field("expression")]
    expression: Box<Expression>,
    #[rust_sitter::leaf(text = "for")]
    _for: (),
    #[rust_sitter::field("variable")]
    variable: Pattern,
    #[rust_sitter::leaf(text = "in")]
    _in: (),
    #[rust_sitter::field("iterable")]
    iterable: Box<Expression>,
    // Could add optional filter: if condition
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
}
```

### 6. Range Syntax

**MOO Pattern:** Built-in range support

```rust
#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    #[rust_sitter::leaf(text = "[")]
    _lbracket: (),
    #[rust_sitter::field("start")]
    start: Box<Expression>,
    #[rust_sitter::leaf(text = "..")]
    _dots: (),
    #[rust_sitter::field("end")]
    end: Box<Expression>,
    #[rust_sitter::leaf(text = "]")]
    _rbracket: (),
}
```

### 7. Error Recovery Nodes

**MOO Pattern:** Graceful error recovery

```rust
pub enum Statement {
    // ... normal variants
    Error(ErrorRecovery),
}

pub struct ErrorRecovery {
    #[rust_sitter::leaf(pattern = r"[^;}\n]+")]
    content: String,
    #[rust_sitter::leaf(pattern = r"[;}\n]")]
    terminator: String,
}
```

### 8. Case-Insensitive Keywords

**MOO Pattern:** Support for case variations

```rust
// Macro for case-insensitive keywords
macro_rules! keyword {
    ($text:expr) => {
        #[rust_sitter::leaf(pattern = concat!("(?i)", $text))]
    };
}

// Usage
If {
    keyword!("if") _if: (),
    // Now accepts IF, If, if, etc.
}
```

## Implementation Priority

1. **Statement/Expression Separation** - Most impactful, improves error messages
2. **Precedence Constants** - Makes grammar maintainable
3. **List Comprehensions** - Frequently requested feature
4. **Unified Patterns** - Reduces code duplication
5. **Range Syntax** - Useful for iterations and slicing

## Reference

- [Rust-Sitter Documentation](https://github.com/hydro-project/rust-sitter)
- [Tree-sitter MOO Grammar](https://github.com/hydro-project/tree-sitter-moo)
- [Grammar Examples](https://github.com/hydro-project/rust-sitter/tree/main/example/src)