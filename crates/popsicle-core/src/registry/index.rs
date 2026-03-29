use std::path::{Path, PathBuf};

use crate::error::{PopsicleError, Result};
use crate::registry::package::{PackageEntry, PackageType, PackageVersion, RegistryConfig, index_path};

/// Default registry URL.
const DEFAULT_REGISTRY: &str = "https://github.com/popsicle-lab/popsicle-registry.git";

/// Local cache directory name under the user home.
const CACHE_DIR: &str = ".popsicle/registry";

/// Git-based package registry index.
///
/// The index is a git repository cloned locally. Each package has a file at
/// `<prefix>/<name>` containing one JSON line per published version (NDJSON).
///
/// The CLI calls [`RegistryIndex::open`] to get a handle, then uses
/// [`search`], [`get`], or [`resolve`] to query packages.
pub struct RegistryIndex {
    /// Local path to the cloned index repository.
    local_path: PathBuf,
    /// Remote URL.
    remote_url: String,
}

impl RegistryIndex {
    /// Open the registry, cloning it on first use and fetching updates otherwise.
    ///
    /// The index is cached at `~/.popsicle/registry/`.
    pub fn open(registry_url: Option<&str>) -> Result<Self> {
        let remote_url = registry_url
            .unwrap_or(DEFAULT_REGISTRY)
            .to_string();

        let local_path = cache_dir()?;

        if local_path.join(".git").is_dir() {
            // Already cloned — fetch latest
            git_fetch(&local_path)?;
        } else {
            // First time — clone
            std::fs::create_dir_all(&local_path)?;
            git_clone(&remote_url, &local_path)?;
        }

        Ok(Self {
            local_path,
            remote_url,
        })
    }

