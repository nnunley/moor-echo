# Echo Bootstrap Code

This directory contains Echo/MOO code for bootstrapping the Echo environment with various system utilities and frameworks.

## SystemTracer Framework

The SystemTracer is a metaprogramming framework written in Echo/MOO that allows transformation of code within the live database. It's inspired by Squeak Smalltalk's SystemTracer but adapted for Echo's object-oriented database environment.

### Core Files

- **`system_tracer.moo`** - Main SystemTracer implementation
  - `$system_tracer` - Core transformation engine
  - `$transformation_rule` - Base class for transformation rules
  - Built-in transformation rules for common MOO→Echo conversions

- **`system_tracer_advanced.moo`** - Advanced features and utilities
  - `$subsystem_tracer` - Transform entire object hierarchies
  - `$pattern_engine` - Pattern-based transformations
  - `$meta_utils` - Metaprogramming utilities
  - Advanced transformation rules (capabilities, async, etc.)

- **`tracer_audit.moo`** - Safety and audit features
  - `$tracer_audit` - Audit trail system
  - Backup/restore functionality
  - Safe transformation wrappers
  - Undo capabilities

### Example Files

- **`tracer_example.moo`** - Practical usage examples
  - String utils migration example
  - Database schema evolution
  - Demonstration scripts

- **`tracer_live_demo.moo`** - Live demonstration code
  - Scripts to create legacy objects
  - Step-by-step transformation demos
  - Batch transformation utilities

## Usage

To use the SystemTracer in your Echo environment:

```moo
// Load and initialize the system
$system:setup_system_tracer()

// Do a dry run first
$system_tracer.dry_run = 1
$system_tracer:transform_object(#123)

// Apply transformations
$system_tracer.dry_run = 0
$system_tracer:transform_object(#123)
```

See the documentation in `/docs/ECHO_SYSTEM_TRACER.md` for detailed usage instructions.