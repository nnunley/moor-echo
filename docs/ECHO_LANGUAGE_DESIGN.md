# ECHO Language Design

**ECHO** - _Event-Centered Hybrid Objects_

A modern, cohesive evolution of the MOO language that combines Wirthian
principles with a powerful, object-centric, and multi-paradigm programming model
designed for the next generation of interactive worlds.

## Language Philosophy

ECHO represents a significant but natural evolution of MOO. It preserves the
core teaching-friendly philosophy while adding powerful, composable features for
building sophisticated, secure, and intelligent interactive systems.

### Core Principles

1.  **The Object as a Module**: The object is the fundamental unit of code,
    state, and logic. It encapsulates its properties, verbs, internal functions,
    event handlers, and declarative queries into a single, inheritable module.
2.  **Cohesive by Design**: The language feels like one system. All structures
    follow established MOO conventions. A unified `define` keyword is used for
    simple type declarations, while major constructs like `object` are
    standalone blocks.
3.  **Clear over Clever**: Syntax is explicit and self-documenting, favoring
    readability to empower learners. A clear distinction between player-facing
    `verbs` and internal `functions` clarifies programmer intent.
4.  **Pervasive & Colorless Concurrency**: All code is non-blocking by default.
    The language has no `async` keyword or "function colors." Concurrency is
    managed transparently by the runtime, making asynchronous code as simple to
    write as synchronous code.
5.  **Backward Compatible**: ECHO is a strict superset of MOO. All existing MOO
    code runs unchanged.

## Core Concepts

### 1. The Self-Contained Object

The `object...endobject` block is the heart of the language. It bundles all
related logic into one place, making code more organized, discoverable, and
reusable through inheritance.

```echo
object a_secure_document extends $thing
    // Properties (State)
    property content = "The secret is a secret.";

    // Query (Declarative Logic)
    query can_be_read_by(user) :-
        has_capability(user, ReadDocument(this, user)).

    // Event Handler (Reactive Logic)
    on DocumentRead(doc, reader) where doc == this
        this.access_log = {@this.access_log, {reader, time()}};
    endon

    // Function (Internal Logic)
    function read_by(user)
        // ...
    endfunction

    // Verb (Player-Facing Command Interface)
    verb "read" (this, "none", "none")
        // ...
    endverb
endobject
```

### 2. Unified Event & Exception Model

Exceptions are treated as high-priority events, unifying error handling with the
core event system.

- **Errors are Objects:** `throw $PermissionError("Access denied.")` raises a
  structured error object.
- **`try...catch` is a Local Listener:** It handles errors within a specific
  block of code, using pattern matching on the error type.
- **`on error thrown` is a Global Supervisor:** An object can listen for
  specific errors happening anywhere in the system, allowing for powerful
  logging, auditing, and security monitoring.

```echo
object a_security_auditor
    // This handler listens globally for any PermissionError
    on error thrown as e ($PermissionError) where this:is_critical(e.target)
        this:page_admin("CRITICAL: ", e.message);
    endon
endobject
```

### 3. Pervasive & Structured Concurrency

All function calls are non-blocking. The runtime automatically and transparently
manages cooperative threading.

- **No `async`/`await` Keywords:** All functions are "colorless."
- **Implicit Await-on-Use:** A task automatically pauses when it tries to use
  the result of a function that hasn't completed yet.
- **`gather` for Efficiency:** A `gather` block tells the runtime to wait for
  multiple concurrent operations in parallel, ensuring efficient scatter/gather
  patterns.
- **Manual Control:** `yield()` and `sleep(seconds)` provide explicit control
  over the scheduler when needed.

```echo
function fetch_dashboard_data(user_id)
    // Scatter: These start concurrently and return Futures immediately.
    let user_future = $api:get_user(user_id);
    let posts_future = $api:get_posts(user_id);

    // Gather: This block waits for BOTH futures to complete in parallel.
    gather
        // Implicit await-on-use happens here, inside the gather block.
        let user_name = user_future.name;
        let post_count = length(posts_future);
        $ui:render_header(user_name, post_count);
    endgather
endfunction
```

