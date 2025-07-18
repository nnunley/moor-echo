// ECHO Reflection and Meta-Object Protocol (MOP)
// This file demonstrates reflective capabilities for discovering and manipulating
// queries, rules, events, and other language constructs at runtime

// ========== META-OBJECT PROTOCOL CORE ==========

// Every object has a meta-object accessible via $meta property
let player_meta = player.$meta;

// Meta-object provides introspection capabilities
player_meta.class                    // => Player
player_meta.parent                    // => $user
player_meta.properties               // => ["name", "location", "health", "mana"]
player_meta.verbs                    // => ["move", "examine", "attack", ...]
player_meta.events                   // => [PlayerMoved, HealthChanged, ...]
player_meta.capabilities             // => [CanMove, CanAttack, ...]
player_meta.queries                  // => [can_see, can_access, ...]

// ========== PROPERTY INTROSPECTION ==========

// Get detailed property information
let prop_info = player_meta.property("health");
prop_info.name                      // => "health"
prop_info.type                      // => Integer
prop_info.value                     // => 100
prop_info.default_value             // => 100
prop_info.readable                  // => true
prop_info.writable                  // => true
prop_info.owner                     // => #123
prop_info.observers                 // => [HealthObserver, CombatSystem, ...]
prop_info.constraints               // => [MinValue(0), MaxValue(player.max_health)]

// Property metadata and annotations
prop_info.metadata                  // => ["ui:display" -> "Health Bar", "ui:color" -> "red"]
prop_info.documentation            // => "Current health points of the player"

// ========== VERB INTROSPECTION ==========

// Get verb metadata
let verb_info = player_meta.verb("move");
verb_info.name                      // => "move"
verb_info.signature                 // => "(this none this)"
verb_info.secure                    // => true
verb_info.capabilities_required     // => [CanMove(this, direction)]
verb_info.source_location          // => #123:45-67
verb_info.bytecode                 // => [compiled bytecode]
verb_info.ast                      // => [abstract syntax tree]

// For LLM-enhanced verbs
let examine_verb = player_meta.verb("examine");
examine_verb.semantics             // => 0.85
examine_verb.embeddings           // => ["look at", "inspect", "check", ...]
examine_verb.context_requirements // => ["mood", "relationship", "location"]

// ========== EVENT INTROSPECTION ==========

// Discover events emitted by an object
let emitted_events = player_meta.emits();
// => [PlayerMoved, HealthChanged, ItemPickedUp, SpellCast, ...]

// Discover event handlers on an object
let handled_events = player_meta.handles();
// => [DamageReceived, SpellTargeted, RoomEntered, ...]

// Get detailed event information
let event_info = $events.get("PlayerMoved");
event_info.name                    // => "PlayerMoved"
event_info.parameters             // => ["player", "from_room", "to_room"]
event_info.emitters               // => [Player, TeleportSystem, ...]
event_info.handlers               // => [Room, Logger, MapSystem, ...]
event_info.total_emissions        // => 15234
event_info.average_handlers       // => 3.4

// Event handler introspection
let handler_info = room_meta.event_handler("PlayerMoved");
handler_info.event                // => PlayerMoved
handler_info.conditions           // => "to == this.location"
handler_info.modifiers            // => ["throttle" -> 500ms]
handler_info.priority             // => 10
handler_info.execution_count      // => 1523
handler_info.average_duration     // => 2.3ms

// ========== QUERY/DATALOG INTROSPECTION ==========

// Discover queries defined on or related to an object
let queries = player_meta.queries();
// => [can_access, can_see, is_friend, guild_member, ...]

// Get query metadata
let query_info = $queries.get("can_access");
query_info.name                   // => "can_access"
query_info.arity                  // => 2
query_info.parameters             // => ["Player", "Object"]
query_info.rules                  // => [Rule1, Rule2, Rule3]
query_info.indexes                // => ["Player", "Object", "Player+Object"]
query_info.materialized           // => false
query_info.cache_stats            // => ["hits" -> 8934, "misses" -> 234]

