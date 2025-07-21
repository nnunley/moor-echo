// Echo language grammar using rust-sitter annotations
// Incrementally building up from a working base
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[rust_sitter::extra(pattern = r"\s+")]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _whitespace: (),
    }


    

    

    

    

    

    

    

    

    

    

    

    

    

    // BindingPattern for let/const declarations
    #[derive(Debug, PartialEq, Clone)]
    pub enum BindingPattern {
        Identifier(Identifier),
        List {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            elements: Vec<BindingPatternElement>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
        Rest {
            #[rust_sitter::leaf(pattern = r"\.\.\.")]
            _dots: (),
            name: Identifier,
        },
        #[rust_sitter::leaf(text = "_")]
        Ignore,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum BindingPatternElement {
        Simple(Identifier),
        Optional {
            #[rust_sitter::leaf(text = "?")]
            _question: (),
            name: Identifier,
            #[rust_sitter::leaf(text = "=")]
            _equals: (),
            default: Box<EchoAst>,
        },
        Rest {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            name: Identifier,
        },
    }

    #[derive(Debug, PartialEq, Clone)]
    #[rust_sitter::language]
    pub enum EchoAst {
        // Literals
        Float(#[rust_sitter::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse().unwrap())] f64),
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i64),
        String(#[rust_sitter::leaf(pattern = r#""[^"]*""#, transform = |v| v[1..v.len()-1].to_string())] String),
        
        // Boolean literals
        #[rust_sitter::leaf(text = "true")]
        True,
        #[rust_sitter::leaf(text = "false")]
        False,
        
        // Identifiers
        Identifier(#[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())] String),
        
        // System property access: $propname
        SysProp(#[rust_sitter::leaf(pattern = r"\$[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v[1..].to_string())] String),
        
        // Object reference: #123
        ObjectRef(#[rust_sitter::leaf(pattern = r"#\d+", transform = |v| v[1..].parse().unwrap())] i64),
        
        
        // Binary operations - Arithmetic (precedence 7 for add/sub, 8 for mul/div/mod)
        #[rust_sitter::prec_left(7)]
        Add {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "+")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(7)]
        Subtract {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "-")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(8)]
        Multiply {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "*")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(8)]
        Divide {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "/")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(8)]
        Modulo {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "%")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        // Assignment Expression - can appear in expression contexts
        #[rust_sitter::prec_right(2)]
        AssignmentExpr {
            left: Box<EchoAst>, // This will be validated as LValue during conversion
            #[rust_sitter::leaf(text = "=")]
            _op: (),
            right: Box<EchoAst>,
        },

        // Local variable declaration statement
        #[rust_sitter::prec(12)]
        LocalAssignment {
            #[rust_sitter::leaf(text = "let")]
            _let_keyword: (),
            target: Box<BindingPattern>,
            #[rust_sitter::leaf(text = "=")]
            _op: (),
            value: Box<EchoAst>,
        },

        // Constant declaration statement
        #[rust_sitter::prec(12)]
        ConstAssignment {
            #[rust_sitter::leaf(text = "const")]
            _const_keyword: (),
            target: Box<BindingPattern>,
            #[rust_sitter::leaf(text = "=")]
            _op: (),
            value: Box<EchoAst>,
        },

        
        
        // Comparison operators (precedence 4)
        #[rust_sitter::prec_left(4)]
        Equal {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "==")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(4)]
        NotEqual {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "!=")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(5)]
        LessThan {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "<")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(5)]
        LessEqual {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "<=")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(5)]
        GreaterThan {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = ">")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(5)]
        GreaterEqual {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = ">=")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        // Logical operators (precedence 2 for &&, 1 for ||)
        #[rust_sitter::prec_left(2)]
        And {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "&&")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        #[rust_sitter::prec_left(1)]
        Or {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "||")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        // Unary not (high precedence)
        #[rust_sitter::prec_right(10)]
        Not {
            #[rust_sitter::leaf(text = "!")]
            _op: (),
            operand: Box<EchoAst>,
        },
        
        // Property access (high precedence 11)
        #[rust_sitter::prec_left(11)]
        PropertyAccess {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Identifier,
        },
        
        // Index access (high precedence 11)
        #[rust_sitter::prec_left(11)]
        IndexAccess {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = "[")]
            _lbracket: (),
            index: Box<EchoAst>,
            #[rust_sitter::leaf(text = "]")]
            _rbracket: (),
        },
        
        // Method calls
        #[rust_sitter::prec_left(11)]
        MethodCall {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            method: Identifier,
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            args: Vec<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        
        // Function call (for lambdas and function values)
        #[rust_sitter::prec_left(10)]
        Call {
            func: Box<EchoAst>,
            #[rust_sitter::leaf(text = "(", add_conflict = true)]
            _lparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            args: Vec<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        
        // Object definitions
        ObjectDef {
            #[rust_sitter::leaf(text = "object")]
            _object: (),
            name: Box<EchoAst>,
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ";")]
                ()
            )]
            members: Vec<ObjectMember>,
            #[rust_sitter::leaf(text = "endobject")]
            _endobject: (),
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
            elements: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
        
        // Arrow function: params => expr
        #[rust_sitter::prec_right(3)]
        ArrowFunction {
            params: Box<EchoAst>, // Can be identifier, list pattern, etc.
            #[rust_sitter::leaf(text = "=>")]
            _arrow: (),
            body: Box<EchoAst>,
        },
        
        // Block function: fn params ... endfn
        BlockFunction {
            #[rust_sitter::leaf(text = "fn")]
            _fn: (),
            params: ParamPattern,
            #[rust_sitter::repeat(non_empty = false)]
            body: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endfn")]
            _endfn: (),
        },
        
        // Parenthesized expression
        Paren {
            #[rust_sitter::leaf(text = "(", add_conflict = true)]
            _lparen: (),
            expr: Box<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        
        // Comma separator (needed for argument lists)
        #[rust_sitter::leaf(text = ",")]
        Comma,
        
        // Control flow - If with MOO syntax: if (condition) ... [else ...] endif
        If {
            #[rust_sitter::leaf(text = "if")]
            _if: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            condition: Box<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            then_body: Vec<EchoAst>,
            else_clause: Option<ElseClause>,
            #[rust_sitter::leaf(text = "endif")]
            _endif: (),
        },
        
        // While loop with MOO syntax: while (condition) ... endwhile
        While {
            #[rust_sitter::leaf(text = "while")]
            _while: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            condition: Box<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            body: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endwhile")]
            _endwhile: (),
        },
        
        // For loop with MOO syntax: for var in (collection) ... endfor
        For {
            #[rust_sitter::leaf(text = "for")]
            _for: (),
            variable: Box<EchoAst>, // Must be an Identifier
            #[rust_sitter::leaf(text = "in")]
            _in: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            collection: Box<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            body: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endfor")]
            _endfor: (),
        },
        
        // Break statement with optional label
        #[rust_sitter::prec_right(9)]
        Break {
            #[rust_sitter::leaf(text = "break")]
            _break: (),
            label: Option<Box<EchoAst>>, // Optional label
        },
        
        // Continue statement with optional label  
        #[rust_sitter::prec_right(9)]
        Continue {
            #[rust_sitter::leaf(text = "continue")]
            _continue: (),
            label: Option<Box<EchoAst>>, // Optional label
        },
        
        // Return statement with optional value
        #[rust_sitter::prec_right(9)]
        Return {
            #[rust_sitter::leaf(text = "return")]
            _return: (),
            value: Option<Box<EchoAst>>, // Optional return value
        },
        
        // // Block statement - commented out due to conflict with List
        // // In MOO, blocks don't exist as a syntax construct
        // Block {
        //     #[rust_sitter::leaf(text = "{")]
        //     _lbrace: (),
        //     #[rust_sitter::repeat(non_empty = false)]
        //     #[rust_sitter::delimited(
        //         #[rust_sitter::leaf(text = ";")]
        //         ()
        //     )]
        //     statements: Vec<EchoAst>,
        //     #[rust_sitter::leaf(text = "}")]
        //     _rbrace: (),
        // },
    }
    
    #[derive(Debug, PartialEq, Clone)]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        pub name: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum ObjectMember {
        PropertyDef {
            #[rust_sitter::leaf(text = "property")]
            _property: (),
            name: Identifier,
            #[rust_sitter::leaf(text = "=", add_conflict = true)]
            _equals: (),
            value: Box<EchoAst>,
        },
        VerbDef {
            #[rust_sitter::leaf(text = "verb")]
            _verb: (),
            name: Identifier,
            params: ParamPattern,
            #[rust_sitter::repeat(non_empty = false)]
            body: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endverb")]
            _endverb: (),
        },
    }
    
    #[derive(Debug, PartialEq, Clone)]
    pub enum ParamElement {
        // Simple identifier: x
        Simple(Identifier),
        // Optional with default: ?name=value
        Optional {
            #[rust_sitter::leaf(text = "?")]
            _question: (),
            name: Identifier,
            #[rust_sitter::leaf(text = "=")]
            _equals: (),
            default: Box<EchoAst>,
        },
        // Rest parameter: @rest
        Rest {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            name: Identifier,
        },
    }
    
    #[derive(Debug, PartialEq, Clone)]
    pub enum ParamPattern {
        // Single parameter (could be simple, optional, or rest)
        Single(ParamElement),
        // Multiple parameters in braces (including empty {})
        Multiple {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            params: Vec<ParamElement>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
    }
    
    #[derive(Debug, PartialEq, Clone)]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        pub _else: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<EchoAst>,
    }
}

pub use echo::{EchoAst, ObjectMember, ElseClause, Identifier, ParamPattern, ParamElement, BindingPattern, BindingPatternElement};
pub use echo::parse as parse_echo;

// Compatibility wrapper for existing API
pub struct EchoParser;

impl EchoParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
    
    pub fn parse(&mut self, input: &str) -> anyhow::Result<EchoAst> {
        parse_echo(input).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}