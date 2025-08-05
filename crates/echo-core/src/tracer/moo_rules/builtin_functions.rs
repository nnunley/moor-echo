/*!
# Builtin Function Resolver

Transformation rule to resolve MOO builtin function calls and ensure
they're properly handled by the Echo evaluator.
*/

use anyhow::Result;

use crate::ast::EchoAst;
use crate::tracer::rules::TransformationRule;
use crate::tracer::{TransformResult, TransformationContext};

/// Resolves MOO builtin function calls to ensure proper evaluation
/// 
/// This rule addresses issues where MOO builtin functions like `valid()`,
/// `typeof()`, `notify()` etc. are not being properly resolved during evaluation.
pub struct BuiltinFunctionResolver {
    priority: u32,
    known_builtins: Vec<String>,
    add_explicit_calls: bool,
}

impl BuiltinFunctionResolver {
    pub fn new() -> Self {
        Self {
            priority: 180,
            known_builtins: vec![
                "valid".to_string(),
                "typeof".to_string(),
                "notify".to_string(),
                "read".to_string(),
                "tostr".to_string(),
                "length".to_string(),
                "raise".to_string(),
                "boot_player".to_string(),
                "players".to_string(),
                "connected_players".to_string(),
                "is_player".to_string(),
                "create".to_string(),
                "recycle".to_string(),
                "move".to_string(),
                "parent".to_string(),
                "children".to_string(),
                "max_object".to_string(),
                "random".to_string(),
                "time".to_string(),
                "ctime".to_string(),
                "strcmp".to_string(),
                "index".to_string(),
                "rindex".to_string(),
                "substitute".to_string(),
                "match".to_string(),
                "rmatch".to_string(),
                "explode".to_string(),
                "implode".to_string(),
            ],
            add_explicit_calls: true,
        }
    }
    
    /// Add custom builtin function names
    pub fn add_builtin(mut self, name: String) -> Self {
        self.known_builtins.push(name);
        self
    }
    
    /// Enable/disable explicit call generation
    pub fn add_explicit_calls(mut self, enable: bool) -> Self {
        self.add_explicit_calls = enable;
        self
    }
    
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

impl Default for BuiltinFunctionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformationRule for BuiltinFunctionResolver {
    fn name(&self) -> &'static str {
        "BuiltinFunctionResolver"
    }
    
    fn description(&self) -> &'static str {
        "Resolves MOO builtin function calls to ensure proper evaluation by Echo"
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn matches(&self, ast: &EchoAst, _context: &TransformationContext) -> bool {
        match ast {
            // Match function calls that might be builtins
            EchoAst::Call { func, .. } => {
                match func.as_ref() {
                    EchoAst::Identifier(name) => self.known_builtins.contains(name),
                    _ => false,
                }
            }
            
            // Match identifiers that might be builtin function names
            EchoAst::Identifier(name) => {
                self.known_builtins.contains(name)
            }
            
            // Match any container that might have builtin calls
            EchoAst::Program(_) | EchoAst::Block(_) => true,
            
            _ => false,
        }
    }
    
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        match ast {
            EchoAst::Call { func, args } => {
                self.resolve_builtin_call(*func, args, context)
            }
            
            EchoAst::Identifier(name) if self.known_builtins.contains(&name) => {
                self.resolve_builtin_identifier(name, context)
            }
            
            EchoAst::Program(stmts) => {
                let resolved_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform(stmt, context))
                    .collect();
                
                Ok(EchoAst::Program(resolved_stmts?))
            }
            
            EchoAst::Block(stmts) => {
                let resolved_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform(stmt, context))
                    .collect();
                
                Ok(EchoAst::Block(resolved_stmts?))
            }
            
            // Recursively transform other node types that might contain calls
            EchoAst::If { condition, then_branch, else_branch } => {
                Ok(EchoAst::If {
                    condition: Box::new(self.transform(*condition, context)?),
                    then_branch: then_branch
                        .into_iter()
                        .map(|stmt| self.transform(stmt, context))
                        .collect::<Result<Vec<_>>>()?,
                    else_branch: else_branch
                        .map(|stmts| 
                            stmts
                                .into_iter()
                                .map(|stmt| self.transform(stmt, context))
                                .collect::<Result<Vec<_>>>()
                        )
                        .transpose()?,
                })
            }
            
            EchoAst::While { label, condition, body } => {
                Ok(EchoAst::While {
                    label,
                    condition: Box::new(self.transform(*condition, context)?),
                    body: body
                        .into_iter()
                        .map(|stmt| self.transform(stmt, context))
                        .collect::<Result<Vec<_>>>()?,
                })
            }
            
            // Pass through other node types unchanged
            other => Ok(other),
        }
    }
}