// Rule introspection
let rule = query_info.rules[0];
rule.head                         // => "can_access(Player, Object)"
rule.body                         // => "owner(Object, Player)"
rule.source_location              // => #123:89-92
rule.execution_count              // => 3421
rule.average_duration             // => 0.05ms

// Query dependencies and relationships
let deps = $queries.dependencies("can_access");
// => ["owner", "permission", "parent"]

let dependents = $queries.dependents("owner");
// => ["can_access", "can_modify", "can_delete"]

// ========== CAPABILITY INTROSPECTION ==========

// Discover capabilities
let caps = player_meta.capabilities();
// => [ReadProperty, WriteProperty, CastSpell, ...]

// Capability details
let cap_info = $capabilities.get("CastSpell");
cap_info.name                     // => "CastSpell"
cap_info.parameters               // => ["spell_type", "min_level"]
cap_info.grantors                // => [SpellSystem, Admin, ...]
cap_info.holders                  // => [#123, #456, ...]
cap_info.check_count              // => 892
cap_info.grant_count              // => 45
cap_info.deny_count               // => 134

// ========== RUNTIME MODIFICATION (MOP) ==========

// Add new properties dynamically
player_meta.add_property("reputation", {
    type: Integer,
    default: 0,
    readable: true,
    writable: false,
    documentation: "Player's reputation score"
});

// Add new verbs dynamically
player_meta.add_verb("greet", {
    signature: "(dobj none none)",
    code: fn(dobj)
        if (dobj && is_player(dobj))
            dobj:tell(this.name + " greets you warmly.");
            emit GreetingExchanged(this, dobj);
        endif
    endfn
});

// Add event handlers dynamically
player_meta.add_event_handler("ItemDropped", {
    condition: "item.value > 100",
    modifiers: {throttle: 1s},
    handler: fn(dropper, item, location)
        this:tell("You notice " + dropper.name + " dropped a valuable " + item.name);
    endfn
});

// Add queries dynamically
$queries.define("is_nearby", {
    parameters: ["Entity1", "Entity2"],
    rules: [
        "is_nearby(E1, E2) :- location(E1, Room), location(E2, Room)",
        "is_nearby(E1, E2) :- location(E1, R1), location(E2, R2), adjacent(R1, R2)"
    ]
});

// ========== HIGHER-ORDER PROGRAMMING ==========

// Method combination and decoration
player_meta.wrap_verb("move", {
    before: fn(direction)
        if (this.paralyzed)
            throw E_PERM, "You cannot move while paralyzed!";
        endif
        emit AttemptingMove(this, direction);
    endfn,
    
    after: fn(direction, result)
        if (result.success)
            this.movement_count = this.movement_count + 1;
            update_map_position(this);
        endif
    endfn,
    
    around: fn(original_verb, direction)
        let start_time = time();
        try
            let result = original_verb(direction);
            return result;
        finally
            log_performance("move", time() - start_time);
        endtry
    endfn
});

// Dynamic proxy objects
let logged_player = create_proxy(player, {
    get_property: fn(name)
        log_access("property", name, this);
        return continue();
    endfn,
    
    call_verb: fn(verb, args)
        log_access("verb", verb, this);
        let result = continue();
        log_result(verb, result);
        return result;
    endfn
});

// ========== REFLECTION QUERIES ==========

// Find all objects that emit a specific event
query emits_event(Object, Event) :-
    object_meta(Object, Meta),
    meta_emits(Meta, Event).

// Find all objects that handle a specific event
query handles_event(Object, Event) :-
    object_meta(Object, Meta),
    meta_handles(Meta, Event).

// Find all objects with a specific capability
query has_capability(Object, Capability) :-
    object_meta(Object, Meta),
    meta_capability(Meta, Capability).

// Find all queries that reference a specific object
query references_object(Query, Object) :-
    query_ast(Query, AST),
    ast_contains_reference(AST, Object).

// ========== META-CIRCULAR EVALUATOR ==========

// The MOP can examine and modify itself
let mop_meta = $mop.$meta;

// Inspect the reflection system
mop_meta.verbs                    // => ["get_meta", "add_property", "wrap_verb", ...]
mop_meta.capabilities             // => [MetaAccess, MetaModify, ...]

