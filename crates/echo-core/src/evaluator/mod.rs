#![allow(dead_code)] // Many methods are kept for future use and API completeness
#![allow(clippy::excessive_nesting)] // Complex control flow in evaluator requires deep nesting

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use errors::EvaluatorError;

use crate::{
    ast::{
        BindingPattern, BindingPatternElement, BindingType, EchoAst, LValue, LambdaParam,
        ObjectMember,
    },
    storage::{EchoObject, ObjectId, PropertyValue, Storage},
    ui_callback::{UiAction, UiEvent, UiEventCallback},
};
// TODO: Re-enable when VerbDef is added back to grammar
// use crate::storage::object_store::{VerbDefinition, VerbPermissions,
// VerbSignature};

// Core evaluator modules
pub mod errors;
pub mod event_system;
pub mod events;
pub mod meta_object;

// JIT compiler module
#[cfg(feature = "jit")]
pub mod jit;
pub use events::{EventDefinition, EventHandler, EventInstance, EventRegistry};
#[cfg(feature = "jit")]
pub use jit::{JitEvaluator, JitStats};
// Export core types
pub use meta_object::{
    EventMetadata, GreenThreadId, MetaObject, PropertyMetadata, QueryMetadata, VerbMetadata,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lambda_tests;

#[cfg(test)]
mod player_tests;

#[cfg(test)]
mod jit_tests;

#[cfg(test)]
mod jit_arithmetic_tests;

#[cfg(test)]
mod jit_literal_tests;

#[cfg(test)]
mod jit_comparison_tests;

#[cfg(test)]
mod jit_logical_tests;

#[cfg(test)]
mod jit_variable_tests;

#[cfg(test)]
mod jit_control_flow_tests;

#[cfg(test)]
mod jit_collections_tests;

#[cfg(test)]
mod jit_access_tests;

#[cfg(test)]
mod jit_function_tests;

#[cfg(test)]
mod jit_reference_tests;

#[cfg(test)]
mod jit_assignment_tests;

#[cfg(test)]
mod jit_block_tests;

#[cfg(test)]
mod jit_program_tests;

#[cfg(test)]
mod jit_match_tests;

#[cfg(test)]
mod jit_try_catch_tests;

#[cfg(test)]
mod jit_integration_test;

#[cfg(test)]
mod mop_source_tests;

#[cfg(test)]
mod verb_tests;

#[cfg(test)]
mod object_ref_tests;

#[cfg(test)]
mod sanity_tests;

#[cfg(test)]
mod event_tests;

// Always available trait
pub trait EvaluatorTrait {
    fn create_player(&mut self, name: &str) -> Result<ObjectId>;
    fn switch_player(&mut self, player_id: ObjectId) -> Result<()>;
    fn current_player(&self) -> Option<ObjectId>;
    fn eval(&mut self, ast: &EchoAst) -> Result<Value>;
    fn eval_with_player(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value>;
}

pub struct Evaluator {
    storage: Arc<Storage>,
    environments: DashMap<ObjectId, Environment>,
    current_player: Option<ObjectId>,
    event_system: Arc<event_system::EventSystem>,
    ui_callback: Option<UiEventCallback>,
}

#[derive(Clone)]
pub struct Environment {
    pub player_id: ObjectId,
    pub variables: HashMap<String, Value>,
    pub const_bindings: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Object(ObjectId),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Lambda {
        params: Vec<LambdaParam>,
        body: crate::ast::EchoAst,
        captured_env: HashMap<String, Value>,
    },
}

/// Control flow result for handling break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None(Value),
    Break(Option<String>),
    Continue(Option<String>),
    Return(Value),
}

/// Arithmetic operation types
#[derive(Debug, Clone, Copy, PartialEq)]
enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

/// Comparison operation types
#[derive(Debug, Clone, Copy, PartialEq)]
enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

impl ControlFlow {
    fn into_value(self) -> Result<Value> {
        match self {
            ControlFlow::None(v) => Ok(v),
            ControlFlow::Break(label) => {
                if let Some(l) = label {
                    Err(anyhow!(
                        "Unexpected break with label '{}' outside of loop",
                        l
                    ))
                } else {
                    Err(anyhow!("Unexpected break outside of loop"))
                }
            }
            ControlFlow::Continue(label) => {
                if let Some(l) = label {
                    Err(anyhow!(
                        "Unexpected continue with label '{}' outside of loop",
                        l
                    ))
                } else {
                    Err(anyhow!("Unexpected continue outside of loop"))
                }
            }
            ControlFlow::Return(_v) => Err(anyhow!("Unexpected return outside of function")),
        }
    }
}

/// Represents a call frame in the execution stack for error handling and
/// debugging
#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    pub function_name: String,
    pub object_id: Option<ObjectId>,
    pub line_number: Option<usize>,
    pub local_variables: HashMap<String, Value>,
}

