# rust-sitter Documentation

## Overview

rust-sitter is a pure Rust implementation of tree-sitter, providing incremental parsing capabilities without C dependencies. Developed by the Hydro project team at UC Berkeley, it offers a memory-safe alternative to the original tree-sitter while maintaining compatibility with existing grammars.

As Shadaj Laddad explains in his introduction, rust-sitter emerged from the need for a WebAssembly-compatible parser that could run in browsers without complex build processes. The project demonstrates how Rust's safety guarantees can be applied to traditionally C-dominated parsing infrastructure.

## Key Features

### 1. Pure Rust Implementation
- **No C Dependencies**: Eliminates the need for C compiler and complex build processes
- **Memory Safety**: Leverages Rust's ownership system for guaranteed memory safety
- **Cross-Platform**: Easier deployment across different platforms
- **Simplified Build**: No need for Node.js or external build tools

### 2. Tree-sitter Compatibility
- **Grammar Format**: Uses the same grammar.js format as tree-sitter
- **API Compatibility**: Similar API design for easy migration
- **Incremental Parsing**: Maintains tree-sitter's efficient incremental parsing
- **Error Recovery**: Robust error recovery for incomplete/invalid syntax

### 3. Performance Characteristics
- **Comparable Speed**: Performance on par with C implementation
- **Memory Efficiency**: Rust's zero-cost abstractions ensure minimal overhead
- **Incremental Updates**: Efficient re-parsing of changed sections
- **Parallel Safety**: Thread-safe parsing operations

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
rust-sitter = "0.5"
rust-sitter-tool = "0.5"
```

## Basic Usage

### 1. Creating a Parser

```rust
use rust_sitter::{Parser, Language};

// Load your compiled language
let language = tree_sitter_echo(); // Your language function
let mut parser = Parser::new();
parser.set_language(language)?;
```

### 2. Parsing Source Code

```rust
let source_code = "let x = 42;";
let tree = parser.parse(source_code, None)?;
let root_node = tree.root_node();
```

### 3. Traversing the AST

```rust
// Walk the syntax tree
let mut cursor = root_node.walk();
for child in root_node.children(&mut cursor) {
    println!("Node kind: {}", child.kind());
    println!("Text: {}", child.utf8_text(source_code.as_bytes())?);
}
```

### 4. Incremental Parsing

```rust
// Initial parse
let old_tree = parser.parse(source_code, None)?;

// Edit the source
let new_source = "let x = 43;";
// Parse with old tree for efficiency
let new_tree = parser.parse(new_source, Some(&old_tree))?;
```

## Grammar Development

### 1. Grammar Definition

Create a `grammar.js` file:
```javascript
module.exports = grammar({
  name: 'echo',
  
  rules: {
    source_file: $ => repeat($._statement),
    
    _statement: $ => choice(
      $.let_statement,
      $.expression_statement
    ),
    
    let_statement: $ => seq(
      'let',
      $.identifier,
      '=',
      $._expression,
      ';'
    ),
    
    _expression: $ => choice(
      $.number,
      $.identifier,
      $.binary_expression
    ),
    
    binary_expression: $ => prec.left(seq(
      $._expression,
      choice('+', '-', '*', '/'),
      $._expression
    )),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    number: $ => /\d+/
  }
});
```

### 2. Compiling the Grammar

rust-sitter uses a build script approach:

```rust
// build.rs
use rust_sitter_tool::compile_grammar;

fn main() {
    compile_grammar("src/grammar.js", "src/parser.rs")
        .expect("Failed to compile grammar");
}
```

## Advanced Features

### 1. Query System

```rust
use rust_sitter::{Query, QueryCursor};

let query = Query::new(
    language,
    "(let_statement name: (identifier) @var)"
)?;

let mut cursor = QueryCursor::new();
let matches = cursor.matches(&query, root_node, source_code.as_bytes());

for match_ in matches {
    for capture in match_.captures {
        let node = capture.node;
        let text = node.utf8_text(source_code.as_bytes())?;
        println!("Variable: {}", text);
    }
}
```

### 2. Syntax Highlighting

```rust
let highlight_query = r#"
(identifier) @variable
(number) @number
"let" @keyword
"=" @operator
"#;

