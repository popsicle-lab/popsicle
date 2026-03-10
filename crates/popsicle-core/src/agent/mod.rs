use std::path::Path;

use crate::error::Result;
use crate::model::SkillDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTarget {
    Claude,
    Cursor,
}

impl AgentTarget {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Cursor => "cursor",
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
    s.push_str(&format!(
        "- **Initial state**: `{}`\n",
        skill.workflow.initial
    ));
    let finals: Vec<&str> = skill
        .workflow
        .states
        .iter()
        .filter(|(_, sd)| sd.r#final)
        .map(|(n, _)| n.as_str())
        .collect();
    if !finals.is_empty() {
        s.push_str(&format!(
            "- **Final state(s)**: `{}`\n",
            finals.join("`, `")
        ));
    }
    s.push_str("- **Transitions**:\n");
    for (state, sd) in &skill.workflow.states {
        for t in &sd.transitions {
            let guard = t
                .guard
                .as_ref()
                .map(|g| format!(" (guard: `{}`)", g))
                .unwrap_or_default();
            let approval = if t.requires_approval {
                " **⚠ requires human approval**"
            } else {
                ""
            };
            s.push_str(&format!(
                "  - `{}` → `{}` via `{}`{}{}\n",
                state, t.to, t.action, guard, approval
            ));
        }
    }

    if !skill.inputs.is_empty() {
        s.push_str("\n## Inputs (upstream dependencies)\n\n");
        for input in &skill.inputs {
            s.push_str(&format!(
                "- `{}` from skill `{}` ({})\n",
                input.artifact_type,
                input.from_skill,
                if input.required {
                    "required"
                } else {
                    "optional"
                }
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
            if t.requires_approval {
                s.push_str(&format!(
                    "# From '{}': {} (⚠ requires human approval)\n",
                    state, t.action
                ));
                s.push_str(
                    "# 禁止代用户执行。必须由用户本人审阅/参与讨论后，由用户自己在终端执行：\n",
                );
                s.push_str(&format!(
                    "popsicle doc transition <doc-id> {} --confirm\n",
                    t.action
                ));
            } else {
                s.push_str(&format!("# From '{}': {}\n", state, t.action));
                s.push_str(&format!("popsicle doc transition <doc-id> {}\n", t.action));
            }
        }
    }
    s.push_str("```\n");

    // Writing guide (from guide.md — your core asset)
    if let Some(ref guide) = skill.guide {
        s.push_str("\n## Writing Guide\n\n");
        s.push_str(guide.trim());
        s.push('\n');
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

## Issue Tracking

- `popsicle issue list --format json` — list all issues
- `popsicle issue show <key> --format json` — show issue details
- `popsicle issue start <key>` — start the workflow for an issue

When the user says "start PROJ-1" or "release requirement PROJ-1":
1. Run `popsicle issue start <key>` to create the pipeline run
2. Then run `popsicle pipeline next --format json` to get the first step

## Workflow Rules

1. Always check `popsicle pipeline next` before starting work
2. Guards enforce upstream document approval before downstream work proceeds
3. Fill document sections with real content — template placeholders are rejected
4. Link commits to documents with `popsicle git link`
5. Transitions marked `requires_approval` (e.g. discussion conclude, doc approve): do NOT run the command with `--confirm` for the user. You must STOP, show the user the suggested command, and ask them to review and run it themselves in their terminal. No exception.
"#,
    );

    if !skills.is_empty() {
        s.push_str("\n## Skill Catalog\n\n");
        s.push_str("| Skill | Artifact | Inputs | States |\n");
        s.push_str("|-------|----------|--------|--------|\n");
        for skill in skills {
            let artifact = skill
                .artifacts
                .first()
                .map(|a| a.artifact_type.as_str())
                .unwrap_or("-");
            let inputs = if skill.inputs.is_empty() {
                "none".to_string()
            } else {
                skill
                    .inputs
                    .iter()
                    .map(|i| i.from_skill.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let states: Vec<&str> = skill.workflow.states.keys().map(|k| k.as_str()).collect();
            s.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                skill.name,
                artifact,
                inputs,
                states.join(" → ")
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

    let mut installed = vec![".claude/CLAUDE.md".to_string()];

    let skills_dir = claude_dir.join("skills");
    for skill in skills {
        let skill_dir = skills_dir.join(format!("popsicle-{}", skill.name));
        std::fs::create_dir_all(&skill_dir)?;

        let content = build_agent_skill(skill);
        std::fs::write(skill_dir.join("SKILL.md"), &content)?;
        installed.push(format!(".claude/skills/popsicle-{}/SKILL.md", skill.name));
    }

    let next_dir = skills_dir.join("popsicle-next");
    std::fs::create_dir_all(&next_dir)?;
    std::fs::write(next_dir.join("SKILL.md"), SKILL_NEXT)?;
    installed.push(".claude/skills/popsicle-next/SKILL.md".to_string());

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

    let mut installed = vec![".cursor/rules/popsicle.mdc".into()];

    let skills_dir = root.join(".cursor").join("skills");
    for skill in skills {
        let skill_dir = skills_dir.join(format!("popsicle-{}", skill.name));
        std::fs::create_dir_all(&skill_dir)?;

        let content = build_agent_skill(skill);
        std::fs::write(skill_dir.join("SKILL.md"), &content)?;
        installed.push(format!(".cursor/skills/popsicle-{}/SKILL.md", skill.name));
    }

    Ok(installed)
}

/// Build a SKILL.md file following the Agent Skills open standard.
/// Used by both Claude Code (.claude/skills/) and Cursor (.cursor/skills/).
fn build_agent_skill(skill: &SkillDef) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "---\nname: popsicle-{}\ndescription: {}\n---\n\n",
        skill.name, skill.description
    ));

    s.push_str(&build_skill_command(skill));

    s
}

const SKILL_NEXT: &str = r#"---
name: popsicle-next
description: Check what to do next in the Popsicle pipeline. Use when starting work, after completing a step, or when unsure what to do next.
---

Check what to do next in the Popsicle pipeline and follow the recommended action.

```bash
popsicle pipeline next --format json
```

Then execute the suggested CLI command. If a step has `requires_approval: true`, STOP and ask the user to review before proceeding.
"#;
