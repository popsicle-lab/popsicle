/// Source span for error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// A node wrapping a value with its source span.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// ── Program ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Spanned<Declaration>>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Import(ImportDecl),
    Type(TypeDecl),
    Enum(EnumDecl),
    Function(FunctionDecl),
    Intent(IntentDecl),
    Safety(SafetyDecl),
    Theorem(TheoremDecl),
    Axiom(AxiomDecl),
    /// `goal "name" { rationale: ...; stakeholder: ...; measure: ...; realized_by: [...] }`
    Goal(GoalDecl),
    /// `coverage "name" { dimensions: { d1: [...]; d2: [...] } }`
    Coverage(CoverageDecl),
}

// ── Declarations ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ImportPath {
    /// Plugin import: `import smarthome` or `import finance.currency`
    Plugin(Vec<String>),
    /// File import: `import "./path/to/file.intent"`
    File(String),
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: ImportPath,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeExpr,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone)]
pub struct IntentDecl {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub params: Vec<Param>,
    pub clauses: Vec<Spanned<Clause>>,
}

#[derive(Debug, Clone)]
pub enum Clause {
    Require(Spanned<Expr>),
    Ensure(Spanned<Expr>),
    Invariant(Spanned<Expr>),
}

#[derive(Debug, Clone)]
pub struct SafetyDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub invariants: Vec<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct TheoremDecl {
    pub name: String,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct AxiomDecl {
    pub name: String,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct GoalDecl {
    /// Human-readable goal name (a string literal)
    pub name: String,
    pub rationale: Option<String>,
    /// Free-form list of stakeholder labels (e.g., "compliance", "finance")
    pub stakeholder: Vec<String>,
    pub measure: Option<String>,
    /// Identifier names of safety/intent declarations that realize this goal
    pub realized_by: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CoverageDecl {
    /// Human-readable coverage scenario name (a string literal)
    pub name: String,
    /// Each dimension: (name, list of values as expressions — usually idents/lits)
    pub dimensions: Vec<CoverageDim>,
}

#[derive(Debug, Clone)]
pub struct CoverageDim {
    pub name: String,
    pub values: Vec<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<AnnotationArg>,
}

#[derive(Debug, Clone)]
pub enum AnnotationArg {
    Positional(Spanned<Expr>),
    Named(String, Spanned<Expr>),
}

// ── Types ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Named(String),
    /// `module.TypeName` — qualified type reference
    Qualified(String, String),
    Generic(String, Vec<TypeExpr>),
}

impl std::fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpr::Named(n) => write!(f, "{n}"),
            TypeExpr::Qualified(module, name) => write!(f, "{module}.{name}"),
            TypeExpr::Generic(n, args) => {
                write!(f, "{n}<")?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{a}")?;
                }
                write!(f, ">")
            }
        }
    }
}

// ── Expressions ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    BoolLit(bool),
    StringLit(String),

    Ident(String),

    /// `x'` or `after(x)` — post-execution value
    Prime(Box<Spanned<Expr>>),

    /// `expr.field`
    FieldAccess(Box<Spanned<Expr>>, String),

    /// `expr[index]`
    Index(Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    BinOp(Box<Spanned<Expr>>, BinOp, Box<Spanned<Expr>>),
    UnaryOp(UnaryOp, Box<Spanned<Expr>>),

    /// `if cond then a else b`
    IfThenElse(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    /// `forall vars, body`
    Forall(Vec<TypedVar>, Box<Spanned<Expr>>),
    /// `exists vars, body`
    Exists(Vec<TypedVar>, Box<Spanned<Expr>>),

    /// `name(args)`
    Call(String, Vec<Spanned<Expr>>),

    Paren(Box<Spanned<Expr>>),
}

#[derive(Debug, Clone)]
pub struct TypedVar {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Implies,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Implies => write!(f, "==>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}
