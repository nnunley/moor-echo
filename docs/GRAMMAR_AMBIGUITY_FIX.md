# Fixing Grammar Ambiguity: Let/Const vs. Assignment

This document outlines the approach to resolve the grammar ambiguity between
`let`/`const` declarations and general assignment expressions within the
`echo-repl` project. The core issue arises from the manual pre-parsing of
`let`/`const` in `EchoParser::parse` conflicting with the `rust-sitter`
grammar's generic `Assignment` rule.

The solution is to move the `let` and `const` parsing entirely into the
`rust-sitter` grammar and define a more general structure for assignment
targets.

## 1. Remove Manual Assignment Parsing from `EchoParser::parse`

The current `EchoParser::parse` method in `echo-repl/src/parser/echo/mod.rs`
manually detects and parses `let`/`const` and property assignments. This manual
intervention is the root cause of the ambiguity and prevents `rust-sitter` from
handling these constructs uniformly.

**Action:**

- Open `echo-repl/src/parser/echo/mod.rs`.
- Locate the `parse` method within the `impl EchoParser` block.
- **Delete the entire
  `if let Some(equals_pos) = input.find('=') { ... } else { ... }` block.**
- The `parse` method should be simplified to directly call the `rust-sitter`
  generated parser and then convert its AST:

  ```rust
  impl EchoParser {
      // ... other methods ...

      pub fn parse(&mut self, input: &str) -> Result<ast::EchoAst> {
          // Parse normally using the rust-sitter generated parser
          let grammar_ast = self.inner.parse(input)?;

          // Convert grammar AST to unified AST
          convert_grammar_to_ast(grammar_ast)
      }
  }
  ```

## 2. Restructure Grammar in `echo-repl/src/parser/echo/grammar.rs`

We need to define distinct grammar rules for `let`/`const` declarations and for
general assignment expressions. This involves introducing a new `LValue`
(Left-Hand Side Value) rule for generic assignments and explicit statement types
for `let`/`const`.

**Action:**

- Open `echo-repl/src/parser/echo/grammar.rs`.

- **Define `LValue` (Left-Hand Side Value):** This new enum will represent valid
  targets for a generic assignment expression (e.g., `x = 5` or `obj.prop = 5`).
  It should _not_ include `let` or `const` keywords, as those are part of the
  declaration syntax.

  ```rust
  // Add this new enum
  #[derive(Debug, PartialEq)]
  pub enum LValue {
      Identifier(Identifier),
      PropertyAccess {
          object: Box<EchoAst>, // Can be any expression that evaluates to an object
          #[rust_sitter::leaf(text = ".")]
          _dot: (),
          property: Box<EchoAst>, // Must resolve to an identifier or string
      },
      // If your language supports `arr[index] = value` as an LValue, add it here:
      // IndexAccess {
      //     object: Box<EchoAst>,
      //     #[rust_sitter::leaf(text = "[")]
      //     _lbracket: (),
      //     index: Box<EchoAst>,
      //     #[rust_sitter::leaf(text = "]")]
      //     _rbracket: (),
      // },
  }
  ```