let query = Query::new(language, highlight_query)?;
// Use query to highlight syntax
```

### 3. Error Handling

```rust
if root_node.has_error() {
    let mut cursor = root_node.walk();
    visit_errors(&mut cursor, |node| {
        if node.is_error() {
            eprintln!("Syntax error at {}: expected {}",
                node.start_position(),
                node.kind()
            );
        }
    });
}
```

## Insights from Shadaj Laddad's Introduction

In his blog post "Introducing rust-sitter," Shadaj Laddad highlights the motivation and implementation details of rust-sitter:

### 1. Motivation: WebAssembly and Beyond

Shadaj explains the genesis of rust-sitter:
> "I wanted to use tree-sitter in the browser for a project, but the C implementation required complex build tooling and didn't play well with WebAssembly."

This led to key design decisions:
- **Pure Rust**: Eliminates C FFI complexity
- **Wasm-First**: Designed with browser deployment in mind
- **Sandboxed Execution**: Safe parsing in restricted environments

### 2. Implementation Insights

From Shadaj's deep dive into the codebase:

**Parser Architecture**:
```rust
// rust-sitter uses a table-driven approach
pub struct Parser {
    stack: Vec<StackEntry>,
    lexer: Lexer,
    trees: TreePool,
}
```

**Key Differences from C tree-sitter**:
- **Memory Management**: Rust's ownership eliminates manual memory management
- **Error Handling**: Result types instead of error codes
- **Concurrency**: Safe parallelism with Send/Sync traits

### 3. Performance Characteristics

Shadaj's benchmarks reveal:
- **Parse Time**: Within 10-20% of C implementation
- **Memory Usage**: Comparable, sometimes better due to Rust optimizations
- **Wasm Performance**: 2-3x faster than C version compiled to Wasm

### 4. Real-World Applications

The blog post highlights several use cases:

**1. Browser-Based IDEs**:
```rust
// Compile to Wasm
wasm-pack build --target web

// Use in JavaScript
import init, { Parser } from './rust_sitter_wasm.js';
await init();
const parser = new Parser();
```

**2. Language Servers**:
- No need for Node.js runtime
- Better integration with Rust-based LSPs
- Improved memory safety for long-running processes

**3. Educational Tools**:
- Easier to understand parsing algorithms
- Safe experimentation environment
- Better debugging capabilities

## Performance Optimization

### 1. Reusing Parsers
```rust
// Create parser once, reuse for multiple parses
let mut parser = Parser::new();
parser.set_language(language)?;

// Reuse for multiple files
for file in files {
    let tree = parser.parse(&file.content, None)?;
    // Process tree
}
```

### 2. Concurrent Parsing
```rust
use rayon::prelude::*;

files.par_iter().map(|file| {
    let mut parser = Parser::new();
    parser.set_language(language)?;
    parser.parse(&file.content, None)
}).collect::<Result<Vec<_>, _>>()?;
```

### 3. Memory Management
```rust
// Trees are automatically dropped when out of scope
{
    let tree = parser.parse(source, None)?;
    // Use tree
} // Tree memory freed here
```

## Common Patterns

### 1. REPL Integration
```rust
pub struct ReplParser {
    parser: Parser,
    previous_tree: Option<Tree>,
}

impl ReplParser {
    pub fn parse_line(&mut self, line: &str) -> Result<Tree> {
        let tree = self.parser.parse(line, self.previous_tree.as_ref())?;
        self.previous_tree = Some(tree.clone());
        Ok(tree)
    }
}
```

### 2. Language Server Protocol
```rust
// Incremental parsing for LSP
pub fn handle_document_change(
    parser: &mut Parser,
    old_tree: &Tree,
    changes: Vec<TextChange>
) -> Result<Tree> {
    // Apply changes to source
    let new_source = apply_changes(&old_source, changes);
    
    // Parse incrementally
    parser.parse(&new_source, Some(old_tree))
}
```

### 3. Custom Node Types
```rust
pub enum EchoNode<'a> {
    LetStatement { name: &'a str, value: Node<'a> },
    Function { name: &'a str, params: Vec<&'a str> },
    // ... other variants
}

