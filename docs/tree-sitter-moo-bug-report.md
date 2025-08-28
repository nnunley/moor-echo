# Bug Report: tree-sitter-moo Grammar Issues

## Summary

The tree-sitter-moo grammar has several parsing issues with valid MOO language constructs, resulting in ERROR nodes or incorrect parse trees for common MOO patterns.

## Environment

- tree-sitter-moo version: (latest as of the test)
- tree-sitter version: 0.25
- Test files: MOO dump files from the cowbell MOO codebase

## Issues Found

### 1. Define Statements Parsed Incorrectly

**Description**: Define statements are not recognized as a distinct construct. Instead, they are parsed as two separate statements: an identifier "define" followed by an assignment.

**Example MOO code**:
```moo
define FAILED_MATCH = #-3;
define SYSOBJ = #0;
```

**Current parse tree**:
```
program
  statement
    expression_statement
      expression
        identifier "define"
  statement  
    expression_statement
      expression
        assignment_operation
          identifier "FAILED_MATCH"
          = "="
          expression
            object_id "#-3"
```

**Expected**: A single `define_statement` node or similar that captures the entire construct.

### 2. Error Catching Expressions (Backtick Syntax) Not Supported

**Description**: MOO's error catching expression syntax using backticks results in ERROR nodes.

**Example MOO code**:
```moo
`foo.bar ! E_PROPNF => "default"'
`caller_perms() ! ANY'
```

**Current parse tree**: Results in ERROR nodes

**Expected**: Proper parsing as error_catching_expression with:
- The expression to evaluate
- The error codes to catch (E_PROPNF, ANY, etc.)
- The default value expression

### 3. Flyweight Object Syntax Not Recognized

**Description**: MOO's flyweight object syntax `<object, [properties]>` is not parsed correctly.

**Example MOO code**:
```moo
<#1, ["name" -> "test", "value" -> 42]>
```

**Current parse tree**: Results in ERROR nodes

**Expected**: A flyweight_object node with:
- The base object reference
- The property overrides as a list

### 4. Map Literal Syntax Conflicts

**Description**: MOO map literals using `[key -> value]` syntax are not distinguished from lists.

**Example MOO code**:
```moo
["key1" -> "value1", "key2" -> 42]
```

**Current parse tree**: May be parsed as a list with binary operations

**Expected**: A distinct map_literal node with key-value pairs

### 5. Destructuring Assignment Not Supported

**Description**: MOO's destructuring assignment with curly braces is not recognized.

**Example MOO code**:
```moo
{a, b, c} = args;
{first, @rest} = some_list;
```

**Current parse tree**: Results in ERROR nodes

**Expected**: A destructuring_assignment node with:
- List of target variables
- Support for rest syntax (@rest)

### 6. Spread Operator Not Recognized

**Description**: The spread operator `...` is not parsed correctly in function calls.

**Example MOO code**:
```moo
foo(a, b, ...rest_args)
```

**Current parse tree**: Results in ERROR nodes

**Expected**: Recognition of spread syntax in argument lists

## Impact

These parsing issues prevent proper static analysis, syntax highlighting, and code transformation tools from working correctly with MOO code. In our use case, we had to implement extensive error recovery logic to handle these cases when building a MOO-to-Echo transpiler.

## Suggested Fix

The grammar should be extended to include:

1. **define_statement** rule:
```js
define_statement: $ => seq(
  'define',
  $.identifier,
  '=',
  $.expression,
  optional(';')
)
```

2. **error_catching_expression** rule:
```js
error_catching_expression: $ => seq(
  '`',
  $.expression,
  '!',
  choice(
    $.error_code,
    seq($.error_code, repeat(seq(',', $.error_code)))
  ),
  optional(seq('=>', $.expression)),
  "'"
)
```

3. **flyweight_object** rule:
```js
flyweight_object: $ => seq(
  '<',
  $.expression,  // base object
  ',',
  '[',
  optional($.property_list),
  ']',
  '>'
)
```

4. **map_literal** rule distinct from list:
```js
map_literal: $ => seq(
  '[',
  optional(seq(
    $.map_entry,
    repeat(seq(',', $.map_entry))
  )),
  ']'
),

map_entry: $ => seq(
  $.expression,
  '->',
  $.expression
)
```

5. **destructuring_assignment** rule:
```js
destructuring_assignment: $ => seq(
  '{',
  $.destructuring_pattern,
  '}',
  '=',
  $.expression
)
```

## Workaround

Currently, we handle these cases by:
1. Detecting ERROR nodes and parsing their text content manually
2. Looking for specific patterns in the parse tree (e.g., "define" identifier followed by assignment)
3. Implementing recovery logic to construct the correct AST nodes

## Test Files

The issues can be reproduced with MOO files from the cowbell MOO codebase, particularly:
- `constants.moo` - demonstrates define statement issues
- `sub.moo` - demonstrates error catching and other advanced features