// Even the meta-object has a meta-object (meta-meta-object)
let meta_meta = mop_meta.$meta;

// ========== SECURITY AND PERMISSIONS ==========

// Reflection operations require appropriate capabilities
capability MetaRead(object);
capability MetaWrite(object);
capability MetaExecute(object);

// Secure reflection
secure fn get_verb_source(object, verb_name) requires MetaRead(object)
    let meta = object.$meta;
    let verb = meta.verb(verb_name);
    return verb ? verb.source | null;
endfn

// Protected meta-operations
secure fn modify_verb(object, verb_name, new_code) 
    requires MetaWrite(object)
    requires ReviewedCode(new_code)
    let meta = object.$meta;
    meta.replace_verb(verb_name, new_code);
    emit VerbModified(object, verb_name);
endfn

// ========== DEVELOPMENT TOOLS ==========

// Object inspector utility
fn inspect(object, detail_level = "summary")
    let meta = object.$meta;
    let report = ["=== Object Inspection: " + tostr(object) + " ==="];
    
    // Basic information
    report = {@report, "Class: " + meta.class};
    report = {@report, "Parent: " + meta.parent};
    
    if (detail_level in ["detailed", "full"])
        // Properties
        report = {@report, "", "Properties:"};
        for prop in (meta.properties)
            let info = meta.property(prop);
            report = {@report, "  " + prop + ": " + info.type + " = " + info.value};
        endfor
        
        // Verbs
        report = {@report, "", "Verbs:"};
        for verb in (meta.verbs)
            let info = meta.verb(verb);
            report = {@report, "  " + verb + info.signature};
        endfor
    endif
    
    if (detail_level == "full")
        // Events
        report = {@report, "", "Emits Events:"};
        for event in (meta.emits())
            report = {@report, "  " + event};
        endfor
        
        // Queries
        report = {@report, "", "Related Queries:"};
        for query in (meta.queries())
            report = {@report, "  " + query};
        endfor
    endif
    
    return join(report, "\n");
endfn

// ========== RUNTIME ANALYSIS ==========

// Performance profiling via reflection
fn profile_object(object, duration = 60s)
    let meta = object.$meta;
    let profile = start_profiling(object);
    
    // Monitor all verb calls
    for verb in (meta.verbs)
        meta.instrument_verb(verb, profile);
    endfor
    
    // Monitor property access
    for prop in (meta.properties)
        meta.instrument_property(prop, profile);
    endfor
    
    // Wait for profiling duration
    suspend(duration);
    
    return profile.get_report();
endfn

// ========== META-PROGRAMMING PATTERNS ==========

// Trait/Mixin system via MOP
fn add_trait(object, trait_name)
    let trait = $traits.(trait_name);
    let meta = object.$meta;
    
    // Add trait properties
    for {name, spec} in (trait.properties)
        if (!meta.has_property(name))
            meta.add_property(name, spec);
        endif
    endfor
    
    // Add trait verbs
    for {name, spec} in (trait.verbs)
        if (!meta.has_verb(name))
            meta.add_verb(name, spec);
        endif
    endfor
    
    // Register trait
    meta.add_trait(trait_name);
    emit TraitAdded(object, trait_name);
endfn

// Aspect-Oriented Programming via MOP
fn add_logging_aspect(object)
    let meta = object.$meta;
    
    // Wrap all verbs with logging
    for verb in (meta.verbs)
        meta.wrap_verb(verb, {
            before: fn(@args)
                log_entry(object, verb, args);
            endfn,
            
            after: fn(@args)
                log_exit(object, verb, args[$ - 1]);
            endfn,
            
            on_error: fn(error)
                log_error(object, verb, error);
            endfn
        });
    endfor
endfn

// ========== BOOTSTRAPPING ==========

// The MOP can even modify how the MOP works
$mop.$meta.wrap_verb("get_meta", {
    before: fn(object)
        // Check if reflection is allowed on this object
        if (!query can_reflect(caller, object))
            throw E_PERM, "Reflection not permitted on " + tostr(object);
        endif
    endfn
});

// ========== END REFLECTION/MOP EXAMPLES ==========