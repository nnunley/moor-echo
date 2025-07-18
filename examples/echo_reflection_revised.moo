// ECHO Reflection - Revised Syntax
// Avoiding conflicts with MOO's $system_property syntax

// ========== META-OBJECT ACCESS PATTERNS ==========

// Option 1: Using @ prefix for meta operations
let player_meta = @meta(player);
let properties = @properties(player);
let verbs = @verbs(player);

// Option 2: Using :: for meta namespace
let player_meta = player::meta;
let properties = player::properties;
let verbs = player::verbs;

// Option 3: Using built-in functions
let player_meta = meta_object(player);
let properties = meta_properties(player);
let verbs = meta_verbs(player);

// Option 4: Special property with different syntax
let player_meta = player.<meta>;
let properties = player.<properties>;
let verbs = player.<verbs>;

// ========== RECOMMENDED SYNTAX: BUILT-IN FUNCTIONS ==========

// Core meta functions that don't conflict with MOO syntax
meta_object(obj)                    // Get meta-object
meta_properties(obj)                // List properties
meta_verbs(obj)                     // List verbs  
meta_events(obj)                    // List events
meta_queries(obj)                   // List associated queries
meta_capabilities(obj)              // List capabilities

// Detailed introspection
meta_property_info(obj, "health")   // Get property details
meta_verb_info(obj, "move")         // Get verb details
meta_event_info(obj, "PlayerMoved") // Get event details

// ========== EXAMPLES WITH CLEAN SYNTAX ==========

// Property introspection
let prop_info = meta_property_info(player, "health");
prop_info["name"]                   // => "health"
prop_info["type"]                   // => "integer"
prop_info["value"]                  // => 100
prop_info["readable"]               // => true
prop_info["writable"]               // => true
prop_info["observers"]              // => [#health_monitor, #combat_system]

// Verb introspection
let verb_info = meta_verb_info(player, "move");
verb_info["signature"]              // => "(this none this)"
verb_info["secure"]                 // => true
verb_info["capabilities"]           // => ["CanMove"]
verb_info["source_location"]        // => "#123:45-67"

// Event discovery
let emitted = meta_emits(player);
// => ["PlayerMoved", "HealthChanged", "ItemPickedUp"]

let handled = meta_handles(room);
// => ["PlayerMoved", "ItemDropped", "SpellCast"]

// ========== RUNTIME MODIFICATION ==========

// Add property dynamically
meta_add_property(player, "reputation", [
    "type" -> "integer",
    "default" -> 0,
    "readable" -> true,
    "writable" -> false
]);

// Add verb dynamically
meta_add_verb(player, "greet", [
    "signature" -> "(dobj none none)",
    "code" -> {dobj} => {
        dobj:tell(this.name + " greets you warmly.");
        emit GreetingExchanged(this, dobj);
    }
]);

// Wrap existing verb
meta_wrap_verb(player, "move", [
    "before" -> {direction} => {
        if (this.paralyzed)
            throw E_PERM, "You cannot move while paralyzed!";
        endif
    },
    "after" -> {direction, result} => {
        if (result["success"])
            this.movement_count = this.movement_count + 1;
        endif
    }
]);

// ========== REFLECTION QUERIES ==========

// These work with existing Datalog syntax
query has_verb(Object, VerbName) :-
    meta_verb_exists(Object, VerbName).

query emits_event(Object, EventName) :-
    meta_emits_list(Object, Events),
    member(EventName, Events).

query property_type(Object, Property, Type) :-
    meta_property_info(Object, Property, Info),
    Info["type"] == Type.

// ========== HIGHER-ORDER FUNCTIONS ==========

// Object inspection utility
fn inspect(obj, ?detail = "summary")
    let info = ["=== Object Inspection: " + tostr(obj) + " ==="];
    
    // Basic information
    info = {@info, "Type: " + typeof(obj)};
    info = {@info, "Parent: " + parent(obj)};
    
    if (detail in ["detailed", "full"])
        // Properties
        info = {@info, "", "Properties:"};
        for prop in (meta_properties(obj))
            let pinfo = meta_property_info(obj, prop);
            info = {@info, "  " + prop + ": " + pinfo["type"] + " = " + pinfo["value"]};
        endfor
        
        // Verbs
        info = {@info, "", "Verbs:"};
        for verb in (meta_verbs(obj))
            let vinfo = meta_verb_info(obj, verb);
            info = {@info, "  " + verb + vinfo["signature"]};
        endfor
    endif
    
    return join(info, "\n");
