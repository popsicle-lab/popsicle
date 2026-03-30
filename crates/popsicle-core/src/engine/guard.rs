use crate::engine::markdown;
use crate::error::{PopsicleError, Result};
use crate::model::{Document, PipelineDef, PipelineRun, StageState};
use crate::registry::SkillRegistry;
use crate::storage::DocumentRow;

/// Result of a guard check.
#[derive(Debug, Clone)]
pub struct GuardResult {
    pub passed: bool,
    pub guard_name: String,
    pub message: String,
}

/// Evaluate a guard condition string against the current state.
///
/// Multiple guards can be combined with `;` (all must pass):
///   `"upstream_approved;has_sections:Summary;checklist_complete:Tasks"`
///
/// Supported guard types:
/// - `upstream_approved` — all required upstream skill documents must be in a final state
/// - `has_sections:<Section1>,<Section2>` — document body must contain these H2 headings
///   with non-template content beneath them
/// - `checklist_complete` — all Markdown checkboxes in the document are checked
/// - `checklist_complete:<Section>` — all checkboxes in the named H2 section are checked
pub fn check_guard(
    guard: &str,
    doc: &Document,
    all_docs: &[DocumentRow],
    registry: &SkillRegistry,
    pipeline: Option<&PipelineDef>,
    run: Option<&PipelineRun>,
) -> Result<GuardResult> {
    let parts: Vec<&str> = guard
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() > 1 {
        let mut failed = Vec::new();
        for part in &parts {
            let result = check_single_guard(part, doc, all_docs, registry, pipeline, run)?;
            if !result.passed {
                failed.push(result.message);
            }
        }
        if failed.is_empty() {
            return Ok(GuardResult {
                passed: true,
                guard_name: guard.to_string(),
                message: "All guards passed.".to_string(),
            });
        }
        return Ok(GuardResult {
            passed: false,
            guard_name: guard.to_string(),
            message: failed.join(". "),
        });
    }

    check_single_guard(guard.trim(), doc, all_docs, registry, pipeline, run)
}

fn check_single_guard(
    guard: &str,
    doc: &Document,
    _all_docs: &[DocumentRow],
    registry: &SkillRegistry,
    pipeline: Option<&PipelineDef>,
    run: Option<&PipelineRun>,
) -> Result<GuardResult> {
    let guard = guard.trim();

    if guard == "upstream_approved" {
        return check_upstream_completed(doc, registry, pipeline, run);
    }

    if let Some(sections_str) = guard.strip_prefix("has_sections:") {
        let required: Vec<&str> = sections_str.split(',').map(|s| s.trim()).collect();
        return check_has_sections(doc, &required);
    }

    if guard == "checklist_complete" {
        return check_checklist_complete(doc, None);
    }

    if let Some(section) = guard.strip_prefix("checklist_complete:") {
        return check_checklist_complete(doc, Some(section.trim()));
    }

    Err(PopsicleError::InvalidSkillDef(format!(
        "Unknown guard type: {}",
        guard
    )))
}

/// Check that all upstream stages (those containing required input skills) are Completed.
///
/// With the pipeline-as-state-source model, we check STAGE states rather than
/// individual document states. If no pipeline/run is provided, we fall back to
/// checking that upstream documents exist (any status).
fn check_upstream_completed(
    doc: &Document,
    registry: &SkillRegistry,
    pipeline: Option<&PipelineDef>,
    run: Option<&PipelineRun>,
) -> Result<GuardResult> {
    let skill = registry.get(&doc.skill_name)?;
    let pipeline_skills: Option<Vec<&str>> = pipeline.map(|p| p.all_skill_names());

    let mut not_completed = Vec::new();

    for input in &skill.inputs {
        if !input.required {
            continue;
        }

        // Skip required inputs whose from_skill is not in the current pipeline
        if let Some(ref skills) = pipeline_skills
            && !skills.contains(&input.from_skill.as_str())
        {
            continue;
        }

        // Find which stage contains the upstream skill
        if let Some(pdef) = pipeline {
            if let Some(run) = run {
                let upstream_stage = pdef
                    .stages
                    .iter()
                    .find(|s| s.skill_names().contains(&input.from_skill.as_str()));

                if let Some(stage) = upstream_stage {
                    let stage_state = run
                        .stage_states
                        .get(&stage.name)
                        .copied()
                        .unwrap_or(StageState::Blocked);

                    if !matches!(stage_state, StageState::Completed | StageState::Skipped) {
                        not_completed.push(format!(
                            "stage '{}' is '{}' (needed for {})",
                            stage.name, stage_state, input.from_skill
                        ));
                    }
                }
                continue;
            }
        }

        // Fallback: no pipeline/run context — just check that doc exists
        // (shouldn't normally happen with the new model)
    }

    if not_completed.is_empty() {
        Ok(GuardResult {
            passed: true,
            guard_name: "upstream_approved".to_string(),
            message: "All upstream stages completed.".to_string(),
        })
    } else {
        Ok(GuardResult {
            passed: false,
            guard_name: "upstream_approved".to_string(),
            message: format!("Upstream not completed: {}", not_completed.join("; ")),
        })
    }
}

