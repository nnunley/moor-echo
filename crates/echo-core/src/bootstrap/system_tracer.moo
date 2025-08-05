// SystemTracer - Echo/MOO implementation for live code transformation
// Similar to Squeak Smalltalk's SystemTracer, but for Echo
//
// This allows rewriting the codebase using transformation rules
// defined in Echo itself, operating on the live database.

object $system_tracer
  name: "System Tracer"
  parent: $root
  owner: $system
  
  // Core properties
  property rules: {}           // List of transformation rule objects
  property statistics: {}      // Transformation statistics
  property dry_run: 0         // If true, don't actually apply changes
  property max_iterations: 10  // Max transformation passes
  property verbose: 0         // Debug output level
  
  // Public API verbs
  
  verb add_rule (rule) this none this
    // Add a transformation rule to the tracer
    if (!is_a(rule, $transformation_rule))
      raise(E_TYPE, "Rule must inherit from $transformation_rule");
    endif
    this.rules = {@this.rules, rule};
    this:log(1, tostr("Added rule: ", rule.name));
  endverb
  
  verb remove_rule (rule) this none this
    // Remove a transformation rule
    this.rules = setremove(this.rules, rule);
  endverb
  
  verb transform_system () this none this
    // Transform all objects in the system
    this:log(1, "Starting system transformation...");
    let results = {};
    let total_transformed = 0;
    
    // Get all objects in the system
    for obj in (children($root))
      let result = this:transform_object(obj);
      if (result["transformed"])
        total_transformed = total_transformed + 1;
        results[obj] = result;
      endif
    endfor
    
    this:log(1, tostr("Transformation complete. ", total_transformed, " objects transformed."));
    return results;
  endverb
  
  verb transform_object (obj) this none this
    // Transform a single object's verbs and properties
    if (!valid(obj))
      return {"transformed" -> 0, "error" -> "Invalid object"};
    endif
    
    this:log(2, tostr("Transforming object: ", obj.name, " (#", obj, ")"));
    
    let transformed = 0;
    let changes = {};
    
    // Transform properties first
    let prop_result = this:transform_properties(obj);
    if (prop_result["transformed"])
      transformed = 1;
      changes["properties"] = prop_result["changes"];
    endif
    
    // Transform each verb on the object
    for vname in (verbs(obj))
      let vinfo = verb_info(obj, vname);
      let vcode = verb_code(obj, vname);
      
      // Apply transformation rules to the verb code
      let new_code = this:transform_code(vcode, obj, vname);
      
      if (new_code != vcode)
        transformed = 1;
        changes[vname] = {"old" -> vcode, "new" -> new_code};
        
        if (!this.dry_run)
          // Actually update the verb code in the database
          try
            set_verb_code(obj, vname, new_code);
            this:log(2, tostr("  Transformed verb: ", vname));
          except e (ANY)
            this:log(1, tostr("  ERROR transforming verb ", vname, ": ", e[2]));
            this:update_stats(this.rules[1], "failed");
          endtry
        else
          this:log(2, tostr("  Would transform verb: ", vname, " (dry run)"));
        endif
      endif
    endfor
    
    return {"transformed" -> transformed, "changes" -> changes};
  endverb
  
  verb transform_properties (obj) this none this
    // Transform object properties
    let transformed = 0;
    let changes = {};
    
    // Get all properties on this object (not inherited)
    for prop in (properties(obj))
      if (property_info(obj, prop)[1] == obj)  // Property defined on this object
        let old_value = obj.(prop);
        let new_value = this:transform_property_value(old_value, obj, prop);
        
        if (new_value != old_value)
          transformed = 1;
          changes[prop] = {"old" -> old_value, "new" -> new_value};
          
          if (!this.dry_run)
            try
              obj.(prop) = new_value;
              this:log(3, tostr("  Transformed property: ", prop));
            except e (ANY)
              this:log(1, tostr("  ERROR transforming property ", prop, ": ", e[2]));
            endtry
          else
            this:log(3, tostr("  Would transform property: ", prop, " (dry run)"));
          endif
        endif
      endif
    endfor
    
    return {"transformed" -> transformed, "changes" -> changes};
  endverb
  
  verb transform_property_value (value, obj, prop) this none this
    // Transform a property value based on rules
    let context = {"object" -> obj, "property" -> prop};
    
    // Handle different value types
    if (typeof(value) == STR)
      // Apply string transformation rules
      for rule in (this.rules)
        if (rule:matches_property && rule:matches_property(value, context))
          value = rule:transform_property(value, context);
        endif
      endfor
    elseif (typeof(value) == LIST)
      // Recursively transform list elements
      let new_list = {};
      let changed = 0;
      for item in (value)
        let new_item = this:transform_property_value(item, obj, prop);
        new_list = {@new_list, new_item};
        if (new_item != item)
          changed = 1;
        endif
      endfor
      if (changed)
        value = new_list;
      endif
    elseif (typeof(value) == OBJ)
      // Transform object references
      for rule in (this.rules)
        if (rule:matches_object_ref && rule:matches_object_ref(value, context))
          value = rule:transform_object_ref(value, context);
        endif
      endfor
    endif
    
    return value;
  endverb
  
  verb transform_code (code, obj, vname) this none this
    // Transform verb code using registered rules
    let original_code = code;
    let iterations = 0;
    let changed = 1;
    
    // Keep applying rules until no more changes occur
    while (changed && iterations < this.max_iterations)
      changed = 0;
      iterations = iterations + 1;
      
      // Apply each rule in sequence
      for rule in (this.rules)
        let context = {"object" -> obj, "verb" -> vname, "iteration" -> iterations};
        
        if (rule:matches(code, context))
          let new_code = rule:transform(code, context);
          
          if (new_code != code)
            code = new_code;
            changed = 1;
            
            // Update statistics
            this:update_stats(rule, "applied");
            this:log(3, tostr("    Applied rule: ", rule.name));
          endif
        endif
      endfor
    endwhile
    
    if (iterations >= this.max_iterations)
      this:log(1, tostr("Warning: Max iterations reached for ", obj, ":", vname));
    endif
    
    return code;
  endverb
  
  verb transform_ast (ast) this none this
    // Transform an AST representation (for future use)
    // This would work with Echo's AST once we have proper metaprogramming
    return ast;  // Placeholder
  endverb
  
  // Utility verbs
  
  verb update_stats (rule, action) this none this
    // Update transformation statistics
    let rule_name = rule.name;
    if (!(rule_name in this.statistics))
      this.statistics[rule_name] = {"applied" -> 0, "failed" -> 0};
    endif
    
    if (action == "applied")
      this.statistics[rule_name]["applied"] = this.statistics[rule_name]["applied"] + 1;
    elseif (action == "failed")
      this.statistics[rule_name]["failed"] = this.statistics[rule_name]["failed"] + 1;
    endif
  endverb
  
  verb clear_stats () this none this
    // Clear all statistics
    this.statistics = {};
  endverb
  
  verb show_stats () this none this
    // Display transformation statistics
    player:tell("=== Transformation Statistics ===");
    for rule_name in (keys(this.statistics))
      let stats = this.statistics[rule_name];
      player:tell(tostr(rule_name, ": ", stats["applied"], " applied, ", 
                       stats["failed"], " failed"));
    endfor
  endverb
  
  verb log (level, message) this none this
    // Log messages based on verbosity level
    if (this.verbose >= level)
      player:tell(tostr("[SystemTracer] ", message));
    endif
  endverb
  
  // Rule management
  
  verb sort_rules_by_priority () this none this
    // Sort rules by their priority (higher priority first)
    this.rules = sort(this.rules, 
                     lambda (a, b) -> b:priority() - a:priority());
  endverb
  
  verb list_rules () this none this
    // List all registered transformation rules
    player:tell("=== Registered Transformation Rules ===");
    for rule in (this.rules)
      player:tell(tostr(rule.name, " (priority: ", rule:priority(), 
                       ") - ", rule.description));
    endfor
  endverb