impl BuiltinFunctionResolver {
    /// Resolve a builtin function call
    fn resolve_builtin_call(&self, func: EchoAst, args: Vec<EchoAst>, context: &TransformationContext) -> TransformResult<EchoAst> {
        match func {
            EchoAst::Identifier(name) if self.known_builtins.contains(&name) => {
                // Transform arguments recursively
                let resolved_args: Result<Vec<_>> = args
                    .into_iter()
                    .map(|arg| self.transform(arg, context))
                    .collect();
                
                if self.add_explicit_calls {
                    // Generate explicit builtin call
                    self.generate_explicit_builtin_call(name, resolved_args?)
                } else {
                    // Keep as regular call but ensure it's marked as builtin
                    Ok(EchoAst::Call {
                        func: Box::new(EchoAst::Identifier(name)),
                        args: resolved_args?,
                    })
                }
            }
            
            _ => {
                // Not a builtin, transform recursively
                let resolved_func = self.transform(func, context)?;
                let resolved_args: Result<Vec<_>> = args
                    .into_iter()
                    .map(|arg| self.transform(arg, context))
                    .collect();
                
                Ok(EchoAst::Call {
                    func: Box::new(resolved_func),
                    args: resolved_args?,
                })
            }
        }
    }
    
    /// Resolve a builtin function identifier (when used without call)
    fn resolve_builtin_identifier(&self, name: String, _context: &TransformationContext) -> TransformResult<EchoAst> {
        if self.add_explicit_calls {
            // Generate a reference to the builtin function
            // This could be a property access on a builtins object
            Ok(EchoAst::PropertyAccess {
                object: Box::new(EchoAst::Identifier("$builtins".to_string())),
                property: name,
            })
        } else {
            // Keep as identifier
            Ok(EchoAst::Identifier(name))
        }
    }
    
    /// Generate explicit builtin function call
    fn generate_explicit_builtin_call(&self, name: String, args: Vec<EchoAst>) -> TransformResult<EchoAst> {
        match name.as_str() {
            // Special handling for specific builtins
            "valid" => {
                // Generate: $builtins:valid(arg)
                Ok(EchoAst::MethodCall {
                    object: Box::new(EchoAst::Identifier("$builtins".to_string())),
                    method: "valid".to_string(),
                    args,
                })
            }
            
            "typeof" => {
                // Generate: $builtins:typeof(arg)
                Ok(EchoAst::MethodCall {
                    object: Box::new(EchoAst::Identifier("$builtins".to_string())),
                    method: "typeof".to_string(),
                    args,
                })
            }
            
            "notify" => {
                // Generate: $builtins:notify(player, message)
                Ok(EchoAst::MethodCall {
                    object: Box::new(EchoAst::Identifier("$builtins".to_string())),
                    method: "notify".to_string(),
                    args,
                })
            }
            
            // Default: generate method call on $builtins
            _ => {
                Ok(EchoAst::MethodCall {
                    object: Box::new(EchoAst::Identifier("$builtins".to_string())),
                    method: name,
                    args,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_resolver_matches() {
        let resolver = BuiltinFunctionResolver::new();
        let context = TransformationContext::new();
        
        // Should match builtin function calls
        let call = EchoAst::Call {
            func: Box::new(EchoAst::Identifier("valid".to_string())),
            args: vec![EchoAst::ObjectRef(1)],
        };
        assert!(resolver.matches(&call, &context));
        
        // Should match builtin identifiers
        assert!(resolver.matches(&EchoAst::Identifier("typeof".to_string()), &context));
        
        // Should not match non-builtin functions
        let non_builtin_call = EchoAst::Call {
            func: Box::new(EchoAst::Identifier("custom_function".to_string())),
            args: vec![],
        };
        assert!(!resolver.matches(&non_builtin_call, &context));
    }
    
    #[test]
    fn test_resolve_builtin_call() {
        let resolver = BuiltinFunctionResolver::new();
        let context = TransformationContext::new();
        
        let call = EchoAst::Call {
            func: Box::new(EchoAst::Identifier("valid".to_string())),
            args: vec![EchoAst::ObjectRef(1)],
        };
        
        let result = resolver.transform(call, &context).unwrap();
        
        // Should generate method call on $builtins
        match result {
            EchoAst::MethodCall { object, method, args } => {
                assert!(matches!(*object, EchoAst::Identifier(name) if name == "$builtins"));
                assert_eq!(method, "valid");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], EchoAst::ObjectRef(1)));
            }
            _ => panic!("Expected MethodCall, got {:?}", result),
        }
    }
    
    #[test]
    fn test_resolve_builtin_identifier() {
        let resolver = BuiltinFunctionResolver::new();
        let context = TransformationContext::new();
        
        let identifier = EchoAst::Identifier("typeof".to_string());
        let result = resolver.transform(identifier, &context).unwrap();
        
        // Should generate property access on $builtins
        match result {
            EchoAst::PropertyAccess { object, property } => {
                assert!(matches!(*object, EchoAst::Identifier(name) if name == "$builtins"));
                assert_eq!(property, "typeof");
            }
            _ => panic!("Expected PropertyAccess, got {:?}", result),
        }
    }
}