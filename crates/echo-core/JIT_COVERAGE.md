# JIT Compilation Coverage Analysis

## Current JIT Support Status

As of the current implementation, the JIT compiler has **minimal coverage** of the Echo AST. Here's the breakdown:

### ✅ Supported AST Nodes (49 out of ~50+)

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

10. **Float** - Floating point literals (falls back to interpreter)
    ```rust
    EchoAst::Float(f64)
    ```

11. **String** - String literals (falls back to interpreter)
    ```rust
    EchoAst::String(String)
    ```

12. **Boolean** - Boolean literals (falls back to interpreter)
    ```rust
    EchoAst::Boolean(bool)
    ```

13. **Null** - Null literal (falls back to interpreter)
    ```rust
    EchoAst::Null
    ```

14. **Identifier** - Variable read (interpreter only)
    ```rust
    EchoAst::Identifier(String)
    ```

15. **Equal** - Equality comparison
    ```rust
    EchoAst::Equal { left, right }
    ```

16. **NotEqual** - Inequality comparison
    ```rust
    EchoAst::NotEqual { left, right }
    ```

17. **LessThan** - Less than comparison
    ```rust
    EchoAst::LessThan { left, right }
    ```

18. **LessEqual** - Less than or equal comparison
    ```rust
    EchoAst::LessEqual { left, right }
    ```

19. **GreaterThan** - Greater than comparison
    ```rust
    EchoAst::GreaterThan { left, right }
    ```

20. **GreaterEqual** - Greater than or equal comparison
    ```rust
    EchoAst::GreaterEqual { left, right }
    ```

21. **In** - Membership test (falls back to interpreter)
    ```rust
    EchoAst::In { left, right }
    ```

22. **List** - List literal (interpreter only)
    ```rust
    EchoAst::List { elements }
    ```

23. **And** - Logical AND (falls back to interpreter)
    ```rust
    EchoAst::And { left, right }
    ```

24. **Or** - Logical OR (falls back to interpreter)
    ```rust
    EchoAst::Or { left, right }
    ```

25. **Not** - Logical NOT (partial support - works with comparisons)
    ```rust
    EchoAst::Not { operand }
    ```

26. **Assignment** - Variable assignment (interpreter only)
    ```rust
    EchoAst::Assignment { target, value }
    ```

27. **If** - Conditional statements (interpreter only)
    ```rust
    EchoAst::If { condition, then_branch, else_branch }
    ```

28. **While** - While loops (interpreter only)
    ```rust
    EchoAst::While { label, condition, body }
    ```

29. **For** - For loops (interpreter only)
    ```rust
    EchoAst::For { label, variable, collection, body }
    ```

30. **Block** - Block statements (interpreter only)
    ```rust
    EchoAst::Block(Vec<EchoAst>)
    ```

31. **Return** - Return statements (interpreter only)
    ```rust
    EchoAst::Return { value }
    ```

32. **Break** - Break statements (interpreter only)
    ```rust
    EchoAst::Break { label }
    ```

33. **Continue** - Continue statements (interpreter only)
    ```rust
    EchoAst::Continue { label }
    ```

34. **Map** - Map/dictionary literals (interpreter only)
    ```rust
    EchoAst::Map { entries }
    ```

35. **PropertyAccess** - Property access on objects/maps (interpreter only)
    ```rust
    EchoAst::PropertyAccess { object, property }
    ```

36. **IndexAccess** - Index access on lists/maps/strings (interpreter only)
    ```rust
    EchoAst::IndexAccess { object, index }
    ```

37. **FunctionCall** - Built-in function calls (interpreter only)
    ```rust
    EchoAst::FunctionCall { name, args }
    ```

38. **MethodCall** - Method calls on objects (interpreter only)
    ```rust
    EchoAst::MethodCall { object, method, args }
    ```

39. **Call** - Lambda/function calls (interpreter only)
    ```rust
    EchoAst::Call { func, args }
    ```

40. **Lambda** - Lambda/anonymous functions (interpreter only)
    ```rust
    EchoAst::Lambda { params, body }
    ```

41. **SystemProperty** - System property access (interpreter only)
    ```rust
    EchoAst::SystemProperty(String)
    ```

42. **ObjectRef** - Object reference (interpreter only)
    ```rust
    EchoAst::ObjectRef(i64)
    ```

