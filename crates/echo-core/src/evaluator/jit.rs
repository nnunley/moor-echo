//! JIT Compiler for Echo language using Cranelift
//!
//! This module provides a JIT compilation backend for the Echo language,
//! compiling rust-sitter AST nodes to native machine code for performance.

#[cfg(feature = "jit")]
use cranelift::prelude::*;
#[cfg(feature = "jit")]
use cranelift::codegen::ir::condcodes::IntCC;
#[cfg(feature = "jit")]
use cranelift_jit::{JITBuilder, JITModule};
#[cfg(feature = "jit")]
use cranelift_module::Module;

// NewType wrappers to avoid conflicts with our Value type
#[cfg(feature = "jit")]
#[derive(Debug, Clone, Copy)]
pub struct CraneliftValue(cranelift::prelude::Value);

#[cfg(feature = "jit")]
impl CraneliftValue {
    pub fn new(value: cranelift::prelude::Value) -> Self {
        CraneliftValue(value)
    }

    pub fn inner(self) -> cranelift::prelude::Value {
        self.0
    }
}

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use dashmap::DashMap;

use super::{Environment, EvaluatorTrait, Value};
use crate::{
    ast::{EchoAst, LValue, BindingType, BindingPattern},
    storage::{ObjectId, Storage},
};

/// JIT-compiled evaluator for Echo language
pub struct JitEvaluator {
    #[cfg(feature = "jit")]
    builder_context: Option<FunctionBuilderContext>,
    #[cfg(feature = "jit")]
    ctx: Option<codegen::Context>,
    #[cfg(feature = "jit")]
    module: Option<JITModule>,

    storage: Arc<Storage>,
    environments: DashMap<ObjectId, Environment>,
    current_player: Option<ObjectId>,

    // Compilation cache
    compiled_functions: HashMap<String, *const u8>,

    // Performance metrics
    compilation_count: usize,
    execution_count: usize,
    hot_threshold: usize,
    
    // Whether JIT is actually enabled/supported
    jit_enabled: bool,
}

/// Control flow result for handling break/continue/return
enum ControlFlow {
    None(Value),
    Return(Value),
    Break(Option<String>),
    Continue(Option<String>),
}

impl ControlFlow {
    fn into_value(self) -> Result<Value> {
        match self {
            ControlFlow::None(v) | ControlFlow::Return(v) => Ok(v),
            ControlFlow::Break(_) => Err(anyhow!("Break used outside of loop")),
            ControlFlow::Continue(_) => Err(anyhow!("Continue used outside of loop")),
        }
    }
}

impl JitEvaluator {
    /// Create a new JIT evaluator with fallback to interpreter-only mode if JIT is unsupported
    pub fn new_with_fallback(storage: Arc<Storage>) -> Self {
        #[cfg(feature = "jit")]
        {
            if Self::is_jit_supported() {
                match Self::try_create_jit(storage.clone()) {
                    Ok((builder_context, ctx, module)) => Self {
                        builder_context: Some(builder_context),
                        ctx: Some(ctx),
                        module: Some(module),
                        storage,
                        environments: DashMap::new(),
                        current_player: None,
                        compiled_functions: HashMap::new(),
                        compilation_count: 0,
                        execution_count: 0,
                        hot_threshold: 10,
                        jit_enabled: true,
                    },
                    Err(_) => Self::fallback_evaluator(storage),
                }
            } else {
                Self::fallback_evaluator(storage)
            }
        }

        #[cfg(not(feature = "jit"))]
        {
            Self::fallback_evaluator(storage)
        }
    }
    
    fn fallback_evaluator(storage: Arc<Storage>) -> Self {
        Self {
            #[cfg(feature = "jit")]
            builder_context: None,
            #[cfg(feature = "jit")]
            ctx: None,
            #[cfg(feature = "jit")]
            module: None,
            storage,
            environments: DashMap::new(),
            current_player: None,
            compiled_functions: HashMap::new(),
            compilation_count: 0,
            execution_count: 0,
            hot_threshold: 10,
            jit_enabled: false,
        }
    }
    
