use std::fs;
use std::path::Path;
use echo_core::parser::moo_compat::import_moo_objects;
use echo_core::storage::object_store::ObjectStore;

fn main() -> anyhow::Result<()> {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    
    // Create a temporary database for testing
    let temp_dir = tempfile::tempdir()?;
    let store = ObjectStore::new(temp_dir.path())?;
    
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
        "constants.moo",    // Constants definitions
    ];

    println!("Importing MOO objects from cowbell...\n");
    
    let mut total_imported = 0;
    let mut total_errors = 0;
    
    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        print!("Importing {}: ", file);
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                match import_moo_objects(&content, &store) {
                    Ok(imported_ids) => {
                        println!("SUCCESS - imported {} objects", imported_ids.len());
                        total_imported += imported_ids.len();
                        
                        // Show what was imported
                        for id in &imported_ids {
                            if let Ok(obj) = store.get(*id) {
                                println!("  - Object {:?}: parent={:?}, properties={}, verbs={}", 
                                    id, 
                                    obj.parent,
                                    obj.properties.len(),
                                    obj.verbs.len()
                                );
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
    
    // Show the object hierarchy
    println!("\n=== OBJECT HIERARCHY ===");
    // Note: The current ObjectStore API doesn't provide a way to list all objects
    // This would need to be added to the ObjectStore implementation
    
    println!("\n=== STATISTICS ===");
    println!("Total objects imported: {}", total_imported);
    println!("Database size: ~{} bytes", store.estimated_size()?);
    
    Ok(())
}