use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

use crate::parser::EchoAst;
use crate::storage::{Storage, ObjectId, EchoObject, PropertyValue};
// Remove unused imports - verb execution will be re-implemented when we add MethodCall to rust-sitter grammar

pub struct Evaluator {
    storage: Arc<Storage>,
    environments: DashMap<ObjectId, Environment>,
    current_player: Option<ObjectId>,
}

#[derive(Clone)]
pub struct Environment {
    pub player_id: ObjectId,
    pub variables: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Object(ObjectId),
    List(Vec<Value>),
}

impl Evaluator {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            environments: DashMap::new(),
            current_player: None,
        }
    }
    
    pub fn create_player(&mut self, name: &str) -> Result<ObjectId> {
        // Create a new player object extending from $player (or $root for now)
        let player_id = ObjectId::new();
        let player = EchoObject {
            id: player_id,
            parent: Some(ObjectId::root()),
            name: format!("player_{}", name),
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), PropertyValue::String(name.to_string()));
                props.insert("location".to_string(), PropertyValue::Object(ObjectId::root()));
                props
            },
            verbs: HashMap::new(),
            queries: HashMap::new(),
            event_handlers: vec![],
        };
        
        self.storage.objects.store(player)?;
        
        // Create environment for the player
        let env = Environment {
            player_id,
            variables: HashMap::new(),
        };
        self.environments.insert(player_id, env);
        
        Ok(player_id)
    }
    
    pub fn switch_player(&mut self, player_id: ObjectId) -> Result<()> {
        // Verify player exists
        self.storage.objects.get(player_id)?;
        self.current_player = Some(player_id);
        Ok(())
    }
    
    pub fn current_player(&self) -> Option<ObjectId> {
        self.current_player
    }
    
    pub fn eval(&mut self, ast: &EchoAst) -> Result<Value> {
        let player_id = self.current_player
            .ok_or_else(|| anyhow!("No player selected"))?;
            
        self.eval_with_player(ast, player_id)
    }
    
    pub fn eval_with_player(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        match ast {
            EchoAst::Number(n) => Ok(Value::Integer(*n)),
            EchoAst::String(s) => Ok(Value::String(s.clone())),
            EchoAst::Identifier(s) => {
                // Look up variable in player's environment
                if let Some(env) = self.environments.get(&player_id) {
                    if let Some(value) = env.variables.get(s) {
                        Ok(value.clone())
                    } else {
                        Err(anyhow!("Undefined variable: {}", s))
                    }
                } else {
                    Err(anyhow!("No environment for player"))
                }
            }
            EchoAst::Add { left, right, .. } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    _ => Err(anyhow!("Type error in addition")),
                }
            }
            EchoAst::PropertyAccess { object, property, .. } => {
                let obj_val = self.eval_with_player(object, player_id)?;
                
                if let Value::Object(obj_id) = obj_val {
                    let obj = self.storage.objects.get(obj_id)?;
                    
                    // Extract property name from identifier
                    let prop_name = match property.as_ref() {
                        EchoAst::Identifier(s) => s,
                        _ => return Err(anyhow!("Property access must use identifier")),
                    };
                    
                    if let Some(prop_val) = obj.properties.get(prop_name) {
                        Ok(property_value_to_value(prop_val.clone())?)
                    } else {
                        Err(anyhow!("Property '{}' not found on object", prop_name))
                    }
                } else {
                    Err(anyhow!("Property access on non-object"))
                }
            }
            EchoAst::Let { name, value, .. } => {
                let val = self.eval_with_player(value, player_id)?;
                
                // Extract name from identifier
                let var_name = match name.as_ref() {
                    EchoAst::Identifier(s) => s,
                    _ => return Err(anyhow!("Let statement must use identifier for variable name")),
                };
                
                // Store in player's environment
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(var_name.clone(), val.clone());
                });
                
                Ok(val)
            }
            EchoAst::ObjectDef { name, members, .. } => {
                // Create new object
                let obj_id = ObjectId::new();
                
                // Extract object name from identifier
                let obj_name = match name.as_ref() {
                    EchoAst::Identifier(s) => s,
                    _ => return Err(anyhow!("Object definition must use identifier for name")),
                };
                
                let mut properties = HashMap::new();
                let mut verbs = HashMap::new();
                
                // Process object members
                for member in members {
                    match member {
                        EchoAst::PropertyDef { name: prop_name, value, .. } => {
                            let prop_name_str = match prop_name.as_ref() {
                                EchoAst::Identifier(s) => s,
                                _ => return Err(anyhow!("Property name must be identifier")),
                            };
                            
                            let val = self.eval_with_player(value, player_id)?;
                            properties.insert(prop_name_str.clone(), value_to_property_value(val)?);
                        }
                        _ => {} // Other members not implemented yet
                    }
                }
                
                let obj = EchoObject {
                    id: obj_id,
                    parent: Some(ObjectId::root()),
                    name: obj_name.clone(),
                    properties,
                    verbs,
                    queries: HashMap::new(),
                    event_handlers: vec![],
                };
                
                self.storage.objects.store(obj)?;
                
                // Store the object reference by name for easy access
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(obj_name.clone(), Value::Object(obj_id));
                });
                
                Ok(Value::Object(obj_id))
            }
            EchoAst::PropertyDef { .. } => {
                // Property definitions should only appear inside objects
                Err(anyhow!("Property definition outside of object context"))
            }
            // All current rust-sitter AST variants are handled above
            // Will add more variants as we expand the grammar
        }
    }
}

fn value_to_property_value(val: Value) -> Result<PropertyValue> {
    match val {
        Value::Null => Ok(PropertyValue::Null),
        Value::Boolean(b) => Ok(PropertyValue::Boolean(b)),
        Value::Integer(i) => Ok(PropertyValue::Integer(i)),
        Value::Float(f) => Ok(PropertyValue::Float(f)),
        Value::String(s) => Ok(PropertyValue::String(s)),
        Value::Object(id) => Ok(PropertyValue::Object(id)),
        Value::List(items) => {
            let prop_items: Result<Vec<_>> = items.into_iter()
                .map(value_to_property_value)
                .collect();
            Ok(PropertyValue::List(prop_items?))
        }
    }
}

fn property_value_to_value(prop_val: PropertyValue) -> Result<Value> {
    match prop_val {
        PropertyValue::Null => Ok(Value::Null),
        PropertyValue::Boolean(b) => Ok(Value::Boolean(b)),
        PropertyValue::Integer(i) => Ok(Value::Integer(i)),
        PropertyValue::Float(f) => Ok(Value::Float(f)),
        PropertyValue::String(s) => Ok(Value::String(s)),
        PropertyValue::Object(id) => Ok(Value::Object(id)),
        PropertyValue::List(items) => {
            let val_items: Result<Vec<_>> = items.into_iter()
                .map(property_value_to_value)
                .collect();
            Ok(Value::List(val_items?))
        }
        PropertyValue::Map(_) => {
            // For now, just return null for maps - full implementation would convert to Value::Map
            Ok(Value::Null)
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Object(id) => write!(f, "{}", id),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}