// Test scatter assignment patterns

// Basic scatter assignment  
{a, b, c} = some_list;

// Optional and rest parameters in let statements
let {x, ?y = 5, @rest} = some_list;
const {first, second, @remaining} = data;

// In function parameters (already supported)
fn process_args({name, ?age = 18, @others})
    return name + " is " + tostr(age);
endfn

// Scatter in lambda parameters
let handler = {x, ?default = 0, @args} => x + default;