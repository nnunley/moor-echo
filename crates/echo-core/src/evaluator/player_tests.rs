#[cfg(test)]
mod player_registry_tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::super::*;
    use crate::storage::{ObjectId, Storage};

    fn setup_test() -> (Evaluator, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        let evaluator = Evaluator::new(storage);
        (evaluator, temp_dir)
    }

    #[test]
    fn test_player_creation_and_registry() {
        let (mut evaluator, _temp_dir) = setup_test();

        // Create a player
        let player1_id = evaluator.create_player("alice").unwrap();

        // Verify player exists
        let player1 = evaluator.storage.objects.get(player1_id).unwrap();
        assert_eq!(
            player1.properties.get("username").unwrap(),
            &crate::storage::PropertyValue::String("alice".to_string())
        );
        assert_eq!(
            player1.properties.get("display_name").unwrap(),
            &crate::storage::PropertyValue::String("alice".to_string())
        );

        // Verify player is in registry
        let found = evaluator.find_player_by_username("alice").unwrap();
        assert_eq!(found, Some(player1_id));

        // Verify system object has player_registry
        let system_obj = evaluator.storage.objects.get(ObjectId::system()).unwrap();
        assert!(system_obj.properties.contains_key("player_registry"));
    }

    #[test]
    fn test_duplicate_player_names() {
        let (mut evaluator, _temp_dir) = setup_test();

        // Create first player
        evaluator.create_player("bob").unwrap();

        // Try to create another player with same name
        let result = evaluator.create_player("bob");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_player_username_change() {
        let (mut evaluator, _temp_dir) = setup_test();

        // Create a player
        let player_id = evaluator.create_player("charlie").unwrap();

        // Change username
        evaluator
            .change_player_username(player_id, "charles")
            .unwrap();

        // Verify old username no longer works
        let old_lookup = evaluator.find_player_by_username("charlie").unwrap();
        assert_eq!(old_lookup, None);

        // Verify new username works
        let new_lookup = evaluator.find_player_by_username("charles").unwrap();
        assert_eq!(new_lookup, Some(player_id));

        // Verify player properties updated
        let player = evaluator.storage.objects.get(player_id).unwrap();
        assert_eq!(
            player.properties.get("username").unwrap(),
            &crate::storage::PropertyValue::String("charles".to_string())
        );
    }

    #[test]
    fn test_player_not_bound_to_system() {
        let (mut evaluator, _temp_dir) = setup_test();

        // Create a player
        let player_id = evaluator.create_player("dave").unwrap();

        // Verify player is NOT bound as a property on #0
        let system_obj = evaluator.storage.objects.get(ObjectId::system()).unwrap();
        assert!(!system_obj.properties.contains_key("dave"));

        // Player should have an anonymous-style name
        let player = evaluator.storage.objects.get(player_id).unwrap();
        assert!(player.name.starts_with("player_"));
        assert!(!player.name.contains("dave"));
    }

    #[test]
    fn test_username_conflict_on_change() {
        let (mut evaluator, _temp_dir) = setup_test();

        // Create two players
        let player1_id = evaluator.create_player("emily").unwrap();
        let _player2_id = evaluator.create_player("frank").unwrap();

        // Try to change emily's username to frank
        let result = evaluator.change_player_username(player1_id, "frank");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already taken"));
    }
}
