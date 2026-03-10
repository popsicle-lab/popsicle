use crate::error::{PopsicleError, Result};
use crate::model::Document;
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
) -> Result<GuardResult> {
    let parts: Vec<&str> = guard.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if parts.len() > 1 {
        let mut failed = Vec::new();
        for part in &parts {
            let result = check_single_guard(part, doc, all_docs, registry)?;
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

    check_single_guard(guard.trim(), doc, all_docs, registry)
}

fn check_single_guard(
    guard: &str,
    doc: &Document,
    all_docs: &[DocumentRow],
    registry: &SkillRegistry,
) -> Result<GuardResult> {
    let guard = guard.trim();

    if guard == "upstream_approved" {
        return check_upstream_approved(doc, all_docs, registry);
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

/// Check that all required upstream skill docs exist and are in a final state.
fn check_upstream_approved(
    doc: &Document,
    all_docs: &[DocumentRow],
    registry: &SkillRegistry,
) -> Result<GuardResult> {
    let skill = registry.get(&doc.skill_name)?;

    let mut missing = Vec::new();
    let mut not_final = Vec::new();

    for input in &skill.inputs {
        if !input.required {
            continue;
        }

        let upstream_docs: Vec<&DocumentRow> = all_docs
            .iter()
            .filter(|d| {
                d.skill_name == input.from_skill && d.pipeline_run_id == doc.pipeline_run_id
            })
            .collect();

        if upstream_docs.is_empty() {
            missing.push(format!(
                "{} (from {})",
                input.artifact_type, input.from_skill
            ));
            continue;
        }

        for ud in &upstream_docs {
            if let Ok(upstream_skill) = registry.get(&ud.skill_name)
                && !upstream_skill.is_final_state(&ud.status)
            {
                not_final.push(format!(
                    "{} '{}' is '{}', not final",
                    ud.skill_name, ud.title, ud.status
                ));
            }
        }
    }

    if missing.is_empty() && not_final.is_empty() {
        Ok(GuardResult {
            passed: true,
            guard_name: "upstream_approved".to_string(),
            message: "All upstream documents approved.".to_string(),
        })
    } else {
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!("Missing: {}", missing.join(", ")));
        }
        if !not_final.is_empty() {
            reasons.push(format!("Not approved: {}", not_final.join("; ")));
        }
        Ok(GuardResult {
            passed: false,
            guard_name: "upstream_approved".to_string(),
            message: reasons.join(". "),
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
            let section_content = extract_section_content(after_header);
            if is_template_placeholder(&section_content) {
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
                    extract_section_content(after_header)
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
            message: format!(
                "{}/{} checklist items still unchecked.",
                unchecked, total
            ),
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

/// Extract content between the current H2 header and the next H2 header.
fn extract_section_content(after_header: &str) -> String {
    let lines: Vec<&str> = after_header.lines().collect();
    let mut content_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            continue; // skip the header line itself (empty after split)
        }
        if line.starts_with("## ") {
            break;
        }
        content_lines.push(*line);
    }

    content_lines.join("\n").trim().to_string()
}

/// Heuristic: check if content looks like an unfilled template.
fn is_template_placeholder(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return true;
    }

    let placeholder_patterns = [
        "...",
        "[Name]",
        "[Title]",
        "Description...",
        "Add detailed content here",
        "Brief description",
        "Describe ",
        "TODO",
        "TBD",
    ];

    let non_placeholder_lines: Vec<&str> = trimmed
        .lines()
        .filter(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            !placeholder_patterns.iter().any(|p| l.contains(p))
        })
        .collect();

    non_placeholder_lines.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_template_placeholder() {
        assert!(is_template_placeholder(""));
        assert!(is_template_placeholder("..."));
        assert!(is_template_placeholder("Describe the purpose"));
        assert!(is_template_placeholder("[Name]"));
        assert!(!is_template_placeholder(
            "We use Redis for caching with a 5-minute TTL."
        ));
        assert!(!is_template_placeholder(
            "The system handles user authentication via JWT tokens."
        ));
    }

    #[test]
    fn test_extract_section_content() {
        let text = "\n\nSome real content here.\nMore details.\n\n## Next Section\n\nOther stuff.";
        let content = extract_section_content(text);
        assert!(content.contains("Some real content here."));
        assert!(!content.contains("Next Section"));
    }

    #[test]
    fn test_has_sections_pass() {
        let doc = Document {
            id: "d1".into(),
            doc_type: "prd".into(),
            title: "Test".into(),
            status: "draft".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            status: "draft".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            status: "draft".into(),
            skill_name: "product-prd".into(),
            pipeline_run_id: "r1".into(),
            tags: vec![],
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            body: "## Tasks\n\n- [x] Build API\n- [ ] Write docs\n- [ ] Deploy\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let result = check_checklist_complete(&doc, None).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("2/3 checklist items still unchecked"));
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
            body: "## Open Questions\n\n- [ ] Decide on DB\n\n## Task Checklist\n\n- [x] Build API\n- [x] Write tests\n\n## Notes\n\nSome notes.\n".into(),
            file_path: std::path::PathBuf::new(),
        };

        let scoped = check_checklist_complete(&doc, Some("Task Checklist")).unwrap();
        assert!(scoped.passed, "Scoped should pass: {}", scoped.message);

        let global = check_checklist_complete(&doc, None).unwrap();
        assert!(!global.passed, "Global should fail due to Open Questions checkbox");
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
            metadata: serde_yaml_ng::Value::Null,
            created_at: None,
            updated_at: None,
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
}
