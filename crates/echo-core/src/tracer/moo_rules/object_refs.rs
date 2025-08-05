/*!
# Object Reference Normalizer

Transformation rule to normalize object references across the codebase.
Handles negative object references, connection objects, and error constants consistently.
*/

use anyhow::Result;

use crate::ast::EchoAst;
use crate::tracer::rules::TransformationRule;
use crate::tracer::{TransformResult, TransformationContext};

/// Normalizes object reference handling across the codebase
/// 
/// This rule ensures consistent handling of:
/// - Negative object references (connections vs constants)
/// - MOO object number mappings
/// - Object reference resolution patterns
pub struct ObjectReferenceNormalizer {
    priority: u32,
    normalize_negative_refs: bool,
    preserve_connection_semantics: bool,
}

impl ObjectReferenceNormalizer {
    pub fn new() -> Self {
        Self {
            priority: 150,
            normalize_negative_refs: true,
            preserve_connection_semantics: true,
        }
    }
    
    /// Enable/disable normalization of negative object references
    pub fn normalize_negative_refs(mut self, enable: bool) -> Self {
        self.normalize_negative_refs = enable;
        self
    }
    
    /// Enable/disable preservation of connection object semantics
    pub fn preserve_connection_semantics(mut self, enable: bool) -> Self {
        self.preserve_connection_semantics = enable;
        self
    }
    
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

impl Default for ObjectReferenceNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformationRule for ObjectReferenceNormalizer {
    fn name(&self) -> &'static str {
        "ObjectReferenceNormalizer"
    }
    
    fn description(&self) -> &'static str {
        "Normalizes object reference handling including negative references, connections, and error constants"
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn matches(&self, ast: &EchoAst, _context: &TransformationContext) -> bool {
        match ast {
            // Match object references
            EchoAst::ObjectRef(_) => true,
            
            // Match expressions that might contain object references
            EchoAst::Call { .. } => true,
            EchoAst::MethodCall { .. } => true,
            EchoAst::PropertyAccess { .. } => true,
            
            // Match any node that might contain object references
            EchoAst::Program(_) | EchoAst::Block(_) => true,
            
            _ => false,
        }
    }
    
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        match ast {
            EchoAst::ObjectRef(num) => {
                self.normalize_object_reference(num, context)
            }
            
            EchoAst::Call { func, args } => {
                let normalized_func = Box::new(self.transform(*func, context)?);
                let normalized_args: Result<Vec<_>> = args
                    .into_iter()
                    .map(|arg| self.transform(arg, context))
                    .collect();
                
                Ok(EchoAst::Call {
                    func: normalized_func,
                    args: normalized_args?,
                })
            }
            
            EchoAst::MethodCall { object, method, args } => {
                let normalized_object = Box::new(self.transform(*object, context)?);
                let normalized_args: Result<Vec<_>> = args
                    .into_iter()
                    .map(|arg| self.transform(arg, context))
                    .collect();
                
                Ok(EchoAst::MethodCall {
                    object: normalized_object,
                    method,
                    args: normalized_args?,
                })
            }
            
            EchoAst::PropertyAccess { object, property } => {
                let normalized_object = Box::new(self.transform(*object, context)?);
                
                Ok(EchoAst::PropertyAccess {
                    object: normalized_object,
                    property,
                })
            }
            
            EchoAst::Program(stmts) => {
                let normalized_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform(stmt, context))
                    .collect();
                
                Ok(EchoAst::Program(normalized_stmts?))
            }
            
            EchoAst::Block(stmts) => {
                let normalized_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform(stmt, context))
                    .collect();
                
                Ok(EchoAst::Block(normalized_stmts?))
            }
            
            // Pass through other node types unchanged
            other => Ok(other),
        }
    }
}

impl ObjectReferenceNormalizer {
    /// Normalize a specific object reference number
    fn normalize_object_reference(&self, num: i64, context: &TransformationContext) -> TransformResult<EchoAst> {
        if num < 0 && self.normalize_negative_refs {
            self.normalize_negative_reference(num, context)
        } else if num >= 0 {
            self.normalize_positive_reference(num, context)
        } else {
            // Keep negative references as-is if normalization is disabled
            Ok(EchoAst::ObjectRef(num))
        }
    }
    
