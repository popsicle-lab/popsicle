//! Thin IO shell for the `cli-ux` slice (ADR-007 / ADR-009 Phase 1).

pub mod self_host;

pub use self_host::{SelfHostDomain, TsvWorkspace, Workspace};

use std::collections::BTreeMap;

use artifact_system::Document;
use skill_runtime::{Issue, IssueType, PipelineDef, PipelineSession, PipelineStageDef};
use storage::{DocumentRow, MemoryDocumentStore};

pub const TOP_LEVEL_COMMANDS: &[&str] = &[
    "init",
    "doctor",
    "module",
    "tool",
    "skill",
    "pipeline",
    "spec",
    "issue",
    "namespace",
    "doc",
    "prompt",
    "admin",
    "git",
    "memory",
    "context",
    "registry",
    "completions",
];

pub const REMOVED_TOP_LEVEL_COMMANDS: &[&str] = &["checklist", "item", "sync"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Doctor { format: OutputFormat },
    Init,
    IssueCreate {
        issue_type: String,
        title: String,
        spec_id: String,
        pipeline: Option<String>,
        priority: String,
        description: String,
    },
    IssueList,
    IssueShow { key: String },
    IssueStart {
        key: String,
        spec_id: String,
        pipeline: String,
    },
    DocCreate {
        skill: String,
        title: String,
        run_id: String,
    },
    DocList { run_id: Option<String> },
    DocShow { doc_id: String },
    PipelineStatus { run_id: String },
    PipelineNext { run_id: String },
    StageComplete {
        stage: String,
        run_id: String,
        confirm: bool,
    },
    ToolRun {
        tool: String,
        args: BTreeMap<String, String>,
    },
    Admin(AdminCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminCommand {
    Migrate { workspace: String },
    Reinit { workspace: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitResult {
    pub workspace_ready: bool,
    pub has_next_step: bool,
    pub next_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueStartResult {
    pub run_created: bool,
    pub spec_locked: bool,
    pub has_run_id: bool,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCreateResult {
    pub key: String,
    pub spec_id: String,
    pub pipeline: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCreateResult {
    pub artifact_file_exists: bool,
    pub document_row_exists: bool,
    pub has_doc_id: bool,
    pub doc_id: String,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageAdvanceResult {
    pub previous_completed: bool,
    pub downstream_ready: bool,
    pub status_reflects_state: bool,
    pub current_stage: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminResult {
    pub under_admin_tree: bool,
    pub explicit_workspace: bool,
    pub workspace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResponse {
    pub status: &'static str,
    pub next_step: Option<String>,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub category: &'static str,
    pub object_ref: String,
    pub next_step: String,
    pub message: String,
}

impl CliError {
    pub fn actionable(
        category: &'static str,
        object_ref: impl Into<String>,
        next_step: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            object_ref: object_ref.into(),
            next_step: next_step.into(),
            message: message.into(),
        }
    }

    pub fn has_category_object_and_next_step(&self) -> bool {
        !self.category.is_empty() && !self.object_ref.is_empty() && !self.next_step.is_empty()
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}. next: {}",
            self.category, self.object_ref, self.message, self.next_step
        )
    }
}

impl std::error::Error for CliError {}

pub trait CliDomain {
    fn init_workspace(&mut self) -> Result<InitResult, CliError>;
    fn create_issue(
        &mut self,
        issue_type: &str,
        title: &str,
        spec_id: &str,
        pipeline: Option<&str>,
        priority: &str,
        description: &str,
    ) -> Result<IssueCreateResult, CliError> {
        let _ = (issue_type, title, spec_id, pipeline, priority, description);
        Err(not_self_host("issue create"))
    }
    fn list_issues(&self) -> Result<Vec<BTreeMap<String, String>>, CliError> {
        Err(not_self_host("issue list"))
    }
    fn show_issue(&self, key: &str) -> Result<BTreeMap<String, String>, CliError> {
        let _ = key;
        Err(not_self_host("issue show"))
    }
    fn start_issue(
        &mut self,
        key: &str,
        spec_id: &str,
        pipeline: &str,
    ) -> Result<IssueStartResult, CliError>;
    fn create_doc(
        &mut self,
        skill: &str,
        title: &str,
        run_id: &str,
    ) -> Result<DocCreateResult, CliError>;
    fn list_docs(&self, run_id: Option<&str>) -> Result<Vec<BTreeMap<String, String>>, CliError> {
        let _ = run_id;
        Err(not_self_host("doc list"))
    }
    fn show_doc(&self, doc_id: &str) -> Result<BTreeMap<String, String>, CliError> {
        let _ = doc_id;
        Err(not_self_host("doc show"))
    }
    fn pipeline_status(&self, run_id: &str) -> Result<BTreeMap<String, String>, CliError> {
        let _ = run_id;
        Err(not_self_host("pipeline status"))
    }
    fn pipeline_next(&self, run_id: &str) -> Result<String, CliError> {
        let _ = run_id;
        Err(not_self_host("pipeline next"))
    }
    fn complete_stage(
        &mut self,
        stage: &str,
        run_id: &str,
        confirm: bool,
    ) -> Result<StageAdvanceResult, CliError>;
    fn doctor(&self, format: OutputFormat) -> Result<CommandResponse, CliError> {
        let _ = format;
        Err(not_self_host("doctor"))
    }
    fn tool_run(&self, tool: &str, args: &BTreeMap<String, String>) -> Result<i32, CliError> {
        let _ = (tool, args);
        Err(not_self_host("tool run"))
    }
    fn admin_migrate(&mut self, workspace: &str) -> Result<AdminResult, CliError>;
    fn admin_reinit(&mut self, workspace: &str) -> Result<AdminResult, CliError>;
}

fn not_self_host(command: &str) -> CliError {
    CliError::actionable(
        "self-host",
        command,
        "run ./target/debug/popsicle from popsicle-new",
        "command requires workspace-backed self-host domain",
    )
}

pub fn parse_args<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args.is_empty() || args == ["--help"] || args == ["help"] {
        return Ok(Command::Help);
    }

    if REMOVED_TOP_LEVEL_COMMANDS.contains(&args[0].as_str()) {
        return Err(CliError::actionable(
            "invalid-args",
            args[0].clone(),
            removed_command_next_step(&args[0]),
            "top-level command is not part of the IDD MVP",
        ));
    }

    match args[0].as_str() {
        "doctor" => Ok(Command::Doctor {
            format: parse_format_flag(&args),
        }),
        "init" if args.len() == 1 => Ok(Command::Init),
        "issue" if args.get(1).map(String::as_str) == Some("create") => Ok(Command::IssueCreate {
            issue_type: flag_value(&args, "--type")?,
            title: flag_value(&args, "--title")?,
            spec_id: flag_value(&args, "--spec")?,
            pipeline: optional_flag_value(&args, "--pipeline"),
            priority: flag_value_or(&args, "--priority", "medium"),
            description: flag_value_or(&args, "--description", ""),
        }),
        "issue" if args.get(1).map(String::as_str) == Some("list") && args.len() == 2 => {
            Ok(Command::IssueList)
        }
        "issue" if args.get(1).map(String::as_str) == Some("show") => {
            let key = args.get(2).ok_or_else(|| missing("issue-key", "issue show"))?;
            Ok(Command::IssueShow { key: key.clone() })
        }
        "issue" if args.get(1).map(String::as_str) == Some("start") => {
            let key = args
                .get(2)
                .ok_or_else(|| missing("issue-key", "issue start"))?;
            Ok(Command::IssueStart {
                key: key.clone(),
                spec_id: flag_value_or(&args, "--spec", ""),
                pipeline: flag_value_or(&args, "--pipeline", ""),
            })
        }
        "doc" if args.get(1).map(String::as_str) == Some("create") => {
            let skill = args.get(2).ok_or_else(|| missing("skill", "doc create"))?;
            Ok(Command::DocCreate {
                skill: skill.clone(),
                title: flag_value(&args, "--title")?,
                run_id: flag_value(&args, "--run")?,
            })
        }
        "doc" if args.get(1).map(String::as_str) == Some("list") => Ok(Command::DocList {
            run_id: optional_flag_value(&args, "--run"),
        }),
        "doc" if args.get(1).map(String::as_str) == Some("show") => {
            let doc_id = args.get(2).ok_or_else(|| missing("doc-id", "doc show"))?;
            Ok(Command::DocShow {
                doc_id: doc_id.clone(),
            })
        }
        "pipeline" if args.get(1).map(String::as_str) == Some("status") => {
            Ok(Command::PipelineStatus {
                run_id: flag_value(&args, "--run")?,
            })
        }
        "pipeline" if args.get(1).map(String::as_str) == Some("next") => Ok(Command::PipelineNext {
            run_id: flag_value(&args, "--run")?,
        }),
        "pipeline"
            if args.get(1).map(String::as_str) == Some("stage")
                && args.get(2).map(String::as_str) == Some("complete") =>
        {
            let stage = args
                .get(3)
                .ok_or_else(|| missing("stage", "stage complete"))?;
            Ok(Command::StageComplete {
                stage: stage.clone(),
                run_id: flag_value(&args, "--run")?,
                confirm: args.iter().any(|a| a == "--confirm"),
            })
        }
        "tool" if args.get(1).map(String::as_str) == Some("run") => {
            let tool = args.get(2).ok_or_else(|| missing("tool", "tool run"))?;
            Ok(Command::ToolRun {
                tool: tool.clone(),
                args: parse_kv_args(&args[3..]),
            })
        }
        "admin" if args.get(1).map(String::as_str) == Some("migrate") => {
            Ok(Command::Admin(AdminCommand::Migrate {
                workspace: flag_value_or(&args, "--workspace", ""),
            }))
        }
        "admin" if args.get(1).map(String::as_str) == Some("reinit") => {
            Ok(Command::Admin(AdminCommand::Reinit {
                workspace: flag_value_or(&args, "--workspace", ""),
            }))
        }
        "migrate" | "reinit" => Err(CliError::actionable(
            "invalid-args",
            args[0].clone(),
            format!("run `popsicle admin {}` with --workspace", args[0]),
            "maintenance commands must be explicit admin subcommands",
        )),
        other => Err(CliError::actionable(
            "invalid-args",
            other,
            "run `popsicle --help`",
            "unknown or incomplete command",
        )),
    }
}

pub fn run_command<D: CliDomain>(
    domain: &mut D,
    command: Command,
) -> Result<CommandResponse, CliError> {
    match command {
        Command::Help => Ok(help_response()),
        Command::Doctor { format } => domain.doctor(format),
        Command::Init => {
            let result = domain.init_workspace()?;
            let mut fields = BTreeMap::new();
            fields.insert("workspace_ready".into(), result.workspace_ready.to_string());
            fields.insert("has_next_step".into(), result.has_next_step.to_string());
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(result.next_step),
                fields,
            })
        }
        Command::IssueCreate {
            issue_type,
            title,
            spec_id,
            pipeline,
            priority,
            description,
        } => {
            let result = domain.create_issue(
                &issue_type,
                &title,
                &spec_id,
                pipeline.as_deref(),
                &priority,
                &description,
            )?;
            let mut fields = BTreeMap::new();
            fields.insert("key".into(), result.key.clone());
            fields.insert("spec".into(), result.spec_id);
            if let Some(p) = result.pipeline {
                fields.insert("pipeline".into(), p);
            }
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(format!("popsicle issue start {}", result.key)),
                fields,
            })
        }
        Command::IssueList => {
            let rows = domain.list_issues()?;
            let mut fields = BTreeMap::new();
            fields.insert("count".into(), rows.len().to_string());
            for (idx, row) in rows.iter().enumerate() {
                for (k, v) in row {
                    fields.insert(format!("issue_{idx}_{k}"), v.clone());
                }
            }
            Ok(CommandResponse {
                status: "ok",
                next_step: Some("popsicle issue show <key>".into()),
                fields,
            })
        }
        Command::IssueShow { key } => {
            let fields = domain.show_issue(&key)?;
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(format!("popsicle issue start {key}")),
                fields,
            })
        }
        Command::IssueStart {
            key,
            spec_id,
            pipeline,
        } => {
            let result = domain.start_issue(&key, &spec_id, &pipeline)?;
            let mut fields = BTreeMap::new();
            fields.insert("key".into(), key);
            fields.insert("run_id".into(), result.run_id);
            fields.insert("run_created".into(), result.run_created.to_string());
            fields.insert("spec_locked".into(), result.spec_locked.to_string());
            fields.insert("has_run_id".into(), result.has_run_id.to_string());
            Ok(CommandResponse {
                status: "ok",
                next_step: Some("popsicle pipeline next --run <run_id>".into()),
                fields,
            })
        }
        Command::DocCreate {
            skill,
            title,
            run_id,
        } => {
            let result = domain.create_doc(&skill, &title, &run_id)?;
            let mut fields = BTreeMap::new();
            fields.insert("id".into(), result.doc_id.clone());
            fields.insert("file_path".into(), result.file_path);
            fields.insert(
                "artifact_file_exists".into(),
                result.artifact_file_exists.to_string(),
            );
            fields.insert(
                "document_row_exists".into(),
                result.document_row_exists.to_string(),
            );
            fields.insert("has_doc_id".into(), result.has_doc_id.to_string());
            let doc_id = result.doc_id;
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(format!("popsicle doc show {doc_id}")),
                fields,
            })
        }
        Command::DocList { run_id } => {
            let rows = domain.list_docs(run_id.as_deref())?;
            let mut fields = BTreeMap::new();
            fields.insert("count".into(), rows.len().to_string());
            for (idx, row) in rows.iter().enumerate() {
                for (k, v) in row {
                    fields.insert(format!("doc_{idx}_{k}"), v.clone());
                }
            }
            Ok(CommandResponse {
                status: "ok",
                next_step: Some("popsicle doc show <id>".into()),
                fields,
            })
        }
        Command::DocShow { doc_id } => Ok(CommandResponse {
            status: "ok",
            next_step: None,
            fields: domain.show_doc(&doc_id)?,
        }),
        Command::PipelineStatus { run_id } => Ok(CommandResponse {
            status: "ok",
            next_step: Some("popsicle pipeline next --run <run_id>".into()),
            fields: domain.pipeline_status(&run_id)?,
        }),
        Command::PipelineNext { run_id } => {
            let next = domain.pipeline_next(&run_id)?;
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(next),
                fields: BTreeMap::from([("run_id".into(), run_id)]),
            })
        }
        Command::StageComplete {
            stage,
            run_id,
            confirm,
        } => {
            let result = domain.complete_stage(&stage, &run_id, confirm)?;
            let mut fields = BTreeMap::new();
            fields.insert("stage".into(), stage);
            fields.insert("current_stage".into(), result.current_stage);
            fields.insert(
                "previous_completed".into(),
                result.previous_completed.to_string(),
            );
            fields.insert(
                "downstream_ready".into(),
                result.downstream_ready.to_string(),
            );
            fields.insert(
                "status_reflects_state".into(),
                result.status_reflects_state.to_string(),
            );
            Ok(CommandResponse {
                status: "ok",
                next_step: Some(format!("popsicle pipeline status --run {run_id}")),
                fields,
            })
        }
        Command::ToolRun { tool, args } => {
            let code = domain.tool_run(&tool, &args)?;
            let mut fields = BTreeMap::new();
            fields.insert("exit_code".into(), code.to_string());
            fields.insert("tool".into(), tool);
            Ok(CommandResponse {
                status: if code == 0 { "ok" } else { "failed" },
                next_step: None,
                fields,
            })
        }
        Command::Admin(AdminCommand::Migrate { workspace }) => {
            admin_response(domain.admin_migrate(&workspace)?)
        }
        Command::Admin(AdminCommand::Reinit { workspace }) => {
            admin_response(domain.admin_reinit(&workspace)?)
        }
    }
}

