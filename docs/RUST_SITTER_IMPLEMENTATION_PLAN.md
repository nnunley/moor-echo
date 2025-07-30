# Rust-Sitter Implementation Plan

## Overview

This document outlines the complete plan for migrating from hand-rolled parser
to rust-sitter grammar for the Echo language REPL, with eventual JIT compilation
using Cranelift.

## Current Status ✅

- [x] Created working rust-sitter grammar with basic AST nodes
- [x] Grammar compiles successfully with rust-sitter-tool
- [x] Implemented: Number, Identifier, Add, PropertyAccess
- [x] Proper build configuration and whitespace handling

## Implementation Strategy

### Phase 1: AST Migration (Current Phase)

**Goal**: Migrate existing codebase to use rust-sitter generated AST types

**Tasks**:

1. **Create AST Compatibility Layer**
   - Create conversion functions between old `ast::EchoAst` and new
     `echo::EchoAst`
   - Implement `From` traits for seamless conversion
   - Maintain backward compatibility during transition

2. **Update Evaluator**
   - Modify `evaluator/mod.rs` to accept rust-sitter AST types
   - Update pattern matching for new AST structure
   - Handle new field names (e.g., `_op: ()` fields)

3. **Update Parser Integration**
   - Modify `EchoParser` to use rust-sitter parse function
   - Update error handling for rust-sitter errors
   - Ensure REPL integration works with new parser

### Phase 2: Grammar Expansion

**Goal**: Add missing language features to rust-sitter grammar

**Priority Order**:

1. **Let Statements** - `let x = value;`
2. **Method Calls** - `obj:verb(args)`
3. **Object Definitions** - `object name ... endobject`
4. **String Literals** - `"quoted strings"`
5. **Binary Operations** - `-, *, /, ==, !=, <, >`
6. **Verb Definitions** - `verb "name" (...) ... endverb`

### Phase 3: Advanced Features

**Goal**: Implement complex language constructs

**Features**:

1. **Control Flow** - `if/else`, `while`, `for`
2. **Function Definitions** - `function name(...) ... endfunction`
3. **List Literals** - `{1, 2, 3}`
4. **Property Assignment** - `obj.prop = value`
5. **Error Handling** - `try/catch` blocks

### Phase 4: JIT Compilation with Cranelift

**Goal**: Add Just-In-Time compilation for performance optimization

**Features**:

1. **Cranelift Integration** - Generate machine code from rust-sitter AST
2. **Compilation Strategy** - Hybrid interpretation/compilation approach
3. **Hot Path Detection** - Identify frequently executed code for JIT
4. **Performance Benchmarking** - Compare interpreted vs JIT performance
5. **Memory Management** - Efficient code generation and caching

## Technical Architecture

### AST Conversion Strategy

```rust
// Conversion trait for AST migration
trait IntoRustSitter {
    type Target;
    fn into_rust_sitter(self) -> Self::Target;
}

// Conversion from old AST to new AST
impl IntoRustSitter for ast::EchoAst {
    type Target = echo::EchoAst;
    fn into_rust_sitter(self) -> Self::Target {
        match self {
            ast::EchoAst::Number(n) => echo::EchoAst::Number(n),
            ast::EchoAst::Identifier(s) => echo::EchoAst::Identifier(s),
            // ... other conversions
        }
    }
}
```

### Evaluator Updates

```rust
// Updated evaluator signature
impl Evaluator {
    pub fn eval(&mut self, ast: &echo::EchoAst) -> Result<Value> {
        match ast {
            echo::EchoAst::Number(n) => Ok(Value::Integer(*n)),
            echo::EchoAst::Add { left, right, .. } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;
                // Addition logic
            }
            // ... other patterns
        }
    }
}
```

### Grammar Expansion Template

```rust
// Template for adding new AST nodes
#[derive(Debug, PartialEq)]
#[rust_sitter::language]
pub enum EchoAst {
    // Existing nodes...

    // New node template
    NewNode {
        #[rust_sitter::leaf(text = "keyword")]
        _keyword: (),
        field: Box<EchoAst>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    },
}
```

## Testing Strategy

### Test Categories

1. **Parser Tests** - Verify rust-sitter grammar parsing
2. **Evaluator Tests** - Ensure evaluation works with new AST
3. **Integration Tests** - Full REPL functionality
4. **Compatibility Tests** - Old vs new behavior comparison

### Test Coverage Requirements

- [ ] All existing tests pass with rust-sitter
- [ ] New rust-sitter specific tests added
- [ ] Property access tests work with new AST
- [ ] Verb execution tests work with new AST
- [ ] Error handling tests updated

## Quality Gates

### Phase 1 Completion Criteria

- [ ] All existing tests pass
- [ ] No functionality regression
- [ ] Performance maintained or improved
- [ ] Code compiles without warnings

### Phase 2 Completion Criteria

- [ ] All Echo language features implemented
- [ ] Grammar handles complex expressions
- [ ] Error recovery works properly
- [ ] Documentation updated

### Phase 3 Completion Criteria

- [ ] Complete Echo language support
- [ ] Performance optimizations applied
- [ ] Full test coverage achieved
- [ ] Production ready

## Risk Mitigation

### Potential Issues

1. **AST Structure Changes** - Fields renamed/restructured
   - _Mitigation_: Comprehensive conversion layer
2. **Performance Impact** - Rust-sitter overhead
   - _Mitigation_: Benchmark and optimize
3. **Error Handling** - Different error types
   - _Mitigation_: Unified error handling layer
4. **Breaking Changes** - API incompatibilities
   - _Mitigation_: Gradual migration with compatibility layer

### Rollback Strategy

- Maintain old parser as fallback
- Feature flags for rust-sitter vs old parser
- Comprehensive test suite for regression detection

## Implementation Timeline

### Week 1: AST Migration

- Days 1-2: Create conversion layer
- Days 3-4: Update evaluator
- Days 5-7: Fix integration and tests

### Week 2: Grammar Expansion

- Days 1-3: Add missing basic features
- Days 4-5: Implement complex constructs
- Days 6-7: Testing and debugging

### Week 3: Polish and Optimization

- Days 1-3: Performance optimization
- Days 4-5: Documentation updates
- Days 6-7: Final testing and validation

## Success Metrics

- Zero functionality regression
- All tests passing
- Performance within 10% of original
- Code coverage maintained
- Documentation complete

## Next Immediate Actions

1. Create AST conversion functions
2. Update evaluator for basic rust-sitter AST
3. Fix compilation errors
4. Run existing tests with new implementation
5. Add missing grammar features incrementally
