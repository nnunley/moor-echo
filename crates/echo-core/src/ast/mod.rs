// Unified AST for Echo language
// This AST is targeted by both MOO compatibility parser and modern Echo parser

pub mod source_gen;
pub use source_gen::ToSource;

#[cfg(test)]
mod source_gen_tests;

use std::fmt;

use serde::{Deserialize, Serialize};

/// Binding type for variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingType {
    /// let x = ... (mutable binding)
    Let,
    /// const x = ... (immutable binding)
    Const,
    /// x = ... (reassignment, no new binding)
    None,
}

/// Pattern for destructuring/scatter assignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingPattern {
    /// Simple identifier: x
    Identifier(String),
    /// List destructuring: {a, b, c}
    List(Vec<BindingPatternElement>),
    /// Object destructuring: {x, y: renamed}
    Object(Vec<(String, BindingPattern)>),
    /// Rest pattern: ...rest
    Rest(Box<BindingPattern>),
    /// Ignore pattern: _
    Ignore,
}

/// Elements within a BindingPattern (for list/object destructuring)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingPatternElement {
    Simple(String),
    Optional { name: String, default: Box<EchoAst> },
    Rest(String),
}

/// Represents a value that can appear on the left side of an assignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LValue {
    /// Variable binding with optional type: let x, const y, or just z
    Binding {
        binding_type: BindingType,
        pattern: BindingPattern,
    },
    /// Property access: obj.prop
    PropertyAccess {
        object: Box<EchoAst>,
        property: String,
    },
    /// Index access: list[index]
    IndexAccess {
        object: Box<EchoAst>,
        index: Box<EchoAst>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EchoAst {
    // Literals
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,

    // Identifiers and references
    Identifier(String),
    SystemProperty(String), // $propname
    ObjectRef(i64),         // #123

    // Binary operations
    Add {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Subtract {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Multiply {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Divide {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Modulo {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Power {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },

    // Comparison operations
    Equal {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    NotEqual {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    LessThan {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    LessEqual {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    GreaterThan {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    GreaterEqual {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    In {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },

    // Logical operations
    And {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Or {
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    Not {
        operand: Box<EchoAst>,
    },

    // Unary operations
    UnaryMinus {
        operand: Box<EchoAst>,
    },
    UnaryPlus {
        operand: Box<EchoAst>,
    },

    // Variable operations
    Assignment {
        target: LValue,
        value: Box<EchoAst>,
    },
    LocalAssignment {
        target: BindingPattern,
        value: Box<EchoAst>,
    },
    ConstAssignment {
        target: BindingPattern,
        value: Box<EchoAst>,
    },

    // Property and method access
    PropertyAccess {
        object: Box<EchoAst>,
        property: String,
    },
    MethodCall {
        object: Box<EchoAst>,
        method: String,
        args: Vec<EchoAst>,
    },
    FunctionCall {
        name: String,
        args: Vec<EchoAst>,
    },
    Call {
        func: Box<EchoAst>,
        args: Vec<EchoAst>,
    }, // For lambda calls

    // Index access
    IndexAccess {
        object: Box<EchoAst>,
        index: Box<EchoAst>,
    },

    // Collections
    List {
        elements: Vec<EchoAst>,
    },
    Map {
        entries: Vec<(String, EchoAst)>,
    }, // Modern Echo only

    // Anonymous functions (Modern Echo only)
    Lambda {
        params: Vec<LambdaParam>,
        body: Box<EchoAst>,
    },

    // Control structures
    If {
        condition: Box<EchoAst>,
        then_branch: Vec<EchoAst>,
        else_branch: Option<Vec<EchoAst>>,
    },
    While {
        label: Option<String>,
        condition: Box<EchoAst>,
        body: Vec<EchoAst>,
    },
    For {
        label: Option<String>,
        variable: String,
        collection: Box<EchoAst>,
        body: Vec<EchoAst>,
    },

    // Jump statements
    Return {
        value: Option<Box<EchoAst>>,
    },
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },

    // Event emission
    Emit {
        event_name: String,
        args: Vec<EchoAst>,
    },

    // Object definitions
    ObjectDef {
        name: String,
        parent: Option<String>,
        members: Vec<ObjectMember>,
    },

    // Error handling
    Try {
        body: Vec<EchoAst>,
        catch: Option<CatchClause>,
        finally: Option<Vec<EchoAst>>,
    },

    // Modern Echo features (not in MOO)
    // These will only be generated by the modern Echo parser
    Event {
        name: String,
        params: Vec<Parameter>,
        body: Vec<EchoAst>,
    },
    Spawn {
        body: Box<EchoAst>,
    },
    Await {
        expr: Box<EchoAst>,
    },
    Match {
        expr: Box<EchoAst>,
        arms: Vec<MatchArm>,
    },

    // Type annotations (Modern Echo only)
    TypedIdentifier {
        name: String,
        type_annotation: TypeExpression,
    },

    // Statements
    ExpressionStatement(Box<EchoAst>),
    Block(Vec<EchoAst>),

    // Top-level program
    Program(Vec<EchoAst>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectMember {
    Property {
        name: String,
        value: EchoAst,
        permissions: Option<PropertyPermissions>,
        required_capabilities: Vec<String>,
    },
    Verb {
        name: String,
        args: Vec<Parameter>,
        body: Vec<EchoAst>,
        permissions: Option<VerbPermissions>,
        required_capabilities: Vec<String>,
    },
    // Modern Echo only
    Method {
        name: String,
        args: Vec<Parameter>,
        return_type: Option<TypeExpression>,
        body: Vec<EchoAst>,
    },
    Event {
        name: String,
        params: Vec<Parameter>,
        body: Vec<EchoAst>,
    },
    Query {
        name: String,
        params: Vec<String>, // Query parameter names
        clauses: Vec<QueryClause>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeExpression>,
    pub default_value: Option<EchoAst>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LambdaParam {
    Simple(String),
    Optional { name: String, default: Box<EchoAst> },
    Rest(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub error_var: Option<String>,
    pub body: Vec<EchoAst>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<EchoAst>>,
    pub body: Box<EchoAst>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Number(i64),
    String(String),
    Constructor { name: String, args: Vec<Pattern> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpression {
    Named(String),
    Array(Box<TypeExpression>),
    Optional(Box<TypeExpression>),
    Function {
        params: Vec<TypeExpression>,
        return_type: Box<TypeExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyPermissions {
    pub read: String, // e.g., "owner", "anyone"
    pub write: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbPermissions {
    pub read: String,
    pub write: String,
    pub execute: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryClause {
    pub predicate: String,
    pub args: Vec<QueryArg>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryArg {
    Variable(String),
    Constant(EchoAst),
    Wildcard,
}

// Helper methods
impl EchoAst {
    /// Check if this AST node is an expression (vs statement)
    pub fn is_expression(&self) -> bool {
        !matches!(
            self,
            EchoAst::LocalAssignment { .. }
                | EchoAst::ConstAssignment { .. }
                | EchoAst::ObjectDef { .. }
                | EchoAst::Event { .. }
                | EchoAst::If { .. }
                | EchoAst::While { .. }
                | EchoAst::For { .. }
                | EchoAst::Return { .. }
                | EchoAst::Break { .. }
                | EchoAst::Continue { .. }
                | EchoAst::Block(_)
                | EchoAst::ExpressionStatement(_)
        )
    }

    /// Check if this is a MOO-compatible construct
    pub fn is_moo_compatible(&self) -> bool {
        !matches!(
            self,
            EchoAst::LocalAssignment { .. }
                | EchoAst::ConstAssignment { .. }
                | EchoAst::Map { .. }
                | EchoAst::Event { .. }
                | EchoAst::Spawn { .. }
                | EchoAst::Await { .. }
                | EchoAst::Match { .. }
                | EchoAst::TypedIdentifier { .. }
        )
    }
}

impl fmt::Display for EchoAst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EchoAst::Number(n) => write!(f, "{n}"),
            EchoAst::Float(fl) => write!(f, "{fl}"),
            EchoAst::String(s) => write!(f, "\"{s}\""),
            EchoAst::Boolean(b) => write!(f, "{b}"),
            EchoAst::Null => write!(f, "null"),
            EchoAst::Identifier(s) => write!(f, "{s}"),
            EchoAst::SystemProperty(s) => write!(f, "${s}"),
            EchoAst::ObjectRef(n) => write!(f, "#{n}"),
            _ => write!(f, "<expression>"),
        }
    }
}
