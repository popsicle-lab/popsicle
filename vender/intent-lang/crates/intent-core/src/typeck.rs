use std::collections::HashMap;

use intent_syntax::ast::*;

use crate::{DiagLevel, Diagnostic};

// ── Type representation ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Named(String),
    Seq(Box<Type>),
    Set(Box<Type>),
    /// Enum variant resolved to its parent enum name
    EnumVariant(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Str => write!(f, "String"),
            Type::Named(n) => write!(f, "{n}"),
            Type::Seq(t) => write!(f, "Seq<{t}>"),
            Type::Set(t) => write!(f, "Set<{t}>"),
            Type::EnumVariant(n) => write!(f, "{n}"),
        }
    }
}

// ── Environment ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub params: Vec<Type>,
    pub ret: Type,
}

#[derive(Debug, Clone)]
pub struct IntentInfo {
    pub params: Vec<(String, Type)>,
}

pub struct TypeEnv {
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub functions: HashMap<String, FuncInfo>,
    pub intents: HashMap<String, IntentInfo>,
    pub safeties: HashMap<String, ()>,
    pub theorems: HashMap<String, ()>,
    pub locals: HashMap<String, Type>,
    pub errors: Vec<Diagnostic>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            intents: HashMap::new(),
            safeties: HashMap::new(),
            theorems: HashMap::new(),
            locals: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn resolve_type_expr(&self, te: &TypeExpr) -> Type {
        match te {
            TypeExpr::Named(n) => match n.as_str() {
                "Int" => Type::Int,
                "Bool" => Type::Bool,
                "String" => Type::Str,
                _ => Type::Named(n.clone()),
            },
            TypeExpr::Qualified(module, name) => {
                // Qualified type: resolve as `module.Name` for now
                Type::Named(format!("{module}.{name}"))
            }
            TypeExpr::Generic(name, args) => {
                let inner = self.resolve_type_expr(&args[0]);
                match name.as_str() {
                    "Seq" => Type::Seq(Box::new(inner)),
                    "Set" => Type::Set(Box::new(inner)),
                    _ => Type::Named(name.clone()),
                }
            }
        }
    }

