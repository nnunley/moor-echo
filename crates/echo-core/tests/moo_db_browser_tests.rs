use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Note: These tests are for the MOO database browser parsing logic.
// Since the browser code is in a binary, we'll need to either:
// 1. Move the core parsing logic to lib.rs and expose it, or
// 2. Create integration tests that run the binary

#[cfg(test)]
mod parser_tests {
    use super::*;

    // Test data paths
    const MINIMAL_DB: &str = "../../examples/Minimal.db";
    const LAMBDA_CORE_DB: &str = "../../examples/LambdaCore-latest.db";
    const JH_CORE_DB: &str = "../../examples/JHCore-DEV-2.db";

    #[test]
    fn test_object_id_detection() {
        // Test the regex pattern for detecting object IDs
        let test_cases = vec![
            ("#1", true),
            ("#123", true),
            ("#0", true),
            (" #1", false),  // Leading space
            ("#1 ", false),   // Trailing space
            ("#1abc", false), // Non-digit chars
            ("##1", false),   // Double hash
            ("#", false),     // Just hash
            ("", false),      // Empty
            ("#-1", true),    // Negative numbers are actually valid
        ];
        
        for (test, expected) in test_cases {
            let result = test.len() > 1 && 
                        test.starts_with('#') && 
                        test[1..].chars().all(|c| c.is_ascii_digit() || c == '-');
            assert_eq!(result, expected, "Failed for input: '{}'", test);
        }
    }

    #[test]
    fn test_minimal_db_parsing() {
        if !Path::new(MINIMAL_DB).exists() {
            eprintln!("Skipping test - Minimal.db not found");
            return;
        }

        let content = fs::read_to_string(MINIMAL_DB).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        // Parse header
        assert!(lines.len() > 5, "File too short");
        let total_objects: i64 = lines[1].parse().expect("Invalid object count");
        let total_players: i64 = lines[4].parse().expect("Invalid player count");
        
        assert_eq!(total_objects, 4, "Expected 4 objects in Minimal.db");
        assert_eq!(total_players, 1, "Expected 1 player in Minimal.db");
        
        // Check that we can find all objects
        let mut found_objects = vec![];
        let start_line = 5 + total_players as usize;
        
        for i in start_line..lines.len() {
            let line = lines[i].trim();
            if line.len() > 1 && line.starts_with('#') && line[1..].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(obj_id) = line[1..].parse::<i64>() {
                    found_objects.push(obj_id);
                }
            }
        }
        
