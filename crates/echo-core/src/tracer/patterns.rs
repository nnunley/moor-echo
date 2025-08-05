/*!
# AST Pattern Matching Utilities

Utilities for matching patterns in AST nodes, similar to pattern matching
in functional languages but specialized for Echo AST structures.
*/

use crate::ast::EchoAst;

/// Pattern matcher for AST nodes
pub trait AstPattern {
    /// Check if this pattern matches the given AST node
    fn matches(&self, ast: &EchoAst) -> bool;
}

/// Pattern matcher utility
pub struct PatternMatcher;

impl PatternMatcher {
    /// Match any node of a specific type
    pub fn node_type<F>(predicate: F) -> impl AstPattern
    where
        F: Fn(&EchoAst) -> bool,
    {
        NodeTypeMatcher { predicate }
    }
    
    /// Match assignment nodes
    pub fn assignment() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::Assignment { .. }),
        }
    }
    
    /// Match object definitions
    pub fn object_def() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::ObjectDef { .. }),
        }
    }
    
    /// Match property access nodes
    pub fn property_access() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::PropertyAccess { .. }),
        }
    }
    
    /// Match call nodes
    pub fn call() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::Call { .. }),
        }
    }
    
    /// Match method call nodes
    pub fn method_call() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::MethodCall { .. }),
        }
    }
    
    /// Match object reference nodes
    pub fn object_ref() -> impl AstPattern {
        NodeTypeMatcher {
            predicate: |ast| matches!(ast, EchoAst::ObjectRef(_)),
        }
    }
    
    /// Match specific object reference numbers
    pub fn object_ref_number(number: i64) -> impl AstPattern {
        ObjectRefNumberMatcher { number }
    }
    
    /// Match identifiers with specific names
    pub fn identifier(name: &str) -> IdentifierMatcher {
        IdentifierMatcher {
            name: name.to_string(),
        }
    }
    
    /// Match string literals with specific content
    pub fn string_literal(content: &str) -> StringLiteralMatcher {
        StringLiteralMatcher {
            content: content.to_string(),
        }
    }
    
    /// Combine patterns with AND logic
    pub fn all<P1: AstPattern, P2: AstPattern>(p1: P1, p2: P2) -> AndPattern<P1, P2> {
        AndPattern { p1, p2 }
    }
    
    /// Combine patterns with OR logic
    pub fn any<P1: AstPattern, P2: AstPattern>(p1: P1, p2: P2) -> OrPattern<P1, P2> {
        OrPattern { p1, p2 }
    }
    
    /// Negate a pattern
    pub fn not<P: AstPattern>(pattern: P) -> NotPattern<P> {
        NotPattern { pattern }
    }
}

/// Generic node type matcher
struct NodeTypeMatcher<F>
where
    F: Fn(&EchoAst) -> bool,
{
    predicate: F,
}

impl<F> AstPattern for NodeTypeMatcher<F>
where
    F: Fn(&EchoAst) -> bool,
{
    fn matches(&self, ast: &EchoAst) -> bool {
        (self.predicate)(ast)
    }
}

/// Object reference number matcher
struct ObjectRefNumberMatcher {
    number: i64,
}

impl AstPattern for ObjectRefNumberMatcher {
    fn matches(&self, ast: &EchoAst) -> bool {
        matches!(ast, EchoAst::ObjectRef(n) if *n == self.number)
    }
}

/// Identifier name matcher
pub struct IdentifierMatcher {
    name: String,
}

impl AstPattern for IdentifierMatcher {
    fn matches(&self, ast: &EchoAst) -> bool {
        matches!(ast, EchoAst::Identifier(name) if name == &self.name)
    }
}

/// String literal content matcher
pub struct StringLiteralMatcher {
    content: String,
}

impl AstPattern for StringLiteralMatcher {
    fn matches(&self, ast: &EchoAst) -> bool {
        matches!(ast, EchoAst::String(content) if content == &self.content)
    }
}

