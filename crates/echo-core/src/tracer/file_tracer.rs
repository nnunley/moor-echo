/*!
# FileTracer - File-based Transformation System

File-based transformation system for bootstrapping incompatible changes.
Similar to Squeak's write-out SystemTracer functionality.
*/

use std::path::Path;
use std::fs;
use std::collections::HashMap;
use anyhow::{anyhow, Result};

use crate::ast::EchoAst;
use crate::parser::create_parser;

use super::rules::{TransformationRule, RuleStats};
use super::{TransformResult, TransformationContext};

/// File-based transformation system
/// 
/// Reads source files, applies transformations, and writes transformed code.
/// This enables bootstrapping of incompatible changes by generating new codebases.
pub struct FileTracer {
    rules: Vec<Box<dyn TransformationRule>>,
    stats: HashMap<String, RuleStats>,
    source_extensions: Vec<String>,
    output_extension: String,
    preserve_structure: bool,
    backup_originals: bool,
}

impl FileTracer {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            stats: HashMap::new(),
            source_extensions: vec!["moo".to_string(), "echo".to_string()],
            output_extension: "echo".to_string(),
            preserve_structure: true,
            backup_originals: true,
        }
    }
    
    /// Set the file extensions to process
    pub fn source_extensions(mut self, extensions: Vec<String>) -> Self {
        self.source_extensions = extensions;
        self
    }
    
    /// Set the output file extension
    pub fn output_extension(mut self, extension: String) -> Self {
        self.output_extension = extension;
        self
    }
    
    /// Whether to preserve directory structure in output
    pub fn preserve_structure(mut self, preserve: bool) -> Self {
        self.preserve_structure = preserve;
        self
    }
    
    /// Whether to backup original files
    pub fn backup_originals(mut self, backup: bool) -> Self {
        self.backup_originals = backup;
        self
    }
    
    /// Add a transformation rule
    pub fn add_rule(&mut self, rule: Box<dyn TransformationRule>) {
        let rule_name = rule.name().to_string();
        self.stats.insert(rule_name, RuleStats::new(rule.name().to_string()));
        self.rules.push(rule);
    }
    
    /// Transform all files in a directory
    pub fn transform_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self, 
        source_dir: P, 
        output_dir: Q
    ) -> Result<FileTransformationSummary> {
        let source_path = source_dir.as_ref();
        let output_path = output_dir.as_ref();
        
        if !source_path.exists() {
            return Err(anyhow!("Source directory does not exist: {}", source_path.display()));
        }
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_path)?;
        
        let mut summary = FileTransformationSummary::new();
        self.transform_directory_recursive(source_path, output_path, source_path, &mut summary)?;
        
        Ok(summary)
    }
    
    /// Transform a single file
    pub fn transform_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        source_file: P,
        output_file: Q,
    ) -> Result<FileTransformationSummary> {
        let source_path = source_file.as_ref();
        let output_path = output_file.as_ref();
        
        let mut summary = FileTransformationSummary::new();
        
        // Backup original if requested
        if self.backup_originals && output_path.exists() {
            let backup_path = output_path.with_extension(
                format!("{}.backup", output_path.extension().unwrap_or_default().to_string_lossy())
            );
            fs::copy(output_path, backup_path)?;
        }
        
        // Read and parse source file
        let source_content = fs::read_to_string(source_path)?;
        let mut parser = create_parser("moo")?;
        let ast = parser.parse(&source_content)?;
        
        // Transform the AST
        let context = TransformationContext::new()
            .with_source_file(source_path.to_string_lossy().to_string());
        
        let transformed_ast = self.transform_ast(ast, &context)?;
        
        // Generate output code
        let output_content = self.generate_code(&transformed_ast)?;
        
        // Write output file
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, output_content)?;
        
        summary.files_processed += 1;
        summary.files_transformed += 1;
        
        Ok(summary)
    }
    
    /// Recursively transform directories
    fn transform_directory_recursive(
        &mut self,
        current_dir: &Path,
        output_dir: &Path,
        source_root: &Path,
        summary: &mut FileTransformationSummary,
    ) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively process subdirectories
                let relative_path = path.strip_prefix(source_root)?;
                let output_subdir = output_dir.join(relative_path);
                
                if self.preserve_structure {
                    fs::create_dir_all(&output_subdir)?;
                    self.transform_directory_recursive(&path, &output_subdir, source_root, summary)?;
                } else {
                    self.transform_directory_recursive(&path, output_dir, source_root, summary)?;
                }
            } else if self.should_process_file(&path) {
                // Transform individual files
                let output_file = if self.preserve_structure {
                    let relative_path = path.strip_prefix(source_root)?;
                    output_dir.join(relative_path).with_extension(&self.output_extension)
                } else {
                    output_dir.join(path.file_name().unwrap()).with_extension(&self.output_extension)
                };
                
                match self.transform_file(&path, &output_file) {
                    Ok(file_summary) => {
                        summary.merge(file_summary);
                    }
                    Err(e) => {
                        summary.errors.push(format!("Error processing {}: {}", path.display(), e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a file should be processed based on its extension
    fn should_process_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            self.source_extensions.iter().any(|ext| ext.to_lowercase() == ext_str)
        } else {
            false
        }
    }
    
    /// Transform an AST using the registered rules
    fn transform_ast(&mut self, ast: EchoAst, context: &TransformationContext) -> TransformResult<EchoAst> {
        let mut current_ast = ast;
        
        // Apply rules in priority order
        for rule in &self.rules {
            let rule_name = rule.name().to_string();
            let stats = self.stats.get_mut(&rule_name).unwrap();
            
            if rule.matches(&current_ast, context) {
                stats.applications += 1;
                
                match rule.transform(current_ast, context) {
                    Ok(transformed) => {
                        current_ast = transformed;
                        stats.transformations += 1;
                    }
                    Err(e) => {
                        stats.errors += 1;
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(current_ast)
    }
    
    /// Generate code from transformed AST
    fn generate_code(&self, ast: &EchoAst) -> Result<String> {
        // For now, use debug formatting
        // In practice, you'd want a proper code generator
        Ok(format!("{ast:#?}"))
    }
    
    /// Get transformation statistics
    pub fn stats(&self) -> &HashMap<String, RuleStats> {
        &self.stats
    }
}

impl Default for FileTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of file transformation results
#[derive(Debug, Default)]
pub struct FileTransformationSummary {
    pub files_processed: u64,
    pub files_transformed: u64,
    pub directories_created: u64,
    pub errors: Vec<String>,
}

impl FileTransformationSummary {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn merge(&mut self, other: FileTransformationSummary) {
        self.files_processed += other.files_processed;
        self.files_transformed += other.files_transformed;
        self.directories_created += other.directories_created;
        self.errors.extend(other.errors);
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.files_processed == 0 {
            0.0
        } else {
            (self.files_transformed as f64) / (self.files_processed as f64)
        }
    }
    
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}