    #[cfg(feature = "jit")]
    fn try_create_jit(_storage: Arc<Storage>) -> Result<(FunctionBuilderContext, codegen::Context, JITModule)> {
        use cranelift::prelude::settings;
        use cranelift_native;
        use std::panic::{catch_unwind, AssertUnwindSafe};
        
        // Cranelift has platform-specific limitations that may cause panics
        // Specifically, macOS ARM64 has PLT issues that prevent JIT from working
        // Apply workaround: disable is_pic flag as discussed in wasmtime#2735
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<(FunctionBuilderContext, codegen::Context, JITModule)> {
            // Configure flags with macOS ARM64 workaround
            let mut flag_builder = settings::builder();
            flag_builder.set("use_colocated_libcalls", "false").unwrap();
            flag_builder.set("is_pic", "false").unwrap(); // Workaround for macOS ARM64
            
            // Build ISA with custom flags
            let isa_builder = cranelift_native::builder()
                .map_err(|msg| anyhow!("Host machine is not supported: {}", msg))?;
            
            let isa = isa_builder
                .finish(settings::Flags::new(flag_builder))
                .map_err(|e| anyhow!("Failed to create ISA: {}", e))?;
            
            // Create JIT module with custom ISA
            let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
            let module = JITModule::new(builder);
            let builder_context = FunctionBuilderContext::new();
            let ctx = module.make_context();
            
            Ok((builder_context, ctx, module))
        }));
        
        match result {
            Ok(Ok(tuple)) => Ok(tuple),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Cranelift JIT initialization failed - unexpected panic")),
        }
    }

    /// Create a new JIT evaluator (strict - fails if JIT not supported)
    pub fn new(storage: Arc<Storage>) -> Result<Self> {
        #[cfg(feature = "jit")]
        {
            // Check if JIT is supported on this architecture
            if !Self::is_jit_supported() {
                return Err(anyhow!("JIT compilation is not supported on this architecture"));
            }

            let (builder_context, ctx, module) = Self::try_create_jit(storage.clone())?;

            Ok(Self {
                builder_context: Some(builder_context),
                ctx: Some(ctx),
                module: Some(module),
                storage,
                environments: DashMap::new(),
                current_player: None,
                compiled_functions: HashMap::new(),
                compilation_count: 0,
                execution_count: 0,
                hot_threshold: 10, // Compile after 10 interpretations
                jit_enabled: true,
            })
        }

        #[cfg(not(feature = "jit"))]
        {
            Ok(Self {
                storage,
                environments: DashMap::new(),
                current_player: None,
                compiled_functions: HashMap::new(),
                compilation_count: 0,
                execution_count: 0,
                hot_threshold: 10,
                jit_enabled: false,
            })
        }
    }

    /// Check if JIT compilation is supported on the current architecture
    #[cfg(feature = "jit")]
    fn is_jit_supported() -> bool {
        // Cranelift supports x86_64 and aarch64 architectures
        // macOS ARM64 has PLT limitations but we apply the is_pic=false workaround
        // See: https://github.com/bytecodealliance/wasmtime/issues/2735
        cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64")
    }

    #[cfg(not(feature = "jit"))]
    fn is_jit_supported() -> bool {
        false
    }

    /// Create a new player
    pub fn create_player(&mut self, _name: &str) -> Result<ObjectId> {
        // Same implementation as interpreter evaluator
        let player_id = ObjectId::new();

        // Create environment for the player
        let env = Environment {
            player_id,
            variables: HashMap::new(),
            const_bindings: HashSet::new(),
        };
        self.environments.insert(player_id, env);

        Ok(player_id)
    }

    /// Switch to a different player
    pub fn switch_player(&mut self, player_id: ObjectId) -> Result<()> {
        self.current_player = Some(player_id);
        Ok(())
    }

    /// Get current player
    pub fn current_player(&self) -> Option<ObjectId> {
        self.current_player
    }

    /// Evaluate an AST node with JIT compilation
    pub fn eval(&mut self, ast: &EchoAst) -> Result<Value> {
        let player_id = self
            .current_player
            .ok_or_else(|| anyhow!("No player selected"))?;

        self.eval_with_player(ast, player_id)
    }

    /// Evaluate with specific player, using JIT when beneficial
    pub fn eval_with_player(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // Generate a key for this AST pattern
        let ast_key = self.ast_to_key(ast);

        // Check if we should JIT compile this
        if self.should_compile(&ast_key) {
            self.compile_and_execute(ast, player_id)
        } else {
            // Use interpreter for now
            self.interpret(ast, player_id)
        }
    }

    /// Decide whether to JIT compile based on hotness
    fn should_compile(&self, _ast_key: &str) -> bool {
        // Only compile if JIT is enabled and supported
        // Enable JIT compilation for testing now that it works
        self.jit_enabled
    }

    /// Generate a key for caching compiled functions
    fn ast_to_key(&self, ast: &EchoAst) -> String {
        // Simple key generation - in production, use a hash
        format!("{:?}", ast)
    }

    /// Compile AST to machine code and execute
    #[cfg(feature = "jit")]
    fn compile_and_execute(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // For now, we only support compiling simple expressions
        // More complex AST nodes will fall back to interpretation
        match ast {
            EchoAst::Number(_) 
            | EchoAst::Float(_)
            | EchoAst::String(_)
            | EchoAst::Boolean(_)
            | EchoAst::Null
            | EchoAst::Add { .. } 
            | EchoAst::Subtract { .. }
            | EchoAst::Multiply { .. }
            | EchoAst::Divide { .. }
            | EchoAst::Modulo { .. }
            | EchoAst::Power { .. }
            | EchoAst::UnaryMinus { .. }
            | EchoAst::UnaryPlus { .. }
            | EchoAst::Equal { .. }
            | EchoAst::NotEqual { .. }
            | EchoAst::LessThan { .. }
            | EchoAst::LessEqual { .. }
            | EchoAst::GreaterThan { .. }
            | EchoAst::GreaterEqual { .. }
            | EchoAst::In { .. }
            | EchoAst::And { .. }
            | EchoAst::Or { .. }
            | EchoAst::Not { .. }
            | EchoAst::Identifier(_)
            | EchoAst::Assignment { .. }
            | EchoAst::If { .. }
            | EchoAst::While { .. }
            | EchoAst::For { .. }
            | EchoAst::Return { .. }
            | EchoAst::Break { .. }
            | EchoAst::Continue { .. } => {
                // These are the AST types we support compiling
                match self.compile_ast(ast) {
                    Ok(()) => {
                        // Compilation succeeded, but we need more infrastructure
                        // to actually execute the compiled code
                        // For now, fall back to interpretation
                        self.compilation_count += 1;
                        self.interpret(ast, player_id)
                    }
                    Err(e) => {
                        // Compilation failed, fall back to interpretation
                        if !e.to_string().contains("falling back to interpreter") {
                            eprintln!("JIT compilation failed: {}", e);
                        }
                        self.interpret(ast, player_id)
                    }
                }
            }
            _ => {
                // Unsupported AST node, use interpreter
                self.interpret(ast, player_id)
            }
        }
    }

    #[cfg(not(feature = "jit"))]
    fn compile_and_execute(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // JIT feature not enabled, use interpreter
        self.interpret(ast, player_id)
    }

    /// Interpret AST using the same logic as the main evaluator
    fn interpret(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // For now, we'll implement a simplified interpreter
        // In a full implementation, this would be identical to the main evaluator
        match ast {
            EchoAst::Number(n) => Ok(Value::Integer(*n)),
            EchoAst::Float(f) => Ok(Value::Float(*f)),
            EchoAst::String(s) => Ok(Value::String(s.clone())),
            EchoAst::Boolean(b) => Ok(Value::Boolean(*b)),
            EchoAst::Null => Ok(Value::Null),
            EchoAst::Identifier(s) => {
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
            EchoAst::Add { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;

                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::String(l), Value::String(r)) => {
                        Ok(Value::String(format!("{}{}", l, r)))
                    }
                    _ => Err(anyhow!("Type error in addition")),
                }
            }
            EchoAst::Subtract { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;

                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
                    _ => Err(anyhow!("Type error in subtraction")),
                }
            }
            EchoAst::Multiply { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;

                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
                    _ => Err(anyhow!("Type error in multiplication")),
                }
            }
            EchoAst::Divide { left, right } => {
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
                    _ => Err(anyhow!("Type error in division")),
                }
            }
            EchoAst::Modulo { left, right } => {
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
                    (Value::Float(l), Value::Float(r)) => {
                        if *r == 0.0 {
                            Err(anyhow!("Modulo by zero"))
                        } else {
                            Ok(Value::Float(l % r))
                        }
                    }
                    _ => Err(anyhow!("Type error in modulo")),
                }
            }
            EchoAst::Power { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;

                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => {
                        if *r < 0 {
                            Err(anyhow!("Negative exponent not supported for integers"))
                        } else {
                            let result = (*l as f64).powi(*r as i32) as i64;
                            Ok(Value::Integer(result))
                        }
                    }
                    (Value::Float(l), Value::Float(r)) => {
                        Ok(Value::Float(l.powf(*r)))
                    }
                    _ => Err(anyhow!("Type error in power")),
                }
            }
            EchoAst::UnaryMinus { operand } => {
                let val = self.eval_with_player(operand, player_id)?;
                match val {
                    Value::Integer(n) => Ok(Value::Integer(-n)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(anyhow!("Type error in unary minus")),
                }
            }
            EchoAst::UnaryPlus { operand } => {
                // Unary plus is a no-op
                self.eval_with_player(operand, player_id)
            }
            EchoAst::Equal { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l == r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
                    _ => Ok(Value::Boolean(false)), // Different types are not equal
                }
            }
            EchoAst::NotEqual { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l != r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Null, Value::Null) => Ok(Value::Boolean(false)),
                    _ => Ok(Value::Boolean(true)), // Different types are not equal
                }
            }
            EchoAst::LessThan { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l < r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l < r)),
                    _ => Err(anyhow!("Type error in less than comparison")),
                }
            }
            EchoAst::LessEqual { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l <= r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l <= r)),
                    _ => Err(anyhow!("Type error in less than or equal comparison")),
                }
            }
            EchoAst::GreaterThan { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l > r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l > r)),
                    _ => Err(anyhow!("Type error in greater than comparison")),
                }
            }
            EchoAst::GreaterEqual { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Boolean(l >= r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l >= r)),
                    _ => Err(anyhow!("Type error in greater than or equal comparison")),
                }
            }
            EchoAst::In { left, right } => {
                let left_val = self.eval_with_player(left, player_id)?;
                let right_val = self.eval_with_player(right, player_id)?;
                
                match &right_val {
                    Value::List(items) => {
                        for item in items {
                            match (&left_val, item) {
                                (Value::Integer(l), Value::Integer(r)) if l == r => return Ok(Value::Boolean(true)),
                                (Value::Float(l), Value::Float(r)) if l == r => return Ok(Value::Boolean(true)),
                                (Value::String(l), Value::String(r)) if l == r => return Ok(Value::Boolean(true)),
                                (Value::Boolean(l), Value::Boolean(r)) if l == r => return Ok(Value::Boolean(true)),
                                (Value::Null, Value::Null) => return Ok(Value::Boolean(true)),
                                _ => continue,
                            }
                        }
                        Ok(Value::Boolean(false))
                    }
                    Value::String(s) => {
                        // Check if left is a substring of right
                        match &left_val {
                            Value::String(needle) => Ok(Value::Boolean(s.contains(needle))),
                            _ => Err(anyhow!("Type error in 'in' operator")),
                        }
                    }
                    _ => Err(anyhow!("Right side of 'in' must be a list or string")),
                }
            }
            EchoAst::List { elements } => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_with_player(elem, player_id)?);
                }
                Ok(Value::List(values))
            }
            EchoAst::And { left, right } => {
                // Short-circuit evaluation
                let left_val = self.eval_with_player(left, player_id)?;
                match left_val {
                    Value::Boolean(false) => Ok(Value::Boolean(false)), // Short-circuit
                    Value::Boolean(true) => {
                        let right_val = self.eval_with_player(right, player_id)?;
                        match right_val {
                            Value::Boolean(b) => Ok(Value::Boolean(b)),
                            _ => Err(anyhow!("Type error in AND operation")),
                        }
                    }
                    _ => Err(anyhow!("Type error in AND operation")),
                }
            }
            EchoAst::Or { left, right } => {
                // Short-circuit evaluation
                let left_val = self.eval_with_player(left, player_id)?;
                match left_val {
                    Value::Boolean(true) => Ok(Value::Boolean(true)), // Short-circuit
                    Value::Boolean(false) => {
                        let right_val = self.eval_with_player(right, player_id)?;
                        match right_val {
                            Value::Boolean(b) => Ok(Value::Boolean(b)),
                            _ => Err(anyhow!("Type error in OR operation")),
                        }
                    }
                    _ => Err(anyhow!("Type error in OR operation")),
                }
            }
            EchoAst::Not { operand } => {
                let val = self.eval_with_player(operand, player_id)?;
                match val {
                    Value::Boolean(b) => Ok(Value::Boolean(!b)),
                    _ => Err(anyhow!("Type error in NOT operation")),
                }
            }
            EchoAst::Assignment { target, value } => {
                let val = self.eval_with_player(value, player_id)?;
                
                // For now, only support simple identifier assignment
                match target {
                    LValue::Binding { binding_type, pattern } => {
                        match pattern {
                            BindingPattern::Identifier(name) => {
                                // Get or create environment for the player
                                if let Some(mut env) = self.environments.get_mut(&player_id) {
                                    env.variables.insert(name.clone(), val.clone());
                                    if matches!(binding_type, BindingType::Const) {
                                        env.const_bindings.insert(name.clone());
                                    }
                                } else {
                                    return Err(anyhow!("No environment for player"));
                                }
                                Ok(val)
                            }
                            _ => Err(anyhow!("Complex binding patterns not yet supported")),
                        }
                    }
                    _ => Err(anyhow!("Complex assignment targets not yet supported")),
                }
            }
            EchoAst::If { condition, then_branch, else_branch } => {
                // Fallback to interpreter for If statements
                self.eval_if(condition, then_branch, else_branch, player_id)
            }
            EchoAst::While { condition, body, .. } => {
                // Fallback to interpreter for While loops
                self.eval_while(condition, body, player_id)
            }
            EchoAst::For { variable, collection, body, .. } => {
                // Fallback to interpreter for For loops
                self.eval_for(variable, collection, body, player_id)
            }
            EchoAst::Return { value } => {
                // Evaluate return value if present
                if let Some(val_ast) = value {
                    self.eval_with_player(val_ast, player_id)
                } else {
                    Ok(Value::Null)
                }
            }
            EchoAst::Break { .. } => {
                // Break should be handled by loop context
                Err(anyhow!("Break used outside of loop"))
            }
            EchoAst::Continue { .. } => {
                // Continue should be handled by loop context
                Err(anyhow!("Continue used outside of loop"))
            }
            _ => {
                // For other AST nodes, delegate to main evaluator for now
                // In a full implementation, we'd handle all cases
                Err(anyhow!(
                    "AST node not yet implemented in JIT evaluator: {:?}",
                    ast
                ))
            }
        }
    }

    /// Check if JIT compilation is enabled for this evaluator instance
    pub fn is_jit_enabled(&self) -> bool {
        self.jit_enabled
    }

    /// Get performance statistics
    pub fn stats(&self) -> JitStats {
        JitStats {
            compilation_count: self.compilation_count,
            execution_count: self.execution_count,
            compiled_functions: self.compiled_functions.len(),
            hot_threshold: self.hot_threshold,
            jit_enabled: self.jit_enabled,
        }
    }
}

