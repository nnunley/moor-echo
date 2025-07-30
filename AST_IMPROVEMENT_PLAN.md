# AST Improvement Plan

## Status: COMPLETED

### Completed Improvements ✓
1. **Removed duplicate AST implementation** in `parser/ast.rs`
2. **Extracted parsing logic** to separate `simple_parser.rs` module
3. **Added comprehensive TDD tests** for simple parser functionality
4. **Updated all references** to use unified AST from `ast/mod.rs`
5. **Applied SOLID principles** - separated parsing from AST definition
6. **Cleaned up experimental code** - removed grammar_development_backup folder
7. **Removed unused grammar files** - grammar_simple.rs and grammar_full.rs
8. **Fixed all warnings** - removed unused imports and variables
9. **Documented language distinction** - created LANGUAGE_GRAMMARS.md
10. **Designed REPL parser** - created REPL_PARSER_DESIGN.md

## Current Issues

### 1. Multiple AST Implementations
- **Problem**: 3 different AST implementations violate DRY and KISS principles
- **Files**:
  - `crates/echo-core/src/ast/mod.rs` - Main unified AST
  - `crates/echo-core/src/parser/ast.rs` - Simple AST with embedded parser
  - `crates/echo-core/src/parser/echo/grammar.rs` - rust-sitter AST

### 2. SOLID Violations
- **SRP**: AST mixed with parsing logic in `parser/ast.rs`
- **OCP**: Hard to extend without modifying multiple files
- **DIP**: Direct dependencies between modules

### 3. Test Coverage Gaps
- Limited tests for AST operations
- No tests for simple parser
- Missing edge case coverage

### 4. Complexity Issues
- Large enum with 50+ variants
- Nested structures that could be simplified
- Duplicate code between implementations

## Improvement Actions

### Phase 1: Consolidation (High Priority)
1. **Remove duplicate AST in `parser/ast.rs`**
   - Move `parse_simple` logic to a separate parser module
   - Use the unified AST from `ast/mod.rs`
   - Update all references

2. **Clean up grammar development backups**
   - Archive or remove experimental versions
   - Keep only the working implementation

### Phase 2: Refactoring (Medium Priority)
1. **Apply SOLID principles**:
   - Extract parsing logic to dedicated parser modules
   - Create trait-based abstractions for AST operations
   - Implement visitor pattern for AST traversal

2. **Simplify AST structure**:
   - Group related variants into sub-enums
   - Extract common patterns into shared types
   - Reduce nesting depth

### Phase 3: Testing (High Priority)
1. **Comprehensive test suite**:
   - Unit tests for each AST node type
   - Parser tests with edge cases
   - Round-trip tests (parse -> AST -> source -> parse)

2. **Property-based testing**:
   - Use proptest for AST generation
   - Verify invariants hold

### Phase 4: Documentation
1. **AST documentation**:
   - Document each node type's purpose
   - Add examples for complex structures
   - Create visual AST diagrams

## Implementation Order

1. **Immediate**: Remove `parser/ast.rs` duplicate implementation
2. **Week 1**: Consolidate to single AST, add tests
3. **Week 2**: Refactor for SOLID compliance
4. **Week 3**: Complete test coverage and documentation

## Success Metrics

- Single AST implementation used throughout
- 90%+ test coverage for AST operations
- Clear separation of concerns (AST, Parser, Evaluator)
- No duplicate code between modules