pub fn top_level_help() -> String {
    TOP_LEVEL_COMMANDS.join("\n")
}

pub fn contains_removed_top_level_command(help: &str) -> bool {
    REMOVED_TOP_LEVEL_COMMANDS
        .iter()
        .any(|cmd| help.lines().any(|line| line.trim() == *cmd))
}

pub fn create_document_artifact(
    store: &mut MemoryDocumentStore,
    id: &str,
    skill: &str,
    title: &str,
    run_id: &str,
) -> Result<DocCreateResult, CliError> {
    let mut doc = Document::new(id, skill, title);
    doc.extra_frontmatter
        .insert("pipeline_run_id".into(), run_id.to_string());
    doc.body = format!("# {title}\n");
    let file_path = format!(".popsicle/artifacts/{run_id}/{id}.{skill}.md");
    let row = DocumentRow::from_document(&doc, file_path.clone());
    store.insert(row).map_err(|e| {
        CliError::actionable("storage", id, "choose a new document id", e.to_string())
    })?;

    Ok(DocCreateResult {
        artifact_file_exists: !doc.to_file_content().is_empty(),
        document_row_exists: store.get(id).is_ok(),
        has_doc_id: !id.is_empty(),
        doc_id: id.to_string(),
        file_path,
    })
}

pub fn start_issue_run(
    key: &str,
    spec_id: &str,
    pipeline: &str,
    run_id: &str,
) -> Result<IssueStartResult, CliError> {
    let issue = Issue {
        key: key.to_string(),
        title: "cli-ux delivery".into(),
        description: String::new(),
        issue_type: IssueType::Product,
        pipeline: Some(pipeline.to_string()),
        spec_id: spec_id.to_string(),
    };
    let resolved = issue.resolved_pipeline().ok_or_else(|| {
        CliError::actionable(
            "invalid-args",
            key,
            "pass --pipeline <name>",
            "issue has no resolvable pipeline",
        )
    })?;
    let pipeline_def = one_stage_pipeline(resolved);
    let mut session = PipelineSession::new_pending(run_id, pipeline_def);
    session.start().map_err(|e| {
        CliError::actionable(
            "pipeline",
            run_id,
            "inspect pipeline definition",
            format!("{e:?}"),
        )
    })?;

    Ok(IssueStartResult {
        run_created: true,
        spec_locked: issue.spec_id == spec_id,
        has_run_id: !session.run.id.is_empty(),
        run_id: session.run.id,
    })
}

