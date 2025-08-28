# Parser Architecture and Grammar Implementation

Comprehensive documentation covering the Echo REPL parser architecture, grammar design, rust-sitter integration, and language implementation details.

## Table of Contents

- [Overview](#overview)
- [Parser Architecture](#parser-architecture)
- [Grammar Design](#grammar-design)
- [Rust-Sitter Integration](#rust-sitter-integration)
- [AST Structure](#ast-structure)
- [Language Implementation](#language-implementation)
- [Testing Strategy](#testing-strategy)
- [Performance Characteristics](#performance-characteristics)
- [Future Improvements](#future-improvements)

## Overview

The Echo REPL parser system is built on **rust-sitter**, a Rust procedural macro system that generates efficient parsers from annotated AST definitions. This approach provides several advantages:

- **Performance**: Generated parsers are highly optimized
- **Type Safety**: AST nodes are fully type-checked at compile time
- **Maintainability**: Grammar and AST structure are defined together
- **Incremental Parsing**: Support for live editing and syntax highlighting
- **Error Recovery**: Graceful handling of syntax errors

### Key Design Principles

1. **Unified AST/Grammar**: Grammar rules and AST nodes are defined together
2. **MOO Compatibility**: Full support for legacy MOO syntax
3. **Modern Extensions**: Enhanced syntax while maintaining backward compatibility
4. **Error Recovery**: Continue parsing after errors for better IDE support
5. **Performance First**: Optimized for both parse time and memory usage

## Parser Architecture

### Dual-Grammar System

Echo employs a dual-grammar architecture supporting both legacy MOO and modern Echo syntax:

```
┌─────────────────┐    ┌─────────────────┐
│   MOO Legacy    │    │  Modern Echo    │
│    Syntax       │    │     Syntax      │
└─────────────────┘    └─────────────────┘
         │                       │
         └───────┐       ┌───────┘
                 │       │
         ┌─────────────────────┐
         │   Unified Parser    │
         │  (rust-sitter)      │
         └─────────────────────┘
                 │
         ┌─────────────────────┐
         │    Unified AST      │
         │   (Type-Safe)       │
         └─────────────────────┘
```

### Parser Components

#### rust-sitter Integration
```rust
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[derive(Debug, PartialEq)]
    #[rust_sitter::language]
    pub enum EchoAst {
        // Leaf nodes with literal text
        #[rust_sitter::leaf(text = "true")]
        True,
        
        // Pattern matching with transformation
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i64),
        
        // Complex nodes with precedence
        #[rust_sitter::prec_left(7)]
        Add {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "+")]
            op: (),
            right: Box<EchoAst>,
        },
    }
}
```

#### Grammar Generation Process
1. **Annotation Processing**: Procedural macros analyze AST annotations
2. **Grammar Generation**: Automatic generation of parsing rules
3. **Parser Compilation**: Creation of optimized parser code
4. **Runtime Integration**: Seamless integration with evaluator

### Error Recovery Strategy

The parser implements sophisticated error recovery:

#### Recovery Techniques
- **Insertion Recovery**: Insert missing tokens (semicolons, keywords)
- **Deletion Recovery**: Skip unexpected tokens
- **Substitution Recovery**: Replace incorrect tokens with expected ones
- **Synchronization Points**: Resume parsing at statement boundaries

#### Error Reporting
```rust
pub struct ParseError {
    pub location: SourceLocation,
    pub message: String,
    pub suggestions: Vec<String>,
    pub severity: ErrorSeverity,
}
```

Provides:
- **Precise Locations**: Exact line/column information
- **Helpful Messages**: Clear explanation of the problem
- **Suggestions**: Possible fixes for common errors
- **Severity Levels**: Distinguish errors from warnings

## Grammar Design

### Language Structure

#### Core Grammar Elements

```rust
pub enum Statement {
    // Variable declarations
    LetBinding {
        pattern: Pattern,
        value: Expression,
    },
    
    // Expression statements
    ExpressionStatement(Expression),
    
    // Control flow
    IfStatement {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    
    // Object definitions
    ObjectDefinition {
        name: Identifier,
        parent: Option<Expression>,
        body: ObjectBody,
    },
}

pub enum Expression {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    
    // Identifiers
    Identifier(String),
    
    // Binary operations
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    
    // Function calls
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    // Property access
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
}
```

#### Precedence Rules

Following MOO's precedence table:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 14 | `()`, `[]`, `.` | Left |
| 13 | `!`, `-` (unary), `+` (unary) | Right |
| 12 | `^` | Right |
| 11 | `*`, `/`, `%` | Left |
| 10 | `+`, `-` | Left |
| 9 | `<<`, `>>` | Left |
| 8 | `<`, `<=`, `>`, `>=`, `in` | Left |
| 7 | `==`, `!=` | Left |
| 6 | `&` | Left |
| 5 | `^` | Left |
| 4 | `\|` | Left |
| 3 | `&&` | Left |
| 2 | `\|\|` | Left |
| 1 | `? :` | Right |

### Pattern System

#### Unified Pattern Design
```rust
pub enum Pattern {
    // Simple identifier
    Identifier(String),
    
    // Destructuring patterns  
    Destructuring {
        patterns: Vec<Pattern>,
        rest: Option<String>,
    },
    
    // Optional patterns with defaults
    Optional {
        pattern: Box<Pattern>,
        default: Option<Expression>,
    },
    
    // Type patterns for matching
    TypePattern {
        type_name: String,
        inner: Option<Box<Pattern>>,
    },
}
```

Used consistently across:
- Function parameters
- Variable bindings  
- Match expressions
- Destructuring assignments

#### Pattern Examples
```echo
// Simple patterns
let x = 42;
let {a, b} = coordinates;

// Optional patterns with defaults
function greet({name, ?title = "Friend"}) {
    return "Hello, " + title + " " + name + "!";
}

// Rest patterns
let {first, @rest} = items;

// Type patterns in match expressions
match value {
    case String(s) => "Got string: " + s,
    case Number(n) if n > 0 => "Positive number",
    case _ => "Something else"
}
```

### Object System Grammar

#### Object Definition Syntax
```echo
// Modern Echo syntax
object Player extends $thing
    // Properties
    property name = "Anonymous"
    property level = 1
    property experience = 0
    
    // Methods (verbs)
    verb examine()
        return "You see " + this.name + ", level " + this.level + ".";
    endverb
    
    // Internal functions
    function calculate_damage(weapon) {
        return weapon.damage * this.level;
    }
    
    // Event handlers
    event on_level_up(new_level) {
        this.level = new_level;
        emit("player_leveled", this, new_level);
    }
    
    // Datalog queries
    query can_attack(target) :-
        same_location(this, target),
        hostile(this, target).
endobject
```

#### Legacy MOO Compatibility
```moo
// Traditional MOO syntax (still supported)
@create $thing named Player
@property Player.name "Anonymous"
@property Player.level 1
@verb Player:examine this none this
return "You see " + this.name + ", level " + tostr(this.level) + ".";
@endverb
```

## Rust-Sitter Integration

### Procedural Macro System

rust-sitter uses procedural macros to transform AST definitions into efficient parsers:

#### Annotation Types
```rust
// Leaf nodes (terminals)
#[rust_sitter::leaf(text = "keyword")]     // Literal text
#[rust_sitter::leaf(pattern = r"\d+")]     // Regex pattern  
#[rust_sitter::leaf(transform = parse_fn)] // Custom parsing

// Precedence and associativity
#[rust_sitter::prec_left(7)]   // Left associative, precedence 7
#[rust_sitter::prec_right(8)]  // Right associative, precedence 8
#[rust_sitter::prec(9)]        // No associativity, precedence 9

// Field annotations  
#[rust_sitter::skip]           // Skip in AST construction
#[rust_sitter::optional]       // Optional field
```

#### Grammar Rule Generation
The macro system automatically generates:
- **Parsing Functions**: Efficient recursive descent parsers
- **AST Constructors**: Type-safe AST node creation
- **Error Recovery**: Automatic error handling and recovery
- **Tree-Sitter Grammar**: For editor integration

### Tree-Sitter Integration

#### Grammar Output
The system generates standard tree-sitter grammars:

```javascript
// Generated grammar.js excerpt
module.exports = grammar({
  name: 'echo',
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => choice(
      $.let_binding,
      $.expression_statement,
      $.if_statement,
      $.object_definition,
    ),
    
    expression: $ => choice(
      $.integer,
      $.string,
      $.identifier,
      $.binary_operation,
      $.function_call,
    ),
    
    binary_operation: $ => prec.left(7, seq(
      $.expression,
      choice('+', '-'),
      $.expression
    )),
  }
});
```

#### Editor Support
- **Syntax Highlighting**: Rich highlighting in VS Code, Neovim, etc.
- **Code Folding**: Intelligent folding of blocks and functions
- **Incremental Parsing**: Fast re-parsing for live editing
- **Error Squiggles**: Real-time error underlining

## AST Structure

### Unified AST Design

The AST is designed to support both MOO and Echo syntax while maintaining type safety:

#### Core AST Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    LetBinding { pattern: Pattern, value: Expression },
    ExpressionStatement(Expression),
    IfStatement { condition: Expression, then_branch: Block, else_branch: Option<Block> },
    WhileLoop { condition: Expression, body: Block },
    ForLoop { pattern: Pattern, iterator: Expression, body: Block },
    TryCatch { try_block: Block, catch_var: Option<String>, catch_block: Block },
    ObjectDefinition { name: String, parent: Option<Expression>, body: ObjectBody },
    Return(Option<Expression>),
    Break,
    Continue,
    Throw(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    
    // Identifiers and access
    Identifier(String),
    PropertyAccess { object: Box<Expression>, property: String },
    IndexAccess { object: Box<Expression>, index: Box<Expression> },
    
    // Operations  
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    UnaryOp { op: UnaryOperator, operand: Box<Expression> },
    
    // Control flow
    IfExpression { condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    Match { expr: Box<Expression>, arms: Vec<MatchArm> },
    
    // Functions and calls
    FunctionCall { function: Box<Expression>, arguments: Vec<Expression> },
    MethodCall { object: Box<Expression>, method: String, arguments: Vec<Expression> },
    Lambda { params: Vec<Pattern>, body: Box<Expression> },
    
    // Advanced features
    ListComprehension { expr: Box<Expression>, pattern: Pattern, iterator: Box<Expression> },
    TryExpression { expr: Box<Expression>, error_codes: Vec<String> },
}
```

#### AST Transformation Pipeline

```rust
Source Code → Tokens → Parse Tree → AST → Evaluated AST
     ↓           ↓         ↓         ↓          ↓
   Lexical   Syntax    Type      Semantic   Runtime
  Analysis   Parse    Check    Analysis   Evaluation
```

### Error Recovery Nodes

Special AST nodes handle partial parsing:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryNode {
    MissingExpression { expected: String, location: SourceLocation },
    UnexpectedToken { token: String, location: SourceLocation },
    IncompleteBlock { partial: Block, location: SourceLocation },
}
```

Benefits:
- **IDE Support**: Continue providing completions and hints
- **Error Analysis**: Better error messages with context
- **Incremental Parsing**: Reuse valid portions during editing

## Language Implementation

### Parsing Process

#### Multi-Pass Parsing
1. **Lexical Analysis**: Convert source to tokens
2. **Syntax Parsing**: Build concrete syntax tree (CST)
3. **AST Construction**: Transform CST to typed AST
4. **Semantic Analysis**: Type checking and validation
5. **Evaluation**: Execute or compile AST

#### Parser Integration
```rust
pub struct EchoParser {
    grammar: Grammar,
    error_recovery: ErrorRecovery,
    source_map: SourceMap,
}

impl EchoParser {
    pub fn parse(&mut self, source: &str) -> Result<Program, Vec<ParseError>> {
        let tokens = self.tokenize(source)?;
        let cst = self.parse_tokens(tokens)?;
        let ast = self.build_ast(cst)?;
        Ok(ast)
    }
    
    pub fn parse_incremental(&mut self, 
        old_tree: &Tree, 
        edit: &Edit
    ) -> Result<Program, Vec<ParseError>> {
        // Incremental parsing for live editing
        let new_tree = self.update_tree(old_tree, edit)?;
        self.build_ast_from_tree(new_tree)
    }
}
```

### MOO Compatibility Layer

#### Syntax Translation
The parser handles both syntaxes seamlessly:

```rust
// Internal representation - same for both syntaxes
ObjectDefinition {
    name: "Player".to_string(),
    parent: Some(Expression::Identifier("$thing".to_string())),
    properties: vec![
        PropertyDef { name: "name", value: Expression::String("Anonymous") }
    ],
    verbs: vec![
        VerbDef { name: "examine", code: "return this.name;" }
    ]
}

// Can be parsed from either:
// Modern: object Player extends $thing ...
// Legacy: @create $thing named Player ...
```

#### Built-in Function Integration
```rust
// Built-in functions are parsed as regular function calls
// but resolved to native implementations during evaluation
FunctionCall {
    function: Expression::Identifier("typeof"),
    arguments: vec![Expression::Identifier("value")]
}
```

### Advanced Language Features

#### Pattern Matching Implementation
```rust
// Match expression AST
Match {
    expr: Box<Expression>,
    arms: Vec<MatchArm>
}

pub struct MatchArm {
    pattern: Pattern,
    guard: Option<Expression>,
    body: Expression,
}

// Pattern types
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Constructor { name: String, patterns: Vec<Pattern> },
    Guard { pattern: Box<Pattern>, condition: Expression },
}
```

#### List Comprehension Support
```rust  
// {x * 2 for x in [1, 2, 3]} parsed as:
ListComprehension {
    expr: BinaryOp {
        left: Identifier("x"),
        op: Multiply,
        right: Integer(2)
    },
    pattern: Pattern::Identifier("x"),
    iterator: List(vec![Integer(1), Integer(2), Integer(3)])
}
```

## Testing Strategy

### Grammar Testing

#### Unit Tests
```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    
    #[test]
    fn test_arithmetic_expression() {
        let source = "2 + 3 * 4";
        let ast = parse(source).unwrap();
        
        assert_matches!(ast, Expression::BinaryOp {
            left: box Expression::Integer(2),
            op: Add,
            right: box Expression::BinaryOp {
                left: box Expression::Integer(3),
                op: Multiply,
                right: box Expression::Integer(4)
            }
        });
    }
    
    #[test]
    fn test_precedence_ordering() {
        // Ensure 2 + 3 * 4 parses as 2 + (3 * 4), not (2 + 3) * 4
        let ast = parse("2 + 3 * 4").unwrap();
        let result = evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(14)); // Not 20
    }
}
```

#### Integration Tests
```bash
# Test with real MOO databases
cargo test -- --test moo_compatibility

# Test parser performance  
cargo bench parser_benchmarks

# Test error recovery
cargo test parser_error_recovery
```

#### Corpus Testing
Testing against large collections of MOO code:

```rust
#[test]
fn test_lambdacore_parsing() {
    let db_files = glob("test-data/**/*.moo").unwrap();
    for file in db_files {
        let source = std::fs::read_to_string(file).unwrap();
        let result = parse(&source);
        assert!(result.is_ok(), "Failed to parse {}", file.display());
    }
}
```

### Error Recovery Testing

#### Recovery Scenarios
```echo
// Missing semicolon - should recover
let x = 42
let y = 43  // Parser should insert semicolon and continue

// Missing closing brace - should recover  
if condition {
    do_something()
// Should detect missing } and continue parsing

// Invalid token - should skip and continue
let x = 42 @invalid_token let y = 43
```

#### Test Framework
```rust
fn test_error_recovery(source: &str, expected_errors: usize, expected_ast_nodes: usize) {
    let result = parse_with_recovery(source);
    assert_eq!(result.errors.len(), expected_errors);
    assert_eq!(count_valid_nodes(&result.ast), expected_ast_nodes);
}
```

## Performance Characteristics

### Parsing Performance

#### Benchmarks
- **Simple Expressions**: ~1-10μs
- **Object Definitions**: ~50-200μs  
- **Large Files (1000+ lines)**: ~1-10ms
- **MOO Database Import**: ~100ms-1s (depending on size)

#### Memory Usage
- **AST Node Size**: ~64-128 bytes per node
- **Parse Tree**: ~2-5x source size in memory
- **Incremental Updates**: ~10-50x faster than full reparse

#### Optimization Techniques
- **String Interning**: Reduce memory usage for identifiers
- **Arena Allocation**: Fast allocation for AST nodes
- **Lazy Evaluation**: Parse function bodies on demand
- **Caching**: Cache parsed modules and objects

### Scalability Characteristics

#### Large File Handling
- **Streaming Parsing**: Process files larger than memory
- **Parallel Parsing**: Parse multiple files concurrently  
- **Incremental Updates**: Fast re-parsing for editors
- **Error Boundaries**: Limit error propagation

#### Memory Management
```rust
// Arena-based allocation for fast parsing
pub struct ParseArena {
    strings: StringInterner,
    nodes: Arena<AstNode>,
    source_map: SourceMap,
}

impl ParseArena {
    pub fn alloc_node<T: Into<AstNode>>(&mut self, node: T) -> &mut AstNode {
        self.nodes.alloc(node.into())
    }
}
```

## Future Improvements

### Planned Enhancements

#### Advanced Error Recovery
- **Semantic Error Recovery**: Continue parsing after type errors
- **Contextual Suggestions**: Better error messages based on context
- **Auto-Fix Capabilities**: Automatic correction of common mistakes

#### Performance Optimizations
- **Parallel Parsing**: Multi-threaded parsing for large files
- **SIMD Optimization**: Vectorized tokenization
- **JIT Grammar**: Runtime-optimized parsing for hot paths

#### Language Server Features
- **Real-time Validation**: Continuous parsing and error checking
- **Code Completion**: Context-aware completions
- **Refactoring Support**: Safe automated code transformations
- **Symbol Navigation**: Go-to-definition, find-references

### Research Directions

#### Advanced Parsing Techniques
- **GLR Parsing**: Generalized LR parsing for ambiguous grammars
- **PEG Integration**: Parsing Expression Grammars for complex patterns
- **Neural Parsing**: ML-assisted error recovery and completion

#### Language Evolution
- **Gradual Typing**: Optional static type checking
- **Module System**: Namespace and import management  
- **Macro System**: Compile-time code generation
- **Query Optimization**: Datalog query compilation and optimization

This comprehensive parser and grammar documentation provides a complete understanding of the Echo REPL's parsing system, from low-level implementation details to high-level language design decisions. The emphasis on performance, error recovery, and MOO compatibility makes it suitable for both educational use and production deployment.