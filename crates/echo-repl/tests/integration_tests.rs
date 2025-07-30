use echo_repl::repl::{Repl, ReplCommand};
use tempfile::TempDir;

#[test]
fn test_player_creation() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();

    // Create a player
    let result = repl.handle_command(ReplCommand::CreatePlayer("alice".to_string()));
    assert!(result.is_ok());
    assert!(result
        .unwrap()
        .contains("Created and switched to player 'alice'"));
}

#[test]
fn test_player_switching() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();

    // Create two players
    repl.handle_command(ReplCommand::CreatePlayer("alice".to_string()))
        .unwrap();
    repl.handle_command(ReplCommand::CreatePlayer("bob".to_string()))
        .unwrap();

    // Switch back to alice
    let result = repl.handle_command(ReplCommand::SwitchPlayer("alice".to_string()));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Switched to player 'alice'"));
}

#[test]
fn test_basic_arithmetic() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();

    // Execute simple arithmetic
    let result = repl.execute("1 + 2");
    assert!(result.is_ok());
    let (output, _duration) = result.unwrap();
    assert_eq!(output, "3");
}
