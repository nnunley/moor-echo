# MOO Language Notes

This document contains notes about the MOO programming language based on the LambdaMOO Programmer's Manual and implementation details.

## Case Sensitivity

MOO is generally **case-insensitive** for:
- Keywords (`if`, `IF`, `If` are all valid)
- Error constants (`E_TYPE`, `e_type`, `E_Type` are equivalent)
- Variable names (according to the manual: "the case of the letters in variable names is insignificant")
- Built-in function names

## Expression Hierarchy

The MOO language has a well-defined operator precedence (from lowest to highest):

1. Assignment: `=` (right associative)
2. Conditional: `? |` (ternary operator)
3. Logical OR: `||`
4. Logical AND: `&&`
5. Equality/Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`
6. Bitwise OR/AND/XOR: `|.`, `&.`, `^.`
7. Bit shift: `<<`, `>>`
8. Addition/Subtraction: `+`, `-`
9. Multiplication/Division/Modulo: `*`, `/`, `%`
10. Exponentiation: `^` (right associative)
11. Unary: `!`, `-` (logical not, negation)
12. Postfix: `.`, `:`, `[]` (property access, verb call, indexing)

## Special Syntax

### Flyweight Objects
Flyweight objects use angle bracket syntax:
```moo
<#123>                    // Basic flyweight
<#123, [prop -> value]>   // With property map
<#123, [], {1, 2, 3}>     // With property map and verb list
```

### Scatter Assignment
MOO supports destructuring assignment:
```moo
{a, b, c} = {1, 2, 3};              // Basic scatter
{x, ?y = 10, @rest} = data;         // With optional and rest
```

### Try-Except
Error handling with specific error codes:
```moo
try
  // code that might error
except e (E_TYPE, E_DIV)
  // handle specific errors
except (ANY)
  // handle any error
finally
  // cleanup code
endtry
```

### Lambda and Function Expressions
```moo
// Lambda expression
{x, y} => x + y

// Function expression
fn(x, y)
  return x + y;
endfn

// Function statement
fn factorial(n)
  return n <= 1 ? 1 | n * factorial(n - 1);
endfn
```

## Object Definition Syntax

MOO supports an extended syntax for defining objects (not part of standard LambdaMOO):
```moo
object MyObject
  property name (owner: #1, flags: "r") = "unnamed";
  
  verb initialize (this none this)
    this.name = "initialized";
  endverb
endobject
```

## Built-in Values

- Boolean: `true`, `false` (case-sensitive keywords)
- Special value: `$` (length marker in indexing/slicing contexts)
  - In `arr[$]`, `$` evaluates to the length of `arr`
  - In `str[1..$]`, `$` evaluates to the length of `str`
  - Can be used in expressions: `arr[$-1]` for the last element
- Object IDs: `#123`, `#-1`, `#"string_id"`
- System properties: `$list`, `$string`, etc.

## Comments

MOO supports two comment styles:
```moo
// Single-line comment
/* Multi-line
   comment */
```

## Tree-sitter AST Node Types

The tree-sitter-moo parser uses semantic node types for better AST representation:

- **`call`**: Function calls like `foo(args)`
  - Fields: `function`, `arguments`
- **`method_call`**: Verb/method calls like `obj:method(args)`
  - Fields: `object`, `method`, `arguments`
- **`property_access`**: Property access like `obj.prop`
  - Fields: `object`, `property`
- **`index_access`**: Array/list indexing like `arr[5]`
  - Fields: `object`, `index`
- **`slice`**: Array/string slicing like `str[1..3]`
  - Fields: `object`, `from`, `to`

These semantic nodes replaced the generic `postfix_expr` for clearer AST structure.