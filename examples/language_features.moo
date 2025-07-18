// MOO/MOOR Language Features - Comprehensive Example Fragments
// This file covers all the functionality we've been experimenting with

// ========== LITERALS AND BASIC TYPES ==========

// Integers
let positive = 42;
let negative = -123;
let zero = 0;

// Floats
let pi = 3.14159;
let scientific = 1.23e-4;
let negative_float = -2.5;
let leading_decimal = .5;

// Strings
let single_quote = 'Hello, world!';
let double_quote = "Hello, world!";
let escaped = "Line 1\nLine 2\tTabbed";

// Booleans
let truth = true;
let falsehood = false;

// Symbols
let symbol_name = 'my_symbol;
let quoted_symbol = 'another_symbol;

// Object IDs
let numbered_object = #123;
let negative_object = #-456;
let named_object = #generic_object;

// System properties
let server_version = $server_version;
let max_object = $max_object;

// Error codes
let no_error = E_NONE;
let permission_error = E_PERM;
let type_error = E_TYPE;

// Range end marker
let range_end = $;

// ========== VARIABLE DECLARATIONS ==========

// Let statements (mutable)
let x = 10;
let y = "hello";
let z = {1, 2, 3};

// Const statements (immutable)
const PI = 3.14159;
const MESSAGE = "Welcome";
const ITEMS = {"apple", "banana", "cherry"};

// Global statements
global counter = 0;
global player_count;  // Declaration without initialization

// ========== SCATTER ASSIGNMENTS ==========

// Basic scatter assignment
{a, b, c} = some_list;

// With optional parameters
let {x, ?y = 5, @rest} = data;
const {first, ?second = "default", @remaining} = get_coordinates();

// Mixed patterns
{name, ?age = 18, @others} = player_info;

// ========== EXPRESSIONS ==========

// Arithmetic
let sum = a + b;
let product = x * y / 2;
let power = base ^ exponent;
let remainder = dividend % divisor;

// Comparison
let equal = a == b;
let not_equal = x != y;
let less_than = a < b;
let greater_equal = x >= y;
let contains = item in list;

// Logical
let and_result = condition1 && condition2;
let or_result = flag1 || flag2;
let not_result = !boolean_value;

// Bitwise
let bit_and = a &. b;
let bit_or = x |. y;
let bit_xor = a ^. b;
let left_shift = value << 2;
let right_shift = value >> 1;

// Assignment operations
x = 42;
name = "New Name";
obj.property = "value";

// Conditional (ternary)
let result = condition ? true_value | false_value;

// ========== PROPERTY AND METHOD ACCESS ==========

// Property access
let value = obj.property;
let computed = obj.(property_name);

// Method calls
obj:method();
player:tell("Hello");
obj:compute(x, y, z);

// Chained access
let deep_value = obj.nested.property;
let chained_method = obj:get_child():method();

// ========== INDEXING AND SLICING ==========

// Index access
let first_item = list[1];
let map_value = dict["key"];

// Slicing
let substring = text[1..5];
let sublist = items[2..10];
let range_slice = data[start..end];

// ========== FUNCTION CALLS ==========

// Simple calls
result = calculate(x, y);
formatted = sprintf("Value: %d", count);

// With splat arguments
call_with_spread(@args);
method:invoke(@parameters);

// Pass statement
pass();
pass(arg1, arg2);
pass(@all_args);

// ========== COLLECTIONS ==========

