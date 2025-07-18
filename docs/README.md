# Tree-sitter-moo Documentation

## Overview

This directory contains comprehensive documentation for the tree-sitter-moo parser, including CST structure, named fields, production rules, and usage examples.

## Documentation Index

### Core References
- **[CST Reference](CST_REFERENCE.md)** - Complete CST hierarchy documentation with tree structures and named fields
- **[Named Fields Reference](NAMED_FIELDS.md)** - Comprehensive guide to all named fields and their usage
- **[Production Reference](PRODUCTION_REFERENCE.md)** - Quick reference for all grammar production rules
- **[CST Examples](CST_EXAMPLES.md)** - Concrete examples of CST structures for all language constructs

## Quick Navigation

### By Language Feature

#### Basic Constructs
- [Variable Assignments](CST_REFERENCE.md#assignment-statements) - `local_assignment`, `const_assignment`, `global_assignment`
- [Expressions](CST_REFERENCE.md#expressions) - Binary, unary, conditional expressions
- [Control Flow](CST_REFERENCE.md#statements) - `if`, `while`, `for`, `try`, `break`, `continue`, `return`

#### Modern Binding System
- [Binding Patterns](CST_REFERENCE.md#binding-patterns) - Unified parameter system
- [Flattened Structure](CST_REFERENCE.md#binding-patterns) - Direct parameter access pattern
- [Parameter Types](NAMED_FIELDS.md#binding-patterns) - `binding_target`, `binding_optional`, `binding_rest`

#### Functions and Lambdas
- [Lambda Expressions](CST_REFERENCE.md#lambda_expr) - Arrow functions with `{} =>`
- [Function Expressions](CST_REFERENCE.md#fn_expr) - Anonymous functions
- [Function Statements](CST_REFERENCE.md#fn_statement) - Named functions
- [Parameter Lists](CST_REFERENCE.md#lambda_params) - Shared parameter structure

#### Data Structures
- [Lists](CST_REFERENCE.md#list) - List literals with `{}`
- [Maps](CST_REFERENCE.md#map) - Map literals with `[]`
- [Flyweights](CST_REFERENCE.md#flyweight) - Flyweight objects with `<>`

#### Object Access
- [Property Access](CST_REFERENCE.md#property_access) - Static and dynamic property access
- [Method Calls](CST_REFERENCE.md#method_call) - Method invocation with `:`
- [Indexing](CST_REFERENCE.md#index_access) - Array/object indexing
- [Slicing](CST_REFERENCE.md#slice) - Array slicing with ranges

### By Use Case

#### Language Tool Development
- [Named Fields API](NAMED_FIELDS.md#api-examples) - Programmatic access patterns
- [Tree-sitter Queries](NAMED_FIELDS.md#tree-sitter-cli-queries) - Query examples for analysis
- [Field Access Patterns](NAMED_FIELDS.md#field-access-patterns) - Single, multiple, and optional fields

#### Parser Integration
- [Node.js Integration](NAMED_FIELDS.md#tree-sitter-nodejs-api) - JavaScript API examples
- [Python Integration](NAMED_FIELDS.md#python-tree-sitter-api) - Python API examples
- [Production Rules](PRODUCTION_REFERENCE.md) - Grammar categories and precedence

#### Code Analysis
- [Complete Examples](CST_EXAMPLES.md) - Real CST structures for major constructs
- [Complex Expressions](CST_EXAMPLES.md#complex-expressions) - Advanced parsing examples
- [Field Reference](NAMED_FIELDS.md#complete-field-reference) - All 35+ named fields

## Key Features Highlighted

### Unified Binding System
The tree-sitter-moo parser features a modern unified binding system:

```moo
// All use the same binding structure:
let {a, ?b = 10, @rest} = list;           // Destructuring assignment  
let handler = {a, ?b = 10, @rest} => ...;  // Lambda parameters
fn process(a, ?b = 10, @rest) ... endfn    // Function parameters
```

**CST Benefits:**
- **Flattened structure**: Direct access to parameters via `binding_pattern.parameters[i]`
- **Consistent semantics**: Same binding types work everywhere
- **Simplified tooling**: Easier tree-sitter queries and analysis

### Comprehensive Named Fields
Every production rule uses named fields for semantic access:

```javascript
// JavaScript API example
const target = assignmentNode.childForFieldName('target');
const value = assignmentNode.childForFieldName('value');
const parameters = bindingNode.childrenForFieldName('parameters');
```

### Modern Grammar Features
- **Context-sensitive parsing**: `{}` resolves to list or lambda params based on context
- **Dynamic access**: `obj.(expr)` and `(expr)(args)` for computed access
- **Rich precedence**: 9 precedence levels for binary operators
- **Comprehensive coverage**: All MOO language features supported

## Getting Started

1. **Start with [CST Reference](CST_REFERENCE.md)** for complete overview
2. **Review [Named Fields](NAMED_FIELDS.md)** for programmatic access
3. **Browse [Examples](CST_EXAMPLES.md)** for specific constructs
4. **Use [Production Reference](PRODUCTION_REFERENCE.md)** for quick lookup

## Development Notes

### Grammar Evolution
The grammar has evolved to use modern "binding" terminology instead of "scatter" to better reflect the unified parameter system used across multiple language constructs.

### Performance Considerations
- **Flattened binding patterns** reduce tree depth and improve access performance
- **Named fields** enable efficient programmatic navigation
- **Comprehensive conflicts** resolve ambiguities for reliable parsing

### Testing Coverage
The parser includes 69 comprehensive tests covering:
- All statement types and expressions
- Binding patterns in all contexts  
- Complex nested structures
- Edge cases and error conditions
- Object definition syntax (.objdef files)

## Contributing

When extending the grammar or documentation:
1. Update production rules in `grammar.js`
2. Add corresponding test cases
3. Update documentation in this directory
4. Regenerate parser with `npm run build`
5. Verify with `npm test`

For questions about CST structure or named fields, refer to the comprehensive references in this directory.