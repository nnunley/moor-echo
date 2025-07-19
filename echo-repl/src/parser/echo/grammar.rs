// Echo language grammar using rust-sitter annotations
// Incrementally building up from a working base
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[rust_sitter::extra(pattern = r"\s+")]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _whitespace: (),
    }

    #[derive(Debug, PartialEq)]
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
        
        // Assignment - temporarily commented out due to grammar conflict
        // Will need to be handled at statement level, not expression level
        // #[rust_sitter::prec_right(2)]
        // Assignment {
        //     target: Box<EchoAst>, // Must be an Identifier
        //     #[rust_sitter::leaf(text = "=", add_conflict = true)]
        //     _op: (),
        //     value: Box<EchoAst>,
        // },
        
        // Comparison operators
        #[rust_sitter::prec_left(4)]
        Equal {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "==")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        // Property access (high precedence 11)
        #[rust_sitter::prec_left(11)]
        PropertyAccess {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Box<EchoAst>, // Must be an Identifier
        },
        
        
        // Method calls
        #[rust_sitter::prec_left(11)]
        MethodCall {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            method: Box<EchoAst>,
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
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
        
        // Parenthesized expression
        Paren {
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            expr: Box<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        
        // Comma separator (needed for argument lists)
        #[rust_sitter::leaf(text = ",")]
        Comma,
    }
    
    #[derive(Debug, PartialEq)]
    pub enum ObjectMember {
        PropertyDef {
            #[rust_sitter::leaf(text = "property")]
            _property: (),
            name: Box<EchoAst>,
            #[rust_sitter::leaf(text = "=", add_conflict = true)]
            _equals: (),
            value: Box<EchoAst>,
        },
    }
}

pub use echo::{EchoAst, ObjectMember};
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