    fn field_type(&self, base_type: &Type, field: &str) -> Option<Type> {
        match base_type {
            Type::Named(name) => {
                if let Some(info) = self.structs.get(name) {
                    info.fields.get(field).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn err(&mut self, code: &str, msg: String, span: &Span) {
        self.errors.push(Diagnostic {
            level: DiagLevel::Error,
            code: code.to_string(),
            message: msg,
            span: span.clone(),
            notes: Vec::new(),
        });
    }

    /// Look up an identifier: local variable, enum variant, or function/intent name.
    fn lookup_ident(&self, name: &str) -> Option<Type> {
        if let Some(t) = self.locals.get(name) {
            return Some(t.clone());
        }
        // Check if it's an enum variant
        for (enum_name, info) in &self.enums {
            if info.variants.contains(&name.to_string()) {
                return Some(Type::EnumVariant(enum_name.clone()));
            }
        }
        None
    }
}

// ── Type checking ────────────────────────────────────────────

pub fn check_program(prog: &Program) -> Vec<Diagnostic> {
    let mut env = TypeEnv::new();

    // Pass 1: register all type/enum/function/intent signatures
    for decl in &prog.declarations {
        match &decl.node {
            Declaration::Type(t) => {
                let mut fields = HashMap::new();
                for f in &t.fields {
                    fields.insert(f.name.clone(), env.resolve_type_expr(&f.ty));
                }
                env.structs.insert(t.name.clone(), StructInfo { fields });
            }
            Declaration::Enum(e) => {
                env.enums.insert(
                    e.name.clone(),
                    EnumInfo {
                        variants: e.variants.clone(),
                    },
                );
            }
            Declaration::Function(f) => {
                let params = f
                    .params
                    .iter()
                    .map(|p| env.resolve_type_expr(&p.ty))
                    .collect();
                let ret = env.resolve_type_expr(&f.return_type);
                env.functions
                    .insert(f.name.clone(), FuncInfo { params, ret });
            }
            Declaration::Intent(i) => {
                let params = i
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), env.resolve_type_expr(&p.ty)))
                    .collect();
                env.intents.insert(i.name.clone(), IntentInfo { params });
            }
            Declaration::Safety(s) => {
                env.safeties.insert(s.name.clone(), ());
            }
            Declaration::Theorem(t) => {
                env.theorems.insert(t.name.clone(), ());
            }
            _ => {}
        }
    }

    // Pass 2: check bodies
    for decl in &prog.declarations {
        match &decl.node {
            Declaration::Intent(i) => check_intent(&mut env, i),
            Declaration::Safety(s) => check_safety(&mut env, s),
            Declaration::Theorem(t) => check_theorem(&mut env, t),
            Declaration::Axiom(a) => check_axiom(&mut env, a),
            Declaration::Function(f) => check_function(&mut env, f),
            Declaration::Goal(g) => check_goal(&mut env, g, &decl.span),
            Declaration::Coverage(c) => check_coverage(&mut env, c),
            _ => {}
        }
    }

    env.errors
}

fn check_goal(env: &mut TypeEnv, goal: &GoalDecl, span: &Span) {
    // RFC A1: realized_by must reference existing safety / intent names.
    for ref_name in &goal.realized_by {
        if !env.intents.contains_key(ref_name)
            && !env.safeties.contains_key(ref_name)
            && !env.theorems.contains_key(ref_name)
        {
            env.errors.push(Diagnostic {
                level: DiagLevel::Warning,
                code: "W0010".to_string(),
                message: format!(
                    "goal `{}` realized_by references unknown declaration `{ref_name}`",
                    goal.name
                ),
                span: span.clone(),
                notes: vec!["expected an `intent` or `safety` name".to_string()],
            });
        }
    }
}

fn check_coverage(_env: &mut TypeEnv, _cov: &CoverageDecl) {
    // RFC A3: coverage dimension values are intentionally treated as
    // opaque domain labels — they often reference domain enum variants
    // or symbolic names that need not be defined as variables. The
    // coverage analysis tool only matches them syntactically against
    // intent/safety clause text, so strict type-checking would cause
    // false positives.
}

fn check_intent(env: &mut TypeEnv, intent: &IntentDecl) {
    let saved = env.locals.clone();
    for p in &intent.params {
        env.locals
            .insert(p.name.clone(), env.resolve_type_expr(&p.ty));
    }
    for clause in &intent.clauses {
        let expr = match &clause.node {
            Clause::Require(e) | Clause::Ensure(e) | Clause::Invariant(e) => e,
        };
        let ty = check_expr(env, expr);
        if ty != Some(Type::Bool) && ty.is_some() {
            env.err(
                "E0002",
                format!("clause expression must be Bool, found {}", ty.unwrap()),
                &expr.span,
            );
        }
    }
    env.locals = saved;
}

fn check_safety(env: &mut TypeEnv, safety: &SafetyDecl) {
    let saved = env.locals.clone();
    for p in &safety.params {
        env.locals
            .insert(p.name.clone(), env.resolve_type_expr(&p.ty));
    }
    for inv in &safety.invariants {
        let ty = check_expr(env, inv);
        if ty != Some(Type::Bool) && ty.is_some() {
            env.err(
                "E0002",
                format!("invariant must be Bool, found {}", ty.unwrap()),
                &inv.span,
            );
        }
    }
    env.locals = saved;
}

fn check_theorem(env: &mut TypeEnv, thm: &TheoremDecl) {
    let ty = check_expr(env, &thm.body);
    if ty != Some(Type::Bool) && ty.is_some() {
        env.err(
            "E0002",
            format!("theorem body must be Bool, found {}", ty.unwrap()),
            &thm.body.span,
        );
    }
}

fn check_axiom(env: &mut TypeEnv, ax: &AxiomDecl) {
    let ty = check_expr(env, &ax.body);
    if ty != Some(Type::Bool) && ty.is_some() {
        env.err(
            "E0002",
            format!("axiom body must be Bool, found {}", ty.unwrap()),
            &ax.body.span,
        );
    }
}

fn check_function(env: &mut TypeEnv, func: &FunctionDecl) {
    let saved = env.locals.clone();
    for p in &func.params {
        env.locals
            .insert(p.name.clone(), env.resolve_type_expr(&p.ty));
    }
    let _body_ty = check_expr(env, &func.body);
    env.locals = saved;
}

/// Check an expression and return its type (or None if unresolvable).
fn check_expr(env: &mut TypeEnv, expr: &Spanned<Expr>) -> Option<Type> {
    match &expr.node {
        Expr::IntLit(_) => Some(Type::Int),
        Expr::BoolLit(_) => Some(Type::Bool),
        Expr::StringLit(_) => Some(Type::Str),

        Expr::Ident(name) => {
            if let Some(ty) = env.lookup_ident(name) {
                Some(ty)
            } else {
                env.err("E0003", format!("undefined variable `{name}`"), &expr.span);
                None
            }
        }

        Expr::Prime(inner) => check_expr(env, inner),

        Expr::FieldAccess(base, field) => {
            let base_ty = check_expr(env, base)?;
            if let Some(ft) = env.field_type(&base_ty, field) {
                Some(ft)
            } else {
                env.err(
                    "E0004",
                    format!("type `{base_ty}` has no field `{field}`"),
                    &expr.span,
                );
                None
            }
        }

        Expr::Index(base, idx) => {
            let base_ty = check_expr(env, base)?;
            let idx_ty = check_expr(env, idx);
            if let Some(ref it) = idx_ty {
                if *it != Type::Int {
                    env.err("E0001", format!("index must be Int, found {it}"), &idx.span);
                }
            }
            match base_ty {
                Type::Seq(inner) => Some(*inner),
                _ => {
                    env.err(
                        "E0005",
                        format!("cannot index into `{base_ty}`"),
                        &expr.span,
                    );
                    None
                }
            }
        }

        Expr::BinOp(lhs, op, rhs) => {
            let lt = check_expr(env, lhs);
            let rt = check_expr(env, rhs);
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                    if lt.as_ref() == Some(&Type::Int) && rt.as_ref() == Some(&Type::Int) {
                        Some(Type::Int)
                    } else {
                        if lt.is_some() && rt.is_some() {
                            env.err(
                                "E0001",
                                format!(
                                    "arithmetic on non-Int: {} {op} {}",
                                    lt.unwrap(),
                                    rt.unwrap()
                                ),
                                &expr.span,
                            );
                        }
                        None
                    }
                }
                BinOp::Eq | BinOp::Neq => Some(Type::Bool),
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Some(Type::Bool),
                BinOp::And | BinOp::Or | BinOp::Implies => Some(Type::Bool),
            }
        }

