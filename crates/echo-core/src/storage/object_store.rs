use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use crate::evaluator::meta_object::MetaObject;

/// Object ID type - similar to MOO's object numbers but using UUIDs for distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub Uuid);

impl ObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// System object #0 in MOO becomes a well-known UUID
    pub fn system() -> Self {
        Self(Uuid::from_u128(0))
    }
    
    /// Root object #1 in MOO
    pub fn root() -> Self {
        Self(Uuid::from_u128(1))
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", &self.0.to_string()[..8])
    }
}

/// Echo object representation - stores code, properties, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoObject {
    pub id: ObjectId,
    pub parent: Option<ObjectId>,
    pub name: String,
    pub properties: HashMap<String, PropertyValue>,
    pub verbs: HashMap<String, VerbDefinition>,
    pub queries: HashMap<String, String>,
    pub meta: crate::evaluator::meta_object::MetaObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Object(ObjectId),
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbDefinition {
    pub name: String,
    pub signature: VerbSignature,
    pub code: String,  // Source code for display
    pub ast: Vec<crate::ast::EchoAst>,  // The actual AST to execute
    pub params: Vec<crate::ast::Parameter>,  // Parameters for the verb
    pub permissions: VerbPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbSignature {
    pub dobj: String,
    pub prep: String,
    pub iobj: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_pattern: String,
    pub condition: Option<String>,
    pub code: String,
}

/// Object store using Sled for persistence
pub struct ObjectStore {
    db: Db,
    objects: sled::Tree,
    indices: sled::Tree,
}

impl ObjectStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(path)?;
        Self::new_with_db(db)
    }
    
    pub fn new_with_db(db: Db) -> Result<Self> {
        let objects = db.open_tree("objects")?;
        let indices = db.open_tree("indices")?;
        
        let store = Self { db, objects, indices };
        
        // Initialize system objects if they don't exist
        store.init_system_objects()?;
        
        Ok(store)
    }
    
    fn init_system_objects(&self) -> Result<()> {
        // Create system object #0 if it doesn't exist
        let system_id = ObjectId::system();
        if self.get(system_id).is_err() {
            let mut system_properties = HashMap::new();
            // Initialize #0.system to point to #0 itself
            system_properties.insert("system".to_string(), PropertyValue::Object(system_id));
            
            let system_obj = EchoObject {
                id: system_id,
                parent: None,
                name: "$system".to_string(),
                properties: system_properties,
                verbs: HashMap::new(),
                queries: HashMap::new(),
                meta: MetaObject::new(system_id),
            };
            self.store(system_obj)?;
        }
        
        // Create root object #1 if it doesn't exist
        let root_id = ObjectId::root();
        if self.get(root_id).is_err() {
            let root_obj = EchoObject {
                id: root_id,
                parent: Some(system_id),
                name: "$root".to_string(),
                properties: HashMap::new(),
                verbs: HashMap::new(),
                queries: HashMap::new(),
                meta: MetaObject::new(root_id),
            };
            self.store(root_obj)?;
        }
        
        Ok(())
    }
    
    pub fn store(&self, obj: EchoObject) -> Result<()> {
        let key = obj.id.0.as_bytes();
        let value = bincode::serialize(&obj)?;
        self.objects.insert(key, value)?;
        
        // Update name index
        let name_key = format!("name:{}", obj.name);
        self.indices.insert(name_key.as_bytes(), obj.id.0.as_bytes())?;
        
        self.db.flush()?;
        Ok(())
    }
    
    pub fn get(&self, id: ObjectId) -> Result<EchoObject> {
        let key = id.0.as_bytes();
        let value = self.objects.get(key)?
            .ok_or_else(|| anyhow!("Object {} not found", id))?;
        let obj: EchoObject = bincode::deserialize(&value)?;
        Ok(obj)
    }
    
    pub fn find_by_name(&self, name: &str) -> Result<Option<ObjectId>> {
        let name_key = format!("name:{}", name);
        if let Some(id_bytes) = self.indices.get(name_key.as_bytes())? {
            let id = Uuid::from_slice(&id_bytes)?;
            Ok(Some(ObjectId(id)))
        } else {
            Ok(None)
        }
    }
    
    pub fn delete(&self, id: ObjectId) -> Result<()> {
        if let Ok(obj) = self.get(id) {
            // Remove from name index
            let name_key = format!("name:{}", obj.name);
            self.indices.remove(name_key.as_bytes())?;
            
            // Remove object
            let key = id.0.as_bytes();
            self.objects.remove(key)?;
            
            self.db.flush()?;
        }
        Ok(())
    }
    
    pub fn list_all(&self) -> Result<Vec<ObjectId>> {
        let mut ids = Vec::new();
        for item in self.objects.iter() {
            let (key, _) = item?;
            let id = Uuid::from_slice(&key)?;
            ids.push(ObjectId(id));
        }
        Ok(ids)
    }
    
    pub fn estimated_size(&self) -> Result<u64> {
        // Return the estimated size of the database
        Ok(self.db.size_on_disk()?)
    }
}