# Echo SystemTracer

The Echo SystemTracer is a metaprogramming framework written in Echo/MOO that allows transformation of code within the live database. It's inspired by Squeak Smalltalk's SystemTracer but adapted for Echo's object-oriented database environment.

## Overview

The SystemTracer operates entirely within the Echo database, allowing you to:
- Define transformation rules in Echo code
- Apply transformations to live objects and their verbs
- Track transformation statistics
- Perform dry runs before applying changes
- Transform entire subsystems with dependency analysis

## Architecture

### Core Components

1. **$system_tracer** - The main tracer object that orchestrates transformations
2. **$transformation_rule** - Base class for all transformation rules
3. **Built-in Rules**:
   - `$property_syntax_fixer` - Converts MOO constants to property access
   - `$negative_ref_normalizer` - Handles negative object references
   - `$builtin_function_resolver` - Converts MOO builtins to method calls
   - `$capability_converter` - Migrates permission checks to capabilities
   - `$list_modernizer` - Updates old list operations
   - `$async_transformer` - Marks fork/endfork for async conversion

### Advanced Components

- **$subsystem_tracer** - Transforms entire object hierarchies
- **$pattern_engine** - Pattern-based code transformation
- **$meta_utils** - Metaprogramming utilities

## Basic Usage

### Setting Up the SystemTracer

```moo
// Initialize with default rules
$system:setup_system_tracer()

// Add a custom rule
$system_tracer:add_rule($my_custom_rule)

// List all rules
$system_tracer:list_rules()
```

### Transforming Objects

```moo
// Transform a single object
$system_tracer:transform_object($my_object)

// Transform all objects (dry run first!)
$system_tracer.dry_run = 1
$system_tracer:transform_system()

// Apply transformations for real
$system_tracer.dry_run = 0
$system_tracer:transform_system()
```

### Creating Custom Transformation Rules

```moo
// Create a new rule
@create $transformation_rule named "My Custom Rule"

// Set its properties
@prop #123.name "My Custom Rule"
@prop #123.description "Does something special"
@prop #123.priority 150

// Define the matches verb
@verb #123:matches this none this
  // Return 1 if this rule should apply to the code
  return index(args[1], "old_pattern");

// Define the transform verb
@verb #123:transform this none this
  // Transform the code and return it
  return substitute(args[1], "old_pattern", "new_pattern");
```

## Advanced Usage

### Subsystem Transformation

```moo
// Transform an entire subsystem with dependency ordering
$subsystem_tracer:transform_subsystem($my_subsystem_root, {})
```

### Pattern-Based Transformation

```moo
// Add transformation patterns
$pattern_engine:add_pattern(
  "player\\.location",
  "player:location()",
  "Convert property access to method call"
)

// Apply patterns to code
new_code = $pattern_engine:apply_patterns(old_code)
```

### Code Analysis

```moo
// Analyze code metrics
metrics = $meta_utils:analyze_code_metrics(verb_code($obj, "verbname"))
player:tell("Complexity: " + tostr(metrics["complexity"]))
```

## Example Transformations

### 1. Converting MOO Constants

Before:
```moo
if (player.wizard == WIZARD)
  owner = HACKER;
endif
```

After:
```moo
if (player.wizard == #0.WIZARD)
  owner = #0.HACKER;
endif
```

### 2. Modernizing List Operations

Before:
```moo
mylist = listappend(mylist, newitem);
mylist = listdelete(mylist, 3);
```

After:
```moo
mylist = {@mylist, newitem};
mylist = list_delete(mylist, 3);
```

### 3. Capability-Based Security

Before:
```moo
if (is_wizard(player))
  do_admin_stuff();
endif
```

After:
```moo
if (player:has_capability("wizard"))
  do_admin_stuff();
endif
```

## Best Practices

1. **Always do a dry run first** - Set `$system_tracer.dry_run = 1`
2. **Test on a single object** - Before transforming everything
3. **Check statistics** - Use `$system_tracer:show_stats()`
4. **Backup important code** - The tracer modifies live code
5. **Order rules by priority** - Higher priority rules run first
6. **Write idempotent rules** - Rules should be safe to run multiple times

## Verbosity Levels

- **0** - Silent
- **1** - Basic progress messages
- **2** - Object and verb transformation details  
- **3** - Individual rule application details

Set with: `$system_tracer.verbose = 2`

## Future Enhancements

Once Echo has full AST support and metaprogramming capabilities:

1. **True AST transformation** - Work with parsed AST instead of text
2. **Type-aware transformations** - Use type information for safer transforms
3. **Incremental transformation** - Transform only changed code
4. **Reversible transformations** - Undo capability
5. **Transformation composition** - Combine multiple transformations
6. **Live code migration** - Transform running code without downtime

## Integration with Development Workflow

```moo
// Developer's transformation workflow
@create $transformation_rule named "Fix My Legacy Code"
@verb #123:matches this none this
  return match(args[1], "my_old_api\\(");
@verb #123:transform this none this  
  return substitute(args[1], "my_old_api(", "my_new_api(");

// Test on one object
$system_tracer.dry_run = 1
$system_tracer:add_rule(#123)
$system_tracer:transform_object($my_test_object)

// Apply to all my objects
for obj in (player.owned_objects)
  $system_tracer:transform_object(obj)
endfor
```

This Echo-based SystemTracer provides a powerful foundation for code transformation and modernization within the live Echo environment, true to the spirit of Smalltalk's image-based development.