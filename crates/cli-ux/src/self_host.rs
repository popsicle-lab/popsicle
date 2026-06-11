//! Local workspace store: SQLite at `.popsicle/self-host/state.db` (ADR-009
//! Phase 2, PROJ-11) with read compatibility for the Phase 1 TSV backend.
//!
//! Backend is auto-detected at open: an existing `popsicle.db` wins, an
//! existing legacy `state.tsv` keeps the TSV backend, and fresh workspaces
//! default to SQLite. `admin migrate` performs the TSV → SQLite migration.
//! Pipeline session working files stay as per-run JSON (ADR-013).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use artifact_system::Document;
use skill_runtime::domain::{PipelineRunStatus, StageStatus};
use skill_runtime::loader::PipelineDef;
use skill_runtime::pipeline_session::PipelineSession;
use skill_runtime::{Issue, IssueType};
use storage::{
    DocCheckRow, DocCreateRow, DocumentRow, IssueRow, PipelineStatusRow, RunRow, RunStartRow,
    SqliteStateDb, StageCompleteRow, StateSnapshot, WorkspaceError, WorkspaceStore,
};

use crate::global_config::{
    global_config_path, resolve_init_root, resolve_workspace_root, ResolvedWorkspace,
    WorkspaceSource,
};
use crate::project_config::{
    ensure_project_config, load_project_config, project_config_path, prompt_context_block,
    sync_agents_md,
};

const SELF_HOST_DIR: &str = ".popsicle/self-host";
const STATE_FILE: &str = "state.tsv";
// NOT `.popsicle/popsicle.db`: that path belongs to the legacy binary's
// database (different schema). See ADR-013.
const DB_FILE: &str = "state.db";
const RUNS_DIR: &str = "runs";
const PIPELINES_DIR: &str = ".popsicle/pipelines";
const INTENT_CODER_MODULE_REL: &str = ".popsicle/modules/intent-coder";

/// Pipelines bundled into the binary so `popsicle init` can bootstrap a brand-new
/// project without copying the intent-coder module first.
const BUNDLED_PIPELINES: &[(&str, &str)] = &[
    (
        "greenfield-product-spec",
        include_str!("../assets/pipelines/greenfield-product-spec.pipeline.yaml"),
    ),
    (
        "migration-bootstrap",
        include_str!("../assets/pipelines/migration-bootstrap.pipeline.yaml"),
    ),
    (
        "slice-spec",
        include_str!("../assets/pipelines/slice-spec.pipeline.yaml"),
    ),
    (
        "slice-delivery",
        include_str!("../assets/pipelines/slice-delivery.pipeline.yaml"),
    ),
    (
        "tech-decision",
        include_str!("../assets/pipelines/tech-decision.pipeline.yaml"),
    ),
    (
        "bugfix",
        include_str!("../assets/pipelines/bugfix.pipeline.yaml"),
    ),
];

/// Names of all pipelines bundled into the binary.
pub fn bundled_pipeline_names() -> Vec<&'static str> {
    BUNDLED_PIPELINES.iter().map(|(name, _)| *name).collect()
}

/// Pipeline template names available in a workspace (installed + bundled).
pub fn list_installed_pipeline_names(workspace: &Workspace) -> Vec<String> {
    let dir = workspace.pipelines_dir();
    let mut names: Vec<String> = bundled_pipeline_names()
        .into_iter()
        .map(str::to_string)
        .collect();
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(stem) = entry
                    .file_name()
                    .to_str()
                    .and_then(|s| s.strip_suffix(".pipeline.yaml"))
                {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Resolved workspace root (directory containing `.popsicle/`).
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn discover() -> Result<Self, WorkspaceError> {
        resolve_workspace_root(None).map(|r| Self { root: r.root })
    }

    pub fn discover_with(cli_project: Option<&str>) -> Result<ResolvedWorkspace, WorkspaceError> {
        resolve_workspace_root(cli_project)
    }

    /// Discover an existing workspace, or fall back to the current directory
    /// so `popsicle init` can bootstrap a brand-new project.
    pub fn discover_or_current_dir() -> Result<Self, WorkspaceError> {
        match resolve_workspace_root(None) {
            Ok(r) => Ok(Self { root: r.root }),
            Err(_) => {
                let cwd = std::env::current_dir().map_err(|e| io_err(e.to_string()))?;
                Ok(Self { root: cwd })
            }
        }
    }

    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn self_host_dir(&self) -> PathBuf {
        self.root.join(SELF_HOST_DIR)
    }

    pub fn state_path(&self) -> PathBuf {
        self.self_host_dir().join(STATE_FILE)
    }

    pub fn db_path(&self) -> PathBuf {
        self.self_host_dir().join(DB_FILE)
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.self_host_dir().join(RUNS_DIR)
    }

    pub fn artifacts_dir(&self, run_id: &str) -> PathBuf {
        self.root.join(".popsicle/artifacts").join(run_id)
    }

    pub fn expected_binary(&self) -> PathBuf {
        self.root.join("target/debug/popsicle")
    }

    pub fn pipelines_dir(&self) -> PathBuf {
        self.root.join(PIPELINES_DIR)
    }

    pub fn ensure_layout(&self) -> Result<(), WorkspaceError> {
        fs::create_dir_all(self.self_host_dir()).map_err(io_err)?;
        fs::create_dir_all(self.runs_dir()).map_err(io_err)?;
        Ok(())
    }

    /// Write bundled pipeline definitions into `.popsicle/pipelines/`, skipping
    /// any pipeline the workspace already defines. Returns the names installed.
    pub fn install_bundled_pipelines(&self) -> Result<Vec<&'static str>, WorkspaceError> {
        let dir = self.pipelines_dir();
        fs::create_dir_all(&dir).map_err(io_err)?;
        let mut installed = Vec::new();
        for (name, content) in BUNDLED_PIPELINES {
            let path = dir.join(format!("{name}.pipeline.yaml"));
            if path.exists() {
                continue;
            }
            fs::write(&path, content).map_err(io_err)?;
            installed.push(*name);
        }
        Ok(installed)
    }

    pub fn intent_coder_source(&self) -> PathBuf {
        self.root.join("intent-coder")
    }

    pub fn intent_coder_module_dir(&self) -> PathBuf {
        self.root.join(INTENT_CODER_MODULE_REL)
    }
}

/// Where the module payload came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentCoderSource {
    /// Live `intent-coder/` at workspace root (dogfood / dev checkout).
    WorkspaceRoot,
    /// Compile-time bundle inside the `popsicle` binary (DMG / `cargo install`).
    Embedded,
}

impl IntentCoderSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceRoot => "workspace_root",
            Self::Embedded => "embedded",
        }
    }
}

/// Result of syncing intent-coder into `.popsicle/modules/intent-coder/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentCoderInstallResult {
    pub installed: bool,
    pub version: Option<String>,
    pub dest: String,
    pub source: Option<IntentCoderSource>,
    pub skipped_reason: Option<String>,
}

fn read_module_version(dir: &Path) -> Option<String> {
    let content = fs::read_to_string(dir.join("module.yaml")).ok()?;
    content
        .lines()
        .find(|line| line.starts_with("version:"))
        .and_then(|line| line.split('"').nth(1).map(str::to_string))
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), WorkspaceError> {
    fs::create_dir_all(dest).map_err(io_err)?;
    for entry in fs::read_dir(src).map_err(io_err)? {
        let entry = entry.map_err(|e| io_err(e.to_string()))?;
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let from = entry.path();
        let to = dest.join(&name);
        let ft = entry.file_type().map_err(|e| io_err(e.to_string()))?;
        if ft.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ft.is_file() {
            fs::copy(&from, &to).map_err(io_err)?;
        }
    }
    Ok(())
}

