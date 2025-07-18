# Production Rule Quick Reference

## Grammar Categories

### Program Structure
- `program` - Root node (object_definition OR statements*)

### Statements
- `break` - Break statement with optional label
- `continue` - Continue statement with optional label  
- `return` - Return statement with optional value
- `block` - Begin/end block statement
- `if` / `_elseif` / `_else` - Conditional statements
- `while` - While loop with optional label
- `for` / `for_iterable` - For-in loop
- `fork` - Fork statement for parallelism
- `try` / `except` - Exception handling
- `local_assignment` - Let variable assignment
- `const_assignment` - Const variable assignment
- `global_assignment` - Global variable declaration

### Expressions  
- `assignment_expr` - Assignment expression
- `conditional_expr` - Ternary conditional (? |)
- `binary_expr` - Binary operations (+, -, *, etc.)
- `unary_expr` - Unary operations (!, -)
- `property_access` - Object property access (.)
- `method_call` - Method calls (:)
- `index_access` - Array indexing ([])
- `slice` - Array slicing ([..])
- `call` - Function calls
- `try_expr` - Try expressions (` ! ')

### Data Structures
- `list` - List literals ({})
- `scatter_element` - Scatter in lists (@expr)  
- `map` - Map literals ([])
- `pair` - Key-value pairs (->)
- `flyweight` - Flyweight objects (<>)

### Binding Patterns (Unified)
- `binding_pattern` - Parameter binding pattern
- `binding_target` - Simple parameter
- `binding_optional` - Optional parameter (?name = default)
- `binding_rest` - Rest parameter (@name)

### Functions
- `lambda_expr` - Lambda expressions ({} =>)
- `lambda_params` - Parameter lists for functions/lambdas
- `fn_expr` - Anonymous function expressions
- `fn_statement` - Named function statements

### Object Definition (.objdef)
- `object_definition` - Object definition block
- `object_property` - Simple property (name: value)
- `property_definition` - Full property with attributes
- `property_attrs` - Property attribute list
- `verb_definition` - Verb definition
- `verb_args` - Verb parameters (dobj, prep, iobj)
- `verb_attr` - Verb attributes

### Miscellaneous
- `range_comprehension` - {expr for var in range}
- `pass` - Pass statement
- `arglist` - Argument lists
- `argument` - Function arguments (value or @splat)
- `symbol` - Symbol literals ('name)
- `objid` - Object IDs (#123 or #name)
- `sysprop` - System properties ($name)

### Terminals
- `identifier` - Variable/function names
- `INTEGER` - Integer literals
- `FLOAT` - Floating point literals  
- `STRING` - String literals
- `ERR` - Error constants (E_PERM, etc.)
- `comment` - Comments (// and /* */)

## Key Features

### Modern Binding System
The grammar uses a unified binding system with:
- **Flattened structure**: `binding_pattern` has direct `parameters` children
- **Consistent semantics**: Same binding types work in assignments, lambdas, and functions
- **Three parameter types**: target, optional (?), rest (@)

### Precedence Levels
Binary expressions use precedence levels 3-9:
- Level 3: `||`, `&&` (logical)
- Level 4: `==`, `!=`, `<`, `<=`, `>`, `>=`, `in` (comparison)  
- Level 5: `|.`, `&.`, `^.` (bitwise)
- Level 6: `<<`, `>>` (shift)
- Level 7: `+`, `-` (additive)
- Level 8: `*`, `/`, `%` (multiplicative)
- Level 9: `^` (exponentiation, right associative)

### Context-Sensitive Grammar
- `{...}` can be list literal OR lambda parameters (resolved by context)
- Function calls can use dynamic names: `(expr)(args)`
- Property access supports dynamic properties: `obj.(expr)`

This reference provides quick lookup for all production rules in the tree-sitter-moo grammar.