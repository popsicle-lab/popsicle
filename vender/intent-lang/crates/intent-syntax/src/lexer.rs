use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    // ── Keywords ─────────────────────────────────────────
    #[token("type")]
    Type,
    #[token("enum")]
    Enum,
    #[token("intent")]
    Intent,
    #[token("safety")]
    Safety,
    #[token("theorem")]
    Theorem,
    #[token("axiom")]
    Axiom,
    #[token("function")]
    Function,
    #[token("import")]
    Import,
    #[token("require")]
    Require,
    #[token("ensure")]
    Ensure,
    #[token("invariant")]
    Invariant,
    #[token("forall")]
    Forall,
    #[token("exists")]
    Exists,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("after")]
    After,
    #[token("as")]
    As,

    // ── Requirements-modeling keywords (RFC: A1, A3) ─────
    #[token("goal")]
    Goal,
    #[token("rationale")]
    Rationale,
    #[token("stakeholder")]
    Stakeholder,
    #[token("measure")]
    Measure,
    #[token("realized_by")]
    RealizedBy,
    #[token("coverage")]
    Coverage,
    #[token("dimensions")]
    Dimensions,

    // ── Symbols ──────────────────────────────────────────
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("->")]
    Arrow,
    #[token("@")]
    At,
    #[token("'")]
    Prime,

    // ── Operators ────────────────────────────────────────
    #[token("==>")]
    Implies,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    Bang,

    // ── Literals ─────────────────────────────────────────
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLit(i64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLit(String),

    // ── Identifiers ──────────────────────────────────────
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Type => write!(f, "type"),
            Token::Enum => write!(f, "enum"),
            Token::Intent => write!(f, "intent"),
            Token::Safety => write!(f, "safety"),
            Token::Theorem => write!(f, "theorem"),
            Token::Axiom => write!(f, "axiom"),
            Token::Function => write!(f, "function"),
            Token::Import => write!(f, "import"),
            Token::Require => write!(f, "require"),
            Token::Ensure => write!(f, "ensure"),
            Token::Invariant => write!(f, "invariant"),
            Token::Forall => write!(f, "forall"),
            Token::Exists => write!(f, "exists"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::After => write!(f, "after"),
            Token::As => write!(f, "as"),
            Token::Goal => write!(f, "goal"),
            Token::Rationale => write!(f, "rationale"),
            Token::Stakeholder => write!(f, "stakeholder"),
            Token::Measure => write!(f, "measure"),
            Token::RealizedBy => write!(f, "realized_by"),
            Token::Coverage => write!(f, "coverage"),
            Token::Dimensions => write!(f, "dimensions"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, "->"),
            Token::At => write!(f, "@"),
            Token::Prime => write!(f, "'"),
            Token::Implies => write!(f, "==>"),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::AmpAmp => write!(f, "&&"),
            Token::PipePipe => write!(f, "||"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Bang => write!(f, "!"),
            Token::IntLit(v) => write!(f, "{v}"),
            Token::StringLit(s) => write!(f, "\"{s}\""),
            Token::Ident(s) => write!(f, "{s}"),
        }
    }
}