/// AND pattern combinator
pub struct AndPattern<P1: AstPattern, P2: AstPattern> {
    p1: P1,
    p2: P2,
}

impl<P1: AstPattern, P2: AstPattern> AstPattern for AndPattern<P1, P2> {
    fn matches(&self, ast: &EchoAst) -> bool {
        self.p1.matches(ast) && self.p2.matches(ast)
    }
}

/// OR pattern combinator
pub struct OrPattern<P1: AstPattern, P2: AstPattern> {
    p1: P1,
    p2: P2,
}

impl<P1: AstPattern, P2: AstPattern> AstPattern for OrPattern<P1, P2> {
    fn matches(&self, ast: &EchoAst) -> bool {
        self.p1.matches(ast) || self.p2.matches(ast)
    }
}

/// NOT pattern combinator
pub struct NotPattern<P: AstPattern> {
    pattern: P,
}

impl<P: AstPattern> AstPattern for NotPattern<P> {
    fn matches(&self, ast: &EchoAst) -> bool {
        !self.pattern.matches(ast)
    }
}

/// Utility for deep AST traversal and pattern matching
pub struct AstWalker;

impl AstWalker {
    /// Find all nodes matching a pattern in an AST tree
    pub fn find_all<'a, P: AstPattern>(ast: &'a EchoAst, pattern: &P) -> Vec<&'a EchoAst> {
        let mut matches = Vec::new();
        Self::find_all_recursive(ast, pattern, &mut matches);
        matches
    }
    
    /// Find the first node matching a pattern in an AST tree
    pub fn find_first<'a, P: AstPattern>(ast: &'a EchoAst, pattern: &P) -> Option<&'a EchoAst> {
        if pattern.matches(ast) {
            return Some(ast);
        }
        
        Self::find_first_recursive(ast, pattern)
    }
    
    fn find_all_recursive<'a, P: AstPattern>(ast: &'a EchoAst, pattern: &P, matches: &mut Vec<&'a EchoAst>) {
        if pattern.matches(ast) {
            matches.push(ast);
        }
        
        // Recursively search child nodes
        match ast {
            EchoAst::Program(stmts) => {
                for stmt in stmts {
                    Self::find_all_recursive(stmt, pattern, matches);
                }
            }
            EchoAst::Block(stmts) => {
                for stmt in stmts {
                    Self::find_all_recursive(stmt, pattern, matches);
                }
            }
            EchoAst::If { condition, then_branch, else_branch } => {
                Self::find_all_recursive(condition, pattern, matches);
                for stmt in then_branch {
                    Self::find_all_recursive(stmt, pattern, matches);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        Self::find_all_recursive(stmt, pattern, matches);
                    }
                }
            }
            EchoAst::PropertyAccess { object, .. } => {
                Self::find_all_recursive(object, pattern, matches);
            }
            EchoAst::Call { func, args } => {
                Self::find_all_recursive(func, pattern, matches);
                for arg in args {
                    Self::find_all_recursive(arg, pattern, matches);
                }
            }
            EchoAst::MethodCall { object, args, .. } => {
                Self::find_all_recursive(object, pattern, matches);
                for arg in args {
                    Self::find_all_recursive(arg, pattern, matches);
                }
            }
            // Add more recursive cases as needed
            _ => {}
        }
    }
    
    fn find_first_recursive<'a, P: AstPattern>(ast: &'a EchoAst, pattern: &P) -> Option<&'a EchoAst> {
        match ast {
            EchoAst::Program(stmts) => {
                for stmt in stmts {
                    if let Some(found) = Self::find_first(stmt, pattern) {
                        return Some(found);
                    }
                }
            }
            EchoAst::Block(stmts) => {
                for stmt in stmts {
                    if let Some(found) = Self::find_first(stmt, pattern) {
                        return Some(found);
                    }
                }
            }
            // Add more recursive cases as needed
            _ => {}
        }
        
        None
    }
}