/// Performance statistics for JIT compilation
#[derive(Debug, Clone)]
pub struct JitStats {
    pub compilation_count: usize,
    pub execution_count: usize,
    pub compiled_functions: usize,
    pub hot_threshold: usize,
    pub jit_enabled: bool,
}

#[cfg(feature = "jit")]
impl JitEvaluator {
    /// Compile an AST node to Cranelift IR
    pub fn compile_ast(&mut self, ast: &EchoAst) -> Result<()> {
        if !self.jit_enabled {
            return Err(anyhow!("JIT compilation is not enabled"));
        }

        #[cfg(feature = "jit")]
        {
            let ctx = self.ctx.as_mut().ok_or_else(|| anyhow!("JIT context not available"))?;
            let module = self.module.as_ref().ok_or_else(|| anyhow!("JIT module not available"))?;

            // Clear previous function
            ctx.func.clear();

            // Set up function signature
            let int_type = module.target_config().pointer_type();
            ctx.func.signature.returns.push(AbiParam::new(int_type));

            // Create a fresh builder context for each compilation
            let mut fresh_builder_context = FunctionBuilderContext::new();
            
            // Build the function
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fresh_builder_context);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block); // Seal the entry block

            // Compile the AST
            let value = Self::compile_ast_node(ast, &mut builder)?;
            builder.ins().return_(&[value.inner()]);