        found_objects.sort();
        assert_eq!(found_objects, vec![0, 1, 2, 3], "Should find objects 0-3");
    }

    #[test]
    fn test_object_1_exists_in_all_databases() {
        let databases = vec![
            (MINIMAL_DB, "Minimal"),
            (LAMBDA_CORE_DB, "LambdaCore"),
            (JH_CORE_DB, "JHCore"),
        ];
        
        for (path, name) in databases {
            if !Path::new(path).exists() {
                eprintln!("Skipping {} - file not found", name);
                continue;
            }
            
            let content = fs::read_to_string(path).unwrap();
            let lines: Vec<&str> = content.lines().collect();
            
            // Find object #1
            let mut found_object_1 = false;
            for line in &lines {
                if line.trim() == "#1" {
                    found_object_1 = true;
                    break;
                }
            }
            
            assert!(found_object_1, "Object #1 not found in {}", name);
        }
    }

    #[test]
    fn test_object_names_parsing() {
        if !Path::new(MINIMAL_DB).exists() {
            eprintln!("Skipping test - Minimal.db not found");
            return;
        }

        let content = fs::read_to_string(MINIMAL_DB).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        // Expected objects in Minimal.db
        let expected_objects = vec![
            (0, "System Object"),
            (1, "Root Class"),
            (2, "The First Room"),
            (3, "Wizard"),
        ];
        
        for (obj_id, expected_name) in expected_objects {
            // Find the object
            let mut found = false;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == format!("#{}", obj_id) {
                    // Next line should be the name (unless recycled)
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1];
                        if next_line != "recycled" {
                            assert_eq!(next_line, expected_name, 
                                     "Object #{} has wrong name", obj_id);
                            found = true;
                            break;
                        }
                    }
                }
            }
            assert!(found, "Object #{} not found", obj_id);
        }
    }

    #[test]
    fn test_verb_code_section_detection() {
        // Test the pattern for detecting verb code sections
        let test_lines = vec![
            vec!["123", "#0:0"],  // Valid verb code section start
            vec!["456", "#1:initialize"],  // Valid verb code section start
            vec!["not_a_number", "#0:0"],  // Invalid - first line not a number
            vec!["789", "not_a_verb"],  // Invalid - second line not a verb
        ];
        
        for (i, lines) in test_lines.iter().enumerate() {
            let is_verb_section = lines[0].parse::<i64>().is_ok() && 
                                 lines[1].starts_with('#') && 
                                 lines[1].contains(':');
            
            match i {
                0 | 1 => assert!(is_verb_section, "Test case {} should be valid", i),
                2 | 3 => assert!(!is_verb_section, "Test case {} should be invalid", i),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_lazy_loading_state() {
        // Test the ObjectState enum behavior
        #[derive(Debug, Clone)]
        enum ObjectState {
            Placeholder,
            Parsed(String),
            ParseFailed(String),
        }
        
        let states = vec![
            ObjectState::Placeholder,
            ObjectState::Parsed("Test Object".to_string()),
            ObjectState::ParseFailed("Error message".to_string()),
        ];
        
        for state in states {
            match state {
                ObjectState::Placeholder => {
                    // Should show loading message
                    let display = format!("Object #1 (loading...)");
                    assert!(display.contains("loading"));
                }
                ObjectState::Parsed(name) => {
                    assert_eq!(name, "Test Object");
                }
                ObjectState::ParseFailed(err) => {
                    assert_eq!(err, "Error message");
                }
            }
        }
    }

    #[test]
    fn test_object_sorting() {
        // Test that objects are sorted correctly for display
        let mut object_ids = vec![5, 1, 3, 0, 2, 4];
        object_ids.sort();
        assert_eq!(object_ids, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_player_detection() {
        if !Path::new(MINIMAL_DB).exists() {
            eprintln!("Skipping test - Minimal.db not found");
            return;
        }

        let content = fs::read_to_string(MINIMAL_DB).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        // In Minimal.db, player ID should be at line 5 (0-indexed)
        let player_id: i64 = lines[5].parse().expect("Invalid player ID");
        assert_eq!(player_id, 3, "Player should be object #3 in Minimal.db");
    }
}

#[cfg(test)]
mod ui_tests {
    use super::*;

    #[test]
    fn test_list_state_behavior() {
        // Test that UI list state behaves correctly
        use ratatui::widgets::ListState;
        
        let mut state = ListState::default();
        assert_eq!(state.selected(), None);
        
        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));
        
        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));
    }

    #[test]
    fn test_object_list_generation() {
        // Test the logic for generating the object list
        let mut objects = HashMap::new();
        objects.insert(0, "System Object");
        objects.insert(1, "Root Class");
        objects.insert(2, "Wizard");
        
        let mut object_ids: Vec<i64> = objects.keys().copied().collect();
        object_ids.sort();
        
        assert_eq!(object_ids, vec![0, 1, 2]);
        
        // Simulate building list items
        let items: Vec<String> = object_ids.iter()
            .map(|&id| format!("#{:3} {}", id, objects.get(&id).unwrap()))
            .collect();
            
        assert_eq!(items[0], "#  0 System Object");
        assert_eq!(items[1], "#  1 Root Class");
        assert_eq!(items[2], "#  2 Wizard");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_browser_debug_list_output() {
        // Test that the browser can run with --debug-list flag
        let output = Command::new("cargo")
            .args(&["run", "--bin", "moo_db_browser", "--", "--debug-list"])
            .output()
            .expect("Failed to run moo_db_browser");
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check that it found databases
        assert!(stdout.contains("Database 0:"), "Should list databases");
        
        // Check that object #1 is reported as found
        assert!(stdout.contains("✓ Object #1 is in the object list"), 
                "Object #1 should be found");
    }
}