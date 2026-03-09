use anyhow::{Context, Result};
use clap::Subcommand;

use popsicle_core::dto::{
    DiscussionFull, DiscussionInfo, DiscussionMessageInfo, DiscussionRoleInfo,
};
use popsicle_core::model::{
    Discussion, DiscussionMessage, DiscussionRole, DiscussionStatus, MessageType, RoleSource,
};
use popsicle_core::storage::{IndexDb, ProjectLayout};

use crate::OutputFormat;

fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("msg-{:x}", ts)
}

#[derive(Subcommand)]
pub enum DiscussionCommand {
    /// Create a new discussion session
    Create {
        /// Skill this discussion belongs to (e.g., arch-debate)
        #[arg(long)]
        skill: String,
        /// Discussion topic
        #[arg(long)]
        topic: String,
        /// Pipeline run ID
        #[arg(long)]
        run: String,
        /// Link to an existing document ID
        #[arg(long)]
        doc: Option<String>,
        /// User confidence level (1-5)
        #[arg(long)]
        confidence: Option<i32>,
    },

    /// Add a message to a discussion
    Message {
        /// Discussion ID
        discussion_id: String,
        /// Role ID of the speaker (e.g., arch, sec, user)
        #[arg(long)]
        role: String,
        /// Role display name
        #[arg(long, default_value = "")]
        role_name: Option<String>,
        /// Discussion phase (e.g., "Phase 1: Problem Definition")
        #[arg(long)]
        phase: String,
        /// Message type
        #[arg(long, value_enum, default_value = "role-statement")]
        r#type: MessageTypeArg,
        /// Message content
        #[arg(long)]
        content: String,
        /// Reply to a specific message ID
        #[arg(long)]
        reply_to: Option<String>,
    },

    /// Add a role to a discussion
    Role {
        /// Discussion ID
        discussion_id: String,
        /// Role ID
        #[arg(long)]
        role_id: String,
        /// Role display name
        #[arg(long)]
        name: String,
        /// Role perspective/focus area
        #[arg(long)]
        perspective: Option<String>,
        /// Role source
        #[arg(long, value_enum, default_value = "builtin")]
        source: RoleSourceArg,
    },

    /// List discussions
    List {
        /// Filter by pipeline run
        #[arg(long)]
        run: Option<String>,
        /// Filter by skill
        #[arg(long)]
        skill: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },

    /// Show a discussion with all messages
    Show {
        /// Discussion ID
        discussion_id: String,
    },

    /// Conclude a discussion
    Conclude {
        /// Discussion ID
        discussion_id: String,
    },

    /// Export a discussion to Markdown file
    Export {
        /// Discussion ID
        discussion_id: String,
        /// Output file path (defaults to stdout)
        #[arg(long, short)]
        output: Option<String>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum MessageTypeArg {
    RoleStatement,
    UserInput,
    PausePoint,
    PhaseSummary,
    Decision,
    SystemNote,
}

impl From<MessageTypeArg> for MessageType {
    fn from(val: MessageTypeArg) -> Self {
        match val {
            MessageTypeArg::RoleStatement => MessageType::RoleStatement,
            MessageTypeArg::UserInput => MessageType::UserInput,
            MessageTypeArg::PausePoint => MessageType::PausePoint,
            MessageTypeArg::PhaseSummary => MessageType::PhaseSummary,
            MessageTypeArg::Decision => MessageType::Decision,
            MessageTypeArg::SystemNote => MessageType::SystemNote,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum RoleSourceArg {
    Builtin,
    ProjectConfig,
    AutoDiscovered,
    UserDefined,
}

impl From<RoleSourceArg> for RoleSource {
    fn from(val: RoleSourceArg) -> Self {
        match val {
            RoleSourceArg::Builtin => RoleSource::Builtin,
            RoleSourceArg::ProjectConfig => RoleSource::ProjectConfig,
            RoleSourceArg::AutoDiscovered => RoleSource::AutoDiscovered,
            RoleSourceArg::UserDefined => RoleSource::UserDefined,
        }
    }
}

pub fn execute(cmd: DiscussionCommand, format: &OutputFormat) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = ProjectLayout::new(&cwd);
    layout
        .ensure_initialized()
        .context("Project not initialized")?;
    let db = IndexDb::open(&layout.db_path())?;

    match cmd {
        DiscussionCommand::Create {
            skill,
            topic,
            run,
            doc,
            confidence,
        } => {
            let mut disc = Discussion::new(&skill, &run, &topic);
            disc.document_id = doc;
            disc.user_confidence = confidence;
            db.upsert_discussion(&disc)?;

            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "id": disc.id,
                            "status": "created"
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("Created discussion: {}", disc.id);
                    println!("  Skill: {}", disc.skill);
                    println!("  Topic: {}", disc.topic);
                    println!("  Run: {}", disc.pipeline_run_id);
                }
            }
        }

        DiscussionCommand::Message {
            discussion_id,
            role,
            role_name,
            phase,
            r#type,
            content,
            reply_to,
        } => {
            let disc = db
                .get_discussion(&discussion_id)?
                .context(format!("Discussion not found: {discussion_id}"))?;

            if disc.status != DiscussionStatus::Active {
                anyhow::bail!("Discussion is not active (status: {})", disc.status);
            }

            let display_name = role_name
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| role.clone());

            let msg = DiscussionMessage {
                id: new_id(),
                discussion_id: discussion_id.clone(),
                phase,
                role_id: role,
                role_name: display_name,
                content,
                message_type: r#type.into(),
                reply_to,
                timestamp: chrono::Utc::now(),
            };

            db.insert_discussion_message(&msg)?;

            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "id": msg.id,
                            "status": "recorded"
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("Message recorded: {}", msg.id);
                }
            }
        }

