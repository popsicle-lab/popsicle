use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use intent_syntax::ast::*;

use crate::vcgen::{VcKind, VerificationCondition};

// ── Result type ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum VerifyResult {
    Verified,
    Failed { counterexample: String },
    Unknown { reason: String },
    Error { message: String },
}

// ── Type info for encoding ───────────────────────────────────

#[derive(Debug, Clone)]
struct FieldInfo {
    sort: String,
}

struct TypeInfo {
    /// struct_name -> { field_name -> smt_sort }
    structs: HashMap<String, HashMap<String, FieldInfo>>,
    enums: HashSet<String>,
}

impl TypeInfo {
    fn from_program(prog: &Program) -> Self {
        let mut structs = HashMap::new();
        let mut enums = HashSet::new();
        for decl in &prog.declarations {
            match &decl.node {
                Declaration::Type(t) => {
                    let mut fields = HashMap::new();
                    for f in &t.fields {
                        fields.insert(
                            f.name.clone(),
                            FieldInfo {
                                sort: type_expr_to_smt(&f.ty),
                            },
                        );
                    }
                    structs.insert(t.name.clone(), fields);
                }
                Declaration::Enum(e) => {
                    enums.insert(e.name.clone());
                }
                _ => {}
            }
        }
        TypeInfo { structs, enums }
    }

    fn field_sort(&self, struct_name: &str, field: &str) -> Option<&str> {
        self.structs
            .get(struct_name)
            .and_then(|f| f.get(field))
            .map(|fi| fi.sort.as_str())
    }
}

fn type_expr_to_smt(te: &TypeExpr) -> String {
    match te {
        TypeExpr::Named(n) => match n.as_str() {
            "Int" => "Int".to_string(),
            "Bool" => "Bool".to_string(),
            "String" => "String".to_string(),
            other => other.to_string(),
        },
        TypeExpr::Qualified(module, name) => {
            // Use mangled name for SMT: module_Type
            let full = format!("{module}_{name}");
            match full.as_str() {
                "Int" | "Bool" | "String" => full,
                _ => full,
            }
        }
        TypeExpr::Generic(name, args) => match name.as_str() {
            "Seq" => format!("(Array Int {})", type_expr_to_smt(&args[0])),
            "Set" => format!("(Array {} Bool)", type_expr_to_smt(&args[0])),
            _ => name.clone(),
        },
    }
}

// ── SMT-LIB2 encoder ────────────────────────────────────────

pub struct SmtEncoder {
    /// Lines of SMT output, built top-down.
    lines: Vec<String>,
    declared: HashSet<String>,
    type_info: TypeInfo,
    /// param_name -> type_name (for struct parameters)
    param_types: HashMap<String, String>,
}

impl SmtEncoder {
    pub fn new(prog: &Program) -> Self {
        Self {
            lines: Vec::new(),
            declared: HashSet::new(),
            type_info: TypeInfo::from_program(prog),
            param_types: HashMap::new(),
        }
    }