    /// Open the registry from an already-cloned local path (for testing / offline).
    pub fn open_local(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            return Err(PopsicleError::Storage(format!(
                "Registry path does not exist: {}",
                path.display()
            )));
        }
        Ok(Self {
            local_path: path.to_path_buf(),
            remote_url: String::new(),
        })
    }

    /// The local filesystem path of the index.
    pub fn path(&self) -> &Path {
        &self.local_path
    }

    /// The remote URL.
    pub fn remote_url(&self) -> &str {
        &self.remote_url
    }

    /// Load the registry config.json.
    pub fn config(&self) -> Result<RegistryConfig> {
        let config_path = self.local_path.join("config.json");
        if !config_path.exists() {
            return Ok(RegistryConfig {
                version: 1,
                name: None,
                registry_url: Some(self.remote_url.clone()),
                website_url: None,
            });
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: RegistryConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Get a specific package by name.
    pub fn get(&self, name: &str) -> Result<PackageEntry> {
        let rel_path = index_path(name);
        let file_path = self.local_path.join(&rel_path);

        if !file_path.exists() {
            return Err(PopsicleError::Storage(format!(
                "Package '{}' not found in registry",
                name
            )));
        }

        let versions = read_package_file(&file_path)?;
        PackageEntry::from_versions(versions).ok_or_else(|| {
            PopsicleError::Storage(format!("Package '{}' has no versions", name))
        })
    }

    /// Resolve a package name (optionally with `@version`) to a source string.
    ///
    /// Accepted formats:
    /// - `"spec-development"` → latest non-yanked version
    /// - `"spec-development@1.0.0"` → exact version
    pub fn resolve(&self, name_at_version: &str) -> Result<ResolvedPackage> {
        let (name, version) = match name_at_version.split_once('@') {
            Some((n, v)) => (n, Some(v)),
            None => (name_at_version, None),
        };

        let entry = self.get(name)?;

        let pkg_version = if let Some(v) = version {
            entry.version(v).ok_or_else(|| {
                PopsicleError::Storage(format!(
                    "Version '{}' not found for package '{}'",
                    v, name
                ))
            })?
        } else {
            entry.latest().ok_or_else(|| {
                PopsicleError::Storage(format!(
                    "No available version for package '{}'",
                    name
                ))
            })?
        };

        Ok(ResolvedPackage {
            name: pkg_version.name.clone(),
            version: pkg_version.vers.clone(),
            pkg_type: pkg_version.pkg_type,
            source: pkg_version.source.clone(),
            deps: pkg_version.deps.clone(),
        })
    }

    /// Search packages by a text query, optionally filtered by type.
    ///
    /// Searches name, description, keywords, skills, pipelines, and tools.
    pub fn search(
        &self,
        query: &str,
        pkg_type: Option<PackageType>,
    ) -> Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Walk all package files in the index
        let packages = self.list_all_packages()?;

        for entry in packages {
            if let Some(latest) = entry.latest() {
                // Type filter
                if let Some(t) = pkg_type {
                    if latest.pkg_type != t {
                        continue;
                    }
                }

                // Text match: name, description, keywords, skills, pipelines, tools
                let score = compute_search_score(latest, &query_lower);
                if score > 0 {
                    results.push(SearchResult {
                        name: latest.name.clone(),
                        version: latest.vers.clone(),
                        pkg_type: latest.pkg_type,
                        description: latest.description.clone(),
                        keywords: latest.keywords.clone(),
                        score,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
        Ok(results)
    }

    /// List all packages in the index.
    pub fn list_all_packages(&self) -> Result<Vec<PackageEntry>> {
        let mut entries = Vec::new();
        walk_index_dir(&self.local_path, &self.local_path, &mut entries)?;
        Ok(entries)
    }

    /// Add a new version entry to the index (used by `publish`).
    /// This modifies the local index — the caller must commit and push.
    pub fn add_version(&self, version: &PackageVersion) -> Result<PathBuf> {
        let rel_path = index_path(&version.name);
        let file_path = self.local_path.join(&rel_path);

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check for duplicate version
        if file_path.exists() {
            let existing = read_package_file(&file_path)?;
            if existing.iter().any(|v| v.vers == version.vers) {
                return Err(PopsicleError::Storage(format!(
                    "Version '{}' already published for package '{}'",
                    version.vers, version.name
                )));
            }
        }

        // Append NDJSON line
        let line = serde_json::to_string(version)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        writeln!(file, "{}", line)?;

        Ok(file_path)
    }

    /// Commit and push changes to the remote index.
    pub fn commit_and_push(&self, message: &str) -> Result<()> {
        git_add_all(&self.local_path)?;
        git_commit(&self.local_path, message)?;
        git_push(&self.local_path)?;
        Ok(())
    }
}

/// Result of resolving a package from the registry.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub pkg_type: PackageType,
    pub source: String,
    pub deps: Vec<crate::registry::package::PackageDep>,
}

/// A search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub pkg_type: PackageType,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub score: u32,
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| PopsicleError::Storage("Cannot determine home directory".into()))?;
    Ok(PathBuf::from(home).join(CACHE_DIR))
}

fn read_package_file(path: &Path) -> Result<Vec<PackageVersion>> {
    let content = std::fs::read_to_string(path)?;
    let mut versions = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: PackageVersion = serde_json::from_str(line)?;
        versions.push(v);
    }
    Ok(versions)
}

/// Recursively walk the index directory, skipping `.git`, `config.json`, etc.
fn walk_index_dir(
    base: &Path,
    dir: &Path,
    entries: &mut Vec<PackageEntry>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // Skip hidden dirs (.git), config files, README, etc.
        if name_str.starts_with('.') || name_str == "config.json" || name_str == "README.md" {
            continue;
        }

        if path.is_dir() {
            walk_index_dir(base, &path, entries)?;
        } else if path.is_file() {
            // This should be a package NDJSON file
            if let Ok(versions) = read_package_file(&path) {
                if let Some(pkg) = PackageEntry::from_versions(versions) {
                    entries.push(pkg);
                }
            }
        }
    }

    Ok(())
}

