// Test new MOOR language features

// Lambda expressions
let add = {x, y} => x + y;
let square = {x} => x * x;
let greet = {name} => "Hello, " + name;
let random_func = {} => random(100);

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

// List comprehensions
let numbers = {1, 2, 3, 4, 5};
let doubled = {x * 2 for x in (numbers)};
let squares = {x * x for x in [1..10]};

// Nested function calls
let result = calculate_damage(player, monster);
let processed = add(square(5), 10);