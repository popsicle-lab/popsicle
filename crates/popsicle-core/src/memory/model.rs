use std::fmt;
use std::str::FromStr;

use serde::Serialize;

/// Type of memory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MemoryType {
    Bug,
    Decision,
    Pattern,
    Gotcha,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bug => write!(f, "BUG"),
            Self::Decision => write!(f, "DECISION"),
            Self::Pattern => write!(f, "PATTERN"),
            Self::Gotcha => write!(f, "GOTCHA"),
        }
    }
}

impl FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BUG" => Ok(Self::Bug),
            "DECISION" => Ok(Self::Decision),
            "PATTERN" => Ok(Self::Pattern),
            "GOTCHA" => Ok(Self::Gotcha),
            other => Err(format!("unknown memory type: {other}")),
        }
    }
}

/// Memory layer — short-term (unverified) or long-term (validated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MemoryLayer {
    ShortTerm,
    LongTerm,
}

impl fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShortTerm => write!(f, "short-term"),
            Self::LongTerm => write!(f, "long-term"),
        }
    }
}

impl FromStr for MemoryLayer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "short-term" | "shortterm" | "short" => Ok(Self::ShortTerm),
            "long-term" | "longterm" | "long" => Ok(Self::LongTerm),
            other => Err(format!("unknown memory layer: {other}")),
        }
    }
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize)]
pub struct Memory {
    pub id: u32,
    pub memory_type: MemoryType,
    pub summary: String,
    /// Creation date in "YYYY-MM-DD" format.
    pub created: String,
    pub layer: MemoryLayer,
    /// Number of times this memory was injected and referenced by an Agent.
    pub refs: u32,
    pub tags: Vec<String>,
    /// Related source files (relative paths).
    pub files: Vec<String>,
    /// Associated pipeline run ID.
    pub run: Option<String>,
    /// Whether this memory is considered potentially outdated.
    pub stale: bool,
    /// 1-5 line natural-language description.
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_type_display_roundtrip() {
        for ty in [
            MemoryType::Bug,
            MemoryType::Decision,
            MemoryType::Pattern,
            MemoryType::Gotcha,
        ] {
            let s = ty.to_string();
            let parsed: MemoryType = s.parse().unwrap();
            assert_eq!(parsed, ty);
        }
    }

    #[test]
    fn memory_type_case_insensitive() {
        assert_eq!("bug".parse::<MemoryType>().unwrap(), MemoryType::Bug);
        assert_eq!("Bug".parse::<MemoryType>().unwrap(), MemoryType::Bug);
        assert_eq!(
            "decision".parse::<MemoryType>().unwrap(),
            MemoryType::Decision
        );
    }

    #[test]
    fn memory_type_invalid() {
        assert!("unknown".parse::<MemoryType>().is_err());
    }

    #[test]
    fn memory_layer_display_roundtrip() {
        for layer in [MemoryLayer::ShortTerm, MemoryLayer::LongTerm] {
            let s = layer.to_string();
            let parsed: MemoryLayer = s.parse().unwrap();
            assert_eq!(parsed, layer);
        }
    }

    #[test]
    fn memory_layer_aliases() {
        assert_eq!(
            "short".parse::<MemoryLayer>().unwrap(),
            MemoryLayer::ShortTerm
        );
        assert_eq!(
            "long".parse::<MemoryLayer>().unwrap(),
            MemoryLayer::LongTerm
        );
        assert_eq!(
            "short_term".parse::<MemoryLayer>().unwrap(),
            MemoryLayer::ShortTerm
        );
    }
}
