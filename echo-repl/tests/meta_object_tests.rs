use crate::evaluator::meta_object::MetaObject;
use crate::storage::object_store::ObjectId;
use crate::storage::EchoObject;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_object_creation() {
        let obj_id = ObjectId::new();
        let meta_obj = MetaObject::new(obj_id);
        assert_eq!(meta_obj.object_id, obj_id);
        // Add assertions for other default fields if they exist in MetaObject
        // e.g., assert!(meta_obj.properties.is_empty());
        // assert!(meta_obj.verbs.is_empty());
    }

    #[test]
    fn test_echo_object_meta_field() {
        let obj_id = ObjectId::new();
        let meta_obj = MetaObject::new(obj_id);
        let echo_obj = EchoObject {
            id: obj_id,
            parent: None,
            name: "test_object".to_string(),
            properties: HashMap::new(),
            verbs: HashMap::new(),
            queries: HashMap::new(),
            meta: meta_obj.clone(),
        };

        assert_eq!(echo_obj.meta.object_id, obj_id);
    }
}