use crate::storage::object_store::ObjectId;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Unique identifier for green threads (lightweight cooperative tasks)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GreenThreadId(pub u64);

impl GreenThreadId {
    /// Generate a new unique green thread ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        GreenThreadId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaObject {
    pub object_id: ObjectId,
    /// List of active green threads associated with this object
    pub active_tasks: Vec<GreenThreadId>,
    /// Metadata about object properties (will be expanded as features are implemented)
    pub properties_meta: HashMap<String, PropertyMetadata>,
    /// Metadata about object verbs (will be expanded as features are implemented)  
    pub verbs_meta: HashMap<String, VerbMetadata>,
    /// Metadata about object events (will be expanded as features are implemented)
    pub events_meta: HashMap<String, EventMetadata>,
    /// Metadata about object queries (will be expanded as features are implemented)
    pub queries_meta: HashMap<String, QueryMetadata>,
}

/// Metadata about an object property
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyMetadata {
    pub name: String,
    pub readable: bool,
    pub writable: bool,
    pub inheritable: bool,
}

/// Metadata about an object verb  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbMetadata {
    pub name: String,
    pub callable: bool,
    pub inheritable: bool,
}

/// Metadata about an object event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub name: String,
    pub emittable: bool,
    pub inheritable: bool,
}

/// Metadata about an object query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub name: String,
    pub executable: bool,
    pub inheritable: bool,
}

impl MetaObject {
    pub fn new(object_id: ObjectId) -> Self {
        MetaObject {
            object_id,
            active_tasks: Vec::new(),
            properties_meta: HashMap::new(),
            verbs_meta: HashMap::new(), 
            events_meta: HashMap::new(),
            queries_meta: HashMap::new(),
        }
    }

    /// Get reference to active tasks for this object
    pub fn get_active_tasks(&self) -> &[GreenThreadId] {
        &self.active_tasks
    }

    /// Add a new active task to this object
    pub fn add_active_task(&mut self, task_id: GreenThreadId) {
        self.active_tasks.push(task_id);
    }

    /// Remove a task from this object's active tasks
    pub fn remove_active_task(&mut self, task_id: &GreenThreadId) {
        self.active_tasks.retain(|id| id != task_id);
    }
}