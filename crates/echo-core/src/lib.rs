//! # Echo Core
//! 
//! Core implementation of the Echo programming language, including:
//! - Abstract Syntax Tree (AST) definitions
//! - Language parser and grammar
//! - Expression evaluator and runtime
//! - Persistent object storage
//! - Event system and runtime components
//!
//! This crate provides the foundational components that can be used to build
//! various Echo language interfaces (REPL, web service, embedded runtime, etc.)

//#![deny(missing_docs)]  // Temporarily disabled during crate extraction
#![warn(clippy::all)]

pub mod ast;
pub mod evaluator;
pub mod parser;
pub mod storage;

pub mod runtime;
pub mod ui_callback;

pub mod security;

// Re-export commonly used types
pub use ast::{EchoAst, LValue, BindingType, ObjectMember, LambdaParam};
pub use evaluator::{
    Evaluator, EvaluatorTrait, Value, Environment, ControlFlow,
    meta_object::MetaObject,
    event_system::{EventSystem, Event, EventHandler, EventResult},
};
pub use parser::{Parser, create_parser};
pub use storage::{
    Storage, ObjectId, EchoObject, PropertyValue,
    object_store::ObjectStore,
};
pub use runtime::EchoRuntime;
pub use ui_callback::{UiEvent, UiAction, UiEventHandler, UiEventCallback, convert_ui_event};

// Re-export storage error from the storage module
pub use crate::storage::StorageError;

/// Echo language version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Echo language features supported by this build
pub fn features() -> Vec<&'static str> {
    #[allow(unused_mut)]
    let mut features = vec!["core"];
    
    #[cfg(feature = "jit")]
    features.push("jit");
    
    #[cfg(feature = "reflection")]
    features.push("reflection");
    
    features
}

/// Initialize tracing for Echo core components
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("echo_core=info".parse().unwrap())
        )
        .init();
}

/// Core Echo runtime configuration
#[derive(Debug, Clone)]
pub struct EchoConfig {
    /// Database storage path
    pub storage_path: std::path::PathBuf,
    /// Enable debug mode
    pub debug: bool,
    /// Maximum object count
    pub max_objects: usize,
    /// Maximum evaluation depth
    pub max_eval_depth: usize,
    /// Enable JIT compilation
    pub enable_jit: bool,
}

impl Default for EchoConfig {
    fn default() -> Self {
        Self {
            storage_path: "./echo-db".into(),
            debug: false,
            max_objects: 100_000,
            max_eval_depth: 1000,
            enable_jit: false,
        }
    }
}

/// Error types for Echo core operations
#[derive(thiserror::Error, Debug)]
pub enum EchoError {
    /// Storage-related error
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    
    /// Parser error
    #[error("Parse error: {0}")]
    Parse(#[from] anyhow::Error),
    
    /// Evaluation error
    #[error("Evaluation error: {0}")]
    Evaluation(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type for Echo core operations
pub type Result<T> = std::result::Result<T, EchoError>;

