use logos::Logos;

use crate::ast::*;
use crate::lexer::Token;

// ── Error type ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parse error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// ── Parser state ─────────────────────────────────────────────

pub struct Parser<'src> {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    source: &'src str,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Result<Self, ParseError> {
        let mut tokens = Vec::new();
        let mut lex = Token::lexer(source);
        while let Some(tok) = lex.next() {
            let span = Span::new(lex.span().start, lex.span().end);
            match tok {
                Ok(t) => tokens.push((t, span)),
                Err(()) => {
                    return Err(ParseError {
                        message: format!(
                            "unexpected character: {:?}",
                            &source[lex.span().start..lex.span().end]
                        ),
                        span,
                    });
                }
            }
        }
        Ok(Parser {
            tokens,
            pos: 0,
            source,
        })
    }

    // ── Helpers ──────────────────────────────────────────

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| s.clone())
            .unwrap_or_else(|| Span::new(self.source.len(), self.source.len()))
    }

    fn advance(&mut self) -> (Token, Span) {
        let item = self.tokens[self.pos].clone();
        self.pos += 1;
        item
    }

    fn expect(&mut self, expected: &Token) -> Result<Span, ParseError> {
        if self.peek() == Some(expected) {
            let (_, span) = self.advance();
            Ok(span)
        } else {
            Err(self.error(format!("expected `{expected}`, found {}", self.found())))
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ParseError> {
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                let (_, span) = self.advance();
                Ok((name, span))
            }
            _ => Err(self.error(format!("expected identifier, found {}", self.found()))),
        }
    }

    fn found(&self) -> String {
        match self.peek() {
            Some(t) => format!("`{t}`"),
            None => "end of file".to_string(),
        }
    }

    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.peek_span(),
        }
    }

    fn at(&self, tok: &Token) -> bool {
        self.peek() == Some(tok)
    }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.at(tok) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ── Program ──────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();
        while self.peek().is_some() {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program { declarations })
    }

    fn parse_declaration(&mut self) -> Result<Spanned<Declaration>, ParseError> {
        // Collect annotations
        let mut annotations = Vec::new();
        while self.at(&Token::At) {
            annotations.push(self.parse_annotation()?);
        }

        let start_span = self.peek_span();
        match self.peek() {
            Some(Token::Import) => {
                let decl = self.parse_import()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Import(decl), span))
            }
            Some(Token::Type) => {
                let decl = self.parse_type_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Type(decl), span))
            }
            Some(Token::Enum) => {
                let decl = self.parse_enum_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Enum(decl), span))
            }
            Some(Token::Function) => {
                let decl = self.parse_function_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Function(decl), span))
            }
            Some(Token::Intent) => {
                let decl = self.parse_intent_decl(annotations)?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Intent(decl), span))
            }
            Some(Token::Safety) => {
                let decl = self.parse_safety_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Safety(decl), span))
            }
            Some(Token::Theorem) => {
                let decl = self.parse_theorem_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Theorem(decl), span))
            }
            Some(Token::Axiom) => {
                let decl = self.parse_axiom_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Axiom(decl), span))
            }
            Some(Token::Goal) => {
                let decl = self.parse_goal_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Goal(decl), span))
            }
            Some(Token::Coverage) => {
                let decl = self.parse_coverage_decl()?;
                let span = start_span.merge(&self.prev_span());
                Ok(Spanned::new(Declaration::Coverage(decl), span))
            }
            _ => Err(self.error(format!("expected declaration, found {}", self.found()))),
        }
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].1.clone()
        } else {
            Span::new(0, 0)
        }
    }

    // ── Import ───────────────────────────────────────────

    fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        self.expect(&Token::Import)?;
        let (path, _default_alias) = match self.peek() {
            // File import: `import "./path/to/file.intent"`
            Some(Token::StringLit(_)) => {
                let (tok, _) = self.advance();
                let s = match tok {
                    Token::StringLit(s) => s,
                    _ => unreachable!(),
                };
                // Default alias is the file stem (filename without .intent)
                let default = std::path::Path::new(&s)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                (ImportPath::File(s), default)
            }
            // Plugin import: `import smarthome` or `import finance.currency`
            _ => {
                let (first, _) = self.expect_ident()?;
                let mut segments = vec![first];
                while self.eat(&Token::Dot) {
                    let (seg, _) = self.expect_ident()?;
                    segments.push(seg);
                }
                let default = segments.last().cloned().unwrap_or_default();
                (ImportPath::Plugin(segments), default)
            }
        };
        // Optional alias: `as name`
        let alias = if self.eat(&Token::As) {
            let (name, _) = self.expect_ident()?;
            Some(name)
        } else {
            None
        };
        Ok(ImportDecl { path, alias })
    }

    // ── Type ─────────────────────────────────────────────

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        self.expect(&Token::Type)?;
        let (name, _) = self.expect_ident()?;

        let type_params = if self.eat(&Token::Lt) {
            let mut params = Vec::new();
            loop {
                let (p, _) = self.expect_ident()?;
                params.push(p);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::Gt)?;
            params
        } else {
            Vec::new()
        };

        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(&Token::RBrace) {
            let (fname, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type_expr()?;
            fields.push(Field { name: fname, ty });
        }
        self.expect(&Token::RBrace)?;

        Ok(TypeDecl {
            name,
            type_params,
            fields,
        })
    }

    // ── Enum ─────────────────────────────────────────────

    fn parse_enum_decl(&mut self) -> Result<EnumDecl, ParseError> {
        self.expect(&Token::Enum)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut variants = Vec::new();
        loop {
            let (v, _) = self.expect_ident()?;
            variants.push(v);
            if !self.eat(&Token::Comma) {
                break;
            }
            // Allow trailing comma
            if self.at(&Token::RBrace) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;

        Ok(EnumDecl { name, variants })
    }

    // ── Function ─────────────────────────────────────────

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        self.expect(&Token::Function)?;
        let (name, _) = self.expect_ident()?;
        let params = self.parse_param_list()?;
        self.expect(&Token::Arrow)?;
        let return_type = self.parse_type_expr()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(&Token::RBrace)?;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    // ── Intent ───────────────────────────────────────────

    fn parse_intent_decl(
        &mut self,
        annotations: Vec<Annotation>,
    ) -> Result<IntentDecl, ParseError> {
        self.expect(&Token::Intent)?;
        let (name, _) = self.expect_ident()?;
        let params = self.parse_param_list()?;
        self.expect(&Token::LBrace)?;

        let mut clauses = Vec::new();
        while !self.at(&Token::RBrace) {
            let clause_start = self.peek_span();
            let clause = match self.peek() {
                Some(Token::Require) => {
                    self.advance();
                    let e = self.parse_expr()?;
                    Clause::Require(e)
                }
                Some(Token::Ensure) => {
                    self.advance();
                    let e = self.parse_expr()?;
                    Clause::Ensure(e)
                }
                Some(Token::Invariant) => {
                    self.advance();
                    let e = self.parse_expr()?;
                    Clause::Invariant(e)
                }
                _ => {
                    return Err(self.error(format!(
                        "expected require/ensure/invariant, found {}",
                        self.found()
                    )));
                }
            };
            let span = clause_start.merge(&self.prev_span());
            clauses.push(Spanned::new(clause, span));
        }
        self.expect(&Token::RBrace)?;

        Ok(IntentDecl {
            name,
            annotations,
            params,
            clauses,
        })
    }

    // ── Safety ───────────────────────────────────────────

    fn parse_safety_decl(&mut self) -> Result<SafetyDecl, ParseError> {
        self.expect(&Token::Safety)?;
        let (name, _) = self.expect_ident()?;
        let params = self.parse_param_list()?;
        self.expect(&Token::LBrace)?;

        let mut invariants = Vec::new();
        while !self.at(&Token::RBrace) {
            self.expect(&Token::Invariant)?;
            let e = self.parse_expr()?;
            invariants.push(e);
        }
        self.expect(&Token::RBrace)?;

        Ok(SafetyDecl {
            name,
            params,
            invariants,
        })
    }

    // ── Theorem / Axiom ──────────────────────────────────

    fn parse_theorem_decl(&mut self) -> Result<TheoremDecl, ParseError> {
        self.expect(&Token::Theorem)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(TheoremDecl { name, body })
    }

    fn parse_axiom_decl(&mut self) -> Result<AxiomDecl, ParseError> {
        self.expect(&Token::Axiom)?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(AxiomDecl { name, body })
    }

    // ── Goal (RFC A1) ────────────────────────────────────

    fn parse_goal_decl(&mut self) -> Result<GoalDecl, ParseError> {
        self.expect(&Token::Goal)?;
        // Name must be a string literal: `goal "User balance never negative" { ... }`
        let name = match self.peek().cloned() {
            Some(Token::StringLit(s)) => {
                self.advance();
                s
            }
            _ => {
                return Err(self.error(format!(
                    "expected string literal goal name, found {}",
                    self.found()
                )));
            }
        };
        self.expect(&Token::LBrace)?;

        let mut rationale = None;
        let mut stakeholder = Vec::new();
        let mut measure = None;
        let mut realized_by = Vec::new();

        while !self.at(&Token::RBrace) {
            match self.peek() {
                Some(Token::Rationale) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    rationale = Some(self.expect_string_lit()?);
                }
                Some(Token::Stakeholder) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    // Accept either single string or [a, b, c]
                    if self.eat(&Token::LBracket) {
                        if !self.at(&Token::RBracket) {
                            loop {
                                stakeholder.push(self.expect_string_lit()?);
                                if !self.eat(&Token::Comma) {
                                    break;
                                }
                            }
                        }
                        self.expect(&Token::RBracket)?;
                    } else {
                        // Comma-separated string list inside one string
                        let s = self.expect_string_lit()?;
                        for part in s.split(',') {
                            stakeholder.push(part.trim().to_string());
                        }
                    }
                }
                Some(Token::Measure) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    measure = Some(self.expect_string_lit()?);
                }
                Some(Token::RealizedBy) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    self.expect(&Token::LBracket)?;
                    if !self.at(&Token::RBracket) {
                        loop {
                            let (id, _) = self.expect_ident()?;
                            realized_by.push(id);
                            if !self.eat(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::RBracket)?;
                }
                _ => {
                    return Err(self.error(format!(
                        "expected rationale/stakeholder/measure/realized_by, found {}",
                        self.found()
                    )));
                }
            }
            // Optional semicolon-like: comma between fields is allowed
            self.eat(&Token::Comma);
        }
        self.expect(&Token::RBrace)?;

        Ok(GoalDecl {
            name,
            rationale,
            stakeholder,
            measure,
            realized_by,
        })
    }

    fn expect_string_lit(&mut self) -> Result<String, ParseError> {
        match self.peek().cloned() {
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(s)
            }
            _ => Err(self.error(format!("expected string literal, found {}", self.found()))),
        }
    }

    // ── Coverage (RFC A3) ────────────────────────────────

    fn parse_coverage_decl(&mut self) -> Result<CoverageDecl, ParseError> {
        self.expect(&Token::Coverage)?;
        let name = match self.peek().cloned() {
            Some(Token::StringLit(s)) => {
                self.advance();
                s
            }
            _ => {
                return Err(self.error(format!(
                    "expected string literal coverage name, found {}",
                    self.found()
                )));
            }
        };
        self.expect(&Token::LBrace)?;

        let mut dimensions = Vec::new();

        while !self.at(&Token::RBrace) {
            match self.peek() {
                Some(Token::Dimensions) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    self.expect(&Token::LBrace)?;
                    while !self.at(&Token::RBrace) {
                        let (dname, _) = self.expect_ident()?;
                        self.expect(&Token::Colon)?;
                        self.expect(&Token::LBracket)?;
                        let mut values = Vec::new();
                        if !self.at(&Token::RBracket) {
                            loop {
                                values.push(self.parse_expr()?);
                                if !self.eat(&Token::Comma) {
                                    break;
                                }
                            }
                        }
                        self.expect(&Token::RBracket)?;
                        self.eat(&Token::Comma);
                        dimensions.push(CoverageDim {
                            name: dname,
                            values,
                        });
                    }
                    self.expect(&Token::RBrace)?;
                }
                _ => {
                    return Err(
                        self.error(format!("expected `dimensions`, found {}", self.found()))
                    );
                }
            }
            self.eat(&Token::Comma);
        }
        self.expect(&Token::RBrace)?;

        Ok(CoverageDecl { name, dimensions })
    }

    // ── Annotations ──────────────────────────────────────

    fn parse_annotation(&mut self) -> Result<Annotation, ParseError> {
        self.expect(&Token::At)?;
        let (name, _) = self.expect_ident()?;
        let mut args = Vec::new();

        if self.eat(&Token::LParen) {
            if !self.at(&Token::RParen) {
                loop {
                    // Check for named arg: `key: value`
                    let saved = self.pos;
                    if let Some(Token::Ident(key)) = self.peek().cloned() {
                        self.advance();
                        if self.eat(&Token::Colon) {
                            let val = self.parse_expr()?;
                            args.push(AnnotationArg::Named(key, val));
                        } else {
                            self.pos = saved;
                            let val = self.parse_expr()?;
                            args.push(AnnotationArg::Positional(val));
                        }
                    } else {
                        let val = self.parse_expr()?;
                        args.push(AnnotationArg::Positional(val));
                    }
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Token::RParen)?;
        }

        Ok(Annotation { name, args })
    }

    // ── Param list ───────────────────────────────────────

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        if !self.at(&Token::RParen) {
            loop {
                let (name, _) = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let ty = self.parse_type_expr()?;
                params.push(Param { name, ty });
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::RParen)?;
        Ok(params)
    }

    // ── Type expressions ─────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let (name, _) = self.expect_ident()?;
        // Check for qualified type: `module.TypeName`
        if self.at(&Token::Dot) {
            // Peek ahead: if next is Dot then Ident (then NOT Dot again), it's qualified
            // But we also need to distinguish from field access in expressions.
            // In type position, `ident.Ident` is always qualified.
            let saved = self.pos;
            self.advance(); // eat dot
            match self.peek() {
                Some(Token::Ident(type_name)) => {
                    let type_name = type_name.clone();
                    // Check if the type_name starts with uppercase (convention)
                    // or if followed by `<` (generic). In type position after dot,
                    // this is always a qualified reference.
                    self.advance();
                    if self.eat(&Token::Lt) {
                        let mut args = Vec::new();
                        loop {
                            args.push(self.parse_type_expr()?);
                            if !self.eat(&Token::Comma) {
                                break;
                            }
                        }
                        self.expect(&Token::Gt)?;
                        // Qualified generic not yet supported — treat as Generic with qualified base
                        // For now, just use the type_name as generic
                        Ok(TypeExpr::Generic(format!("{name}.{type_name}"), args))
                    } else {
                        Ok(TypeExpr::Qualified(name, type_name))
                    }
                }
                _ => {
                    // Not a qualified type, backtrack
                    self.pos = saved;
                    Ok(TypeExpr::Named(name))
                }
            }
        } else if self.eat(&Token::Lt) {
            let mut args = Vec::new();
            loop {
                args.push(self.parse_type_expr()?);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::Gt)?;
            Ok(TypeExpr::Generic(name, args))
        } else {
            Ok(TypeExpr::Named(name))
        }
    }

    // ── Expression parsing (Pratt) ───────────────────────

    pub fn parse_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        self.parse_pratt(0)
    }

    /// Pratt parser: parse expression with minimum binding power.
    fn parse_pratt(&mut self, min_bp: u8) -> Result<Spanned<Expr>, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix: `.field`, `'`, `[index]`
            lhs = match self.peek() {
                Some(Token::Dot) => {
                    self.advance();
                    let (field, field_span) = self.expect_ident()?;
                    // Check for field prime: `expr.field'`
                    if self.eat(&Token::Prime) {
                        let span = lhs.span.merge(&self.prev_span());
                        let field_access =
                            Spanned::new(Expr::FieldAccess(Box::new(lhs), field), span.clone());
                        Spanned::new(Expr::Prime(Box::new(field_access)), span)
                    } else {
                        let span = lhs.span.merge(&field_span);
                        Spanned::new(Expr::FieldAccess(Box::new(lhs), field), span)
                    }
                }
                Some(Token::Prime) => {
                    self.advance();
                    let span = lhs.span.merge(&self.prev_span());
                    Spanned::new(Expr::Prime(Box::new(lhs)), span)
                }
                Some(Token::LBracket) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let end = self.expect(&Token::RBracket)?;
                    let span = lhs.span.merge(&end);
                    Spanned::new(Expr::Index(Box::new(lhs), Box::new(index)), span)
                }
                _ => break,
            };
        }

        // Infix binary operators
        loop {
            let op = match self.peek() {
                Some(Token::Implies) => BinOp::Implies,
                Some(Token::PipePipe) => BinOp::Or,
                Some(Token::AmpAmp) => BinOp::And,
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::BangEq) => BinOp::Neq,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::LtEq) => BinOp::Le,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::GtEq) => BinOp::Ge,
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Mod,
                _ => break,
            };

            let (l_bp, r_bp) = infix_binding_power(op);
            if l_bp < min_bp {
                break;
            }

            self.advance(); // consume operator
            let rhs = self.parse_pratt(r_bp)?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(Expr::BinOp(Box::new(lhs), op, Box::new(rhs)), span);
        }

        Ok(lhs)
    }

    /// Parse prefix / atom expressions.
    fn parse_prefix(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        match self.peek().cloned() {
            // Unary !
            Some(Token::Bang) => {
                self.advance();
                let operand = self.parse_pratt(PREFIX_BP)?;
                let span = start.merge(&operand.span);
                Ok(Spanned::new(
                    Expr::UnaryOp(UnaryOp::Not, Box::new(operand)),
                    span,
                ))
            }
            // Unary -
            Some(Token::Minus) => {
                self.advance();
                let operand = self.parse_pratt(PREFIX_BP)?;
                let span = start.merge(&operand.span);
                Ok(Spanned::new(
                    Expr::UnaryOp(UnaryOp::Neg, Box::new(operand)),
                    span,
                ))
            }
            // Parenthesized
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                let end = self.expect(&Token::RParen)?;
                let span = start.merge(&end);
                Ok(Spanned::new(Expr::Paren(Box::new(inner)), span))
            }
            // If-then-else
            Some(Token::If) => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(&Token::Then)?;
                let then_expr = self.parse_expr()?;
                self.expect(&Token::Else)?;
                let else_expr = self.parse_expr()?;
                let span = start.merge(&else_expr.span);
                Ok(Spanned::new(
                    Expr::IfThenElse(Box::new(cond), Box::new(then_expr), Box::new(else_expr)),
                    span,
                ))
            }
            // Forall / Exists
            Some(Token::Forall) => {
                self.advance();
                let vars = self.parse_typed_vars()?;
                self.expect(&Token::Comma)?;
                let body = self.parse_expr()?;
                let span = start.merge(&body.span);
                Ok(Spanned::new(Expr::Forall(vars, Box::new(body)), span))
            }
            Some(Token::Exists) => {
                self.advance();
                let vars = self.parse_typed_vars()?;
                self.expect(&Token::Comma)?;
                let body = self.parse_expr()?;
                let span = start.merge(&body.span);
                Ok(Spanned::new(Expr::Exists(vars, Box::new(body)), span))
            }
            // `after(expr)` — desugar to Prime
            Some(Token::After) => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_expr()?;
                let end = self.expect(&Token::RParen)?;
                let span = start.merge(&end);
                Ok(Spanned::new(Expr::Prime(Box::new(inner)), span))
            }
            // Integer literal
            Some(Token::IntLit(v)) => {
                self.advance();
                Ok(Spanned::new(Expr::IntLit(v), start))
            }
            // Boolean
            Some(Token::True) => {
                self.advance();
                Ok(Spanned::new(Expr::BoolLit(true), start))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Spanned::new(Expr::BoolLit(false), start))
            }
            // String
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(Spanned::new(Expr::StringLit(s), start))
            }
            // Identifier or function call
            Some(Token::Ident(name)) => {
                self.advance();
                if self.at(&Token::LParen) {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    if !self.at(&Token::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.eat(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    let end = self.expect(&Token::RParen)?;
                    let span = start.merge(&end);
                    Ok(Spanned::new(Expr::Call(name, args), span))
                } else {
                    Ok(Spanned::new(Expr::Ident(name), start))
                }
            }
            _ => Err(self.error(format!("expected expression, found {}", self.found()))),
        }
    }

    /// Parse typed variable list for quantifiers: `x: T, y: U`
    /// Stops when next comma is not followed by `ident : type` pattern.
    fn parse_typed_vars(&mut self) -> Result<Vec<TypedVar>, ParseError> {
        let mut vars = Vec::new();
        loop {
            let (name, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type_expr()?;
            vars.push(TypedVar { name, ty });
            // Peek ahead: if next is comma then ident then colon, more typed vars
            if self.at(&Token::Comma) && self.is_typed_var_ahead() {
                self.advance(); // eat comma
            } else {
                break;
            }
        }
        Ok(vars)
    }

    /// Look ahead to check if comma is followed by `ident : type` (another typed var).
    fn is_typed_var_ahead(&self) -> bool {
        // We're at a comma. Check pos+1 is ident and pos+2 is colon.
        let after_comma = self.pos + 1;
        if after_comma + 1 >= self.tokens.len() {
            return false;
        }
        matches!(self.tokens[after_comma].0, Token::Ident(_))
            && matches!(self.tokens[after_comma + 1].0, Token::Colon)
    }
}