        Expr::UnaryOp(op, operand) => {
            let ot = check_expr(env, operand);
            match op {
                UnaryOp::Not => Some(Type::Bool),
                UnaryOp::Neg => {
                    if ot.as_ref() == Some(&Type::Int) {
                        Some(Type::Int)
                    } else {
                        if ot.is_some() {
                            env.err(
                                "E0001",
                                format!("negation on non-Int: {}", ot.unwrap()),
                                &expr.span,
                            );
                        }
                        None
                    }
                }
            }
        }

        Expr::IfThenElse(cond, then_e, else_e) => {
            let _ct = check_expr(env, cond);
            let tt = check_expr(env, then_e);
            let _et = check_expr(env, else_e);
            tt
        }

        Expr::Forall(vars, body) | Expr::Exists(vars, body) => {
            let saved = env.locals.clone();
            for v in vars {
                env.locals
                    .insert(v.name.clone(), env.resolve_type_expr(&v.ty));
            }
            let bt = check_expr(env, body);
            env.locals = saved;
            if bt != Some(Type::Bool) && bt.is_some() {
                env.err(
                    "E0002",
                    format!("quantifier body must be Bool, found {}", bt.unwrap()),
                    &body.span,
                );
            }
            Some(Type::Bool)
        }

        Expr::Call(name, args) => {
            for arg in args {
                check_expr(env, arg);
            }
            if let Some(fi) = env.functions.get(name).cloned() {
                Some(fi.ret)
            } else if env.intents.contains_key(name) {
                Some(Type::Bool)
            } else {
                // Unknown function — allow (might be from an import)
                None
            }
        }

        Expr::Paren(inner) => check_expr(env, inner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intent_syntax::parse;

    #[test]
    fn check_transfer_no_errors() {
        let src = std::fs::read_to_string("../../examples/basics/transfer.intent").unwrap();
        let prog = parse(&src).unwrap();
        let diags = check_program(&prog);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.level == DiagLevel::Error)
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
    }

    #[test]
    fn detect_undefined_variable() {
        let src = r#"
intent Bad(x: Int) {
  require y > 0
}
"#;
        let prog = parse(src).unwrap();
        let diags = check_program(&prog);
        assert!(
            diags.iter().any(|d| d.code == "E0003"),
            "should detect undefined var"
        );
    }
}