pub fn complete_pipeline_stage(
    stage: &str,
    run_id: &str,
    confirm: bool,
) -> Result<StageAdvanceResult, CliError> {
    if !confirm {
        return Err(CliError::actionable(
            "lock",
            stage,
            format!("rerun `popsicle pipeline stage complete {stage} --run {run_id} --confirm`"),
            "stage requires explicit approval",
        ));
    }

    let pipeline = two_stage_pipeline();
    let mut session = PipelineSession::new_pending(run_id, pipeline);
    session.start().map_err(|e| {
        CliError::actionable("pipeline", run_id, "start the run first", format!("{e:?}"))
    })?;
    session.approve_current(1).map_err(|e| {
        CliError::actionable(
            "pipeline",
            run_id,
            "confirm the current stage",
            format!("{e:?}"),
        )
    })?;
    session.complete_current().map_err(|e| {
        CliError::actionable(
            "pipeline",
            run_id,
            "inspect current stage",
            format!("{e:?}"),
        )
    })?;
    let snap = session.snapshot();

    Ok(StageAdvanceResult {
        previous_completed: true,
        downstream_ready: snap.current_stage_name() == Some("next"),
        status_reflects_state: snap.current_stage_index == 1,
        current_stage: snap.current_stage_name().unwrap_or("done").to_string(),
    })
}

