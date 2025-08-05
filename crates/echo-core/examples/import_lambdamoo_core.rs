use std::fs;
use std::path::Path;
use std::collections::HashMap;

use echo_core::parser::moo_compat::MooCompatParser;
use echo_core::storage::object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};
use echo_core::evaluator::meta_object::MetaObject;
use echo_core::ast::EchoAst;

/// Simple parser for LambdaMOO database format version 1
/// This is a minimal implementation focused on parsing Minimal.db
struct MinimalDbParser {
    lines: Vec<String>,
    current: usize,
}

impl MinimalDbParser {
    fn new(content: &str) -> Self {
        Self {
            lines: content.lines().map(|s| s.to_string()).collect(),
            current: 0,
        }
    }
    
    fn next_line(&mut self) -> Option<String> {
        if self.current < self.lines.len() {
            let line = self.lines[self.current].clone();
            self.current += 1;
            Some(line)
        } else {
            None
        }
    }
    
    fn parse_number(&mut self) -> i64 {
        self.next_line()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
    
    fn parse_objid(&mut self) -> i64 {
        self.parse_number()
    }
}

fn main() -> anyhow::Result<()> {
    // Path to the Minimal.db file
    let db_path = Path::new("../../../examples/Minimal.db");
    
    println!("=== LambdaMOO Core Import Example ===\n");
    println!("Reading database from: {:?}", db_path);
    
    // Read the database file
    let content = fs::read_to_string(db_path)?;
    let mut parser = MinimalDbParser::new(&content);
    
    // Parse header
    let header = parser.next_line().unwrap();
    println!("Database header: {}", header);
    
    // Parse counts
    let total_objects = parser.parse_number();
    let total_verbs = parser.parse_number();
    let _dummy = parser.parse_number();
    let total_players = parser.parse_number();
    
    println!("Total objects: {}", total_objects);
    println!("Total verbs: {}", total_verbs);
    println!("Total players: {}", total_players);
    
    // Parse player list
    let player_id = parser.parse_objid();
    println!("Player: #{}", player_id);
    
    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let mut store = ObjectStore::new(temp_dir.path())?;
    
    // Create ID mappings
    let mut id_map = HashMap::new();
    
    // Parse objects
    println!("\nParsing objects:");
    for i in 0..total_objects {
        // Parse object header
        let obj_header = parser.next_line().unwrap();
        println!("\n{}", obj_header);
        
        // Parse object name
        let name = parser.next_line().unwrap();
        println!("  Name: {}", name);
        
        // Skip empty line (old handles)
        parser.next_line();
        
        // Parse object fields
        let flags = parser.parse_number();
        let owner = parser.parse_objid();
        let location = parser.parse_objid();
        let contents = parser.parse_objid();
        let next = parser.parse_objid();
        let parent = parser.parse_objid();
        let child = parser.parse_objid();
        let sibling = parser.parse_objid();
        
        println!("  Flags: {}", flags);
        println!("  Owner: #{}", owner);
        println!("  Location: #{}", location);
        println!("  Parent: #{}", parent);
        
        // Create Echo object
        let echo_id = store.get_or_create_moo_id(i);
        id_map.insert(i, echo_id);
        
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), PropertyValue::String(name.clone()));
        properties.insert("owner".to_string(), PropertyValue::Object(
            if owner >= 0 { store.get_or_create_moo_id(owner) } else { ObjectId::new() }
        ));
        
        let echo_obj = EchoObject {
            id: echo_id,
            parent: if parent >= 0 { 
                Some(store.get_or_create_moo_id(parent))
            } else { 
                None 
            },
            name: name.clone(),
            properties,
            property_capabilities: HashMap::new(),
            verbs: HashMap::new(),
            queries: HashMap::new(),
            meta: MetaObject::new(echo_id),
        };
        
        store.store(echo_obj)?;
        
        // Parse verb count
        let verb_count = parser.parse_number();
        println!("  Verbs: {}", verb_count);
        
        // Parse verbs
        for j in 0..verb_count {
            let verb_name = parser.next_line().unwrap();
            let verb_owner = parser.parse_objid();
            let verb_perms = parser.parse_number();
            let verb_prep = parser.parse_number();
            
            println!("    Verb {}: {} (owner: #{}, perms: {}, prep: {})", 
                     j, verb_name, verb_owner, verb_perms, verb_prep);
        }
        
        // Parse property definitions
        let propdef_count = parser.parse_number();
        println!("  Property definitions: {}", propdef_count);
        
        for _ in 0..propdef_count {
            let prop_name = parser.next_line().unwrap();
            println!("    Property: {}", prop_name);
        }
        
        // Parse property values
        let propval_count = parser.parse_number();
        println!("  Property values: {}", propval_count);
        
        // For now, skip property value parsing as it requires recursive value parsing
    }
    
    // Skip to verb programs section
    while let Some(line) = parser.next_line() {
        if line.starts_with("#0:0") {
            println!("\nFound verb program: {}", line);
            
            // Read verb code until '.'
            let mut code_lines = Vec::new();
            while let Some(code_line) = parser.next_line() {
                if code_line == "." {
                    break;
                }
                code_lines.push(code_line);
            }
            
            let code = code_lines.join("\n");
            println!("  Code: {}", code);
        } else if line.starts_with("#0:1") {
            println!("\nFound verb program: {}", line);
            
            // Read verb code until '.'
            let mut code_lines = Vec::new();
            while let Some(code_line) = parser.next_line() {
                if code_line == "." {
                    break;
                }
                code_lines.push(code_line);
            }
            
            let code = code_lines.join("\n");
            println!("  Code: {}", code);
        }
    }
    
    // Summary
    println!("\n=== Import Summary ===");
    println!("Created {} objects", id_map.len());
    
    // Show object hierarchy
    println!("\nObject hierarchy:");
    for i in 0..total_objects {
        if let Some(&echo_id) = id_map.get(&i) {
            if let Ok(obj) = store.get(echo_id) {
                let parent_desc = if let Some(parent_id) = obj.parent {
                    format!("parent: {}", parent_id)
                } else {
                    "no parent".to_string()
                };
                println!("  #{} ({}) - {}", i, obj.name, parent_desc);
            }
        }
    }
    
    // Test that we can retrieve objects
    println!("\nTesting object retrieval:");
    if let Some(&system_id) = id_map.get(&0) {
        let system_obj = store.get(system_id)?;
        println!("  System Object: {} (id: {})", system_obj.name, system_obj.id);
    }
    
    if let Some(&wizard_id) = id_map.get(&3) {
        let wizard_obj = store.get(wizard_id)?;
        println!("  Wizard: {} (id: {})", wizard_obj.name, wizard_obj.id);
    }
    
    Ok(())
}