/// Install intent-coder into `.popsicle/modules/intent-coder/`.
///
/// Prefers workspace-root `intent-coder/` when present; otherwise extracts the
/// compile-time bundle (ADR-017) so DMG/`cargo install` projects still work.
pub fn install_intent_coder_module(
    workspace: &Workspace,
    force: bool,
) -> Result<IntentCoderInstallResult, WorkspaceError> {
    let src = workspace.intent_coder_source();
    let dest = workspace.intent_coder_module_dir();
    let source = if src.join("module.yaml").is_file() {
        IntentCoderSource::WorkspaceRoot
    } else {
        IntentCoderSource::Embedded
    };

    if dest.exists() && !force {
        let dest_ver = read_module_version(&dest);
        let src_ver = match source {
            IntentCoderSource::WorkspaceRoot => read_module_version(&src),
            IntentCoderSource::Embedded => crate::intent_coder_bundle::embedded_module_version(),
        };
        if src_ver == dest_ver && dest.join("skills").is_dir() {
            return Ok(IntentCoderInstallResult {
                installed: false,
                version: dest_ver,
                dest: dest.display().to_string(),
                source: Some(source),
                skipped_reason: Some(
                    "already installed (same version; run admin sync-intent-coder to refresh)"
                        .into(),
                ),
            });
        }
    }

    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(io_err)?;
    }

    match source {
        IntentCoderSource::WorkspaceRoot => copy_dir_recursive(&src, &dest)?,
        IntentCoderSource::Embedded => {
            crate::intent_coder_bundle::extract_embedded_intent_coder(&dest)?
        }
    }

    let version = read_module_version(&dest);
    Ok(IntentCoderInstallResult {
        installed: true,
        version,
        dest: dest.display().to_string(),
        source: Some(source),
        skipped_reason: None,
    })
}

pub fn intent_coder_module_version(workspace: &Workspace) -> Option<String> {
    let dest = workspace.intent_coder_module_dir();
    if dest.join("module.yaml").is_file() {
        read_module_version(&dest)
    } else {
        None
    }
}

struct StoreState {
    next_issue_num: u32,
    next_run_num: u32,
    next_doc_num: u32,
    issues: BTreeMap<String, IssueRow>,
    runs: BTreeMap<String, RunRow>,
    documents: BTreeMap<String, DocumentRow>,
}

impl StoreState {
    fn to_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            next_issue_num: self.next_issue_num,
            next_run_num: self.next_run_num,
            next_doc_num: self.next_doc_num,
            issues: self.issues.values().cloned().collect(),
            runs: self.runs.values().cloned().collect(),
            documents: self.documents.values().cloned().collect(),
        }
    }

    fn from_snapshot(snap: StateSnapshot) -> Self {
        Self {
            next_issue_num: snap.next_issue_num,
            next_run_num: snap.next_run_num,
            next_doc_num: snap.next_doc_num,
            issues: snap
                .issues
                .into_iter()
                .map(|i| (i.key.clone(), i))
                .collect(),
            runs: snap
                .runs
                .into_iter()
                .map(|r| (r.run_id.clone(), r))
                .collect(),
            documents: snap
                .documents
                .into_iter()
                .map(|d| (d.id.clone(), d))
                .collect(),
        }
    }
}

/// Which on-disk format backs the indexed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateBackend {
    /// Legacy ADR-009 Phase 1 `state.tsv`.
    Tsv,
    /// ADR-009 Phase 2 SQLite `.popsicle/popsicle.db` (PROJ-11).
    Sqlite,
}

impl StateBackend {
    pub fn describe(&self, workspace: &Workspace) -> String {
        match self {
            Self::Tsv => format!("tsv ({})", rel_display(workspace, &workspace.state_path())),
            Self::Sqlite => format!("sqlite ({})", rel_display(workspace, &workspace.db_path())),
        }
    }
}

