# Named Fields Reference

## Overview

This document provides a comprehensive reference for all named fields used in the tree-sitter-moo grammar. Named fields provide semantic labels for important child nodes, enabling programmatic access to specific parts of the syntax tree.

## Table of Contents

- [Field Access Patterns](#field-access-patterns)
- [Fields by Category](#fields-by-category)
- [Complete Field Reference](#complete-field-reference)
- [Usage Examples](#usage-examples)
- [API Examples](#api-examples)

## Field Access Patterns

### Single Fields
Most named fields contain a single child node:
```javascript
const name = functionNode.childForFieldName('name');
const condition = ifNode.childForFieldName('condition');
```

### Multiple Fields (Same Name)
Some fields can appear multiple times (e.g., `parameters` in `binding_pattern`):
```javascript
const parameters = bindingNode.childrenForFieldName('parameters');
parameters.forEach(param => { /* process each parameter */ });
```

### Optional Fields
Many fields are optional and may return `null`:
```javascript
const label = breakNode.childForFieldName('label'); // May be null
if (label) {
    console.log('Labeled break:', label.text);
}
```

## Fields by Category

### Identifiers and Names
Fields that contain identifier nodes for names, labels, and references.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `name` | binding_target, binding_optional, binding_rest, symbol, sysprop, global_assignment, fn_statement, object_definition, object_property, property_definition, property_attrs, verb_attr | identifier | No | Primary identifier/name |
| `label` | break, continue, while, fork | identifier | Yes | Statement label for control flow |
| `variable` | for, except | identifier | Yes | Loop or exception variable |
| `symbol` | objid | identifier | No | Symbolic object identifier |

### Expressions and Values
Fields that contain expression nodes.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `value` | return, local_assignment, const_assignment, global_assignment, argument, pair, object_property, property_definition, property_attrs, verb_attr | _expr | Yes/No* | Expression value |
| `condition` | if, _elseif, while, conditional_expr | _expr | No | Conditional expression |
| `expression` | scatter_element, for_iterable, fork, try_expr | _expr | No | General expression |
| `default` | binding_optional | _expr | Yes | Default parameter value |
| `fallback` | try_expr | _expr | Yes | Fallback expression for try |

*Optional depends on context

### Binary Operations
Fields specific to binary and unary operations.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `left` | binary_expr, assignment_expr | _expr | No | Left operand or assignment target |
| `right` | binary_expr, assignment_expr | _expr | No | Right operand or assignment value |
| `operator` | binary_expr, unary_expr | token | No | Operation operator |
| `operand` | unary_expr | _expr | No | Unary operation operand |

### Ternary Conditional
Fields for conditional expressions.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `condition` | conditional_expr | _expr | No | Condition to test |
| `consequence` | conditional_expr | _expr | No | Value when condition is true |
| `alternative` | conditional_expr | _expr | No | Value when condition is false |

### Access and Calls
Fields for property access, method calls, and function calls.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `object` | property_access, method_call, index_access, slice | _expr | No | Object being accessed |
| `property` | property_access | identifier/_expr | No | Property being accessed |
| `method` | method_call | identifier/_expr | No | Method being called |
| `function` | call | identifier/_expr | No | Function being called |
| `arguments` | method_call, call, arglist, pass | arglist/argument* | Yes | Function/method arguments |
| `index` | index_access | _index_expr | No | Array/object index |
| `from` | slice | _index_expr | No | Slice start index |
| `to` | slice | _index_expr | No | Slice end index |

### Data Structure Contents
Fields for collections and their contents.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `elements` | list | _list_element* | Yes | List elements |
| `entries` | map | pair* | Yes | Map key-value pairs |
| `key` | pair | _expr | No | Map entry key |
| `parameters` | binding_pattern | binding_target/binding_optional/binding_rest* | Yes | Binding parameters (flattened) |
| `params` | lambda_expr, fn_expr, fn_statement | lambda_params | Yes | Function parameters |

### Control Flow Bodies
Fields containing statement bodies for control structures.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `body` | block, if, _elseif, _else, while, for, fork, try, except, fn_expr, fn_statement | _statement* | No | Statement sequence |
| `elseif_clauses` | if | _elseif* | No | Elseif clauses (may be empty) |
| `else_clause` | if | _else | Yes | Else clause |

### Range and Iteration
Fields for ranges and iteration constructs.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `start` | for_iterable | _expr | No | Range start value |
| `end` | for_iterable | _expr | No | Range end value |
| `iterable` | for | for_iterable | No | Expression to iterate over |

### Assignment Targets
Fields for various assignment contexts.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `target` | local_assignment, const_assignment | identifier/binding_pattern | No | Assignment target |
| `splat` | argument | _expr | No | Splat argument expression |

### Object Definition
Fields specific to object definition syntax.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `dobj` | verb_args | identifier | No | Direct object parameter |
| `prep` | verb_args | identifier | No | Preposition parameter |
| `iobj` | verb_args | identifier | No | Indirect object parameter |

### Flyweight Objects
Fields for flyweight object construction.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `parent` | flyweight | _expr | No | Parent object reference |
| `properties` | flyweight | map | Yes | Property overrides |
| `values` | flyweight | list | Yes | Value overrides |

### Error Handling
Fields for exception handling.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `codes` | except, try_expr | _try_expr_codes | No | Exception codes to catch |
| `finally` | try | special | Yes | Finally block (has nested body) |

### Object Literals
Fields for special object references.

| Field | Used In | Type | Optional | Description |
|-------|---------|------|----------|-------------|
| `number` | objid | INTEGER | No | Numeric object ID |

## Complete Field Reference

### Alphabetical Field List

| Field Name | Production Rules | Multiplicity | Type | Description |
|------------|------------------|--------------|------|-------------|
| `alternative` | conditional_expr | Single | _expr | False branch of ternary operator |
| `arguments` | method_call, call, arglist, pass | Single | arglist/argument* | Argument list for calls |
| `body` | block, if, _elseif, _else, while, for, fork, try, except, fn_expr, fn_statement | Single | _statement* | Statement body/sequence |
| `codes` | except, try_expr | Single | _try_expr_codes | Exception error codes |
| `condition` | if, _elseif, while, conditional_expr | Single | _expr | Boolean condition expression |
| `consequence` | conditional_expr | Single | _expr | True branch of ternary operator |
| `default` | binding_optional | Single | _expr | Default value for optional parameter |
| `dobj` | verb_args | Single | identifier | Direct object verb parameter |
| `elements` | list | Single | _list_element* | Elements within list literal |
| `elseif_clauses` | if | Single | _elseif* | Collection of elseif clauses |
| `else_clause` | if | Single | _else | Optional else clause |
| `end` | for_iterable | Single | _expr | End value for range iteration |
| `entries` | map | Single | pair* | Key-value pairs in map literal |
| `expression` | scatter_element, for_iterable, fork, try_expr | Single | _expr | Expression in various contexts |
| `fallback` | try_expr | Single | _expr | Fallback value for failed try expression |
| `finally` | try | Single | special | Finally block with nested body field |
| `from` | slice | Single | _index_expr | Start index for slice operation |
| `function` | call | Single | identifier/_expr | Function being called |
| `index` | index_access | Single | _index_expr | Index for array/object access |
| `iobj` | verb_args | Single | identifier | Indirect object verb parameter |
| `iterable` | for | Single | for_iterable | Iterable expression for for loop |
| `key` | pair | Single | _expr | Key in key-value pair |
| `label` | break, continue, while, fork | Single | identifier | Optional label for control flow |
| `left` | binary_expr, assignment_expr | Single | _expr | Left operand or assignment target |
| `method` | method_call | Single | identifier/_expr | Method being called |
| `name` | binding_target, binding_optional, binding_rest, symbol, sysprop, global_assignment, fn_statement, object_definition, object_property, property_definition, property_attrs, verb_attr | Single | identifier | Primary identifier or name |
| `number` | objid | Single | INTEGER | Numeric object identifier |
| `object` | property_access, method_call, index_access, slice | Single | _expr | Object being accessed |
| `operand` | unary_expr | Single | _expr | Operand for unary operation |
| `operator` | binary_expr, unary_expr | Single | token | Operation operator symbol |
| `parameters` | binding_pattern | **Multiple** | binding_target/binding_optional/binding_rest | Flattened parameter list |
| `params` | lambda_expr, fn_expr, fn_statement | Single | lambda_params | Function parameter specification |
| `parent` | flyweight | Single | _expr | Parent object for flyweight |
| `prep` | verb_args | Single | identifier | Preposition verb parameter |
| `properties` | flyweight | Single | map | Property overrides for flyweight |
| `property` | property_access | Single | identifier/_expr | Property being accessed |
| `right` | binary_expr, assignment_expr | Single | _expr | Right operand or assignment value |
| `splat` | argument | Single | _expr | Splat argument expression |
| `start` | for_iterable | Single | _expr | Start value for range iteration |
| `symbol` | objid | Single | identifier | Symbolic object identifier |
| `target` | local_assignment, const_assignment | Single | identifier/binding_pattern | Target of assignment |
| `to` | slice | Single | _index_expr | End index for slice operation |
| `value` | return, local_assignment, const_assignment, global_assignment, argument, pair, object_property, property_definition, property_attrs, verb_attr | Single | _expr | Value expression in various contexts |
| `values` | flyweight | Single | list | Value overrides for flyweight |
| `variable` | for, except | Single | identifier | Loop or exception variable |

## Usage Examples

### Accessing Simple Fields
```moo
let x = 5;
```

**Tree Structure:**
```
local_assignment
├── target: identifier("x")
└── value: INTEGER("5")
```

**API Access:**
```javascript
const target = assignmentNode.childForFieldName('target');
const value = assignmentNode.childForFieldName('value');
console.log(target.text); // "x"
console.log(value.text);  // "5"
```

### Accessing Multiple Fields
```moo
{a, ?b = 10, @rest} = data;
```

**Tree Structure:**
```
binding_pattern
├── parameters: binding_target(name: "a")
├── parameters: binding_optional(name: "b", default: 10)
└── parameters: binding_rest(name: "rest")
```

**API Access:**
```javascript
const parameters = bindingNode.childrenForFieldName('parameters');
parameters.forEach((param, index) => {
    const name = param.childForFieldName('name');
    console.log(`Parameter ${index}: ${name.text} (${param.type})`);
    
    if (param.type === 'binding_optional') {
        const defaultValue = param.childForFieldName('default');
        if (defaultValue) {
            console.log(`  Default: ${defaultValue.text}`);
        }
    }
});
```

### Accessing Optional Fields
```moo
break outer;
```

**Tree Structure:**
```
break
└── label: identifier("outer")
```

**API Access:**
```javascript
const label = breakNode.childForFieldName('label');
if (label) {
    console.log('Labeled break to:', label.text);
} else {
    console.log('Unlabeled break');
}
```

### Accessing Nested Fields
```moo
if (condition)
    statement1;
else
    statement2;
endif
```

**Tree Structure:**
```
if
├── condition: identifier("condition")
├── body: [statement1]
├── elseif_clauses: []
└── else_clause: _else
    └── body: [statement2]
```

**API Access:**
```javascript
const condition = ifNode.childForFieldName('condition');
const body = ifNode.childForFieldName('body');
const elseClause = ifNode.childForFieldName('else_clause');

if (elseClause) {
    const elseBody = elseClause.childForFieldName('body');
    console.log('Has else clause with', elseBody.childCount, 'statements');
}
```

## API Examples

### Tree-sitter Node.js API
```javascript
const Parser = require('tree-sitter');
const MOO = require('tree-sitter-moo');

const parser = new Parser();
parser.setLanguage(MOO);

const code = 'let {x, ?y = 10} = getData();';
const tree = parser.parse(code);

function visitNode(node, depth = 0) {
    const indent = '  '.repeat(depth);
    console.log(`${indent}${node.type}`);
    
    // Print named fields
    if (node.fieldNames.length > 0) {
        for (const fieldName of node.fieldNames) {
            const children = node.childrenForFieldName(fieldName);
            for (const child of children) {
                console.log(`${indent}  ${fieldName}: ${child.type}`);
            }
        }
    }
    
    // Recursively visit children
    for (const child of node.children) {
        visitNode(child, depth + 1);
    }
}

visitNode(tree.rootNode);
```

### Tree-sitter CLI Queries
```scheme
; Query for all function definitions with their names
(fn_statement
  name: (identifier) @function.name
  params: (lambda_params) @function.params
  body: (_) @function.body)

; Query for all binding patterns with parameter details
(binding_pattern
  parameters: [
    (binding_target name: (identifier) @param.regular)
    (binding_optional name: (identifier) @param.optional 
                     default: (_) @param.default)
    (binding_rest name: (identifier) @param.rest)
  ])

; Query for method calls with object and method
(method_call
  object: (_) @call.object
  method: (identifier) @call.method
  arguments: (arglist) @call.args)
```

### Python Tree-sitter API
```python
import tree_sitter_moo as ts_moo
from tree_sitter import Language, Parser

MOO_LANGUAGE = Language(ts_moo.language(), 'moo')
parser = Parser()
parser.set_language(MOO_LANGUAGE)

code = b'let handler = {x, y} => x + y;'
tree = parser.parse(code)

def extract_field(node, field_name):
    """Extract named field from node."""
    field_child = node.child_by_field_name(field_name.encode())
    return field_child.text.decode() if field_child else None

def print_assignment(node):
    """Print details of assignment node."""
    if node.type == 'local_assignment':
        target = extract_field(node, 'target')
        value_node = node.child_by_field_name(b'value')
        
        print(f"Assignment target: {target}")
        if value_node and value_node.type == 'lambda_expr':
            params_node = value_node.child_by_field_name(b'params')
            body_node = value_node.child_by_field_name(b'body')
            print(f"Lambda params: {params_node.text.decode()}")
            print(f"Lambda body: {body_node.text.decode()}")

# Walk the tree and process assignments
def walk_tree(node):
    if node.type == 'local_assignment':
        print_assignment(node)
    
    for child in node.children:
        walk_tree(child)

walk_tree(tree.root_node)
```

This comprehensive reference enables effective programmatic manipulation of tree-sitter-moo syntax trees through named field access patterns.