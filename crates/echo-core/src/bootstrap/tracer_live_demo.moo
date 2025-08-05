// Live demonstration of the Echo SystemTracer actually modifying database objects
// This shows real transformation of verbs and properties

// First, let's create a legacy object that needs transformation
verb $wiz:create_legacy_demo () none none none
  if (!$perm_utils:controls(player, $system))
    return player:tell("Permission denied.");
  endif
  
  player:tell("=== Creating Legacy Demo Objects ===");
  
  // Create a legacy user object
  let legacy_user = create($player, $system);
  legacy_user.name = "Legacy User";
  legacy_user.wizard_level = "WIZARD";  // Old MOO constant
  legacy_user.home = "ROOM";            // Another MOO constant
  legacy_user.builder_bit = "BUILDER";  // And another
  
  add_property(legacy_user, "permissions", "rxc");
  legacy_user.permissions = {"read" -> 1, "write" -> is_wizard(player), "admin" -> "HACKER"};
  
  player:tell(tostr("Created legacy user object: ", legacy_user, " (#", tonum(legacy_user), ")"));
  
  // Add a legacy verb
  add_verb(legacy_user, {legacy_user, "xd", "check_access"}, {"this", "none", "this"});
  set_verb_code(legacy_user, "check_access", {
    "// Legacy permission checking code",
    "if (is_wizard(caller))",
    "  return 1;",
    "elseif (caller.wizard_level == WIZARD)",
    "  return 1;",  
    "elseif (caller.builder_bit == BUILDER && this.owner == HACKER)",
    "  return 1;",
    "endif",
    "return 0;"
  });
  
  // Create a legacy room
  let legacy_room = create($room, $system);
  legacy_room.name = "Legacy Conference Room";
  legacy_room.owner_name = "HACKER";  // Storing MOO constant as string
  
  add_verb(legacy_room, {legacy_room, "xd", "announce"}, {"this", "none", "this"});
  set_verb_code(legacy_room, "announce", {
    "// Legacy announcement code",
    "let msg = args[1];",
    "msg = $string_utils:upper_case(msg);",
    "msg = $string_utils:trim_spaces(msg);",
    "for p in (this.contents)",
    "  if (valid(p))",
    "    notify(p, msg);",
    "  endif",
    "endfor",
    "if (valid(#-1))",
    "  notify(#-1, \"[Guest] \" + msg);",
    "endif"
  });
  
  player:tell(tostr("Created legacy room: ", legacy_room, " (#", tonum(legacy_room), ")"));
  
  // Show current state
  player:tell("");
  player:tell("Current legacy_user properties:");
  player:tell(tostr("  wizard_level: ", legacy_user.wizard_level));
  player:tell(tostr("  home: ", legacy_user.home));
  player:tell(tostr("  builder_bit: ", legacy_user.builder_bit));
  player:tell(tostr("  permissions: ", toliteral(legacy_user.permissions)));
  
  player:tell("");
  player:tell("Current legacy_user:check_access code:");
  for line in (verb_code(legacy_user, "check_access"))
    player:tell("  " + line);
  endfor
  
  player:tell("");
  player:tell("Current legacy_room:announce code:");
  for line in (verb_code(legacy_room, "announce"))
    player:tell("  " + line);
  endfor
  
  player:tell("");
  player:tell(tostr("Demo objects created. Use '#", tonum(legacy_user), 
                   "' and '#", tonum(legacy_room), "' for testing."));
  
  return {legacy_user, legacy_room};
endverb

