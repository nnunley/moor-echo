

# Documentation for rust-sitter

This document provides an overview of `rust-sitter`, a library for building parsers in Rust using Tree Sitter.

## Introduction

`rust-sitter` is a tool that simplifies the creation of efficient parsers in Rust. It leverages the power of the Tree Sitter parser generator, allowing you to define your grammar directly within your Rust code using annotations. This approach eliminates the need for a separate grammar file and automatically generates type-safe bindings for your language.

The key benefits of using `rust-sitter`:

*   **Idiomatic Rust:** Define your grammar using familiar Rust structs and enums.
*   **Type-Safe:** The generated parser produces a strongly-typed Abstract Syntax Tree (AST).
*   **Efficient:** Leverages the high-performance Tree Sitter parsing library.
*   **Great Error Recovery:** Tree Sitter is known for its ability to produce a useful CST even with syntax errors.

## Core Concepts

`rust-sitter` works by transforming your annotated Rust code into a Tree Sitter grammar and then generating the necessary bindings to parse that grammar into your defined data structures.

### Grammar Definition

You define the grammar of your language using Rust structs and enums, and use the `#[sitter(...)]` attribute to specify the parsing rules.

**Example: A simple JSON-like grammar**

```rust
#[derive(Debug, PartialEq)]
#[sitter(grammar = "json")]
pub enum Json {
    #[sitter(word = "null")]
    Null,
    #[sitter(word = "true")]
    True,
    #[sitter(word = "false")]
    False,
    #[sitter(pattern = r#""[^\"]*""#)]
    String(String),
    #[sitter(pattern = r"\d+")]
    Number(u64),
    #[sitter(prec = 1, left = ",")]
    Array(Vec<Json>),
}
```

In this example:

*   `#[sitter(grammar = "json")]` at the top-level enum declares the name of the grammar.
*   `#[sitter(word = "...")]` matches an exact keyword.
*   `#[sitter(pattern = "...")]` matches a regular expression.
*   The fields of the structs/enums define the structure of your AST.

### Two-Phase Process

`rust-sitter` uses a two-phase process:

1.  **Grammar Generation:** A procedural macro (`#[sitter(grammar = ...)]`) processes your annotated Rust code and generates a Tree Sitter grammar in JSON format.
2.  **Parser Generation:** This grammar is then used by a `build.rs` script to generate the actual Tree Sitter parser. Another macro then generates the Rust bindings that convert the output of the Tree Sitter parser into your custom data types.

## End-to-End Walkthrough: A Simple Expression Evaluator

Let's build a parser for simple arithmetic expressions like `1 + 2 * 3`.

### 1. Project Setup

First, you need to add `rust-sitter` and `tree-sitter-cli` to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.2"

[build-dependencies]
rust-sitter-build = "0.2"
```

You will also need to install the `tree-sitter` CLI.

```bash
cargo install tree-sitter-cli
```

### 2. Defining the Grammar in Rust

Now, let's define the grammar for our expression language in `src/lib.rs`:

```rust
use rust_sitter::sitter;

#[sitter(grammar = "expression")]
#[derive(Debug, PartialEq)]
pub enum Expression {
    #[sitter(pattern = r"\d+")]
    Number(u64),
    #[sitter(prec = 1, left = "+")]
    Add(Box<Expression>, Box<Expression>),
    #[sitter(prec = 2, left = "*")]
    Mul(Box<Expression>, Box<Expression>),
}
```

Here we define our `Expression` enum with three variants:
*   A `Number`, which is a sequence of digits.
*   An `Add` operation, which is left-associative with precedence 1.
*   A `Mul` operation, which is left-associative with precedence 2.

### 3. Build Script

Next, create a `build.rs` file in your project root to generate the parser:

```rust
fn main() {
    rust_sitter_build::build_parsers(&["src/lib.rs"]).unwrap();
}
```

This script tells `rust-sitter` to look for grammar definitions in `src/lib.rs` and generate the necessary parsers.

### 4. Using the Parser

Now you can use the generated parser in your code.

```rust
use rust_sitter::errors::{ParseError, ParseErrorReason};

mod expression {
    use rust_sitter::sitter;

    #[sitter(grammar = "expression")]
    #[derive(Debug, PartialEq)]
    pub enum Expression {
        #[sitter(pattern = r"\d+")]
        Number(#[sitter(leaf)] u64),
        #[sitter(prec_left = 1, non_assoc)]
        Add(Box<Expression>, #[sitter(leaf)] char, Box<Expression>),
        #[sitter(prec_left = 2, non_assoc)]
        Mul(Box<Expression>, #[sitter(leaf)] char, Box<Expression>),
    }
}

fn main() {
    let input = "1 + 2 * 3";
    let result = rust_sitter::parse::<expression::Expression>(input);

    match result {
        Ok(ast) => println!("Parsed AST: {:?}", ast),
        Err(e) => eprintln!("Error parsing: {:?}", e),
    }
}
```

### 5. Running the code

When you run your project, `build.rs` will be executed first, generating the parser. Then, your `main` function will run, parsing the input string and printing the resulting AST.

The output will be:

```
Parsed AST:
    Number(1),
    Mul(
        Number(2),
        Number(3)
    )
)
```

This demonstrates how `rust-sitter` correctly handles operator precedence.

## Advanced Features

`rust-sitter` also supports more advanced features:

*   **Repetition:** Use `Vec<T>` to parse repeated elements.
*   **Optional Values:** Use `Option<T>` for optional parts of your grammar.
*   **Tokenization:** Use `#[sitter(leaf)]` to capture the text of a token.
*   **Error Recovery:** Tree Sitter automatically handles errors, and `rust-sitter` exposes them through its `ParseError` type.

For more detailed information and examples, please refer to the official `rust-sitter` repository and the introductory blog post.

