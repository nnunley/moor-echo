# MOO Features Needed for Cowbell Import

Based on inspection of the cowbell MOO files, here are features that need implementation:

## 1. Destructuring with Optional Parameters ✅ CRITICAL
MOO supports optional parameters in destructuring with default values:
```moo
{name, ?password = 0} = args;
{things, ?nothingstr = "nothing", ?andstr = " and "} = args;
```
This is used extensively throughout the codebase.

## 2. Try/Except/Endtry Exception Handling ✅ CRITICAL
MOO has structured exception handling:
```moo
try
  object = object.(pn);
except (ANY)
  return $failed_match;
endtry

except (E_ARGS)
  notify(player, "Usage error");
endtry
```

## 3. Verb Definition Parsing Patterns ✅ CRITICAL
Verbs can have multiple names and wildcards:
```moo
verb "l look" (any none none)
verb "co*nnect @co*nnect" (any none any)
verb "pronoun_*" (this none this)
```

## 4. Range Syntax for Lists/Strings ✅ CRITICAL
MOO uses `[start..end]` syntax for slicing:
```moo
string[1..dot - 1]
fill[1..n]
for i in [1..length(string)]
```

## 5. Ternary Operator ✅ CRITICAL
MOO uses `condition ? true_val | false_val`:
```moo
return len > 0 ? out | out[1..abslen];
typelist = typeof(args[2]) == list ? args[2] | {args[2]};
```

## 6. Dynamic Property Access ✅ IMPORTANT
Accessing properties with computed names:
```moo
object = object.(pn);  // where pn is a string variable
```

## 7. raise() Built-in Function ✅ CRITICAL
For throwing exceptions:
```moo
raise(E_PERM);
raise(E_INVARG, "Invalid password.");
```

## 8. Special Variables and Built-ins
- `player` - the current player object
- `caller` - who called this verb
- `this` - the current object
- `args` - verb arguments
- `verb` - the verb name that was matched
- `E_PERM`, `E_ARGS`, `E_INVARG` etc. - error constants

## 9. List Comprehensions
```moo
{ ic + "." for ic in (integrated_contents) }
```

## 10. Built-in Functions Used
- `notify(player, message)` - send message to player
- `valid(object)` - check if object is valid
- `is_player(object)` - check if object is a player
- `players()` - get list of all players
- `strsub(str, find, replace)` - string substitution
- `index(str, substr)` - find substring position
- `tostr(...)` - convert to string
- `typeof(value)` - get type of value
- `length(list/string)` - get length
- `listappend/listdelete` - list operations
- `create(parent, owner)` - create new object
- `move(object, destination)` - move object
- `set_player_flag(object, bool)` - set player flag
- `verb_args(object, verb_name)` - get verb argument pattern
- `verb_info(object, verb_name)` - get verb info
- `connection_name(player)` - get connection info
- `server_log(message)` - log to server

## Test Coverage Needed

1. **Parser Tests**
   - Test all new syntax features
   - Test error recovery for malformed MOO code
   - Test preprocessor with complex define scenarios

2. **Evaluator Tests**  
   - Test exception handling flow
   - Test optional parameter destructuring
   - Test dynamic property access
   - Test ternary operator evaluation
   - Test range/slice operations

3. **Integration Tests**
   - Import all cowbell MOO files
   - Execute simple verbs
   - Test verb pattern matching
   - Test exception propagation

## Current Status

✅ Completed:
- Basic destructuring (without optional parameters)
- Map literals
- Spread operator (basic)
- Error catching (backtick syntax)
- Flyweight objects
- Define preprocessing
- Object import with ID mapping

❌ Still Needed:
- Optional parameters in destructuring
- Try/except/endtry blocks
- Verb pattern parsing
- Range/slice syntax
- Ternary operator
- Dynamic property access
- raise() builtin
- List comprehensions
- MOO built-in functions
- Comprehensive test suite