// Now run the actual transformation
verb $wiz:demo_live_transformation () none none none
  if (!$perm_utils:controls(player, $system))
    return player:tell("Permission denied.");
  endif
  
  player:tell("=== Live SystemTracer Transformation Demo ===");
  player:tell("");
  
  // Find our demo objects
  let legacy_user = #0;  // Replace with actual object number
  let legacy_room = #0;  // Replace with actual object number
  
  player:tell("Enter the legacy user object number:");
  legacy_user = toobj(read());
  player:tell("Enter the legacy room object number:");
  legacy_room = toobj(read());
  
  if (!valid(legacy_user) || !valid(legacy_room))
    return player:tell("Invalid object numbers.");
  endif
  
  // Set up the tracer with all transformation rules
  $system_tracer.rules = {};
  $system_tracer:add_rule($property_syntax_fixer);
  $system_tracer:add_rule($capability_converter);
  $system_tracer:add_rule($old_to_new_string_rule);
  $system_tracer:add_rule($negative_ref_normalizer);
  $system_tracer:add_rule($builtin_function_resolver);
  $system_tracer:sort_rules_by_priority();
  
  // First do a dry run
  player:tell("Phase 1: Dry run to show what will change...");
  $system_tracer.dry_run = 1;
  $system_tracer.verbose = 2;
  
  player:tell("");
  player:tell("--- Analyzing legacy_user ---");
  let user_result = $system_tracer:transform_object(legacy_user);
  
  player:tell("");
  player:tell("--- Analyzing legacy_room ---");
  let room_result = $system_tracer:transform_object(legacy_room);
  
  player:tell("");
  player:tell("Continue with actual transformation? (yes/no)");
  if (read() != "yes")
    return player:tell("Transformation cancelled.");
  endif
  
  // Now do it for real
  player:tell("");
  player:tell("Phase 2: Applying transformations...");
  $system_tracer.dry_run = 0;
  $system_tracer.verbose = 1;
  
  // Transform the objects
  $system_tracer:transform_object(legacy_user);
  $system_tracer:transform_object(legacy_room);
  
  // Show the results
  player:tell("");
  player:tell("=== Transformation Complete ===");
  player:tell("");
  
  player:tell("Transformed legacy_user properties:");
  player:tell(tostr("  wizard_level: ", toliteral(legacy_user.wizard_level)));
  player:tell(tostr("  home: ", toliteral(legacy_user.home)));
  player:tell(tostr("  builder_bit: ", toliteral(legacy_user.builder_bit)));
  player:tell(tostr("  permissions: ", toliteral(legacy_user.permissions)));
  
  player:tell("");
  player:tell("Transformed legacy_user:check_access code:");
  for line in (verb_code(legacy_user, "check_access"))
    player:tell("  " + line);
  endfor
  
  player:tell("");
  player:tell("Transformed legacy_room:announce code:");
  for line in (verb_code(legacy_room, "announce"))
    player:tell("  " + line);
  endfor
  
  player:tell("");
  $system_tracer:show_stats();
endverb

