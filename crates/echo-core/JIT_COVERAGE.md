# JIT Compilation Coverage Analysis

## Current JIT Support Status

As of the current implementation, the JIT compiler has **very minimal coverage** of the Echo AST. Here's the breakdown:

### ✅ Supported AST Nodes (9 out of ~50+)

1. **Number** - Integer literals
   ```rust
   EchoAst::Number(i64)
   ```

2. **Add** - Addition operation
   ```rust
   EchoAst::Add { left, right }
   ```

3. **Subtract** - Subtraction operation
   ```rust
   EchoAst::Subtract { left, right }
   ```

4. **Multiply** - Multiplication operation
   ```rust
   EchoAst::Multiply { left, right }
   ```

5. **Divide** - Division operation
   ```rust
   EchoAst::Divide { left, right }
   ```

6. **Modulo** - Modulo operation
   ```rust
   EchoAst::Modulo { left, right }
   ```

7. **Power** - Power operation (falls back to interpreter)
   ```rust
   EchoAst::Power { left, right }
   ```

8. **UnaryMinus** - Unary negation
   ```rust
   EchoAst::UnaryMinus { operand }
   ```

9. **UnaryPlus** - Unary plus (no-op)
   ```rust
   EchoAst::UnaryPlus { operand }
   ```

### ❌ Not Supported (Everything Else)

#### Literals & Basic Types
- [ ] Float
- [ ] String
- [ ] Boolean
- [ ] Null

#### Identifiers & References
- [ ] Identifier
- [ ] SystemProperty ($propname)
- [ ] ObjectRef (#123)

#### Arithmetic Operations
- [x] Subtract
- [x] Multiply
- [x] Divide
- [x] Modulo
- [x] Power (falls back to interpreter)
- [x] UnaryMinus
- [x] UnaryPlus

#### Comparison Operations
- [ ] Equal
- [ ] NotEqual
- [ ] LessThan
- [ ] LessEqual
- [ ] GreaterThan
- [ ] GreaterEqual
- [ ] In

#### Logical Operations
- [ ] And
- [ ] Or
- [ ] Not

#### Variable Operations
- [ ] Assignment
- [ ] LocalAssignment
- [ ] ConstAssignment

#### Property & Method Access
- [ ] PropertyAccess
- [ ] MethodCall
- [ ] FunctionCall
- [ ] Call (lambda calls)
- [ ] IndexAccess

#### Collections
- [ ] List
- [ ] Map

#### Functions
- [ ] Lambda

#### Control Flow
- [ ] If
- [ ] While
- [ ] For
- [ ] Return
- [ ] Break
- [ ] Continue

#### Advanced Features
- [ ] Emit (events)
- [ ] ObjectDef
- [ ] Try/Catch/Finally
- [ ] Event
- [ ] Spawn
- [ ] Await
- [ ] Match
- [ ] TypedIdentifier
- [ ] ExpressionStatement
- [ ] Block
- [ ] Program

## Coverage Statistics

- **Total AST Node Types**: ~50+
- **JIT Supported**: 9
- **Coverage**: ~18%

## Current Limitations

1. **No Variable Support**: Can't access or assign variables
2. **No Function Calls**: Can't call functions or methods
3. **No Control Flow**: No if/else, loops, or jumps
4. **No String Operations**: Only integers are supported
5. **No Memory Access**: Can't read/write to storage

## What Actually Gets Compiled?

When JIT is enabled, these expressions compile to native code:
```echo
42                    // Number literal
10 + 32              // Addition
42 - 10              // Subtraction
6 * 7                // Multiplication
84 / 2               // Division
17 % 5               // Modulo
2 ** 5               // Power (falls back to interpreter)
-42                  // Unary minus
+42                  // Unary plus
1 + 2 * 3            // Nested arithmetic
```

Everything else falls back to the interpreter.

## Next Steps for JIT Implementation

To make the JIT useful, the following should be prioritized:

### Phase 1: Basic Expressions
1. ~~Other arithmetic operations (Subtract, Multiply, Divide)~~ ✓ COMPLETED
2. Boolean literals and operations
3. Comparison operations
4. Variable reads (Identifier)

### Phase 2: Memory & Control
1. Variable assignment
2. If/else statements
3. Function calls
4. Return statements

### Phase 3: Advanced Features
1. Loops (While, For)
2. Property access
3. Method calls
4. Lists and indexing

## Technical Notes

The JIT uses Cranelift's intermediate representation (IR) and compiles to native machine code. The current implementation:

- Uses `iconst` for integer constants
- Uses `iadd` for integer addition
- Returns i64 values
- Has proper block sealing for Cranelift
- Works on x86_64 and ARM64 (with is_pic workaround)

However, it lacks:
- Variable storage/retrieval mechanism
- Function call conventions
- Memory management integration
- Type system integration
- Error handling in compiled code