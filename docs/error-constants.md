# MOO Error Constants

## Case Sensitivity

According to the LambdaMOO Programmer's Manual, error constants in MOO are **case-insensitive**, following the general MOO principle that "the case of the letters in variable names is insignificant."

This means that all of the following are equivalent:
- `E_TYPE`
- `e_type`
- `E_Type`
- `e_TyPe`

The tree-sitter-moo parser correctly implements this behavior by using case-insensitive pattern matching for all error constants.

## Complete List of Error Constants

The following error constants are defined in the MOO language:

| Constant | Description |
|----------|-------------|
| `E_NONE` | No error |
| `E_TYPE` | Type mismatch |
| `E_DIV` | Division by zero |
| `E_PERM` | Permission denied |
| `E_PROPNF` | Property not found |
| `E_VERBNF` | Verb not found |
| `E_VARNF` | Variable not found |
| `E_INVIND` | Invalid indirection |
| `E_RECMOVE` | Recursive move |
| `E_MAXREC` | Too many verb calls |
| `E_RANGE` | Range error |
| `E_ARGS` | Incorrect number of arguments |
| `E_NACC` | Move refused by destination |
| `E_INVARG` | Invalid argument |
| `E_QUOTA` | Resource limit exceeded |
| `E_FLOAT` | Floating-point arithmetic error |
| `E_ASSERT` | Assertion failed |

## Implementation Details

In the tree-sitter-moo grammar, error constants are defined using a `caseInsensitive()` helper function that generates regex patterns accepting any case combination. The parser then aliases these patterns to their canonical uppercase form (e.g., `E_TYPE`).

This ensures that while the parser accepts any case variation during parsing, the AST always contains the normalized uppercase form for consistency.

## Usage Examples

```moo
// All of these are valid and equivalent:
if (error == E_TYPE)
  ...
endif

if (error == e_type)
  ...
endif

if (error == E_Type)
  ...
endif

// In try-except blocks:
try
  // some code
except e (E_PERM, e_propnf, E_Invarg)  // Mixed case is fine
  // handle errors
endtry

// In scatter assignments with defaults:
let {?code = e_none, @args} = data;  // lowercase e_none is valid
```

## Parser Implementation Note

In the current tree-sitter-moo grammar, error constants are only recognized as `ERR` tokens in specific contexts:
- Inside `except` clauses (e.g., `except e (E_PERM, E_DIV)`)
- Inside try expressions (e.g., `` `x.y ! E_PROPNF => 0' ``)

In other contexts (like regular expressions or variable assignments), error constants are parsed as regular `identifier` tokens. This is a grammar implementation detail and doesn't affect the semantic meaning, since MOO treats these identifiers as error constants at runtime when appropriate.

For example:
```moo
// Parsed as ERR token:
except e (E_TYPE)

// Parsed as identifier token:
x = E_TYPE;
let {?code = E_NONE} = data;
```

Both are semantically valid MOO code, just represented differently in the AST.