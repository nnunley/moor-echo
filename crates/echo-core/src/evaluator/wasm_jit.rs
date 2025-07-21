//! WebAssembly JIT Compiler for Echo language using Wasmtime
//!
//! This module provides a WebAssembly JIT compilation backend for the Echo language,
//! compiling rust-sitter AST nodes to WASM bytecode and executing them with Wasmtime.
//!
//! Benefits over Cranelift:
//! - Universal platform compatibility (ARM64, x86_64, etc.)
//! - Built-in sandboxing and security
//! - Rich ecosystem and tooling
//! - Better debugging support
//! - Portable bytecode format

#[cfg(feature = "wasm-jit")]
use wasmtime::{Engine, Store, Module as WasmModule, Instance};
#[cfg(feature = "wasm-jit")]
use wasm_encoder::{
    Module as WasmEncoderModule, TypeSection, FunctionSection, ExportSection, CodeSection,
    Function, Instruction, ValType, ExportKind
};

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

use crate::ast::EchoAst;
use crate::storage::{Storage, ObjectId};
use super::{Value, Environment, EvaluatorTrait};
use std::any::Any;

/// WebAssembly JIT-compiled evaluator for Echo language
pub struct WasmJitEvaluator {
    #[cfg(feature = "wasm-jit")]
    engine: Engine,
    #[cfg(feature = "wasm-jit")]
    store: Store<WasmState>,
    
    storage: Arc<Storage>,
    environments: DashMap<ObjectId, Environment>,
    current_player: Option<ObjectId>,
    
    // Compilation cache
    compiled_modules: HashMap<String, WasmModule>,
    compiled_instances: HashMap<String, Instance>,
    
    // Performance metrics
    compilation_count: usize,
    execution_count: usize,
    hot_threshold: usize,
}

/// WebAssembly runtime state
#[cfg(feature = "wasm-jit")]
struct WasmState {
    // Runtime data accessible from WASM
    current_values: Vec<Value>,
    string_table: Vec<String>,
}

impl WasmJitEvaluator {
    /// Create a new WebAssembly JIT evaluator
    pub fn new(storage: Arc<Storage>) -> Result<Self> {
        #[cfg(feature = "wasm-jit")]
        {
            let engine = Engine::default();
            let wasm_state = WasmState {
                current_values: Vec::new(),
                string_table: Vec::new(),
            };
            let store = Store::new(&engine, wasm_state);
            
            Ok(Self {
                engine,
                store,
                storage,
                environments: DashMap::new(),
                current_player: None,
                compiled_modules: HashMap::new(),
                compiled_instances: HashMap::new(),
                compilation_count: 0,
                execution_count: 0,
                hot_threshold: 5, // Lower threshold for WASM (faster compilation)
            })
        }
        
        #[cfg(not(feature = "wasm-jit"))]
        {
            Ok(Self {
                storage,
                environments: DashMap::new(),
                current_player: None,
                compiled_modules: HashMap::new(),
                compiled_instances: HashMap::new(),
                compilation_count: 0,
                execution_count: 0,
                hot_threshold: 5,
            })
        }
    }
    
