// Audit trail and safety features for the Echo SystemTracer
// This ensures we can track and potentially undo transformations

object $tracer_audit
  name: "Tracer Audit System"
  parent: $root
  owner: $system
  
  property audit_log: {}        // List of all transformations
  property undo_stack: {}       // Stack for undoing changes
  property max_log_size: 1000   // Maximum audit entries to keep
  
  verb log_transformation (obj, verb_or_prop, old_value, new_value) this none this
    // Record a transformation for audit purposes
    let entry = {
      "timestamp" -> time(),
      "object" -> obj,
      "type" -> (verb_or_prop in verbs(obj)) ? "verb" : "property",
      "name" -> verb_or_prop,
      "old_value" -> old_value,
      "new_value" -> new_value,
      "player" -> player
    };
    
    this.audit_log = {@this.audit_log, entry};
    
    // Trim log if too large
    if (length(this.audit_log) > this.max_log_size)
      this.audit_log = this.audit_log[(length(this.audit_log) - this.max_log_size + 1)..$];
    endif
    
    // Add to undo stack
    this.undo_stack = {@this.undo_stack, entry};
  endverb
  
  verb undo_last (count) this none this
    // Undo the last N transformations
    if (!$perm_utils:controls(player, $system))
      return E_PERM;
    endif
    
    count = count || 1;
    let undone = 0;
    
    while (count > 0 && this.undo_stack)
      let entry = this.undo_stack[$];
      this.undo_stack = this.undo_stack[1..$-1];
      
      try
        if (entry["type"] == "verb")
          set_verb_code(entry["object"], entry["name"], entry["old_value"]);
          player:tell(tostr("Reverted verb ", entry["object"], ":", entry["name"]));
        else
          entry["object"].(entry["name"]) = entry["old_value"];
          player:tell(tostr("Reverted property ", entry["object"], ".", entry["name"]));
        endif
        undone = undone + 1;
      except e (ANY)
        player:tell(tostr("Failed to undo: ", e[2]));
      endtry
      
      count = count - 1;
    endwhile
    
    return undone;
  endverb
  
  verb show_recent (count) this none this
    // Show recent transformations
    count = count || 10;
    let start = max(1, length(this.audit_log) - count + 1);
    
    player:tell("=== Recent Transformations ===");
    for entry in (this.audit_log[start..$])
      player:tell(tostr(ctime(entry["timestamp"]), " - ", 
                       entry["player"].name, " transformed ",
                       entry["object"].name, " (#", entry["object"], ") ",
                       entry["type"], " '", entry["name"], "'"));
    endfor
  endverb
  
  verb create_backup (obj) this none this
    // Create a complete backup of an object before transformation
    let backup = {
      "object" -> obj,
      "timestamp" -> time(),
      "name" -> obj.name,
      "properties" -> {},
      "verbs" -> {}
    };
    
    // Backup all properties
    for prop in (properties(obj))
      if (property_info(obj, prop)[1] == obj)
        backup["properties"][prop] = {
          "value" -> obj.(prop),
          "info" -> property_info(obj, prop)
        };
      endif
    endfor
    
    // Backup all verbs
    for vname in (verbs(obj))
      if (verb_info(obj, vname)[1] == obj)
        backup["verbs"][vname] = {
          "code" -> verb_code(obj, vname),
          "info" -> verb_info(obj, vname)
        };
      endif
    endfor
    
    return backup;
  endverb
  
  verb restore_backup (backup) this none this
    // Restore an object from backup
    if (!$perm_utils:controls(player, $system))
      return E_PERM;
    endif
    
    let obj = backup["object"];
    if (!valid(obj))
      return player:tell("Object no longer exists!");
    endif
    
    let restored_props = 0;
    let restored_verbs = 0;
    
    // Restore properties
    for prop in (keys(backup["properties"]))
      try
        obj.(prop) = backup["properties"][prop]["value"];
        restored_props = restored_props + 1;
      except e (ANY)
        player:tell(tostr("Failed to restore property ", prop, ": ", e[2]));
      endtry
    endfor
    
    // Restore verbs
    for vname in (keys(backup["verbs"]))
      try
        set_verb_code(obj, vname, backup["verbs"][vname]["code"]);
        restored_verbs = restored_verbs + 1;
      except e (ANY)
        player:tell(tostr("Failed to restore verb ", vname, ": ", e[2]));
      endtry
    endfor
    
    player:tell(tostr("Restored ", restored_props, " properties and ", 
                     restored_verbs, " verbs to ", obj.name));
  endverb
endobject

// Enhanced SystemTracer with audit integration
// Add these verbs to $system_tracer

verb $system_tracer:transform_object_with_audit (obj) this none this
  // Transform with full audit trail
  if (!valid(obj))
    return {"transformed" -> 0, "error" -> "Invalid object"};
  endif
  
  // Create backup first
  let backup = $tracer_audit:create_backup(obj);
  
  // Store original transform_object method
  let original_method = this.transform_object;
  
  // Temporarily override to capture changes
  this.audit_mode = 1;
  this.audit_backup = backup;
  
  // Do the transformation
  let result = this:transform_object(obj);
  
  this.audit_mode = 0;
  
  // If successful, record the backup
  if (result["transformed"] && !this.dry_run)
    this.last_backup = backup;
    player:tell(tostr("Backup created. Use $system_tracer:rollback(", 
                     obj, ") to undo."));
  endif
  
  return result;