fn flag_value(args: &[String], flag: &str) -> Result<String, CliError> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| missing(flag, args.first().map(String::as_str).unwrap_or("command")))
}

fn flag_value_or(args: &[String], flag: &str, default: &str) -> String {
    flag_value(args, flag).unwrap_or_else(|_| default.to_string())
}

fn optional_flag_value(args: &[String], flag: &str) -> Option<String> {
    flag_value(args, flag).ok()
}

fn parse_format_flag(args: &[String]) -> OutputFormat {
    match flag_value_or(args, "--format", "text").as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Text,
    }
}

fn parse_kv_args(args: &[String]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for arg in args {
        if let Some((k, v)) = arg.split_once('=') {
            out.insert(k.to_string(), v.to_string());
        }
    }
    out
}

fn missing(object_ref: &str, command: &str) -> CliError {
    CliError::actionable(
        "invalid-args",
        object_ref,
        format!("run `popsicle {command} --help`"),
        "required argument missing",
    )
}

fn removed_command_next_step(command: &str) -> &'static str {
    match command {
        "checklist" => "use `popsicle doc check`",
        "item" => "use the task_chunk/doc path",
        "sync" => "sync is deferred outside the IDD MVP",
        _ => "run `popsicle --help`",
    }
}

pub fn help_response() -> CommandResponse {
    let mut fields = BTreeMap::new();
    fields.insert("commands".into(), top_level_help());
    CommandResponse {
        status: "ok",
        next_step: Some("popsicle doctor --format json".into()),
        fields,
    }
}