/// Compute a simple relevance score for search.
fn compute_search_score(v: &PackageVersion, query: &str) -> u32 {
    let mut score = 0u32;

    // Exact name match
    if v.name.to_lowercase() == query {
        score += 100;
    } else if v.name.to_lowercase().contains(query) {
        score += 50;
    }

    // Description
    if let Some(desc) = &v.description {
        if desc.to_lowercase().contains(query) {
            score += 20;
        }
    }

    // Keywords
    for kw in &v.keywords {
        if kw.to_lowercase() == query {
            score += 30;
        } else if kw.to_lowercase().contains(query) {
            score += 10;
        }
    }

    // Skills, pipelines, tools
    for s in v.skills.iter().chain(v.pipelines.iter()).chain(v.tools.iter()) {
        if s.to_lowercase().contains(query) {
            score += 5;
        }
    }

    score
}

// ── Git subprocess helpers ────────────────────────────────────────────────

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output().map_err(|e| {
        PopsicleError::Storage(format!("Failed to run git: {}", e))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PopsicleError::Storage(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        )));
    }
    Ok(())
}

fn git_clone(url: &str, dest: &Path) -> Result<()> {
    // Remove the dir if it exists but is empty / partial
    if dest.is_dir() {
        std::fs::remove_dir_all(dest)?;
    }
    run_git(&["clone", "--depth", "1", url, &dest.to_string_lossy()], None)
}

fn git_fetch(repo: &Path) -> Result<()> {
    // Discard any uncommitted local changes (e.g. leftover from a failed publish)
    let _ = run_git(&["checkout", "--", "."], Some(repo));

    run_git(&["fetch", "--depth", "1", "origin", "main"], Some(repo))
        .or_else(|_| run_git(&["fetch", "--depth", "1", "origin"], Some(repo)))
        .or_else(|_| run_git(&["pull", "--ff-only"], Some(repo)))?;

    // Reset local branch to match remote so the working tree is up to date.
    // Without this, `git fetch` updates refs but leaves files stale, causing
    // false "version already exists" errors and push rejections.
    run_git(&["reset", "--hard", "origin/main"], Some(repo))
        .or_else(|_| run_git(&["reset", "--hard", "FETCH_HEAD"], Some(repo)))
}

fn git_add_all(repo: &Path) -> Result<()> {
    run_git(&["add", "-A"], Some(repo))
}

fn git_commit(repo: &Path, message: &str) -> Result<()> {
    run_git(&["commit", "-m", message], Some(repo))
}

fn git_push(repo: &Path) -> Result<()> {
    run_git(&["push"], Some(repo))
}