        DiscussionCommand::Role {
            discussion_id,
            role_id,
            name,
            perspective,
            source,
        } => {
            let role = DiscussionRole {
                discussion_id: discussion_id.clone(),
                role_id: role_id.clone(),
                role_name: name.clone(),
                perspective,
                source: source.into(),
            };
            db.upsert_discussion_role(&role)?;

            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "role_id": role_id,
                            "status": "added"
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("Role added: {} ({})", name, role_id);
                }
            }
        }

        DiscussionCommand::List { run, skill, status } => {
            let discussions =
                db.query_discussions(run.as_deref(), skill.as_deref(), status.as_deref())?;

            let infos: Vec<DiscussionInfo> = discussions
                .iter()
                .map(|d| {
                    let msg_count = db
                        .get_discussion_messages(&d.id)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    DiscussionInfo {
                        id: d.id.clone(),
                        document_id: d.document_id.clone(),
                        skill: d.skill.clone(),
                        pipeline_run_id: d.pipeline_run_id.clone(),
                        topic: d.topic.clone(),
                        status: d.status.to_string(),
                        user_confidence: d.user_confidence,
                        message_count: msg_count,
                        created_at: d.created_at.to_rfc3339(),
                        concluded_at: d.concluded_at.map(|t| t.to_rfc3339()),
                    }
                })
                .collect();

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&infos)?);
                }
                OutputFormat::Text => {
                    if infos.is_empty() {
                        println!("No discussions found.");
                    } else {
                        for info in &infos {
                            println!(
                                "[{}] {} — {} ({}, {} messages)",
                                info.status, info.skill, info.topic, info.id, info.message_count
                            );
                        }
                    }
                }
            }
        }

        DiscussionCommand::Show { discussion_id } => {
            let disc = db
                .get_discussion(&discussion_id)?
                .context(format!("Discussion not found: {discussion_id}"))?;
            let roles = db.get_discussion_roles(&discussion_id)?;
            let messages = db.get_discussion_messages(&discussion_id)?;

            let full = DiscussionFull {
                id: disc.id,
                document_id: disc.document_id,
                skill: disc.skill,
                pipeline_run_id: disc.pipeline_run_id,
                topic: disc.topic,
                status: disc.status.to_string(),
                user_confidence: disc.user_confidence,
                roles: roles
                    .iter()
                    .map(|r| DiscussionRoleInfo {
                        role_id: r.role_id.clone(),
                        role_name: r.role_name.clone(),
                        perspective: r.perspective.clone(),
                        source: r.source.to_string(),
                    })
                    .collect(),
                messages: messages
                    .iter()
                    .map(|m| DiscussionMessageInfo {
                        id: m.id.clone(),
                        phase: m.phase.clone(),
                        role_id: m.role_id.clone(),
                        role_name: m.role_name.clone(),
                        content: m.content.clone(),
                        message_type: m.message_type.to_string(),
                        reply_to: m.reply_to.clone(),
                        timestamp: m.timestamp.to_rfc3339(),
                    })
                    .collect(),
                created_at: disc.created_at.to_rfc3339(),
                concluded_at: disc.concluded_at.map(|t| t.to_rfc3339()),
            };

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&full)?);
                }
                OutputFormat::Text => {
                    println!("Discussion: {}", full.topic);
                    println!("  ID: {}", full.id);
                    println!("  Skill: {}", full.skill);
                    println!("  Status: {}", full.status);
                    if let Some(c) = full.user_confidence {
                        println!("  Confidence: {c}/5");
                    }
                    println!("  Roles: {}", full.roles.len());
                    println!("  Messages: {}", full.messages.len());
                }
            }
        }

        DiscussionCommand::Conclude { discussion_id } => {
            let mut disc = db
                .get_discussion(&discussion_id)?
                .context(format!("Discussion not found: {discussion_id}"))?;

            disc.status = DiscussionStatus::Concluded;
            disc.concluded_at = Some(chrono::Utc::now());
            db.upsert_discussion(&disc)?;

            let roles = db.get_discussion_roles(&discussion_id)?;
            let messages = db.get_discussion_messages(&discussion_id)?;
            let markdown = disc.to_markdown(&roles, &messages);

            let run_dir = layout.run_dir(&disc.pipeline_run_id);
            let slug = popsicle_core::helpers::slugify(&disc.topic);
            let export_path = run_dir.join(format!("{slug}.{}.discussion.md", disc.skill));
            if let Some(parent) = export_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&export_path, &markdown)?;

            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "id": disc.id,
                            "status": "concluded",
                            "exported_to": export_path.display().to_string()
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("Discussion concluded: {}", disc.id);
                    println!("  Exported to: {}", export_path.display());
                }
            }
        }

        DiscussionCommand::Export {
            discussion_id,
            output,
        } => {
            let disc = db
                .get_discussion(&discussion_id)?
                .context(format!("Discussion not found: {discussion_id}"))?;
            let roles = db.get_discussion_roles(&discussion_id)?;
            let messages = db.get_discussion_messages(&discussion_id)?;
            let markdown = disc.to_markdown(&roles, &messages);

            if let Some(path) = output {
                std::fs::write(&path, &markdown)?;
                match format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "exported_to": path
                            }))?
                        );
                    }
                    OutputFormat::Text => {
                        println!("Exported to: {path}");
                    }
                }
            } else {
                print!("{markdown}");
            }
        }
    }

    Ok(())
}