endobject

// Base class for transformation rules
object $transformation_rule
  name: "Transformation Rule"
  parent: $root
  owner: $system
  
  property name: "Base Rule"
  property description: "Base transformation rule"
  property priority: 100  // Higher numbers run first
  
  verb matches (code, context) this none this
    // Override this: Return true if this rule applies to the code
    return 0;
  endverb
  
  verb transform (code, context) this none this
    // Override this: Transform the code and return the result
    return code;
  endverb
  
  verb priority () this none this
    // Return the priority of this rule
    return this.priority;
  endverb
endobject

// Example transformation rule: Fix MOO property syntax
object $property_syntax_fixer
  name: "Property Syntax Fixer"
  parent: $transformation_rule
  owner: $system
  
  property name: "Property Syntax Fixer"
  property description: "Converts MOO constants to property access"
  property priority: 200
  
  // MOO constants that should be converted to property access
  property moo_constants: {"HACKER", "ROOT", "PLAYER", "BUILDER", "PROG", 
                          "WIZ", "SYSOBJ", "ARCH_WIZARD", "ROOM", "THING"}
  
  verb matches (code, context) this none this
    // Check if the code contains any MOO constants
    for constant in (this.moo_constants)
      if (index(code, constant))
        return 1;
      endif
    endfor
    return 0;
  endverb
  
  verb transform (code, context) this none this
    // Replace MOO constants with property access on #0
    let new_code = code;
    
    for constant in (this.moo_constants)
      // Use string substitution to replace the constant
      // In real implementation, we'd use proper AST transformation
      new_code = substitute(new_code, constant, tostr("#0.", constant));
    endfor
    
    return new_code;
  endverb
  
  verb matches_property (value, context) this none this
    // Check if property value contains MOO constants
    if (typeof(value) != STR)
      return 0;
    endif
    
    for constant in (this.moo_constants)
      if (value == constant)
        return 1;
      endif
    endfor
    return 0;
  endverb
  
  verb transform_property (value, context) this none this
    // Transform property values that are MOO constants
    if (value in this.moo_constants)
      // Instead of storing "HACKER", store #0
      return #0;
    endif
    return value;
  endverb
  
  verb matches_object_ref (value, context) this none this
    // Check if this is a reference that needs transformation
    return 0;  // This rule doesn't transform object refs
  endverb
