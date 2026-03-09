use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    pub id: String,
    pub document_id: Option<String>,
    pub skill: String,
    pub pipeline_run_id: String,
    pub topic: String,
    pub status: DiscussionStatus,
    pub user_confidence: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub concluded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscussionStatus {
    Active,
    Concluded,
    Archived,
}

impl std::fmt::Display for DiscussionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Concluded => write!(f, "concluded"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for DiscussionStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "concluded" => Ok(Self::Concluded),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown discussion status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionMessage {
    pub id: String,
    pub discussion_id: String,
    pub phase: String,
    pub role_id: String,
    pub role_name: String,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    RoleStatement,
    UserInput,
    PausePoint,
    PhaseSummary,
    Decision,
    SystemNote,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoleStatement => write!(f, "role_statement"),
            Self::UserInput => write!(f, "user_input"),
            Self::PausePoint => write!(f, "pause_point"),
            Self::PhaseSummary => write!(f, "phase_summary"),
            Self::Decision => write!(f, "decision"),
            Self::SystemNote => write!(f, "system_note"),
        }
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "role_statement" => Ok(Self::RoleStatement),
            "user_input" => Ok(Self::UserInput),
            "pause_point" => Ok(Self::PausePoint),
            "phase_summary" => Ok(Self::PhaseSummary),
            "decision" => Ok(Self::Decision),
            "system_note" => Ok(Self::SystemNote),
            _ => Err(format!("Unknown message type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionRole {
    pub discussion_id: String,
    pub role_id: String,
    pub role_name: String,
    pub perspective: Option<String>,
    pub source: RoleSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleSource {
    Builtin,
    ProjectConfig,
    AutoDiscovered,
    UserDefined,
}

impl std::fmt::Display for RoleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin => write!(f, "builtin"),
            Self::ProjectConfig => write!(f, "project_config"),
            Self::AutoDiscovered => write!(f, "auto_discovered"),
            Self::UserDefined => write!(f, "user_defined"),
        }
    }
}

impl std::str::FromStr for RoleSource {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "builtin" => Ok(Self::Builtin),
            "project_config" => Ok(Self::ProjectConfig),
            "auto_discovered" => Ok(Self::AutoDiscovered),
            "user_defined" => Ok(Self::UserDefined),
            _ => Err(format!("Unknown role source: {s}")),
        }
    }
}

impl Discussion {
    pub fn new(
        skill: impl Into<String>,
        pipeline_run_id: impl Into<String>,
        topic: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            document_id: None,
            skill: skill.into(),
            pipeline_run_id: pipeline_run_id.into(),
            topic: topic.into(),
            status: DiscussionStatus::Active,
            user_confidence: None,
            created_at: Utc::now(),
            concluded_at: None,
        }
    }

    pub fn to_markdown(&self, roles: &[DiscussionRole], messages: &[DiscussionMessage]) -> String {
        let mut out = String::new();

        out.push_str("---\n");
        out.push_str(&format!("type: discussion\n"));
        out.push_str(&format!("discussion_id: \"{}\"\n", self.id));
        if let Some(ref doc_id) = self.document_id {
            out.push_str(&format!("document_id: \"{doc_id}\"\n"));
        }
        out.push_str(&format!("skill: {}\n", self.skill));
        out.push_str(&format!("topic: \"{}\"\n", self.topic));
        if let Some(c) = self.user_confidence {
            out.push_str(&format!("user_confidence: {c}\n"));
        }
        out.push_str(&format!("status: {}\n", self.status));
        out.push_str(&format!("created_at: \"{}\"\n", self.created_at.to_rfc3339()));
        if let Some(ref t) = self.concluded_at {
            out.push_str(&format!("concluded_at: \"{}\"\n", t.to_rfc3339()));
        }
        if !roles.is_empty() {
            out.push_str("roles:\n");
            for r in roles {
                out.push_str(&format!(
                    "  - {{ id: {}, name: {}, source: {} }}\n",
                    r.role_id, r.role_name, r.source
                ));
            }
        }
        out.push_str("---\n\n");

        let mut current_phase = String::new();
        for msg in messages {
            if msg.phase != current_phase {
                current_phase = msg.phase.clone();
                out.push_str(&format!("## {}\n\n", current_phase));
            }

            let time = msg.timestamp.format("%H:%M");
            match msg.message_type {
                MessageType::UserInput => {
                    out.push_str(&format!("**[User]** ({time}):\n> {}\n\n", msg.content));
                }
                MessageType::PausePoint => {
                    out.push_str(&format!(
                        "🎤 **Pause Point** ({time}):\n> {}\n\n",
                        msg.content
                    ));
                }
                MessageType::PhaseSummary => {
                    out.push_str(&format!("### Phase Summary\n{}\n\n", msg.content));
                }
                MessageType::Decision => {
                    out.push_str(&format!("### Decision\n{}\n\n", msg.content));
                }
                MessageType::SystemNote => {
                    out.push_str(&format!("_📋 {}_\n\n", msg.content));
                }
                MessageType::RoleStatement => {
                    out.push_str(&format!(
                        "**[{} - {}]** ({time}):\n> {}\n\n",
                        msg.role_id, msg.role_name, msg.content
                    ));
                }
            }
        }

        out
    }
}
