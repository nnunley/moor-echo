// MOO/MOOR Language Syntax Highlighting Demo

// ========== Basic Constructs ==========

// Variables and assignments
let x = 42;
const name = "Ryan";
global counter = 0;

// Basic arithmetic with binary operators
let result = a + b * c / 2 - 1;
let comparison = (x >= 10) && (y < 20) || (z == 0);

// ========== MOO-specific Features ==========

// Object references
let obj = #123;
let parent = #"generic object";

// Property access
let value = obj.property;
let computed = obj.(prop_name);

// System properties
let version = $server_version;

// ========== MOOR Extensions ==========

// Lambda expressions (new in MOOR)
let add = {x, y} => x + y;
let no_params = {} => "hello world";
let with_defaults = {x, ?y = 5, @rest} => x + y;

// Function definitions (new in MOOR)
fn factorial(n)
    if (n <= 1)
        return 1;
    else
        return n * factorial(n - 1);
    endif
endfn

// List comprehensions (new in MOOR)
let squares = {x * x for x in [1..10]};
let filtered = {item for item in list if (item > 0)};

// Scatter assignments (enhanced in MOOR)
let {first, ?second = "default", @remaining} = some_list;
const {x, y, z} = get_coordinates();

// ========== Control Flow ==========

// Conditional statements
if (condition)
    do_something();
elseif (other_condition)
    do_other();
else
    do_default();
endif

// Loops
for item in (my_list)
    process(item);
endfor

for i in [1..100]
    if (i % 2 == 0)
        continue;
    endif
    print(i);
endfor

// While loops with labels
while main_loop (keep_running)
    // Some processing
    if (should_break)
        break main_loop;
    endif
endwhile

// ========== Error Handling ==========

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

// ========== Collections ==========

// Lists
let numbers = {1, 2, 3, 4, 5};
let mixed = {"string", 42, #123, 'symbol};
let empty = {};

// Maps/dictionaries
let config = ["host" -> "localhost", "port" -> 8080];
let nested = [
    "user" -> ["name" -> "Ryan", "id" -> 1001],
    "prefs" -> ["theme" -> "dark", "lang" -> "en"]
];

// ========== Function Calls and Verbs ==========

// Function calls
result = calculate(x, y, z);
formatted = sprintf("Value: %d", count);

// Verb calls
obj:method(arg1, arg2);
player:tell("Hello, world!");

// Pass statement
pass(@args);

// ========== Advanced Features ==========

// Flyweights (prototypes)
let instance = <parent_obj, {"prop" -> "value"}, {"item1", "item2"}>;

// Fork statements
fork background_task (60)
    // Long-running operation
    process_data();
endfork

// ========== Object Definitions ==========

object TestObject
    property name = "Test Object";
    property value (readable: true, writable: false) = 42;
    
    verb "initialize" (this none this)
        this.name = "Initialized";
        this.value = random(100);
    endverb
    
    verb "describe" (this none this)
        return sprintf("Object %s with value %d", this.name, this.value);
    endverb
endobj