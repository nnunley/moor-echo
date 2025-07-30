use anyhow::Result;
use sled::Tree;

use crate::storage::ObjectId;

/// Manages various indices for efficient object lookups
pub struct IndexManager {
    /// Object parent -> children mapping
    parent_index: Tree,
    /// Object type/class -> instances mapping  
    type_index: Tree,
    /// Property name -> objects with that property
    property_index: Tree,
    /// Verb name -> objects with that verb
    verb_index: Tree,
}

impl IndexManager {
    pub fn new(db: &sled::Db) -> Result<Self> {
        Ok(Self {
            parent_index: db.open_tree("idx_parent")?,
            type_index: db.open_tree("idx_type")?,
            property_index: db.open_tree("idx_property")?,
            verb_index: db.open_tree("idx_verb")?,
        })
    }

    /// Update parent-child relationships
    pub fn update_parent(&self, child: ObjectId, parent: Option<ObjectId>) -> Result<()> {
        // Remove from old parent if exists
        self.remove_child_from_all_parents(child)?;

        // Add to new parent
        if let Some(parent_id) = parent {
            let key = parent_id.0.as_bytes();
            let mut children = self.get_children(parent_id)?;
            children.push(child);
            let value = bincode::serialize(&children)?;
            self.parent_index.insert(key, value)?;
        }

        Ok(())
    }

    /// Get all children of an object
    pub fn get_children(&self, parent: ObjectId) -> Result<Vec<ObjectId>> {
        let key = parent.0.as_bytes();
        if let Some(value) = self.parent_index.get(key)? {
            let children: Vec<ObjectId> = bincode::deserialize(&value)?;
            Ok(children)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find all descendants recursively
    pub fn get_descendants(&self, parent: ObjectId) -> Result<Vec<ObjectId>> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![parent];

        while let Some(current) = to_visit.pop() {
            let children = self.get_children(current)?;
            descendants.extend(&children);
            to_visit.extend(&children);
        }

        Ok(descendants)
    }

    /// Index an object by its type/class
    pub fn update_type(&self, obj_id: ObjectId, type_name: &str) -> Result<()> {
        let key = type_name.as_bytes();
        let mut objects = self.get_objects_by_type(type_name)?;
        if !objects.contains(&obj_id) {
            objects.push(obj_id);
            let value = bincode::serialize(&objects)?;
            self.type_index.insert(key, value)?;
        }
        Ok(())
    }

    /// Get all objects of a specific type
    pub fn get_objects_by_type(&self, type_name: &str) -> Result<Vec<ObjectId>> {
        let key = type_name.as_bytes();
        if let Some(value) = self.type_index.get(key)? {
            let objects: Vec<ObjectId> = bincode::deserialize(&value)?;
            Ok(objects)
        } else {
            Ok(Vec::new())
        }
    }

    /// Index objects by property names they contain
    pub fn update_properties(&self, obj_id: ObjectId, properties: &[String]) -> Result<()> {
        for prop_name in properties {
            let key = format!("prop:{}", prop_name);
            let mut objects = self.get_objects_with_property(prop_name)?;
            if !objects.contains(&obj_id) {
                objects.push(obj_id);
                let value = bincode::serialize(&objects)?;
                self.property_index.insert(key.as_bytes(), value)?;
            }
        }
        Ok(())
    }

    /// Get all objects that have a specific property
    pub fn get_objects_with_property(&self, prop_name: &str) -> Result<Vec<ObjectId>> {
        let key = format!("prop:{}", prop_name);
        if let Some(value) = self.property_index.get(key.as_bytes())? {
            let objects: Vec<ObjectId> = bincode::deserialize(&value)?;
            Ok(objects)
        } else {
            Ok(Vec::new())
        }
    }

    /// Index objects by verb names they contain
    pub fn update_verbs(&self, obj_id: ObjectId, verbs: &[String]) -> Result<()> {
        for verb_name in verbs {
            let key = format!("verb:{}", verb_name);
            let mut objects = self.get_objects_with_verb(verb_name)?;
            if !objects.contains(&obj_id) {
                objects.push(obj_id);
                let value = bincode::serialize(&objects)?;
                self.verb_index.insert(key.as_bytes(), value)?;
            }
        }
        Ok(())
    }

    /// Get all objects that have a specific verb
    pub fn get_objects_with_verb(&self, verb_name: &str) -> Result<Vec<ObjectId>> {
        let key = format!("verb:{}", verb_name);
        if let Some(value) = self.verb_index.get(key.as_bytes())? {
            let objects: Vec<ObjectId> = bincode::deserialize(&value)?;
            Ok(objects)
        } else {
            Ok(Vec::new())
        }
    }

    fn remove_child_from_all_parents(&self, child: ObjectId) -> Result<()> {
        // This is inefficient but works for now
        // In production, we'd maintain a reverse index
        for item in self.parent_index.iter() {
            let (key, value) = item?;
            let mut children: Vec<ObjectId> = bincode::deserialize(&value)?;
            if children.contains(&child) {
                children.retain(|&id| id != child);
                let new_value = bincode::serialize(&children)?;
                self.parent_index.insert(key, new_value)?;
            }
        }
        Ok(())
    }
}
