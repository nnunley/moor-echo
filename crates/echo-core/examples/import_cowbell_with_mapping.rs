use std::fs;
use std::path::Path;
use std::collections::HashMap;
use echo_core::parser::moo_compat::MooCompatParser;
use echo_core::parser::Parser;
use echo_core::ast::{EchoAst, ObjectMember};
use echo_core::storage::object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};

/// Mapping from MOO object numbers to Echo ObjectIds
struct MooObjectMapping {
    /// Maps MOO object numbers (e.g., 1 for #1) to Echo ObjectIds
    number_to_id: HashMap<i64, ObjectId>,
    /// Maps MOO constant names (e.g., "ROOT") to MOO object numbers
    constant_to_number: HashMap<String, i64>,
}

impl MooObjectMapping {
    fn new() -> Self {
        // Pre-populate with system objects
        let mut number_to_id = HashMap::new();
        number_to_id.insert(0, ObjectId::system()); // #0 is system
        number_to_id.insert(1, ObjectId::root());    // #1 is root
        
        Self {
            number_to_id,
            constant_to_number: HashMap::new(),
        }
    }
    
    /// Get or create an ObjectId for a MOO object number
    fn get_or_create_id(&mut self, moo_number: i64) -> ObjectId {
        *self.number_to_id.entry(moo_number).or_insert_with(ObjectId::new)
    }
    
    /// Resolve a constant name to an ObjectId
    fn resolve_constant(&self, name: &str) -> Option<ObjectId> {
        self.constant_to_number.get(name)
            .and_then(|num| self.number_to_id.get(num))
            .copied()
    }
    