// Lists
let empty_list = {};
let numbers = {1, 2, 3, 4, 5};
let mixed = {"string", 42, #123, 'symbol};
let with_spread = {first, @middle, last};

// Maps
let empty_map = [];
let config = ["host" -> "localhost", "port" -> 8080];
let nested = [
    "user" -> ["name" -> "Ryan", "id" -> 1001],
    "settings" -> ["theme" -> "dark"]
];

// ========== RANGES ==========

// Simple ranges
let range1 = [1..10];
let range2 = [start..end];
let range3 = [x..y];

// In for loops
for i in [1..100]
    // Process i
endfor

// ========== LAMBDA EXPRESSIONS ==========

// Simple lambdas
let add = {x, y} => x + y;
let square = {x} => x * x;
let greet = {name} => "Hello, " + name;

// No parameters
let random_func = {} => random(100);

// With optional and rest parameters
let flexible = {x, ?y = 5, @rest} => x + y;
let handler = {?default = 0, @args} => process(default, args);

// ========== FUNCTION DEFINITIONS ==========

// Function expressions
let max_func = fn(a, b)
    if (a > b)
        return a;
    else
        return b;
    endif
endfn;

// Function statements
fn calculate_damage(attacker, defender)
    let base_damage = attacker.strength * 2;
    let defense = defender.armor;
    let final_damage = max(1, base_damage - defense);
    return final_damage;
endfn

// With scatter parameters
fn process_data({name, ?age = 18, @others})
    return name + " is " + tostr(age);
endfn

// ========== COMPREHENSIONS ==========

// List comprehensions
let squares = {x * x for x in [1..10]};
let doubled = {x * 2 for x in numbers};
let filtered = {item for item in list};

// With ranges
let range_squares = {i * i for i in [1..5]};

// ========== CONTROL FLOW ==========

// If statements
if (condition)
    do_something();
endif

if (x > 0)
    positive_action();
elseif (x < 0)
    negative_action();
else
    zero_action();
endif

// While loops
while (condition)
    process();
endwhile

// While with labels
while main_loop (keep_running)
    if (should_break)
        break main_loop;
    endif
    if (should_continue)
        continue main_loop;
    endif
endwhile

// For loops
for item in (my_list)
    process(item);
endfor

for i in [1..100]
    if (i % 2 == 0)
        continue;
    endif
    print(i);
endfor

// Block statements
begin
    let temp = calculate();
    process(temp);
    cleanup();
end

// ========== CONTROL FLOW EXPRESSIONS ==========

// Break and continue as expressions
let result = condition ? break | continue;
let value = some_test ? return 42 | process();

// In complex expressions
let outcome = (state == "done") ? return success | (should_retry ? continue | break);

// ========== FORK STATEMENTS ==========

// Simple fork
fork (60)
    background_task();
endfork

// With labels
fork background_task (120)
    long_running_operation();
    cleanup();
endfork

// ========== ERROR HANDLING ==========

// Try-except blocks
try
    risky_operation();
except err (E_PERM, E_ARGS)
    notify("Permission or argument error: " + err);
except (ANY)
    log("Unexpected error occurred");
finally
    cleanup();
endtry

// Try expressions
let safe_value = `risky_call() ! E_NONE => default_value';
let result = `calculate(x, y) ! E_DIV => 0';
let data = `fetch_data() ! ANY => {}';

// ========== FLYWEIGHTS ==========

// Simple flyweight
let instance = <parent_obj>;

// With properties
let configured = <base_obj, ["prop" -> "value"]>;

// With properties and values
let full_flyweight = <parent, ["name" -> "test"], {"item1", "item2"}>;

// ========== OBJECT DEFINITIONS ==========
// Note: Object definitions should be in their own files
// See test_object_only.moo for a complete example

// ========== COMPLEX EXPRESSIONS ==========

// Nested calls and property access
let result = obj:method(x, y).property[index];

// Complex conditionals
let outcome = (a > b) ? obj:higher_method(a) | obj:lower_method(b);

// Chained operations
let processed = data.items[1..5]:map({x} => x * 2):filter({x} => x > 10);

// Mixed operators
let complex = (a + b * c) >= (d - e) && list[index] in valid_values;

// ========== COMMENTS ==========

// Single-line comments
/* Multi-line comments
   can span multiple lines
   and are useful for documentation */

// ========== EXPRESSION STATEMENTS ==========

// Simple expression statements
calculate(x, y);
obj:method();
process_data();

// Complex expression statements
(condition ? action1() | action2());
{x, y, z} = get_coordinates();
list[index] = new_value;

// Assignment expressions
(x = calculate()) > 0 ? success() | failure();

// ========== SEMICOLON HANDLING ==========

// Optional semicolons in many contexts
let x = 42
let y = "hello"
const z = {1, 2, 3}

// Required in some contexts
let a = 5;
if (a > 0)
    process();
endif

// Mixed usage
let result = calculate(x, y)
if (result > 0)
    success();
else
    failure();  
endif