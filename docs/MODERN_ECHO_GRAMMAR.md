# Modern Echo Grammar Design

Since we're using MOO import via CST transformation, Echo can have a cleaner,
more modern grammar:

## Variable Declaration & Assignment

```echo
// Explicit variable declaration (avoids conflicts)
let x = 42;
let name = "Alice";

// Reassignment
x = 100;

// Const for immutability
const PI = 3.14159;
```

## Object Definitions

```echo
// Echo will retain its prototype-based inheritance model,
// using MOO-style `object` definitions for all object creation.

object Person
    property name = "Anonymous";
    property age = 0;

    verb greet() {
        return "Hello, " + this.name;
    }
endobject
```

## Control Structures

```echo
// Modern block syntax with braces
if (x > 10) {
    print("Large number");
} else {
    print("Small number");
}

// Or Python-like indentation
if x > 10:
    print("Large number")
else:
    print("Small number")

// Loops
for item in list {
    process(item);
}

while (condition) {
    doWork();
}
```

## Type Annotations (optional)

```echo
let count: number = 0;
let name: string = "Echo";
let items: list<string> = ["a", "b", "c"];

function add(a: number, b: number): number {
    return a + b;
}
```

## Modern Features

```echo
// String interpolation
let message = `Hello, ${name}! You are ${age} years old.`;

// Destructuring
let {name, age} = person;
let [first, second, ...rest] = items;

// Arrow functions
let double = (x) => x * 2;
let greet = (name) => `Hello, ${name}!`;

// Async/await for green threads
async function fetchData() {
    let result = await queryDatabase();
    return process(result);
}

// Pattern matching
match value {
    Number(n) => print(`Got number: ${n}`),
    String(s) => print(`Got string: ${s}`),
    Object(id) => print(`Got object: ${id}`),
    _ => print("Unknown type")
}
```

## Benefits of This Approach

1. **No Grammar Conflicts**: Modern syntax avoids LR(1) conflicts
2. **Better Tooling**: Cleaner AST for IDE support, linting, etc.
3. **Familiar to Modern Developers**: Similar to TypeScript, Rust, Swift
4. **MOO Compatibility**: Import legacy code via transformation
5. **Progressive Enhancement**: Can add features without breaking compatibility