/// Check if a package name looks like a registry name (no paths, no prefixes).
///
/// Returns `true` for: `spec-development`, `draw-diagram@1.0.0`
/// Returns `false` for: `github:org/repo`, `./local-path`, `/abs/path`
pub fn is_registry_name(source: &str) -> bool {
    let name = source.split('@').next().unwrap_or(source);
    !name.contains('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.starts_with('.')
        && name.len() >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_registry_name() {
        assert!(is_registry_name("spec-development"));
        assert!(is_registry_name("draw-diagram@1.0.0"));
        assert!(is_registry_name("my-tool"));

        assert!(!is_registry_name("github:org/repo"));
        assert!(!is_registry_name("./local"));
        assert!(!is_registry_name("/abs/path"));
        assert!(!is_registry_name("a")); // too short
    }

    #[test]
    fn test_search_score_exact_name() {
        let v = PackageVersion {
            name: "draw-diagram".into(),
            vers: "1.0.0".into(),
            pkg_type: PackageType::Tool,
            description: Some("Generate diagrams".into()),
            author: None,
            repository: None,
            source: "s".into(),
            skills: vec![],
            pipelines: vec![],
            tools: vec![],
            deps: vec![],
            keywords: vec!["diagram".into()],
            yanked: false,
            published_at: None,
        };
        assert_eq!(compute_search_score(&v, "draw-diagram"), 100);
        assert!(compute_search_score(&v, "diagram") > 0);
        assert_eq!(compute_search_score(&v, "unrelated"), 0);
    }

    #[test]
    fn test_open_local_index() {
        let tmp = tempfile::tempdir().unwrap();
        let index = RegistryIndex::open_local(tmp.path()).unwrap();
        let pkgs = index.list_all_packages().unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn test_add_and_read_version() {
        let tmp = tempfile::tempdir().unwrap();
        let index = RegistryIndex::open_local(tmp.path()).unwrap();

        let v = PackageVersion {
            name: "test-pkg".into(),
            vers: "0.1.0".into(),
            pkg_type: PackageType::Module,
            description: Some("A test".into()),
            author: None,
            repository: None,
            source: "github:x/y#v0.1.0".into(),
            skills: vec!["prd".into()],
            pipelines: vec!["full".into()],
            tools: vec![],
            deps: vec![],
            keywords: vec!["test".into()],
            yanked: false,
            published_at: None,
        };

        index.add_version(&v).unwrap();

        let entry = index.get("test-pkg").unwrap();
        assert_eq!(entry.versions.len(), 1);
        assert_eq!(entry.latest().unwrap().vers, "0.1.0");
    }

    #[test]
    fn test_add_duplicate_version_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let index = RegistryIndex::open_local(tmp.path()).unwrap();

        let v = PackageVersion {
            name: "dup-pkg".into(),
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
            yanked: false,
            published_at: None,
        };

        index.add_version(&v).unwrap();
        assert!(index.add_version(&v).is_err());
    }

    #[test]
    fn test_search() {
        let tmp = tempfile::tempdir().unwrap();
        let index = RegistryIndex::open_local(tmp.path()).unwrap();

        let v1 = PackageVersion {
            name: "draw-diagram".into(),
            vers: "1.0.0".into(),
            pkg_type: PackageType::Tool,
            description: Some("Generate Mermaid diagrams".into()),
            author: None,
            repository: None,
            source: "s".into(),
            skills: vec![],
            pipelines: vec![],
            tools: vec![],
            deps: vec![],
            keywords: vec!["diagram".into(), "mermaid".into()],
            yanked: false,
            published_at: None,
        };
        let v2 = PackageVersion {
            name: "spec-development".into(),
            vers: "2.0.0".into(),
            pkg_type: PackageType::Module,
            description: Some("Full SDLC spec module".into()),
            author: None,
            repository: None,
            source: "s".into(),
            skills: vec!["prd".into()],
            pipelines: vec!["full-sdlc".into()],
            tools: vec![],
            deps: vec![],
            keywords: vec!["sdlc".into()],
            yanked: false,
            published_at: None,
        };

        index.add_version(&v1).unwrap();
        index.add_version(&v2).unwrap();

        let results = index.search("diagram", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "draw-diagram");

        let results = index.search("sdlc", Some(PackageType::Module)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "spec-development");

        // Type filter excludes
        let results = index.search("diagram", Some(PackageType::Module)).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let index = RegistryIndex::open_local(tmp.path()).unwrap();

        let v1 = PackageVersion {
            name: "my-tool".into(),
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
            vers: "1.0.0".into(),
            source: "github:x/y#v1.0.0".into(),
            ..v1.clone()
        };

        index.add_version(&v1).unwrap();
        index.add_version(&v2).unwrap();

        // Latest
        let resolved = index.resolve("my-tool").unwrap();
        assert_eq!(resolved.version, "1.0.0");
        assert_eq!(resolved.source, "github:x/y#v1.0.0");

        // Exact version
        let resolved = index.resolve("my-tool@0.1.0").unwrap();
        assert_eq!(resolved.version, "0.1.0");

        // Not found
        assert!(index.resolve("nonexistent").is_err());
        assert!(index.resolve("my-tool@9.9.9").is_err());
    }
}
