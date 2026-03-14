use std::path::{Path, PathBuf};

/// Scans a project directory to produce a technical profile.
///
/// Detection is purely file-system based — no build commands are executed.
pub struct ProjectScanner {
    root: PathBuf,
}

// ── Structured detection results ──

#[derive(Debug, Default)]
struct TechStack {
    languages: Vec<LanguageInfo>,
    build_system: Option<String>,
}

#[derive(Debug)]
struct LanguageInfo {
    name: String,
    version: Option<String>,
    edition: Option<String>,
    frameworks: Vec<String>,
}

#[derive(Debug, Default)]
struct ProjectStructure {
    kind: String,
    members: Vec<String>,
    notable_dirs: Vec<String>,
}

#[derive(Debug, Default)]
struct DevPractices {
    ci: Vec<String>,
    linting: Vec<String>,
    formatting: Vec<String>,
    testing: Vec<String>,
}

#[derive(Debug)]
struct Dependency {
    name: String,
    version: Option<String>,
    description: Option<String>,
}

// ── Scanner implementation ──

impl ProjectScanner {
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.to_path_buf(),
        }
    }

    /// Run all detectors and render the result as Markdown.
    pub fn scan(&self) -> String {
        let tech = self.detect_tech_stack();
        let structure = self.detect_structure();
        let practices = self.detect_practices();
        let deps = self.detect_key_dependencies();
        self.render(&tech, &structure, &practices, &deps)
    }

    // ── Language & framework detection ──

    fn detect_tech_stack(&self) -> TechStack {
        let mut stack = TechStack::default();

        if let Some(info) = self.detect_rust() {
            stack.build_system = Some(if info.edition.is_some() {
                "Cargo workspace".into()
            } else {
                "Cargo".into()
            });
            stack.languages.push(info);
        }

        if let Some(info) = self.detect_node() {
            if stack.build_system.is_none() {
                stack.build_system = Some(self.detect_node_pkg_manager());
            }
            stack.languages.push(info);
        }

        if let Some(info) = self.detect_go() {
            if stack.build_system.is_none() {
                stack.build_system = Some("Go modules".into());
            }
            stack.languages.push(info);
        }

        if let Some(info) = self.detect_python() {
            stack.languages.push(info);
        }

        if let Some(info) = self.detect_java() {
            stack.languages.push(info);
        }

        stack
    }

    fn detect_rust(&self) -> Option<LanguageInfo> {
        let cargo_toml = self.root.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml).ok()?;
        let parsed: toml::Value = content.parse().ok()?;

        let edition = parsed
            .get("package")
            .and_then(|p| p.get("edition"))
            .and_then(|e| e.as_str())
            .or_else(|| {
                parsed
                    .get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("edition"))
                    .and_then(|e| e.as_str())
            })
            .map(String::from);

        let rust_version = parsed
            .get("package")
            .and_then(|p| p.get("rust-version"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                parsed
                    .get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("rust-version"))
                    .and_then(|v| v.as_str())
            })
            .map(String::from);

        let mut frameworks = Vec::new();

        let deps_tables = [
            parsed.get("dependencies"),
            parsed.get("workspace").and_then(|w| w.get("dependencies")),
        ];
        for deps in deps_tables.iter().flatten() {
            if let Some(table) = deps.as_table() {
                for name in [
                    "tauri",
                    "axum",
                    "actix-web",
                    "rocket",
                    "tokio",
                    "clap",
                    "serde",
                ] {
                    if table.contains_key(name) {
                        frameworks.push(name.to_string());
                    }
                }
            }
        }

        Some(LanguageInfo {
            name: "Rust".into(),
            version: rust_version,
            edition,
            frameworks,
        })
    }

    fn detect_node(&self) -> Option<LanguageInfo> {
        let pkg_json = self.root.join("package.json");
        let content = std::fs::read_to_string(&pkg_json).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;

        let has_ts = self.root.join("tsconfig.json").exists();
        let lang_name = if has_ts { "TypeScript" } else { "JavaScript" };

        let version = parsed
            .get("engines")
            .and_then(|e| e.get("node"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut frameworks = Vec::new();
        let dep_sections = ["dependencies", "devDependencies"];
        for section in &dep_sections {
            if let Some(deps) = parsed.get(*section).and_then(|d| d.as_object()) {
                for name in [
                    "react", "next", "vue", "nuxt", "angular", "svelte", "express", "fastify",
                    "nestjs", "electron", "vite", "webpack",
                ] {
                    if deps.contains_key(name) && !frameworks.contains(&name.to_string()) {
                        frameworks.push(name.to_string());
                    }
                }
            }
        }

        Some(LanguageInfo {
            name: lang_name.into(),
            version,
            edition: None,
            frameworks,
        })
    }

    fn detect_node_pkg_manager(&self) -> String {
        if self.root.join("pnpm-lock.yaml").exists() {
            "pnpm".into()
        } else if self.root.join("yarn.lock").exists() {
            "yarn".into()
        } else if self.root.join("bun.lockb").exists() || self.root.join("bun.lock").exists() {
            "bun".into()
        } else {
            "npm".into()
        }
    }

    fn detect_go(&self) -> Option<LanguageInfo> {
        let go_mod = self.root.join("go.mod");
        let content = std::fs::read_to_string(&go_mod).ok()?;

        let version = content
            .lines()
            .find(|l| l.starts_with("go "))
            .map(|l| l.trim_start_matches("go ").trim().to_string());

        let mut frameworks = Vec::new();
        for name in ["gin", "echo", "fiber", "chi", "grpc"] {
            if content.contains(&format!("/{name}")) {
                frameworks.push(name.to_string());
            }
        }

        Some(LanguageInfo {
            name: "Go".into(),
            version,
            edition: None,
            frameworks,
        })
    }

    fn detect_python(&self) -> Option<LanguageInfo> {
        let pyproject = self.root.join("pyproject.toml");
        let requirements = self.root.join("requirements.txt");
        let setup_py = self.root.join("setup.py");

        if !pyproject.exists() && !requirements.exists() && !setup_py.exists() {
            return None;
        }

        let mut version = None;
        let mut frameworks = Vec::new();

        if let Ok(content) = std::fs::read_to_string(&pyproject)
            && let Ok(parsed) = content.parse::<toml::Value>()
        {
            version = parsed
                .get("project")
                .and_then(|p| p.get("requires-python"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let dep_list = parsed
                .get("project")
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_array());
            if let Some(deps) = dep_list {
                for dep in deps {
                    if let Some(s) = dep.as_str() {
                        let name = s
                            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                            .next()
                            .unwrap_or("");
                        match name {
                            "django" | "flask" | "fastapi" | "pytest" | "torch" | "numpy" => {
                                frameworks.push(name.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Some(LanguageInfo {
            name: "Python".into(),
            version,
            edition: None,
            frameworks,
        })
    }

    fn detect_java(&self) -> Option<LanguageInfo> {
        let pom = self.root.join("pom.xml");
        let gradle = self.root.join("build.gradle");
        let gradle_kts = self.root.join("build.gradle.kts");

        if !pom.exists() && !gradle.exists() && !gradle_kts.exists() {
            return None;
        }

        let build = if pom.exists() { "Maven" } else { "Gradle" };

        let mut frameworks = Vec::new();
        let files = [&pom, &gradle, &gradle_kts];
        for f in &files {
            if let Ok(content) = std::fs::read_to_string(f) {
                for name in ["spring-boot", "quarkus", "micronaut"] {
                    if content.contains(name) && !frameworks.contains(&name.to_string()) {
                        frameworks.push(name.to_string());
                    }
                }
            }
        }

        Some(LanguageInfo {
            name: "Java".into(),
            version: None,
            edition: Some(format!("{} build", build)),
            frameworks,
        })
    }

    // ── Structure detection ──

    fn detect_structure(&self) -> ProjectStructure {
        let mut structure = ProjectStructure {
            kind: "Single project".into(),
            ..Default::default()
        };

        // Cargo workspace
        if let Ok(content) = std::fs::read_to_string(self.root.join("Cargo.toml"))
            && let Ok(parsed) = content.parse::<toml::Value>()
            && let Some(members) = parsed
                .get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
        {
            structure.kind = "Monorepo (Cargo workspace)".into();
            structure.members = members
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        // npm/pnpm workspaces
        if structure.kind.starts_with("Single") {
            if let Ok(content) = std::fs::read_to_string(self.root.join("package.json"))
                && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(workspaces) = parsed.get("workspaces").and_then(|w| w.as_array())
            {
                structure.kind = "Monorepo (npm workspaces)".into();
                structure.members = workspaces
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            if structure.kind.starts_with("Single")
                && let Ok(content) = std::fs::read_to_string(self.root.join("pnpm-workspace.yaml"))
                && content.contains("packages:")
            {
                structure.kind = "Monorepo (pnpm workspace)".into();
            }
        }

        let notable = [
            "src", "lib", "crates", "packages", "apps", "services", "tests", "docs", "scripts",
            "ui", "web", "api", "cmd", "internal", "pkg",
        ];
        for d in &notable {
            if self.root.join(d).is_dir() {
                structure.notable_dirs.push(d.to_string());
            }
        }

        structure
    }

    // ── Dev practices detection ──

    fn detect_practices(&self) -> DevPractices {
        let mut practices = DevPractices::default();

        // CI
        if self.root.join(".github/workflows").is_dir() {
            practices.ci.push("GitHub Actions".into());
        }
        if self.root.join(".gitlab-ci.yml").exists() {
            practices.ci.push("GitLab CI".into());
        }
        if self.root.join("Jenkinsfile").exists() {
            practices.ci.push("Jenkins".into());
        }
        if self.root.join(".circleci").is_dir() {
            practices.ci.push("CircleCI".into());
        }

        // Linting
        let lint_markers = [
            (".eslintrc", "ESLint"),
            (".eslintrc.js", "ESLint"),
            (".eslintrc.json", "ESLint"),
            (".eslintrc.yml", "ESLint"),
            ("eslint.config.js", "ESLint"),
            ("eslint.config.mjs", "ESLint"),
            ("clippy.toml", "clippy"),
            (".clippy.toml", "clippy"),
            (".pylintrc", "pylint"),
            ("ruff.toml", "ruff"),
            (".golangci.yml", "golangci-lint"),
            (".golangci.yaml", "golangci-lint"),
            ("biome.json", "Biome"),
        ];
        for (file, tool) in &lint_markers {
            if self.root.join(file).exists() && !practices.linting.contains(&tool.to_string()) {
                practices.linting.push(tool.to_string());
            }
        }

        // Formatting
        let fmt_markers = [
            ("rustfmt.toml", "rustfmt"),
            (".rustfmt.toml", "rustfmt"),
            (".prettierrc", "Prettier"),
            (".prettierrc.json", "Prettier"),
            (".prettierrc.js", "Prettier"),
            ("prettier.config.js", "Prettier"),
            (".editorconfig", "EditorConfig"),
        ];
        for (file, tool) in &fmt_markers {
            if self.root.join(file).exists() && !practices.formatting.contains(&tool.to_string()) {
                practices.formatting.push(tool.to_string());
            }
        }

        // Testing
        if self.root.join("Cargo.toml").exists() {
            practices.testing.push("cargo test".into());
        }
        let test_markers = [
            ("jest.config.js", "Jest"),
            ("jest.config.ts", "Jest"),
            ("vitest.config.ts", "Vitest"),
            ("vitest.config.js", "Vitest"),
            ("playwright.config.ts", "Playwright"),
            ("cypress.config.ts", "Cypress"),
            ("cypress.config.js", "Cypress"),
            ("pytest.ini", "pytest"),
            ("conftest.py", "pytest"),
        ];
        for (file, tool) in &test_markers {
            if self.root.join(file).exists() && !practices.testing.contains(&tool.to_string()) {
                practices.testing.push(tool.to_string());
            }
        }
        // pytest from pyproject.toml
        if !practices.testing.contains(&"pytest".to_string())
            && let Ok(content) = std::fs::read_to_string(self.root.join("pyproject.toml"))
            && content.contains("[tool.pytest")
        {
            practices.testing.push("pytest".into());
        }

        practices
    }

    // ── Key dependency extraction ──

    fn detect_key_dependencies(&self) -> Vec<Dependency> {
        let mut deps = Vec::new();

        // Rust deps from workspace Cargo.toml
        if let Ok(content) = std::fs::read_to_string(self.root.join("Cargo.toml"))
            && let Ok(parsed) = content.parse::<toml::Value>()
        {
            let tables = [
                parsed.get("dependencies"),
                parsed.get("workspace").and_then(|w| w.get("dependencies")),
            ];
            for table in tables.into_iter().flatten() {
                if let Some(t) = table.as_table() {
                    for (name, val) in t {
                        let version = match val {
                            toml::Value::String(s) => Some(s.clone()),
                            toml::Value::Table(t) => {
                                t.get("version").and_then(|v| v.as_str()).map(String::from)
                            }
                            _ => None,
                        };
                        deps.push(Dependency {
                            name: name.clone(),
                            version,
                            description: None,
                        });
                    }
                }
            }
        }

        // Node deps from package.json (top-level only)
        if let Ok(content) = std::fs::read_to_string(self.root.join("package.json"))
            && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(d) = parsed.get("dependencies").and_then(|d| d.as_object())
        {
            for (name, val) in d {
                deps.push(Dependency {
                    name: name.clone(),
                    version: val.as_str().map(String::from),
                    description: None,
                });
            }
        }

        // Go deps from go.mod
        if let Ok(content) = std::fs::read_to_string(self.root.join("go.mod")) {
            let in_require = content
                .lines()
                .skip_while(|l| !l.starts_with("require"))
                .skip(1)
                .take_while(|l| !l.starts_with(')'));
            for line in in_require {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mod_path = parts[0];
                    let short = mod_path.rsplit('/').next().unwrap_or(mod_path);
                    deps.push(Dependency {
                        name: short.to_string(),
                        version: Some(parts[1].to_string()),
                        description: Some(mod_path.to_string()),
                    });
                }
            }
        }

        deps
    }

    // ── Markdown rendering ──

    fn render(
        &self,
        tech: &TechStack,
        structure: &ProjectStructure,
        practices: &DevPractices,
        deps: &[Dependency],
    ) -> String {
        let mut out = String::new();

        out.push_str("# Project Context\n\n");
        out.push_str(
            "> Auto-generated by `popsicle context scan`. You may edit this file freely.\n\n",
        );

        // Tech Stack
        out.push_str("## Tech Stack\n\n");
        if tech.languages.is_empty() {
            out.push_str("- (no languages detected)\n");
        }
        for lang in &tech.languages {
            let mut label = lang.name.clone();
            if let Some(ref v) = lang.version {
                label.push_str(&format!(" {}", v));
            }
            if let Some(ref ed) = lang.edition {
                label.push_str(&format!(" ({})", ed));
            }
            out.push_str(&format!("- **Language**: {}\n", label));
            if !lang.frameworks.is_empty() {
                out.push_str(&format!(
                    "- **Frameworks**: {}\n",
                    lang.frameworks.join(", ")
                ));
            }
        }
        if let Some(ref bs) = tech.build_system {
            out.push_str(&format!("- **Build**: {}\n", bs));
        }
        out.push('\n');

        // Project Structure
        out.push_str("## Project Structure\n\n");
        out.push_str(&format!("- **Type**: {}\n", structure.kind));
        if !structure.members.is_empty() {
            out.push_str(&format!(
                "- **Members**: {}\n",
                structure.members.join(", ")
            ));
        }
        if !structure.notable_dirs.is_empty() {
            out.push_str(&format!(
                "- **Directories**: {}\n",
                structure.notable_dirs.join(", ")
            ));
        }
        out.push('\n');

        // Dev Practices
        out.push_str("## Development Practices\n\n");
        if !practices.ci.is_empty() {
            out.push_str(&format!("- **CI**: {}\n", practices.ci.join(", ")));
        }
        if !practices.linting.is_empty() {
            out.push_str(&format!(
                "- **Linting**: {}\n",
                practices.linting.join(", ")
            ));
        }
        if !practices.formatting.is_empty() {
            out.push_str(&format!(
                "- **Formatting**: {}\n",
                practices.formatting.join(", ")
            ));
        }
        if !practices.testing.is_empty() {
            out.push_str(&format!(
                "- **Testing**: {}\n",
                practices.testing.join(", ")
            ));
        }
        if practices.ci.is_empty()
            && practices.linting.is_empty()
            && practices.formatting.is_empty()
            && practices.testing.is_empty()
        {
            out.push_str("- (no dev tooling detected)\n");
        }
        out.push('\n');

        // Key Dependencies
        if !deps.is_empty() {
            out.push_str("## Key Dependencies\n\n");
            for dep in deps {
                let ver = dep.version.as_deref().unwrap_or("*");
                if let Some(ref desc) = dep.description {
                    out.push_str(&format!("- {} {} — {}\n", dep.name, ver, desc));
                } else {
                    out.push_str(&format!("- {} {}\n", dep.name, ver));
                }
            }
            out.push('\n');
        }

        // Notes section for user edits
        out.push_str("## Notes\n\n");
        out.push_str("(Add project-specific context, conventions, or constraints here)\n");

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_scan_empty_project() {
        let dir = setup_temp_dir();
        let scanner = ProjectScanner::new(dir.path());
        let result = scanner.scan();
        assert!(result.contains("# Project Context"));
        assert!(result.contains("(no languages detected)"));
    }

    #[test]
    fn test_detect_rust_workspace() {
        let dir = setup_temp_dir();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["crates/core", "crates/cli"]

[workspace.package]
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
serde = "1.0"
clap = { version = "4.0" }
tokio = { version = "1", features = ["full"] }
"#,
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("crates")).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        fs::write(dir.path().join("rustfmt.toml"), "").unwrap();

        let scanner = ProjectScanner::new(dir.path());
        let result = scanner.scan();

        assert!(result.contains("Rust"));
        assert!(result.contains("2021"));
        assert!(result.contains("1.75"));
        assert!(result.contains("Monorepo (Cargo workspace)"));
        assert!(result.contains("crates/core"));
        assert!(result.contains("GitHub Actions"));
        assert!(result.contains("rustfmt"));
        assert!(result.contains("serde"));
        assert!(result.contains("clap"));
    }

    #[test]
    fn test_detect_node_typescript() {
        let dir = setup_temp_dir();
        fs::write(
            dir.path().join("package.json"),
            r#"{
  "name": "my-app",
  "engines": { "node": ">=18" },
  "dependencies": { "react": "^18.0", "next": "^14.0" },
  "devDependencies": { "typescript": "^5.0" }
}"#,
        )
        .unwrap();
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let scanner = ProjectScanner::new(dir.path());
        let result = scanner.scan();

        assert!(result.contains("TypeScript"));
        assert!(result.contains(">=18"));
        assert!(result.contains("react"));
        assert!(result.contains("next"));
    }

    #[test]
    fn test_detect_go() {
        let dir = setup_temp_dir();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/myapp\n\ngo 1.22\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)\n",
        )
        .unwrap();

        let scanner = ProjectScanner::new(dir.path());
        let result = scanner.scan();

        assert!(result.contains("Go"));
        assert!(result.contains("1.22"));
        assert!(result.contains("gin"));
    }

    #[test]
    fn test_notes_section_present() {
        let dir = setup_temp_dir();
        let scanner = ProjectScanner::new(dir.path());
        let result = scanner.scan();
        assert!(result.contains("## Notes"));
        assert!(result.contains("Add project-specific context"));
    }
}
