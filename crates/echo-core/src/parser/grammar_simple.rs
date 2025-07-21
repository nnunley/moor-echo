// Echo language grammar using rust-sitter annotations
// Simplified version matching tree-sitter-moo precedence
#[rust_sitter::grammar("echo")]
pub mod echo {
    #[rust_sitter::extra(pattern = r"\s+")]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _whitespace: (),
    }

    #[derive(Debug, PartialEq)]
    #[rust_sitter::language]
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
        
        // Assignment (precedence 1, right associative)
        #[rust_sitter::prec_right(1)]
        Assignment {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "=")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Logical OR (precedence 3, left associative)
        #[rust_sitter::prec_left(3)]
        Or {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "||")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Logical AND (precedence 3, left associative)
        #[rust_sitter::prec_left(3)]
        And {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "&&")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Equality (precedence 4, left associative)
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
        
        // Addition/Subtraction (precedence 7, left associative)
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
        
        // Multiplication/Division (precedence 8, left associative)
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
        
        // Power (precedence 9, right associative)
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
        
        // Property access (precedence 11, left associative)
        #[rust_sitter::prec_left(11)]
        PropertyAccess {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Box<Expression>,
        },
        
        // Method call (precedence 11, left associative)
        #[rust_sitter::prec_left(11)]
        MethodCall {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            method: Box<Expression>,
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
        
        // Index access (precedence 11, left associative)
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

pub use echo::Expression;
pub use echo::parse as parse_echo;

// For backwards compatibility
pub type EchoAst = Expression;