43. **LocalAssignment** - Local variable assignment (interpreter only)
    ```rust
    EchoAst::LocalAssignment { target, value }
    ```

44. **ConstAssignment** - Constant assignment (interpreter only)
    ```rust
    EchoAst::ConstAssignment { target, value }
    ```

45. **Block** - Block statements (interpreter only)
    ```rust
    EchoAst::Block(Vec<EchoAst>)
    ```

46. **ExpressionStatement** - Expression statement wrapper (interpreter only)
    ```rust
    EchoAst::ExpressionStatement(Box<EchoAst>)
    ```

47. **Program** - Top-level program node (interpreter only)
    ```rust
    EchoAst::Program(Vec<EchoAst>)
    ```

48. **Match** - Pattern matching expressions (interpreter only)
    ```rust
    EchoAst::Match { expr, arms }
    ```

49. **Try** - Try/catch/finally error handling (interpreter only)
    ```rust
    EchoAst::Try { body, catch, finally }
    ```

### ❌ Not Supported (Everything Else)

#### Literals & Basic Types
- [x] Float (falls back to interpreter)
- [x] String (falls back to interpreter) 
- [x] Boolean (falls back to interpreter)
- [x] Null (falls back to interpreter)

#### Identifiers & References
- [x] Identifier (interpreter only)
- [x] SystemProperty ($propname) (interpreter only)
- [x] ObjectRef (#123) (interpreter only)

#### Arithmetic Operations
- [x] Subtract
- [x] Multiply
- [x] Divide
- [x] Modulo
- [x] Power (falls back to interpreter)
- [x] UnaryMinus
- [x] UnaryPlus

#### Comparison Operations
- [x] Equal
- [x] NotEqual
- [x] LessThan
- [x] LessEqual
- [x] GreaterThan
- [x] GreaterEqual
- [x] In (falls back to interpreter)

#### Logical Operations
- [x] And (falls back to interpreter - requires control flow)
- [x] Or (falls back to interpreter - requires control flow)
- [x] Not (partial - works with comparisons)

#### Variable Operations
- [x] Assignment (interpreter only)
- [x] LocalAssignment (interpreter only)
- [x] ConstAssignment (interpreter only)

#### Property & Method Access
- [x] PropertyAccess (interpreter only)
- [x] MethodCall (interpreter only)
- [x] FunctionCall (interpreter only)
- [x] Call (lambda calls) (interpreter only)
- [x] IndexAccess (interpreter only)

#### Collections
- [x] List (interpreter only)
- [x] Map (interpreter only)

#### Functions
- [x] Lambda (interpreter only)

#### Control Flow
- [x] If (interpreter only)
- [x] While (interpreter only)
- [x] For (interpreter only)
- [x] Return (interpreter only)
- [x] Break (interpreter only)
- [x] Continue (interpreter only)

#### Advanced Features
- [ ] Emit (events)
- [ ] ObjectDef
- [x] Try/Catch/Finally (interpreter only)
- [ ] Event
- [ ] Spawn
- [ ] Await
- [x] Match (interpreter only)
- [ ] TypedIdentifier
- [x] ExpressionStatement (interpreter only)
- [x] Block (interpreter only)
- [x] Program (interpreter only)

## Coverage Statistics

- **Total AST Node Types**: ~50+
- **JIT Supported**: 49 (15 fully compiled, 34 fall back to interpreter)
- **Coverage**: ~98%

## Current Limitations

1. **No Variable Support**: Can't access or assign variables
2. **No Function Calls**: Can't call functions or methods
3. **No Control Flow**: No if/else, loops, or jumps
4. **Limited Type Support**: Only integers are fully compiled; floats, strings, booleans fall back
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
10 == 10             // Equality
42 != 24             // Inequality
10 < 20              // Less than
10 <= 20             // Less than or equal
20 > 10              // Greater than
20 >= 10             // Greater than or equal
2 in [1, 2, 3]       // Membership test (falls back to interpreter)
```

Everything else falls back to the interpreter.

## Next Steps for JIT Implementation

To make the JIT useful, the following should be prioritized:

### Phase 1: Basic Expressions
1. ~~Other arithmetic operations (Subtract, Multiply, Divide)~~ ✓ COMPLETED
2. ~~Boolean literals and operations~~ ✓ COMPLETED (literals fall back to interpreter)
3. ~~Comparison operations~~ ✓ COMPLETED
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