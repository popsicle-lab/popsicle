//! Golden equivalence baselines for `artifact-system` slice-delivery.
//!
//! Each `golden_*` test is referenced from
//! `docs/baseline/2026-06-09/artifact-system/run-all.sh`.

use std::collections::BTreeMap;

use artifact_system::context::{
    assemble_layers, context_includes_full_text, ContextLayer, Relevance,
};
use artifact_system::document::Document;
use artifact_system::extractor::{
    extract_bugs, extract_test_cases, extract_user_stories, ChunkKind,
};
use artifact_system::guard::{
    check_guard, GuardError, GuardResult, UpstreamApprovalChecker,
};
use artifact_system::task_chunk::{rename_work_item_to_task_chunk, CKind, WorkItem};

/// G-001: document file content round-trips id, version, metadata, and body.
#[test]
fn golden_001_document_roundtrip() {
    let mut doc = Document::new("doc-1", "prd", "Artifact System PRD");
    doc.status = "final".into();
    doc.extra_frontmatter
        .insert("spec_id".into(), "slice-2-artifact-system".into());
    doc.body = "\n## Summary\n\nLeading blank line and trailing spaces   \n".into();

    let parsed = Document::from_file_content(&doc.to_file_content()).unwrap();
    assert_eq!(parsed, doc);

    let rev = parsed.new_revision("doc-2");
    assert_eq!(rev.version, 2);
    assert_eq!(rev.parent_id.as_deref(), Some("doc-1"));
    assert_eq!(rev.status, "active");
}

/// G-002: guard checks preserve legacy-facing checklist/section outcomes.
#[test]
fn golden_002_guard_sections_and_checklist() {
    let mut doc = Document::new("doc-guard", "prd", "Guard Demo");
    doc.body = "## Summary\n\nFilled.\n\n## Checklist\n\n- [x] one\n- [ ] two\n".into();

    let sections = check_guard("has_sections:Summary,Checklist", &doc, None).unwrap();
    assert!(sections.passed);

    let checklist = check_guard("checklist_complete:Checklist", &doc, None).unwrap();
    assert!(!checklist.passed);
    assert!(checklist.message.contains("1/2"));

    doc.body = doc.body.replace("- [ ] two", "- [x] two");
    let checklist = check_guard("checklist_complete:Checklist", &doc, None).unwrap();
    assert!(checklist.passed);
}

struct GoldenLayer {
    id: &'static str,
    relevance: Relevance,
    priority: i32,
    content: &'static str,
}

impl ContextLayer for GoldenLayer {
    fn id(&self) -> &str {
        self.id
    }

    fn relevance(&self) -> Relevance {
        self.relevance
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn render(&self) -> String {
        self.content.to_string()
    }
}

/// G-003: context assembly is deterministic and relevance-aware.
#[test]
fn golden_003_context_assembly_order() {
    assert!(context_includes_full_text(Relevance::RelHigh));
    assert!(!context_includes_full_text(Relevance::RelLow));

    let layers: Vec<Box<dyn ContextLayer>> = vec![
        Box::new(GoldenLayer {
            id: "upstream",
            relevance: Relevance::RelHigh,
            priority: 0,
            content: "UPSTREAM",
        }),
        Box::new(GoldenLayer {
            id: "project",
            relevance: Relevance::RelLow,
            priority: 0,
            content: "PROJECT",
        }),
        Box::new(GoldenLayer {
            id: "refs",
            relevance: Relevance::RelMedium,
            priority: 0,
            content: "REFS",
        }),
    ];

    let out = assemble_layers(layers, "BASE");
    assert_eq!(
        out,
        "PROJECT\n\n---\n\nREFS\n\n---\n\nUPSTREAM\n\n---\n\nBASE"
    );
}

/// G-004: extractors preserve kind and no-match returns empty output.
#[test]
fn golden_004_extractors_preserve_kind() {
    let body = "\
## User Stories

### Story one

## Bugs

### Bug one

## Test Cases

### Case one
";

    let stories = extract_user_stories(body);
    let bugs = extract_bugs(body);
    let tests = extract_test_cases(body);

    assert_eq!(stories.len(), 1);
    assert!(stories.iter().all(|i| i.kind == ChunkKind::KindStory));
    assert_eq!(bugs.len(), 1);
    assert!(bugs.iter().all(|i| i.kind == ChunkKind::KindBug));
    assert_eq!(tests.len(), 3);
    assert!(tests.iter().all(|i| i.kind == ChunkKind::KindTestCase));
    assert!(extract_bugs("no markdown headings").is_empty());
}

/// G-005: work_item -> task_chunk rename preserves kind and field blob.
#[test]
fn golden_005_task_chunk_rename_preserves_fields() {
    let fields = BTreeMap::from([
        ("acceptance".to_string(), "document round-trips".to_string()),
        ("stage".to_string(), "daily-ops".to_string()),
    ]);
    let work_item = WorkItem {
        kind: CKind::CStory,
        fields: fields.clone(),
    };
    let before_hash = work_item.fields_hash();

    let task_chunk = rename_work_item_to_task_chunk(work_item);

    assert_eq!(task_chunk.kind, CKind::CStory);
    assert_eq!(task_chunk.fields, fields);
    assert_eq!(task_chunk.fields_hash(), before_hash);
}

struct GoldenUpstreamChecker;

impl UpstreamApprovalChecker for GoldenUpstreamChecker {
    fn check_upstream_approved(&self, _doc: &Document) -> GuardResult {
        GuardResult {
            passed: true,
            guard_name: "upstream_approved".into(),
            message: "approved by golden checker".into(),
        }
    }
}

/// G-006: upstream_approved is an injected port, not an artifact-system dependency.
#[test]
fn golden_006_upstream_port_requires_checker() {
    let doc = Document::new("doc-upstream", "rfc", "ADR Port");

    assert!(matches!(
        check_guard("upstream_approved", &doc, None),
        Err(GuardError::InvalidSkillDef(_))
    ));

    let result = check_guard("upstream_approved", &doc, Some(&GoldenUpstreamChecker)).unwrap();
    assert!(result.passed);
    assert_eq!(result.message, "approved by golden checker");
}
