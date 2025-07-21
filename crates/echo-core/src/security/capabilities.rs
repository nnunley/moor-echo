use crate::storage::ObjectId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    ReadProperty(ObjectId, String),
    WriteProperty(ObjectId, String),
    CallFunction(ObjectId, String),
    CallVerb(ObjectId, String),
    EmitEvent(String),
    ExecuteQuery(ObjectId, String),
    AccessRoom(ObjectId),
    ModifyHealth(ObjectId),
    SystemAccess(String),
}

pub struct CapabilityManager {
    grants: HashMap<ObjectId, HashSet<Capability>>,
    denials: HashMap<ObjectId, HashSet<Capability>>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        CapabilityManager {
            grants: HashMap::new(),
            denials: HashMap::new(),
        }
    }
    
    pub fn check_capability(&self, subject: ObjectId, capability: &Capability) -> bool {
        // Check if explicitly denied
        if let Some(denials) = self.denials.get(&subject) {
            if denials.contains(capability) {
                return false;
            }
        }
        
        // Check if explicitly granted
        if let Some(grants) = self.grants.get(&subject) {
            if grants.contains(capability) {
                return true;
            }
        }
        
        // Default deny
        false
    }

    pub fn grant_capability(&mut self, subject: ObjectId, capability: Capability) {
        self.grants.entry(subject).or_insert_with(HashSet::new).insert(capability);
    }

    pub fn deny_capability(&mut self, subject: ObjectId, capability: Capability) {
        self.denials.entry(subject).or_insert_with(HashSet::new).insert(capability);
    }
}
