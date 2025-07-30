use thiserror::Error;

use crate::storage::ObjectId;

/// Evaluator-specific error types for better error handling
#[derive(Error, Debug)]
pub enum EvaluatorError {
    #[error("Type error: {operation} requires {expected}, got {actual}")]
    TypeError {
        operation: String,
        expected: String,
        actual: String,
    },

    #[error("Type error: cannot {operation} {left_type} and {right_type}")]
    BinaryTypeError {
        operation: String,
        left_type: String,
        right_type: String,
    },

    #[error("Object not found: {id:?}")]
    ObjectNotFound { id: ObjectId },

    #[error("Property '{property}' not found on object {object:?}")]
    PropertyNotFound { property: String, object: ObjectId },

    #[error("Verb '{verb}' not found on object {object:?}")]
    VerbNotFound { verb: String, object: ObjectId },

    #[error("Variable '{name}' not found")]
    VariableNotFound { name: String },

    #[error("Permission denied: cannot {action} on {target}")]
    PermissionDenied { action: String, target: String },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl EvaluatorError {
    /// Create a type error for unary operations
    pub fn unary_type_error(operation: &str, expected: &str, actual: &str) -> Self {
        Self::TypeError {
            operation: operation.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create a type error for binary operations
    pub fn binary_type_error(operation: &str, left_type: &str, right_type: &str) -> Self {
        Self::BinaryTypeError {
            operation: operation.to_string(),
            left_type: left_type.to_string(),
            right_type: right_type.to_string(),
        }
    }

    /// Create a property not found error
    pub fn property_not_found(property: &str, object: ObjectId) -> Self {
        Self::PropertyNotFound {
            property: property.to_string(),
            object,
        }
    }

    /// Create a verb not found error
    pub fn verb_not_found(verb: &str, object: ObjectId) -> Self {
        Self::VerbNotFound {
            verb: verb.to_string(),
            object,
        }
    }

    /// Create a variable not found error
    pub fn variable_not_found(name: &str) -> Self {
        Self::VariableNotFound {
            name: name.to_string(),
        }
    }
}
