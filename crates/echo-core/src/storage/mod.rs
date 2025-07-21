use anyhow::Result;
use std::path::Path;

/// Storage-related errors
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    /// Object not found
    #[error("Object not found: {id}")]
    ObjectNotFound { id: String },
    
    /// Generic storage error
    #[error("Storage error: {message}")]
    Generic { message: String },
}

pub mod object_store;
pub mod event_store;
pub mod index;

pub use object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};
pub use event_store::{EventStore, EchoEvent, EventId, EventPattern, Subscription};
pub use index::IndexManager;


/// Complete storage system combining objects, events, and indices
pub struct Storage {
    pub objects: ObjectStore,
    pub events: EventStore,
    pub indices: IndexManager,
}

impl Storage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(&path)?;
        let objects = ObjectStore::new_with_db(db.clone())?;
        let events = EventStore::new(&db)?;
        let indices = IndexManager::new(&db)?;
        
        Ok(Self {
            objects,
            events,
            indices,
        })
    }
    
    /// Get estimated size of the database in bytes
    pub fn estimated_size(&self) -> Result<u64> {
        self.objects.estimated_size()
    }
}

/// Initialize the storage subsystem
pub async fn init_storage(path: impl AsRef<Path>) -> Result<Storage> {
    Storage::new(path)
}