fn rel_display(workspace: &Workspace, path: &std::path::Path) -> String {
    path.strip_prefix(&workspace.root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

impl Default for StoreState {
    fn default() -> Self {
        Self {
            next_issue_num: 1,
            next_run_num: 1,
            next_doc_num: 1,
            issues: BTreeMap::new(),
            runs: BTreeMap::new(),
            documents: BTreeMap::new(),
        }
    }
}

/// Local workspace store (ADR-009): SQLite Phase 2 backend with TSV Phase 1
/// read compatibility. Implements [`WorkspaceStore`].
pub struct LocalWorkspace {
    pub workspace: Workspace,
    pub workspace_source: WorkspaceSource,
    backend: StateBackend,
    state: StoreState,
}

impl LocalWorkspace {
    pub fn open() -> Result<Self, WorkspaceError> {
        let resolved = resolve_workspace_root(None)?;
        Self::open_resolved(resolved)
    }

    pub fn open_with(cli_project: Option<&str>) -> Result<Self, WorkspaceError> {
        let resolved = resolve_workspace_root(cli_project)?;
        Self::open_resolved(resolved)
    }

    pub fn open_resolved(resolved: ResolvedWorkspace) -> Result<Self, WorkspaceError> {
        Self::open_at_workspace_with_source(Workspace::at(resolved.root), resolved.source)
    }

    pub fn open_at(root: PathBuf) -> Result<Self, WorkspaceError> {
        Self::open_at_workspace_with_source(Workspace::at(root), WorkspaceSource::CwdWalk)
    }

    fn open_at_workspace_with_source(
        workspace: Workspace,
        workspace_source: WorkspaceSource,
    ) -> Result<Self, WorkspaceError> {
        let backend = detect_backend(&workspace);
        let mut state = load_state(&workspace, backend)?;
        normalize_issue_rows(&workspace, &mut state);
        Ok(Self {
            workspace,
            workspace_source,
            backend,
            state,
        })
    }

    pub fn backend(&self) -> StateBackend {
        self.backend
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn load_run_session(&self, run_id: &str) -> Result<PipelineSession, WorkspaceError> {
        load_session(&self.workspace, run_id)
    }

    pub fn pipeline_name_for_run(&self, run_id: &str) -> Result<String, WorkspaceError> {
        self.state
            .runs
            .get(run_id)
            .map(|r| r.pipeline_name.clone())
            .ok_or_else(|| WorkspaceError::NotFound(run_id.into()))
    }

    pub fn issue_key_for_run(&self, run_id: &str) -> Option<String> {
        self.state.runs.get(run_id).map(|r| r.issue_key.clone())
    }

    /// Migrate a legacy TSV workspace to the SQLite backend. Idempotent: an
    /// already-SQLite workspace reports `migrated=false`. The TSV file is kept
    /// as `state.tsv.migrated` for rollback/audit.
    pub fn migrate_to_sqlite(&mut self) -> Result<bool, WorkspaceError> {
        if self.backend == StateBackend::Sqlite {
            return Ok(false);
        }
        self.backend = StateBackend::Sqlite;
        self.save()?;
        let tsv = self.workspace.state_path();
        if tsv.is_file() {
            fs::rename(&tsv, tsv.with_extension("tsv.migrated")).map_err(io_err)?;
        }
        Ok(true)
    }

    fn save(&self) -> Result<(), WorkspaceError> {
        self.workspace.ensure_layout()?;
        match self.backend {
            StateBackend::Sqlite => {
                let mut db = SqliteStateDb::open(&self.workspace.db_path())?;
                db.save(&self.state.to_snapshot())
            }
            StateBackend::Tsv => self.save_tsv(),
        }
    }

    fn save_tsv(&self) -> Result<(), WorkspaceError> {
        let mut out = String::new();
        writeln!(
            out,
            "# self-host state (ADR-009 Phase 1; Phase 2 → PROJ-11 SQLite)"
        )
        .map_err(io_err)?;
        writeln!(out, "meta\tnext_issue_num\t{}", self.state.next_issue_num).map_err(io_err)?;
        writeln!(out, "meta\tnext_run_num\t{}", self.state.next_run_num).map_err(io_err)?;
        writeln!(out, "meta\tnext_doc_num\t{}", self.state.next_doc_num).map_err(io_err)?;
        for issue in self.state.issues.values() {
            writeln!(
                out,
                "issue\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                issue.key,
                issue.issue_type,
                issue.priority,
                issue.status,
                escape_tsv(&issue.title),
                issue.product_id,
                issue.pipeline.as_deref().unwrap_or(""),
                escape_tsv(&issue.description),
            )
            .map_err(io_err)?;
        }
        for run in self.state.runs.values() {
            writeln!(
                out,
                "run\t{}\t{}\t{}\t{}\t{}",
                run.run_id, run.issue_key, run.pipeline_name, run.spec_id, run.spec_locked,
            )
            .map_err(io_err)?;
        }
        for doc in self.state.documents.values() {
            writeln!(
                out,
                "doc\t{}\t{}\t{}\t{}\t{}\t{}",
                doc.id,
                doc.doc_type,
                escape_tsv(&doc.title),
                doc.status,
                doc.version,
                doc.file_path,
            )
            .map_err(io_err)?;
        }
        atomic_write(&self.workspace.state_path(), &out)?;
        Ok(())
    }

    pub fn active_run_id(&self, issue_key: &str) -> Result<Option<String>, WorkspaceError> {
        for run in self.state.runs.values() {
            if run.issue_key != issue_key {
                continue;
            }
            let session = load_session(&self.workspace, &run.run_id)?;
            if session.run.status != PipelineRunStatus::RunCompleted {
                return Ok(Some(run.run_id.clone()));
            }
        }
        Ok(None)
    }

    pub fn run_ids_for_issue(&self, issue_key: &str) -> Vec<String> {
        self.state
            .runs
            .values()
            .filter(|run| run.issue_key == issue_key)
            .map(|run| run.run_id.clone())
            .collect()
    }
}

impl WorkspaceStore for LocalWorkspace {
    fn init(&mut self) -> Result<(), WorkspaceError> {
        self.workspace.ensure_layout()?;
        self.save()
    }

    fn create_issue(
        &mut self,
        issue_type: &str,
        title: &str,
        product_id: &str,
        pipeline: Option<&str>,
        priority: &str,
        description: &str,
    ) -> Result<IssueRow, WorkspaceError> {
        parse_issue_type(issue_type)?;
        let product_id =
            crate::workspace_readers::resolve_product_id(&self.workspace.root, product_id)?;
        let key = format!("PROJ-{}", self.state.next_issue_num);
        self.state.next_issue_num += 1;
        let row = IssueRow {
            key: key.clone(),
            issue_type: issue_type.into(),
            priority: priority.into(),
            status: "open".into(),
            title: title.into(),
            product_id: product_id.clone(),
            spec_id: product_id,
            pipeline: pipeline.map(str::to_string),
            description: description.into(),
        };
        self.state.issues.insert(key, row.clone());
        self.save()?;
        Ok(row)
    }

    fn list_issues(&self) -> Result<Vec<IssueRow>, WorkspaceError> {
        Ok(self.state.issues.values().cloned().collect())
    }

    fn get_issue(&self, key: &str) -> Result<IssueRow, WorkspaceError> {
        self.state
            .issues
            .get(key)
            .cloned()
            .ok_or_else(|| WorkspaceError::NotFound(key.into()))
    }

    fn close_issue(&mut self, key: &str) -> Result<IssueRow, WorkspaceError> {
        let _ = self.get_issue(key)?;
        if let Some(active) = self.active_run_id(key)? {
            return Err(WorkspaceError::InvalidState(format!(
                "active-run:{active}:complete all stages of the active run before closing"
            )));
        }
        let row = self
            .state
            .issues
            .get_mut(key)
            .ok_or_else(|| WorkspaceError::NotFound(key.into()))?;
        row.status = "done".into();
        let row = row.clone();
        self.save()?;
        Ok(row)
    }

    fn start_issue(
        &mut self,
        key: &str,
        spec_id: &str,
        pipeline: &str,
    ) -> Result<RunStartRow, WorkspaceError> {
        let issue = self.get_issue(key)?;
        if let Some(active) = self.active_run_id(key)? {
            return Err(WorkspaceError::InvalidState(format!(
                "active-run:{active}:complete or cancel the active run before starting another"
            )));
        }
        let lock_key = issue.product_id.clone();
        if !spec_id.is_empty() && spec_id != lock_key {
            return Err(WorkspaceError::InvalidState(format!(
                "product-lock:{}:{}:omit --spec/--product or pass the issue product",
                lock_key, spec_id
            )));
        }
        let resolved_spec = lock_key;
        let pipeline_name = if pipeline.is_empty() {
            issue
                .pipeline
                .clone()
                .or_else(|| {
                    issue_row_to_domain(&issue)
                        .resolved_pipeline()
                        .map(str::to_string)
                })
                .ok_or_else(|| WorkspaceError::InvalidState("no resolvable pipeline".into()))?
        } else {
            pipeline.to_string()
        };
        let pipeline_def = load_pipeline_def(&self.workspace, &pipeline_name)?;
        let run_id = alloc_run_id(&mut self.state.next_run_num);
        let mut session = PipelineSession::new_pending(&run_id, pipeline_def);
        session
            .start()
            .map_err(|e| WorkspaceError::InvalidState(format!("pipeline start failed: {e:?}")))?;
        save_session(&self.workspace, &session)?;
        self.state.runs.insert(
            run_id.clone(),
            RunRow {
                run_id: run_id.clone(),
                issue_key: key.to_string(),
                pipeline_name,
                spec_id: resolved_spec.clone(),
                spec_locked: true,
            },
        );
        if let Some(row) = self.state.issues.get_mut(key) {
            row.status = "in_progress".into();
            row.spec_id = resolved_spec;
        }
        self.save()?;
        Ok(RunStartRow {
            run_id,
            spec_locked: true,
        })
    }

    fn create_doc(
        &mut self,
        skill: &str,
        title: &str,
        run_id: &str,
    ) -> Result<DocCreateRow, WorkspaceError> {
        if !self.state.runs.contains_key(run_id) {
            return Err(WorkspaceError::NotFound(format!("run {run_id}")));
        }
        let doc_id = format!("doc-{}", self.state.next_doc_num);
        self.state.next_doc_num += 1;
        let mut doc = Document::new(&doc_id, skill, title);
        doc.extra_frontmatter
            .insert("pipeline_run_id".into(), run_id.to_string());
        if let Ok(cfg) = load_project_config(&self.workspace.root) {
            if cfg.workflow.inject_on_run {
                doc.extra_frontmatter
                    .insert("agent_context".into(), prompt_context_block(&cfg));
            }
        }
        doc.body = format!("# {title}\n");
        let rel_path = format!(".popsicle/artifacts/{run_id}/{doc_id}.{skill}.md");
        let abs_path = self.workspace.root.join(&rel_path);
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        fs::write(&abs_path, doc.to_file_content()).map_err(io_err)?;
        let row = DocumentRow::from_document(&doc, rel_path.clone());
        self.state.documents.insert(doc_id.clone(), row);
        self.save()?;
        Ok(DocCreateRow {
            doc_id,
            file_path: rel_path,
            artifact_file_exists: abs_path.is_file(),
        })
    }

    fn list_docs(&self, run_id: Option<&str>) -> Result<Vec<DocumentRow>, WorkspaceError> {
        Ok(self
            .state
            .documents
            .values()
            .filter(|d| {
                run_id.is_none_or(|rid| {
                    d.file_path
                        .starts_with(&format!(".popsicle/artifacts/{rid}/"))
                })
            })
            .cloned()
            .collect())
    }

    fn get_doc(&self, doc_id: &str) -> Result<DocumentRow, WorkspaceError> {
        self.state
            .documents
            .get(doc_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::NotFound(doc_id.into()))
    }

    fn check_doc(&self, doc_id: &str) -> Result<DocCheckRow, WorkspaceError> {
        let row = self.get_doc(doc_id)?;
        let abs_path = self.workspace.root.join(&row.file_path);
        let file_exists = abs_path.is_file();
        let content = if file_exists {
            fs::read_to_string(&abs_path).map_err(io_err)?
        } else {
            String::new()
        };

        // Frontmatter: a leading `---` block carrying at least id/doc_type/title.
        let mut frontmatter_complete = false;
        let mut body = content.as_str();
        if let Some(rest) = content.strip_prefix("---") {
            if let Some(end) = rest.find("\n---") {
                let frontmatter = &rest[..end];
                frontmatter_complete = ["id:", "doc_type:", "title:"]
                    .iter()
                    .all(|key| frontmatter.lines().any(|l| l.trim_start().starts_with(key)));
                body = rest[end + 4..].trim_start_matches('\n');
            }
        }

        // Body counts as filled when it has prose beyond the `# title` heading.
        let body_filled = body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .any(|l| !l.trim_start().starts_with('#'));

        let placeholder_count =
            (count_occurrences(body, "[TBD") + count_occurrences(body, "{{")) as u32;
        let checkboxes_checked =
            (count_occurrences(body, "- [x]") + count_occurrences(body, "- [X]")) as u32;
        let checkboxes_total = checkboxes_checked + count_occurrences(body, "- [ ]") as u32;

        let passed = file_exists && frontmatter_complete && body_filled && placeholder_count == 0;
        Ok(DocCheckRow {
            doc_id: row.id,
            file_path: row.file_path,
            file_exists,
            frontmatter_complete,
            body_filled,
            placeholder_count,
            checkboxes_total,
            checkboxes_checked,
            passed,
        })
    }

    fn pipeline_status(&self, run_id: &str) -> Result<PipelineStatusRow, WorkspaceError> {
        let session = load_session(&self.workspace, run_id)?;
        let snap = session.snapshot();
        let current_stage = snap.current_stage_name().unwrap_or("done").to_string();
        let stages = snap
            .stages
            .iter()
            .map(|s| {
                BTreeMap::from([
                    ("name".into(), s.name.clone()),
                    ("status".into(), stage_status_to_str(s.status).into()),
                ])
            })
            .collect();
        Ok(PipelineStatusRow {
            run_id: snap.run_id,
            pipeline_name: snap.pipeline_name,
            run_status: run_status_to_str(snap.run_status).into(),
            current_stage,
            current_stage_index: snap.current_stage_index,
            total_stages: snap.total_stages,
            stages,
        })
    }

    fn pipeline_next(&self, run_id: &str) -> Result<String, WorkspaceError> {
        let session = load_session(&self.workspace, run_id)?;
        let snap = session.snapshot();
        if snap.run_status == PipelineRunStatus::RunCompleted {
            return Ok("all stages completed".into());
        }
        let stage = snap.current_stage_name().unwrap_or("unknown");
        let current = snap
            .stages
            .get(snap.current_stage_index as usize)
            .ok_or_else(|| WorkspaceError::InvalidState("stage index out of range".into()))?;
        if current.status == StageStatus::StageInProgress {
            if current.requires_approval && current.approved_at == 0 {
                return Ok(format!(
                    "approve then `popsicle pipeline stage complete {stage} --run {run_id} --confirm`"
                ));
            }
            return Ok(format!(
                "popsicle pipeline stage complete {stage} --run {run_id}"
            ));
        }
        Ok(format!("popsicle pipeline status --run {run_id}"))
    }

    fn complete_stage(
        &mut self,
        stage: &str,
        run_id: &str,
        confirm: bool,
    ) -> Result<StageCompleteRow, WorkspaceError> {
        let mut session = load_session(&self.workspace, run_id)?;
        let snap = session.snapshot();
        let current_name = snap.current_stage_name().unwrap_or("");
        if current_name != stage {
            return Err(WorkspaceError::InvalidState(format!(
                "current stage is `{current_name}`, not `{stage}`"
            )));
        }
        let idx = session.run.current_stage_index as usize;
        let requires_approval = session
            .stages
            .get(idx)
            .map(|s| s.requires_approval)
            .unwrap_or(false);
        if requires_approval && !confirm {
            return Err(WorkspaceError::InvalidState(format!(
                "lock:{stage}:rerun `popsicle pipeline stage complete {stage} --run {run_id} --confirm`"
            )));
        }
        if requires_approval {
            session
                .approve_current(1)
                .map_err(|e| WorkspaceError::InvalidState(format!("approve failed: {e:?}")))?;
        }
        session
            .complete_current()
            .map_err(|e| WorkspaceError::InvalidState(format!("complete failed: {e:?}")))?;
        save_session(&self.workspace, &session)?;
        let snap = session.snapshot();
        Ok(StageCompleteRow {
            current_stage: snap.current_stage_name().unwrap_or("done").into(),
            downstream_ready: snap.run_status != PipelineRunStatus::RunCompleted,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryProvenance {
    pub executable_path: String,
    pub workspace_root: String,
    pub workspace_source: String,
    pub global_config_path: String,
    pub registered_projects: usize,
    pub package: String,
    pub build_source: String,
    pub expected_workspace_binary: String,
    /// The workspace builds its own `target/debug/popsicle` (popsicle-new dev repo).
    pub dev_workspace: bool,
    pub current_workspace_binary_match: bool,
    pub used_parent_binary: bool,
    pub used_system_binary: bool,
    pub used_local_bin: bool,
}

impl BinaryProvenance {
    /// In a dev workspace the running binary must be the workspace's own build.
    /// In a regular project (installed binary) any provenance is acceptable.
    pub fn is_trusted(&self) -> bool {
        !self.dev_workspace || self.current_workspace_binary_match
    }
}

pub fn binary_provenance() -> Result<BinaryProvenance, WorkspaceError> {
    let resolved = resolve_workspace_root(None)?;
    binary_provenance_for(&Workspace::at(resolved.root), resolved.source)
}

pub fn binary_provenance_for(
    workspace: &Workspace,
    source: WorkspaceSource,
) -> Result<BinaryProvenance, WorkspaceError> {
    let exe = std::env::current_exe().map_err(|e| io_err(e.to_string()))?;
    let expected = workspace.expected_binary();
    let exe_canon = fs::canonicalize(&exe).unwrap_or(exe.clone());
    let dev_workspace = expected.exists();
    let expected_canon = fs::canonicalize(&expected).unwrap_or(expected.clone());
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let used_system = home
        .as_ref()
        .map(|h| h.join(".cargo/bin/popsicle"))
        .and_then(|p| fs::canonicalize(p).ok())
        .is_some_and(|sys| sys == exe_canon);
    let used_local_bin = home
        .as_ref()
        .map(|h| h.join(".local/bin/popsicle"))
        .and_then(|p| fs::canonicalize(p).ok())
        .is_some_and(|local| local == exe_canon);
    let used_parent = workspace
        .root
        .parent()
        .map(|p| p.join("target/debug/popsicle"))
        .and_then(|p| fs::canonicalize(p).ok())
        .is_some_and(|parent| parent == exe_canon && parent != expected_canon);
    let global_cfg = global_config_path()?;
    let registered_projects = crate::global_config::load_global_config()
        .map(|c| c.projects.len())
        .unwrap_or(0);
    Ok(BinaryProvenance {
        executable_path: exe_canon.display().to_string(),
        workspace_root: workspace.root.display().to_string(),
        workspace_source: source.as_str().to_string(),
        global_config_path: global_cfg.display().to_string(),
        registered_projects,
        package: "cli-ux".into(),
        build_source: "popsicle-new/crates/cli-ux".into(),
        expected_workspace_binary: expected_canon.display().to_string(),
        dev_workspace,
        current_workspace_binary_match: exe_canon == expected_canon,
        used_parent_binary: used_parent,
        used_system_binary: used_system,
        used_local_bin,
    })
}

pub fn run_intent_validate(
    workspace: &Workspace,
    path: &str,
    format: &str,
) -> Result<i32, WorkspaceError> {
    // Resolve strictly inside the workspace. The old `root.parent()` lookup
    // predates the repo-root promotion (4d8b5c6) and could silently pick up an
    // unrelated sibling checkout — the same provenance bug class ADR-010 D-003
    // blocks for binaries.
    let tool_yaml = [
        workspace
            .root
            .join("intent-coder/tools/intent-validate/tool.yaml"),
        workspace
            .root
            .join(".popsicle/modules/intent-coder/tools/intent-validate/tool.yaml"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .ok_or_else(|| WorkspaceError::NotFound("intent-validate tool.yaml".into()))?;
    let content = fs::read_to_string(&tool_yaml).map_err(io_err)?;
    let command_block = content
        .split("command: |")
        .nth(1)
        .ok_or_else(|| WorkspaceError::InvalidState("tool.yaml missing command".into()))?
        .trim();
    let rendered = command_block
        .replace("{{path}}", path)
        .replace("{{format}}", format)
        .replace("{{include_asis}}", "");
    let status = ProcessCommand::new("sh")
        .arg("-c")
        .arg(&rendered)
        .current_dir(&workspace.root)
        .status()
        .map_err(|e| io_err(e.to_string()))?;
    Ok(status.code().unwrap_or(1))
}

/// Backend detection: an existing db wins; a legacy `state.tsv` keeps TSV;
/// fresh workspaces default to SQLite (ADR-009 Phase 2).
fn detect_backend(workspace: &Workspace) -> StateBackend {
    if workspace.db_path().is_file() {
        StateBackend::Sqlite
    } else if workspace.state_path().is_file() {
        StateBackend::Tsv
    } else {
        StateBackend::Sqlite
    }
}

fn load_state(workspace: &Workspace, backend: StateBackend) -> Result<StoreState, WorkspaceError> {
    workspace.ensure_layout()?;
    match backend {
        StateBackend::Sqlite => {
            if !workspace.db_path().is_file() {
                return Ok(StoreState::default());
            }
            let db = SqliteStateDb::open(&workspace.db_path())?;
            Ok(StoreState::from_snapshot(db.load()?))
        }
        StateBackend::Tsv => load_state_tsv(workspace),
    }
}

fn load_state_tsv(workspace: &Workspace) -> Result<StoreState, WorkspaceError> {
    let path = workspace.state_path();
    if !path.is_file() {
        return Ok(StoreState::default());
    }
    let content = fs::read_to_string(&path).map_err(io_err)?;
    let mut state = StoreState::default();
    for line in content.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        match cols.first().copied() {
            Some("meta") if cols.len() >= 3 => match cols[1] {
                "next_issue_num" => state.next_issue_num = cols[2].parse().unwrap_or(1),
                "next_run_num" => state.next_run_num = cols[2].parse().unwrap_or(1),
                "next_doc_num" => state.next_doc_num = cols[2].parse().unwrap_or(1),
                _ => {}
            },
            Some("issue") if cols.len() >= 9 => {
                let key = cols[1].to_string();
                let legacy = cols[6].to_string();
                state.issues.insert(
                    key.clone(),
                    IssueRow {
                        key,
                        issue_type: cols[2].into(),
                        priority: cols[3].into(),
                        status: cols[4].into(),
                        title: cols[5].into(),
                        product_id: legacy.clone(),
                        spec_id: legacy,
                        pipeline: if cols[7].is_empty() {
                            None
                        } else {
                            Some(cols[7].into())
                        },
                        description: cols[8].into(),
                    },
                );
            }
            Some("run") if cols.len() >= 6 => {
                let run_id = cols[1].to_string();
                state.runs.insert(
                    run_id.clone(),
                    RunRow {
                        run_id,
                        issue_key: cols[2].into(),
                        pipeline_name: cols[3].into(),
                        spec_id: cols[4].into(),
                        spec_locked: cols[5] == "true",
                    },
                );
            }
            Some("doc") if cols.len() >= 7 => {
                let id = cols[1].to_string();
                state.documents.insert(
                    id.clone(),
                    DocumentRow {
                        id,
                        doc_type: cols[2].into(),
                        title: cols[3].into(),
                        status: cols[4].into(),
                        version: cols[5].parse().unwrap_or(1),
                        parent_id: None,
                        file_path: cols[6].into(),
                        body: String::new(),
                    },
                );
            }
            _ => {}
        }
    }
    Ok(state)
}

fn escape_tsv(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

fn atomic_write(path: &PathBuf, content: &str) -> Result<(), WorkspaceError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).map_err(io_err)?;
    fs::rename(&tmp, path).map_err(io_err)?;
    Ok(())
}

fn io_err(e: impl ToString) -> WorkspaceError {
    WorkspaceError::Io(e.to_string())
}

fn parse_issue_type(raw: &str) -> Result<(), WorkspaceError> {
    match raw {
        "product" | "technical" | "bug" | "idea" => Ok(()),
        other => Err(WorkspaceError::InvalidState(format!(
            "unknown issue type: {other}"
        ))),
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

fn normalize_issue_rows(workspace: &Workspace, state: &mut StoreState) {
    for row in state.issues.values_mut() {
        crate::workspace_readers::backfill_issue_products(
            &workspace.root,
            &mut row.product_id,
            &mut row.spec_id,
        );
    }
}

fn issue_row_to_domain(row: &IssueRow) -> Issue {
    let issue_type = match row.issue_type.as_str() {
        "technical" => IssueType::Technical,
        "bug" => IssueType::Bug,
        "idea" => IssueType::Idea,
        _ => IssueType::Product,
    };
    Issue {
        key: row.key.clone(),
        title: row.title.clone(),
        description: row.description.clone(),
        issue_type,
        pipeline: row.pipeline.clone(),
        spec_id: row.product_id.clone(),
    }
}

/// Load a pipeline definition (used by CLI and Tauri UI).
pub(crate) fn load_pipeline_def(
    workspace: &Workspace,
    name: &str,
) -> Result<PipelineDef, WorkspaceError> {
    let dirs = [
        workspace.root.join(".popsicle/pipelines"),
        workspace
            .root
            .join(".popsicle/modules/intent-coder/pipelines"),
    ];
    for dir in &dirs {
        let path = dir.join(format!("{name}.pipeline.yaml"));
        if path.is_file() {
            return PipelineDef::load(&path).map_err(|e| WorkspaceError::Io(e.to_string()));
        }
    }
    // Self-heal: workspaces bootstrapped by older binaries miss newer bundled
    // templates; install on demand instead of failing.
    if let Some((_, content)) = BUNDLED_PIPELINES.iter().find(|(n, _)| *n == name) {
        let dir = workspace.pipelines_dir();
        fs::create_dir_all(&dir).map_err(io_err)?;
        let path = dir.join(format!("{name}.pipeline.yaml"));
        fs::write(&path, content).map_err(io_err)?;
        return PipelineDef::load(&path).map_err(|e| WorkspaceError::Io(e.to_string()));
    }
    let mut available: Vec<String> = dirs
        .iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()?
                .strip_suffix(".pipeline.yaml")
                .map(str::to_string)
        })
        .chain(bundled_pipeline_names().into_iter().map(str::to_string))
        .collect();
    available.sort();
    available.dedup();
    Err(WorkspaceError::NotFound(format!(
        "pipeline {name} (available: {})",
        if available.is_empty() {
            "none".to_string()
        } else {
            available.join(", ")
        }
    )))
}

fn session_path(workspace: &Workspace, run_id: &str) -> PathBuf {
    workspace.runs_dir().join(format!("{run_id}.json"))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSession {
    pipeline_name: String,
    run_id: String,
    run_status: String,
    current_stage_index: i64,
    total_stages: i64,
    stages: Vec<PersistedStage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedStage {
    name: String,
    status: String,
    requires_approval: bool,
    approved_at: i64,
}

fn save_session(workspace: &Workspace, session: &PipelineSession) -> Result<(), WorkspaceError> {
    let persisted = PersistedSession {
        pipeline_name: session.pipeline.name.clone(),
        run_id: session.run.id.clone(),
        run_status: run_status_to_str(session.run.status).into(),
        current_stage_index: session.run.current_stage_index,
        total_stages: session.run.total_stages,
        stages: session
            .stages
            .iter()
            .map(|s| PersistedStage {
                name: s.name.clone(),
                status: stage_status_to_str(s.status).into(),
                requires_approval: s.requires_approval,
                approved_at: s.approved_at,
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&persisted).map_err(|e| io_err(e.to_string()))?;
    fs::write(session_path(workspace, &session.run.id), json).map_err(io_err)?;
    Ok(())
}

/// Load a persisted pipeline session (used by CLI and Tauri UI).
pub(crate) fn load_session(
    workspace: &Workspace,
    run_id: &str,
) -> Result<PipelineSession, WorkspaceError> {
    let content = fs::read_to_string(session_path(workspace, run_id))
        .map_err(|_| WorkspaceError::NotFound(format!("run {run_id}")))?;
    let persisted: PersistedSession =
        serde_json::from_str(&content).map_err(|e| io_err(e.to_string()))?;
    let pipeline = load_pipeline_def(workspace, &persisted.pipeline_name)?;
    let mut session = PipelineSession::new_pending(&persisted.run_id, pipeline);
    if session.stages.len() != persisted.stages.len() {
        return Err(WorkspaceError::InvalidState(format!(
            "pipeline `{}` stage count changed since run `{}` was created",
            persisted.pipeline_name, persisted.run_id
        )));
    }
    session.run.status = run_status_from_str(&persisted.run_status);
    session.run.current_stage_index = persisted.current_stage_index;
    session.run.total_stages = persisted.total_stages;
    for (stage, saved) in session.stages.iter_mut().zip(persisted.stages.iter()) {
        stage.status = stage_status_from_str(&saved.status);
        stage.requires_approval = saved.requires_approval;
        stage.approved_at = saved.approved_at;
    }
    Ok(session)
}

fn alloc_run_id(counter: &mut u32) -> String {
    let n = *counter;
    *counter += 1;
    format!(
        "{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}",
        n,
        (n >> 8) & 0xffff,
        n & 0xfff,
        (n >> 4) & 0xfff,
        n as u64 * 0x1000000000001
    )
}

fn stage_status_to_str(s: StageStatus) -> &'static str {
    match s {
        StageStatus::StageBlocked => "blocked",
        StageStatus::StageReady => "ready",
        StageStatus::StageInProgress => "in_progress",
        StageStatus::StageCompleted => "completed",
        StageStatus::StageError => "error",
    }
}

fn stage_status_from_str(s: &str) -> StageStatus {
    match s {
        "ready" => StageStatus::StageReady,
        "in_progress" => StageStatus::StageInProgress,
        "completed" => StageStatus::StageCompleted,
        "error" => StageStatus::StageError,
        _ => StageStatus::StageBlocked,
    }
}

fn run_status_to_str(s: PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::RunPending => "pending",
        PipelineRunStatus::RunInProgress => "in_progress",
        PipelineRunStatus::RunCompleted => "completed",
        PipelineRunStatus::RunBlocked => "blocked",
    }
}

fn run_status_from_str(s: &str) -> PipelineRunStatus {
    match s {
        "in_progress" => PipelineRunStatus::RunInProgress,
        "completed" => PipelineRunStatus::RunCompleted,
        "blocked" => PipelineRunStatus::RunBlocked,
        _ => PipelineRunStatus::RunPending,
    }
}

fn ws_err(e: storage::WorkspaceError) -> crate::CliError {
    let msg = e.to_string();
    if let Some(rest) = msg.strip_prefix("invalid state: lock:") {
        let mut parts = rest.splitn(3, ':');
        let stage = parts.next().unwrap_or("stage");
        let next = parts.next().unwrap_or("add --confirm");
        return crate::CliError::actionable(
            "lock",
            stage,
            next,
            "stage requires explicit approval",
        );
    }
    if let Some(rest) = msg.strip_prefix("invalid state: active-run:") {
        let mut parts = rest.splitn(2, ':');
        let run_id = parts.next().unwrap_or("run");
        let next = parts.next().unwrap_or("complete the active pipeline run");
        return crate::CliError::actionable(
            "pipeline",
            run_id,
            next,
            "issue already has an active run",
        );
    }
    if let Some(rest) = msg.strip_prefix("invalid state: spec-lock:") {
        let mut parts = rest.splitn(3, ':');
        let issue_spec = parts.next().unwrap_or("issue-spec");
        let provided = parts.next().unwrap_or("provided-spec");
        let next = parts.next().unwrap_or("omit --spec or pass the issue spec");
        return crate::CliError::actionable(
            "invalid-args",
            provided,
            next,
            format!("issue spec is `{issue_spec}`"),
        );
    }
    let (category, next) = match &e {
        storage::WorkspaceError::NotFound(id) => ("not-found", format!("check `{id}` exists")),
        storage::WorkspaceError::InvalidState(_) => ("pipeline", msg.clone()),
        storage::WorkspaceError::Io(msg) => ("io", msg.clone()),
    };
    crate::CliError::actionable(category, "workspace", next, msg)
}

/// Binary entrypoint domain (ADR-009 Phase 1 TSV backend).
pub struct SelfHostDomain {
    store: LocalWorkspace,
}

impl SelfHostDomain {
    pub fn open() -> Result<Self, crate::CliError> {
        Self::open_with(None)
    }

    pub fn open_with(cli_project: Option<&str>) -> Result<Self, crate::CliError> {
        LocalWorkspace::open_with(cli_project)
            .map(|store| Self { store })
            .map_err(ws_err)
    }

    /// Open for `popsicle init`: when no `.popsicle/` exists anywhere up the
    /// tree, bootstrap the current directory as a new workspace root.
    pub fn open_or_bootstrap() -> Result<Self, crate::CliError> {
        Self::open_or_bootstrap_with(None)
    }

    pub fn open_or_bootstrap_with(cli_project: Option<&str>) -> Result<Self, crate::CliError> {
        let resolved = resolve_init_root(cli_project).map_err(ws_err)?;
        LocalWorkspace::open_resolved(resolved)
            .map(|store| Self { store })
            .map_err(ws_err)
    }

    pub fn workspace_root(&self) -> &std::path::Path {
        &self.store.workspace.root
    }

    pub fn workspace_source(&self) -> WorkspaceSource {
        self.store.workspace_source
    }
}

impl crate::CliDomain for SelfHostDomain {
    fn current_workspace(&self) -> Result<BTreeMap<String, String>, crate::CliError> {
        Ok(BTreeMap::from([
            (
                "workspace_root".into(),
                self.store.workspace.root.display().to_string(),
            ),
            (
                "workspace_source".into(),
                self.store.workspace_source.as_str().to_string(),
            ),
        ]))
    }
    fn init_workspace(&mut self) -> Result<crate::InitResult, crate::CliError> {
        self.store.init().map_err(ws_err)?;
        let installed = self
            .store
            .workspace
            .install_bundled_pipelines()
            .map_err(ws_err)?;
        let module = install_intent_coder_module(&self.store.workspace, false).map_err(ws_err)?;
        let project_cfg = ensure_project_config(&self.store.workspace.root).map_err(ws_err)?;
        let next_step = {
            let mut parts = Vec::new();
            if !installed.is_empty() {
                parts.push(format!("pipelines installed ({})", installed.join(", ")));
            }
            if module.installed {
                parts.push(format!(
                    "intent-coder module v{} → {}",
                    module.version.as_deref().unwrap_or("?"),
                    module.dest
                ));
            } else if let Some(reason) = &module.skipped_reason {
                parts.push(format!("intent-coder: {reason}"));
            }
            if parts.is_empty() {
                "popsicle issue create --type product --title \"<title>\" --spec <spec> --pipeline <pipeline>".to_string()
            } else {
                format!(
                    "{}; read intent-coder/guides/pipeline-selection.md before issue create",
                    parts.join("; ")
                )
            }
        };
        let next_with_config = if project_cfg.workflow.sync_agents_md {
            format!("{next_step}; project preferences synced to AGENTS.md")
        } else {
            next_step
        };
        Ok(crate::InitResult {
            workspace_ready: true,
            has_next_step: true,
            next_step: next_with_config,
        })
    }

    fn create_issue(
        &mut self,
        issue_type: &str,
        title: &str,
        product_id: &str,
        pipeline: Option<&str>,
        priority: &str,
        description: &str,
    ) -> Result<crate::IssueCreateResult, crate::CliError> {
        let row = self
            .store
            .create_issue(
                issue_type,
                title,
                product_id,
                pipeline,
                priority,
                description,
            )
            .map_err(ws_err)?;
        Ok(crate::IssueCreateResult {
            key: row.key,
            product_id: row.product_id,
            pipeline: row.pipeline,
        })
    }

    fn list_issues(&self) -> Result<Vec<BTreeMap<String, String>>, crate::CliError> {
        self.store
            .list_issues()
            .map(|rows| {
                rows.into_iter()
                    .map(|issue| {
                        BTreeMap::from([
                            ("key".into(), issue.key),
                            ("type".into(), issue.issue_type),
                            ("priority".into(), issue.priority),
                            ("status".into(), issue.status),
                            ("title".into(), issue.title),
                            ("product".into(), issue.product_id),
                        ])
                    })
                    .collect()
            })
            .map_err(ws_err)
    }

    fn show_issue(&self, key: &str) -> Result<BTreeMap<String, String>, crate::CliError> {
        let issue = self.store.get_issue(key).map_err(ws_err)?;
        let mut fields = BTreeMap::from([
            ("key".into(), issue.key),
            ("type".into(), issue.issue_type),
            ("priority".into(), issue.priority),
            ("status".into(), issue.status),
            ("title".into(), issue.title),
            ("product".into(), issue.product_id),
            ("description".into(), issue.description),
        ]);
        if let Some(p) = issue.pipeline {
            fields.insert("pipeline".into(), p);
        }
        let run_ids = self.store.run_ids_for_issue(key);
        fields.insert("run_count".into(), run_ids.len().to_string());
        if let Some(active) = self.store.active_run_id(key).map_err(ws_err)? {
            fields.insert("active_run_id".into(), active);
        }
        for (idx, run_id) in run_ids.iter().enumerate() {
            fields.insert(format!("run_{idx}_id"), run_id.clone());
        }
        Ok(fields)
    }

    fn close_issue(&mut self, key: &str) -> Result<BTreeMap<String, String>, crate::CliError> {
        let issue = self.store.close_issue(key).map_err(ws_err)?;
        Ok(BTreeMap::from([
            ("key".into(), issue.key),
            ("issue_status".into(), issue.status),
            ("title".into(), issue.title),
        ]))
    }

    fn start_issue(
        &mut self,
        key: &str,
        spec_id: &str,
        pipeline: &str,
    ) -> Result<crate::IssueStartResult, crate::CliError> {
        let run = self
            .store
            .start_issue(key, spec_id, pipeline)
            .map_err(ws_err)?;
        let agent_context = load_project_config(&self.store.workspace.root)
            .ok()
            .filter(|c| c.workflow.inject_on_run)
            .map(|c| prompt_context_block(&c))
            .unwrap_or_default();
        Ok(crate::IssueStartResult {
            run_created: true,
            spec_locked: run.spec_locked,
            has_run_id: !run.run_id.is_empty(),
            run_id: run.run_id,
            agent_context,
        })
    }

    fn create_doc(
        &mut self,
        skill: &str,
        title: &str,
        run_id: &str,
    ) -> Result<crate::DocCreateResult, crate::CliError> {
        let doc = self
            .store
            .create_doc(skill, title, run_id)
            .map_err(ws_err)?;
        Ok(crate::DocCreateResult {
            artifact_file_exists: doc.artifact_file_exists,
            document_row_exists: true,
            has_doc_id: true,
            doc_id: doc.doc_id,
            file_path: doc.file_path,
        })
    }

    fn list_docs(
        &self,
        run_id: Option<&str>,
    ) -> Result<Vec<BTreeMap<String, String>>, crate::CliError> {
        self.store
            .list_docs(run_id)
            .map(|rows| {
                rows.into_iter()
                    .map(|doc| {
                        BTreeMap::from([
                            ("id".into(), doc.id),
                            ("title".into(), doc.title),
                            ("doc_type".into(), doc.doc_type),
                            ("file_path".into(), doc.file_path),
                        ])
                    })
                    .collect()
            })
            .map_err(ws_err)
    }

    fn show_doc(&self, doc_id: &str) -> Result<BTreeMap<String, String>, crate::CliError> {
        let doc = self.store.get_doc(doc_id).map_err(ws_err)?;
        Ok(BTreeMap::from([
            ("id".into(), doc.id),
            ("title".into(), doc.title),
            ("doc_type".into(), doc.doc_type),
            ("status".into(), doc.status),
            ("file_path".into(), doc.file_path),
        ]))
    }

    fn check_doc(&self, doc_id: &str) -> Result<BTreeMap<String, String>, crate::CliError> {
        let check = self.store.check_doc(doc_id).map_err(ws_err)?;
        Ok(BTreeMap::from([
            ("doc_id".into(), check.doc_id),
            ("file_path".into(), check.file_path),
            ("file_exists".into(), check.file_exists.to_string()),
            (
                "frontmatter_complete".into(),
                check.frontmatter_complete.to_string(),
            ),
            ("body_filled".into(), check.body_filled.to_string()),
            (
                "placeholder_count".into(),
                check.placeholder_count.to_string(),
            ),
            (
                "checkboxes_total".into(),
                check.checkboxes_total.to_string(),
            ),
            (
                "checkboxes_checked".into(),
                check.checkboxes_checked.to_string(),
            ),
            ("passed".into(), check.passed.to_string()),
        ]))
    }

    fn pipeline_status(&self, run_id: &str) -> Result<BTreeMap<String, String>, crate::CliError> {
        let snap = self.store.pipeline_status(run_id).map_err(ws_err)?;
        let mut fields = BTreeMap::from([
            ("run_id".into(), snap.run_id),
            ("pipeline".into(), snap.pipeline_name),
            ("run_status".into(), snap.run_status),
            ("current_stage".into(), snap.current_stage),
            (
                "current_stage_index".into(),
                snap.current_stage_index.to_string(),
            ),
            ("total_stages".into(), snap.total_stages.to_string()),
        ]);
        for (idx, stage) in snap.stages.iter().enumerate() {
            if let Some(name) = stage.get("name") {
                fields.insert(format!("stage_{idx}_name"), name.clone());
            }
            if let Some(status) = stage.get("status") {
                fields.insert(format!("stage_{idx}_status"), status.clone());
            }
        }
        Ok(fields)
    }

    fn pipeline_next(&self, run_id: &str) -> Result<String, crate::CliError> {
        self.store.pipeline_next(run_id).map_err(ws_err)
    }

    fn complete_stage(
        &mut self,
        stage: &str,
        run_id: &str,
        confirm: bool,
    ) -> Result<crate::StageAdvanceResult, crate::CliError> {
        let result = self
            .store
            .complete_stage(stage, run_id, confirm)
            .map_err(ws_err)?;
        Ok(crate::StageAdvanceResult {
            previous_completed: true,
            downstream_ready: result.downstream_ready,
            status_reflects_state: true,
            current_stage: result.current_stage,
        })
    }

    fn doctor(
        &self,
        _format: crate::OutputFormat,
    ) -> Result<crate::CommandResponse, crate::CliError> {
        let prov = binary_provenance_for(&self.store.workspace, self.store.workspace_source)
            .map_err(ws_err)?;
        let trusted = prov.is_trusted();
        let next_step = if trusted {
            "popsicle issue list".to_string()
        } else {
            "cargo build -p cli-ux && ./target/debug/popsicle doctor".to_string()
        };
        let mut fields = BTreeMap::from([
            ("executable_path".into(), prov.executable_path),
            ("workspace_root".into(), prov.workspace_root),
            ("workspace_source".into(), prov.workspace_source),
            ("global_config_path".into(), prov.global_config_path),
            (
                "registered_projects".into(),
                prov.registered_projects.to_string(),
            ),
            ("package".into(), prov.package),
            ("build_source".into(), prov.build_source),
            (
                "expected_workspace_binary".into(),
                prov.expected_workspace_binary,
            ),
            ("dev_workspace".into(), prov.dev_workspace.to_string()),
            (
                "current_workspace_binary_match".into(),
                prov.current_workspace_binary_match.to_string(),
            ),
            (
                "used_parent_binary".into(),
                prov.used_parent_binary.to_string(),
            ),
            (
                "used_system_binary".into(),
                prov.used_system_binary.to_string(),
            ),
            ("used_local_bin".into(), prov.used_local_bin.to_string()),
            (
                "storage_backend".into(),
                self.store.backend().describe(&self.store.workspace),
            ),
            ("phase_2_issue".into(), "PROJ-11".into()),
            (
                "intent_coder_module".into(),
                intent_coder_module_version(&self.store.workspace)
                    .unwrap_or_else(|| "not installed".into()),
            ),
            (
                "intent_coder_bundle".into(),
                if self
                    .store
                    .workspace
                    .intent_coder_source()
                    .join("module.yaml")
                    .is_file()
                {
                    "workspace_root_override".into()
                } else {
                    "embedded".into()
                },
            ),
        ]);
        if let Ok(cfg) = load_project_config(&self.store.workspace.root) {
            fields.insert(
                "project_config_path".into(),
                project_config_path(&self.store.workspace.root)
                    .display()
                    .to_string(),
            );
            fields.insert(
                "agent_language".into(),
                cfg.agent.language.as_str().to_string(),
            );
            fields.insert("products_dir".into(), cfg.paths.products_dir.clone());
            if !cfg.paths.default_spec.is_empty() {
                fields.insert("default_spec".into(), cfg.paths.default_spec.clone());
            }
        }
        Ok(crate::CommandResponse {
            status: if trusted { "ok" } else { "warn" },
            next_step: Some(next_step),
            fields,
        })
    }

    fn tool_run(
        &self,
        tool: &str,
        args: &BTreeMap<String, String>,
    ) -> Result<i32, crate::CliError> {
        if tool != "intent-validate" {
            return Err(crate::CliError::actionable(
                "not-found",
                tool,
                "install tool under .popsicle/tools/",
                "unknown tool",
            ));
        }
        let path = args.get("path").ok_or_else(|| {
            crate::CliError::actionable(
                "invalid-args",
                "path",
                "pass path=products",
                "intent-validate requires path=",
            )
        })?;
        let format = args.get("format").map(String::as_str).unwrap_or("text");
        run_intent_validate(&self.store.workspace, path, format).map_err(ws_err)
    }

    fn admin_migrate(&mut self, workspace: &str) -> Result<crate::AdminResult, crate::CliError> {
        let migrated = self.store.migrate_to_sqlite().map_err(ws_err)?;
        let details = BTreeMap::from([
            ("migrated".into(), migrated.to_string()),
            (
                "storage_backend".into(),
                self.store.backend().describe(&self.store.workspace),
            ),
        ]);
        Ok(crate::AdminResult {
            under_admin_tree: true,
            explicit_workspace: !workspace.is_empty(),
            workspace: if workspace.is_empty() {
                self.store.workspace.root.display().to_string()
            } else {
                workspace.to_string()
            },
            details,
        })
    }

    fn admin_reinit(&mut self, workspace: &str) -> Result<crate::AdminResult, crate::CliError> {
        self.admin_migrate(workspace)
    }

    fn admin_sync_intent_coder(&mut self) -> Result<crate::AdminResult, crate::CliError> {
        let result = install_intent_coder_module(&self.store.workspace, true).map_err(ws_err)?;
        let mut details = BTreeMap::from([
            ("installed".into(), result.installed.to_string()),
            ("dest".into(), result.dest),
        ]);
        if let Some(v) = result.version {
            details.insert("version".into(), v);
        }
        if let Some(src) = result.source {
            details.insert("source".into(), src.as_str().into());
        }
        if let Some(reason) = result.skipped_reason {
            details.insert("skipped_reason".into(), reason);
        }
        Ok(crate::AdminResult {
            under_admin_tree: true,
            explicit_workspace: false,
            workspace: self.store.workspace.root.display().to_string(),
            details,
        })
    }

    fn admin_sync_project_config(&mut self) -> Result<crate::AdminResult, crate::CliError> {
        let root = &self.store.workspace.root;
        let config = load_project_config(root).map_err(ws_err)?;
        sync_agents_md(root, &config).map_err(ws_err)?;
        let details = BTreeMap::from([
            ("synced".into(), "true".into()),
            (
                "project_config_path".into(),
                project_config_path(root).display().to_string(),
            ),
            (
                "agent_language".into(),
                config.agent.language.as_str().to_string(),
            ),
            ("products_dir".into(), config.paths.products_dir),
        ]);
        Ok(crate::AdminResult {
            under_admin_tree: true,
            explicit_workspace: false,
            workspace: root.display().to_string(),
            details,
        })
    }
}