            // Seal all blocks and finalize the function
            builder.seal_all_blocks();
            builder.finalize();

            Ok(())
        }

        #[cfg(not(feature = "jit"))]
        {
            Err(anyhow!("JIT feature not enabled"))
        }
    }

    /// Compile a single AST node to Cranelift IR
    fn compile_ast_node(ast: &EchoAst, builder: &mut FunctionBuilder) -> Result<CraneliftValue> {
        match ast {
            EchoAst::Number(n) => {
                let imm = builder.ins().iconst(types::I64, *n);
                Ok(CraneliftValue::new(imm))
            }
            EchoAst::Float(_) => {
                // Float compilation requires type system changes
                // For now, fall back to interpreter
                return Err(anyhow!("Float literals require type system support, falling back to interpreter"));
            }
            EchoAst::Boolean(_) => {
                // Boolean literals require type system support because:
                // 1. Our JIT functions return i64, not a tagged union type
                // 2. We can't distinguish between integer 0/1 and boolean false/true
                // 3. Comparisons work because they return 0/1 as integers
                // For now, fall back to interpreter
                return Err(anyhow!("Boolean literals require type system support, falling back to interpreter"));
            }
            EchoAst::Null => {
                // Null compilation requires type system changes
                // For now, fall back to interpreter
                return Err(anyhow!("Null literal requires type system support, falling back to interpreter"));
            }
            EchoAst::String(_) => {
                // String compilation is complex - requires heap allocation
                // For now, fall back to interpreter
                return Err(anyhow!("String literals require heap allocation, falling back to interpreter"));
            }
            EchoAst::Add { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                let result = builder.ins().iadd(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::Subtract { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                let result = builder.ins().isub(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::Multiply { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                let result = builder.ins().imul(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::Divide { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Use signed division
                let result = builder.ins().sdiv(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::Modulo { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Use signed remainder
                let result = builder.ins().srem(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::Power { .. } => {
                // Power operation is complex and requires runtime library support
                // For now, we fall back to the interpreter
                // A full implementation would need to:
                // 1. Link with libm for pow() function
                // 2. Handle integer overflow
                // 3. Support negative exponents
                return Err(anyhow!("Power operation requires runtime library support, falling back to interpreter"));
            }
            EchoAst::UnaryMinus { operand } => {
                let operand_val = Self::compile_ast_node(operand, builder)?;
                let zero = builder.ins().iconst(types::I64, 0);
                let result = builder.ins().isub(zero, operand_val.inner());
                Ok(CraneliftValue::new(result))
            }
            EchoAst::UnaryPlus { operand } => {
                // Unary plus is a no-op, just return the operand
                Self::compile_ast_node(operand, builder)
            }
            EchoAst::Equal { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare integers for equality
                let cmp = builder.ins().icmp(IntCC::Equal, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::NotEqual { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare integers for inequality
                let cmp = builder.ins().icmp(IntCC::NotEqual, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::LessThan { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare signed integers
                let cmp = builder.ins().icmp(IntCC::SignedLessThan, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::LessEqual { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare signed integers
                let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::GreaterThan { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare signed integers
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::GreaterEqual { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                // Compare signed integers
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(cmp))
            }
            EchoAst::In { .. } => {
                // In operator requires runtime support for lists/strings
                return Err(anyhow!("In operator requires runtime support, falling back to interpreter"));
            }
            EchoAst::And { .. } => {
                // Logical AND requires control flow for short-circuit evaluation
                // For now, fall back to interpreter
                return Err(anyhow!("Logical AND requires control flow support, falling back to interpreter"));
            }
            EchoAst::Or { .. } => {
                // Logical OR requires control flow for short-circuit evaluation
                // For now, fall back to interpreter
                return Err(anyhow!("Logical OR requires control flow support, falling back to interpreter"));
            }
            EchoAst::Not { operand } => {
                // Check if operand is a boolean literal (which would fall back)
                match operand.as_ref() {
                    EchoAst::Boolean(_) => {
                        return Err(anyhow!("NOT with boolean literal requires type system support, falling back to interpreter"));
                    }
                    _ => {
                        // NOT is simple - just invert the boolean
                        let operand_val = Self::compile_ast_node(operand, builder)?;
                        // In Cranelift, booleans are represented as integers (0 or 1)
                        // NOT can be implemented as XOR with 1
                        let one = builder.ins().iconst(types::I64, 1);
                        let result = builder.ins().bxor(operand_val.inner(), one);
                        Ok(CraneliftValue::new(result))
                    }
                }
            }
            EchoAst::Identifier(_) => {
                // Variable reads require runtime environment access
                return Err(anyhow!("Variable reads require runtime support, falling back to interpreter"));
            }
            EchoAst::Assignment { .. } => {
                // Variable assignment requires runtime environment access
                return Err(anyhow!("Variable assignment requires runtime support, falling back to interpreter"));
            }
            EchoAst::If { .. } => {
                // If statements require control flow branching
                return Err(anyhow!("If statements require control flow support, falling back to interpreter"));
            }
            EchoAst::While { .. } => {
                // While loops require control flow branching and loops
                return Err(anyhow!("While loops require control flow support, falling back to interpreter"));
            }
            EchoAst::For { .. } => {
                // For loops require control flow and runtime iteration
                return Err(anyhow!("For loops require control flow support, falling back to interpreter"));
            }
            EchoAst::Return { .. } => {
                // Return statements require function context
                return Err(anyhow!("Return statements require function context, falling back to interpreter"));
            }
            EchoAst::Break { .. } => {
                // Break statements require loop context
                return Err(anyhow!("Break statements require loop context, falling back to interpreter"));
            }
            EchoAst::Continue { .. } => {
                // Continue statements require loop context  
                return Err(anyhow!("Continue statements require loop context, falling back to interpreter"));
            }
            _ => Err(anyhow!(
                "AST node not yet supported in JIT compilation: {:?}",
                ast
            )),
        }
    }
}

// Implement the trait for JIT evaluator
impl EvaluatorTrait for JitEvaluator {
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

impl JitEvaluator {
    /// Evaluate an expression with control flow support
    fn eval_with_control_flow(
        &mut self,
        ast: &EchoAst,
        player_id: ObjectId,
    ) -> Result<ControlFlow> {
        match ast {
            EchoAst::Return { value } => {
                if let Some(val_ast) = value {
                    let val = self.eval_with_player(val_ast, player_id)?;
                    Ok(ControlFlow::Return(val))
                } else {
                    Ok(ControlFlow::Return(Value::Null))
                }
            }
            EchoAst::Break { label } => Ok(ControlFlow::Break(label.clone())),
            EchoAst::Continue { label } => Ok(ControlFlow::Continue(label.clone())),
            _ => {
                let val = self.eval_with_player(ast, player_id)?;
                Ok(ControlFlow::None(val))
            }
        }
    }

    /// Evaluate if statement with control flow
    fn eval_if_control(
        &mut self,
        condition: &EchoAst,
        then_branch: &[EchoAst],
        else_branch: &Option<Vec<EchoAst>>,
        player_id: ObjectId,
    ) -> Result<ControlFlow> {
        let cond_val = self.eval_with_player(condition, player_id)?;
        match cond_val {
            Value::Boolean(true) => {
                // Execute then branch
                let mut last_val = Value::Null;
                for stmt in then_branch {
                    match self.eval_with_control_flow(stmt, player_id)? {
                        ControlFlow::None(v) => last_val = v,
                        // Propagate control flow (Return, Break, Continue)
                        flow => return Ok(flow),
                    }
                }
                Ok(ControlFlow::None(last_val))
            }
            Value::Boolean(false) => {
                if let Some(else_stmts) = else_branch {
                    let mut last_val = Value::Null;
                    for stmt in else_stmts {
                        match self.eval_with_control_flow(stmt, player_id)? {
                            ControlFlow::None(v) => last_val = v,
                            flow => return Ok(flow),
                        }
                    }
                    Ok(ControlFlow::None(last_val))
                } else {
                    Ok(ControlFlow::None(Value::Null))
                }
            }
            _ => Err(anyhow!("Condition must evaluate to boolean")),
        }
    }

    /// Evaluate if statement (wrapper for compatibility)
    fn eval_if(
        &mut self,
        condition: &EchoAst,
        then_branch: &[EchoAst],
        else_branch: &Option<Vec<EchoAst>>,
        player_id: ObjectId,
    ) -> Result<Value> {
        match self.eval_if_control(condition, then_branch, else_branch, player_id)? {
            ControlFlow::None(v) => Ok(v),
            ControlFlow::Return(v) => Ok(v),
            ControlFlow::Break(_) => Err(anyhow!("Break used outside of loop")),
            ControlFlow::Continue(_) => Err(anyhow!("Continue used outside of loop")),
        }
    }

    /// Evaluate while loop
    fn eval_while(
        &mut self,
        condition: &EchoAst,
        body: &[EchoAst],
        player_id: ObjectId,
    ) -> Result<Value> {
        'outer: loop {
            let cond_val = self.eval_with_player(condition, player_id)?;
            match cond_val {
                Value::Boolean(true) => {
                    for stmt in body {
                        let flow = if let EchoAst::If { condition, then_branch, else_branch } = stmt {
                            self.eval_if_control(condition, then_branch, else_branch, player_id)?
                        } else {
                            self.eval_with_control_flow(stmt, player_id)?
                        };
                        
                        match flow {
                            ControlFlow::None(_) => {},
                            ControlFlow::Return(v) => return Ok(v),
                            ControlFlow::Break(label) => {
                                if label.is_none() {
                                    break 'outer;
                                } else {
                                    return Err(anyhow!("Labeled breaks not yet supported"));
                                }
                            }
                            ControlFlow::Continue(label) => {
                                if label.is_none() {
                                    continue 'outer;
                                } else {
                                    return Err(anyhow!("Labeled continues not yet supported"));
                                }
                            }
                        }
                    }
                }
                Value::Boolean(false) => break,
                _ => return Err(anyhow!("While condition must evaluate to boolean")),
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
        let coll_val = self.eval_with_player(collection, player_id)?;
        
        match coll_val {
            Value::List(items) => {
                'outer: for item in items {
                    // Set loop variable
                    if let Some(mut env) = self.environments.get_mut(&player_id) {
                        env.variables.insert(variable.to_string(), item);
                    }
                    
                    // Execute body
                    for stmt in body {
                        let flow = if let EchoAst::If { condition, then_branch, else_branch } = stmt {
                            self.eval_if_control(condition, then_branch, else_branch, player_id)?
                        } else {
                            self.eval_with_control_flow(stmt, player_id)?
                        };
                        
                        match flow {
                            ControlFlow::None(_) => {},
                            ControlFlow::Return(v) => return Ok(v),
                            ControlFlow::Break(label) => {
                                if label.is_none() {
                                    break 'outer;
                                } else {
                                    return Err(anyhow!("Labeled breaks not yet supported"));
                                }
                            }
                            ControlFlow::Continue(label) => {
                                if label.is_none() {
                                    continue 'outer;
                                } else {
                                    return Err(anyhow!("Labeled continues not yet supported"));
                                }
                            }
                        }
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(anyhow!("For loop requires a list")),
        }
    }
}

// Trait implementation for main evaluator is in mod.rs
