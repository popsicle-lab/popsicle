use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type of package in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Module,
    Tool,
}

impl std::fmt::Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module => write!(f, "module"),
            Self::Tool => write!(f, "tool"),
        }
    }
}

/// A single published version of a package.
///
/// In the git-based index each line of a package file is one `PackageVersion`
/// serialized as JSON (NDJSON format, like the crates.io index).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    /// Package name.
    pub name: String,
    /// Semver version string (e.g. "1.2.0").
    pub vers: String,
    /// Module or Tool.
    #[serde(rename = "type")]
    pub pkg_type: PackageType,
    /// One-line description.
    #[serde(default)]
    pub description: Option<String>,
    /// Author or organization.
    #[serde(default)]
    pub author: Option<String>,
    /// Source repository URL (for linking, not for install).
    #[serde(default)]
    pub repository: Option<String>,
    /// Install source reference (e.g. "github:org/repo#v1.0.0").
    pub source: String,

    // ── Contents summary (helps search / display without cloning) ──────
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub pipelines: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,

    // ── Dependencies ───────────────────────────────────────────────────
    #[serde(default)]
    pub deps: Vec<PackageDep>,

    // ── Discovery ──────────────────────────────────────────────────────
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Whether this version has been yanked (hidden from default search).
    #[serde(default)]
    pub yanked: bool,

    /// ISO-8601 publication timestamp.
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
}

/// A dependency on another registry package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDep {
    /// Dependency package name.
    pub name: String,
    /// Version requirement (e.g. ">=1.0.0", "^1.0").
    #[serde(default)]
    pub req: Option<String>,
    /// Dependency type — "module" or "tool".
    #[serde(default = "default_dep_kind")]
    pub kind: PackageType,
}

fn default_dep_kind() -> PackageType {
    PackageType::Tool
}

/// Aggregated view of a package across all published versions.
///
/// Constructed by reading all NDJSON lines from a package's index file.
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub pkg_type: PackageType,
    pub versions: Vec<PackageVersion>,
}

impl PackageEntry {
    /// Build from a list of versions (assumes all have the same name/type).
    pub fn from_versions(versions: Vec<PackageVersion>) -> Option<Self> {
        let first = versions.first()?;
        Some(Self {
            name: first.name.clone(),
            pkg_type: first.pkg_type,
            versions,
        })
    }

    /// The latest non-yanked version, or the latest yanked if all are yanked.
    pub fn latest(&self) -> Option<&PackageVersion> {
        self.versions
            .iter()
            .rev()
            .find(|v| !v.yanked)
            .or_else(|| self.versions.last())
    }

    /// Find a specific version string.
    pub fn version(&self, vers: &str) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.vers == vers)
    }
}

/// Registry-level configuration stored in `config.json` at the index root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Schema version (currently 1).
    #[serde(default = "default_config_version")]
    pub version: u32,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// URL of the index repository itself.
    #[serde(default)]
    pub registry_url: Option<String>,
    /// URL of the companion website (if any).
    #[serde(default)]
    pub website_url: Option<String>,
}

fn default_config_version() -> u32 {
    1
}

// ── Index path helpers (crates.io-style prefix directories) ─────────────

/// Compute the prefix directory path for a package name.
///
/// Naming scheme (matches crates.io):
/// - 1-char names  → `1/<name>`
/// - 2-char names  → `2/<name>`
/// - 3-char names  → `3/<first-char>/<name>`
/// - 4+ char names → `<first-2>/<next-2>/<name>`
pub fn index_path(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.len() {
        0 => panic!("empty package name"),
        1 => format!("1/{}", lower),
        2 => format!("2/{}", lower),
        3 => format!("3/{}/{}", &lower[..1], lower),
        _ => format!("{}/{}/{}", &lower[..2], &lower[2..4], lower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_path_short_names() {
        assert_eq!(index_path("a"), "1/a");
        assert_eq!(index_path("ab"), "2/ab");
        assert_eq!(index_path("abc"), "3/a/abc");
    }

    #[test]
    fn test_index_path_long_names() {
        assert_eq!(index_path("draw-diagram"), "dr/aw/draw-diagram");
        assert_eq!(index_path("spec-development"), "sp/ec/spec-development");
        assert_eq!(index_path("test"), "te/st/test");
    }

    #[test]
    fn test_index_path_case_insensitive() {
        assert_eq!(index_path("DrawDiagram"), "dr/aw/drawdiagram");
    }

    #[test]
    fn test_package_version_roundtrip() {
        let json = r#"{"name":"spec-dev","vers":"1.0.0","type":"module","source":"github:org/repo#v1.0.0","skills":["prd"],"pipelines":["full"],"tools":[],"deps":[],"keywords":["spec"],"yanked":false}"#;
        let v: PackageVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.name, "spec-dev");
        assert_eq!(v.vers, "1.0.0");
        assert_eq!(v.pkg_type, PackageType::Module);
        assert_eq!(v.skills, vec!["prd"]);
    }

    #[test]
    fn test_package_entry_latest() {
        let v1 = PackageVersion {
            name: "test".into(),
            vers: "0.1.0".into(),
            pkg_type: PackageType::Tool,
            description: None,
            author: None,
            repository: None,
            source: "github:x/y#v0.1.0".into(),
            skills: vec![],
            pipelines: vec![],
            tools: vec![],
            deps: vec![],
            keywords: vec![],
            yanked: false,
            published_at: None,
        };
        let v2 = PackageVersion {
            vers: "0.2.0".into(),
            yanked: true,
            ..v1.clone()
        };
        let v3 = PackageVersion {
            vers: "1.0.0".into(),
            yanked: false,
            ..v1.clone()
        };
        let entry = PackageEntry::from_versions(vec![v1, v2, v3]).unwrap();
        assert_eq!(entry.latest().unwrap().vers, "1.0.0");
    }

    #[test]
    fn test_package_entry_latest_all_yanked() {
        let v = PackageVersion {
            name: "test".into(),
            vers: "1.0.0".into(),
            pkg_type: PackageType::Tool,
            description: None,
            author: None,
            repository: None,
            source: "s".into(),
            skills: vec![],
            pipelines: vec![],
            tools: vec![],
            deps: vec![],
            keywords: vec![],
            yanked: true,
            published_at: None,
        };
        let entry = PackageEntry::from_versions(vec![v]).unwrap();
        // Falls back to last version even if yanked
        assert_eq!(entry.latest().unwrap().vers, "1.0.0");
    }
}
