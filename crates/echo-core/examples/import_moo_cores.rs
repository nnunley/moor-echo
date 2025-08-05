use std::fs;
use std::path::Path;
use std::collections::HashMap;

use echo_core::storage::object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};
use echo_core::evaluator::meta_object::MetaObject;

/// Simple MOO database reader that focuses on object structure
fn main() -> anyhow::Result<()> {
    println!("=== MOO Core Database Import Examples ===\n");
    
    // Available databases
    let databases = vec![
        ("Minimal.db", "Minimal MOO database"),
        ("LambdaCore-latest.db", "Official LambdaCore database"),
        ("toastcore.db", "ToastCore (updated LambdaCore)"),
        ("JHCore-DEV-2.db", "JaysHouseCore (JHCore) database"),
    ];
    
    for (filename, description) in databases {
        let path = Path::new("examples").join(filename);
        if path.exists() {
            println!("\n=== Importing {} ===", description);
            println!("File: {:?}", path);
            
            match import_moo_database(&path) {
                Ok(summary) => println!("{}", summary),
                Err(e) => println!("Error importing {}: {}", filename, e),
            }
        } else {
            println!("\n{} not found at {:?}", description, path);
        }
    }
    
    Ok(())
}

fn import_moo_database(path: &Path) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    
    // Parse header
    let header = lines.next().ok_or_else(|| anyhow::anyhow!("Empty file"))?;
    let version = if header.contains("Format Version 4") {
        4
    } else if header.contains("Format Version 1") {
        1
    } else {
        return Err(anyhow::anyhow!("Unknown database format: {}", header));
    };
    
    // Parse counts
    let total_objects: i64 = lines.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing object count"))?;
    
    let total_verbs: i64 = lines.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing verb count"))?;
    
    let _dummy: i64 = lines.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing dummy value"))?;
    
    let total_players: i64 = lines.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing player count"))?;
    
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let mut store = ObjectStore::new(temp_dir.path())?;
    
    // Parse player list
    let mut players = Vec::new();
    for _ in 0..total_players {
        if let Some(line) = lines.next() {
            if let Ok(player_id) = line.parse::<i64>() {
                players.push(player_id);
            }
        }
    }
    
    // Count objects (quick scan)
    let mut object_count = 0;
    let mut recycled_count = 0;
    let mut important_objects = Vec::new();
    
    // Re-read file for object parsing
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 5 + total_players as usize; // Skip header and player list
    
    while i < lines.len() {
        if lines[i].starts_with('#') {
            let obj_line = &lines[i];
            if let Some(obj_num) = obj_line.strip_prefix('#').and_then(|s| s.parse::<i64>().ok()) {
                i += 1;
                
                if i < lines.len() {
                    if lines[i].trim() == "recycled" {
                        recycled_count += 1;
                        i += 1;
                    } else {
                        object_count += 1;
                        
                        // Get object name
                        if i < lines.len() {
                            let name = lines[i].to_string();
                            
                            // Track important objects
                            if obj_num < 10 || name.contains("System") || name.contains("Root") 
                                || name.contains("Wizard") || name.contains("Generic") {
                                important_objects.push((obj_num, name.clone()));
                                
                                // Create Echo object for important objects
                                let echo_id = store.get_or_create_moo_id(obj_num);
                                let echo_obj = EchoObject {
                                    id: echo_id,
                                    parent: None,
                                    name: name.clone(),
                                    properties: HashMap::new(),
                                    property_capabilities: HashMap::new(),
                                    verbs: HashMap::new(),
                                    queries: HashMap::new(),
                                    meta: MetaObject::new(echo_id),
                                };
                                store.store(echo_obj)?;
                            }
                        }
                        
                        // Skip to next object
                        while i < lines.len() && !lines[i].starts_with('#') {
                            i += 1;
                        }
                    }
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    // Build summary
    let mut summary = String::new();
    summary.push_str(&format!("  Database format version: {}\n", version));
    summary.push_str(&format!("  Total objects declared: {}\n", total_objects));
    summary.push_str(&format!("  Active objects found: {}\n", object_count));
    summary.push_str(&format!("  Recycled objects: {}\n", recycled_count));
    summary.push_str(&format!("  Total verbs: {}\n", total_verbs));
    summary.push_str(&format!("  Players: {:?}\n", players));
    
    if !important_objects.is_empty() {
        summary.push_str("\n  Important objects:\n");
        for (id, name) in important_objects.iter().take(10) {
            summary.push_str(&format!("    #{}: {}\n", id, name));
        }
        if important_objects.len() > 10 {
            summary.push_str(&format!("    ... and {} more\n", important_objects.len() - 10));
        }
    }
    
    Ok(summary)
}