- **Define `BindingPattern` and `BindingPatternElement`:** These are used
  specifically for `let`/`const` declarations, allowing for destructuring
  assignments. (Ensure these are correctly defined as per your
  `CST_REFERENCE.md` if they don't already exist in this exact form).

  ```rust
  // Ensure these are defined as per CST_REFERENCE.md
  #[derive(Debug, PartialEq)]
  pub enum BindingPattern {
      Identifier(Identifier),
      List {
          #[rust_sitter::leaf(text = "{")]
          _lbrace: (),
          #[rust_sitter::repeat(non_empty = false)]
          #[rust_sitter::delimited(
              #[rust_sitter::leaf(text = ",")]
              ()
          )]
          elements: Vec<BindingPatternElement>,
          #[rust_sitter::leaf(text = "}")]
          _rbrace: (),
      },
      Rest {
          #[rust_sitter::leaf(pattern = r"\.\.\.")]
          _dots: (),
          name: Identifier,
      },
      #[rust_sitter::leaf(text = "_")]
      Ignore,
  }

  #[derive(Debug, PartialEq)]
  pub enum BindingPatternElement {
      Simple(Identifier),
      Optional {
          #[rust_sitter::leaf(text = "?")]
          _question: (),
          name: Identifier,
          #[rust_sitter::leaf(text = "=")]
          _equals: (),
          default: Box<EchoAst>,
      },
      Rest {
          #[rust_sitter::leaf(text = "@")]
          _at: (),
          name: Identifier,
      },
  }
  ```

- **Modify `EchoAst` Enum:**
  - **Rename `Assignment` to `AssignmentExpr`:** This clarifies that it's an
    expression (something that produces a value), not a standalone statement.
  - **Update `AssignmentExpr` to use `LValue`:** Its `target` field should now
    be of the new `LValue` type.
  - **Add `LocalAssignment` and `ConstAssignment` as new top-level statements:**
    These will explicitly start with `let` or `const` keywords and use
    `BindingPattern` for their target.

  ```rust
  #[rust_sitter::language]
  pub enum EchoAst {
      // ... existing literals, binary ops, etc. ...

      // Generic Assignment Expression (e.g., `x = 5`, `obj.prop = 5`)
      // This is an expression, not a statement.
      #[rust_sitter::prec_right(2)] // Keep its precedence relative to other expressions
      AssignmentExpr { // Renamed from original `Assignment`
          left: Box<LValue>, // Now uses the new LValue enum
          #[rust_sitter::leaf(text = "=")]
          _op: (),
          right: Box<EchoAst>,
      },

      // New Top-Level Statements for Declarations
      // These are distinct from AssignmentExpr because they introduce new bindings.
      LocalAssignment {
          #[rust_sitter::leaf(text = "let")]
          _let_keyword: (),
          target: Box<BindingPattern>, // Can be an identifier or destructuring pattern
          #[rust_sitter::leaf(text = "=")]
          _op: (),
          value: Box<EchoAst>,
          // If your language requires statements to end with a semicolon, add it here:
          // #[rust_sitter::leaf(text = ";")]
          // _semicolon: (),
      },
      ConstAssignment {
          #[rust_sitter::leaf(text = "const")]
          _const_keyword: (),
          target: Box<BindingPattern>, // Can be an identifier or destructuring pattern
          #[rust_sitter::leaf(text = "=")]
          _op: (),
          value: Box<EchoAst>,
          // #[rust_sitter::leaf(text = ";")]
          // _semicolon: (),
      },

      // ... other statements and expressions ...
  }
  ```

- **Update Top-Level Statement Rule:** Ensure that the rule defining what
  constitutes a valid top-level statement (e.g., `_statement` or `program_body`
  in your grammar) explicitly includes these new `LocalAssignment` and
  `ConstAssignment` variants. You might also need an `ExpressionStatement` to
  wrap `AssignmentExpr` if `x = 5;` is a valid standalone statement.

  ```rust
  // Example: If you have a rule like this in your grammar.rs:
  // #[derive(Debug, PartialEq)]
  // pub enum EchoAst {
  //     // ...
  //     #[rust_sitter::prec_left(0)] // Example precedence for statements
  //     Statement(Box<Statement>),
  //     // ...
  // }
  //
  // #[derive(Debug, PartialEq)]
  // pub enum Statement {
  //     // ... existing statements like Return, If, While, For, etc. ...
  //     LocalAssignment(LocalAssignment), // Add this
  //     ConstAssignment(ConstAssignment), // Add this
  //     // If `x = 5;` is a valid statement, you'll need to wrap AssignmentExpr:
  //     ExpressionStatement {
  //         expr: Box<EchoAst>,
  //         #[rust_sitter::leaf(text = ";")]
  //         _semicolon: (),
  //     },
  // }
  ```

## 3. Update `convert_grammar_to_ast` Function

The `convert_grammar_to_ast` function in `echo-repl/src/parser/echo/mod.rs` will
need to be updated to correctly handle the new AST nodes generated by
`rust-sitter` and map them to your unified `ast::EchoAst` types.

**Action:**

- Open `echo-repl/src/parser/echo/mod.rs`.
- Locate the `convert_grammar_to_ast` function.
- Add new `match` arms for `G::LocalAssignment` and `G::ConstAssignment`,
  converting them to their corresponding `ast::EchoAst` variants.
- Update the `G::Assignment` (now `G::AssignmentExpr`) conversion to use the new
  `LValue` structure.
- You'll also need helper functions to convert `grammar::LValue` and
  `grammar::BindingPattern` (and its elements) into their `ast` equivalents.

  ```rust
  // In echo-repl/src/parser/echo/mod.rs

  // ... (existing code) ...

  fn convert_grammar_to_ast(node: grammar::EchoAst) -> Result<ast::EchoAst> {
      use grammar::EchoAst as G;
      use ast::EchoAst as A;

      Ok(match node {
          // ... existing conversions for literals, binary ops, etc. ...

          // Handle the new LocalAssignment statement
          G::LocalAssignment { _let_keyword, target, value } => {
              A::LocalAssignment {
                  target: convert_grammar_binding_pattern_to_ast(*target)?,
                  value: Box::new(convert_grammar_to_ast(*value)?),
              }
          },
          // Handle the new ConstAssignment statement
          G::ConstAssignment { _const_keyword, target, value } => {
              A::ConstAssignment {
                  target: convert_grammar_binding_pattern_to_ast(*target)?,
                  value: Box::new(convert_grammar_to_ast(*value)?),
              }
          },
          // Handle the generic Assignment Expression (renamed from G::Assignment)
          G::AssignmentExpr { left, right, .. } => {
              A::Assignment { // Assuming ast::EchoAst::Assignment is your target
                  target: convert_grammar_lvalue_to_ast(*left)?, // New conversion
                  value: Box::new(convert_grammar_to_ast(*right)?),
              }
          },
          // If you introduced ExpressionStatement:
          // G::ExpressionStatement { expr, .. } => {
          //     A::ExpressionStatement {
          //         expr: Box::new(convert_grammar_to_ast(*expr)?),
          //     }
          // },

          // ... other conversions ...
      })
  }

  // New helper function for LValue conversion
  fn convert_grammar_lvalue_to_ast(node: grammar::LValue) -> Result<ast::LValue> {
      use grammar::LValue as G_LValue;
      use ast::LValue as A_LValue;

      Ok(match node {
          G_LValue::Identifier(ident) => A_LValue::Binding {
              binding_type: ast::BindingType::None, // No let/const keyword here
              pattern: ast::BindingPattern::Identifier(ident.name),
          },
          G_LValue::PropertyAccess { object, property, .. } => A_LValue::PropertyAccess {
              object: Box::new(convert_grammar_to_ast(*object)?),
              property: match *property {
                  grammar::EchoAst::Identifier(s) => s,
                  _ => anyhow::bail!("Property name must be an identifier"),
              },
          },
          // Add IndexAccess conversion if you added it to LValue
          // G_LValue::IndexAccess { object, index, .. } => A_LValue::IndexAccess {
          //     object: Box::new(convert_grammar_to_ast(*object)?),
          //     index: Box::new(convert_grammar_to_ast(*index)?),
          // },
      })
  }

  // New helper function for BindingPattern conversion
  fn convert_grammar_binding_pattern_to_ast(node: grammar::BindingPattern) -> Result<ast::BindingPattern> {
      use grammar::BindingPattern as G_BP;
      use ast::BindingPattern as A_BP;

      Ok(match node {
          G_BP::Identifier(ident) => A_BP::Identifier(ident.name),
          G_BP::List { elements, .. } => {
              let converted_elements: Result<Vec<_>> = elements.into_iter()
                  .map(|elem| convert_grammar_binding_pattern_element_to_ast(elem))
                  .collect();
              A_BP::List(converted_elements?)
          },
          G_BP::Rest { name, .. } => A_BP::Rest(Box::new(A_BP::Identifier(name.name))),
          G_BP::Ignore => A_BP::Ignore,
      })
  }

  // New helper function for BindingPatternElement conversion
  fn convert_grammar_binding_pattern_element_to_ast(node: grammar::BindingPatternElement) -> Result<ast::LambdaParam> {
      use grammar::BindingPatternElement as G_BPE;
      use ast::LambdaParam as A_LP;

      Ok(match node {
          G_BPE::Simple(ident) => A_LP::Simple(ident.name),
          G_BPE::Optional { name, default, .. } => A_LP::Optional {
              name: name.name,
              default: Box::new(convert_grammar_to_ast(*default)?),
          },
          G_BPE::Rest { name, .. } => A_LP::Rest(name.name),
      })
  }
  ```

## 4. Rebuild and Test

After making these changes:

- Run `cargo build` in your `echo-repl` directory. This will trigger
  `rust-sitter` to regenerate the parser with the new grammar rules.
- Run your tests. The ambiguity should be resolved, and the parser should now
  correctly distinguish between `let`/`const` declarations and generic
  assignments.

This restructuring provides a clear and unambiguous grammar, moving all parsing
logic into `rust-sitter` and aligning with the desired AST structure from your
documentation.
