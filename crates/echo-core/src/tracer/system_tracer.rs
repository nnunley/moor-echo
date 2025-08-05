/*!
# SystemTracer - In-Memory Transformation Engine

Core transformation engine for in-memory transformations of the Echo system.
Similar to Squeak's in-image SystemTracer.
*/

use std::collections::HashMap;
use anyhow::{anyhow, Result};

use crate::ast::EchoAst;
use crate::evaluator::Evaluator;
use crate::storage::ObjectId;

use super::rules::{TransformationRule, RuleStats};
use super::{TransformResult, TransformationContext};

/// In-memory system transformation engine
/// 
/// Transforms objects and code within the running Echo system.
/// This is similar to Squeak's in-image SystemTracer.
pub struct SystemTracer {
    rules: Vec<Box<dyn TransformationRule>>,
    stats: HashMap<String, RuleStats>,
    max_iterations: usize,
    dry_run: bool,
}

impl SystemTracer {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            stats: HashMap::new(),
            max_iterations: 10,
            dry_run: false,
        }
    }
    
    /// Enable dry-run mode (transformations are computed but not applied)
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }
    
    /// Set maximum number of transformation iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
    
    /// Add a transformation rule
    pub fn add_rule(&mut self, rule: Box<dyn TransformationRule>) {
        let rule_name = rule.name().to_string();
        self.stats.insert(rule_name, RuleStats::new(rule.name().to_string()));
        self.rules.push(rule);
    }
    
    /// Sort rules by priority (higher priority first)
    pub fn sort_rules_by_priority(&mut self) {
        self.rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority()));
    }
    
    /// Transform the entire Echo system
    pub fn transform_system(&mut self, evaluator: &mut Evaluator) -> Result<TransformationSummary> {
        let mut summary = TransformationSummary::new();
        
        // Get all objects in the system
        let object_ids = evaluator.storage().objects.list_all()?;
        
        for object_id in object_ids {
            let result = self.transform_object(evaluator, object_id)?;
            summary.merge(result);
        }
        
        Ok(summary)
    }
    
    /// Transform a single object
    pub fn transform_object(&mut self, evaluator: &mut Evaluator, object_id: ObjectId) -> Result<TransformationSummary> {
        let mut summary = TransformationSummary::new();
        let mut object = evaluator.storage().objects.get(object_id)?;
        let mut changed = false;
        
        let context = TransformationContext::new()
            .with_object_name(object.name.clone());
        
        // Transform verb bodies
        let verb_names: Vec<String> = object.verbs.keys().cloned().collect();
        for verb_name in verb_names {
            let verb_context = context.clone()
                .with_object_name(format!("{}:{}", object.name, verb_name));
            
            if let Some(verb_def) = object.verbs.get_mut(&verb_name) {
                let mut transformed_ast = Vec::new();
                let mut verb_changed = false;
                
                for ast in &verb_def.ast {
                    let (new_ast, changed_this) = self.transform_single_ast(
                        ast.clone(), 
                        &verb_context, 
                        &mut summary,
                        &object.name,
                        &verb_name
                    );
                    transformed_ast.push(new_ast);
                    if changed_this {
                        verb_changed = true;
                    }
                }
                
                if verb_changed {
                    verb_def.ast = transformed_ast;
                    changed = true;
                }
            }
        }
        
        // Store the transformed object if changes were made
        if changed && !self.dry_run {
            evaluator.storage().objects.store(object)?;
            summary.objects_changed += 1;
        } else if changed {
            summary.objects_changed += 1;
        }
        
        Ok(summary)
    }
    
    /// Transform a single AST node
    pub fn transform_ast(&mut self, mut ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        if context.at_max_depth() {
            return Ok(ast);
        }
        
        let mut iterations = 0;
        
        while iterations < self.max_iterations {
            let mut changed = false;
            
            // Apply each rule in priority order
            for rule in &self.rules {
                let start_time = std::time::Instant::now();
                let rule_name = rule.name().to_string();
                
                if rule.matches(&ast, context) {
                    let stats = self.stats.get_mut(&rule_name).unwrap();
                    stats.applications += 1;
                    
                    match rule.transform(ast.clone(), context) {
                        Ok(transformed) => {
                            if !ast_equal(&ast, &transformed) {
                                ast = transformed;
                                stats.transformations += 1;
                                changed = true;
                            }
                        }
                        Err(e) => {
                            stats.errors += 1;
                            return Err(e);
                        }
                    }
                    
                    stats.total_time_ms += start_time.elapsed().as_millis() as u64;
                }
            }
            
            if !changed {
                break;
            }
            
            iterations += 1;
        }
        
        if iterations >= self.max_iterations {
            return Err(anyhow!("Transformation exceeded maximum iterations ({})", self.max_iterations));
        }
        
        // Apply transformations recursively to child nodes
        ast = self.transform_ast_recursive(ast, context)?;
        
        Ok(ast)
    }
    
    /// Recursively transform child AST nodes
    fn transform_ast_recursive(&mut self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        let child_context = context.descend();
        
        match ast {
            EchoAst::Program(stmts) => {
                let transformed_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform_ast(stmt, &child_context))
                    .collect();
                Ok(EchoAst::Program(transformed_stmts?))
            }
            
            EchoAst::Block(stmts) => {
                let transformed_stmts: Result<Vec<_>> = stmts
                    .into_iter()
                    .map(|stmt| self.transform_ast(stmt, &child_context))
                    .collect();
                Ok(EchoAst::Block(transformed_stmts?))
            }
            
            EchoAst::If { condition, then_branch, else_branch } => {
                Ok(EchoAst::If {
                    condition: Box::new(self.transform_ast(*condition, &child_context)?),
                    then_branch: then_branch
                        .into_iter()
                        .map(|stmt| self.transform_ast(stmt, &child_context))
                        .collect::<Result<Vec<_>>>()?,
                    else_branch: else_branch
                        .map(|stmts| 
                            stmts
                                .into_iter()
                                .map(|stmt| self.transform_ast(stmt, &child_context))
                                .collect::<Result<Vec<_>>>()
                        )
                        .transpose()?,
                })
            }
            
            // Add more recursive transformations as needed
            _ => Ok(ast),
        }
    }
    
    /// Transform a single AST node and track changes
    fn transform_single_ast(
        &mut self, 
        ast: EchoAst, 
        context: &TransformationContext,
        summary: &mut TransformationSummary,
        object_name: &str,
        verb_name: &str
    ) -> (EchoAst, bool) {
        match self.transform_ast(ast.clone(), context) {
            Ok(transformed) => {
                let changed = !ast_equal(&ast, &transformed);
                if changed {
                    summary.transformations += 1;
                }
                if self.dry_run {
                    (transformed, false)
                } else {
                    (transformed, changed)
                }
            }
            Err(e) => {
                summary.errors.push(format!("Error transforming verb {}:{}: {}", object_name, verb_name, e));
                (ast, false)
            }
        }
    }
    
    /// Get transformation statistics
    pub fn stats(&self) -> &HashMap<String, RuleStats> {
        &self.stats
    }
    
    /// Clear statistics
    pub fn clear_stats(&mut self) {
        for stats in self.stats.values_mut() {
            *stats = RuleStats::new(stats.rule_name.clone());
        }
    }
}

impl Default for SystemTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of transformation results
#[derive(Debug, Default)]
pub struct TransformationSummary {
    pub objects_changed: u64,
    pub transformations: u64,
    pub errors: Vec<String>,
}

impl TransformationSummary {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn merge(&mut self, other: TransformationSummary) {
        self.objects_changed += other.objects_changed;
        self.transformations += other.transformations;
        self.errors.extend(other.errors);
    }
    
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Compare two AST nodes for equality
fn ast_equal(a: &EchoAst, b: &EchoAst) -> bool {
    // Use the derived PartialEq implementation
    a == b
}