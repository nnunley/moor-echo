/*!
# Transformation Rules

Core trait and utilities for defining transformation rules.
*/

use crate::ast::EchoAst;
use super::{TransformResult, TransformationContext};

/// Core trait for transformation rules
/// 
/// Similar to SystemTracer's transformation methods, this provides a way to
/// systematically transform AST nodes based on patterns and conditions.
pub trait TransformationRule: Send + Sync {
    /// Human-readable name for this rule
    fn name(&self) -> &'static str;
    
    /// Detailed description of what this rule does
    fn description(&self) -> &'static str;
    
    /// Priority for rule ordering (higher priority runs first)
    fn priority(&self) -> u32 {
        100
    }
    
    /// Check if this rule applies to the given AST node
    fn matches(&self, ast: &EchoAst, context: &TransformationContext) -> bool;
    
    /// Apply the transformation to the AST node
    /// 
    /// Returns the transformed AST, or the original if no transformation was applied.
    /// Should handle the transformation recursively if needed.
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst>;
    
    /// Optional validation that the transformation was successful
    fn validate(&self, original: &EchoAst, transformed: &EchoAst, context: &TransformationContext) -> TransformResult<()> {
        // Default implementation does no validation
        let _ = (original, transformed, context);
        Ok(())
    }
    
    /// Check if this rule conflicts with another rule
    fn conflicts_with(&self, other: &dyn TransformationRule) -> bool {
        // Default: no conflicts
        let _ = other;
        false
    }
}

/// A composite rule that applies multiple rules in sequence
pub struct CompositeRule {
    pub name: &'static str,
    pub description: &'static str,
    pub rules: Vec<Box<dyn TransformationRule>>,
}

impl CompositeRule {
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            rules: Vec::new(),
        }
    }
    
    pub fn add_rule(mut self, rule: Box<dyn TransformationRule>) -> Self {
        self.rules.push(rule);
        self
    }
}

impl TransformationRule for CompositeRule {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn description(&self) -> &'static str {
        self.description
    }
    
    fn matches(&self, ast: &EchoAst, context: &TransformationContext) -> bool {
        // A composite rule matches if any of its sub-rules match
        self.rules.iter().any(|rule| rule.matches(ast, context))
    }
    
    fn transform(&self, mut ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        // Apply each rule in sequence
        for rule in &self.rules {
            if rule.matches(&ast, context) {
                ast = rule.transform(ast, context)?;
            }
        }
        Ok(ast)
    }
}

/// A rule that only applies to specific AST node types
pub struct TypedRule<F> 
where 
    F: Fn(EchoAst, &TransformationContext) -> TransformResult<EchoAst> + Send + Sync,
{
    pub name: &'static str,
    pub description: &'static str,
    pub priority: u32,
    pub matcher: fn(&EchoAst) -> bool,
    pub transformer: F,
}

impl<F> TypedRule<F>
where
    F: Fn(EchoAst, &TransformationContext) -> TransformResult<EchoAst> + Send + Sync,
{
    pub fn new(
        name: &'static str,
        description: &'static str,
        matcher: fn(&EchoAst) -> bool,
        transformer: F,
    ) -> Self {
        Self {
            name,
            description,
            priority: 100,
            matcher,
            transformer,
        }
    }
    
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

impl<F> TransformationRule for TypedRule<F>
where
    F: Fn(EchoAst, &TransformationContext) -> TransformResult<EchoAst> + Send + Sync,
{
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn description(&self) -> &'static str {
        self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn matches(&self, ast: &EchoAst, _context: &TransformationContext) -> bool {
        (self.matcher)(ast)
    }
    
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        (self.transformer)(ast, context)
    }
}

/// Rule execution statistics
#[derive(Debug, Default)]
pub struct RuleStats {
    pub rule_name: String,
    pub applications: u64,
    pub transformations: u64,
    pub errors: u64,
    pub total_time_ms: u64,
}

impl RuleStats {
    pub fn new(rule_name: String) -> Self {
        Self {
            rule_name,
            applications: 0,
            transformations: 0,
            errors: 0,
            total_time_ms: 0,
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.applications == 0 {
            0.0
        } else {
            (self.transformations as f64) / (self.applications as f64)
        }
    }
    
    pub fn average_time_ms(&self) -> f64 {
        if self.applications == 0 {
            0.0
        } else {
            (self.total_time_ms as f64) / (self.applications as f64)
        }
    }
}