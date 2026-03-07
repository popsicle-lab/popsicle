use std::path::Path;

use crate::error::Result;
use crate::model::SkillDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTarget {
    Claude,
    Cursor,
    Codex,
}

impl AgentTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            "codex" | "openai" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Codex => "codex",
        }
    }
}

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install(
        project_root: &Path,
        targets: &[AgentTarget],
        skills: &[&SkillDef],
    ) -> Result<Vec<String>> {
        let targets = if targets.is_empty() {
            vec![AgentTarget::Claude]
        } else {
            targets.to_vec()
        };

        let overview = build_overview(skills);
        let mut installed = Vec::new();

        for target in &targets {
            match target {
                AgentTarget::Claude => {
                    installed.extend(install_claude(project_root, skills, &overview)?);
                }
                AgentTarget::Cursor => {
                    installed.extend(install_cursor(project_root, skills, &overview)?);
                }
                AgentTarget::Codex => {
                    installed.extend(install_codex(project_root, skills, &overview)?);
                }
            }
        }

        Ok(installed)
    }
}

/// Build a per-skill command file: workflow info + CLI commands + guide content.
fn build_skill_command(skill: &SkillDef) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "Perform the \"{}\" step in the Popsicle pipeline.\n\n",
        skill.name
    ));

    // Workflow section (auto-generated)
    s.push_str("## Workflow\n\n");
    s.push_str(&format!("- **Initial state**: `{}`\n", skill.workflow.initial));
    let finals: Vec<&str> = skill
        .workflow
        .states
        .iter()
        .filter(|(_, sd)| sd.r#final)
        .map(|(n, _)| n.as_str())
        .collect();
    if !finals.is_empty() {
        s.push_str(&format!("- **Final state(s)**: `{}`\n", finals.join("`, `")));
    }
    s.push_str("- **Transitions**:\n");
    for (state, sd) in &skill.workflow.states {
        for t in &sd.transitions {
            let guard = t.guard.as_ref().map(|g| format!(" (guard: `{}`)", g)).unwrap_or_default();
            s.push_str(&format!("  - `{}` → `{}` via `{}`{}\n", state, t.to, t.action, guard));
        }
    }

    if !skill.inputs.is_empty() {
        s.push_str("\n## Inputs (upstream dependencies)\n\n");
        for input in &skill.inputs {
            s.push_str(&format!(
                "- `{}` from skill `{}` ({})\n",
                input.artifact_type,
                input.from_skill,
                if input.required { "required" } else { "optional" }
            ));
        }
    }

    // CLI commands section (auto-generated)
    s.push_str("\n## Commands\n\n");
    s.push_str("```bash\n");
    s.push_str("# Check if this skill is the current step\n");
    s.push_str("popsicle pipeline next --format json\n\n");
    s.push_str("# Create the document\n");
    s.push_str(&format!(
        "popsicle doc create {} --title \"<title>\" --run <run-id>\n\n",
        skill.name
    ));
    s.push_str("# View the created document\n");
    s.push_str("popsicle doc show <doc-id>\n\n");

    let non_final_states: Vec<(&String, &crate::model::skill::StateDef)> = skill
        .workflow
        .states
        .iter()
        .filter(|(_, sd)| !sd.r#final)
        .collect();
    for (state, sd) in &non_final_states {
        for t in &sd.transitions {
            s.push_str(&format!(
                "# From '{}': {}\n",
                state, t.action
            ));
            s.push_str(&format!(
                "popsicle doc transition <doc-id> {}\n",
                t.action
            ));
        }
    }
    s.push_str("```\n");

    // Writing guide (from guide.md — your core asset)
    if let Some(ref guide) = skill.guide {
        s.push_str("\n## Writing Guide\n\n");
        s.push_str(guide.trim());
        s.push_str("\n");
    }

    s
}

/// Build the overview section: skill catalog + pipeline info.
fn build_overview(skills: &[&SkillDef]) -> String {
    let mut s = String::from(
        r#"This project uses Popsicle for spec-driven development orchestration.

## Before Starting Any Task

```bash
popsicle pipeline next --format json
```

## Key Commands

- `popsicle pipeline next --format json` — what to do next (with CLI command + guide)
- `popsicle pipeline status` — current pipeline state
- `popsicle context --format json` — all documents for current run
- `popsicle doc create <skill> --title "<t>" --run <id>` — create document
- `popsicle doc transition <id> <action>` — advance workflow (guards enforced)
- `popsicle git link --doc <id> --stage <s>` — link commit to document

## Workflow Rules

1. Always check `popsicle pipeline next` before starting work
2. Guards enforce upstream document approval before downstream work proceeds
3. Fill document sections with real content — template placeholders are rejected
4. Link commits to documents with `popsicle git link`
"#,
    );

    if !skills.is_empty() {
        s.push_str("\n## Skill Catalog\n\n");
        s.push_str("| Skill | Artifact | Inputs | States |\n");
        s.push_str("|-------|----------|--------|--------|\n");
        for skill in skills {
            let artifact = skill.artifacts.first().map(|a| a.artifact_type.as_str()).unwrap_or("-");
            let inputs = if skill.inputs.is_empty() {
                "none".to_string()
            } else {
                skill.inputs.iter().map(|i| i.from_skill.as_str()).collect::<Vec<_>>().join(", ")
            };
            let states: Vec<&str> = skill.workflow.states.keys().map(|k| k.as_str()).collect();
            s.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                skill.name, artifact, inputs, states.join(" → ")
            ));
        }
    }

    s
}

