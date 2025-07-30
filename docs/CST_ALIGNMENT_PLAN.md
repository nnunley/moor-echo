# CST Alignment Plan: Echo → MOO

This document provides a concrete plan to align Echo's CST with MOO's grammar
structure.

## Core Structural Changes

### 1. Separate Program Structure

**Current Echo**:

```rust
pub enum EchoAst {
    // Everything mixed together
}
```

**Target Structure (MOO-aligned)**:

```rust
// Top-level program
pub enum Program {
    ObjectDefinition(ObjectDefinition),
    Statements(Vec<Statement>),
}

// Statements (non-value-producing)
pub enum Statement {
    Expression(ExpressionStatement),
    Let(LetStatement),
    Const(ConstStatement),
    Global(GlobalStatement),
    Block(BlockStatement),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Fork(ForkStatement),
    Try(TryStatement),
    Function(FunctionStatement),
}

// Expressions (value-producing)
pub enum Expression {
    // Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    ErrorCode(String),
    ObjectId(ObjectId),
    SystemProperty(String),
    Symbol(String),

    // Operations
    Assignment(AssignmentOperation),
    Conditional(ConditionalOperation),
    Binary(BinaryOperation),
    Unary(UnaryOperation),

    // Control flow as expressions
    Break(BreakExpression),
    Continue(ContinueExpression),
    Return(ReturnExpression),

    // Access patterns
    PropertyAccess(PropertyAccess),
    MethodCall(MethodCall),
    IndexAccess(IndexAccess),
    Slice(Slice),
    Call(Call),

    // Compound
    List(List),
    Map(Map),
    Lambda(Lambda),
    FunctionExpr(FunctionExpression),
    RangeComprehension(RangeComprehension),
    TryExpr(TryExpression),
    Pass(PassExpression),
}
```

### 2. Parameter/Binding Pattern Unification

**MOO** uses the same pattern for:

- Lambda parameters
- Function parameters
- Binding patterns in assignments

**Current Echo** has separate:

- `ParamPattern` for functions/verbs
- `BindingPattern` for assignments

**Solution**: Unify into a single pattern type:

```rust
pub enum Pattern {
    Identifier(String),
    List {
        elements: Vec<PatternElement>,
    },
}

pub enum PatternElement {
    Simple(String),
    Optional {
        name: String,
        default: Option<Expression>,
    },
    Rest(String),
}
```

### 3. Fix List Literal Syntax

**Current**: Echo uses `[]` in tests but `{}` in grammar **Target**: Use `{}`
consistently to match MOO

```rust
// Change all list literals to use {}
List {
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    elements: Vec<ListElement>,
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
}

enum ListElement {
    Expression(Expression),
    Scatter(Expression), // @expr
}
```

### 4. Add Range Syntax

```rust
Range {
    #[rust_sitter::leaf(text = "[")]
    _lbracket: (),
    start: Box<Expression>,
    #[rust_sitter::leaf(text = "..")]
    _dots: (),
    end: Box<Expression>,
    #[rust_sitter::leaf(text = "]")]
    _rbracket: (),
}
```

### 5. List Comprehensions

```rust
RangeComprehension {
    #[rust_sitter::leaf(text = "{")]
    _lbrace: (),
    expression: Box<Expression>,
    #[rust_sitter::leaf(text = "for")]
    _for: (),
    variable: String,
    #[rust_sitter::leaf(text = "in")]
    _in: (),
    iterable: Box<Expression>, // Can be Range or any expression
    #[rust_sitter::leaf(text = "}")]
    _rbrace: (),
}
```

## Precedence Alignment

Replace Echo's ad-hoc precedence with MOO's table:

