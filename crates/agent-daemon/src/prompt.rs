//! Load intent-coder skill guide as agent prompt (ADR-001 §4).

use std::io;
use std::path::Path;

pub fn skill_guide_path(workspace: &Path, skill: &str) -> Option<std::path::PathBuf> {
    let candidates = [
        workspace
            .join("intent-coder/skills")
            .join(skill)
            .join("guide.md"),
        workspace
            .join(".popsicle/modules/intent-coder/skills")
            .join(skill)
            .join("guide.md"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn load_skill_prompt(workspace: &Path, skill: &str, run_id: &str) -> io::Result<String> {
    let path = skill_guide_path(workspace, skill).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("skill guide not found for {skill}"),
        )
    })?;
    let guide = std::fs::read_to_string(&path)?;
    Ok(format!(
        "You are executing Popsicle pipeline skill `{skill}` for run `{run_id}`.\n\
         Follow the guide below. Use popsicle CLI with --format json where applicable.\n\n\
         {guide}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn loads_shadow_implementer_guide() {
        let workspace = env::current_dir().expect("cwd");
        if skill_guide_path(&workspace, "shadow-implementer").is_none() {
            return;
        }
        let prompt = load_skill_prompt(&workspace, "shadow-implementer", "run-test").expect("load");
        assert!(prompt.contains("shadow-implementer"));
        assert!(prompt.contains("run-test"));
    }
}
