# Grammar Comparison: Echo vs MOO

This document compares the Echo language grammar with the MOO language grammar (from tree-sitter-moo).

## Overview

- **MOO Grammar**: Complete implementation including all MOO features
- **Echo Grammar**: Simplified subset focusing on core features

## Major Differences

### 1. List Syntax
- **MOO**: Uses `{}` for lists (e.g., `{1, 2, 3}`)
- **Echo**: Uses `[]` for lists (e.g., `[1, 2, 3]`)
- **Reason**: Echo follows modern language conventions

### 2. Object Syntax
- **MOO**: Traditional MOO syntax with verb definitions
- **Echo**: Simplified `object...endobject` blocks

### 3. Missing in Echo

#### Core Language Features
1. **List Comprehensions**
   - MOO: `{x * 2 for x in [1..10]}`
   - Echo: Not implemented

2. **Range Syntax**
   - MOO: `[1..10]` for ranges
   - Echo: Not implemented

3. **Maps/Dictionaries**
   - MOO: `[key -> value, ...]` syntax
   - Echo: Mentioned in AST but no parser support

4. **Try-Catch Expressions**
   - MOO: `` `expression ! codes => fallback' ``
   - Echo: Not implemented

5. **Scatter/Splat Operations**
   - MOO: `@variable` for scatter in lists and calls
   - Echo: Partially implemented (binding patterns exist)

#### Control Flow
1. **Fork Statements**
   - MOO: `fork (delay) ... endfork` for async execution
   - Echo: Not implemented

2. **Pass Expression**
   - MOO: `pass(args)` to call parent verb
   - Echo: Not implemented

3. **Labeled Loops**
   - MOO: `while label (condition) ... endwhile`
   - Echo: Labels in AST but not in parser

#### Object System
1. **Verb Argument Specifications**
   - MOO: `verb name(dobj prep iobj)` with preposition support
   - Echo: Simplified `verb name {params}` syntax

2. **Property Attributes**
   - MOO: Properties with access permissions
   - Echo: Simple property definitions only

3. **Object IDs**
   - MOO: Both `#123` and `#name` syntax
   - Echo: Only numeric `#123` syntax

#### Advanced Features
1. **Flyweight Objects**
   - MOO: `<parent, properties, values>` syntax
   - Echo: Not implemented

2. **Error Codes**
   - MOO: Built-in error constants (E_TYPE, E_PERM, etc.)
   - Echo: Not implemented

3. **Symbol Literals**
   - MOO: `'symbol` syntax
   - Echo: Not implemented

4. **Binary Operators**
   - MOO: Bitwise operators (`|.`, `&.`, `^.`, `<<`, `>>`)
   - Echo: Not implemented

## Grammar Structure Comparison

### MOO Grammar Structure
```javascript
program: choice(
  object_definition,    // Object definition file
  repeat(statement)     // Traditional MOO program
)

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

expression: // Flat hierarchy with precedence
```

### Echo Grammar Structure
```rust
EchoAst: enum {
  // Literals
  // Binary operations with precedence
  // Control flow
  // Object definitions
}
```

## Operator Precedence

### MOO Precedence (from lowest to highest)
1. `=` (assignment) - right associative
2. `?` `|` (conditional) - non-associative
3. `||` `&&` (logical) - left associative
4. `==` `!=` `<` `<=` `>` `>=` `in` - left associative
5. `|.` `&.` `^.` (bitwise) - left associative
6. `<<` `>>` (bit shift) - left associative
7. `+` `-` - left associative
8. `*` `/` `%` - left associative
9. `^` (power) - right associative
10. `!` `~` `-` (unary) - left associative
11. `.` `:` `[` `$` (access) - non-associative

### Echo Precedence
- Currently implements arithmetic precedence (7-8)
- Missing many precedence levels
- No bitwise operators

## Recommendations for Echo

### High Priority Additions
1. **List Comprehensions**: Essential MOO feature
2. **Range Syntax**: Widely used in MOO code
3. **Try-Catch Expressions**: Error handling
4. **Fork Statements**: Async execution
5. **Error Constants**: Built-in error handling

### Medium Priority
1. **Maps/Dictionaries**: Modern data structure
2. **Symbol Literals**: Useful for keys
3. **Pass Expression**: OOP feature
4. **Labeled Loops**: Control flow
5. **Scatter/Splat**: Destructuring

### Low Priority
1. **Flyweight Objects**: Advanced feature
2. **Bitwise Operators**: Less commonly used
3. **Object name references**: `#name` syntax

## Implementation Notes

The MOO grammar uses a more sophisticated approach:
- Separate rules for each construct
- Clear precedence table
- Helper functions for case-insensitive keywords
- Conflict resolution declarations

Echo could benefit from:
- Adopting MOO's precedence structure
- Adding missing operators
- Implementing case-insensitive keywords
- Supporting full MOO compatibility mode