```rust
// Define all operators with MOO precedence
impl Expression {
    // Assignment (1, right)
    Assignment { left: LValue, right: Box<Expression> },

    // Conditional (2, none)
    Conditional { condition: Box<Expression>, then: Box<Expression>, else: Box<Expression> },

    // Logical (3, left)
    Or { left: Box<Expression>, right: Box<Expression> },
    And { left: Box<Expression>, right: Box<Expression> },

    // Comparison (4, left)
    Equal { left: Box<Expression>, right: Box<Expression> },
    NotEqual { left: Box<Expression>, right: Box<Expression> },
    Less { left: Box<Expression>, right: Box<Expression> },
    LessEqual { left: Box<Expression>, right: Box<Expression> },
    Greater { left: Box<Expression>, right: Box<Expression> },
    GreaterEqual { left: Box<Expression>, right: Box<Expression> },
    In { left: Box<Expression>, right: Box<Expression> },

    // Bitwise (5, left) - TO ADD
    BitOr { left: Box<Expression>, right: Box<Expression> },
    BitAnd { left: Box<Expression>, right: Box<Expression> },
    BitXor { left: Box<Expression>, right: Box<Expression> },

    // Shift (6, left) - TO ADD
    ShiftLeft { left: Box<Expression>, right: Box<Expression> },
    ShiftRight { left: Box<Expression>, right: Box<Expression> },

    // Arithmetic (7-8, left)
    Add { left: Box<Expression>, right: Box<Expression> },
    Subtract { left: Box<Expression>, right: Box<Expression> },
    Multiply { left: Box<Expression>, right: Box<Expression> },
    Divide { left: Box<Expression>, right: Box<Expression> },
    Modulo { left: Box<Expression>, right: Box<Expression> },

    // Power (9, right)
    Power { left: Box<Expression>, right: Box<Expression> },

    // Unary (10, left)
    Not { operand: Box<Expression> },
    Negate { operand: Box<Expression> },
    BitNot { operand: Box<Expression> }, // TO ADD

    // Access (11, none)
    PropertyAccess { object: Box<Expression>, property: String },
    MethodCall { object: Box<Expression>, method: String, args: Vec<Expression> },
    IndexAccess { object: Box<Expression>, index: Box<Expression> },
    SystemProperty { name: String },
}
```

## Migration Steps

### Phase 1: Structural Refactoring

1. Create separate Statement and Expression enums
2. Update parser to produce Program instead of EchoAst
3. Update evaluator to handle new structure

### Phase 2: Syntax Alignment

1. Change list literals from `[]` to `{}`
2. Add range syntax `[start..end]`
3. Unify parameter and binding patterns

### Phase 3: Missing Features

1. Add list comprehensions
2. Add fork statements
3. Add try expressions
4. Add pass expressions
5. Add map literals
6. Add symbol literals

### Phase 4: Operator Completion

1. Add bitwise operators
2. Add shift operators
3. Add `in` operator
4. Fix operator precedence

### Phase 5: Advanced Features

1. Add elseif clauses
2. Add labeled loops
3. Add scatter/splat in lists and calls
4. Add flyweight objects

## Benefits

1. **MOO Compatibility**: Can parse MOO code directly
2. **Clear Structure**: Separation of concerns
3. **Better Error Messages**: Know if expecting statement vs expression
4. **Easier Extensions**: Clear where to add new features
5. **Type Safety**: Rust's type system enforces correct AST construction

## Example Transformations

### Current Echo:

```rust
EchoAst::LocalAssignment {
    target: BindingPattern::Identifier("x"),
    value: Box::new(EchoAst::Number(42)),
}
```

### Target (MOO-aligned):

```rust
Statement::Let(LetStatement {
    target: Pattern::Identifier("x"),
    expression: Expression::Integer(42),
})
```

### List Comprehension:

```rust
Expression::RangeComprehension(RangeComprehension {
    expression: Box::new(Expression::Binary(BinaryOp::Multiply,
        Box::new(Expression::Identifier("x")),
        Box::new(Expression::Integer(2))
    )),
    variable: "x",
    iterable: Box::new(Expression::Range(Range {
        start: Box::new(Expression::Integer(1)),
        end: Box::new(Expression::Integer(10)),
    })),
})
```

This alignment would make Echo a proper superset of MOO while maintaining its
own extensions.