    /// Handle negative object references (connections vs constants)
    fn normalize_negative_reference(&self, num: i64, _context: &TransformationContext) -> TransformResult<EchoAst> {
        if self.preserve_connection_semantics {
            // Generate code that checks for connection objects at runtime
            // This creates a conditional that prefers connections over constants
            self.generate_connection_aware_reference(num)
        } else {
            // Treat all negative references as constants
            Ok(EchoAst::ObjectRef(num))
        }
    }
    
    /// Handle positive object references
    fn normalize_positive_reference(&self, num: i64, _context: &TransformationContext) -> TransformResult<EchoAst> {
        // For positive references, we can add object mapping logic
        match num {
            0 => {
                // System object - always valid
                Ok(EchoAst::ObjectRef(0))
            }
            1 => {
                // Root object - always valid
                Ok(EchoAst::ObjectRef(1))
            }
            _ => {
                // For other references, we might want to add runtime resolution
                self.generate_mapped_object_reference(num)
            }
        }
    }
    
    /// Generate connection-aware object reference code
    fn generate_connection_aware_reference(&self, num: i64) -> TransformResult<EchoAst> {
        // This generates Echo code that looks like:
        // `connection_object(#-1) ! E_ARGS => #-1'
        // Which tries to resolve as connection first, falls back to constant
        
        Ok(EchoAst::ErrorCatch {
            expr: Box::new(EchoAst::Call {
                func: Box::new(EchoAst::Identifier("connection_object".to_string())),
                args: vec![EchoAst::ObjectRef(num)],
            }),
            error_patterns: vec!["E_ARGS".to_string(), "E_INVARG".to_string()],
            default: Box::new(EchoAst::ObjectRef(num)), // Fallback to constant
        })
    }
    
    /// Generate mapped object reference code
    fn generate_mapped_object_reference(&self, num: i64) -> TransformResult<EchoAst> {
        // This generates Echo code that looks like:
        // `resolve_object_ref(#123)'
        // Which handles object mapping at runtime
        
        Ok(EchoAst::Call {
            func: Box::new(EchoAst::Identifier("resolve_object_ref".to_string())),
            args: vec![EchoAst::ObjectRef(num)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalizer_matches() {
        let normalizer = ObjectReferenceNormalizer::new();
        let context = TransformationContext::new();
        
        // Should match object references
        assert!(normalizer.matches(&EchoAst::ObjectRef(123), &context));
        assert!(normalizer.matches(&EchoAst::ObjectRef(-1), &context));
        
        // Should match calls that might contain object references
        let call = EchoAst::Call {
            func: Box::new(EchoAst::Identifier("test".to_string())),
            args: vec![EchoAst::ObjectRef(123)],
        };
        assert!(normalizer.matches(&call, &context));
    }
    
    #[test]
    fn test_normalize_negative_reference() {
        let normalizer = ObjectReferenceNormalizer::new();
        let context = TransformationContext::new();
        
        let result = normalizer.transform(EchoAst::ObjectRef(-1), &context).unwrap();
        
        // Should generate connection-aware reference
        match result {
            EchoAst::ErrorCatch { expr, .. } => {
                match *expr {
                    EchoAst::Call { func, args } => {
                        assert!(matches!(*func, EchoAst::Identifier(name) if name == "connection_object"));
                        assert_eq!(args.len(), 1);
                        assert!(matches!(args[0], EchoAst::ObjectRef(-1)));
                    }
                    _ => panic!("Expected Call in ErrorCatch expr"),
                }
            }
            _ => panic!("Expected ErrorCatch for negative reference"),
        }
    }
    
    #[test]
    fn test_normalize_positive_reference() {
        let normalizer = ObjectReferenceNormalizer::new();
        let context = TransformationContext::new();
        
        // System and root objects should pass through unchanged
        let result = normalizer.transform(EchoAst::ObjectRef(0), &context).unwrap();
        assert!(matches!(result, EchoAst::ObjectRef(0)));
        
        let result = normalizer.transform(EchoAst::ObjectRef(1), &context).unwrap();
        assert!(matches!(result, EchoAst::ObjectRef(1)));
        
        // Other objects should get mapping calls
        let result = normalizer.transform(EchoAst::ObjectRef(123), &context).unwrap();
        match result {
            EchoAst::Call { func, args } => {
                assert!(matches!(*func, EchoAst::Identifier(name) if name == "resolve_object_ref"));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], EchoAst::ObjectRef(123)));
            }
            _ => panic!("Expected Call for positive object reference"),
        }
    }
}