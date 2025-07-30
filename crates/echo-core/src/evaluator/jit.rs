//! JIT Compiler for Echo language using Cranelift
//!
//! This module provides a JIT compilation backend for the Echo language,
//! compiling rust-sitter AST nodes to native machine code for performance.

#[cfg(feature = "jit")]
use cranelift::prelude::*;
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
    ast::EchoAst,
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
            EchoAst::Number(_) | EchoAst::Add { .. } => {
                // These are the only AST types we support compiling so far
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
                        eprintln!("JIT compilation failed: {}", e);
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
                    (Value::String(l), Value::String(r)) => {
                        Ok(Value::String(format!("{}{}", l, r)))
                    }
                    _ => Err(anyhow!("Type error in addition")),
                }
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
            let builder_context = self.builder_context.as_mut().ok_or_else(|| anyhow!("JIT builder context not available"))?;

            // Clear previous function
            ctx.func.clear();

            // Set up function signature
            let int_type = module.target_config().pointer_type();
            ctx.func.signature.returns.push(AbiParam::new(int_type));

            // Build the function
            let mut builder = FunctionBuilder::new(&mut ctx.func, builder_context);
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
            EchoAst::Add { left, right } => {
                let left_val = Self::compile_ast_node(left, builder)?;
                let right_val = Self::compile_ast_node(right, builder)?;
                let result = builder.ins().iadd(left_val.inner(), right_val.inner());
                Ok(CraneliftValue::new(result))
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

// Trait implementation for main evaluator is in mod.rs
