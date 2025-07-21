use echo_repl::repl::{Repl, ReplCommand};
use tempfile::TempDir;

#[test]
fn test_player_creation() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create a player
    let result = repl.handle_command(ReplCommand::CreatePlayer("alice".to_string()));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Created and switched to player 'alice'"));
}

#[test]
fn test_player_switching() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create two players
    repl.handle_command(ReplCommand::CreatePlayer("alice".to_string())).unwrap();
    repl.handle_command(ReplCommand::CreatePlayer("bob".to_string())).unwrap();
    
    // Switch back to alice
    let result = repl.handle_command(ReplCommand::SwitchPlayer("alice".to_string()));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Switched to player 'alice'"));
}

#[test]
fn test_player_list() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create players
    repl.handle_command(ReplCommand::CreatePlayer("alice".to_string())).unwrap();
    repl.handle_command(ReplCommand::CreatePlayer("bob".to_string())).unwrap();
    
    // List players
    let result = repl.handle_command(ReplCommand::ListPlayers);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("alice"));
    assert!(output.contains("bob"));
}

#[test]
fn test_player_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create alice and set a variable
    repl.handle_command(ReplCommand::CreatePlayer("alice".to_string())).unwrap();
    repl.execute("let x = 100;").unwrap();
    assert_eq!(repl.execute("x").unwrap().0, "100");
    
    // Create bob and verify x doesn't exist
    repl.handle_command(ReplCommand::CreatePlayer("bob".to_string())).unwrap();
    let result = repl.execute("x");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Undefined variable"));
    
    // Set different value for bob
    repl.execute("let x = 200;").unwrap();
    assert_eq!(repl.execute("x").unwrap(), "200");
    
    // Switch back to alice and verify original value
    repl.handle_command(ReplCommand::SwitchPlayer("alice".to_string())).unwrap();
    assert_eq!(repl.execute("x").unwrap().0, "100");
}

#[test]
fn test_current_player() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Initially no player
    let result = repl.handle_command(ReplCommand::CurrentPlayer);
    assert!(result.unwrap().contains("No player selected"));
    
    // Create and check current
    repl.handle_command(ReplCommand::CreatePlayer("alice".to_string())).unwrap();
    let result = repl.handle_command(ReplCommand::CurrentPlayer);
    assert!(result.unwrap().contains("Current player: alice"));
}