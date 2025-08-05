#[cfg(test)]
mod tests {
    use super::super::moo_preprocessor::MooPreprocessor;
    use super::super::moo_object_parser::MooObjectParser;
    use crate::ast::{EchoAst, ObjectMember};

    #[test]
    fn test_moo_preprocessor_multiple_defines() {
        let mut preprocessor = MooPreprocessor::new();
        
        let source = r#"
define ROOT = #1;
define PLAYER = #4;
define NOTHING = #-1;

object ROOT
  name: "Root Object"
  parent: NOTHING
endobject

object PLAYER
  parent: ROOT
  location: NOTHING
endobject
"#;
        
        let processed = preprocessor.process(source);
        
        // Defines should be removed and substitutions made
        assert!(!processed.contains("define"));
        assert!(processed.contains("object #1"));
        assert!(processed.contains("object #4"));
        assert!(processed.contains("parent: #1"));
        assert!(processed.contains("parent: #-1"));
        assert!(processed.contains("location: #-1"));
    }

    #[test]
    fn test_moo_object_parser_with_properties() {
        let source = r#"object TEST
  name: "Test Object"
  parent: #1
  location: #-1
  owner: #2
  readable: true
  
  property test_prop (owner: #2, flags: "r") = "test_value";
  property number_prop (owner: #2, flags: "rw") = 42;
  override simple_prop = {1, 2, 3};
endobject"#;

        let mut parser = MooObjectParser::new();
        let result = parser.parse_object_file(source);
        
        assert!(result.is_ok());
        
        if let Ok(EchoAst::ObjectDef { name, parent, members }) = result {
            assert_eq!(name, "TEST");
            assert_eq!(parent, Some("#1".to_string()));
            
            // Should have parsed all properties
            let mut property_count = 0;
            for member in &members {
                match member {
                    ObjectMember::Property { name, .. } => {
                        property_count += 1;
                        println!("Found property: {}", name);
                    }
                    _ => {}
                }
            }
            assert!(property_count >= 5); // Basic properties plus test properties
        } else {
            panic!("Expected ObjectDef, got: {:?}", result);
        }
    }

    #[test]
    fn test_moo_object_parser_with_verbs() {
        let source = r#"object TEST
  name: "Test Object"
  
  verb "test_verb" (this none this) owner: #2 flags: "rxd"
    "This is a test verb";
    return "Hello, world!";
  endverb
  
  verb "another_verb @av" (any any any) owner: #2 flags: "rx"
    "Multi-line verb";
    notify(player, "Test message");
    return args[1];
  endverb
endobject"#;

        let mut parser = MooObjectParser::new();
        let result = parser.parse_object_file(source);
        
        assert!(result.is_ok());
        
        if let Ok(EchoAst::ObjectDef { name, members, .. }) = result {
            assert_eq!(name, "TEST");
            
            let mut verb_count = 0;
            for member in &members {
                match member {
                    ObjectMember::Verb { name, body, .. } => {
                        verb_count += 1;
                        println!("Found verb: {}", name);
                        assert!(!body.is_empty());
                    }
                    _ => {}
                }
            }
            assert_eq!(verb_count, 2);
        }
    }

    #[test]
    fn test_moo_object_parser_object_references() {
        let source = r#"object #5
  name: "Numbered Object"
  parent: #1
  test_ref: #42
endobject"#;

        let mut parser = MooObjectParser::new();
        let result = parser.parse_object_file(source);
        
        assert!(result.is_ok());
        
        if let Ok(EchoAst::ObjectDef { name, parent, members }) = result {
            assert_eq!(name, "object_5"); // Should convert #5 to object_5
            assert_eq!(parent, Some("#1".to_string()));
            
            // Check that object reference was parsed correctly in properties
            let mut found_ref = false;
            for member in &members {
                if let ObjectMember::Property { name, value, .. } = member {
                    if name == "test_ref" {
                        if let EchoAst::ObjectRef(42) = value {
                            found_ref = true;
                        }
                    }
                }
            }
            assert!(found_ref, "Object reference #42 should be parsed correctly");
        }
    }

    #[test] 
    fn test_constants_integration() {
        let constants = r#"
define FAILED_MATCH = #-3;
define AMBIGUOUS = #-2;
define NOTHING = #-1;
define SYSOBJ = #0;
define ROOT = #1;
define ARCH_WIZARD = #2;
define FIRST_ROOM = #3;
define PLAYER = #4;
define BUILDER = #5;
define PROG = #6;
define HACKER = #7;
define WIZ = #8;
"#;

        let mut preprocessor = MooPreprocessor::new();
        preprocessor.load_defines(constants);
        
        // Test that constants are loaded (may include more from prior processing)
        assert!(preprocessor.defines().len() >= 9);
        assert_eq!(preprocessor.defines().get("ROOT"), Some(&"#1".to_string()));
        assert_eq!(preprocessor.defines().get("NOTHING"), Some(&"#-1".to_string()));
        
        // Test object definition with constants
        let object_source = r#"object HACKER
  name: "Hacker"
  parent: PROG
  location: FIRST_ROOM
  owner: HACKER
endobject"#;

        let processed = preprocessor.process(object_source);
        
        // All constants should be substituted
        assert!(processed.contains("object #7"));
        assert!(processed.contains("parent: #6"));
        assert!(processed.contains("location: #3"));
        assert!(processed.contains("owner: #7"));
    }

    #[test]
    fn test_end_to_end_parsing() {
        // Test the complete pipeline: constants → preprocessing → object parsing
        let constants = r#"
define ROOT = #1;
define HACKER = #7;
define PROG = #6;
"#;

        let object_source = r#"object HACKER
  name: "Hacker"
  parent: PROG
  location: ROOT
  
  property test_prop (owner: HACKER, flags: "r") = "test";
  
  verb test (this none this) owner: HACKER flags: "rxd"
    return "Hello from hacker";
  endverb
endobject"#;

        let mut preprocessor = MooPreprocessor::new();
        preprocessor.load_defines(constants);
        
        let mut parser = MooObjectParser::new()
            .with_preprocessor(preprocessor);
        
        let result = parser.parse_object_file(object_source);
        assert!(result.is_ok());
        
        if let Ok(EchoAst::ObjectDef { name, parent, members }) = result {
            // The object name gets substituted by the preprocessor: HACKER -> #7 -> object_7
            assert!(name == "HACKER" || name == "object_7");
            assert_eq!(parent, Some("#6".to_string())); // PROG should be substituted
            
            // Should have both property and verb
            let mut has_property = false;
            let mut has_verb = false;
            
            for member in &members {
                match member {
                    ObjectMember::Property { name, .. } if name == "test_prop" => {
                        has_property = true;
                    }
                    ObjectMember::Verb { name, .. } if name == "test" => {
                        has_verb = true;
                    }
                    ObjectMember::Property { name, .. } => {
                        println!("Other property: {}", name);
                    }
                    _ => {}
                }
            }
            
            assert!(has_property, "Should have test_prop property");
            assert!(has_verb, "Should have test verb");
        }
    }
}