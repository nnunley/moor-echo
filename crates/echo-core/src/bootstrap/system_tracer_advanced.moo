// Advanced SystemTracer examples and utilities
// These demonstrate more sophisticated transformations

// Rule for converting old-style verb permissions to capability-based security
object $capability_converter
  name: "Capability Converter"
  parent: $transformation_rule
  owner: $system
  
  property name: "Capability Converter"
  property description: "Converts permission flags to capability requirements"
  property priority: 300
  
  verb matches (code, context) this none this
    // Look for old-style permission checks
    return match(code, "caller_perms\\(\\)") || 
           match(code, "is_wizard\\(") ||
           match(code, "\\$perm_utils:");
  endverb
  
  verb transform (code, context) this none this
    let new_code = code;
    
    // Convert is_wizard(player) to player:has_capability("wizard")
    new_code = substitute_pattern(new_code, 
                                 "is_wizard\\(([^)]+)\\)",
                                 "$1:has_capability(\"wizard\")");
    
    // Convert caller_perms() checks to capability checks
    new_code = substitute_pattern(new_code,
                                 "caller_perms\\(\\)\\s*>=\\s*([0-9]+)",
                                 "caller:has_capability(perm_to_cap($1))");
    
    return new_code;
  endverb
endobject

// Rule for modernizing list operations
object $list_modernizer
  name: "List Modernizer"
  parent: $transformation_rule
  owner: $system
  
  property name: "List Modernizer"
  property description: "Converts old list operations to modern syntax"
  property priority: 250
  
  verb matches (code, context) this none this
    // Look for old-style list operations
    return match(code, "listappend\\(") ||
           match(code, "listinsert\\(") ||
           match(code, "listdelete\\(");
  endverb
  
  verb transform (code, context) this none this
    let new_code = code;
    
    // Convert listappend(list, item) to list = {@list, item}
    new_code = substitute_pattern(new_code,
                                 "([a-zA-Z_][a-zA-Z0-9_]*)\\s*=\\s*listappend\\(\\1,\\s*([^)]+)\\)",
                                 "$1 = {@$1, $2}");
    
    // Convert listinsert(list, item, pos) to modern syntax
    new_code = substitute_pattern(new_code,
                                 "listinsert\\(([^,]+),\\s*([^,]+),\\s*([^)]+)\\)",
                                 "list_insert($1, $2, $3)");
    
    return new_code;
  endverb
endobject

// Rule for async/await pattern transformation
object $async_transformer
  name: "Async Transformer"
  parent: $transformation_rule
  owner: $system
  
  property name: "Async Transformer"
  property description: "Converts fork/endfork to async/await patterns"
  property priority: 400
  
  verb matches (code, context) this none this
    // Look for fork/endfork blocks
    return match(code, "fork\\s*\\(") && match(code, "endfork");
  endverb
  
  verb transform (code, context) this none this
    // This is a complex transformation that would need proper AST support
    // For now, we mark it for manual review
    if (match(code, "fork\\s*\\("))
      // Add a comment marking this for async conversion
      let new_code = tostr("// TODO: Convert to async/await pattern\n", code);
      return new_code;
    endif
    return code;
  endverb
endobject