    pub fn encode_vc(&mut self, vc: &VerificationCondition, prog: &Program) {
        self.lines.clear();
        self.declared.clear();
        self.param_types.clear();

        self.emit("(set-logic ALL)");

        // Declare enum datatypes
        for decl in &prog.declarations {
            if let Declaration::Enum(e) = &decl.node {
                let variants: Vec<String> = e.variants.iter().map(|v| format!("({v})")).collect();
                self.emit(&format!(
                    "(declare-datatype {} ({}))",
                    e.name,
                    variants.join(" ")
                ));
                self.declared.insert(e.name.clone());
            }
        }

        // Flatten struct parameters into individual field constants
        for d in &vc.declarations {
            match d {
                crate::vcgen::SmtDecl::DeclareConst(name, ty) => {
                    let type_name = match ty {
                        TypeExpr::Named(n) => n.clone(),
                        TypeExpr::Qualified(m, n) => format!("{m}.{n}"),
                        TypeExpr::Generic(n, _) => n.clone(),
                    };

                    if self.type_info.structs.contains_key(&type_name) {
                        let field_entries: Vec<(String, String)> = self.type_info.structs
                            [&type_name]
                            .iter()
                            .map(|(fname, fi)| (fname.clone(), fi.sort.clone()))
                            .collect();
                        self.param_types.insert(name.clone(), type_name.clone());
                        for (field_name, sort) in &field_entries {
                            let const_name = format!("{name}_{field_name}");
                            self.declare_const(&const_name, sort);
                            self.declare_const(&format!("{const_name}_prime"), sort);
                        }
                    } else {
                        let sort = type_expr_to_smt(ty);
                        self.declare_const(name, &sort);
                        self.declare_const(&format!("{name}_prime"), &sort);
                    }
                }
                _ => {}
            }
        }

        match vc.kind {
            VcKind::Intent => {
                // Assert assumes (requires + ensures + invariant-pre)
                for e in &vc.assumes {
                    let smt = self.expr_to_smt(e);
                    self.emit(&format!("(assert {smt})"));
                }
                // Negate conjunction of goals (invariants-post)
                if !vc.goals.is_empty() {
                    let goal_parts: Vec<String> =
                        vc.goals.iter().map(|e| self.expr_to_smt(e)).collect();
                    let conj = if goal_parts.len() == 1 {
                        goal_parts[0].clone()
                    } else {
                        format!("(and {})", goal_parts.join(" "))
                    };
                    self.emit(&format!("(assert (not {conj}))"));
                }
            }
            VcKind::Theorem => {
                // Theorems with struct-typed quantifiers are not yet supported.
                // For now, we skip encoding if struct types appear in quantifiers.
                if let Some(body) = vc.goals.first() {
                    let smt = self.expr_to_smt(body);
                    self.emit(&format!("(assert (not {smt}))"));
                }
            }
        }

        self.emit("(check-sat)");
        // get-model only makes sense when sat; Z3 will error on unsat.
        // We handle this by checking the first line of output.
    }

    pub fn get_output(&self) -> String {
        self.lines.join("\n")
    }

    fn declare_const(&mut self, name: &str, sort: &str) {
        if self.declared.insert(name.to_string()) {
            self.emit(&format!("(declare-const {name} {sort})"));
        }
    }

