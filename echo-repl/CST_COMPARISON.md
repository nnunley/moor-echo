# CST Structure Comparison: Echo vs MOO

This document compares the Concrete Syntax Tree structures between Echo and MOO grammars to identify how we can align them.

## Overall Structure

### MOO Grammar Structure
```javascript
// Top level
program: choice(
  object_definition,
  repeat(statement)
)

// Statements are separate from expressions
statement: choice(
  expression_statement,
  assignment_statement,
  block_statement,
  if_statement,
  while_statement,
  for_statement,
  fork_statement,
  try_statement,
  function_statement
)

// Expressions are a flat choice with precedence
expression: choice(
  // literals, operations, access patterns, etc.
)
```

### Echo Grammar Structure
```rust
// Everything is an EchoAst variant
pub enum EchoAst {
  // Mix of expressions and statements
  // No clear separation
}
```

## Key Structural Differences

### 1. Statement vs Expression Separation

**MOO**: Clear separation between statements and expressions
- Statements: control flow, assignments, blocks
- Expressions: values that can be evaluated
- expression_statement wraps expressions when used as statements

**Echo**: Everything is EchoAst
- No distinction between statements and expressions
- Can lead to parsing ambiguities

### 2. Assignment Handling

**MOO**:
```javascript
assignment_statement: choice(
  let_statement,
  const_statement,
  global_statement
)

let_statement: seq(
  keyword("let"),
  field("target", choice($.identifier, $.binding_pattern)),
  "=",
  field("expression", $.expression),
  optional(";")
)
```

**Echo**:
```rust
LocalAssignment {
    #[rust_sitter::leaf(text = "let")]
    _let: (),
    target: BindingPattern,
    #[rust_sitter::leaf(text = "=")]
    _eq: (),
    value: Box<EchoAst>,
}
```

### 3. Control Flow Structure

**MOO If Statement**:
```javascript
if_statement: seq(
  keyword("if"),
  "(",
  field("condition", $.expression),
  ")",
  field("then_body", repeat($.statement)),
  field("elseif_clauses", repeat($.elseif_clause)),
  field("else_clause", optional($.else_clause)),
  keyword("endif")
)
```

**Echo If**:
```rust
If {
    condition: Box<EchoAst>,
    then_body: Vec<EchoAst>,
    else_clause: Option<ElseClause>,
    // No elseif support
}
```

### 4. List Syntax

**MOO**: `{1, 2, 3}` for lists
**Echo**: Uses `{}` but parser expects `[` based on tests

### 5. Function Definitions

**MOO**:
```javascript
function_statement: seq(
  keyword("fn"),
  field("name", $.identifier),
  "(",
  field("parameters", optional($.lambda_parameters)),
  ")",
  field("body", repeat($.statement)),
  keyword("endfn")
)
```

**Echo**: No named function statements, only expressions

## Recommendations to Match MOO CST

### 1. Separate Statements and Expressions

Create distinct types:
```rust
pub enum Statement {
    Expression(Expression),
    Let { target: BindingPattern, value: Expression },
    Const { target: BindingPattern, value: Expression },
    If { condition: Expression, then_body: Vec<Statement>, ... },
    While { condition: Expression, body: Vec<Statement> },
    For { variable: String, iterable: Expression, body: Vec<Statement> },
    Block(Vec<Statement>),
    Return(Option<Expression>),
    Break(Option<String>),
    Continue(Option<String>),
}

pub enum Expression {
    Number(i64),
    String(String),
    Identifier(String),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    // ... other value-producing constructs
}
```

### 2. Match MOO's Precedence Structure

Use MOO's precedence table exactly:
```rust
// In grammar definition
const PRECEDENCES: &[(i32, Assoc, &[&str])] = &[
    (1, Right, &["="]),
    (2, None, &["?", "|"]),
    (3, Left, &["||", "&&"]),
    (4, Left, &["==", "!=", "<", "<=", ">", ">=", "in"]),
    (5, Left, &["|.", "&.", "^."]),
    (6, Left, &["<<", ">>"]),
    (7, Left, &["+", "-"]),
    (8, Left, &["*", "/", "%"]),
    (9, Right, &["^"]),
    (10, Left, &["!", "~", "unary-"]),
    (11, None, &[".", ":", "[", "$"]),
];
```

### 3. Adopt MOO's Grammar Structure

```rust
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[rust_sitter::language]
    pub enum Program {
        ObjectDefinition(ObjectDefinition),
        Statements(Vec<Statement>),
    }
    
    pub enum Statement {
        Expression(ExpressionStatement),
        Assignment(AssignmentStatement),
        Block(BlockStatement),
        If(IfStatement),
        While(WhileStatement),
        For(ForStatement),
        // ... other statements
    }
    
    pub struct ExpressionStatement {
        expression: Expression,
        #[rust_sitter::leaf(text = ";", optional = true)]
        semicolon: Option<()>,
    }
}
```

### 4. Fix List Literal Syntax

Change Echo to use `{}` consistently:
```rust
List {
    #[rust_sitter::leaf(text = "{")]  // Not "["
    _lbrace: (),
    elements: Vec<Expression>,
    #[rust_sitter::leaf(text = "}")]  // Not "]"
    _rbrace: (),
}
```

### 5. Add Missing Constructs

Priority additions to match MOO:
1. **Range syntax**: `[1..10]`
2. **List comprehensions**: `{expr for var in iterable}`
3. **Try expressions**: `` `expr ! codes => fallback' ``
4. **Fork statements**: `fork (delay) ... endfork`
5. **Function statements**: Named function definitions
6. **Elseif clauses**: Multiple conditional branches
7. **Pass expressions**: `pass(args)`

## Migration Path

1. **Phase 1**: Refactor to separate statements/expressions
2. **Phase 2**: Fix precedence to match MOO exactly
3. **Phase 3**: Add missing statement types
4. **Phase 4**: Add missing expression types
5. **Phase 5**: Add MOO-specific features (fork, pass, etc.)

This would allow Echo to parse MOO code directly while maintaining its own extensions.