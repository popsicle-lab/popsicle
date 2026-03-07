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

    pub fn all() -> Vec<Self> {
        vec![Self::Claude, Self::Cursor, Self::Codex]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Codex => "codex",
        }
    }
}

fn build_instructions(skills: &[&SkillDef]) -> String {
    let mut s = String::from(
        r#"This project uses Popsicle for spec-driven development orchestration.
All development follows a structured pipeline: Domain → PRD → RFC/ADR → TestSpec → Implementation.

## Before Starting Any Task

```bash
popsicle pipeline next --format json
```

This shows what to do next, including the CLI command and AI prompt.

## Key Commands

- `popsicle pipeline status` — current pipeline state
- `popsicle pipeline next --format json` — recommended next steps with prompts
- `popsicle context --format json` — all documents for current pipeline run
- `popsicle prompt <skill> --run <run-id>` — get prompt with upstream context
- `popsicle doc create <skill> --title "<t>" --run <run-id>` — create document
- `popsicle doc transition <doc-id> <action>` — advance document state
- `popsicle git link --doc <doc-id> --stage <stage>` — link commit to document

## Workflow Discipline

1. Always check `popsicle pipeline next` before starting work
2. Follow the suggested skill and action
3. Guard conditions enforce that upstream documents must be approved before downstream work
4. Fill in document sections with real content — template placeholders will be rejected by guards
5. Link commits to documents with `popsicle git link`
6. All output supports `--format json` for structured consumption
"#,
    );

    if !skills.is_empty() {
        s.push_str("\n## Available Skills\n\n");
        s.push_str("| Skill | Artifact | Description | Inputs |\n");
        s.push_str("|-------|----------|-------------|--------|\n");

        for skill in skills {
            let artifacts: Vec<&str> = skill
                .artifacts
                .iter()
                .map(|a| a.artifact_type.as_str())
                .collect();
            let inputs: Vec<String> = skill
                .inputs
                .iter()
                .map(|i| {
                    format!(
                        "{} ({})",
                        i.from_skill,
                        if i.required { "required" } else { "optional" }
                    )
                })
                .collect();
            let inputs_str = if inputs.is_empty() {
                "none".to_string()
            } else {
                inputs.join(", ")
            };

            s.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                skill.name,
                artifacts.join(", "),
                skill.description,
                inputs_str,
            ));
        }

        s.push_str("\n## Skill Workflow Details\n");
        for skill in skills {
            s.push_str(&format!("\n### {}\n\n", skill.name));
            s.push_str(&format!(
                "- **Initial state**: `{}`\n",
                skill.workflow.initial
            ));

            let finals: Vec<&str> = skill
                .workflow
                .states
                .iter()
                .filter(|(_, sd)| sd.r#final)
                .map(|(name, _)| name.as_str())
                .collect();
            if !finals.is_empty() {
                s.push_str(&format!("- **Final state(s)**: `{}`\n", finals.join("`, `")));
            }

            s.push_str("- **Transitions**:\n");
            for (state_name, state_def) in &skill.workflow.states {
                for t in &state_def.transitions {
                    let guard = t
                        .guard
                        .as_ref()
                        .map(|g| format!(" [guard: {}]", g))
                        .unwrap_or_default();
                    s.push_str(&format!(
                        "  - `{}` --`{}`--> `{}`{}\n",
                        state_name, t.action, t.to, guard
                    ));
                }
            }

            if !skill.prompts.is_empty() {
                s.push_str("- **Prompts available for states**: ");
                let prompt_states: Vec<&str> =
                    skill.prompts.keys().map(|k| k.as_str()).collect();
                s.push_str(&format!("`{}`\n", prompt_states.join("`, `")));
            }
        }
    }

    s
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

        let instructions = build_instructions(skills);
        let mut installed = Vec::new();

        for target in &targets {
            match target {
                AgentTarget::Claude => {
                    let files = Self::install_claude(project_root, &instructions)?;
                    installed.extend(files);
                }
                AgentTarget::Cursor => {
                    let files = Self::install_cursor(project_root, &instructions)?;
                    installed.extend(files);
                }
                AgentTarget::Codex => {
                    let files = Self::install_codex(project_root, &instructions)?;
                    installed.extend(files);
                }
            }
        }

        Ok(installed)
    }

    fn install_claude(root: &Path, instructions: &str) -> Result<Vec<String>> {
        let claude_dir = root.join(".claude");
        std::fs::create_dir_all(&claude_dir)?;

        let content = format!(
            "# Popsicle — Claude Code Instructions\n\n{}\n",
            instructions
        );
        std::fs::write(claude_dir.join("CLAUDE.md"), content)?;

        let commands_dir = claude_dir.join("commands");
        std::fs::create_dir_all(&commands_dir)?;

        std::fs::write(commands_dir.join("popsicle-next.md"), SLASH_CMD_NEXT)?;
        std::fs::write(commands_dir.join("popsicle-status.md"), SLASH_CMD_STATUS)?;
        std::fs::write(commands_dir.join("popsicle-start.md"), SLASH_CMD_START)?;

        Ok(vec![
            ".claude/CLAUDE.md".into(),
            ".claude/commands/popsicle-next.md".into(),
            ".claude/commands/popsicle-status.md".into(),
            ".claude/commands/popsicle-start.md".into(),
        ])
    }

    fn install_cursor(root: &Path, instructions: &str) -> Result<Vec<String>> {
        let rules_dir = root.join(".cursor").join("rules");
        std::fs::create_dir_all(&rules_dir)?;

        let rules_content = format!(
            "---\ndescription: Popsicle spec-driven development workflow\nglobs:\nalwaysApply: true\n---\n\n# Popsicle Workflow\n\n{}\n",
            instructions
        );
        std::fs::write(rules_dir.join("popsicle.mdc"), rules_content)?;

        let agents_dir = root.join(".cursor").join("agents");
        std::fs::create_dir_all(&agents_dir)?;

        let agent_def = format!(
            "---\nname: popsicle\ndescription: Popsicle spec-driven development assistant\n---\n\nYou are a Popsicle development assistant. Use the `popsicle` CLI to manage spec-driven development workflows.\n\n{}\n",
            instructions
        );
        std::fs::write(agents_dir.join("popsicle.md"), agent_def)?;

        Ok(vec![
            ".cursor/rules/popsicle.mdc".into(),
            ".cursor/agents/popsicle.md".into(),
        ])
    }

    fn install_codex(root: &Path, instructions: &str) -> Result<Vec<String>> {
        let content = format!(
            "# Popsicle — Agent Instructions (Codex / OpenAI)\n\n{}\n",
            instructions
        );
        std::fs::write(root.join("AGENTS.md"), content)?;
        Ok(vec!["AGENTS.md".into()])
    }
}

const SLASH_CMD_NEXT: &str = r#"Check what to do next in the Popsicle pipeline and follow the recommended action.

Run this command to get the next steps:
```bash
popsicle pipeline next --format json
```

Then execute the suggested CLI command and follow the AI prompt provided.
"#;

const SLASH_CMD_STATUS: &str = r#"Show the current Popsicle pipeline status, including all stages, documents, and their states.

```bash
popsicle pipeline status
```
"#;

const SLASH_CMD_START: &str = r#"Start a new Popsicle pipeline run for a feature.

Ask the user for the feature name, then run:
```bash
popsicle pipeline run full-sdlc --title "<feature name>"
```

After starting, check next steps with:
```bash
popsicle pipeline next --format json
```
"#;