fn install_claude(root: &Path, skills: &[&SkillDef], overview: &str) -> Result<Vec<String>> {
    let claude_dir = root.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let instructions = format!("# Popsicle — Claude Code Instructions\n\n{}\n", overview);
    std::fs::write(claude_dir.join("CLAUDE.md"), instructions)?;

    let commands_dir = claude_dir.join("commands");
    std::fs::create_dir_all(&commands_dir)?;

    let mut installed = vec![".claude/CLAUDE.md".to_string()];

    // Generate a command file per skill
    for skill in skills {
        let content = build_skill_command(skill);
        let filename = format!("{}.md", skill.name);
        std::fs::write(commands_dir.join(&filename), &content)?;
        installed.push(format!(".claude/commands/{}", filename));
    }

    // Meta commands
    std::fs::write(commands_dir.join("next.md"), SLASH_CMD_NEXT)?;
    installed.push(".claude/commands/next.md".to_string());

    Ok(installed)
}

fn install_cursor(root: &Path, skills: &[&SkillDef], overview: &str) -> Result<Vec<String>> {
    let rules_dir = root.join(".cursor").join("rules");
    std::fs::create_dir_all(&rules_dir)?;

    let rules = format!(
        "---\ndescription: Popsicle spec-driven development workflow\nglobs:\nalwaysApply: true\n---\n\n# Popsicle Workflow\n\n{}\n",
        overview
    );
    std::fs::write(rules_dir.join("popsicle.mdc"), rules)?;

    let agents_dir = root.join(".cursor").join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    // Build agent file with all skill commands embedded
    let mut agent = String::from(
        "---\nname: popsicle\ndescription: Popsicle spec-driven development assistant\n---\n\n",
    );
    agent.push_str(overview);
    agent.push('\n');
    for skill in skills {
        agent.push_str(&format!("\n---\n\n# Skill: {}\n\n", skill.name));
        agent.push_str(&build_skill_command(skill));
    }
    std::fs::write(agents_dir.join("popsicle.md"), agent)?;

    Ok(vec![
        ".cursor/rules/popsicle.mdc".into(),
        ".cursor/agents/popsicle.md".into(),
    ])
}

fn install_codex(root: &Path, skills: &[&SkillDef], overview: &str) -> Result<Vec<String>> {
    let mut content = format!("# Popsicle — Agent Instructions (Codex)\n\n{}\n", overview);
    for skill in skills {
        content.push_str(&format!("\n---\n\n# Skill: {}\n\n", skill.name));
        content.push_str(&build_skill_command(skill));
    }
    std::fs::write(root.join("AGENTS.md"), content)?;
    Ok(vec!["AGENTS.md".into()])
}

const SLASH_CMD_NEXT: &str = r#"Check what to do next in the Popsicle pipeline and follow the recommended action.

```bash
popsicle pipeline next --format json
```

Then execute the suggested CLI command. If a skill-specific command exists (e.g., `/domain-analysis`), use it for detailed guidance.
"#;
