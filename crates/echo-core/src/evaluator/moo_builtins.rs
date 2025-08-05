//! MOO-compatible builtin functions
//! 
//! This module implements builtin functions that provide compatibility with
//! LambdaMOO's built-in function set. These functions operate in the MOO
//! object space using MOO object numbers rather than Echo's native ObjectIds.

use crate::evaluator::{Value, Evaluator};
use anyhow::Result;

impl Evaluator {
    /// valid(obj) - Check if a MOO object number is valid (exists)
    /// 
    /// This function operates in the MOO object space, checking if the given
    /// MOO object number exists in our MOO ID mapping.
    /// 
    /// Returns: 1 if the object exists, 0 if it doesn't
    pub fn moo_valid(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("valid() requires exactly one argument"));
        }
        
        let obj_num = match &args[0] {
            Value::Integer(n) => *n,
            Value::Object(obj_id) => {
                // Try to find the MOO number for this ObjectId
                let storage = self.storage.clone();
                let object_store = &storage.objects;
                
                // Look through the MOO ID map to find the number
                for (moo_num, mapped_id) in &object_store.moo_id_map {
                    if mapped_id == obj_id {
                        return Ok(Value::Integer(if object_store.is_valid_moo_id(*moo_num) { 1 } else { 0 }));
                    }
                }
                // If not found in MOO space, it's not valid in MOO terms
                return Ok(Value::Integer(0));
            }
            _ => return Err(anyhow::anyhow!("valid() requires an object number or object reference")),
        };
        
        let storage = self.storage.clone();
        let is_valid = storage.objects.is_valid_moo_id(obj_num);
        
        Ok(Value::Integer(if is_valid { 1 } else { 0 }))
    }
    
    /// typeof(value) - Return the type of a MOO value
    /// 
    /// Returns one of the MOO type constants:
    /// - INT (0) for integers
    /// - FLOAT (1) for floating-point numbers  
    /// - STR (2) for strings
    /// - LIST (3) for lists
    /// - OBJ (4) for objects
    /// - ERR (5) for errors
    pub fn moo_typeof(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("typeof() requires exactly one argument"));
        }
        
        let type_code = match &args[0] {
            Value::Integer(_) => 0, // INT
            Value::Float(_) => 1,   // FLOAT  
            Value::String(_) => 2,  // STR
            Value::List(_) => 3,    // LIST
            Value::Object(_) => 4,  // OBJ
            Value::Boolean(_) => 0, // Booleans are integers in MOO
            Value::Null => 4,       // NULL is represented as object #-1 in MOO
            Value::Map(_) => 3,     // Maps are treated as lists in basic MOO
            Value::Lambda { .. } => 3, // Lambdas are treated as lists in basic MOO
        };
        
        Ok(Value::Integer(type_code))
    }
    
    /// tostr(value, ...) - Convert values to strings and concatenate
    /// 
    /// Converts all given MOO values into strings and returns the concatenation.
    pub fn moo_tostr(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::String(String::new()));
        }
        
        let mut result = String::new();
        
        for value in args {
            let str_repr = match value {
                Value::Integer(n) => n.to_string(),
                Value::Float(f) => f.to_string(),
                Value::String(s) => s.clone(),
                Value::Object(obj_id) => {
                    // Try to find the MOO number for this ObjectId
                    let storage = self.storage.clone();
                    let object_store = &storage.objects;
                    
                    let mut found_moo_num = None;
                    for (moo_num, mapped_id) in &object_store.moo_id_map {
                        if mapped_id == obj_id {
                            found_moo_num = Some(*moo_num);
                            break;
                        }
                    }
                    
                    match found_moo_num {
                        Some(moo_num) => format!("#{}", moo_num),
                        None => format!("{}", obj_id), // Use Echo representation if not in MOO space
                    }
                }
                Value::Boolean(b) => if *b { "1".to_string() } else { "0".to_string() },
                Value::Null => "#-1".to_string(), // NULL as object #-1 in MOO
                Value::List(_) => "{list}".to_string(), // MOO's standard list representation
                Value::Map(_) => "{list}".to_string(), // Maps shown as lists in basic MOO
                Value::Lambda { .. } => "{list}".to_string(), // Lambdas shown as lists in basic MOO
            };
            result.push_str(&str_repr);
        }
        
        Ok(Value::String(result))
    }
    
    /// notify(obj, message) - Send a message to a player object
    /// 
    /// In MOO, this sends a line of text to the player's client.
    /// This implementation routes the message through the UI callback system.
    pub fn moo_notify(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!("notify() requires exactly two arguments"));
        }
        
        // Get the target player object
        let target_player_id = match &args[0] {
            Value::Object(obj_id) => *obj_id,
            Value::Integer(moo_num) => {
                // Convert MOO object number to ObjectId
                let storage = self.storage.clone();
                let object_store = &storage.objects;
                match object_store.resolve_moo_id(*moo_num) {
                    Some(obj_id) => obj_id,
                    None => return Err(anyhow::anyhow!("notify() target object #{} does not exist", moo_num)),
                }
            }
            _ => return Err(anyhow::anyhow!("notify() target must be an object or object number")),
        };
        
        // Convert message to string
        let message = match &args[1] {
            Value::String(s) => s.clone(),
            other => {
                // Convert to string using tostr logic
                let str_args = vec![other.clone()];
                match self.moo_tostr(&str_args)? {
                    Value::String(s) => s,
                    _ => return Err(anyhow::anyhow!("Failed to convert message to string")),
                }
            }
        };
        
        // Send notification through UI callback system
        self.send_ui_event(crate::ui_callback::UiAction::NotifyPlayer {
            player_id: target_player_id,
            message: message.clone(),
        });
        
        // MOO's notify() returns the message that was sent
        Ok(Value::String(message))
    }
    
    /// raise(error_msg) - Throw an exception with the given error message
    /// 
    /// In MOO, this raises an error that can be caught by try/except.
    /// The error message should be a string.
    pub fn moo_raise(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("raise() requires exactly one argument"));
        }
        
        let error_msg = match &args[0] {
            Value::String(s) => s.clone(),
            other => {
                // Convert to string using tostr logic
                let str_args = vec![other.clone()];
                match self.moo_tostr(&str_args)? {
                    Value::String(s) => s,
                    _ => return Err(anyhow::anyhow!("Failed to convert error message to string")),
                }
            }
        };
        
        // Throw the error
        Err(anyhow::anyhow!("{}", error_msg))
    }
}