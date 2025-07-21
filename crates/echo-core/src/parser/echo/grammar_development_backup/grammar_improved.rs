// MOO-inspired improved grammar for Echo
// This grammar separates statements and expressions, uses MOO's precedence table,
// and unifies pattern matching across the language

#[rust_sitter::grammar("echo_improved")]
pub mod echo_improved {
    #[rust_sitter::extra(pattern = r"\s+")]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _whitespace: (),
    }

    // Top-level program structure - statements only
    #[rust_sitter::language]
    #[derive(Debug, PartialEq, Clone)]
    pub struct Program {
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<Statement>,
    }

    // Statement types (cannot be used as expressions)
    #[derive(Debug, PartialEq, Clone)]
    pub enum Statement {
        Expression(Box<ExpressionStatement>),
        Let(Box<LetStatement>),
        Const(Box<ConstStatement>),
        Global(Box<GlobalStatement>),
        If(Box<IfStatement>),
        While(Box<WhileStatement>),
        For(Box<ForStatement>),
        Fork(Box<ForkStatement>),
        Try(Box<TryStatement>),
        Return(Box<ReturnStatement>),
        Break(Box<BreakStatement>),
        Continue(Box<ContinueStatement>),
        Block(Box<BlockStatement>),
    }

    // Expression wrapper for statements
    #[derive(Debug, PartialEq, Clone)]
    pub struct ExpressionStatement {
        pub expression: Box<Expression>,
    }

    // Variable declarations
    #[derive(Debug, PartialEq, Clone)]
    pub struct LetStatement {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
        pub pattern: Box<Pattern>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ConstStatement {
        #[rust_sitter::leaf(text = "const")]
        _const: (),
        pub pattern: Box<Pattern>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct GlobalStatement {
        #[rust_sitter::leaf(text = "global")]
        _global: (),
        pub pattern: Box<Pattern>,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    // Control flow statements
    #[derive(Debug, PartialEq, Clone)]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub then_body: Vec<Statement>,
        #[rust_sitter::repeat(non_empty = false)]
        pub elseif_clauses: Vec<ElseIfClause>,
        #[rust_sitter::node(optional = true)]
        pub else_clause: Option<Box<ElseClause>>,
        #[rust_sitter::leaf(text = "endif")]
        _endif: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ElseIfClause {
        #[rust_sitter::leaf(text = "elseif")]
        _elseif: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct WhileStatement {
        #[rust_sitter::node(optional = true)]
        pub label: Option<Identifier>,
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub condition: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
        #[rust_sitter::leaf(text = "endwhile")]
        _endwhile: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ForStatement {
        #[rust_sitter::node(optional = true)]
        pub label: Option<Identifier>,
        #[rust_sitter::leaf(text = "for")]
        _for: (),
        pub pattern: Box<Pattern>,
        #[rust_sitter::leaf(text = "in")]
        _in: (),
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub iterable: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
        #[rust_sitter::leaf(text = "endfor")]
        _endfor: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ForkStatement {
        #[rust_sitter::leaf(text = "fork")]
        _fork: (),
        pub id: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub time: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
        #[rust_sitter::leaf(text = "endfork")]
        _endfork: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct TryStatement {
        #[rust_sitter::leaf(text = "try")]
        _try: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
        #[rust_sitter::repeat(non_empty = false)]
        pub handlers: Vec<ExceptClause>,
        #[rust_sitter::node(optional = true)]
        pub finally_clause: Option<Box<FinallyClause>>,
        #[rust_sitter::leaf(text = "endtry")]
        _endtry: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ExceptClause {
        #[rust_sitter::leaf(text = "except")]
        _except: (),
        #[rust_sitter::node(optional = true)]
        pub id: Option<Identifier>,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub codes: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FinallyClause {
        #[rust_sitter::leaf(text = "finally")]
        _finally: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    // Jump statements
    #[derive(Debug, PartialEq, Clone)]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        #[rust_sitter::node(optional = true)]
        pub value: Option<Box<Expression>>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        _break: (),
        #[rust_sitter::node(optional = true)]
        pub label: Option<Identifier>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        _continue: (),
        #[rust_sitter::node(optional = true)]
        pub label: Option<Identifier>,
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    // Block statement
    #[derive(Debug, PartialEq, Clone)]
    pub struct BlockStatement {
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    // Expression types (can be used as values)
    #[derive(Debug, PartialEq, Clone)]
    pub enum Expression {
        // Literals
        Integer(IntegerLiteral),
        Float(FloatLiteral),
        String(StringLiteral),
        True,
        False,
        Null,
        
        // Identifiers and references
        Identifier(Identifier),
        ObjectRef(ObjectRefLiteral),
        SystemProperty(SystemPropertyLiteral),
        Symbol(SymbolLiteral),
        ErrorCode(ErrorCodeLiteral),
        
        // Collections
        List(Box<ListLiteral>),
        Map(Box<MapLiteral>),
        Range(Box<RangeLiteral>),
        ListComprehension {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            expression: Box<Expression>,
            #[rust_sitter::leaf(text = "for")]
            _for: (),
            pattern: Box<Pattern>,
            #[rust_sitter::leaf(text = "in")]
            _in: (),
            iterable: Box<Expression>,
            #[rust_sitter::node(optional = true)]
            condition: Option<Box<Expression>>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
        
        // Binary operations (in precedence order)
        #[rust_sitter::prec_right(1)]
        Assignment {
            target: Box<Expression>,
            #[rust_sitter::leaf(text = "=")]
            _eq: (),
            value: Box<Expression>,
        },
        #[rust_sitter::prec(2)]
        Conditional {
            condition: Box<Expression>,
            #[rust_sitter::leaf(text = "?")]
            _question: (),
            then_expr: Box<Expression>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            else_expr: Box<Expression>,
        },
        #[rust_sitter::prec_left(3)]
        Or {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "||")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(4)]
        And {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "&&")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(5)]
        In {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "in")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(6)]
        Equal {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "==")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(6)]
        NotEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "!=")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(7)]
        Less {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "<")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(7)]
        LessEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "<=")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(7)]
        Greater {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = ">")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(7)]
        GreaterEqual {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = ">=")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(8)]
        BitwiseOr {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "|")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(9)]
        BitwiseXor {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "^")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(10)]
        BitwiseAnd {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "&")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(11)]
        ShiftLeft {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "<<")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(11)]
        ShiftRight {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = ">>")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(12)]
        Add {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "+")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(12)]
        Subtract {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "-")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(13)]
        Multiply {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "*")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(13)]
        Divide {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "/")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(13)]
        Modulo {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "%")]
            _op: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_right(14)]
        Power {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "**")]
            _op: (),
            right: Box<Expression>,
        },
        
        // Unary operations
        #[rust_sitter::prec_right(15)]
        Not {
            #[rust_sitter::leaf(text = "!")]
            _op: (),
            operand: Box<Expression>,
        },
        #[rust_sitter::prec_right(15)]
        Negate {
            #[rust_sitter::leaf(text = "-")]
            _op: (),
            operand: Box<Expression>,
        },
        #[rust_sitter::prec_right(15)]
        BitwiseNot {
            #[rust_sitter::leaf(text = "~")]
            _op: (),
            operand: Box<Expression>,
        },
        
        // Access operations
        #[rust_sitter::prec_left(16)]
        PropertyAccess {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ".")]
            _dot: (),
            property: Identifier,
        },
        #[rust_sitter::prec_left(16)]
        MethodCall {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = ":")]
            _colon: (),
            method: Identifier,
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
        #[rust_sitter::prec_left(16)]
        IndexAccess {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = "[")]
            _lbracket: (),
            index: Box<Expression>,
            #[rust_sitter::leaf(text = "]")]
            _rbracket: (),
        },
        #[rust_sitter::prec_left(16)]
        Slice {
            object: Box<Expression>,
            #[rust_sitter::leaf(text = "[")]
            _lbracket: (),
            #[rust_sitter::node(optional = true)]
            start: Option<Box<Expression>>,
            #[rust_sitter::leaf(text = "..")]
            _dots: (),
            #[rust_sitter::node(optional = true)]
            end: Option<Box<Expression>>,
            #[rust_sitter::leaf(text = "]")]
            _rbracket: (),
        },
        
        // Function-related
        #[rust_sitter::prec_left(16)]
        Call {
            function: Box<Expression>,
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            arguments: Vec<CallArgument>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        #[rust_sitter::prec_right(3)]
        Lambda {
            parameters: Box<Pattern>,
            #[rust_sitter::leaf(text = "=>")]
            _arrow: (),
            body: LambdaBody,
        },
        #[rust_sitter::prec(10)]
        Function {
            #[rust_sitter::leaf(text = "function")]
            _function: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            parameters: Box<Pattern>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            body: Vec<Statement>,
            #[rust_sitter::leaf(text = "endfunction")]
            _endfunction: (),
        },
        
        // Special expressions
        Pass {
            #[rust_sitter::leaf(text = "pass")]
            _pass: (),
            #[rust_sitter::leaf(text = "(")]
            _lparen: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            args: Vec<Expression>,
            #[rust_sitter::leaf(text = ")")]
            _rparen: (),
        },
        TryExpression {
            #[rust_sitter::leaf(text = "`")]
            _backtick_open: (),
            expression: Box<Expression>,
            #[rust_sitter::leaf(text = "!")]
            _bang: (),
            codes: Box<Expression>,
            #[rust_sitter::node(optional = true)]
            default: Option<Box<Expression>>,
            #[rust_sitter::leaf(text = "'")]
            _backtick_close: (),
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

    // Unified Pattern type (replaces ParamPattern and BindingPattern)
    #[derive(Debug, PartialEq, Clone)]
    pub enum Pattern {
        Identifier(Identifier),
        List {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            #[rust_sitter::repeat(non_empty = false)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            elements: Vec<PatternElement>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
        Rest {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            name: Identifier,
        },
        Ignore,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum PatternElement {
        Simple(Identifier),
        Optional {
            #[rust_sitter::leaf(text = "?")]
            _question: (),
            name: Identifier,
            #[rust_sitter::leaf(text = "=")]
            _eq: (),
            default: Box<Expression>,
        },
        Rest {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            name: Identifier,
        },
    }

    // Expression node types
    #[derive(Debug, PartialEq, Clone)]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        pub name: String,
    }

    // Literal wrapper types  
    #[derive(Debug, PartialEq, Clone)]
    pub struct IntegerLiteral {
        #[rust_sitter::leaf(pattern = r"-?[0-9]+", transform = |v| v.parse().unwrap())]
        pub value: i64,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FloatLiteral {
        #[rust_sitter::leaf(pattern = r"-?[0-9]+\.[0-9]+", transform = |v| v.parse().unwrap())]
        pub value: f64,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |v| v[1..v.len()-1].to_string())]
        pub value: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ObjectRefLiteral {
        #[rust_sitter::leaf(pattern = r"#[0-9]+", transform = |v| v[1..].parse().unwrap())]
        pub value: i64,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct SystemPropertyLiteral {
        #[rust_sitter::leaf(pattern = r"\$[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v[1..].to_string())]
        pub value: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct SymbolLiteral {
        #[rust_sitter::leaf(pattern = r"'[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v[1..].to_string())]
        pub value: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ErrorCodeLiteral {
        #[rust_sitter::leaf(pattern = r"E_[A-Z_]+", transform = |v| v.to_string())]
        pub value: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ListLiteral {
        #[rust_sitter::leaf(text = "{")]
        _lbrace: (),
        #[rust_sitter::repeat(non_empty = false)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub elements: Vec<ListElement>,
        #[rust_sitter::leaf(text = "}")]
        _rbrace: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum ListElement {
        Expression(Box<Expression>),
        Scatter {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            expression: Box<Expression>,
        },
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct MapLiteral {
        #[rust_sitter::leaf(text = "[")]
        _lbracket: (),
        #[rust_sitter::repeat(non_empty = false)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub entries: Vec<MapEntry>,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct MapEntry {
        pub key: Box<Expression>,
        #[rust_sitter::leaf(text = "->")]
        _arrow: (),
        pub value: Box<Expression>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct RangeLiteral {
        #[rust_sitter::leaf(text = "[")]
        _lbracket: (),
        pub start: Box<Expression>,
        #[rust_sitter::leaf(text = "..")]
        _dots: (),
        pub end: Box<Expression>,
        #[rust_sitter::leaf(text = "]")]
        _rbracket: (),
    }



    #[derive(Debug, PartialEq, Clone)]
    pub enum CallArgument {
        Expression(Box<Expression>),
        Scatter {
            #[rust_sitter::leaf(text = "@")]
            _at: (),
            expression: Box<Expression>,
        },
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum LambdaBody {
        Expression(Box<Expression>),
        Block {
            #[rust_sitter::leaf(text = "{")]
            _lbrace: (),
            #[rust_sitter::repeat(non_empty = false)]
            statements: Vec<Statement>,
            #[rust_sitter::leaf(text = "}")]
            _rbrace: (),
        },
    }

    // Object definitions (top-level)
    #[derive(Debug, PartialEq, Clone)]
    pub struct ObjectDefinition {
        #[rust_sitter::leaf(text = "$")]
        _dollar: (),
        pub name: Identifier,
        #[rust_sitter::node(optional = true)]
        pub parent: Option<Box<ParentClause>>,
        #[rust_sitter::repeat(non_empty = false)]
        pub members: Vec<ObjectMember>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct ParentClause {
        #[rust_sitter::leaf(text = "isa")]
        _isa: (),
        pub parent: Box<Expression>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum ObjectMember {
        Property(Box<PropertyDefinition>),
        Verb(Box<VerbDefinition>),
        Event(Box<EventDefinition>),
        Query(Box<QueryDefinition>),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct PropertyDefinition {
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Box<Expression>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct VerbDefinition {
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub parameters: Box<Pattern>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct EventDefinition {
        #[rust_sitter::leaf(text = "on")]
        _on: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub parameters: Box<Pattern>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct QueryDefinition {
        #[rust_sitter::leaf(text = "query")]
        _query: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub parameters: Box<Pattern>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }
}

// Re-export for convenience
pub use echo_improved::*;