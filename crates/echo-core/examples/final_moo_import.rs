use std::fs;
use std::path::Path;
use std::collections::HashMap;
use echo_core::parser::moo_preprocessor::MooPreprocessor;
use echo_core::parser::moo_object_parser::MooObjectParser;
use echo_core::ast::{EchoAst, ObjectMember};
use echo_core::storage::object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};

/// Complete MOO import system with proper object ID mapping
struct MooImporter {
    preprocessor: MooPreprocessor,
    store: ObjectStore,
}

impl MooImporter {
    fn new(store: ObjectStore) -> Self {
        Self {
            preprocessor: MooPreprocessor::new(),
            store,
        }
    }
    
    /// Load constants from a MOO constants file
    fn load_constants(&mut self, path: &Path) -> anyhow::Result<()> {
        let content = fs::read_to_string(path)?;
        self.preprocessor.load_defines(&content);
        
        // Also ensure we have ObjectIds for all defined objects
        // Clone the defines to avoid borrow issues
        let defines: Vec<(String, String)> = self.preprocessor.defines()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
            
        for (_name, value) in defines {
            if value.starts_with('#') {
                if let Ok(num) = value[1..].parse::<i64>() {
                    self.store.get_or_create_moo_id(num);
                }
            }
        }
        
        Ok(())
    }
    
    
    /// Import a MOO object file
    fn import_object_file(&mut self, path: &Path, expected_number: Option<i64>) -> anyhow::Result<ObjectId> {
        let content = fs::read_to_string(path)?;
        
        let mut parser = MooObjectParser::new()
            .with_preprocessor(self.preprocessor.clone());
        
        let ast = parser.parse_object_file(&content)?;
        
        if let EchoAst::ObjectDef { name, parent, members } = ast {
            // Determine the object ID
            let obj_id = if let Some(num) = expected_number {
                self.store.get_or_create_moo_id(num)
            } else if name.starts_with("object_") {
                // Extract number from "object_N"
                if let Ok(num) = name[7..].parse::<i64>() {
                    self.store.get_or_create_moo_id(num)
                } else {
                    ObjectId::new()
                }
            } else {
                ObjectId::new()
            };
            
            // Register the MOO number mapping if we have one
            if let Some(num) = expected_number {
                self.store.register_moo_id(num, obj_id);
            }
            
            // Parse parent reference
            let parent_id = parent.as_ref().and_then(|p| {
                if p.starts_with('#') {
                    p[1..].parse::<i64>().ok()
                        .map(|num| self.store.get_or_create_moo_id(num))
                } else {
                    None
                }
            });
            
            // Convert members to properties and verbs
            let mut properties = HashMap::new();
            let verbs: HashMap<String, echo_core::storage::object_store::VerbDefinition> = HashMap::new();
            
            for member in members {
                match member {
                    ObjectMember::Property { name: prop_name, value, .. } => {
                        if let Ok(prop_value) = self.ast_to_property_value(&value) {
                            properties.insert(prop_name, prop_value);
                        }
                    }
                    ObjectMember::Verb { name: verb_name, body, .. } => {
                        // For now, store verb as a string property
                        let verb_str = format!("{:?}", body);
                        properties.insert(format!("verb:{}", verb_name), PropertyValue::String(verb_str));
                    }
                    _ => {}
                }
            }
            
            // Create the object
            let obj = EchoObject {
                id: obj_id,
                parent: parent_id,
                name: path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&name)
                    .to_string(),
                properties,
                property_capabilities: HashMap::new(),
                verbs: HashMap::new(),
                queries: HashMap::new(),
                meta: echo_core::evaluator::meta_object::MetaObject::new(obj_id),
            };
            
            self.store.store(obj)?;
            Ok(obj_id)
        } else {
            Err(anyhow::anyhow!("Expected ObjectDef from parser"))
        }
    }
    
    fn ast_to_property_value(&self, ast: &EchoAst) -> anyhow::Result<PropertyValue> {
        match ast {
            EchoAst::String(s) => Ok(PropertyValue::String(s.clone())),
            EchoAst::Number(n) => Ok(PropertyValue::Integer(*n)),
            EchoAst::Float(f) => Ok(PropertyValue::Float(*f)),
            EchoAst::Boolean(b) => Ok(PropertyValue::Boolean(*b)),
            EchoAst::Null => Ok(PropertyValue::Null),
            EchoAst::ObjectRef(n) => {
                let obj_id = self.store.resolve_moo_id(*n)
                    .unwrap_or_else(|| ObjectId::new());
                Ok(PropertyValue::Object(obj_id))
            }
            EchoAst::Map { entries } if entries.is_empty() => {
                Ok(PropertyValue::Map(HashMap::new()))
            }
            _ => Err(anyhow::anyhow!("Cannot convert {:?} to property value", ast)),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    
    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let store = ObjectStore::new(temp_dir.path())?;
    let mut importer = MooImporter::new(store);
    
    println!("=== MOO Import System ===\n");
    
    // Phase 1: Load constants
    println!("Phase 1: Loading constants...");
    let constants_path = Path::new(moo_dir).join("constants.moo");
    importer.load_constants(&constants_path)?;
    println!("  Loaded {} constants", importer.preprocessor.defines().len());
    println!("  Created {} object ID mappings", importer.store.moo_id_map.len());
    
    // Phase 2: Import objects in dependency order
    println!("\nPhase 2: Importing objects...");
    
    let objects_to_import = vec![
        ("sysobj.moo", Some(0)),
        ("root.moo", Some(1)),
        ("arch_wizard.moo", Some(2)),
        ("room.moo", Some(3)),
        ("player.moo", Some(4)),
        ("builder.moo", Some(5)),
        ("prog.moo", Some(6)),
        ("hacker.moo", Some(7)),
        ("wiz.moo", Some(8)),
        ("string.moo", Some(10)),
        ("password.moo", Some(11)),
        ("first_room.moo", Some(12)),
        ("login.moo", Some(13)),
        ("event.moo", Some(14)),
        ("sub.moo", Some(15)),
        ("block.moo", Some(16)),
        ("look.moo", Some(17)),
        ("list.moo", Some(18)),
        ("thing.moo", Some(19)),
    ];
    
    let mut imported_count = 0;
    let mut failed_count = 0;
    
    for (filename, expected_num) in objects_to_import {
        let path = Path::new(moo_dir).join(filename);
        print!("  Importing {}: ", filename);
        
        match importer.import_object_file(&path, expected_num) {
            Ok(obj_id) => {
                // Verify it worked
                if let Ok(obj) = importer.store.get(obj_id) {
                    println!("OK - {} (parent: {:?}, {} properties)", 
                        obj_id, obj.parent, obj.properties.len());
                    imported_count += 1;
                } else {
                    println!("ERROR - Failed to retrieve object");
                    failed_count += 1;
                }
            }
            Err(e) => {
                println!("FAILED - {}", e);
                failed_count += 1;
            }
        }
    }
    
    // Phase 3: Summary
    println!("\n=== Import Summary ===");
    println!("Total objects imported: {}", imported_count);
    println!("Failed imports: {}", failed_count);
    
    // Show object hierarchy
    println!("\n=== Object Hierarchy ===");
    for (moo_num, obj_id) in &importer.store.moo_id_map {
        if let Ok(obj) = importer.store.get(*obj_id) {
            let parent_desc = if let Some(parent_id) = obj.parent {
                // Find the MOO number for this parent
                importer.store.moo_id_map.iter()
                    .find(|(_, id)| **id == parent_id)
                    .map(|(num, _)| format!("#{}", num))
                    .unwrap_or_else(|| format!("{:?}", parent_id))
            } else {
                "None".to_string()
            };
            
            println!("  #{} ({}) -> parent: {}", moo_num, obj.name, parent_desc);
        }
    }
    
    Ok(())
}