// ── Binding power tables ─────────────────────────────────────

const PREFIX_BP: u8 = 15;

fn infix_binding_power(op: BinOp) -> (u8, u8) {
    match op {
        // ==> is right-associative: (l=2, r=1)
        BinOp::Implies => (2, 1),
        BinOp::Or => (3, 4),
        BinOp::And => (5, 6),
        BinOp::Eq | BinOp::Neq => (7, 8),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => (9, 10),
        BinOp::Add | BinOp::Sub => (11, 12),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (13, 14),
    }
}

// ── Public convenience ───────────────────────────────────────

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_type_decl() {
        let src = r#"type Account { balance: Int  owner: String  active: Bool }"#;
        let prog = parse(src).unwrap();
        assert_eq!(prog.declarations.len(), 1);
        match &prog.declarations[0].node {
            Declaration::Type(t) => {
                assert_eq!(t.name, "Account");
                assert_eq!(t.fields.len(), 3);
            }
            _ => panic!("expected type decl"),
        }
    }

    #[test]
    fn parse_enum_decl() {
        let src = "enum Role { Admin, Editor, Viewer }";
        let prog = parse(src).unwrap();
        assert_eq!(prog.declarations.len(), 1);
        match &prog.declarations[0].node {
            Declaration::Enum(e) => {
                assert_eq!(e.name, "Role");
                assert_eq!(e.variants, vec!["Admin", "Editor", "Viewer"]);
            }
            _ => panic!("expected enum decl"),
        }
    }

    #[test]
    fn parse_intent_basic() {
        let src = r#"
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0
  require sender.balance >= amount
  ensure sender.balance' == sender.balance - amount
  ensure receiver.balance' == receiver.balance + amount
  invariant sender.balance' >= 0
}
"#;
        let prog = parse(src).unwrap();
        assert_eq!(prog.declarations.len(), 1);
        match &prog.declarations[0].node {
            Declaration::Intent(i) => {
                assert_eq!(i.name, "TransferSafe");
                assert_eq!(i.params.len(), 3);
                assert_eq!(i.clauses.len(), 5);
            }
            _ => panic!("expected intent decl"),
        }
    }

    #[test]
    fn parse_theorem_with_forall() {
        let src = r#"
theorem TransferPreservesTotal {
  forall s: Account, r: Account, a: Int,
    TransferSafe(s, r, a) ==>
      s.balance' + r.balance' == s.balance + r.balance
}
"#;
        let prog = parse(src).unwrap();
        assert_eq!(prog.declarations.len(), 1);
        match &prog.declarations[0].node {
            Declaration::Theorem(t) => {
                assert_eq!(t.name, "TransferPreservesTotal");
            }
            _ => panic!("expected theorem decl"),
        }
    }

    #[test]
    fn parse_function_decl() {
        let src = "function max(a: Int, b: Int) -> Int { if a >= b then a else b }";
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Function(f) => {
                assert_eq!(f.name, "max");
                assert_eq!(f.params.len(), 2);
            }
            _ => panic!("expected function decl"),
        }
    }

    #[test]
    fn parse_safety_decl() {
        let src = r#"
safety HomeSafety(home: Home) {
  invariant !home.occupied ==> home.frontDoor.locked
}
"#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Safety(s) => {
                assert_eq!(s.name, "HomeSafety");
                assert_eq!(s.invariants.len(), 1);
            }
            _ => panic!("expected safety decl"),
        }
    }

    #[test]
    fn parse_after_desugar() {
        let src = r#"
intent T(x: Account) {
  ensure after(x.balance) == x.balance - 1
}
"#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Intent(i) => {
                // The ensure clause should contain a Prime node
                match &i.clauses[0].node {
                    Clause::Ensure(e) => match &e.node {
                        Expr::BinOp(lhs, BinOp::Eq, _) => match &lhs.node {
                            Expr::Prime(_) => {} // correct
                            other => panic!("expected Prime, got {other:?}"),
                        },
                        other => panic!("expected BinOp, got {other:?}"),
                    },
                    _ => panic!("expected ensure"),
                }
            }
            _ => panic!("expected intent decl"),
        }
    }

    #[test]
    fn parse_import() {
        let src = "import smarthome";
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Import(i) => {
                assert!(matches!(&i.path, ImportPath::Plugin(p) if p == &["smarthome"]));
                assert_eq!(i.alias, None);
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_dotted() {
        let src = "import finance.currency";
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Import(i) => {
                assert!(matches!(&i.path, ImportPath::Plugin(p) if p == &["finance", "currency"]));
                assert_eq!(i.alias, None);
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_file() {
        let src = r#"import "./domains/payment/types.intent""#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Import(i) => {
                assert!(
                    matches!(&i.path, ImportPath::File(p) if p == "./domains/payment/types.intent")
                );
                assert_eq!(i.alias, None);
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_file_with_alias() {
        let src = r#"import "./domains/payment/types.intent" as payment"#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Import(i) => {
                assert!(
                    matches!(&i.path, ImportPath::File(p) if p == "./domains/payment/types.intent")
                );
                assert_eq!(i.alias, Some("payment".to_string()));
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_plugin_with_alias() {
        let src = "import finance.currency as money";
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Import(i) => {
                assert!(matches!(&i.path, ImportPath::Plugin(p) if p == &["finance", "currency"]));
                assert_eq!(i.alias, Some("money".to_string()));
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_qualified_type() {
        let src = r#"
intent Checkout(wallet: payment.Account, profile: user.Account) {
  require wallet.balance >= 0
}
"#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Intent(i) => {
                assert_eq!(
                    i.params[0].ty,
                    TypeExpr::Qualified("payment".into(), "Account".into())
                );
                assert_eq!(
                    i.params[1].ty,
                    TypeExpr::Qualified("user".into(), "Account".into())
                );
            }
            _ => panic!("expected intent"),
        }
    }

    #[test]
    fn parse_exists_expr() {
        let src = r#"
intent Arrive(home: Home) {
  ensure exists l: Light, l.room == "living" && l.on' == true
}
"#;
        let prog = parse(src).unwrap();
        match &prog.declarations[0].node {
            Declaration::Intent(i) => {
                assert_eq!(i.clauses.len(), 1);
            }
            _ => panic!("expected intent"),
        }
    }
}
