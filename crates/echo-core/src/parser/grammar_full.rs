// Echo language grammar using rust-sitter annotations
// Following tree-sitter-moo structure for compatibility
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[rust_sitter::extra(pattern = r"\s+")]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _whitespace: (),
    }
    
    #[rust_sitter::extra]
    struct Comment {
        #[rust_sitter::leaf(pattern = r"//[^\n]*")]
        _comment: (),
    }

    #[derive(Debug, PartialEq)]
    #[rust_sitter::language]
    pub struct Program {
        pub statements: Vec<Statement>,
    }
    
    #[derive(Debug, PartialEq)]
    pub enum Statement {
        Expression(ExpressionStatement),
        Let(LetStatement),
        If(IfStatement),
        While(WhileStatement),
        For(ForStatement),
        Return(ReturnStatement),
        Break(BreakStatement),
        Continue(ContinueStatement),
        Object(ObjectDefinition),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ExpressionStatement {
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ";", optional = true)]
        _semicolon: Option<()>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct LetStatement {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
        pub target: Box<Expression>, // Should be identifier or pattern
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ";", optional = true)]
        _semicolon: Option<()>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub then_statements: Vec<Statement>,
        pub else_branch: Option<ElseBranch>,
        #[rust_sitter::leaf(text = "endif")]
        _endif: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ElseBranch {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub statements: Vec<Statement>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "endwhile")]
        _endwhile: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        _for: (),
        pub variable: Box<Expression>, // Identifier
        #[rust_sitter::leaf(text = "in")]
        _in: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub collection: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "endfor")]
        _endfor: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub value: Box<Expression>,
        #[rust_sitter::leaf(text = ";", optional = true)]
        _semicolon: Option<()>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        _break: (),
        #[rust_sitter::leaf(text = ";", optional = true)]
        _semicolon: Option<()>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        _continue: (),
        #[rust_sitter::leaf(text = ";", optional = true)]
        _semicolon: Option<()>,
    }
    
    #[derive(Debug, PartialEq)]
    pub struct ObjectDefinition {
        #[rust_sitter::leaf(text = "object")]
        _object: (),
        pub name: Box<Expression>, // Identifier
        #[rust_sitter::repeat(non_empty = false)]
        pub members: Vec<ObjectMember>,
        #[rust_sitter::leaf(text = "endobject")]
        _endobject: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub enum ObjectMember {
        Property(PropertyDefinition),
        Verb(VerbDefinition),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct PropertyDefinition {
        #[rust_sitter::leaf(text = "property")]
        _property: (),
        pub name: Box<Expression>, // Identifier
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub value: Box<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub struct VerbDefinition {
        #[rust_sitter::leaf(text = "verb")]
        _verb: (),
        pub name: Box<Expression>, // String or Identifier
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub parameters: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "endverb")]
        _endverb: (),
    }
    
    #[derive(Debug, PartialEq)]
    pub enum Expression {
        // Literals
        Number(#[rust_sitter::leaf(pattern = r"-?\d+", transform = |v| v.parse().unwrap())] i64),
        String(#[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |v| v[1..v.len()-1].to_string())] String),
        #[rust_sitter::leaf(text = "true")]
        True,
        #[rust_sitter::leaf(text = "false")]
        False,
        
        // Identifiers
        Identifier(#[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())] String),
        
        // System property: $propname
        SystemProperty(#[rust_sitter::leaf(pattern = r"\$[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v[1..].to_string())] String),
        
        // Object reference: #123
        ObjectReference(#[rust_sitter::leaf(pattern = r"#-?\d+", transform = |v| v[1..].parse().unwrap())] i64),
        
        // Assignment (precedence 1)
        #[rust_sitter::prec_right(1)]
        Assignment {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "=")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Ternary conditional (precedence 2)
        #[rust_sitter::prec_right(2)]
        Conditional {
            condition: Box<Expression>,
            #[rust_sitter::leaf(text = "?")]
            _question: (),
            then_expr: Box<Expression>,
            #[rust_sitter::leaf(text = "|")]
            _pipe: (),
            else_expr: Box<Expression>,
        },
        
        // Logical operators (precedence 3)
        #[rust_sitter::prec_left(3)]
        Or {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "||")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(3)]
        And {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "&&")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Comparison operators (precedence 4)
        #[rust_sitter::prec_left(4)]
        Equal {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "==")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        NotEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "!=")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        LessThan {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "<")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        LessEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "<=")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        GreaterThan {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = ">")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        GreaterEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = ">=")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(4)]
        In {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "in")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Arithmetic operators (precedence 7)
        #[rust_sitter::prec_left(7)]
        Add {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "+")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(7)]
        Subtract {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "-")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Multiplicative operators (precedence 8)
        #[rust_sitter::prec_left(8)]
        Multiply {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "*")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(8)]
        Divide {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "/")]
            _op: (),
            right: Box<Expression>,
        },
        
        #[rust_sitter::prec_left(8)]
        Modulo {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "%")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Power operator (precedence 9)
        #[rust_sitter::prec_right(9)]
        Power {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "^")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Unary operators (precedence 10)
        #[rust_sitter::prec(10)]
        Not {
            #[rust_sitter::leaf(text = "!")]
            _op: (),
            operand: Box<Expression>,
        },
        
        #[rust_sitter::prec(10)]
        UnaryMinus {
            #[rust_sitter::leaf(text = "-")]
            _op: (),
            operand: Box<Expression>,
        },
        
        // Property/method access (precedence 11)
        #[rust_sitter::prec_left(11)]
        PropertyAccess {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Box<Expression>, // Identifier or expression in parens
        },
        
        #[rust_sitter::prec_left(11)]
        MethodCall {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            method: Box<Expression>, // Identifier or expression in parens
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            arguments: Vec<Expression>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        
        #[rust_sitter::prec_left(11)]
        IndexAccess {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = "[")]
            _lbracket: (),
            index: Box<Expression>,
            #[rust_sitter::leaf(text = "]")]
            _rbracket: (),
        },
        
        // List literal
        List {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            elements: Vec<Expression>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
        
        // Parenthesized expression
        Parenthesized {
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            expression: Box<Expression>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
    }
}

// Export all the types
pub use echo::{
    Program, Statement, Expression, ExpressionStatement, LetStatement, 
    IfStatement, ElseBranch, WhileStatement, ForStatement, ReturnStatement,
    BreakStatement, ContinueStatement, ObjectDefinition, ObjectMember,
    PropertyDefinition, VerbDefinition
};
pub use echo::parse as parse_echo;

// Compatibility wrapper for existing API
pub struct EchoParser;

impl EchoParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
    
    pub fn parse(&mut self, input: &str) -> anyhow::Result<Expression> {
        // Parse as a program and extract the first statement's expression
        let program = parse_echo(input).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        
        if let Some(Statement::Expression(expr_stmt)) = program.statements.into_iter().next() {
            Ok(*expr_stmt.expression)
        } else if program.statements.is_empty() {
            Err(anyhow::anyhow!("Empty input"))
        } else {
            Err(anyhow::anyhow!("Expected expression, got statement"))
        }
    }
    
    pub fn parse_program(&mut self, input: &str) -> anyhow::Result<Program> {
        parse_echo(input).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}

// For backwards compatibility with the old EchoAst
pub type EchoAst = Expression;