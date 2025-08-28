//! Common MOO database structures used by all parsers and tools

use std::collections::HashMap;

/// Database version representation
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseVersion {
    /// Simple numeric version (e.g., "4" - original LambdaMOO format)
    Numeric(i32),
    /// Dialect with semantic version (e.g., "LambdaMOO-1.8.1", "ToastStunt-2.7.0")
    Dialect {
        name: String,
        major: u32,
        minor: u32,
        patch: u32,
    },
    /// Arbitrary string version (fallback for unknown formats)
    Other(String),
}

impl DatabaseVersion {
    /// Convert to a simple numeric version for compatibility
    pub fn as_numeric(&self) -> i32 {
        match self {
            DatabaseVersion::Numeric(n) => *n,
            DatabaseVersion::Dialect { major, .. } => *major as i32,
            DatabaseVersion::Other(_) => 4, // Default to version 4
        }
    }
}

/// Object ID type
pub type ObjectId = i64;

/// Represents a MOO value
#[derive(Debug, Clone, PartialEq)]
pub enum MooValue {
    Int(i64),
    Obj(ObjectId),
    Str(String),
    Err(i64),
    List(Vec<MooValue>),
    Clear,
    None,
    Float(f64),
    Map(Vec<(MooValue, MooValue)>),
}

/// Represents a verb definition
#[derive(Debug, Clone)]
pub struct MooVerb {
    pub name: String,
    pub owner: ObjectId,
    pub perms: i64,
    pub prep: i64,
}

/// Represents a property definition
#[derive(Debug, Clone)]
pub struct MooProperty {
    pub name: String,
}

/// Represents a property value
#[derive(Debug, Clone)]
pub struct MooPropertyValue {
    pub value: MooValue,
    pub owner: ObjectId,
    pub perms: i64,
}

/// Represents a single object in the database
#[derive(Debug, Clone)]
pub struct MooObject {
    pub id: ObjectId,
    pub name: String,
    pub flags: i64,
    pub owner: ObjectId,
    pub location: ObjectId,
    pub contents: ObjectId,
    pub next: ObjectId,
    pub parent: ObjectId,
    pub child: ObjectId,
    pub sibling: ObjectId,
    pub verbs: Vec<MooVerb>,
    pub properties: Vec<MooProperty>,
    pub property_values: Vec<MooPropertyValue>,
    pub is_recycled: bool,
}

impl Default for MooObject {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            flags: 0,
            owner: 0,
            location: 0,
            contents: 0,
            next: 0,
            parent: 0,
            child: 0,
            sibling: 0,
            verbs: Vec::new(),
            properties: Vec::new(),
            property_values: Vec::new(),
            is_recycled: false,
        }
    }
}

/// Complete MOO database representation
#[derive(Debug)]
pub struct MooDatabase {
    pub name: String,
    pub path: String,
    pub version: DatabaseVersion,
    pub total_objects: i64,
    pub total_verbs: i64,
    pub total_players: i64,
    pub players: Vec<ObjectId>,
    pub objects: HashMap<ObjectId, MooObject>,
    pub verb_programs: HashMap<(ObjectId, String), String>,
}

// Type constants from LambdaMOO structures.h
pub const TYPE_INT: i64 = 0;
pub const TYPE_OBJ: i64 = 1;
pub const TYPE_STR: i64 = 2;
pub const TYPE_ERR: i64 = 3;
pub const TYPE_LIST: i64 = 4;
pub const TYPE_CLEAR: i64 = 5;
pub const TYPE_NONE: i64 = 6;
pub const TYPE_CATCH: i64 = 7;
pub const TYPE_FINALLY: i64 = 8;
pub const TYPE_FLOAT: i64 = 9;
pub const TYPE_MAP: i64 = 12; // Extension in newer MOO variants