    fn emit(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    // ── Expression encoding ──────────────────────────────

    fn expr_to_smt(&mut self, expr: &Spanned<Expr>) -> String {
        match &expr.node {
            Expr::IntLit(v) => {
                if *v < 0 {
                    format!("(- {})", -v)
                } else {
                    v.to_string()
                }
            }
            Expr::BoolLit(b) => b.to_string(),
            Expr::StringLit(s) => format!("\"{s}\""),

            Expr::Ident(name) => {
                // Check if it's an enum variant
                for decl_name in &self.type_info.enums {
                    if let Some(fields) = self.type_info.structs.get(decl_name) {
                        // not an enum
                        let _ = fields;
                    }
                }
                self.sanitize(name)
            }

            Expr::Prime(inner) => {
                let base = self.expr_to_smt(inner);
                format!("{base}_prime")
            }

            Expr::FieldAccess(base, field) => {
                let b = self.expr_to_smt(base);
                let name = format!("{b}_{field}");
                // Ensure this field constant is declared
                if !self.declared.contains(&name) {
                    // Try to find the sort from type info
                    let sort = self
                        .resolve_field_sort(&b, field)
                        .unwrap_or_else(|| "Int".to_string());
                    self.declare_const(&name, &sort);
                    // Also declare primed
                    self.declare_const(&format!("{name}_prime"), &sort);
                }
                name
            }

            Expr::Index(base, idx) => {
                let b = self.expr_to_smt(base);
                let i = self.expr_to_smt(idx);
                format!("(select {b} {i})")
            }

            Expr::BinOp(lhs, op, rhs) => {
                let l = self.expr_to_smt(lhs);
                let r = self.expr_to_smt(rhs);
                match op {
                    BinOp::Add => format!("(+ {l} {r})"),
                    BinOp::Sub => format!("(- {l} {r})"),
                    BinOp::Mul => format!("(* {l} {r})"),
                    BinOp::Div => format!("(div {l} {r})"),
                    BinOp::Mod => format!("(mod {l} {r})"),
                    BinOp::Eq => format!("(= {l} {r})"),
                    BinOp::Neq => format!("(not (= {l} {r}))"),
                    BinOp::Lt => format!("(< {l} {r})"),
                    BinOp::Le => format!("(<= {l} {r})"),
                    BinOp::Gt => format!("(> {l} {r})"),
                    BinOp::Ge => format!("(>= {l} {r})"),
                    BinOp::And => format!("(and {l} {r})"),
                    BinOp::Or => format!("(or {l} {r})"),
                    BinOp::Implies => format!("(=> {l} {r})"),
                }
            }

            Expr::UnaryOp(op, operand) => {
                let o = self.expr_to_smt(operand);
                match op {
                    UnaryOp::Not => format!("(not {o})"),
                    UnaryOp::Neg => format!("(- {o})"),
                }
            }

            Expr::IfThenElse(c, t, e) => {
                let sc = self.expr_to_smt(c);
                let st = self.expr_to_smt(t);
                let se = self.expr_to_smt(e);
                format!("(ite {sc} {st} {se})")
            }

            Expr::Forall(vars, body) => {
                let bindings: Vec<String> = vars
                    .iter()
                    .map(|v| {
                        let sort = type_expr_to_smt(&v.ty);
                        format!("({} {})", self.sanitize(&v.name), sort)
                    })
                    .collect();
                let b = self.expr_to_smt(body);
                format!("(forall ({}) {})", bindings.join(" "), b)
            }

            Expr::Exists(vars, body) => {
                let bindings: Vec<String> = vars
                    .iter()
                    .map(|v| {
                        let sort = type_expr_to_smt(&v.ty);
                        format!("({} {})", self.sanitize(&v.name), sort)
                    })
                    .collect();
                let b = self.expr_to_smt(body);
                format!("(exists ({}) {})", bindings.join(" "), b)
            }

            Expr::Call(name, args) => {
                if args.is_empty() {
                    self.sanitize(name)
                } else {
                    let smt_args: Vec<String> = args.iter().map(|a| self.expr_to_smt(a)).collect();
                    format!("({} {})", self.sanitize(name), smt_args.join(" "))
                }
            }

            Expr::Paren(inner) => self.expr_to_smt(inner),
        }
    }

    fn resolve_field_sort(&self, base_var: &str, field: &str) -> Option<String> {
        // Find what struct type this variable is
        if let Some(type_name) = self.param_types.get(base_var) {
            return self
                .type_info
                .field_sort(type_name, field)
                .map(|s| s.to_string());
        }
        // Try to find nested: e.g., base_var = "home_frontDoor", field = "locked"
        // Walk through all structs to find a match
        for (_, fields) in &self.type_info.structs {
            for (fname, fi) in fields {
                let compound = format!("{base_var}_{fname}");
                if compound == format!("{base_var}_{field}") {
                    return Some(fi.sort.clone());
                }
            }
        }
        None
    }

    fn sanitize(&self, name: &str) -> String {
        match name {
            "and" | "or" | "not" | "true" | "false" | "ite" | "let" | "forall" | "exists" => {
                format!("intent_{name}")
            }
            _ => name.to_string(),
        }
    }
}

// ── Z3 invocation ────────────────────────────────────────────

pub fn run_z3(smt_input: &str) -> VerifyResult {
    fn spawn_z3(input: &str) -> Result<String, VerifyResult> {
        let result = Command::new("z3")
            .args(["-smt2", "-in", "-t:5000"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(c) => c,
            Err(e) => {
                return Err(VerifyResult::Error {
                    message: format!("failed to run z3: {e}. Is Z3 installed and on PATH?"),
                });
            }
        };

        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(input.as_bytes());
        }

        match child.wait_with_output() {
            Ok(o) => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Err(e) => Err(VerifyResult::Error {
                message: format!("z3 process error: {e}"),
            }),
        }
    }

    let stdout = match spawn_z3(smt_input) {
        Ok(s) => s,
        Err(r) => return r,
    };

    if stdout.contains("(error") {
        return VerifyResult::Error {
            message: format!("SMT encoding error:\n{stdout}"),
        };
    }

    let first_line = stdout.lines().next().unwrap_or("").trim();

    match first_line {
        "unsat" => VerifyResult::Verified,
        "sat" => {
            // Re-run with (get-model) to extract counterexample
            let with_model = format!("{smt_input}\n(get-model)\n");
            let model_stdout = match spawn_z3(&with_model) {
                Ok(s) => s,
                Err(_) => {
                    return VerifyResult::Failed {
                        counterexample: String::new(),
                    }
                }
            };
            let raw_model = model_stdout.lines().skip(1).collect::<Vec<_>>().join("\n");
            let counterexample = parse_z3_model(&raw_model);
            VerifyResult::Failed { counterexample }
        }
        "unknown" => VerifyResult::Unknown {
            reason: stdout.to_string(),
        },
        _ => VerifyResult::Error {
            message: format!("unexpected z3 output: {stdout}"),
        },
    }
}

/// Parse Z3 model output into human-readable assignments.
/// Input looks like: (model (define-fun sender_balance () Int 100) ...)
fn parse_z3_model(raw: &str) -> String {
    let mut assignments = Vec::new();
    // Match patterns like (define-fun <name> () <sort> <value>)
    let mut remaining = raw;
    while let Some(pos) = remaining.find("define-fun ") {
        remaining = &remaining[pos + 11..];
        // Parse name
        let name_end = remaining
            .find(|c: char| c.is_whitespace())
            .unwrap_or(remaining.len());
        let name = &remaining[..name_end];
        remaining = &remaining[name_end..];

        // Skip past "() <sort> " to get value
        if let Some(paren_pos) = remaining.find("()") {
            remaining = &remaining[paren_pos + 2..].trim_start();
            // Skip sort
            let sort_end = remaining
                .find(|c: char| c.is_whitespace())
                .unwrap_or(remaining.len());
            remaining = &remaining[sort_end..].trim_start();
            // Read value until closing paren
            let value = extract_smt_value(remaining);

            // Convert flattened name back: sender_balance → sender.balance
            let display_name = name.replace('_', ".");
            // Skip primed versions — only show original values
            if !name.ends_with("_prime") {
                assignments.push(format!("{display_name} = {value}"));
            }
        }
    }

    if assignments.is_empty() {
        raw.to_string()
    } else {
        assignments.join(", ")
    }
}

/// Extract a value from SMT-LIB2 output (handles negative numbers like (- 1))
fn extract_smt_value(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('(') {
        // Could be (- N) for negative number or complex expression
        if let Some(close) = find_matching_paren(s) {
            let inner = &s[1..close].trim();
            if inner.starts_with("- ") {
                return format!("-{}", inner[2..].trim());
            }
            return inner.to_string();
        }
        // Fallback
        s.chars().take(30).collect()
    } else {
        // Simple value: number, true, false, string literal
        s.split(|c: char| c == ')' || c == '\n')
            .next()
            .unwrap_or(s)
            .trim()
            .to_string()
    }
}

fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Verify a single VC: encode → call Z3 → return result.
pub fn verify_vc(vc: &VerificationCondition, prog: &Program) -> VerifyResult {
    // Intents with no goals are trivially verified (nothing to prove).
    if vc.kind == VcKind::Intent && vc.goals.is_empty() {
        return VerifyResult::Verified;
    }

    let mut encoder = SmtEncoder::new(prog);
    encoder.encode_vc(vc, prog);
    let smt = encoder.get_output();
    run_z3(&smt)
}