endfn

// ========== SAFE META-OPERATIONS ==========

// Using standard MOO security model
fn secure_meta_read(obj, aspect)
    // Check permissions using standard MOO patterns
    if (!obj:is_readable_by(caller_perms()))
        return E_PERM;
    endif
    
    if (aspect == "properties")
        return meta_properties(obj);
    elseif (aspect == "verbs")  
        return meta_verbs(obj);
    else
        return E_INVARG;
    endif
endfn

// ========== ALTERNATIVE: META AS VIRTUAL OBJECT ==========

// Another approach: $meta as a system object that handles all reflection
// This fits MOO's existing patterns better

// Query meta information through $meta system object
$meta:properties(player)            // => ["name", "health", "location", ...]
$meta:verbs(player)                 // => ["move", "look", "say", ...]
$meta:events(player)                // => ["PlayerMoved", "HealthChanged", ...]

// Modification through $meta
$meta:add_property(player, "reputation", [...]);
$meta:wrap_verb(player, "move", [...]);
$meta:add_handler(player, "ItemDropped", [...]);

// This approach has advantages:
// 1. Fits MOO's object-oriented model
// 2. Security through normal verb permissions on $meta
// 3. No new syntax needed
// 4. Can be implemented in pure MOO

// ========== HYBRID APPROACH ==========

// Combine convenience functions with $meta system object

// Convenience functions for common operations
properties(obj)                     // => $meta:properties(obj)
verbs(obj)                         // => $meta:verbs(obj)
events(obj)                        // => $meta:events(obj)

// Direct $meta access for advanced operations
$meta:wrap_verb(obj, "move", wrapper_spec);
$meta:add_trait(obj, trait_name);
$meta:profile(obj, duration);

// ========== PRACTICAL EXAMPLES ==========

// Development tool using clean syntax
fn find_objects_with_verb(verb_name)
    let results = {};
    
    for obj in (all_objects())
        if (verb_name in verbs(obj))
            results = {@results, obj};
        endif
    endfor
    
    return results;
endfn

// Add logging to all verbs on an object
fn add_logging(obj)
    for verb_name in (verbs(obj))
        $meta:wrap_verb(obj, verb_name, [
            "before" -> {@args} => {
                $logger:log_call(obj, verb_name, args);
            }
        ]);
    endfor
endfn

// Property observer pattern
fn observe_property(obj, prop_name, observer)
    $meta:add_observer(obj, prop_name, observer);
    
    // Observer will be called as:
    // observer:property_changed(obj, prop_name, old_value, new_value)
endfn

// ========== CAPABILITY-BASED META ACCESS ==========

// Standard MOO-style permission checking
fn get_verb_code(obj, verb_name)
    // Use existing MOO permission model
    if (!$perm_utils:controls(caller_perms(), obj))
        return E_PERM;
    endif
    
    let info = $meta:verb_info(obj, verb_name);
    return info ? info["code"] | E_VERBNF;
endfn

// ========== BACKWARDS COMPATIBILITY ==========

// These reflection features are additive to MOO
// All existing MOO code continues to work:

// Traditional MOO system properties still work
$server_version                     // => "1.8.1"
$maxint                            // => 2147483647
$list                              // => {list utilities}

// New reflection via functions or $meta object
properties(player)                  // => ["name", "health", ...]
$meta:properties(player)           // => ["name", "health", ...]

// Clear distinction between:
// - System properties: $foo (existing MOO)
// - Object properties: obj.foo (existing MOO)  
// - Meta operations: meta_foo(obj) or $meta:foo(obj) (new ECHO)

// ========== END REVISED REFLECTION EXAMPLES ==========