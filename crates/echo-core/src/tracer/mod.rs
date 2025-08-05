/*!
# SystemTracer - Codebase Transformation System

A SystemTracer-like system for systematic codebase transformation, inspired by Squeak Smalltalk's SystemTracer.
Supports both in-memory transformations and file-based bootstrapping for incompatible changes.

## Overview

The SystemTracer provides two main modes of operation:

1. **InMemoryTracer**: Transforms objects in the running Echo system
2. **FileTracer**: Reads source files, applies transformations, writes transformed code

## Architecture

- `TransformationRule`: Trait for defining transformation rules
- `SystemTracer`: Core transformation engine with rule management
- `FileTracer`: File-based transformation system
- Pattern matching utilities for AST transformations
- MOO-specific transformation rules

## Example Usage

```rust
use echo_core::tracer::{SystemTracer, FileTracer};
use echo_core::tracer::moo_rules::PropertySyntaxFixer;

// In-memory transformation
let mut tracer = SystemTracer::new();
tracer.add_rule(PropertySyntaxFixer::new());
tracer.transform_system(&mut evaluator)?;

// File-based transformation
let mut file_tracer = FileTracer::new();
file_tracer.add_rule(PropertySyntaxFixer::new());
file_tracer.transform_directory("src/moo_files", "output/echo_files")?;
```
*/

pub mod rules;
pub mod system_tracer;
pub mod file_tracer;
pub mod patterns;
pub mod moo_rules;

// Re-export main types
pub use rules::TransformationRule;
pub use system_tracer::SystemTracer;
pub use file_tracer::FileTracer;
pub use patterns::{AstPattern, PatternMatcher};

// Common result type for transformations
pub type TransformResult<T> = anyhow::Result<T>;

#[derive(Debug, Clone)]
pub struct TransformationContext {
    pub source_file: Option<String>,
    pub object_name: Option<String>,
    pub current_depth: usize,
    pub max_depth: usize,
}

impl Default for TransformationContext {
    fn default() -> Self {
        Self {
            source_file: None,
            object_name: None,
            current_depth: 0,
            max_depth: 100,
        }
    }
}

impl TransformationContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_source_file(mut self, file: String) -> Self {
        self.source_file = Some(file);
        self
    }
    
    pub fn with_object_name(mut self, name: String) -> Self {
        self.object_name = Some(name);
        self
    }
    
    pub fn descend(&self) -> Self {
        Self {
            source_file: self.source_file.clone(),
            object_name: self.object_name.clone(),
            current_depth: self.current_depth + 1,
            max_depth: self.max_depth,
        }
    }
    
    pub fn at_max_depth(&self) -> bool {
        self.current_depth >= self.max_depth
    }
}