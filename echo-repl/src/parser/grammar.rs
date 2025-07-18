// Echo language grammar using rust-sitter annotations
// Start with minimal working grammar
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
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i64),
        String(#[rust_sitter::leaf(pattern = r#""[^"]*""#, transform = |v| v[1..v.len()-1].to_string())] String),
        
        // Identifiers
        Identifier(#[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())] String),
        
        // Binary operations
        #[rust_sitter::prec_left(1)]
        Add {
            left: Box<EchoAst>,
            #[rust_sitter::leaf(text = "+")]
            _op: (),
            right: Box<EchoAst>,
        },
        
        // Property access
        #[rust_sitter::prec_left(10)]
        PropertyAccess {
            object: Box<EchoAst>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Box<EchoAst>, // Must be an Identifier
        },
        
        // Let statements
        Let {
            #[rust_sitter::leaf(text = "let")]
            _let: (),
            name: Box<EchoAst>,
            #[rust_sitter::leaf(text = "=")]
            _equals: (),
            value: Box<EchoAst>,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        
        // Object definitions
        ObjectDef {
            #[rust_sitter::leaf(text = "object")]
            _object: (),
            name: Box<EchoAst>,
            members: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endobject")]
            _endobject: (),
        },
        
        // Property definitions within objects
        PropertyDef {
            #[rust_sitter::leaf(text = "property")]
            _property: (),
            name: Box<EchoAst>,
            #[rust_sitter::leaf(text = "=")]
            _equals: (),
            value: Box<EchoAst>,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        
        // Verb definitions within objects
        VerbDef {
            #[rust_sitter::leaf(text = "verb")]
            _verb: (),
            name: Box<EchoAst>,
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            signature: Vec<EchoAst>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            body: Vec<EchoAst>,
            #[rust_sitter::leaf(text = "endverb")]
            _endverb: (),
        },
        
        // Comma separator
        #[rust_sitter::leaf(text = ",")]
        Comma,
        
        // Return statements
        Return {
            #[rust_sitter::leaf(text = "return")]
            _return: (),
            value: Box<EchoAst>,
            #[rust_sitter::leaf(text = ";")]
            _semicolon: (),
        },
        
        // Method calls
        #[rust_sitter::prec_left(12)]
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
    }
}

pub use echo::EchoAst;
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