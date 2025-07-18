use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

use crate::parser::{EchoAst, BinaryOperator, ObjectMember};
use crate::storage::{Storage, ObjectId, EchoObject, PropertyValue};

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
            EchoAst::Integer(n) => Ok(Value::Integer(*n)),
            EchoAst::Float(f) => Ok(Value::Float(*f)),
            EchoAst::String(s) => Ok(Value::String(s.clone())),
            EchoAst::Boolean(b) => Ok(Value::Boolean(*b)),
            EchoAst::Null => Ok(Value::Null),
            
            EchoAst::Identifier(name) => {
                // Look up variable in player's environment
                if let Some(env) = self.environments.get(&player_id) {
                    if let Some(value) = env.variables.get(name) {
                        Ok(value.clone())
                    } else {
                        Err(anyhow!("Undefined variable: {}", name))
                    }
                } else {
                    Err(anyhow!("No environment for player"))
                }
            }
            
            EchoAst::BinaryOp { op, left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (op, &left_val, &right_val) {
                    (BinaryOperator::Add, Value::Integer(l), Value::Integer(r)) => {
                        Ok(Value::Integer(l + r))
                    }
                    (BinaryOperator::Sub, Value::Integer(l), Value::Integer(r)) => {
                        Ok(Value::Integer(l - r))
                    }
                    (BinaryOperator::Mul, Value::Integer(l), Value::Integer(r)) => {
                        Ok(Value::Integer(l * r))
                    }
                    (BinaryOperator::Div, Value::Integer(l), Value::Integer(r)) => {
                        if *r == 0 {
                            Err(anyhow!("Division by zero"))
                        } else {
                            Ok(Value::Integer(l / r))
                        }
                    }
                    _ => Err(anyhow!("Type error in binary operation")),
                }
            }
            
            EchoAst::Let { name, value } => {
                let val = self.eval_with_player(value, player_id)?;
                
                // Store in player's environment
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(name.clone(), val.clone());
                });
                
                Ok(val)
            }
            
            EchoAst::ObjectDef { name, parent, members } => {
                // Create new object
                let obj_id = ObjectId::new();
                let parent_id = if let Some(parent_name) = parent {
                    // Look up parent object
                    self.storage.objects.find_by_name(parent_name)?
                        .unwrap_or(ObjectId::root())
                } else {
                    ObjectId::root()
                };
                
                let mut properties = HashMap::new();
                for member in members {
                    if let ObjectMember::Property { name: prop_name, value } = member {
                        if let Some(ast_val) = value {
                            let val = self.eval_with_player(ast_val, player_id)?;
                            properties.insert(prop_name.clone(), value_to_property_value(val)?);
                        }
                    }
                }
                
                let obj = EchoObject {
                    id: obj_id,
                    parent: Some(parent_id),
                    name: name.clone(),
                    properties,
                    verbs: HashMap::new(),
                    queries: HashMap::new(),
                    event_handlers: vec![],
                };
                
                self.storage.objects.store(obj)?;
                Ok(Value::Object(obj_id))
            }
            
            _ => Err(anyhow!("Not implemented: {:?}", ast)),
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