#[cfg(test)]
mod tests {
    use super::super::{Evaluator, Value};
    use crate::storage::{Storage, ObjectId};
    use std::sync::Arc;

    fn create_test_evaluator() -> (Evaluator, ObjectId) {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        let mut evaluator = Evaluator::new(storage);
        let player_id = evaluator.create_player("test_player").unwrap();
        evaluator.switch_player(player_id).unwrap();
        (evaluator, player_id)
    }

    #[test]
    fn test_moo_valid_existing_object() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test with MOO object #1 which should exist (root object)
        let result = evaluator.moo_valid(&[Value::Integer(1)]).unwrap();
        assert_eq!(result, Value::Integer(1)); // Should be valid
    }

    #[test]
    fn test_moo_valid_nonexistent_object() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test with a high MOO object number that shouldn't exist
        let result = evaluator.moo_valid(&[Value::Integer(99999)]).unwrap();
        assert_eq!(result, Value::Integer(0)); // Should be invalid
    }

    #[test]
    fn test_moo_typeof_values() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test various value types
        assert_eq!(evaluator.moo_typeof(&[Value::Integer(42)]).unwrap(), Value::Integer(0)); // INT
        assert_eq!(evaluator.moo_typeof(&[Value::Float(3.14)]).unwrap(), Value::Integer(1)); // FLOAT
        assert_eq!(evaluator.moo_typeof(&[Value::String("hello".to_string())]).unwrap(), Value::Integer(2)); // STR
        assert_eq!(evaluator.moo_typeof(&[Value::List(vec![])]).unwrap(), Value::Integer(3)); // LIST
        assert_eq!(evaluator.moo_typeof(&[Value::Object(ObjectId::new())]).unwrap(), Value::Integer(4)); // OBJ
        assert_eq!(evaluator.moo_typeof(&[Value::Boolean(true)]).unwrap(), Value::Integer(0)); // BOOL -> INT
        assert_eq!(evaluator.moo_typeof(&[Value::Null]).unwrap(), Value::Integer(4)); // NULL -> OBJ
    }

    #[test]
    fn test_moo_tostr_simple_values() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test string conversion of various values
        assert_eq!(evaluator.moo_tostr(&[Value::Integer(42)]).unwrap(), Value::String("42".to_string()));
        assert_eq!(evaluator.moo_tostr(&[Value::String("hello".to_string())]).unwrap(), Value::String("hello".to_string()));
        assert_eq!(evaluator.moo_tostr(&[Value::Boolean(true)]).unwrap(), Value::String("1".to_string()));
        assert_eq!(evaluator.moo_tostr(&[Value::Boolean(false)]).unwrap(), Value::String("0".to_string()));
        assert_eq!(evaluator.moo_tostr(&[Value::Null]).unwrap(), Value::String("#-1".to_string()));
    }

    #[test]
    fn test_moo_tostr_multiple_args() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test concatenation of multiple arguments
        let result = evaluator.moo_tostr(&[
            Value::String("Hello ".to_string()),
            Value::String("world ".to_string()),
            Value::Integer(42),
        ]).unwrap();
        
        assert_eq!(result, Value::String("Hello world 42".to_string()));
    }

    #[test]
    fn test_moo_notify() {
        let (mut evaluator, player_id) = create_test_evaluator();
        
        // Test notify function (should return the message)
        let player_obj = Value::Object(player_id);
        let message = Value::String("Test message".to_string());
        
        let result = evaluator.moo_notify(&[player_obj, message.clone()]).unwrap();
        assert_eq!(result, message); // Should return the same message
    }

    #[test]
    fn test_moo_notify_with_moo_id() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test notify with MOO object number
        let moo_obj_num = Value::Integer(1); // Root object should exist
        let message = Value::String("MOO message".to_string());
        
        let result = evaluator.moo_notify(&[moo_obj_num, message.clone()]).unwrap();
        assert_eq!(result, message); // Should return the same message
    }

    #[test]
    fn test_moo_notify_nonexistent_object() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test notify with non-existent MOO object number
        let bad_obj_num = Value::Integer(99999);
        let message = Value::String("This should fail".to_string());
        
        let result = evaluator.moo_notify(&[bad_obj_num, message]);
        assert!(result.is_err()); // Should fail for non-existent object
    }

    #[test]
    fn test_moo_valid_invalid_args() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test with wrong number of arguments
        let result = evaluator.moo_valid(&[]);
        assert!(result.is_err());
        
        let result = evaluator.moo_valid(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
        
        // Test with wrong argument type
        let result = evaluator.moo_valid(&[Value::String("not an object".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_moo_typeof_invalid_args() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test with wrong number of arguments
        let result = evaluator.moo_typeof(&[]);
        assert!(result.is_err());
        
        let result = evaluator.moo_typeof(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }
}