    /// Create a new player
    pub fn create_player(&mut self, _name: &str) -> Result<ObjectId> {
        let player_id = ObjectId::new();
        
        // Create environment for the player
        let env = Environment {
            player_id,
            variables: HashMap::new(),
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
    
    /// Evaluate an AST node with WebAssembly JIT compilation
    pub fn eval(&mut self, ast: &EchoAst) -> Result<Value> {
        let player_id = self.current_player
            .ok_or_else(|| anyhow!("No player selected"))?;
            
        self.eval_with_player(ast, player_id)
    }
    
    /// Evaluate with specific player, using WASM JIT when beneficial
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
        #[cfg(feature = "wasm-jit")]
        {
            // For now, always use interpreter
            // In a full implementation, we'd track execution frequency
            false
        }
        
        #[cfg(not(feature = "wasm-jit"))]
        {
            false
        }
    }
    
    /// Generate a key for caching compiled modules
    fn ast_to_key(&self, ast: &EchoAst) -> String {
        // Simple key generation - in production, use a hash
        format!("{:?}", ast)
    }
    
    /// Compile AST to WebAssembly and execute
    #[cfg(feature = "wasm-jit")]
    fn compile_and_execute(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        let ast_key = self.ast_to_key(ast);
        
        // Check if already compiled
        if self.compiled_instances.contains_key(&ast_key) {
            // Get the instance (need to handle borrowing carefully)
            let _instance = self.compiled_instances.get(&ast_key).unwrap();
            // For now, we'll just use the interpreter due to borrowing complexity
            // In a real implementation, we'd need to restructure the storage
            return self.interpret(ast, player_id);
        }
        
        // For now, just use the interpreter
        // The full WASM JIT implementation would require more complex storage management
        self.interpret(ast, player_id)
    }
    
    #[cfg(not(feature = "wasm-jit"))]
    fn compile_and_execute(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // WASM JIT feature not enabled, use interpreter
        self.interpret(ast, player_id)
    }
    
    /// Compile AST to WebAssembly bytecode
    #[cfg(feature = "wasm-jit")]
    fn compile_to_wasm(&self, ast: &EchoAst) -> Result<Vec<u8>> {
        let mut module = WasmEncoderModule::new();
        
        // Add type section
        let mut types = TypeSection::new();
        types.ty().function([], [ValType::I64]); // Function type: () -> i64
        module.section(&types);
        
        // Add function section
        let mut functions = FunctionSection::new();
        functions.function(0); // Function 0 uses type 0
        module.section(&functions);
        
        // Add export section
        let mut exports = ExportSection::new();
        exports.export("eval", ExportKind::Func, 0);
        module.section(&exports);
        
        // Add code section
        let mut codes = CodeSection::new();
        let mut func_body = Function::new([]);
        
        // Generate WASM instructions for the AST
        self.compile_ast_to_wasm(ast, &mut func_body)?;
        
        codes.function(&func_body);
        module.section(&codes);
        
        Ok(module.finish())
    }
    
    /// Generate WASM instructions for an AST node
    #[cfg(feature = "wasm-jit")]
    fn compile_ast_to_wasm(&self, ast: &EchoAst, func: &mut Function) -> Result<()> {
        match ast {
            EchoAst::Number(n) => {
                // Push the number as i64 constant
                func.instruction(&Instruction::I64Const(*n));
                Ok(())
            }
            EchoAst::Add { left, right } => {
                // Compile left operand
                self.compile_ast_to_wasm(left, func)?;
                // Compile right operand
                self.compile_ast_to_wasm(right, func)?;
                // Add them
                func.instruction(&Instruction::I64Add);
                Ok(())
            }
            _ => Err(anyhow!("AST node not yet supported in WASM JIT compilation: {:?}", ast)),
        }
    }
    
    /// Execute a compiled WASM instance
    #[cfg(feature = "wasm-jit")]
    fn execute_wasm_instance(&mut self, instance: &Instance, _player_id: ObjectId) -> Result<Value> {
        // Get the exported function
        let eval_func = instance.get_typed_func::<(), i64>(&mut self.store, "eval")?;
        
        // Call the function
        let result = eval_func.call(&mut self.store, ())?;
        
        // Convert result back to our Value type
        Ok(Value::Integer(result))
    }
    
    /// Interpret AST using the same logic as the main evaluator
    fn interpret(&mut self, ast: &EchoAst, player_id: ObjectId) -> Result<Value> {
        // For now, we'll implement a simplified interpreter
        // In a full implementation, this would be identical to the main evaluator
        match ast {
            EchoAst::Number(n) => Ok(Value::Integer(*n)),
            EchoAst::String(s) => Ok(Value::String(s.clone())),
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
                    (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    _ => Err(anyhow!("Type error in addition")),
                }
            }
            _ => {
                // For other AST nodes, delegate to main evaluator for now
                // In a full implementation, we'd handle all cases
                Err(anyhow!("AST node not yet implemented in WASM JIT evaluator: {:?}", ast))
            }
        }
    }
    
    /// Get performance statistics
    pub fn stats(&self) -> WasmJitStats {
        WasmJitStats {
            compilation_count: self.compilation_count,
            execution_count: self.execution_count,
            compiled_modules: self.compiled_modules.len(),
            compiled_instances: self.compiled_instances.len(),
            hot_threshold: self.hot_threshold,
        }
    }
}

/// Performance statistics for WebAssembly JIT compilation
#[derive(Debug, Clone)]
pub struct WasmJitStats {
    pub compilation_count: usize,
    pub execution_count: usize,
    pub compiled_modules: usize,
    pub compiled_instances: usize,
    pub hot_threshold: usize,
}

// Implement the trait for WASM JIT evaluator
impl EvaluatorTrait for WasmJitEvaluator {
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
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}