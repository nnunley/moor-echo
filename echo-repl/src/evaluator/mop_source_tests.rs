// Tests for MOP source code access

#[cfg(test)]
mod mop_source_tests {
    use crate::evaluator::meta_object::{MetaObject, VerbMetadata, PropertyMetadata};
    use crate::storage::ObjectId;
    
    #[test]
    fn test_verb_source_code_in_metadata() {
        let mut meta = MetaObject::new(ObjectId::new());
        
        // Add a verb with source code
        let verb_meta = VerbMetadata {
            name: "greet".to_string(),
            callable: true,
            inheritable: true,
            source_code: Some("verb greet(name)\n  return \"Hello, \" + name;\nendverb".to_string()),
        };
        
        meta.verbs_meta.insert("greet".to_string(), verb_meta);
        
        // Test accessing source code
        let source = meta.get_source_code("verb", "greet");
        assert!(source.is_some());
        assert_eq!(source.unwrap(), "verb greet(name)\n  return \"Hello, \" + name;\nendverb");
    }
    
    #[test]
    fn test_property_lambda_source_code() {
        let mut meta = MetaObject::new(ObjectId::new());
        
        // Add a property containing a lambda with source code
        let prop_meta = PropertyMetadata {
            name: "calculator".to_string(),
            readable: true,
            writable: false,
            inheritable: true,
            source_code: Some("fn {x, y} x + y endfn".to_string()),
        };
        
        meta.properties_meta.insert("calculator".to_string(), prop_meta);
        
        // Test accessing source code
        let source = meta.get_source_code("property", "calculator");
        assert!(source.is_some());
        assert_eq!(source.unwrap(), "fn {x, y} x + y endfn");
    }
    
    #[test]
    fn test_nonexistent_element_source_code() {
        let meta = MetaObject::new(ObjectId::new());
        
        // Test accessing non-existent elements
        assert_eq!(meta.get_source_code("verb", "nonexistent"), None);
        assert_eq!(meta.get_source_code("property", "missing"), None);
        assert_eq!(meta.get_source_code("invalid_type", "something"), None);
    }
}