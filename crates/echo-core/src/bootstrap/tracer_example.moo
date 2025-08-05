// Simple example of using the Echo SystemTracer
// This shows a practical migration scenario

// Scenario: We want to update all uses of the old $string_utils
// to use the new $str object with different method names

object $old_to_new_string_rule
  name: "String Utils Modernizer"
  parent: $transformation_rule
  owner: $system
  
  property name: "String Utils Modernizer"
  property description: "Updates old $string_utils calls to new $str methods"
  property priority: 500  // Run this early
  
  // Mapping of old methods to new methods
  property method_map: {
    {"contains_string", "contains"},
    {"string_length", "length"},
    {"string_match", "match"},
    {"left_str", "left"},
    {"right_str", "right"},
    {"upper_case", "upper"},
    {"lower_case", "lower"},
    {"trim_spaces", "trim"}
  }
  
  verb matches (code, context) this none this
    // Check if code references $string_utils
    return index(code, "$string_utils:");
  endverb
  
  verb transform (code, context) this none this
    let new_code = code;
    
    // First, replace the object reference
    new_code = substitute(new_code, "$string_utils:", "$str:");
    
    // Then update method names
    for mapping in (this.method_map)
      let old_method = mapping[1];
      let new_method = mapping[2];
      
      // Replace $str:old_method( with $str:new_method(
      let pattern = tostr("\\$str:", old_method, "\\(");
      let replacement = tostr("$str:", new_method, "(");
      new_code = substitute_pattern(new_code, pattern, replacement);
    endfor
    
    // Log what we changed
    if (new_code != code)
      $system_tracer:log(3, tostr("    Updated string utils in ", 
                                  context["object"], ":", context["verb"]));
    endif
    
    return new_code;
  endverb
endobject

// Example: A verb that needs updating
verb $sample_object:process_text (text) this none this
  // Old code using $string_utils
  if ($string_utils:contains_string(text, "hello"))
    text = $string_utils:upper_case(text);
    text = $string_utils:trim_spaces(text);
    let len = $string_utils:string_length(text);
    player:tell(tostr("Processed text (", len, " chars): ", text));
  endif
  
  return text;
endverb

// The migration script
verb $wiz:migrate_string_utils () none none none
  if (!$perm_utils:controls(player, $system))
    return player:tell("Permission denied.");
  endif
  
  player:tell("=== String Utils Migration ===");
  player:tell("This will update all code from $string_utils to $str");
  player:tell("");
  
  // Set up the tracer
  $system_tracer:clear_stats();
  $system_tracer:add_rule($old_to_new_string_rule);
  $system_tracer.verbose = 1;
  
  // First, do a dry run to see what would change
  player:tell("Phase 1: Dry run analysis...");
  $system_tracer.dry_run = 1;
  
  let affected = {};
  for obj in (objects())
    if (valid(obj))
      let result = $system_tracer:transform_object(obj);
      if (result["transformed"])
        affected = {@affected, obj};
      endif
    endif
  endfor
  
  player:tell(tostr("Found ", length(affected), " objects that would be updated:"));
  for obj in (affected[1..min(10, length(affected))])
    player:tell(tostr("  ", obj.name, " (#", obj, ")"));
  endfor
  if (length(affected) > 10)
    player:tell(tostr("  ... and ", length(affected) - 10, " more"));
  endif
  
  player:tell("");
  player:tell("Type 'yes' to proceed with the actual migration:");
  
  let answer = read();
  if (answer != "yes")
    return player:tell("Migration cancelled.");
  endif
  
  // Now do it for real
  player:tell("");
  player:tell("Phase 2: Applying transformations...");
  $system_tracer.dry_run = 0;
  
  let transformed = 0;
  for obj in (affected)
    let result = $system_tracer:transform_object(obj);
    if (result["transformed"])
      transformed = transformed + 1;
      if (transformed % 10 == 0)
        player:tell(tostr("  Processed ", transformed, " objects..."));
      endif
    endif
  endfor
  
  player:tell("");
  player:tell(tostr("Migration complete! Transformed ", transformed, " objects."));
  $system_tracer:show_stats();
endverb

// After transformation, the verb would look like:
// verb $sample_object:process_text (text) this none this
//   if ($str:contains(text, "hello"))
//     text = $str:upper(text);
//     text = $str:trim(text);
//     let len = $str:length(text);
//     player:tell(tostr("Processed text (", len, " chars): ", text));
//   endif
//   
//   return text;
// endverb

// Another example: Database schema evolution
object $schema_migrator
  name: "Schema Migrator"
  parent: $transformation_rule
  owner: $system
  
  property name: "Schema Migrator"
  property description: "Migrates old property access patterns"
  property priority: 600
  
  verb matches (code, context) this none this
    // Look for direct property access that should use accessors
    return match(code, "\\.location[^(]") ||
           match(code, "\\.contents[^(]") ||
           match(code, "\\.owner[^(]");
  endverb
  
  verb transform (code, context) this none this
    let new_code = code;
    
    // Convert direct property access to method calls
    // player.location -> player:location()
    new_code = substitute_pattern(new_code, 
                                 "([a-zA-Z_][a-zA-Z0-9_]*)\\.location([^(])",
                                 "$1:location()$2");
    
    // obj.contents -> obj:contents()
    new_code = substitute_pattern(new_code,
                                 "([a-zA-Z_][a-zA-Z0-9_]*)\\.contents([^(])",
                                 "$1:contents()$2");
    
    // obj.owner = player -> obj:set_owner(player)
    new_code = substitute_pattern(new_code,
                                 "([a-zA-Z_][a-zA-Z0-9_]*)\\.owner\\s*=\\s*([^;]+)",
                                 "$1:set_owner($2)");
    
    return new_code;
  endverb
endobject

// Demonstration of rule composition
verb $wiz:demo_tracer () none none none
  player:tell("=== SystemTracer Demo ===");
  
  // Create a test object with some legacy code
  let test_obj = create($root);
  test_obj.name = "Tracer Test Object";
  add_verb(test_obj, {player, "x", "test", {"this", "none", "this"}});
  set_verb_code(test_obj, "test", {
    "// Legacy code with multiple issues",
    "if (is_wizard(player))",
    "  let msg = $string_utils:upper_case(\"hello\");",
    "  msg = $string_utils:trim_spaces(msg);", 
    "  if (valid(#-1))",
    "    notify(#-1, msg);",
    "  endif",
    "  return player.location;",
    "endif"
  });
  
  player:tell("Created test object with legacy code.");
  player:tell("Original code:");
  for line in (verb_code(test_obj, "test"))
    player:tell("  " + line);
  endfor
  
  // Set up multiple transformation rules
  $system_tracer:clear_stats();
  $system_tracer.rules = {};
  $system_tracer:add_rule($capability_converter);
  $system_tracer:add_rule($old_to_new_string_rule);
  $system_tracer:add_rule($negative_ref_normalizer);
  $system_tracer:add_rule($builtin_function_resolver);
  $system_tracer:add_rule($schema_migrator);
  $system_tracer:sort_rules_by_priority();
  
  // Transform it
  $system_tracer.dry_run = 0;
  $system_tracer.verbose = 2;
  
  player:tell("");
  player:tell("Applying transformations...");
  let result = $system_tracer:transform_object(test_obj);
  
  player:tell("");
  player:tell("Transformed code:");
  for line in (verb_code(test_obj, "test"))
    player:tell("  " + line);
  endfor
  
  player:tell("");
  $system_tracer:show_stats();
  
  // Clean up
  recycle(test_obj);
endverb