/// Check that the document body contains the specified H2 sections
/// with meaningful content (not just the template placeholder text).
fn check_has_sections(doc: &Document, required: &[&str]) -> Result<GuardResult> {
    let mut missing = Vec::new();
    let mut empty_sections = Vec::new();

    for &section in required {
        let header = format!("## {}", section);
        if let Some(pos) = doc.body.find(&header) {
            let after_header = &doc.body[pos + header.len()..];
            let section_content = markdown::extract_section_content(after_header);
            if markdown::is_template_placeholder(&section_content) {
                empty_sections.push(section.to_string());
            }
        } else {
            missing.push(section.to_string());
        }
    }

    if missing.is_empty() && empty_sections.is_empty() {
        Ok(GuardResult {
            passed: true,
            guard_name: format!("has_sections:{}", required.join(",")),
            message: "All required sections present and filled.".to_string(),
        })
    } else {
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!("Missing sections: {}", missing.join(", ")));
        }
        if !empty_sections.is_empty() {
            reasons.push(format!(
                "Sections still have template placeholders: {}",
                empty_sections.join(", ")
            ));
        }
        Ok(GuardResult {
            passed: false,
            guard_name: format!("has_sections:{}", required.join(",")),
            message: reasons.join(". "),
        })
    }
}

/// Check that all Markdown checkboxes are checked.
/// If `section` is provided, only checkboxes in that H2 section are examined.
fn check_checklist_complete(doc: &Document, section: Option<&str>) -> Result<GuardResult> {
    let text = match section {
        Some(name) => {
            let header = format!("## {}", name);
            match doc.body.find(&header) {
                Some(pos) => {
                    let after_header = &doc.body[pos + header.len()..];
                    markdown::extract_section_content(after_header)
                }
                None => {
                    return Ok(GuardResult {
                        passed: false,
                        guard_name: format!("checklist_complete:{}", name),
                        message: format!("Section '{}' not found in document.", name),
                    });
                }
            }
        }
        None => doc.body.clone(),
    };

    let (checked, unchecked) = count_checkboxes(&text);
    let total = checked + unchecked;
    let guard_name = match section {
        Some(name) => format!("checklist_complete:{}", name),
        None => "checklist_complete".to_string(),
    };

    if total == 0 {
        return Ok(GuardResult {
            passed: false,
            guard_name,
            message: "No checklist items found.".to_string(),
        });
    }

    if unchecked == 0 {
        Ok(GuardResult {
            passed: true,
            guard_name,
            message: format!("All {} checklist items complete.", total),
        })
    } else {
        Ok(GuardResult {
            passed: false,
            guard_name,
            message: format!("{}/{} checklist items still unchecked.", unchecked, total),
        })
    }
}

