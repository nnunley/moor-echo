/*!
# Property Syntax Fixer

Transformation rule to fix MOO property syntax parsing issues.
Converts problematic MOO property syntax to Echo-compatible forms.
*/

use anyhow::{anyhow, Result};

use crate::ast::{EchoAst, ObjectMember, LValue};
use crate::tracer::rules::TransformationRule;
use crate::tracer::{TransformResult, TransformationContext};

/// Fixes MOO property syntax issues that the parser can't handle
/// 
/// This rule addresses the specific parsing issues we encountered with
/// MOO files like sub.moo where property syntax like "owner: HACKER"
/// causes parser errors.
pub struct PropertySyntaxFixer {
    priority: u32,
}

impl PropertySyntaxFixer {
    pub fn new() -> Self {
        Self { priority: 200 }
    }
    
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

impl Default for PropertySyntaxFixer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformationRule for PropertySyntaxFixer {
    fn name(&self) -> &'static str {
        "PropertySyntaxFixer"
    }
    
    fn description(&self) -> &'static str {
        "Fixes MOO property syntax parsing issues by converting problematic syntax to Echo-compatible forms"
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn matches(&self, ast: &EchoAst, _context: &TransformationContext) -> bool {
        match ast {
            // Look for object definitions that might have property syntax issues
            EchoAst::ObjectDef { members, .. } => {
                members.iter().any(|member| {
                    matches!(member, ObjectMember::Property { .. })
                })
            }
            // Also look for standalone assignments that might be property definitions
            EchoAst::Assignment { .. } => true,
            _ => false,
        }
    }
    
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        match ast {
            EchoAst::ObjectDef { name, parent, members } => {
                let fixed_members: Result<Vec<_>> = members
                    .into_iter()
                    .map(|member| self.fix_object_member(member, context))
                    .collect();
                
                Ok(EchoAst::ObjectDef {
                    name,
                    parent,
                    members: fixed_members?,
                })
            }
            
            EchoAst::Assignment { target, value } => {
                // Check if this looks like a MOO property definition that needs fixing
                let fixed_assignment = self.fix_assignment(target, *value, context)?;
                Ok(fixed_assignment)
            }
            
            _ => Ok(ast),
        }
    }
    
    fn validate(&self, original: &EchoAst, transformed: &EchoAst, _context: &TransformationContext) -> TransformResult<()> {
        // Basic validation - ensure we haven't changed the fundamental structure
        if let (EchoAst::ObjectDef { name: orig_name, .. }, EchoAst::ObjectDef { name: trans_name, .. }) = (original, transformed) {
            if orig_name != trans_name {
                return Err(anyhow!("Object name changed during transformation: {} -> {}", orig_name, trans_name));
            }
        }
        
        Ok(())
    }
}

impl PropertySyntaxFixer {
    /// Fix property syntax in object members
    fn fix_object_member(&self, member: ObjectMember, _context: &TransformationContext) -> Result<ObjectMember> {
        match member {
            ObjectMember::Property { name, value, permissions, required_capabilities } => {
                // Fix common MOO property syntax issues
                let fixed_value = self.fix_property_value(value)?;
                
                Ok(ObjectMember::Property {
                    name,
                    value: fixed_value,
                    permissions,
                    required_capabilities,
                })
            }
            
            ObjectMember::Verb { name, args, body, permissions, required_capabilities } => {
                // Fix verb bodies that might have property-related syntax issues
                let fixed_body: Result<Vec<_>> = body
                    .into_iter()
                    .map(|stmt| self.fix_statement(stmt))
                    .collect();
                
                Ok(ObjectMember::Verb {
                    name,
                    args,
                    body: fixed_body?,
                    permissions,
                    required_capabilities,
                })
            }
            
            // Other member types pass through unchanged
            other => Ok(other),
        }
    }
    
    /// Fix property values that might have MOO-specific syntax
    fn fix_property_value(&self, value: EchoAst) -> Result<EchoAst> {
        match value {
            // Convert MOO object references to proper Echo references
            EchoAst::Identifier(name) => {
                // Check if this looks like a MOO constant that should be an object reference
                match name.as_str() {
                    "HACKER" | "ROOT" | "PLAYER" | "BUILDER" | "PROG" | "WIZ" | 
                    "SYSOBJ" | "ARCH_WIZARD" | "ROOM" | "THING" | "STRING" | 
                    "PASSWORD" | "FIRST_ROOM" | "LOGIN" | "EVENT" | "SUB" | 
                    "BLOCK" | "LOOK" | "LIST" => {
                        // These should be treated as object references, not identifiers
                        // For now, convert to property access on system object
                        Ok(EchoAst::PropertyAccess {
                            object: Box::new(EchoAst::ObjectRef(0)), // #0 (system object)
                            property: name,
                        })
                    }
                    _ => Ok(EchoAst::Identifier(name)),
                }
            }
            
            // Recursively fix other AST nodes
            other => Ok(other),
        }
    }
    
    /// Fix assignment statements that might be property definitions
    fn fix_assignment(&self, target: LValue, value: EchoAst, _context: &TransformationContext) -> Result<EchoAst> {
        let fixed_value = self.fix_property_value(value)?;
        
        Ok(EchoAst::Assignment {
            target,
            value: Box::new(fixed_value),
        })
    }
    
    /// Fix statements that might contain property syntax issues
    fn fix_statement(&self, stmt: EchoAst) -> Result<EchoAst> {
        match stmt {
            EchoAst::Assignment { target, value } => {
                self.fix_assignment(target, *value, &TransformationContext::new())
            }
            
            // Add more statement types as needed
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ObjectMember, LValue, BindingType, BindingPattern};
    
    #[test]
    fn test_property_syntax_fixer_matches() {
        let fixer = PropertySyntaxFixer::new();
        
        // Should match object definitions with properties
        let obj_def = EchoAst::ObjectDef {
            name: "TEST".to_string(),
            parent: None,
            members: vec![
                ObjectMember::Property {
                    name: "owner".to_string(),
                    value: EchoAst::Identifier("HACKER".to_string()),
                    permissions: None,
                    required_capabilities: Vec::new(),
                }
            ],
        };
        
        assert!(fixer.matches(&obj_def, &TransformationContext::new()));
        
        // Should match assignments
        let assignment = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("test".to_string()),
            },
            value: Box::new(EchoAst::Identifier("HACKER".to_string())),
        };
        
        assert!(fixer.matches(&assignment, &TransformationContext::new()));
    }
    
    #[test]
    fn test_fix_moo_constants() {
        let fixer = PropertySyntaxFixer::new();
        let context = TransformationContext::new();
        
        let obj_def = EchoAst::ObjectDef {
            name: "TEST".to_string(),
            parent: None,
            members: vec![
                ObjectMember::Property {
                    name: "owner".to_string(),
                    value: EchoAst::Identifier("HACKER".to_string()),
                    permissions: None,
                    required_capabilities: Vec::new(),
                }
            ],
        };
        
        let result = fixer.transform(obj_def, &context).unwrap();
        
        if let EchoAst::ObjectDef { members, .. } = result {
            if let ObjectMember::Property { value, .. } = &members[0] {
                match value {
                    EchoAst::PropertyAccess { object, property } => {
                        assert!(matches!(**object, EchoAst::ObjectRef(0)));
                        assert_eq!(property, "HACKER");
                    }
                    _ => panic!("Expected PropertyAccess, got {:?}", value),
                }
            } else {
                panic!("Expected Property member");
            }
        } else {
            panic!("Expected ObjectDef");
        }
    }
}