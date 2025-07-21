// Improved Echo parser using separated statement/expression grammar
use anyhow::Result;
use crate::ast;
use super::grammar_improved;
use super::grammar_converter_improved;

pub struct ImprovedEchoParser {
    inner: grammar_improved::ProgramParser,
}

impl ImprovedEchoParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: grammar_improved::ProgramParser::new()?,
        })
    }
    
    pub fn parse(&mut self, input: &str) -> Result<ast::EchoAst> {
        // Parse using the improved grammar
        let program = self.inner.parse(input)?;
        
        // Convert to unified AST
        let statements = grammar_converter_improved::convert_program(program)?;
        
        // Return the result
        if statements.is_empty() {
            Ok(ast::EchoAst::Null)
        } else if statements.len() == 1 {
            Ok(statements.into_iter().next().unwrap())
        } else {
            Ok(ast::EchoAst::Program(statements))
        }
    }
}

impl super::super::Parser for ImprovedEchoParser {
    fn parse(&mut self, source: &str) -> Result<ast::EchoAst> {
        self.parse(source)
    }
    
    fn parse_program(&mut self, source: &str) -> Result<ast::EchoAst> {
        // Parse the entire source as a program
        let program = self.inner.parse(source)?;
        let statements = grammar_converter_improved::convert_program(program)?;
        
        if statements.is_empty() {
            Ok(ast::EchoAst::Null)
        } else {
            Ok(ast::EchoAst::Program(statements))
        }
    }
    
    fn name(&self) -> &'static str {
        "echo_improved"
    }
}