// Advanced tracer that can analyze and transform entire subsystems
object $subsystem_tracer
  name: "Subsystem Tracer"
  parent: $system_tracer
  owner: $system
  
  property name: "Subsystem Tracer"
  property description: "Traces and transforms entire subsystems"
  
  verb transform_subsystem (root_obj, options) this none this
    // Transform an entire subsystem rooted at root_obj
    this:log(1, tostr("Transforming subsystem rooted at ", root_obj.name));
    
    let objects = this:collect_subsystem_objects(root_obj);
    let results = {};
    
    // Analyze dependencies
    let deps = this:analyze_dependencies(objects);
    
    // Transform in dependency order
    for obj in (this:topological_sort(objects, deps))
      results[obj] = this:transform_object(obj);
    endfor
    
    return results;
  endverb
  
  verb collect_subsystem_objects (root) this none this
    // Collect all objects in a subsystem
    let objects = {root};
    let queue = {root};
    
    while (queue)
      let obj = queue[1];
      queue = queue[2..$];
      
      // Add children
      for child in (children(obj))
        if (!(child in objects))
          objects = {@objects, child};
          queue = {@queue, child};
        endif
      endfor
      
      // Add objects referenced in properties
      for prop in (properties(obj))
        let value = obj.(prop);
        if (typeof(value) == OBJ && valid(value) && !(value in objects))
          objects = {@objects, value};
          queue = {@queue, value};
        endif
      endfor
    endwhile
    
    return objects;
  endverb
  
  verb analyze_dependencies (objects) this none this
    // Analyze dependencies between objects
    let deps = {};
    
    for obj in (objects)
      deps[obj] = {};
      
      // Check verb code for references
      for vname in (verbs(obj))
        let code = verb_code(obj, vname);
        for other in (objects)
          if (other != obj && index(code, tostr("#", other)))
            deps[obj] = {@deps[obj], other};
          endif
        endfor
      endfor
    endfor
    
    return deps;
  endverb
  
  verb topological_sort (objects, deps) this none this
    // Sort objects in dependency order
    // Simplified topological sort
    let sorted = {};
    let remaining = {@objects};
    
    while (remaining)
      let found = 0;
      
      for obj in (remaining)
        let can_add = 1;
        
        // Check if all dependencies are already in sorted
        for dep in (deps[obj] || {})
          if (dep in remaining)
            can_add = 0;
            break;
          endif
        endfor
        
        if (can_add)
          sorted = {@sorted, obj};
          remaining = setremove(remaining, obj);
          found = 1;
        endif
      endfor
      
      if (!found && remaining)
        // Circular dependency - just add remaining
        sorted = {@sorted, @remaining};
        remaining = {};
      endif
    endwhile
    
    return sorted;
  endverb
endobject

// Pattern-based transformation engine
object $pattern_engine
  name: "Pattern Engine"
  parent: $root
  owner: $system
  
  property patterns: {}  // Pattern -> replacement mapping
  
  verb add_pattern (pattern, replacement, description) this none this
    this.patterns = {@this.patterns, {pattern, replacement, description}};
  endverb
  
  verb apply_patterns (code) this none this
    let new_code = code;
    
    for p in (this.patterns)
      let pattern = p[1];
      let replacement = p[2];
      new_code = substitute_pattern(new_code, pattern, replacement);
    endfor
    
    return new_code;
  endverb
  
  verb show_patterns () this none this
    player:tell("=== Registered Patterns ===");
    for p in (this.patterns)
      player:tell(tostr(p[3], ": ", p[1], " -> ", p[2]));
    endfor
  endverb
endobject

// Example usage verb
verb $prog:trace_my_code () none none none
  // Example of using the system tracer on your own code
  if (!$perm_utils:controls(player, this))
    return E_PERM;
  endif
  
  player:tell("Analyzing your objects for potential transformations...");
  
  // Set up tracer
  $system_tracer.dry_run = 1;  // Don't actually change anything
  $system_tracer.verbose = 2;  // Show what would be done
  
  // Transform all objects owned by the player
  let count = 0;
  for obj in (player.owned_objects)
    let result = $system_tracer:transform_object(obj);
    if (result["transformed"])
      count = count + 1;
      player:tell(tostr("Would transform ", obj.name, " (#", obj, ")"));
      
      for vname in (keys(result["changes"]))
        player:tell(tostr("  Verb ", vname, " would be updated"));
      endfor
    endif
  endfor
  
  player:tell(tostr("Analysis complete. ", count, " objects would be transformed."));
  $system_tracer:show_stats();
endverb

// Metaprogramming utilities
object $meta_utils
  name: "Metaprogramming Utilities"
  parent: $root
  owner: $system
  
  verb parse_verb_code (code) this none this
    // Parse verb code into a simple AST representation
    // This is a placeholder - real implementation would use Echo's parser
    return {"type" -> "code", "lines" -> code};
  endverb
  
  verb generate_code (ast) this none this
    // Generate code from AST
    // Placeholder for real code generation
    if (ast["type"] == "code")
      return ast["lines"];
    endif
    return "";
  endverb
  
  verb transform_ast (ast, transformer) this none this
    // Apply a transformation function to an AST
    return transformer:transform(ast);
  endverb
  
  verb analyze_code_metrics (code) this none this
    // Analyze code complexity and metrics
    let metrics = {};
    
    metrics["lines"] = length(code);
    metrics["conditionals"] = length(match_all(code, "if\\s*\\("));
    metrics["loops"] = length(match_all(code, "(for|while)\\s*\\("));
    metrics["calls"] = length(match_all(code, "[a-zA-Z_][a-zA-Z0-9_]*\\s*\\("));
    
    // Cyclomatic complexity estimate
    metrics["complexity"] = 1 + metrics["conditionals"] + metrics["loops"];
    
    return metrics;
  endverb
endobject