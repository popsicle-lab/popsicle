use intent_syntax::ast::*;
use std::collections::HashSet;

/// A verification condition to be checked by SMT.
#[derive(Debug, Clone)]
pub struct VerificationCondition {
    pub name: String,
    pub kind: VcKind,
    /// Declarations needed (types, uninterpreted functions).
    pub declarations: Vec<SmtDecl>,
    /// Assertions: requires + invariants(unprimed).
    pub assumes: Vec<Spanned<Expr>>,
    /// Goal: ensures + invariants(as written, may contain primes).
    pub goals: Vec<Spanned<Expr>>,
    /// Safety rules merged from `safety` declarations.
    pub safety_rules: Vec<SafetySource>,
    /// If set, this VC cannot be encoded yet — reason string.
    pub unsupported: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SafetySource {
    pub safety_name: String,
    pub index: usize,
    pub expr: Spanned<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcKind {
    Intent,
    Theorem,
}

#[derive(Debug, Clone)]
pub enum SmtDecl {
    DeclareSort(String),
    DeclareConst(String, TypeExpr),
    DeclareFun(String, Vec<TypeExpr>, TypeExpr),
}

/// Remove all Prime nodes from an expression (replace `x'` with `x`).
fn unprime_expr(expr: &Spanned<Expr>) -> Spanned<Expr> {
    let node = match &expr.node {
        Expr::Prime(inner) => return unprime_expr(inner),
        Expr::IntLit(_) | Expr::BoolLit(_) | Expr::StringLit(_) | Expr::Ident(_) => {
            return expr.clone()
        }
        Expr::FieldAccess(base, field) => {
            Expr::FieldAccess(Box::new(unprime_expr(base)), field.clone())
        }
        Expr::Index(base, idx) => {
            Expr::Index(Box::new(unprime_expr(base)), Box::new(unprime_expr(idx)))
        }
        Expr::BinOp(l, op, r) => {
            Expr::BinOp(Box::new(unprime_expr(l)), *op, Box::new(unprime_expr(r)))
        }
        Expr::UnaryOp(op, o) => Expr::UnaryOp(*op, Box::new(unprime_expr(o))),
        Expr::IfThenElse(c, t, e) => Expr::IfThenElse(
            Box::new(unprime_expr(c)),
            Box::new(unprime_expr(t)),
            Box::new(unprime_expr(e)),
        ),
        Expr::Forall(vars, body) => Expr::Forall(vars.clone(), Box::new(unprime_expr(body))),
        Expr::Exists(vars, body) => Expr::Exists(vars.clone(), Box::new(unprime_expr(body))),
        Expr::Call(name, args) => Expr::Call(name.clone(), args.iter().map(unprime_expr).collect()),
        Expr::Paren(inner) => Expr::Paren(Box::new(unprime_expr(inner))),
    };
    Spanned::new(node, expr.span.clone())
}

/// Generate verification conditions from a program.
pub fn generate_vcs(prog: &Program) -> Vec<VerificationCondition> {
    let mut vcs = Vec::new();

    // Collect struct type names for theorem analysis
    let struct_names: HashSet<String> = prog
        .declarations
        .iter()
        .filter_map(|d| match &d.node {
            Declaration::Type(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    // Collect safety rules
    let mut safety_rules: Vec<SafetySource> = Vec::new();
    for decl in &prog.declarations {
        if let Declaration::Safety(s) = &decl.node {
            for (i, inv) in s.invariants.iter().enumerate() {
                safety_rules.push(SafetySource {
                    safety_name: s.name.clone(),
                    index: i + 1,
                    expr: inv.clone(),
                });
            }
        }
    }

    for decl in &prog.declarations {
        match &decl.node {
            Declaration::Intent(intent) => {
                let mut assumes = Vec::new();
                let mut goals = Vec::new();
                let mut declarations = Vec::new();

                for p in &intent.params {
                    declarations.push(SmtDecl::DeclareConst(p.name.clone(), p.ty.clone()));
                }

                for clause in &intent.clauses {
                    match &clause.node {
                        Clause::Require(e) => assumes.push(e.clone()),
                        // Ensures DEFINE post-state — they are assumed
                        Clause::Ensure(e) => assumes.push(e.clone()),
                        Clause::Invariant(e) => {
                            // Pre-state invariant: assumed (unprimed)
                            assumes.push(unprime_expr(e));
                            // Post-state invariant: must be proved
                            goals.push(e.clone());
                        }
                    }
                }

                // Safety rules: assume unprimed, prove primed
                let sr = safety_rules.clone();
                for rule in &sr {
                    assumes.push(unprime_expr(&rule.expr));
                    goals.push(rule.expr.clone());
                }

                vcs.push(VerificationCondition {
                    name: intent.name.clone(),
                    kind: VcKind::Intent,
                    declarations,
                    assumes,
                    goals,
                    safety_rules: sr,
                    unsupported: None,
                });
            }
            Declaration::Theorem(thm) => {
                // Check if theorem references struct-typed quantifier variables
                let has_struct_quantifiers = uses_struct_quantifiers(&thm.body, &struct_names);
                vcs.push(VerificationCondition {
                    name: thm.name.clone(),
                    kind: VcKind::Theorem,
                    declarations: Vec::new(),
                    assumes: Vec::new(),
                    goals: vec![thm.body.clone()],
                    safety_rules: Vec::new(),
                    unsupported: if has_struct_quantifiers {
                        Some("theorem uses struct-typed quantifiers (requires intent expansion, not yet implemented)".to_string())
                    } else {
                        None
                    },
                });
            }
            _ => {}
        }
    }

    vcs
}

/// Check if an expression contains forall/exists with struct-typed variables.
fn uses_struct_quantifiers(expr: &Spanned<Expr>, struct_names: &HashSet<String>) -> bool {
    match &expr.node {
        Expr::Forall(vars, body) | Expr::Exists(vars, body) => {
            let has_struct = vars.iter().any(|v| match &v.ty {
                TypeExpr::Named(name) => struct_names.contains(name),
                _ => false,
            });
            has_struct || uses_struct_quantifiers(body, struct_names)
        }
        Expr::BinOp(l, _, r) => {
            uses_struct_quantifiers(l, struct_names) || uses_struct_quantifiers(r, struct_names)
        }
        Expr::UnaryOp(_, o) => uses_struct_quantifiers(o, struct_names),
        Expr::Paren(inner) | Expr::Prime(inner) => uses_struct_quantifiers(inner, struct_names),
        Expr::IfThenElse(c, t, e) => {
            uses_struct_quantifiers(c, struct_names)
                || uses_struct_quantifiers(t, struct_names)
                || uses_struct_quantifiers(e, struct_names)
        }
        Expr::Call(_, args) => args
            .iter()
            .any(|a| uses_struct_quantifiers(a, struct_names)),
        Expr::FieldAccess(base, _) => uses_struct_quantifiers(base, struct_names),
        Expr::Index(base, idx) => {
            uses_struct_quantifiers(base, struct_names)
                || uses_struct_quantifiers(idx, struct_names)
        }
        _ => false,
    }
}
