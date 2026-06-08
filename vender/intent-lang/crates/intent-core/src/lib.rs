pub mod analysis;
pub mod smt;
pub mod typeck;
pub mod vcgen;

use intent_syntax::ast::Span;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub code: String,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.level {
            DiagLevel::Error => "error",
            DiagLevel::Warning => "warning",
            DiagLevel::Info => "info",
        };
        write!(f, "{}[{}]: {}", level, self.code, self.message)?;
        for note in &self.notes {
            write!(f, "\n  = {note}")?;
        }
        Ok(())
    }
}