### 4. Intent-Driven NLP

The LLM is integrated as a first-class "sense," translating player input into
structured `IntentRecognized` events. Verbs subscribe to these intents.

```echo
object a_shopkeeper
    // This verb handles both direct calls to :quote_price and
    // natural language questions about price.
    verb "quote_price" (this, "about", dobjstr)
        handles intent "ask_price" with confidence 0.85

        let item = this:find_item_by_name(dobjstr);
        if (item)
            caller:tell("That will be ", item.price, " gold.");
        else
            caller:tell("I don't have that in stock.");
        endif
    endverb
endobject
```

A player typing `"how much for the shield?"` would trigger an `IntentRecognized`
event with the intent `"ask_price"`, which in turn executes the `quote_price`
verb.

## ECHO Language Grammar (EBNF)

```ebnf
(* ========== PROGRAM STRUCTURE ========== *)
program = { global_definition | object_definition } ;

(* Use 'define' ONLY for simple, global type declarations *)
global_definition = event_definition
                  | capability_definition ;

event_definition      = "define" "event" identifier "(" [ parameter_list ] ")" ";" ;
capability_definition = "define" "capability" identifier "(" [ parameter_list ] ")" ";" ;


(* ========== OBJECT DEFINITION (The Core Module) ========== *)
object_definition = "object" identifier [ "extends" expression ]
                      { object_member }
                    "endobject" ;

object_member = property_definition
              | function_definition
              | verb_definition
              | query_definition
              | event_handler ;

property_definition = "property" identifier [ "(" property_options ")" ] [ "=" expression ] ";" ;

function_definition = [ "secure" ] "function" identifier "(" [ parameter_list ] ")"
                        [ "requires" capability_list ]
                        statement_block
                      "endfunction" ;

verb_definition = [ "secure" ] "verb" string_literal "(" verb_signature ")"
                    [ handles_clause ]
                    [ "requires" capability_list ]
                    statement_block
                  "endverb" ;

verb_signature = expression "," expression "," expression ;
handles_clause = "handles" "intent" string_literal [ "with" "confidence" expression ] ;

query_definition = "query" query_head ":-" query_body ";" ;

event_handler = "on" ( event_pattern | error_event_pattern ) [ "where" expression ]
                  statement_block
                "endon" ;
event_pattern = identifier "(" [ parameter_list ] ")" ;
error_event_pattern = "error" "thrown" "as" identifier "(" identifier ")" ;


(* ========== STATEMENTS & CONTROL FLOW ========== *)
statement_block = { statement } ;

if_statement = "if" ... "endif" ;
while_statement = "while" ... "endwhile" ;
for_statement = "for" ... "endfor" ;

try_statement = "try"
                  statement_block
                { catch_clause }
                [ "finally" statement_block ]
                "endtry" ;

catch_clause = "catch" identifier "(" identifier ")"
                 statement_block ;

gather_statement = "gather"
                     statement_block
                   "endgather" ;

(* Other statements like return, emit, grant, etc. remain *)


(* ========== EXPRESSIONS & BINDING ========== *)
(* Standard expression operators (+, *, ?, :, .) and binding patterns
   (let {a, ?b, @c} = ...) are preserved from classic MOO. *)
```

## Backward Compatibility & Migration

ECHO is a strict superset of MOO. All existing MOO code runs unchanged. The
migration path is gradual and non-disruptive.

1.  **Drop-in Replacement**: Existing MOO code runs as-is.
2.  **Gradual Enhancement**: Use the `object...endobject` syntax for new
    objects.
3.  **Modern Patterns**: Refactor complex logic using object-scoped event
    handlers, queries, and `gather` blocks.
4.  **Full ECHO**: Leverage the complete, cohesive system, including the
    intent-driven NLP, for all new development.

## Conclusion

ECHO represents a significant but natural evolution of MOO. By focusing on
cohesion, clarity, and capability, it provides a powerful platform for building
the next generation of sophisticated multi-user virtual environments. The
language balances a gentle learning curve with the power required for complex,
concurrent, and intelligent systems, ensuring that both beginners and experts
can build and dream in the same world.