endobject

// Example rule: Convert negative object references
object $negative_ref_normalizer
  name: "Negative Reference Normalizer"
  parent: $transformation_rule
  owner: $system
  
  property name: "Negative Reference Normalizer"
  property description: "Handles negative object references for connections"
  property priority: 150
  
  verb matches (code, context) this none this
    // Check for negative object references like #-1, #-2, etc.
    return match(code, "#-[0-9]+") != {};
  endverb
  
  verb transform (code, context) this none this
    // Wrap negative references in connection_object() calls
    // This is a simplified implementation
    let new_code = code;
    
    // Find all negative references
    let pattern = "#-[0-9]+";
    let matches = match_all(code, pattern);
    
    for m in (matches)
      let neg_ref = m[0];
      let replacement = tostr("connection_object(", neg_ref, ")");
      new_code = substitute(new_code, neg_ref, replacement);
    endfor
    
    return new_code;
  endverb
endobject

// Example rule: MOO builtin function resolver
object $builtin_function_resolver
  name: "Builtin Function Resolver"
  parent: $transformation_rule
  owner: $system
  
  property name: "Builtin Function Resolver"
  property description: "Converts MOO builtins to method calls"
  property priority: 100
  
  property builtins: {"valid", "typeof", "length", "tostr", "toint", 
                     "tofloat", "match", "substitute", "index"}
  
  verb matches (code, context) this none this
    // Check if code contains builtin function calls
    for builtin in (this.builtins)
      if (match(code, tostr(builtin, "\\s*\\(")))
        return 1;
      endif
    endfor
    return 0;
  endverb
  
  verb transform (code, context) this none this
    // Convert builtin() to $builtins:builtin()
    let new_code = code;
    
    for builtin in (this.builtins)
      let pattern = tostr(builtin, "\\s*\\(");
      let replacement = tostr("$builtins:", builtin, "(");
      new_code = substitute_pattern(new_code, pattern, replacement);
    endfor
    
    return new_code;
  endverb
endobject

// Bootstrap verb to set up the system tracer
verb $system:setup_system_tracer () this none this
  // Create the basic transformation rules
  $system_tracer:add_rule($property_syntax_fixer);
  $system_tracer:add_rule($negative_ref_normalizer);  
  $system_tracer:add_rule($builtin_function_resolver);
  
  $system_tracer:sort_rules_by_priority();
  
  player:tell("System tracer initialized with default rules.");
  $system_tracer:list_rules();
endverb