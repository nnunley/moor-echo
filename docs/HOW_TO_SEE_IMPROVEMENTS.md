# How to See the MOO Database Browser Improvements

## Running the Browser

```bash
# From the project root (moor-echo directory):
./target/release/moo_db_browser

# Or from anywhere in the project:
cargo run --bin moo_db_browser

# Or from the crates/echo-core directory:
../../target/release/moo_db_browser
```

## What to Look For

### 1. Properties Tab Improvements
When you select an object and go to the **Properties** tab:

**Short Properties (≤50 chars):**
- Single-line format: `property_name    value    #owner permissions`
- Permissions shown as compact letters: `r`, `rw`, `rwc`

**Long Properties (>50 chars):**
- Two-line format:
  ```
  property_name (#owner) [permissions]
    The long value appears indented on the second line in cyan
  ```

**Example:**
```
name                           "Generic Room"            #2   rw
description (#2) [rw]
  "This is a very long description that will appear on two lines because it exceeds fifty characters"
contents                       {#123, #124, #125}        #2   r
```

### 2. Verbs Tab Improvements
When you go to the **Verbs** tab:

- **Permissions** now show as: `rwxd any none` instead of `read, write, execute, debug, dobj:any, iobj:none (0xad)`
- **Code** is pretty-printed with proper indentation:
  ```
  if (caller != this)
    return E_PERM;
  endif
  for i in [1..10]
    if (i % 2 == 0)
      player:tell("Even: ", i);
    else
      player:tell("Odd: ", i);
    endif
  endfor
  ```

### 3. Overview Tab Improvements
In the **Overview** tab:
- Object flags now show human-readable names: `read, write (0x30)` instead of just `0x30`

## Navigation Tips

1. Use **arrow keys** to navigate the object list
2. Press **Tab** to cycle through tabs (Overview → Properties → Verbs → Relationships)
3. Use **Page Up/Down** or **↑/↓** to scroll within each tab
4. Press **q** to quit

## Testing with Different Databases

The improvements are most visible with databases that have:
- Objects with many properties (to see the two-line format)
- Verbs with code (to see pretty printing)
- Various permission combinations (to see the compact format)

Try loading:
- `LambdaCore-latest.db` - Good variety of objects and verbs
- `JHCore-DEV-2.db` - Large database with complex objects
- `Minimal.db` - Simple database but still shows the improvements