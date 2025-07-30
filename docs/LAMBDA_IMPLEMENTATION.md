# Lambda Implementation in Echo

## Overview

Echo now supports MOO-style anonymous functions (lambdas) with two syntaxes:

### 1. Arrow Functions

Simple expression-based lambdas using the `=>` operator:

```echo
// Single parameter
let inc = x => x + 1

// Multiple parameters (use braces)
let add = {x, y} => x + y

// Nested functions
let curry = f => x => y => f(x, y)
```

### 2. Block Functions

Multi-statement lambdas using `fn...endfn` syntax:

```echo
// Empty parameters
let constant = fn {} 42 endfn

// With parameters
let factorial = fn {n}
  if (n <= 1)
    1
  else
    n * factorial(n - 1)
  endif
endfn
```

## Features Implemented

✅ **Arrow Function Syntax** (`params => expr`)

- Single parameter: `x => x + 1`
- Multiple parameters: `{x, y} => x + y`
- Empty parameters: `{} => 42`

✅ **Block Function Syntax** (`fn params ... endfn`)

- Single line: `fn {x} x * 2 endfn`
- Multiple statements supported (though multi-line REPL parsing needs work)

✅ **Closures**

- Lambdas capture variables from their defining scope
- Example: `let x = 10; let f = y => x + y; f(5)` returns 15

✅ **Higher-Order Functions**

- Functions can return functions
- Functions can take functions as parameters
- Example: `let apply = {f, x} => f(x)`

✅ **Lambda Invocation**

- Call syntax: `func(args)`
- Works with any expression that evaluates to a lambda

## Technical Details

### AST Structure

```rust
Lambda {
    params: Vec<String>,
    body: Box<EchoAst>,
}
```

### Value Type

```rust
Lambda {
    params: Vec<String>,
    body: crate::ast::EchoAst,
    captured_env: HashMap<String, Value>,
}
```

### Grammar Implementation

- Added `ArrowFunction` and `BlockFunction` to grammar
- Created `ParamPattern` enum for parameter patterns
- Resolved grammar conflicts between method calls and lambda calls

## Future Enhancements

The parameter pattern infrastructure supports these MOO features (not yet
implemented):

1. **Optional Parameters**: `?name=default`

   ```echo
   let greet = fn {name, ?greeting="Hello"}
     greeting + ", " + name + "!"
   endfn
   ```

2. **Rest Parameters**: `@rest`

   ```echo
   let sum = fn {@args}
     // sum all arguments
   endfn
   ```

3. **Mixed Patterns**: Combining simple, optional, and rest parameters
   ```echo
   let process = fn {first, ?optional=10, @rest}
     // handle mixed parameters
   endfn
   ```

## Known Issues

1. **Multi-line Block Functions in REPL**: The REPL's line-by-line parsing
   doesn't properly handle multi-line block functions. They work in single-line
   form and in .eval mode.

2. **Block Function Statement Parsing**: Inside block functions, statements like
   `let x = ...` are parsed but not properly converted to AST nodes.

## Testing

See `test_lambda_cases.txt` for comprehensive test cases covering:

- Arrow functions
- Block functions
- Closures
- Higher-order functions
- Nested functions
- Edge cases

## Implementation Files

- **Grammar**: `src/parser/echo/grammar.rs`
- **Parser**: `src/parser/echo/mod.rs`
- **AST**: `src/ast/mod.rs`
- **Evaluator**: `src/evaluator/mod.rs`