/// Count checked `- [x]` and unchecked `- [ ]` Markdown checkboxes in text.
/// Returns (checked, unchecked).
pub fn count_checkboxes(text: &str) -> (usize, usize) {
    let mut checked = 0usize;
    let mut unchecked = 0usize;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            checked += 1;
        } else if trimmed.starts_with("- [ ] ") {
            unchecked += 1;
        }
    }
    (checked, unchecked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_sections_pass() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "prd".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Background\n\nWe need caching for performance.\n\n## Goals\n\n- Reduce latency to under 200ms\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_has_sections(&doc, &["Background", "Goals"]).unwrap();
        assert!(result.passed, "Should pass: {}", result.message);
    }

    #[test]
    fn test_has_sections_fail_missing() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "prd".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Background\n\nSome content.\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_has_sections(&doc, &["Background", "Goals"]).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("Missing sections: Goals"));
    }

    #[test]
    fn test_has_sections_fail_placeholder() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "prd".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body:
                "## Background\n\nDescribe the business context.\n\n## Goals\n\n- Reduce latency\n"
                    .into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_has_sections(&doc, &["Background", "Goals"]).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("template placeholders"));
    }

    #[test]
    fn test_count_checkboxes() {
        let text = "- [x] Done item\n- [ ] Pending item\n- [X] Also done\n- Regular list item\n  - [ ] Nested pending";
        let (checked, unchecked) = count_checkboxes(text);
        assert_eq!(checked, 2);
        assert_eq!(unchecked, 2);
    }

    #[test]
    fn test_checklist_complete_all_checked() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Tasks\n\n- [x] Build API\n- [x] Write docs\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_checklist_complete(&doc, None).unwrap();
        assert!(result.passed, "Should pass: {}", result.message);
        assert!(result.message.contains("2 checklist items complete"));
    }

    #[test]
    fn test_checklist_complete_some_unchecked() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Tasks\n\n- [x] Build API\n- [ ] Write docs\n- [ ] Deploy\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_checklist_complete(&doc, None).unwrap();
        assert!(!result.passed);
        assert!(
            result
                .message
                .contains("2/3 checklist items still unchecked")
        );
    }

    #[test]
    fn test_checklist_complete_no_items() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Summary\n\nJust text, no checkboxes.\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_checklist_complete(&doc, None).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("No checklist items found"));
    }

    #[test]
    fn test_checklist_complete_scoped_to_section() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Open Questions\n\n- [ ] Decide on DB\n\n## Task Checklist\n\n- [x] Build API\n- [x] Write tests\n\n## Notes\n\nSome notes.\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let scoped = check_checklist_complete(&doc, Some("Task Checklist")).unwrap();
        assert!(scoped.passed, "Scoped should pass: {}", scoped.message);

        let global = check_checklist_complete(&doc, None).unwrap();
        assert!(
            !global.passed,
            "Global should fail due to Open Questions checkbox"
        );
    }

    #[test]
    fn test_checklist_complete_missing_section() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Summary\n\nSome content.\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_checklist_complete(&doc, Some("Task Checklist")).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("not found"));
    }

    #[test]
    fn test_compound_guard_all_pass() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "domain-analysis".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Summary\n\nReal content here.\n\n## Checklist\n\n- [x] Done\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let registry = make_registry();
        let docs: Vec<DocumentRow> = vec![];

        let result = check_guard(
            "has_sections:Summary;checklist_complete:Checklist",
            &doc,
            &docs,
            &registry,
            None,
            None,
        )
        .unwrap();
        assert!(result.passed, "Compound should pass: {}", result.message);
    }

    #[test]
    fn test_compound_guard_partial_fail() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl".into(),
            title: "Test".into(),
            status: "coding".into(),
            skill_name: "domain-analysis".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: "## Summary\n\nReal content here.\n\n## Checklist\n\n- [ ] Not done\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let registry = make_registry();
        let docs: Vec<DocumentRow> = vec![];

        let result = check_guard(
            "has_sections:Summary;checklist_complete:Checklist",
            &doc,
            &docs,
            &registry,
            None,
            None,
        )
        .unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("unchecked"));
    }

    fn make_registry() -> SkillRegistry {
        let yaml = r#"
name: domain-analysis
description: Domain boundary analysis
version: "0.1.0"
artifacts:
  - type: domain-model
    template: templates/domain.md
    file_pattern: "{slug}.domain.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
    review:
      transitions:
        - to: approved
          action: approve
    approved:
      final: true
"#;
        let mut registry = SkillRegistry::new();
        let skill: crate::model::SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
        registry.register(skill);
        registry
    }

    fn make_impl_registry() -> SkillRegistry {
        let impl_yaml = r#"
name: implementation
description: Code implementation
version: "0.1.0"
inputs:
  - from_skill: rfc
    artifact_type: rfc
    required: true
  - from_skill: adr
    artifact_type: adr
    required: true
artifacts:
  - type: impl-record
    template: templates/impl-record.md
    file_pattern: "{slug}.impl-record.md"
workflow:
  initial: planning
  states:
    planning:
      transitions:
        - to: coding
          action: start
          guard: "upstream_approved"
    coding:
      transitions:
        - to: review
          action: submit
    review:
      transitions:
        - to: completed
          action: approve
    completed:
      final: true
"#;
        let rfc_yaml = r#"
name: rfc
description: Technical RFC
version: "0.1.0"
artifacts:
  - type: rfc
    template: templates/rfc.md
    file_pattern: "{slug}.rfc.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: accepted
          action: accept
    accepted:
      final: true
"#;
        let adr_yaml = r#"
name: adr
description: Architecture Decision Record
version: "0.1.0"
artifacts:
  - type: adr
    template: templates/adr.md
    file_pattern: "{slug}.adr.md"
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: accepted
          action: accept
    accepted:
      final: true
"#;
        let mut registry = SkillRegistry::new();
        for yaml in [impl_yaml, rfc_yaml, adr_yaml] {
            let skill: crate::model::SkillDef = serde_yaml_ng::from_str(yaml).unwrap();
            registry.register(skill);
        }
        registry
    }

    #[test]
    fn test_upstream_completed_no_run_passes() {
        // Without a PipelineRun, the guard has nothing to check — passes by default
        let registry = make_impl_registry();
        let doc = Document {
            id: "d1".into(),
            doc_type: "impl-record".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: String::new(),
            file_path: std::path::PathBuf::new(),
        };
        let docs: Vec<DocumentRow> = vec![];

        let result = check_guard("upstream_approved", &doc, &docs, &registry, None, None).unwrap();
        assert!(
            result.passed,
            "Should pass without pipeline/run: {}",
            result.message
        );
    }

    #[test]
    fn test_upstream_completed_pipeline_skips_non_pipeline_skills() {
        use crate::model::{PipelineDef, PipelineRun, StageDef};

        let registry = make_impl_registry();

        // Pipeline that only has implementation — no rfc or adr stages
        let pipeline = PipelineDef {
            name: "impl-test".to_string(),
            description: "Light pipeline".to_string(),
            stages: vec![StageDef {
                name: "implementation".to_string(),
                skills: vec![],
                skill: Some("implementation".to_string()),
                description: "Impl".to_string(),
                depends_on: vec![],
                requires_approval: false,
            }],
            keywords: vec![],
            scale: Some("light".to_string()),
        };

        let run = PipelineRun::new(&pipeline, "Test", "spec-1", "issue-1");

        let doc = Document {
            id: "d1".into(),
            doc_type: "impl-record".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: run.id.clone(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: String::new(),
            file_path: std::path::PathBuf::new(),
        };
        let docs: Vec<DocumentRow> = vec![];

        let result = check_guard(
            "upstream_approved",
            &doc,
            &docs,
            &registry,
            Some(&pipeline),
            Some(&run),
        )
        .unwrap();
        assert!(
            result.passed,
            "Should pass — rfc/adr not in pipeline: {}",
            result.message
        );
    }

    #[test]
    fn test_upstream_completed_fails_when_stage_not_completed() {
        use crate::model::{PipelineDef, PipelineRun, StageDef};

        let registry = make_impl_registry();

        // Pipeline with tech-design (rfc, adr) → implementation
        let pipeline = PipelineDef {
            name: "full-sdlc".to_string(),
            description: "Full pipeline".to_string(),
            stages: vec![
                StageDef {
                    name: "tech-design".to_string(),
                    skills: vec!["rfc".to_string(), "adr".to_string()],
                    skill: None,
                    description: "Design".to_string(),
                    depends_on: vec![],
                    requires_approval: false,
                },
                StageDef {
                    name: "implementation".to_string(),
                    skills: vec![],
                    skill: Some("implementation".to_string()),
                    description: "Impl".to_string(),
                    depends_on: vec!["tech-design".to_string()],
                    requires_approval: false,
                },
            ],
            keywords: vec![],
            scale: Some("full".to_string()),
        };

        let run = PipelineRun::new(&pipeline, "Test", "spec-1", "issue-1");
        // tech-design is Ready (not Completed), implementation is Blocked

        let doc = Document {
            id: "d1".into(),
            doc_type: "impl-record".into(),
            title: "Test".into(),
            status: "active".into(),
            skill_name: "implementation".into(),
            pipeline_run_id: run.id.clone(),
            tags: vec![],
            summary: String::new(),
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            spec_id: "test-spec".to_string(),
            version: 1,
            parent_doc_id: None,
            body: String::new(),
            file_path: std::path::PathBuf::new(),
        };
        let docs: Vec<DocumentRow> = vec![];

        let result = check_guard(
            "upstream_approved",
            &doc,
            &docs,
            &registry,
            Some(&pipeline),
            Some(&run),
        )
        .unwrap();
        assert!(
            !result.passed,
            "Should fail — tech-design stage not completed: {}",
            result.message
        );
        assert!(result.message.contains("tech-design"));
    }
}