    /// Parse constants from a MOO file
    fn parse_constants(&mut self, ast: &EchoAst) {
        if let EchoAst::Program(statements) = ast {
            for stmt in statements {
                // Look for define statements or assignments that look like constants
                match stmt {
                    EchoAst::Define { name, value } => {
                        if let EchoAst::ObjectRef(num) = **value {
                            self.constant_to_number.insert(name.clone(), num);
                            self.get_or_create_id(num); // Ensure ID exists
                        }
                    }
                    EchoAst::Assignment { target, value } => {
                        // Handle constants that were parsed as regular assignments
                        if let echo_core::ast::LValue::Binding { 
                            binding_type: _, 
                            pattern: echo_core::ast::BindingPattern::Identifier(name) 
                        } = target {
                            if name.chars().all(|c| c.is_uppercase() || c == '_') {
                                // Looks like a constant
                                if let EchoAst::ObjectRef(num) = **value {
                                    self.constant_to_number.insert(name.clone(), num);
                                    self.get_or_create_id(num);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_moo_object_with_mapping(
    content: &str,
    store: &ObjectStore,
    mapping: &mut MooObjectMapping,
) -> anyhow::Result<Vec<ObjectId>> {
    let mut parser = MooCompatParser::new()?;
    let ast = parser.parse(content)?;
    let mut imported_ids = Vec::new();
    
    // First, extract any constants
    mapping.parse_constants(&ast);
    
    // Then look for object definitions
    if let EchoAst::Program(statements) = ast {
        // Check if the first statement is an identifier that might be an object name
        if statements.len() > 0 {
            if let EchoAst::Identifier(obj_name) = &statements[0] {
                // This might be "object NAME" syntax - check if NAME is a known constant
                if let Some(obj_id) = mapping.resolve_constant(obj_name) {
                    println!("  Found object {} -> {:?}", obj_name, obj_id);
                    
                    // Parse the rest of the file as object properties and verbs
                    let mut properties = HashMap::new();
                    let mut parent = None;
                    
                    // Look for property assignments in subsequent statements
                    for stmt in &statements[1..] {
                        match stmt {
                            EchoAst::Assignment { target, value } => {
                                if let echo_core::ast::LValue::Binding { 
                                    pattern: echo_core::ast::BindingPattern::Identifier(prop_name), 
                                    .. 
                                } = target {
                                    // Check for special properties
                                    if prop_name == "parent" {
                                        // Try to resolve parent
                                        match &**value {
                                            EchoAst::Identifier(parent_name) => {
                                                parent = mapping.resolve_constant(parent_name);
                                            }
                                            EchoAst::ObjectRef(num) => {
                                                parent = Some(mapping.get_or_create_id(*num));
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        // Regular property
                                        if let Ok(prop_value) = ast_to_property_value(value) {
                                            properties.insert(prop_name.clone(), prop_value);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    // Create the object
                    let obj = EchoObject {
                        id: obj_id,
                        parent,
                        name: obj_name.clone(),
                        properties,
                        property_capabilities: HashMap::new(),
                        verbs: HashMap::new(),
                        queries: HashMap::new(),
                        meta: echo_core::evaluator::meta_object::MetaObject::new(obj_id),
                    };
                    
                    store.store(obj)?;
                    imported_ids.push(obj_id);
                }
            }
        }
        
        // Also check for proper ObjectDef nodes
        for stmt in statements {
            if let EchoAst::ObjectDef { name, parent: parent_name, members } = stmt {
                // Try to resolve the object ID from the name
                let obj_id = if let Some(id) = mapping.resolve_constant(&name) {
                    id
                } else {
                    // Not a known constant, create new ID
                    ObjectId::new()
                };
                
                // Resolve parent
                let parent = parent_name.as_ref()
                    .and_then(|p| mapping.resolve_constant(p));
                
                // Process members
                let mut properties = HashMap::new();
                for member in members {
                    if let ObjectMember::Property { name, value, .. } = member {
                        if let Ok(prop_value) = ast_to_property_value(&value) {
                            properties.insert(name, prop_value);
                        }
                    }
                }
                
                let obj = EchoObject {
                    id: obj_id,
                    parent,
                    name: name.clone(),
                    properties,
                    property_capabilities: HashMap::new(),
                    verbs: HashMap::new(),
                    queries: HashMap::new(),
                    meta: echo_core::evaluator::meta_object::MetaObject::new(obj_id),
                };
                
                store.store(obj)?;
                imported_ids.push(obj_id);
            }
        }
    }
    
    Ok(imported_ids)
}

fn ast_to_property_value(ast: &EchoAst) -> anyhow::Result<PropertyValue> {
    match ast {
        EchoAst::String(s) => Ok(PropertyValue::String(s.clone())),
        EchoAst::Number(n) => Ok(PropertyValue::Integer(*n)),
        EchoAst::Float(f) => Ok(PropertyValue::Float(*f)),
        EchoAst::Boolean(b) => Ok(PropertyValue::Boolean(*b)),
        EchoAst::Null => Ok(PropertyValue::Null),
        EchoAst::ObjectRef(_n) => {
            // For now, create a new ObjectId - in a real implementation
            // we'd need to resolve this through the mapping
            Ok(PropertyValue::Object(ObjectId::new()))
        }
        _ => Err(anyhow::anyhow!("Cannot convert AST node to property value")),
    }
}

fn main() -> anyhow::Result<()> {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    
    // Create a temporary database for testing
    let temp_dir = tempfile::tempdir()?;
    let store = ObjectStore::new(temp_dir.path())?;
    let mut mapping = MooObjectMapping::new();
    
    println!("=== PHASE 1: Loading constants ===\n");
    
    // First, load constants.moo to build the mapping
    let constants_path = Path::new(moo_dir).join("constants.moo");
    if let Ok(content) = fs::read_to_string(&constants_path) {
        println!("Loading constants from constants.moo...");
        let mut parser = MooCompatParser::new()?;
        if let Ok(ast) = parser.parse(&content) {
            mapping.parse_constants(&ast);
            println!("Loaded {} constants", mapping.constant_to_number.len());
            for (name, num) in &mapping.constant_to_number {
                println!("  {} = #{}", name, num);
            }
        }
    }
    
    println!("\n=== PHASE 2: Importing objects ===\n");
    
    // List of MOO files in dependency order (base objects first)
    let moo_files = vec![
        "root.moo",         // #1 - Base object
        "arch_wizard.moo",  // #2
        "room.moo",         // #3
        "player.moo",       // #4
        "builder.moo",      // #5
        "prog.moo",         // #6
        "hacker.moo",       // #7
        "wiz.moo",          // #8
        "string.moo",       // #10
        "password.moo",     // #11
        "first_room.moo",   // #12
        "login.moo",        // #13
        "event.moo",        // #14
        "sub.moo",          // #15
        "block.moo",        // #16
        "look.moo",         // #17
        "list.moo",         // #18
        "thing.moo",        // #19
        "sysobj.moo",       // #0 - System object
    ];

    let mut total_imported = 0;
    let mut total_errors = 0;
    
    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        print!("Importing {}: ", file);
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                match parse_moo_object_with_mapping(&content, &store, &mut mapping) {
                    Ok(imported_ids) => {
                        if imported_ids.is_empty() {
                            println!("No objects found");
                        } else {
                            println!("SUCCESS - imported {} objects", imported_ids.len());
                            total_imported += imported_ids.len();
                            
                            for id in &imported_ids {
                                if let Ok(obj) = store.get(*id) {
                                    println!("  - Object {:?}: parent={:?}, properties={}", 
                                        id, 
                                        obj.parent,
                                        obj.properties.len()
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("FAILED - {}", e);
                        total_errors += 1;
                    }
                }
            }
            Err(e) => {
                println!("FAILED - Could not read file: {}", e);
                total_errors += 1;
            }
        }
    }
    
    println!("\n=== IMPORT SUMMARY ===");
    println!("Total files processed: {}", moo_files.len());
    println!("Total objects imported: {}", total_imported);
    println!("Total errors: {}", total_errors);
    
    println!("\n=== OBJECT MAPPING ===");
    for (num, id) in &mapping.number_to_id {
        if let Some((name, _)) = mapping.constant_to_number.iter().find(|(_, n)| **n == *num) {
            println!("#{} ({}) -> {:?}", num, name, id);
        } else {
            println!("#{} -> {:?}", num, id);
        }
    }
    
    Ok(())
}