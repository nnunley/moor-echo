use std::sync::Arc;
use echo_core::{EchoConfig, EchoRuntime};

fn main() -> anyhow::Result<()> {
    // Initialize runtime
    let config = EchoConfig {
        storage_path: "./temp_notify_demo_db".into(),
        debug: false,
        max_objects: 10000,
        max_eval_depth: 100,
        enable_jit: false,
    };
    
    let mut runtime = EchoRuntime::new(config)?;
    
    // Set up a simple UI callback to see notifications
    let ui_callback = Arc::new(|event: echo_core::UiEvent| {
        match event.action {
            echo_core::UiAction::NotifyPlayer { player_id, message } => {
                println!("🔔 NOTIFICATION for player {}: {}", player_id, message);
            }
            _ => {
                println!("📋 UI Event: {:?}", event.action);
            }
        }
    });
    runtime.set_ui_callback(ui_callback);
    
    println!("=== MOO notify() Function Demo ===\n");
    
    // Create a test player with unique name
    let player_name = format!("demo_player_{}", std::process::id());
    let player_id = runtime.create_player(&player_name)?;
    println!("Created player '{}' with ID: {}", player_name, player_id);
    
    // Switch to that player  
    runtime.switch_player(player_id)?;
    
    // Test 1: Basic notify with MOO object number
    println!("\n--- Test 1: notify() with root object ---");
    let code = r#"notify(1, "Hello to root object #1!")"#; // Use root object
    let result = runtime.eval_source(&code)?;
    println!("Result: {:?}", result);
    
    // Test 2: notify with MOO object number (assuming player is in MOO space)
    println!("\n--- Test 2: notify() with MOO object number ---");
    let code = r#"notify(1, "Message to root object")"#; // Root object
    let result = runtime.eval_source(code)?;
    println!("Result: {:?}", result);
    
    // Test 3: notify with string conversion
    println!("\n--- Test 3: notify() with non-string message ---");
    let code = r#"notify(1, 42)"#;
    let result = runtime.eval_source(code)?;
    println!("Result: {:?}", result);
    
    // Test 4: Multiple notifications
    println!("\n--- Test 4: Multiple notifications ---");
    runtime.eval_source(r#"notify(1, "First message")"#)?;
    runtime.eval_source(r#"notify(1, "Second message")"#)?;
    let result = runtime.eval_source(r#""All messages sent""#)?;
    println!("Result: {:?}", result);
    
    println!("\n=== Demo Complete ===");
    
    // Clean up
    std::fs::remove_dir_all("./temp_notify_demo_db").ok();
    
    Ok(())
}