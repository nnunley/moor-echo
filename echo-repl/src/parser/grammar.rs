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