endverb

verb $system_tracer:rollback (obj) this none this
  // Rollback transformations on an object
  if (!$perm_utils:controls(player, $system))
    return E_PERM;
  endif
  
  // Find the most recent backup for this object
  if (this.last_backup && this.last_backup["object"] == obj)
    $tracer_audit:restore_backup(this.last_backup);
    return player:tell("Rollback complete.");
  endif
  
  player:tell("No backup found for this object.");
endverb

// Safe transformation wrapper
verb $system_tracer:safe_transform (obj) this none this
  // Transform with automatic backup and verification
  this:log(1, tostr("Safe transformation of ", obj.name, " (#", obj, ")"));
  
  // Phase 1: Analysis
  this.dry_run = 1;
  let analysis = this:transform_object(obj);
  
  if (!analysis["transformed"])
    return player:tell("No transformations needed.");
  endif
  
  // Show what will change
  player:tell("The following changes will be made:");
  this:show_changes(analysis["changes"]);
  
  player:tell("");
  player:tell("Proceed? (yes/no)");
  if (read() != "yes")
    return player:tell("Transformation cancelled.");
  endif
  
  // Phase 2: Backup
  let backup = $tracer_audit:create_backup(obj);
  
  // Phase 3: Transform
  this.dry_run = 0;
  let result = this:transform_object(obj);
  
  // Phase 4: Verify
  let verification = this:verify_transformation(obj, backup, result);
  
  if (!verification["success"])
    player:tell("Verification failed! Rolling back...");
    $tracer_audit:restore_backup(backup);
    return player:tell("Transformation rolled back due to verification failure.");
  endif
  
  player:tell("Transformation completed successfully.");
  return result;
endverb

verb $system_tracer:verify_transformation (obj, backup, result) this none this
  // Verify that transformation was successful
  let success = 1;
  let issues = {};
  
  // Check that object still exists and is valid
  if (!valid(obj))
    return {"success" -> 0, "issues" -> {"Object no longer valid"}};
  endif
  
  // Verify each transformed verb still compiles
  for vname in (keys(result["changes"]))
    if (vname != "properties")
      try
        let test_compile = verb_code(obj, vname);
        if (!test_compile)
          issues = {@issues, tostr("Verb ", vname, " is empty after transformation")};
          success = 0;
        endif
      except e (ANY)
        issues = {@issues, tostr("Verb ", vname, " failed: ", e[2])};
        success = 0;
      endtry
    endif
  endfor
  
  // Verify properties are of expected types
  if ("properties" in result["changes"])
    for prop in (keys(result["changes"]["properties"]))
      let new_value = obj.(prop);
      let old_type = typeof(backup["properties"][prop]["value"]);
      let new_type = typeof(new_value);
      
      // Type changes might be intentional (e.g., string -> object)
      // but flag them for review
      if (old_type != new_type && 
          !(old_type == STR && new_type == OBJ))  // Allow STR->OBJ for constants
        issues = {@issues, tostr("Property ", prop, " type changed from ",
                                old_type, " to ", new_type)};
      endif
    endfor
  endif
  
  return {"success" -> success, "issues" -> issues};
endverb

verb $system_tracer:show_changes (changes) this none this
  // Display pending changes in a readable format
  if ("properties" in changes)
    player:tell("  Properties:");
    for prop in (keys(changes["properties"]))
      let change = changes["properties"][prop];
      player:tell(tostr("    ", prop, ": ", 
                       toliteral(change["old"]), " => ",
                       toliteral(change["new"])));
    endfor
  endif
  
  for vname in (keys(changes))
    if (vname != "properties")
      player:tell(tostr("  Verb ", vname, ":"));
      player:tell("    (code will be transformed)");
    endif
  endfor
endverb

// Example of using safe transformation
verb $prog:safely_modernize_my_code () none none none
  if (!$perm_utils:controls(player, this))
    return E_PERM;
  endif
  
  player:tell("=== Safe Code Modernization ===");
  player:tell("This will safely transform your objects with automatic backup.");
  
  // Count objects
  let my_objects = {};
  for obj in (objects())
    if (valid(obj) && obj.owner == player)
      my_objects = {@my_objects, obj};
    endif
  endfor
  
  player:tell(tostr("Found ", length(my_objects), " objects owned by you."));
  player:tell("Proceed with safe transformation? (yes/no)");
  
  if (read() != "yes")
    return player:tell("Modernization cancelled.");
  endif
  
  // Transform each object safely
  let transformed = 0;
  let failed = 0;
  
  for obj in (my_objects)
    player:tell(tostr("Processing ", obj.name, " (#", obj, ")..."));
    
    try
      let result = $system_tracer:safe_transform(obj);
      if (result["transformed"])
        transformed = transformed + 1;
      endif
    except e (ANY)
      player:tell(tostr("  ERROR: ", e[2]));
      failed = failed + 1;
    endtry
  endfor
  
  player:tell("");
  player:tell(tostr("Modernization complete: ", transformed, " transformed, ",
                   failed, " failed."));
endverb