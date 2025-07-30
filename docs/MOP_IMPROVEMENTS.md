# Meta-Object Protocol (MOP) Improvements

## Current State

The current MOP implementation stores source code in metadata structures
separate from the actual executable code:

- `VerbMetadata` stores `source_code` as a string
- `PropertyMetadata` stores `source_code` for lambda properties
- The actual verb/property execution uses different data structures

## Issues

1. **Duplication**: Source code is stored separately from the AST/executable
   form, leading to potential inconsistencies
2. **Manual Synchronization**: When code is modified, both the executable form
   and the metadata source need updating
3. **Not Dynamic**: The source code in metadata doesn't automatically reflect
   runtime changes to the AST

## Proper MOP Design

A proper MOP should:

1. **Single Source of Truth**: Generate source code dynamically from the AST
   when requested, not store it separately
2. **Introspection Methods**: Provide methods like:
   - `object.verb_source(verb_name)` - returns source by traversing the stored
     AST
   - `object.property_source(prop_name)` - for properties containing lambdas
   - `object.verb_args(verb_name)` - returns parameter information
   - `object.verb_info(verb_name)` - returns permissions, docs, etc.

3. **Integration with Storage**:
   - Store AST in `VerbDefinition` (not just code string)
   - Use `ToSource` trait to generate source on demand
   - This ensures source always matches what will execute

4. **MOO Compatibility**: Follow LambdaMOO's approach:
   - `verb_code()` - returns source generated from bytecode/AST
   - `verb_args()` - returns argument spec
   - `verb_info()` - returns verb metadata

## Implementation Plan

1. Modify `VerbDefinition` to store AST instead of code string
2. Remove `source_code` fields from metadata structures
3. Add proper MOP methods to objects that generate source dynamically
4. Ensure verb execution uses the same AST that source generation uses

## Example Future API

```rust
// Instead of storing source in metadata:
let source = object.verb_source("greet")?;  // Generates from AST

// Verb introspection:
let info = object.verb_info("greet")?;
// Returns: VerbInfo {
//   name: "greet",
//   args: ["name"],
//   permissions: ...,
//   source: generated_from_ast
// }
```

This would ensure consistency and follow the principle that code has a single
representation (AST) from which both execution and source viewing derive.
