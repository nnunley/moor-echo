# REPL Parser Design

The REPL (Read-Eval-Print Loop) parser needs to handle a complex set of requirements beyond simple expression evaluation.

## Requirements

### 1. Extended Commands
The REPL must recognize and parse special commands:
- `/help` - Show available commands
- `/load <file>` - Load Echo/MOO code from file
- `/save <file>` - Save current session
- `/objects` - List objects in the system
- `/verbs <object>` - List verbs on an object
- `/edit <object:verb>` - Edit a verb
- `/quit` - Exit the REPL
- `/debug on|off` - Toggle debug mode
- `/stats` - Show system statistics

### 2. MOO Statement Parsing
The REPL must parse full MOO language statements:
- Variable assignments: `x = 5;`
- Property access: `player.name`
- Method calls: `player:tell("Hello");`
- Object definitions (multi-line)
- Control structures (if/while/for)

### 3. Multi-line Statement Support
The REPL needs continuation awareness:
- Detect incomplete statements
- Handle explicit continuations (backslash at end of line)
- Track open delimiters: `{`, `[`, `(`, `"`
- Support multi-line constructs:
  ```moo
  if (condition)
      statement1;
      statement2;
  endif
  ```

### 4. Interactive Features
- Tab completion for identifiers, properties, and verbs
- Syntax highlighting hints
- Error recovery - don't crash on parse errors
- History management

## Parser Architecture

```
┌─────────────────────┐
│   REPL Input Line   │
└──────────┬──────────┘
           │
    ┌──────▼──────┐
    │ Line Parser │ ← Detects command vs statement
    └──────┬──────┘
           │
    ┌──────┴───────────────┬─────────────────┐
    │                      │                 │
┌───▼────┐         ┌───────▼──────┐   ┌─────▼─────┐
│Command │         │ Continuation │   │Statement  │
│Parser  │         │   Detector   │   │  Parser   │
└────────┘         └───────┬──────┘   └─────┬─────┘
                           │                 │
                    ┌──────▼──────┐          │
                    │ Line Buffer │◄─────────┘
                    │ Accumulator │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ MOO Parser  │
                    │(rust-sitter)│
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Unified AST │
                    └─────────────┘
```

## Implementation Plan

### Phase 1: Command Parser
```rust
pub enum ReplCommand {
    Help,
    Load { file: PathBuf },
    Save { file: PathBuf },
    Objects,
    Verbs { object: String },
    Edit { object: String, verb: String },
    Debug { enabled: bool },
    Stats,
    Quit,
}

pub fn parse_command(line: &str) -> Option<ReplCommand> {
    if !line.starts_with('/') {
        return None;
    }
    // Parse command...
}
```

### Phase 2: Continuation Detector
```rust
pub struct ContinuationDetector {
    open_parens: i32,
    open_braces: i32,
    open_brackets: i32,
    in_string: bool,
    in_block: bool, // if/while/for/object blocks
}

impl ContinuationDetector {
    pub fn needs_continuation(&self, line: &str) -> bool {
        // Check for:
        // - Unclosed delimiters
        // - Backslash at end
        // - Incomplete block structures
    }
}
```

### Phase 3: Statement Accumulator
```rust
pub struct StatementAccumulator {
    lines: Vec<String>,
    detector: ContinuationDetector,
}

impl StatementAccumulator {
    pub fn add_line(&mut self, line: &str) -> StatementResult {
        self.lines.push(line.to_string());
        
        if self.detector.needs_continuation(line) {
            StatementResult::NeedMore
        } else {
            let complete = self.lines.join("\n");
            self.lines.clear();
            StatementResult::Complete(complete)
        }
    }
}
```

### Phase 4: Integration with MOO Parser
- Use the existing rust-sitter MOO parser for complete statements
- Handle partial parsing for syntax highlighting
- Provide error recovery mechanisms

## Testing Strategy

### Unit Tests
- Command parsing for all command variants
- Continuation detection for various scenarios
- Statement accumulation edge cases

### Integration Tests
- Multi-line if/while/for statements
- Object definitions spanning multiple lines
- String literals with embedded newlines
- Nested structures

### REPL Tests
- Interactive session simulation
- Error recovery scenarios
- Command history navigation

## Future Enhancements

1. **Syntax Highlighting**: Real-time highlighting as user types
2. **Auto-completion**: Context-aware completion for properties and methods
3. **Inline Documentation**: Show help for functions/verbs on hover
4. **Macro System**: User-defined command shortcuts
5. **Script Mode**: Execute REPL scripts from files