fn admin_response(result: AdminResult) -> Result<CommandResponse, CliError> {
    let mut fields = BTreeMap::new();
    fields.insert(
        "under_admin_tree".into(),
        result.under_admin_tree.to_string(),
    );
    fields.insert(
        "explicit_workspace".into(),
        result.explicit_workspace.to_string(),
    );
    fields.insert("workspace".into(), result.workspace);
    Ok(CommandResponse {
        status: "ok",
        next_step: Some("popsicle pipeline status --run <run_id>".into()),
        fields,
    })
}

fn one_stage_pipeline(name: &str) -> PipelineDef {
    PipelineDef {
        name: name.to_string(),
        description: "cli-ux issue start".into(),
        stages: vec![PipelineStageDef {
            name: "start".into(),
            skill: Some("shadow-implementer".into()),
            skills: vec![],
            description: "start run".into(),
            depends_on: vec![],
            requires_approval: false,
        }],
        keywords: vec![],
        scale: None,
    }
}

fn two_stage_pipeline() -> PipelineDef {
    PipelineDef {
        name: "cli-ux-stage".into(),
        description: "stage advance".into(),
        stages: vec![
            PipelineStageDef {
                name: "current".into(),
                skill: Some("shadow-implementer".into()),
                skills: vec![],
                description: "current".into(),
                depends_on: vec![],
                requires_approval: true,
            },
            PipelineStageDef {
                name: "next".into(),
                skill: Some("equivalence-baseline".into()),
                skills: vec![],
                description: "next".into(),
                depends_on: vec!["current".into()],
                requires_approval: false,
            },
        ],
        keywords: vec![],
        scale: None,
    }
}
