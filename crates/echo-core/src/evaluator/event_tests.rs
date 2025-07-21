use super::*;
use crate::parser::create_parser;

#[test]
fn test_emit_statement_basic() {
    // Test basic emit without arguments
    let storage = Storage::new("test_emit_basic").unwrap();
    let mut evaluator = Evaluator::new(Arc::new(storage));
    let player_id = evaluator.create_player(&format!("test_player_{}", uuid::Uuid::new_v4())).unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse("emit startup").unwrap();
    
    // This should execute without error
    let result = evaluator.eval(&ast).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_emit_statement_with_args() {
    // Test emit with arguments
    let storage = Storage::new("test_emit_args").unwrap();
    let mut evaluator = Evaluator::new(Arc::new(storage));
    let player_id = evaluator.create_player(&format!("test_player_{}", uuid::Uuid::new_v4())).unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    let mut parser = create_parser("echo").unwrap();
    
    // Test emit with single argument
    let ast = parser.parse("emit player_moved(\"north\")").unwrap();
    let result = evaluator.eval(&ast).unwrap();
    assert_eq!(result, Value::Null);
    
    // Test emit with multiple arguments
    let ast = parser.parse("emit damage_taken(25, \"goblin\")").unwrap();
    let result = evaluator.eval(&ast).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_object_event_handler() {
    // Test object with event handler
    let storage = Storage::new("test_event_handler").unwrap();
    let mut evaluator = Evaluator::new(Arc::new(storage));
    let player_id = evaluator.create_player(&format!("test_player_{}", uuid::Uuid::new_v4())).unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    let mut parser = create_parser("echo").unwrap();
    
    // Create object with event handler
    let object_code = r#"
    object TestObject
        property count = 0
        
        event increment(amount)
            this.count = this.count + amount
        endevent
    endobject
    "#;
    
    let ast = parser.parse(object_code).unwrap();
    let result = evaluator.eval(&ast).unwrap();
    
    // The object should be created
    match result {
        Value::Object(obj_id) => {
            // Verify the object has the event handler registered
            let obj = evaluator.storage.objects.get(obj_id).unwrap();
            assert!(obj.properties.contains_key("__event_increment"));
            
            // Verify event system has the handler registered
            let event_names = evaluator.event_system.get_event_names();
            assert!(event_names.contains(&"increment".to_string()));
        }
        _ => panic!("Expected object value"),
    }
}

#[test]
fn test_event_system_handler_count() {
    // Test that handlers are properly registered
    let storage = Storage::new("test_handler_count").unwrap();
    let mut evaluator = Evaluator::new(Arc::new(storage));
    let player_id = evaluator.create_player(&format!("test_player_{}", uuid::Uuid::new_v4())).unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    let mut parser = create_parser("echo").unwrap();
    
    // Initial handler count should be 0
    assert_eq!(evaluator.event_system.handler_count(), 0);
    
    // Create object with multiple event handlers
    let object_code = r#"
    object GameEntity
        event damage_taken(amount)
            print("Damage: " + amount)
        endevent
        
        event player_moved(direction)
            print("Moved: " + direction)
        endevent
    endobject
    "#;
    
    let ast = parser.parse(object_code).unwrap();
    evaluator.eval(&ast).unwrap();
    
    // Should have 2 handlers registered
    assert_eq!(evaluator.event_system.handler_count(), 2);
    
    // Event names should be registered
    let event_names = evaluator.event_system.get_event_names();
    assert!(event_names.contains(&"damage_taken".to_string()));
    assert!(event_names.contains(&"player_moved".to_string()));
}