impl<'a> TryFrom<Node<'a>> for EchoNode<'a> {
    type Error = anyhow::Error;
    
    fn try_from(node: Node<'a>) -> Result<Self> {
        match node.kind() {
            "let_statement" => {
                // Extract fields
                Ok(EchoNode::LetStatement { /* ... */ })
            }
            _ => Err(anyhow!("Unknown node kind"))
        }
    }
}
```

## Debugging Tips

### 1. Print Parse Trees
```rust
fn print_tree(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{} [{}:{}]", 
        indent, 
        node.kind(),
        node.start_position().row,
        node.start_position().column
    );
    
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, depth + 1);
    }
}
```

### 2. Grammar Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_let_statement() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_echo()).unwrap();
        
        let tree = parser.parse("let x = 42;", None).unwrap();
        assert!(!tree.root_node().has_error());
        
        let root = tree.root_node();
        assert_eq!(root.child_count(), 1);
        assert_eq!(root.child(0).unwrap().kind(), "let_statement");
    }
}
```

## Migration from tree-sitter

### Key Differences
1. **Build Process**: No Node.js required
2. **API Names**: Some minor API differences
3. **Performance**: Comparable but may vary by use case
4. **Features**: Most tree-sitter features supported

### Migration Checklist
- [ ] Replace tree-sitter dependency with rust-sitter
- [ ] Update build scripts to use rust-sitter-tool
- [ ] Adjust API calls for minor differences
- [ ] Test incremental parsing behavior
- [ ] Verify query compatibility

## Technical Deep Dive from Shadaj's Post

### Grammar Compilation Process

Shadaj details how rust-sitter compiles grammars:

**1. Grammar to Rust Transformation**:
```javascript
// grammar.js
module.exports = grammar({
  name: 'example',
  rules: {
    source_file: $ => repeat($.statement),
    statement: $ => choice($.expression, $.declaration),
    // ...
  }
});
```

Becomes:
```rust
// Generated Rust code
pub fn tree_sitter_example() -> Language {
    let mut builder = LanguageBuilder::new();
    builder.add_rule(/* compiled rule data */);
    builder.build()
}
```

**2. State Machine Generation**:
- Converts grammar rules to finite automata
- Optimizes for common patterns
- Generates compact lookup tables

### Error Recovery Mechanisms

Shadaj emphasizes rust-sitter's robust error recovery:

**1. Missing Token Recovery**:
```rust
// rust-sitter can infer missing tokens
let input = "if (condition) { body"; // Missing '}'
let tree = parser.parse(input, None).unwrap();
// Tree still contains valid structure with ERROR node
```

**2. Skip and Resume**:
- Skips unparseable sections
- Resumes parsing at next valid token
- Maintains partial tree structure

### Integration with Hydro Project

As part of the Hydro ecosystem, rust-sitter benefits from:

**1. Dataflow Integration**:
```rust
// Use with Timely Dataflow
use timely::dataflow::*;
use rust_sitter::Parser;

worker.dataflow::<(), _, _>(|scope| {
    let parser = Parser::new();
    source.flat_map(move |code| {
        parser.parse(&code, None)
            .map(|tree| extract_symbols(tree))
    });
});
```

**2. Distributed Parsing**:
- Parse large codebases in parallel
- Aggregate results across nodes
- Maintain consistency with Hydro's protocols

## Future Directions

Based on Shadaj's vision and Hydro project goals:

### 1. Next-Generation Features
- **Incremental Compilation**: Faster grammar updates
- **Query Optimization**: Better performance for complex queries
- **Streaming Grammars**: Support for infinite input streams

### 2. Ecosystem Growth
- **Grammar Repository**: Centralized grammar sharing
- **Tool Integration**: Better IDE and build tool support
- **Performance Profiling**: Built-in optimization tools

### 3. Research Applications
- **Program Synthesis**: Use parsing for code generation
- **Static Analysis**: Deeper integration with analysis tools
- **Language Design**: Rapid prototyping of new languages

## Resources

- [rust-sitter GitHub](https://github.com/hydro-project/rust-sitter)
- [Original tree-sitter docs](https://tree-sitter.github.io/tree-sitter/)
- [Hydro Project](https://hydro.run/)
- [Shadaj's Introduction](https://www.shadaj.me/writing/introducing-rust-sitter/)