impl Evaluator {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            environments: DashMap::new(),
            current_player: None,
            event_system: Arc::new(event_system::EventSystem::new()),
            ui_callback: None,
        }
    }

    /// Set a UI event callback
    pub fn set_ui_callback(&mut self, callback: UiEventCallback) {
        self.ui_callback = Some(callback);
    }

    /// Send a UI event if a callback is registered
    fn send_ui_event(&self, action: UiAction) {
        if let Some(ref callback) = self.ui_callback {
            let event = UiEvent {
                action: action.clone(),
                target: match &action {
                    UiAction::Clear => "dynamic_ui".to_string(),
                    UiAction::AddButton { id, .. } => id.clone(),
                    UiAction::AddText { id, .. } => id.clone(),
                    UiAction::AddDiv { id, .. } => id.clone(),
                    UiAction::Update { id, .. } => id.clone(),
                },
                data: HashMap::new(),
            };
            callback(event);
        }
    }

    pub fn get_current_environment(&self) -> Option<Environment> {
        self.current_player
            .and_then(|id| self.environments.get(&id).map(|e| e.clone()))
    }

    pub fn get_environment(&self, player_id: ObjectId) -> Option<Environment> {
        self.environments.get(&player_id).map(|e| e.clone())
    }

    pub fn reset_player_environment(&mut self, player_id: ObjectId) -> Result<()> {
        // Create a new empty environment for the player
        let env = Environment {
            player_id,
            variables: HashMap::new(),
            const_bindings: HashSet::new(),
        };
        self.environments.insert(player_id, env);
        Ok(())
    }

    /// Get the environment variables for a player
    pub fn get_player_environment(&self, player_id: ObjectId) -> Option<Vec<(String, Value)>> {
        self.environments.get(&player_id).map(|env| {
            env.variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
    }

    pub fn create_player(&mut self, name: &str) -> Result<ObjectId> {
        // Check if a player with this name already exists in the registry
        let system_obj = self.storage.objects.get(ObjectId::system())?;
        if let Some(PropertyValue::Map(player_registry)) =
            system_obj.properties.get("player_registry")
        {
            if player_registry.contains_key(name) {
                return Err(anyhow!("A player with the name '{}' already exists", name));
            }
        }

        // Create a new player object extending from $player (or $root for now)
        let player_id = ObjectId::new();
        let player = EchoObject {
            id: player_id,
            parent: Some(ObjectId::root()),
            // Don't bind the player to #0 - they're anonymous objects
            name: format!("player_{}", &player_id.0.to_string()[..8]),
            properties: {
                let mut props = HashMap::new();
                // Store the display name as a property that can be changed
                props.insert(
                    "display_name".to_string(),
                    PropertyValue::String(name.to_string()),
                );
                props.insert(
                    "username".to_string(),
                    PropertyValue::String(name.to_string()),
                );
                props.insert(
                    "location".to_string(),
                    PropertyValue::Object(ObjectId::root()),
                );
                props
            },
            verbs: HashMap::new(),
            queries: HashMap::new(),
            meta: MetaObject::new(player_id),
        };

        self.storage.objects.store(player)?;

        // Add the player to the player registry on #0
        self.register_player(name, player_id)?;

        // Create environment for the player
        let env = Environment {
            player_id,
            variables: HashMap::new(),
            const_bindings: HashSet::new(),
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

    /// Register a player in the system player registry
    fn register_player(&self, username: &str, player_id: ObjectId) -> Result<()> {
        let mut system_obj = self.storage.objects.get(ObjectId::system())?;

        // Ensure player_registry exists
        if !system_obj.properties.contains_key("player_registry") {
            system_obj.properties.insert(
                "player_registry".to_string(),
                PropertyValue::Map(HashMap::new()),
            );
        }

        // Add the player to the registry
        if let Some(PropertyValue::Map(registry)) = system_obj.properties.get_mut("player_registry")
        {
            registry.insert(username.to_string(), PropertyValue::Object(player_id));
        }

        self.storage.objects.store(system_obj)?;
        Ok(())
    }

    /// Find a player by username in the registry
    pub fn find_player_by_username(&self, username: &str) -> Result<Option<ObjectId>> {
        let system_obj = self.storage.objects.get(ObjectId::system())?;

        if let Some(PropertyValue::Map(registry)) = system_obj.properties.get("player_registry") {
            if let Some(PropertyValue::Object(player_id)) = registry.get(username) {
                return Ok(Some(*player_id));
            }
        }

        Ok(None)
    }

    /// Change a player's username (ensuring uniqueness)
    pub fn change_player_username(&self, player_id: ObjectId, new_username: &str) -> Result<()> {
        // First check if the new username is already taken
        if let Some(existing_player) = self.find_player_by_username(new_username)? {
            if existing_player != player_id {
                return Err(anyhow!("Username '{}' is already taken", new_username));
            }
        }

        let mut player = self.storage.objects.get(player_id)?;
        let old_username = match player.properties.get("username") {
            Some(PropertyValue::String(name)) => name.clone(),
            _ => return Err(anyhow!("Player has no username property")),
        };

        // Update the player's username property
        player.properties.insert(
            "username".to_string(),
            PropertyValue::String(new_username.to_string()),
        );
        self.storage.objects.store(player)?;

        // Update the registry
        let mut system_obj = self.storage.objects.get(ObjectId::system())?;
        if let Some(PropertyValue::Map(registry)) = system_obj.properties.get_mut("player_registry")
        {
            // Remove old username mapping
            registry.remove(&old_username);
            // Add new username mapping
            registry.insert(new_username.to_string(), PropertyValue::Object(player_id));
        }
        self.storage.objects.store(system_obj)?;

        Ok(())
    }

    pub fn current_player(&self) -> Option<ObjectId> {
        self.current_player
    }

    pub fn current_player_name(&self) -> Option<String> {
        self.current_player.and_then(|id| {
            self.storage.objects.get(id).ok().map(|obj| {
                // Try to get display_name property first, fallback to object name
                if let Some(PropertyValue::String(name)) = obj.properties.get("display_name") {
                    name.clone()
                } else {
                    obj.name.clone()
                }
            })
        })
    }

    pub fn event_system(&self) -> &Arc<event_system::EventSystem> {
        &self.event_system
    }

    pub fn eval(&mut self, ast: &EchoAst) -> Result<Value> {
        let player_id = self
            .current_player
            .ok_or_else(|| anyhow!("No player selected"))?;

        self.eval_with_player(ast, player_id)
    }

    /// Create a new environment for an event handler
    pub fn create_handler_environment(&self, owner: ObjectId) -> Environment {
        Environment {
            player_id: owner,
            variables: HashMap::new(),
            const_bindings: HashSet::new(),
        }
    }

    /// Set a variable in a specific environment
    pub fn set_variable_in_env(
        &mut self,
        env: &Environment,
        name: &str,
        value: Value,
    ) -> Result<()> {
        // Get or create the environment for this player
        let mut env_entry = self
            .environments
            .entry(env.player_id)
            .or_insert_with(|| env.clone());
        env_entry.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Push a new environment onto the environment stack
    pub fn push_environment(&mut self, env: Environment) -> Option<Environment> {
        let player_id = env.player_id;
        let old_env = self.environments.get(&player_id).map(|e| e.clone());
        self.environments.insert(player_id, env);
        old_env
    }

    /// Pop an environment from the stack
    pub fn pop_environment(&mut self, prev_env: Option<Environment>) {
        if let Some(env) = prev_env {
            self.environments.insert(env.player_id, env);
        }
    }

    /// Evaluate program - a sequence of statements
    fn eval_program(&mut self, statements: &[EchoAst], player_id: ObjectId) -> Result<Value> {
        let mut last_result = Value::Null;
        for stmt in statements {
            last_result = self.eval_with_player(stmt, player_id)?;
        }
        Ok(last_result)
    }

    /// Evaluate comparison operation generically
    fn eval_comparison_op(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
        op: ComparisonOp,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        let result = match op {
            ComparisonOp::Equal => self.values_equal(&left_val, &right_val),
            ComparisonOp::NotEqual => !self.values_equal(&left_val, &right_val),
            ComparisonOp::LessThan => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => l < r,
                (Value::Float(l), Value::Float(r)) => l < r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) < *r,
                (Value::Float(l), Value::Integer(r)) => *l < (*r as f64),
                (Value::String(l), Value::String(r)) => l < r,
                _ => {
                    return Err(EvaluatorError::binary_type_error(
                        "less than comparison",
                        left_val.type_name(),
                        right_val.type_name(),
                    )
                    .into())
                }
            },
            ComparisonOp::LessEqual => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => l <= r,
                (Value::Float(l), Value::Float(r)) => l <= r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) <= *r,
                (Value::Float(l), Value::Integer(r)) => *l <= (*r as f64),
                (Value::String(l), Value::String(r)) => l <= r,
                _ => {
                    return Err(EvaluatorError::binary_type_error(
                        "less than or equal comparison",
                        left_val.type_name(),
                        right_val.type_name(),
                    )
                    .into())
                }
            },
            ComparisonOp::GreaterThan => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => l > r,
                (Value::Float(l), Value::Float(r)) => l > r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) > *r,
                (Value::Float(l), Value::Integer(r)) => *l > (*r as f64),
                (Value::String(l), Value::String(r)) => l > r,
                _ => {
                    return Err(EvaluatorError::binary_type_error(
                        "greater than comparison",
                        left_val.type_name(),
                        right_val.type_name(),
                    )
                    .into())
                }
            },
            ComparisonOp::GreaterEqual => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => l >= r,
                (Value::Float(l), Value::Float(r)) => l >= r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) >= *r,
                (Value::Float(l), Value::Integer(r)) => *l >= (*r as f64),
                (Value::String(l), Value::String(r)) => l >= r,
                _ => {
                    return Err(EvaluatorError::binary_type_error(
                        "greater than or equal comparison",
                        left_val.type_name(),
                        right_val.type_name(),
                    )
                    .into())
                }
            },
        };

        Ok(Value::Boolean(result))
    }

    /// Helper to check value equality
    #[allow(clippy::only_used_in_recursion)]
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Integer(l), Value::Integer(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
            (Value::Integer(l), Value::Float(r)) => (*l as f64 - r).abs() < f64::EPSILON,
            (Value::Float(l), Value::Integer(r)) => (l - *r as f64).abs() < f64::EPSILON,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Object(l), Value::Object(r)) => l == r,
            (Value::List(l), Value::List(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }

    fn eval_with_control_flow(
        &mut self,
        ast: &EchoAst,
        player_id: ObjectId,
    ) -> Result<ControlFlow> {
        match ast {
            EchoAst::Break { label } => Ok(ControlFlow::Break(label.clone())),
            EchoAst::Continue { label } => Ok(ControlFlow::Continue(label.clone())),
            EchoAst::Return { value } => {
                let ret_val = if let Some(val) = value {
                    self.eval_with_player(val, player_id)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(ret_val))
            }
            EchoAst::Emit { event_name, args } => {
                // Evaluate all arguments
                let arg_values: Result<Vec<_>> = args
                    .iter()
                    .map(|arg| self.eval_with_player(arg, player_id))
                    .collect();
                let arg_values = arg_values?;

                // Create and emit the event
                let event = event_system::Event {
                    name: event_name.clone(),
                    args: arg_values,
                    emitter: player_id,
                    bubbles: false,    // Default for now
                    cancelable: false, // Default for now
                };

                // Clone the Arc to avoid borrow issues
                let event_system = Arc::clone(&self.event_system);

                // Emit the event through the event system
                let result = event_system.emit(self, event)?;

                println!("EMIT EVENT: {event_name} - Result: {result:?}");

                // Emit returns null (like a void function)
                Ok(ControlFlow::None(Value::Null))
            }
            _ => {
                let value = self.eval_with_player_impl(ast, player_id)?;
                Ok(ControlFlow::None(value))
            }
        }
    }

    pub fn eval_with_player(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        self.eval_with_control_flow(ast, player_id)?.into_value()
    }

    /// Evaluate literal values (numbers, strings, booleans)
    fn eval_literal(&self, ast: &EchoAst) -> Result<Value> {
        match ast {
            EchoAst::Number(n) => Ok(Value::Integer(*n)),
            EchoAst::Float(f) => Ok(Value::Float(*f)),
            EchoAst::String(s) => Ok(Value::String(s.clone())),
            EchoAst::Boolean(b) => Ok(Value::Boolean(*b)),
            _ => unreachable!("eval_literal called with non-literal AST node"),
        }
    }

    /// Resolve identifier values (variables, system identifiers)
    fn eval_identifier(&self, name: &str, player_id: ObjectId) -> Result<Value> {
        // First check if it's an object name bound to #0
        let system_obj = self.storage.objects.get(ObjectId::system())?;
        if let Some(prop_val) = system_obj.properties.get(name) {
            return property_value_to_value(prop_val.clone());
        }

        // Look up variable in player's environment
        if let Some(env) = self.environments.get(&player_id) {
            if let Some(value) = env.variables.get(name) {
                Ok(value.clone())
            } else {
                Err(EvaluatorError::variable_not_found(name).into())
            }
        } else {
            Err(EvaluatorError::Runtime("No environment for player".to_string()).into())
        }
    }

    /// Resolve system property values
    fn eval_system_property(&self, prop_name: &str) -> Result<Value> {
        // $propname resolves to #0.propname property
        let system_obj = self.storage.objects.get(ObjectId::system())?;
        if let Some(prop_val) = system_obj.properties.get(prop_name) {
            Ok(property_value_to_value(prop_val.clone())?)
        } else {
            Err(EvaluatorError::InvalidOperation {
                message: format!("System property '{prop_name}' not found"),
            }
            .into())
        }
    }

    /// Generic numeric binary operation helper
    fn eval_numeric_binop<I, F>(
        &self,
        left: &Value,
        right: &Value,
        int_op: I,
        float_op: F,
        op_name: &str,
    ) -> Result<Value>
    where
        I: Fn(i64, i64) -> i64,
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(int_op(*l, *r))),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(float_op(*l, *r))),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(float_op(*l as f64, *r))),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(float_op(*l, *r as f64))),
            _ => {
                Err(
                    EvaluatorError::binary_type_error(op_name, left.type_name(), right.type_name())
                        .into(),
                )
            }
        }
    }

    /// Evaluate arithmetic operation generically
    fn eval_arithmetic_op(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
        op: ArithmeticOp,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match op {
            ArithmeticOp::Add => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + *r as f64)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{l}{r}"))),
                _ => Err(EvaluatorError::binary_type_error(
                    "addition",
                    left_val.type_name(),
                    right_val.type_name(),
                )
                .into()),
            },
            ArithmeticOp::Subtract => self.eval_numeric_binop(
                &left_val,
                &right_val,
                |a, b| a - b,
                |a, b| a - b,
                "subtraction",
            ),
            ArithmeticOp::Multiply => self.eval_numeric_binop(
                &left_val,
                &right_val,
                |a, b| a * b,
                |a, b| a * b,
                "multiplication",
            ),
            ArithmeticOp::Divide => match (&left_val, &right_val) {
                (_, Value::Integer(0)) => Err(EvaluatorError::DivisionByZero.into()),
                (_, Value::Float(f)) if *f == 0.0 => Err(EvaluatorError::DivisionByZero.into()),
                _ => self.eval_numeric_binop(
                    &left_val,
                    &right_val,
                    |a, b| a / b,
                    |a, b| a / b,
                    "division",
                ),
            },
            ArithmeticOp::Modulo => match (&left_val, &right_val) {
                (Value::Integer(l), Value::Integer(r)) => {
                    if *r == 0 {
                        Err(EvaluatorError::DivisionByZero.into())
                    } else {
                        Ok(Value::Integer(l % r))
                    }
                }
                (Value::Float(l), Value::Float(r)) => {
                    if *r == 0.0 {
                        Err(EvaluatorError::DivisionByZero.into())
                    } else {
                        Ok(Value::Float(l % r))
                    }
                }
                _ => Err(anyhow!("Type error in modulo: operands must be numbers")),
            },
        }
    }

    /// Evaluate addition operation
    fn eval_add(&mut self, left: &EchoAst, right: &EchoAst, player_id: ObjectId) -> Result<Value> {
        self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Add)
    }

    /// Evaluate subtraction operation
    fn eval_subtract(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;
        self.eval_numeric_binop(
            &left_val,
            &right_val,
            |a, b| a - b,
            |a, b| a - b,
            "subtraction",
        )
    }

    /// Evaluate multiplication operation
    fn eval_multiply(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;
        self.eval_numeric_binop(
            &left_val,
            &right_val,
            |a, b| a * b,
            |a, b| a * b,
            "multiplication",
        )
    }

    /// Evaluate division operation
    fn eval_divide(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => {
                if *r == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Value::Integer(l / r))
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Value::Float(l / r))
                }
            }
            (Value::Integer(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Value::Float(*l as f64 / r))
                }
            }
            (Value::Float(l), Value::Integer(r)) => {
                if *r == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Value::Float(l / (*r as f64)))
                }
            }
            _ => Err(anyhow!("Type error in division")),
        }
    }

    /// Evaluate modulo operation
    fn eval_modulo(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => {
                if *r == 0 {
                    Err(anyhow!("Modulo by zero"))
                } else {
                    Ok(Value::Integer(l % r))
                }
            }
            _ => Err(anyhow!("Modulo requires integer operands")),
        }
    }

    /// Generic comparison operation helper
    fn eval_comparison<I, F, S>(
        &self,
        left: &Value,
        right: &Value,
        int_cmp: I,
        float_cmp: F,
        str_cmp: S,
        default_result: bool,
    ) -> Result<Value>
    where
        I: Fn(&i64, &i64) -> bool,
        F: Fn(&f64, &f64) -> bool,
        S: Fn(&str, &str) -> bool,
    {
        let result = match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => int_cmp(l, r),
            (Value::Float(l), Value::Float(r)) => float_cmp(l, r),
            (Value::Integer(l), Value::Float(r)) => float_cmp(&(*l as f64), r),
            (Value::Float(l), Value::Integer(r)) => float_cmp(l, &(*r as f64)),
            (Value::String(l), Value::String(r)) => str_cmp(l, r),
            (Value::Boolean(l), Value::Boolean(r)) if default_result => l == r,
            (Value::Boolean(l), Value::Boolean(r)) if !default_result => l != r,
            (Value::Object(l), Value::Object(r)) if default_result => l == r,
            (Value::Object(l), Value::Object(r)) if !default_result => l != r,
            _ => default_result,
        };
        Ok(Value::Boolean(result))
    }

    /// Evaluate equality comparison
    fn eval_equal(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;
        self.eval_comparison(
            &left_val,
            &right_val,
            |a, b| a == b,
            |a, b| a == b,
            |a, b| a == b,
            false,
        )
    }

    /// Evaluate inequality comparison
    fn eval_not_equal(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;
        self.eval_comparison(
            &left_val,
            &right_val,
            |a, b| a != b,
            |a, b| a != b,
            |a, b| a != b,
            true,
        )
    }

    /// Evaluate less than comparison
    fn eval_less_than(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l < r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Boolean((*l as f64) < *r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Boolean(*l < (*r as f64))),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l < r)),
            _ => Err(anyhow!(
                "Type error in less than comparison: cannot compare {} and {}",
                left_val.type_name(),
                right_val.type_name()
            )),
        }
    }

    /// Evaluate less than or equal comparison
    fn eval_less_equal(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l <= r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Boolean((*l as f64) <= *r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Boolean(*l <= (*r as f64))),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l <= r)),
            _ => Err(anyhow!(
                "Type error in less than or equal comparison: cannot compare {} and {}",
                left_val.type_name(),
                right_val.type_name()
            )),
        }
    }

    /// Evaluate greater than comparison
    fn eval_greater_than(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l > r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Boolean((*l as f64) > *r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Boolean(*l > (*r as f64))),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l > r)),
            _ => Err(anyhow!(
                "Type error in greater than comparison: cannot compare {} and {}",
                left_val.type_name(),
                right_val.type_name()
            )),
        }
    }

    /// Evaluate greater than or equal comparison
    fn eval_greater_equal(
        &mut self,
        left: &EchoAst,
        right: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match (&left_val, &right_val) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l >= r)),
            (Value::Integer(l), Value::Float(r)) => Ok(Value::Boolean((*l as f64) >= *r)),
            (Value::Float(l), Value::Integer(r)) => Ok(Value::Boolean(*l >= (*r as f64))),
            (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l >= r)),
            _ => Err(anyhow!(
                "Type error in greater than or equal comparison: cannot compare {} and {}",
                left_val.type_name(),
                right_val.type_name()
            )),
        }
    }

    /// Evaluate logical AND operation with short-circuit evaluation
    fn eval_and(&mut self, left: &EchoAst, right: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;

        // Short-circuit evaluation
        match left_val {
            Value::Boolean(false) => Ok(Value::Boolean(false)),
            Value::Boolean(true) => {
                let right_val = self.eval_with_player(right, player_id)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(EvaluatorError::unary_type_error(
                        "logical AND (right operand)",
                        "boolean",
                        right_val.type_name(),
                    )
                    .into()),
                }
            }
            _ => Err(EvaluatorError::unary_type_error(
                "logical AND (left operand)",
                "boolean",
                left_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate logical OR operation with short-circuit evaluation
    fn eval_or(&mut self, left: &EchoAst, right: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;

        // Short-circuit evaluation
        match left_val {
            Value::Boolean(true) => Ok(Value::Boolean(true)),
            Value::Boolean(false) => {
                let right_val = self.eval_with_player(right, player_id)?;
                match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(b)),
                    _ => Err(EvaluatorError::unary_type_error(
                        "logical OR (right operand)",
                        "boolean",
                        right_val.type_name(),
                    )
                    .into()),
                }
            }
            _ => Err(EvaluatorError::unary_type_error(
                "logical OR (left operand)",
                "boolean",
                left_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate logical NOT operation
    fn eval_not(&mut self, operand: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let val = self.eval_with_player(operand, player_id)?;
        match val {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(
                EvaluatorError::unary_type_error("logical NOT", "boolean", val.type_name()).into(),
            ),
        }
    }

    /// Evaluate IN operation (membership test)
    fn eval_in(&mut self, left: &EchoAst, right: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let left_val = self.eval_with_player(left, player_id)?;
        let right_val = self.eval_with_player(right, player_id)?;

        match right_val {
            Value::List(items) => {
                // Check if left_val is in the list
                for item in items {
                    if self.values_equal(&left_val, &item) {
                        return Ok(Value::Boolean(true));
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::String(s) => {
                // Check if left_val (as string) is a substring
                match left_val {
                    Value::String(substr) => Ok(Value::Boolean(s.contains(&substr))),
                    _ => Err(EvaluatorError::binary_type_error(
                        "string containment",
                        left_val.type_name(),
                        "string",
                    )
                    .into()),
                }
            }
            _ => Err(EvaluatorError::binary_type_error(
                "membership test",
                left_val.type_name(),
                right_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate unary minus operation
    fn eval_unary_minus(&mut self, operand: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let val = self.eval_with_player(operand, player_id)?;
        match val {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(
                EvaluatorError::unary_type_error("unary minus", "number", val.type_name()).into(),
            ),
        }
    }

    /// Evaluate unary plus operation
    fn eval_unary_plus(&mut self, operand: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let val = self.eval_with_player(operand, player_id)?;
        match val {
            Value::Integer(_) | Value::Float(_) => Ok(val),
            _ => Err(
                EvaluatorError::unary_type_error("unary plus", "number", val.type_name()).into(),
            ),
        }
    }

    /// Evaluate map literal
    fn eval_map(&mut self, entries: &[(String, EchoAst)], player_id: ObjectId) -> Result<Value> {
        let mut map = std::collections::HashMap::new();
        for (key, value_expr) in entries {
            let value = self.eval_with_player(value_expr, player_id)?;
            map.insert(key.clone(), value);
        }
        Ok(Value::Map(map))
    }

    /// Evaluate local assignment (let x = value)
    fn eval_local_assignment(
        &mut self,
        target: &BindingPattern,
        value: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let val = self.eval_with_player(value, player_id)?;
        self.bind_pattern(target, &val, player_id, BindingType::Let)?;
        Ok(val)
    }

    /// Evaluate const assignment (const x = value)
    fn eval_const_assignment(
        &mut self,
        target: &BindingPattern,
        value: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let val = self.eval_with_player(value, player_id)?;
        self.bind_pattern(target, &val, player_id, BindingType::Const)?;
        Ok(val)
    }

    /// Bind a pattern to a value with the given binding type
    fn bind_pattern(
        &mut self,
        pattern: &BindingPattern,
        value: &Value,
        player_id: ObjectId,
        binding_type: BindingType,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Simple variable binding
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(name.clone(), value.clone());

                    // Handle binding type
                    match binding_type {
                        BindingType::Const => {
                            env.const_bindings.insert(name.clone());
                        }
                        BindingType::Let => {
                            // Remove from const set if it was there
                            env.const_bindings.remove(name);
                        }
                        BindingType::None => {
                            // No special handling for reassignment
                        }
                    }
                });
                Ok(())
            }
            BindingPattern::List(patterns) => {
                // List destructuring
                match value {
                    Value::List(values) => {
                        if patterns.len() != values.len() {
                            return Err(EvaluatorError::InvalidOperation {
                                message: format!(
                                    "Pattern length mismatch: expected {}, got {}",
                                    patterns.len(),
                                    values.len()
                                ),
                            }
                            .into());
                        }

                        for (pattern_elem, value) in patterns.iter().zip(values.iter()) {
                            self.bind_pattern_element(
                                pattern_elem,
                                value,
                                player_id,
                                binding_type.clone(),
                            )?;
                        }
                        Ok(())
                    }
                    _ => Err(EvaluatorError::TypeError {
                        operation: "list destructuring".to_string(),
                        expected: "list".to_string(),
                        actual: value.type_name().to_string(),
                    }
                    .into()),
                }
            }
            BindingPattern::Object(_) => {
                // Object destructuring not yet implemented
                Err(EvaluatorError::InvalidOperation {
                    message: "Object destructuring not yet implemented".to_string(),
                }
                .into())
            }
            BindingPattern::Rest(_) => {
                // Rest patterns not yet implemented
                Err(EvaluatorError::InvalidOperation {
                    message: "Rest patterns not yet implemented".to_string(),
                }
                .into())
            }
            BindingPattern::Ignore => {
                // Ignore pattern - do nothing
                Ok(())
            }
        }
    }

    /// Bind a pattern element to a value
    fn bind_pattern_element(
        &mut self,
        element: &BindingPatternElement,
        value: &Value,
        player_id: ObjectId,
        binding_type: BindingType,
    ) -> Result<()> {
        match element {
            BindingPatternElement::Simple(name) => {
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(name.clone(), value.clone());

                    match binding_type {
                        BindingType::Const => {
                            env.const_bindings.insert(name.clone());
                        }
                        BindingType::Let => {
                            env.const_bindings.remove(name);
                        }
                        BindingType::None => {
                            // No special handling
                        }
                    }
                });
                Ok(())
            }
            BindingPatternElement::Optional { name, default } => {
                // For optional elements, use default if value is null
                let actual_value = if matches!(value, Value::Null) {
                    self.eval_with_player(default, player_id)?
                } else {
                    value.clone()
                };

                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(name.clone(), actual_value);

                    match binding_type {
                        BindingType::Const => {
                            env.const_bindings.insert(name.clone());
                        }
                        BindingType::Let => {
                            env.const_bindings.remove(name);
                        }
                        BindingType::None => {
                            // No special handling
                        }
                    }
                });
                Ok(())
            }
            BindingPatternElement::Rest(_) => {
                // Rest elements not yet implemented
                Err(EvaluatorError::InvalidOperation {
                    message: "Rest pattern elements not yet implemented".to_string(),
                }
                .into())
            }
        }
    }

    /// Evaluate if statement
    fn eval_if(
        &mut self,
        condition: &EchoAst,
        then_branch: &[EchoAst],
        else_branch: &Option<Vec<EchoAst>>,
        player_id: ObjectId,
    ) -> Result<Value> {
        let cond_val = self.eval_with_player_impl(condition, player_id)?;

        match cond_val {
            Value::Boolean(true) => {
                // Execute then branch
                let mut last_val = Value::Null;
                for stmt in then_branch {
                    match self.eval_with_control_flow(stmt, player_id)? {
                        ControlFlow::None(v) => last_val = v,
                        flow => return flow.into_value(),
                    }
                }
                Ok(last_val)
            }
            Value::Boolean(false) => {
                // Execute else branch if present
                if let Some(else_stmts) = else_branch {
                    let mut last_val = Value::Null;
                    for stmt in else_stmts {
                        match self.eval_with_control_flow(stmt, player_id)? {
                            ControlFlow::None(v) => last_val = v,
                            flow => return flow.into_value(),
                        }
                    }
                    Ok(last_val)
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Err(EvaluatorError::unary_type_error(
                "if condition",
                "boolean",
                cond_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate while loop
    fn eval_while(
        &mut self,
        condition: &EchoAst,
        body: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        loop {
            let cond_val = self.eval_with_player_impl(condition, player_id)?;

            match cond_val {
                Value::Boolean(false) => break,
                Value::Boolean(true) => {
                    for stmt in body {
                        match self.eval_with_control_flow(stmt, player_id)? {
                            ControlFlow::None(_) => continue,
                            ControlFlow::Break(label) => {
                                if label.is_none() {
                                    // Unlabeled break - exit this loop
                                    return Ok(Value::Null);
                                } else {
                                    // Labeled break - propagate up
                                    return Ok(Value::Null);
                                }
                            }
                            ControlFlow::Continue(label) => {
                                if label.is_none() {
                                    // Unlabeled continue - continue this loop
                                    break;
                                } else {
                                    // Labeled continue - propagate up
                                    break;
                                }
                            }
                            ControlFlow::Return(_val) => {
                                return Err(anyhow!("Unexpected return outside of function"));
                            }
                        }
                    }
                }
                _ => {
                    return Err(EvaluatorError::unary_type_error(
                        "while condition",
                        "boolean",
                        cond_val.type_name(),
                    )
                    .into())
                }
            }
        }
        Ok(Value::Null)
    }

    /// Evaluate for loop
    fn eval_for(
        &mut self,
        variable: &str,
        collection: &EchoAst,
        body: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        let coll_val = self.eval_with_player_impl(collection, player_id)?;

        match coll_val {
            Value::List(items) => {
                'outer: for item in items {
                    // Bind the loop variable
                    self.environments.entry(player_id).and_modify(|env| {
                        env.variables.insert(variable.to_string(), item.clone());
                        // Remove from const set if it was there
                        env.const_bindings.remove(variable);
                    });

                    // Execute loop body
                    for stmt in body {
                        match self.eval_with_control_flow(stmt, player_id)? {
                            ControlFlow::None(_) => continue,
                            ControlFlow::Break(label) => {
                                if label.is_none() {
                                    // Unlabeled break - exit this loop
                                    break 'outer;
                                } else {
                                    // Labeled break - propagate up
                                    return Ok(Value::Null);
                                }
                            }
                            ControlFlow::Continue(label) => {
                                if label.is_none() {
                                    // Unlabeled continue - continue this loop
                                    continue 'outer;
                                } else {
                                    // Labeled continue - propagate up
                                    return Ok(Value::Null);
                                }
                            }
                            ControlFlow::Return(_val) => {
                                return Err(anyhow!("Unexpected return outside of function"));
                            }
                        }
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(EvaluatorError::unary_type_error(
                "for loop collection",
                "list",
                coll_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate index access (obj[index])
    fn eval_index_access(
        &mut self,
        object: &EchoAst,
        index: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        let obj_val = self.eval_with_player(object, player_id)?;
        let index_val = self.eval_with_player(index, player_id)?;

        match (&obj_val, &index_val) {
            (Value::List(items), Value::Integer(i)) => {
                let idx = *i as usize;
                if idx < items.len() {
                    Ok(items[idx].clone())
                } else {
                    Err(EvaluatorError::Runtime(format!(
                        "List index {} out of bounds (length {})",
                        i,
                        items.len()
                    ))
                    .into())
                }
            }
            (Value::Map(map), Value::String(key)) => {
                Ok(map.get(key).cloned().unwrap_or(Value::Null))
            }
            (Value::String(s), Value::Integer(i)) => {
                let idx = *i as usize;
                if idx < s.len() {
                    let ch = s.chars().nth(idx).unwrap_or('\0');
                    Ok(Value::String(ch.to_string()))
                } else {
                    Err(EvaluatorError::Runtime(format!(
                        "String index {} out of bounds (length {})",
                        i,
                        s.len()
                    ))
                    .into())
                }
            }
            _ => Err(EvaluatorError::binary_type_error(
                "index access",
                obj_val.type_name(),
                index_val.type_name(),
            )
            .into()),
        }
    }

    /// Evaluate property access
    fn eval_property_access(
        &mut self,
        object: &EchoAst,
        property: &str,
        player_id: ObjectId,
    ) -> Result<Value> {
        let obj_val = self.eval_with_player(object, player_id)?;

        if let Value::Object(obj_id) = obj_val {
            let obj = self.storage.objects.get(obj_id)?;

            if let Some(prop_val) = obj.properties.get(property) {
                Ok(property_value_to_value(prop_val.clone())?)
            } else {
                Err(anyhow!("Property '{}' not found on object", property))
            }
        } else {
            Err(anyhow!("Property access on non-object"))
        }
    }

    /// Process a property member during object definition.
    ///
    /// Evaluates the property value expression and stores it in the object's
    /// property map. Properties are stored as PropertyValue types which can
    /// be serialized to the object store.
    ///
    /// # Arguments
    /// * `prop_name` - The name of the property being defined
    /// * `value` - The AST expression that defines the property value
    /// * `player_id` - The ID of the player defining the object (for evaluation
    ///   context)
    /// * `properties` - Mutable reference to the property map being built
    ///
    /// # Returns
    /// * `Ok(())` if the property is successfully processed
    /// * `Err` if property evaluation or storage conversion fails
    fn process_property_member(
        &mut self,
        prop_name: &str,
        value: &EchoAst,
        player_id: ObjectId,
        properties: &mut HashMap<String, PropertyValue>,
    ) -> Result<()> {
        let val = self.eval_with_player(value, player_id)?;
        properties.insert(prop_name.to_string(), value_to_property_value(val)?);
        Ok(())
    }

    /// Process a verb member during object definition.
    ///
    /// Creates a VerbDefinition containing the verb's code, parameters, and
    /// permissions. The verb's AST is preserved for future execution, and
    /// source code is reconstructed for debugging and introspection
    /// purposes.
    ///
    /// # Arguments
    /// * `verb_name` - The name of the verb being defined
    /// * `args` - Parameter definitions for the verb
    /// * `body` - The AST statements that make up the verb body
    /// * `permissions` - Optional permission settings for read/write/execute
    ///   access
    /// * `verbs` - Mutable reference to the verb map being built
    ///
    /// # Returns
    /// * `Ok(())` if the verb is successfully processed
    /// * `Err` if verb definition creation fails
    fn process_verb_member(
        &mut self,
        verb_name: &str,
        args: &[crate::ast::Parameter],
        body: &[EchoAst],
        permissions: &Option<crate::ast::VerbPermissions>,
        verbs: &mut HashMap<String, crate::storage::object_store::VerbDefinition>,
    ) -> Result<()> {
        use crate::ast::ToSource;
        let source_code = format!(
            "verb {}({}) {}\nendverb",
            verb_name,
            args.iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            body.iter()
                .map(|stmt| stmt.to_source())
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        let verb_def = crate::storage::object_store::VerbDefinition {
            name: verb_name.to_string(),
            signature: crate::storage::object_store::VerbSignature {
                dobj: String::new(), // TODO: Add dobj/prep/iobj support
                prep: String::new(),
                iobj: String::new(),
            },
            code: source_code,
            ast: body.to_vec(),
            params: args.to_vec(),
            permissions: permissions
                .as_ref()
                .map(|p| crate::storage::object_store::VerbPermissions {
                    read: p.read == "anyone",
                    write: p.write == "anyone",
                    execute: p.execute == "anyone",
                })
                .unwrap_or(crate::storage::object_store::VerbPermissions {
                    read: true,
                    write: true,
                    execute: true,
                }),
        };

        verbs.insert(verb_name.to_string(), verb_def);
        Ok(())
    }

    /// Process an event member during object definition.
    ///
    /// Registers an event handler with the event system. Event handlers are
    /// callable code blocks that respond to specific events emitted within
    /// the system.
    ///
    /// # Arguments
    /// * `obj_id` - The ID of the object that owns this event handler
    /// * `event_name` - The name of the event this handler responds to
    /// * `params` - Parameter definitions for the event handler
    /// * `body` - The AST statements that make up the event handler body
    ///
    /// # Returns
    /// * `Ok(())` if the event handler is successfully registered
    /// * `Err` if event registration fails
    fn process_event_member(
        &mut self,
        obj_id: ObjectId,
        event_name: &str,
        params: &[crate::ast::Parameter],
        body: &[EchoAst],
        properties: &mut HashMap<String, PropertyValue>,
    ) -> Result<()> {
        use crate::ast::ToSource;
        let event_source = format!(
            "event {}({}) {}\nendevent",
            event_name,
            params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            body.iter()
                .map(|stmt| stmt.to_source())
                .collect::<Vec<_>>()
                .join("\n  ")
        );

        // Register event handler with the event system
        self.event_system.register_handler(
            obj_id,
            event_name.to_string(),
            params.iter().map(|p| p.name.clone()).collect(),
            body.to_vec(),
            None, // Default priority
        );

        // Store event handler metadata as a property for introspection
        properties.insert(
            format!("__event_{event_name}"),
            PropertyValue::String(event_source),
        );

        println!("Registered event handler '{event_name}' on object");
        Ok(())
    }

    /// Process a query member during object definition.
    ///
    /// Creates a Datalog-style query definition and stores it as object
    /// metadata. Queries define logical rules that can be used for
    /// inference and data retrieval.
    ///
    /// # Arguments
    /// * `query_name` - The name of the query being defined
    /// * `params` - Parameter names for the query
    /// * `clauses` - The query clauses that define the logical rules
    /// * `properties` - Mutable reference to the property map for storing
    ///   metadata
    ///
    /// # Returns
    /// * `Ok(())` if the query is successfully processed
    /// * `Err` if query processing fails
    fn process_query_member(
        &mut self,
        query_name: &str,
        params: &[String],
        clauses: &[crate::ast::QueryClause],
        properties: &mut HashMap<String, PropertyValue>,
    ) -> Result<()> {
        use crate::ast::ToSource;
        let mut query_source = format!("query {query_name}");
        if !params.is_empty() {
            query_source.push_str(&format!("({})", params.join(", ")));
        }
        query_source.push_str(" :- ");

        let clauses_str = clauses
            .iter()
            .map(|c| {
                let args_str = c
                    .args
                    .iter()
                    .map(|arg| match arg {
                        crate::ast::QueryArg::Variable(v) => v.clone(),
                        crate::ast::QueryArg::Constant(c) => c.to_source(),
                        crate::ast::QueryArg::Wildcard => "_".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", c.predicate, args_str)
            })
            .collect::<Vec<_>>()
            .join(", ");
        query_source.push_str(&clauses_str);
        query_source.push('.');

        // Store query metadata as a property
        properties.insert(
            format!("__query_{query_name}"),
            PropertyValue::String(query_source),
        );

        println!("Registered query '{query_name}' on object");
        Ok(())
    }

    /// Evaluate object definition
    fn eval_object_def(
        &mut self,
        name: &str,
        parent: &Option<String>,
        members: &[ObjectMember],
        player_id: ObjectId,
    ) -> Result<Value> {
        let obj_id = ObjectId::new();
        let mut properties = HashMap::new();
        let mut verbs = HashMap::new();

        // Process object members
        for member in members {
            match member {
                ObjectMember::Property {
                    name: prop_name,
                    value,
                    ..
                } => {
                    self.process_property_member(prop_name, value, player_id, &mut properties)?;
                }
                ObjectMember::Verb {
                    name: verb_name,
                    args,
                    body,
                    permissions,
                } => {
                    self.process_verb_member(verb_name, args, body, permissions, &mut verbs)?;
                }
                ObjectMember::Event {
                    name: event_name,
                    params,
                    body,
                } => {
                    self.process_event_member(obj_id, event_name, params, body, &mut properties)?;
                }
                ObjectMember::Query {
                    name: query_name,
                    params,
                    clauses,
                } => {
                    self.process_query_member(query_name, params, clauses, &mut properties)?;
                }
                _ => {}
            }
        }

        // TODO: Resolve parent object by name
        let parent_id = if let Some(_parent_name) = parent {
            // For now, just use root as parent
            // In a full implementation, we'd look up the parent object by name
            Some(ObjectId::root())
        } else {
            Some(ObjectId::root())
        };

        let obj = EchoObject {
            id: obj_id,
            parent: parent_id,
            name: name.to_string(),
            properties,
            verbs,
            queries: HashMap::new(),
            meta: MetaObject::new(obj_id),
        };

        self.storage.objects.store(obj)?;

        // Bind the object name to a property on #0
        let mut system_obj = self.storage.objects.get(ObjectId::system())?;
        system_obj
            .properties
            .insert(name.to_string(), PropertyValue::Object(obj_id));
        self.storage.objects.store(system_obj)?;

        Ok(Value::Object(obj_id))
    }

    /// Evaluate method call
    fn eval_method_call(
        &mut self,
        object: &EchoAst,
        method: &str,
        args: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        // Evaluate the object expression
        let obj_val = self.eval_with_player(object, player_id)?;

        if let Value::Object(obj_id) = obj_val {
            // Get the object
            let obj = self.storage.objects.get(obj_id)?;

            // Find the verb
            if let Some(_verb_def) = obj.verbs.get(method) {
                // Execute the verb with proper environment
                self.execute_verb(obj_id, method, args, player_id)
            } else {
                Err(anyhow!("Method '{}' not found on object", method))
            }
        } else {
            Err(anyhow!("Method call on non-object"))
        }
    }

    /// Evaluate object reference
    fn eval_object_ref(&mut self, n: &i64) -> Result<Value> {
        // Object reference like #0 or #1
        if *n == 0 {
            return Ok(Value::Object(ObjectId::system()));
        } else if *n == 1 {
            return Ok(Value::Object(ObjectId::root()));
        }

        // Check if there's an object_map property on the system object
        let system_obj = self.storage.objects.get(ObjectId::system())?;

        // First check if object_map is a verb/method
        if system_obj.verbs.contains_key("object_map") {
            // Call the object_map method with the numeric ID as argument
            let args = vec![EchoAst::Number(*n)];
            let player_id = self.current_player.unwrap_or(ObjectId::system());
            match self.execute_verb(ObjectId::system(), "object_map", &args, player_id) {
                Ok(Value::Object(obj_id)) => return Ok(Value::Object(obj_id)),
                Ok(Value::Null) => {
                    // Method returned null, meaning no mapping exists
                }
                Ok(other) => {
                    return Err(anyhow!(
                        "object_map method must return an object or null, got {}",
                        other.type_name()
                    ));
                }
                Err(_) => {
                    // Method failed, fall through to other checks
                }
            }
        }

        // Then check if it's a Map property
        if let Some(PropertyValue::Map(object_map)) = system_obj.properties.get("object_map") {
            // Look up the numeric ID in the map
            let key = n.to_string();
            if let Some(PropertyValue::Object(obj_id)) = object_map.get(&key) {
                return Ok(Value::Object(*obj_id));
            }
        }

        // If no mapping found, return a helpful error
        Err(anyhow!(
            "Object reference #{} not found. In Echo, objects are typically referenced by name. \
             To use numeric references, either:\n1. Define #0:object_map(n) to return the object \
             for numeric ID n\n2. Set #0.object_map as a map with numeric ID mappings",
            n
        ))
    }

    /// Evaluate list literal
    fn eval_list(&mut self, elements: &[EchoAst], player_id: ObjectId) -> Result<Value> {
        let mut list_values = Vec::new();
        for elem in elements {
            let val = self.eval_with_player(elem, player_id)?;
            list_values.push(val);
        }
        Ok(Value::List(list_values))
    }

    /// Evaluate lambda expression
    fn eval_lambda(
        &mut self,
        params: &[LambdaParam],
        body: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        // Capture the current environment
        let mut captured_env = HashMap::new();
        if let Some(env) = self.environments.get(&player_id) {
            captured_env = env.variables.clone();
        }

        Ok(Value::Lambda {
            params: params.to_vec(),
            body: body.clone(),
            captured_env,
        })
    }

    /// Evaluate assignment operation
    fn eval_assignment(
        &mut self,
        target: &LValue,
        value: &EchoAst,
        player_id: ObjectId,
    ) -> Result<Value> {
        // Evaluate the value
        let val = self.eval_with_player(value, player_id)?;

        // Handle different types of LValues
        match target {
            LValue::Binding {
                binding_type,
                pattern,
            } => {
                // Handle binding patterns
                self.handle_binding(binding_type, pattern, val.clone(), player_id)?;
                Ok(val)
            }
            LValue::PropertyAccess { object, property } => {
                // Evaluate the object
                let obj_val = self.eval_with_player(object, player_id)?;

                if let Value::Object(obj_id) = obj_val {
                    // Get the object
                    let mut obj = self.storage.objects.get(obj_id)?;

                    // Update the property
                    obj.properties
                        .insert(property.clone(), value_to_property_value(val.clone())?);

                    // Store the updated object
                    self.storage.objects.store(obj)?;

                    Ok(val)
                } else {
                    Err(anyhow!("Property assignment on non-object"))
                }
            }
            LValue::IndexAccess { object, index } => {
                // Evaluate the object and index
                let obj_val = self.eval_with_player(object, player_id)?;
                let idx_val = self.eval_with_player(index, player_id)?;

                if let (Value::List(mut list), Value::Integer(idx)) = (obj_val, idx_val) {
                    if idx < 0 || idx as usize >= list.len() {
                        Err(anyhow!("List index out of bounds"))
                    } else {
                        list[idx as usize] = val.clone();
                        // Note: In a real implementation, we'd need to update the original list
                        // This is a simplified version
                        Ok(val)
                    }
                } else {
                    Err(anyhow!("Index assignment requires list and integer index"))
                }
            }
        }
    }

    /// Evaluate function call
    /// Evaluate function calls by name (built-in functions)
    fn eval_function_call(
        &mut self,
        name: &str,
        args: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        // Evaluate arguments
        let arg_values: Result<Vec<_>> = args
            .iter()
            .map(|arg| self.eval_with_player(arg, player_id))
            .collect();
        let arg_values = arg_values?;

        // Built-in functions
        match name {
            "len" => {
                if arg_values.len() != 1 {
                    return Err(EvaluatorError::InvalidOperation {
                        message: format!(
                            "len() takes exactly 1 argument, got {}",
                            arg_values.len()
                        ),
                    }
                    .into());
                }

                match &arg_values[0] {
                    Value::List(items) => Ok(Value::Integer(items.len() as i64)),
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::Map(map) => Ok(Value::Integer(map.len() as i64)),
                    _ => Err(EvaluatorError::unary_type_error(
                        "len()",
                        "list, string, or map",
                        arg_values[0].type_name(),
                    )
                    .into()),
                }
            }
            "type" => {
                if arg_values.len() != 1 {
                    return Err(EvaluatorError::InvalidOperation {
                        message: format!(
                            "type() takes exactly 1 argument, got {}",
                            arg_values.len()
                        ),
                    }
                    .into());
                }

                Ok(Value::String(arg_values[0].type_name().to_string()))
            }
            "str" => {
                if arg_values.len() != 1 {
                    return Err(EvaluatorError::InvalidOperation {
                        message: format!(
                            "str() takes exactly 1 argument, got {}",
                            arg_values.len()
                        ),
                    }
                    .into());
                }

                Ok(Value::String(arg_values[0].to_string()))
            }
            "print" => {
                // Print all arguments separated by spaces
                let output = arg_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{output}");
                Ok(Value::Null)
            }
            // UI manipulation functions
            "ui_clear" => {
                self.send_ui_event(UiAction::Clear);
                Ok(Value::Null)
            }
            "ui_add_button" => {
                if arg_values.len() != 3 {
                    return Err(anyhow!(
                        "ui_add_button() takes exactly 3 arguments (id, label, action), got {}",
                        arg_values.len()
                    ));
                }

                let id = match &arg_values[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_button() id must be a string")),
                };

                let label = match &arg_values[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_button() label must be a string")),
                };

                let action = match &arg_values[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_button() action must be a string")),
                };

                self.send_ui_event(UiAction::AddButton { id, label, action });
                Ok(Value::Null)
            }
            "ui_add_text" => {
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(anyhow!(
                        "ui_add_text() takes 2-3 arguments (id, text, [style]), got {}",
                        arg_values.len()
                    ));
                }

                let id = match &arg_values[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_text() id must be a string")),
                };

                let text = match &arg_values[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_text() text must be a string")),
                };

                let style = if arg_values.len() > 2 {
                    match &arg_values[2] {
                        Value::Map(map) => {
                            let mut style_map = HashMap::new();
                            for (k, v) in map {
                                style_map.insert(k.clone(), v.to_string());
                            }
                            Some(style_map)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                self.send_ui_event(UiAction::AddText { id, text, style });
                Ok(Value::Null)
            }
            "ui_add_div" => {
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(anyhow!(
                        "ui_add_div() takes 2-3 arguments (id, content, [style]), got {}",
                        arg_values.len()
                    ));
                }

                let id = match &arg_values[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_div() id must be a string")),
                };

                let content = match &arg_values[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_add_div() content must be a string")),
                };

                let style = if arg_values.len() > 2 {
                    match &arg_values[2] {
                        Value::Map(map) => {
                            let mut style_map = HashMap::new();
                            for (k, v) in map {
                                style_map.insert(k.clone(), v.to_string());
                            }
                            Some(style_map)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                self.send_ui_event(UiAction::AddDiv { id, content, style });
                Ok(Value::Null)
            }
            "ui_update" => {
                if arg_values.len() != 2 {
                    return Err(anyhow!(
                        "ui_update() takes exactly 2 arguments (id, properties)"
                    ));
                }

                let id = match &arg_values[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("ui_update() id must be a string")),
                };

                let properties = match &arg_values[1] {
                    Value::Map(props) => props.clone(),
                    _ => return Err(anyhow!("ui_update() properties must be a map")),
                };

                self.send_ui_event(UiAction::Update { id, properties });
                Ok(Value::Null)
            }
            "emit" => {
                if arg_values.is_empty() {
                    return Err(anyhow!("emit() requires at least 1 argument (event_name)"));
                }

                let event_name = match &arg_values[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(anyhow!("emit() event name must be a string")),
                };

                // Remaining arguments become event args
                let event_args = arg_values[1..].to_vec();

                // Emit the event through the event system
                let event_system = self.event_system.clone();
                event_system.emit(
                    self,
                    event_system::Event {
                        name: event_name.clone(),
                        args: event_args,
                        emitter: player_id,
                        bubbles: false,
                        cancelable: false,
                    },
                )?;

                Ok(Value::Null)
            }
            _ => {
                // Try to resolve as a variable containing a function
                if let Ok(func_val) = self.eval_identifier(name, player_id) {
                    // If it's a lambda, call it with the evaluated arguments
                    match func_val {
                        Value::Lambda {
                            params,
                            body,
                            captured_env,
                        } => {
                            // Create a new environment with the captured environment
                            let mut lambda_env = Environment {
                                player_id,
                                variables: captured_env,
                                const_bindings: HashSet::new(),
                            };

                            // Process parameters based on their type
                            let mut arg_iter = arg_values.into_iter();
                            let mut rest_args = Vec::new();

                            for param in &params {
                                match param {
                                    LambdaParam::Simple(name) => match arg_iter.next() {
                                        Some(val) => {
                                            lambda_env.variables.insert(name.clone(), val);
                                        }
                                        None => {
                                            return Err(anyhow!(
                                                "Missing required argument: {}",
                                                name
                                            ));
                                        }
                                    },
                                    LambdaParam::Optional { name, default } => {
                                        let val = match arg_iter.next() {
                                            Some(v) => v,
                                            None => {
                                                // Evaluate the default value in the lambda's
                                                // environment
                                                let saved = self
                                                    .environments
                                                    .get(&player_id)
                                                    .map(|e| e.clone());
                                                self.environments
                                                    .insert(player_id, lambda_env.clone());
                                                let default_val =
                                                    self.eval_with_player_impl(default, player_id)?;
                                                if let Some(env) = saved {
                                                    self.environments.insert(player_id, env);
                                                }
                                                default_val
                                            }
                                        };
                                        lambda_env.variables.insert(name.clone(), val);
                                    }
                                    LambdaParam::Rest(name) => {
                                        // Collect all remaining arguments
                                        rest_args.extend(arg_iter.by_ref());
                                        lambda_env
                                            .variables
                                            .insert(name.clone(), Value::List(rest_args.clone()));
                                        break;
                                    }
                                }
                            }

                            // Check if there are extra arguments (only an error if no rest
                            // parameter)
                            if arg_iter.next().is_some()
                                && !params.iter().any(|p| matches!(p, LambdaParam::Rest(_)))
                            {
                                return Err(anyhow!("Too many arguments for lambda"));
                            }

                            // Save current environment and set lambda environment
                            let saved_env = self.environments.get(&player_id).map(|e| e.clone());
                            self.environments.insert(player_id, lambda_env);

                            // Evaluate the body
                            let result = self.eval_with_player_impl(&body, player_id);

                            // Restore the original environment
                            if let Some(env) = saved_env {
                                self.environments.insert(player_id, env);
                            }

                            result
                        }
                        _ => Err(anyhow!("{} is not a function", name)),
                    }
                } else {
                    Err(EvaluatorError::InvalidOperation {
                        message: format!("Unknown function: {name}"),
                    }
                    .into())
                }
            }
        }
    }

    fn eval_call(
        &mut self,
        func: &EchoAst,
        args: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        // Evaluate the function expression
        let func_val = self.eval_with_player(func, player_id)?;

        // Evaluate the arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_with_player(arg, player_id)?);
        }

        // Call the function
        match func_val {
            Value::Lambda {
                params,
                body,
                captured_env,
            } => {
                // Create a new environment with the captured environment
                let mut lambda_env = Environment {
                    player_id,
                    variables: captured_env,
                    const_bindings: HashSet::new(),
                };

                // Process parameters based on their type
                let mut arg_iter = arg_values.into_iter();
                let mut rest_args = Vec::new();

                for param in &params {
                    match param {
                        LambdaParam::Simple(name) => match arg_iter.next() {
                            Some(val) => {
                                lambda_env.variables.insert(name.clone(), val);
                            }
                            None => {
                                return Err(anyhow!("Missing required argument: {}", name));
                            }
                        },
                        LambdaParam::Optional { name, default } => {
                            let val = match arg_iter.next() {
                                Some(v) => v,
                                None => {
                                    // Evaluate the default value in the lambda's environment
                                    let saved =
                                        self.environments.get(&player_id).map(|e| e.clone());
                                    self.environments.insert(player_id, lambda_env.clone());
                                    let default_val =
                                        self.eval_with_player_impl(default, player_id)?;
                                    if let Some(env) = saved {
                                        self.environments.insert(player_id, env);
                                    }
                                    default_val
                                }
                            };
                            lambda_env.variables.insert(name.clone(), val);
                        }
                        LambdaParam::Rest(name) => {
                            // Collect all remaining arguments
                            rest_args.extend(arg_iter.by_ref());
                            lambda_env
                                .variables
                                .insert(name.clone(), Value::List(rest_args.clone()));
                            break; // Rest parameter consumes all remaining args
                        }
                    }
                }

                // Check if there are unused arguments (only if no rest parameter)
                let has_rest = params.iter().any(|p| matches!(p, LambdaParam::Rest(_)));
                if !has_rest && arg_iter.next().is_some() {
                    return Err(anyhow!("Too many arguments provided"));
                }

                // Save the current environment
                let saved_env = self.environments.get(&player_id).map(|e| e.clone());

                // Set the lambda environment
                self.environments.insert(player_id, lambda_env);

                // Evaluate the body
                let result = self.eval_with_player(&body, player_id);

                // Restore the original environment
                if let Some(env) = saved_env {
                    self.environments.insert(player_id, env);
                }

                result
            }
            _ => Err(anyhow!(
                "Cannot call non-function value: expected lambda but got {}",
                func_val.type_name()
            )),
        }
    }

    fn eval_with_player_impl(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        match ast {
            // Literals and basic values
            EchoAst::Number(_) | EchoAst::Float(_) | EchoAst::String(_) | EchoAst::Boolean(_) => {
                self.eval_literal(ast)
            }
            EchoAst::Identifier(s) => self.eval_identifier(s, player_id),
            EchoAst::SystemProperty(prop_name) => self.eval_system_property(prop_name),
            EchoAst::ObjectRef(n) => self.eval_object_ref(n),

            // Arithmetic operations
            EchoAst::Add { left, right } => {
                self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Add)
            }
            EchoAst::Subtract { left, right } => {
                self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Subtract)
            }
            EchoAst::Multiply { left, right } => {
                self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Multiply)
            }
            EchoAst::Divide { left, right } => {
                self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Divide)
            }
            EchoAst::Modulo { left, right } => {
                self.eval_arithmetic_op(left, right, player_id, ArithmeticOp::Modulo)
            }

            // Comparison operations
            EchoAst::Equal { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::Equal)
            }
            EchoAst::NotEqual { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::NotEqual)
            }
            EchoAst::LessThan { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::LessThan)
            }
            EchoAst::LessEqual { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::LessEqual)
            }
            EchoAst::GreaterThan { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::GreaterThan)
            }
            EchoAst::GreaterEqual { left, right } => {
                self.eval_comparison_op(left, right, player_id, ComparisonOp::GreaterEqual)
            }
            EchoAst::In { left, right } => self.eval_in(left, right, player_id),

            // Logical operations
            EchoAst::And { left, right } => self.eval_and(left, right, player_id),
            EchoAst::Or { left, right } => self.eval_or(left, right, player_id),
            EchoAst::Not { operand } => self.eval_not(operand, player_id),
            EchoAst::UnaryMinus { operand } => self.eval_unary_minus(operand, player_id),
            EchoAst::UnaryPlus { operand } => self.eval_unary_plus(operand, player_id),

            // Object operations
            EchoAst::PropertyAccess { object, property } => {
                self.eval_property_access(object, property, player_id)
            }
            EchoAst::IndexAccess { object, index } => {
                self.eval_index_access(object, index, player_id)
            }
            EchoAst::ObjectDef {
                name,
                parent,
                members,
            } => self.eval_object_def(name, parent, members, player_id),
            EchoAst::MethodCall {
                object,
                method,
                args,
            } => self.eval_method_call(object, method, args, player_id),

            // Collections and functions
            EchoAst::List { elements } => self.eval_list(elements, player_id),
            EchoAst::Map { entries } => self.eval_map(entries, player_id),
            EchoAst::Lambda { params, body } => self.eval_lambda(params, body, player_id),
            EchoAst::Call { func, args } => self.eval_call(func, args, player_id),
            EchoAst::FunctionCall { name, args } => self.eval_function_call(name, args, player_id),

            // Control flow
            EchoAst::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if(condition, then_branch, else_branch, player_id),
            EchoAst::While {
                condition, body, ..
            } => self.eval_while(condition, body, player_id),
            EchoAst::For {
                variable,
                collection,
                body,
                ..
            } => self.eval_for(variable, collection, body, player_id),
            EchoAst::Break { .. } | EchoAst::Continue { .. } => {
                unreachable!("Break/Continue should be handled by eval_with_control_flow")
            }

            // Program structure
            EchoAst::Program(statements) => self.eval_program(statements, player_id),
            EchoAst::Assignment { target, value } => self.eval_assignment(target, value, player_id),
            EchoAst::LocalAssignment { target, value } => {
                self.eval_local_assignment(target, value, player_id)
            }
            EchoAst::ConstAssignment { target, value } => {
                self.eval_const_assignment(target, value, player_id)
            }

            _ => Err(anyhow!("Evaluation not implemented for this AST node type")),
        }
    }

    fn execute_verb(
        &mut self,
        obj_id: ObjectId,
        method_name: &str,
        args: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        // Get the object
        let obj = self.storage.objects.get(obj_id)?;

        // Get the verb definition
        let verb_def = obj
            .verbs
            .get(method_name)
            .ok_or_else(|| anyhow!("Verb '{}' not found on object", method_name))?;

        // Check permissions
        if !verb_def.permissions.execute {
            return Err(anyhow!(
                "Permission denied: cannot execute verb '{}'",
                method_name
            ));
        }

        // Evaluate the arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_with_player(arg, player_id)?);
        }

        // Create a new environment for the verb execution
        let verb_env = self
            .environments
            .entry(player_id)
            .or_insert_with(|| Environment {
                player_id,
                variables: HashMap::new(),
                const_bindings: HashSet::new(),
            })
            .clone();

        // Bind parameters to arguments
        let mut verb_variables = verb_env.variables.clone();

        // Bind 'this' to the current object
        verb_variables.insert("this".to_string(), Value::Object(obj_id));

        // Bind 'caller' to the calling player
        verb_variables.insert("caller".to_string(), Value::Object(player_id));

        // Bind the parameters
        for (i, param) in verb_def.params.iter().enumerate() {
            if i < arg_values.len() {
                verb_variables.insert(param.name.clone(), arg_values[i].clone());
            } else if let Some(ref default) = param.default_value {
                // Evaluate default value
                let default_val = self.eval_with_player(default, player_id)?;
                verb_variables.insert(param.name.clone(), default_val);
            } else {
                return Err(anyhow!("Missing required argument '{}'", param.name));
            }
        }

        // Save current environment and set up verb environment
        let saved_env = self.environments.get(&player_id).map(|env| env.clone());
        self.environments.insert(
            player_id,
            Environment {
                player_id,
                variables: verb_variables,
                const_bindings: HashSet::new(),
            },
        );

        // Execute the verb body
        let mut result = Value::Null;
        for stmt in &verb_def.ast {
            match self.eval_with_control_flow(stmt, player_id)? {
                ControlFlow::None(val) => result = val,
                ControlFlow::Return(val) => {
                    // Restore environment and return
                    if let Some(env) = saved_env {
                        self.environments.insert(player_id, env);
                    }
                    return Ok(val);
                }
                ControlFlow::Break(_) => {
                    // Restore environment
                    if let Some(env) = saved_env {
                        self.environments.insert(player_id, env);
                    }
                    return Err(anyhow!("Break outside of loop in verb"));
                }
                ControlFlow::Continue(_) => {
                    // Restore environment
                    if let Some(env) = saved_env {
                        self.environments.insert(player_id, env);
                    }
                    return Err(anyhow!("Continue outside of loop in verb"));
                }
            }
        }

        // Restore environment
        if let Some(env) = saved_env {
            self.environments.insert(player_id, env);
        }

        Ok(result)
    }

    fn handle_binding(
        &mut self,
        binding_type: &BindingType,
        pattern: &BindingPattern,
        value: Value,
        player_id: ObjectId,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Check if it's a const reassignment
                if let Some(env) = self.environments.get(&player_id) {
                    if matches!(binding_type, BindingType::None)
                        && env.const_bindings.contains(name)
                    {
                        return Err(anyhow!("Cannot reassign const variable: {}", name));
                    }
                }

                // Store the value
                self.environments.entry(player_id).and_modify(|env| {
                    env.variables.insert(name.clone(), value.clone());

                    // Track const bindings
                    match binding_type {
                        BindingType::Const => {
                            env.const_bindings.insert(name.clone());
                        }
                        BindingType::Let => {
                            // Remove from const set if it was there
                            env.const_bindings.remove(name);
                        }
                        BindingType::None => {
                            // Keep existing const status
                        }
                    }
                });
                Ok(())
            }
            BindingPattern::List(patterns) => {
                // Destructure list
                if let Value::List(values) = value {
                    if patterns.len() != values.len() {
                        return Err(anyhow!(
                            "Pattern length mismatch: expected {}, got {}",
                            patterns.len(),
                            values.len()
                        ));
                    }

                    for (pattern_elem, val) in patterns.iter().zip(values.iter()) {
                        // Convert BindingPatternElement to BindingPattern
                        let pattern = match pattern_elem {
                            BindingPatternElement::Simple(name) => {
                                BindingPattern::Identifier(name.clone())
                            }
                            BindingPatternElement::Optional { name, .. } => {
                                BindingPattern::Identifier(name.clone())
                            }
                            BindingPatternElement::Rest(name) => BindingPattern::Rest(Box::new(
                                BindingPattern::Identifier(name.clone()),
                            )),
                        };
                        self.handle_binding(binding_type, &pattern, val.clone(), player_id)?;
                    }
                    Ok(())
                } else {
                    Err(anyhow!("Cannot destructure non-list value"))
                }
            }
            BindingPattern::Rest(pattern) => {
                // For now, just bind the whole value
                self.handle_binding(binding_type, pattern, value, player_id)
            }
            BindingPattern::Ignore => {
                // Do nothing
                Ok(())
            }
            BindingPattern::Object(_) => {
                // Object destructuring not implemented yet
                Err(anyhow!("Object destructuring not yet implemented"))
            }
        }
    }
}

impl EvaluatorTrait for Evaluator {
    fn create_player(&mut self, name: &str) -> Result<ObjectId> {
        self.create_player(name)
    }

    fn switch_player(&mut self, player_id: ObjectId) -> Result<()> {
        self.switch_player(player_id)
    }

    fn current_player(&self) -> Option<ObjectId> {
        self.current_player()
    }

    fn eval(&mut self, ast: &EchoAst) -> Result<Value> {
        self.eval(ast)
    }

    fn eval_with_player(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        self.eval_with_player(ast, player_id)
    }
}

/// Factory function to create the appropriate evaluator based on features
pub fn create_evaluator(storage: Arc<Storage>) -> Result<Box<dyn EvaluatorTrait>> {
    #[cfg(feature = "jit")]
    {
        Ok(Box::new(JitEvaluator::new(storage)?))
    }

    #[cfg(not(feature = "jit"))]
    {
        Ok(Box::new(Evaluator::new(storage)))
    }
}

/// Enum to choose evaluator type at runtime
#[derive(Debug, Clone, Copy)]
pub enum EvaluatorType {
    Interpreter,
    #[cfg(feature = "jit")]
    Jit,
}

/// Create a specific evaluator type
pub fn create_evaluator_of_type(
    storage: Arc<Storage>,
    eval_type: EvaluatorType,
) -> Result<Box<dyn EvaluatorTrait>> {
    match eval_type {
        EvaluatorType::Interpreter => Ok(Box::new(Evaluator::new(storage))),
        #[cfg(feature = "jit")]
        EvaluatorType::Jit => Ok(Box::new(JitEvaluator::new(storage)?)),
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
            let prop_items: Result<Vec<_>> =
                items.into_iter().map(value_to_property_value).collect();
            Ok(PropertyValue::List(prop_items?))
        }
        Value::Map(map) => {
            let prop_map: Result<HashMap<String, PropertyValue>> = map
                .into_iter()
                .map(|(k, v)| value_to_property_value(v).map(|pv| (k, pv)))
                .collect();
            Ok(PropertyValue::Map(prop_map?))
        }
        Value::Lambda { .. } => {
            // For now, we can't store lambdas as properties
            Err(anyhow!("Cannot store lambda functions as properties"))
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
            let val_items: Result<Vec<_>> =
                items.into_iter().map(property_value_to_value).collect();
            Ok(Value::List(val_items?))
        }
        PropertyValue::Map(map) => {
            let val_map: Result<HashMap<String, Value>> = map
                .into_iter()
                .map(|(k, v)| property_value_to_value(v).map(|val| (k, val)))
                .collect();
            Ok(Value::Map(val_map?))
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Object(id) => write!(f, "{id}"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                    first = false;
                }
                write!(f, "}}")
            }
            Value::Lambda { params, .. } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| match p {
                        LambdaParam::Simple(name) => name.clone(),
                        LambdaParam::Optional { name, .. } => format!("?{name}"),
                        LambdaParam::Rest(name) => format!("@{name}"),
                    })
                    .collect();
                write!(f, "fn({})", param_strs.join(", "))
            }
        }
    }
}

impl Value {
    /// Get a truncated display representation suitable for .env output
    pub fn display_truncated(&self, max_len: usize) -> String {
        let full = self.to_string();
        if full.len() <= max_len {
            full
        } else {
            format!("{}...", &full[..max_len.saturating_sub(3)])
        }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Object(_) => "object",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Lambda { .. } => "lambda",
        }
    }

    /// Get the source code representation of this value (for lambdas and
    /// functions)
    pub fn to_source(&self) -> Option<String> {
        match self {
            Value::Lambda { params, body, .. } => {
                use crate::ast::ToSource;
                let params_str = params
                    .iter()
                    .map(|p| p.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("fn {{{}}} {} endfn", params_str, body.to_source()))
            }
            _ => None, // Other value types don't have source representations
        }
    }
}