// A more complex transformation rule that handles nested data structures
object $deep_property_transformer
  name: "Deep Property Transformer"
  parent: $transformation_rule
  owner: $system
  
  property name: "Deep Property Transformer"
  property description: "Transforms nested properties and data structures"
  property priority: 150
  
  verb matches_property (value, context) this none this
    // Match complex property structures
    if (typeof(value) == LIST)
      // Check if list contains transformable elements
      for item in (value)
        if (typeof(item) == STR && item in {"WIZARD", "HACKER", "BUILDER"})
          return 1;
        elseif (typeof(item) == LIST && this:matches_property(item, context))
          return 1;
        endif
      endfor
    endif
    return 0;
  endverb
  
  verb transform_property (value, context) this none this
    // Deep transformation of property values
    if (typeof(value) == LIST)
      let new_list = {};
      for item in (value)
        if (typeof(item) == STR && item in {"WIZARD", "HACKER", "BUILDER"})
          // Convert string constant to object reference
          new_list = {@new_list, #0};
        elseif (typeof(item) == LIST)
          // Recursively transform nested lists
          new_list = {@new_list, this:transform_property(item, context)};
        else
          new_list = {@new_list, item};
        endif
      endfor
      return new_list;
    endif
    return value;
  endverb
endobject

// Verb to show before/after comparison
verb $wiz:compare_transformation (obj) none none none
  if (!valid(obj))
    return player:tell("Invalid object.");
  endif
  
  player:tell(tostr("=== Transformation Analysis for ", obj.name, " (#", obj, ") ==="));
  
  // Save current state
  let saved_props = {};
  let saved_verbs = {};
  
  for prop in (properties(obj))
    if (property_info(obj, prop)[1] == obj)
      saved_props[prop] = obj.(prop);
    endif
  endfor
  
  for vname in (verbs(obj))
    if (verb_info(obj, vname)[1] == obj)
      saved_verbs[vname] = verb_code(obj, vname);
    endif
  endfor
  
  // Do a dry run transformation
  $system_tracer.dry_run = 1;
  $system_tracer.verbose = 0;
  let result = $system_tracer:transform_object(obj);
  
  if (!result["transformed"])
    return player:tell("No transformations would be applied to this object.");
  endif
  
  // Show what would change
  player:tell("");
  player:tell("PROPERTIES that would change:");
  if ("properties" in result["changes"])
    for prop in (keys(result["changes"]["properties"]))
      let change = result["changes"]["properties"][prop];
      player:tell(tostr("  ", prop, ":"));
      player:tell(tostr("    FROM: ", toliteral(change["old"])));
      player:tell(tostr("    TO:   ", toliteral(change["new"])));
    endfor
  else
    player:tell("  (no property changes)");
  endif
  
  player:tell("");
  player:tell("VERBS that would change:");
  for vname in (keys(result["changes"]))
    if (vname != "properties")
      player:tell(tostr("  ", vname, ":"));
      let old_code = result["changes"][vname]["old"];
      let new_code = result["changes"][vname]["new"];
      
      // Show a diff-like view
      for i in [1..length(old_code)]
        if (i <= length(new_code) && old_code[i] != new_code[i])
          player:tell(tostr("    - ", old_code[i]));
          player:tell(tostr("    + ", new_code[i]));
        elseif (i > length(new_code))
          player:tell(tostr("    - ", old_code[i]));
        endif
      endfor
      
      if (length(new_code) > length(old_code))
        for i in [(length(old_code) + 1)..length(new_code)]
          player:tell(tostr("    + ", new_code[i]));
        endfor
      endif
    endif
  endfor
endverb

// Batch transformation with progress tracking
verb $wiz:batch_transform (pattern) none none none
  if (!$perm_utils:controls(player, $system))
    return player:tell("Permission denied.");
  endif
  
  player:tell("=== Batch Transformation ===");
  player:tell(tostr("Looking for objects matching pattern: ", pattern));
  
  let targets = {};
  
  // Find matching objects
  for obj in (objects())
    if (valid(obj) && match(obj.name, pattern))
      targets = {@targets, obj};
    endif
  endfor
  
  player:tell(tostr("Found ", length(targets), " objects to transform."));
  
  if (!targets)
    return;
  endif
  
  player:tell("Preview first 5 objects:");
  for obj in (targets[1..min(5, length(targets))])
    player:tell(tostr("  ", obj.name, " (#", obj, ")"));
  endfor
  
  player:tell("");
  player:tell("Continue? (yes/no)");
  if (read() != "yes")
    return player:tell("Batch transformation cancelled.");
  endif
  
  // Set up progress tracking
  let total = length(targets);
  let processed = 0;
  let transformed = 0;
  let errors = {};
  
  $system_tracer.dry_run = 0;
  $system_tracer.verbose = 0;
  
  player:tell("");
  player:tell("Processing...");
  
  for obj in (targets)
    processed = processed + 1;
    
    try
      let result = $system_tracer:transform_object(obj);
      if (result["transformed"])
        transformed = transformed + 1;
      endif
    except e (ANY)
      errors = {@errors, {obj, e[2]}};
    endtry
    
    // Progress update every 10 objects
    if (processed % 10 == 0)
      player:tell(tostr("  Processed ", processed, "/", total, 
                       " (", transformed, " transformed)"));
    endif
  endfor
  
  // Final report
  player:tell("");
  player:tell("=== Batch Transformation Complete ===");
  player:tell(tostr("Total objects processed: ", processed));
  player:tell(tostr("Objects transformed: ", transformed));
  player:tell(tostr("Objects unchanged: ", processed - transformed - length(errors)));
  player:tell(tostr("Errors: ", length(errors)));
  
  if (errors)
    player:tell("");
    player:tell("Errors encountered:");
    for e in (errors[1..min(10, length(errors))])
      player:tell(tostr("  ", e[1].name, " (#", e[1], "): ", e[2]));
    endfor
  endif
  
  player:tell("");
  $system_tracer:show_stats();
endverb