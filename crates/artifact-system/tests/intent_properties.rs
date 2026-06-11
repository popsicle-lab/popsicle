//! Property tests mirroring the Z3-verified intents in
//! `products/artifact-system/intents/`. Each test name traces to a named intent
//! / contract so a reviewer can line them up 1:1.

use std::collections::BTreeMap;

use artifact_system::context::{
    assemble_layers, context_includes_full_text, ordering_key, ContextLayer, Relevance,
};
use artifact_system::document::Document;
use artifact_system::extractor::{
    extract_bugs, extract_test_cases, extract_user_stories, ChunkKind,
};
use artifact_system::guard::{
    check_guard, checklist_outcome, guard_outcome_for, guard_recognized, GuardError, GuardOutcome,
    GuardResult, UpstreamApprovalChecker,
};
use artifact_system::task_chunk::{rename_work_item_to_task_chunk, CKind, WorkItem};

// =====================================================================
// acceptance.intent › DocumentRoundTrips
//   require version >= 1; ensure body' == body, id' == id, version' == version
// =====================================================================

fn doc_with_body(body: &str) -> Document {
    let mut d = Document::new("doc-1", "prd", "Title With Spaces");
    d.body = body.to_string();
    d.extra_frontmatter
        .insert("spec_id".into(), "spec-xyz".into());
    d
}

#[test]
fn document_round_trips_preserves_body_id_version() {
    // Deliberately adversarial bodies — including the cases legacy `trim_start`
    // would have mangled (leading/trailing whitespace) and an embedded `\n---\n`.
    let bodies = [
        "",
        "## Background\nSome content.",
        "\n\nleading blank lines preserved",
        "trailing spaces preserved   ",
        "body containing a delimiter\n---\nstill fine",
        "unicode: 文档往返 ✅ café",
        "- [x] done\n- [ ] todo\n",
    ];
    for body in bodies {
        let d = doc_with_body(body);
        assert!(d.version >= 1, "precondition version >= 1");
        let serialized = d.to_file_content();
        let parsed = Document::from_file_content(&serialized)
            .unwrap_or_else(|e| panic!("parse failed for {body:?}: {e:?}"));
        assert_eq!(parsed.body, d.body, "body' == body for {body:?}");
        assert_eq!(parsed.id, d.id, "id' == id for {body:?}");
        assert_eq!(
            parsed.version, d.version,
            "version' == version for {body:?}"
        );
        // Full struct round-trips too (incl. opaque extra frontmatter).
        assert_eq!(parsed, d, "whole document round-trips for {body:?}");
    }
}

#[test]
fn document_new_revision_bumps_version_and_links_parent() {
    let d = doc_with_body("v1");
    let rev = d.new_revision("doc-2");
    assert_eq!(rev.version, d.version + 1);
    assert_eq!(rev.parent_id.as_deref(), Some(d.id.as_str()));
    assert_eq!(rev.status, "active");
    // A revision still round-trips.
    let parsed = Document::from_file_content(&rev.to_file_content()).unwrap();
    assert_eq!(parsed, rev);
}

// =====================================================================
// acceptance.intent › GuardChecklistCompleteIffNoUnchecked
//   (outcome == GuardPass) == (checkedBoxes == totalBoxes)
// =====================================================================

#[test]
fn guard_checklist_complete_iff_no_unchecked() {
    for total in 0u32..=6 {
        for checked in 0u32..=total {
            let outcome = checklist_outcome(total, checked);
            let pass = outcome == GuardOutcome::GuardPass;
            assert_eq!(
                pass,
                checked == total,
                "pass iff checked==total (total={total}, checked={checked})"
            );
        }
    }
}

#[test]
fn guard_checklist_complete_end_to_end_matches_box_counts() {
    let mut d = Document::new("d", "prd", "t");
    d.body = "## Tasks\n\n- [x] a\n- [ ] b\n".into();
    let r = check_guard("checklist_complete:Tasks", &d, None).unwrap();
    assert!(!r.passed, "one unchecked → fail");

    d.body = "## Tasks\n\n- [x] a\n- [x] b\n".into();
    let r = check_guard("checklist_complete:Tasks", &d, None).unwrap();
    assert!(r.passed, "all checked → pass");

    // Legacy emptiness policy (orthogonal to the pure IFF): zero boxes → fail.
    d.body = "## Tasks\n\njust prose, no boxes\n".into();
    let r = check_guard("checklist_complete:Tasks", &d, None).unwrap();
    assert!(!r.passed, "zero boxes → not complete");
    assert!(r.message.contains("No checklist items"));
}

// =====================================================================
// acceptance.intent › ContextAssemblyOrdersByRelevance
//   High ==> includedFullText == true ; Low ==> includedFullText == false
// =====================================================================

#[test]
fn context_assembly_orders_by_relevance() {
    assert!(
        context_includes_full_text(Relevance::RelHigh),
        "High → full text"
    );
    assert!(
        !context_includes_full_text(Relevance::RelLow),
        "Low → summary"
    );
    // Medium is unconstrained by the intent; pinned here to summary for determinism.
    assert!(!context_includes_full_text(Relevance::RelMedium));
}

// =====================================================================
// contracts.intent C3 / ContextOrderIndependentOfRegistration
//   assemble_layers output is identical for any registration permutation.
// =====================================================================

struct TestLayer {
    id: &'static str,
    relevance: Relevance,
    priority: i32,
    content: &'static str,
}

impl ContextLayer for TestLayer {
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

fn make_layers(order: &[usize], specs: &[TestLayer]) -> Vec<Box<dyn ContextLayer>> {
    order
        .iter()
        .map(|&i| {
            Box::new(TestLayer {
                id: specs[i].id,
                relevance: specs[i].relevance,
                priority: specs[i].priority,
                content: specs[i].content,
            }) as Box<dyn ContextLayer>
        })
        .collect()
}

/// All permutations of `0..n` (Heap's algorithm).
fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    let mut c = vec![0usize; n];
    out.push(a.clone());
    let mut i = 0;
    while i < n {
        if c[i] < i {
            if i % 2 == 0 {
                a.swap(0, i);
            } else {
                a.swap(c[i], i);
            }
            out.push(a.clone());
            c[i] += 1;
            i = 0;
        } else {
            c[i] = 0;
            i += 1;
        }
    }
    out
}

#[test]
fn assemble_layers_is_independent_of_registration_order() {
    // Includes a same-relevance tie (RelLow) AND a same-(relevance,priority) tie
    // (memories/notes both Low/10) to exercise BOTH tie-breakers — the case where
    // legacy's stable sort leaked registration order. A regression that dropped
    // the final `id` key (relying on stable sort) would be caught here.
    let specs = [
        TestLayer {
            id: "memories",
            relevance: Relevance::RelLow,
            priority: 10,
            content: "MEM",
        },
        TestLayer {
            id: "project",
            relevance: Relevance::RelLow,
            priority: 0,
            content: "PROJ",
        },
        TestLayer {
            id: "upstream",
            relevance: Relevance::RelHigh,
            priority: 0,
            content: "UP",
        },
        TestLayer {
            id: "refs",
            relevance: Relevance::RelMedium,
            priority: 0,
            content: "REF",
        },
        TestLayer {
            id: "notes",
            relevance: Relevance::RelLow,
            priority: 10,
            content: "NOTE",
        },
    ];

    let base = "INSTRUCTION";
    let canonical = assemble_layers(make_layers(&[0, 1, 2, 3, 4], &specs), base);

    for perm in permutations(specs.len()) {
        let got = assemble_layers(make_layers(&perm, &specs), base);
        assert_eq!(
            got, canonical,
            "assembly must be identical for perm {perm:?}"
        );
    }

    // Sanity: deterministic key orders (Low,prio,id) → High last (closest to base),
    // and base_prompt is appended last.
    assert!(canonical.ends_with("INSTRUCTION"));
    assert!(
        canonical.find("UP").unwrap() > canonical.find("PROJ").unwrap(),
        "High relevance lands after Low"
    );
}

#[test]
fn ordering_key_is_registration_independent_total_order() {
    let l1 = TestLayer {
        id: "a",
        relevance: Relevance::RelLow,
        priority: 0,
        content: "",
    };
    let l2 = TestLayer {
        id: "b",
        relevance: Relevance::RelLow,
        priority: 0,
        content: "",
    };
    // Same relevance+priority → id breaks the tie deterministically.
    assert!(ordering_key(&l1) < ordering_key(&l2));
}

#[test]
fn assemble_layers_skips_empty_renders() {
    let specs = [
        TestLayer {
            id: "a",
            relevance: Relevance::RelLow,
            priority: 0,
            content: "",
        },
        TestLayer {
            id: "b",
            relevance: Relevance::RelHigh,
            priority: 0,
            content: "B",
        },
    ];
    let out = assemble_layers(make_layers(&[0, 1], &specs), "BASE");
    assert_eq!(out, "B\n\n---\n\nBASE");
}

// =====================================================================
// acceptance.intent › ExtractPreservesKind  (e.kind' == e.kind)
// contracts.intent C2 › extractor totality (no panic, no-match → empty)
// =====================================================================

#[test]
fn extract_preserves_kind() {
    let body = "\
## User Stories

### Story one
### Story two

## Bugs

### Bug one
";
    for item in extract_user_stories(body) {
        assert_eq!(item.kind, ChunkKind::KindStory);
    }
    for item in extract_test_cases(body) {
        assert_eq!(item.kind, ChunkKind::KindTestCase);
    }
    for item in extract_bugs(body) {
        assert_eq!(item.kind, ChunkKind::KindBug);
    }
    assert_eq!(extract_user_stories(body).len(), 2);
    assert_eq!(extract_bugs(body).len(), 1);
    // Non-vacuous: extract_test_cases scans every H3 in the body → 3 here. Guards
    // against a regression to `Vec::new()` (which would make the kind loop vacuous).
    assert_eq!(extract_test_cases(body).len(), 3);
}

#[test]
fn extract_is_total_on_noisy_input() {
    // No-match, empty, unicode, unterminated/odd markers, very long line.
    let long = "#".repeat(100_000);
    let noisy = [
        "",
        "no headings here at all",
        "## User Stories\n\n(no h3 items)\n",
        "### orphan h3 with no section",
        "## Bugs\n\n### \n###no-space\n### 真正的标题 🐛\n",
        &long,
    ];
    for body in noisy {
        // Must never panic; results are well-formed (kinds correct).
        for item in extract_user_stories(body) {
            assert_eq!(item.kind, ChunkKind::KindStory);
        }
        let _ = extract_test_cases(body);
        for item in extract_bugs(body) {
            assert_eq!(item.kind, ChunkKind::KindBug);
        }
    }
    assert!(extract_user_stories("nothing").is_empty());
}

// =====================================================================
// acceptance.intent › RenameWorkItemToTaskChunk
//   ensure kind' == kind, fieldsHash' == fieldsHash
// =====================================================================

#[test]
fn rename_work_item_to_task_chunk_preserves_kind_and_fields() {
    let field_sets: [Vec<(&str, &str)>; 3] = [
        vec![],
        vec![("persona", "dev"), ("goal", "ship"), ("benefit", "joy")],
        vec![("severity", "high"), ("steps", "1;2;3")],
    ];
    for (kind, fields) in [
        (CKind::CBug, &field_sets[2]),
        (CKind::CStory, &field_sets[1]),
        (CKind::CTestCase, &field_sets[0]),
    ] {
        let map: BTreeMap<String, String> = fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let wi = WorkItem {
            kind,
            fields: map.clone(),
        };
        let before_hash = wi.fields_hash();

        let tc = rename_work_item_to_task_chunk(wi);

        assert_eq!(tc.kind, kind, "kind' == kind");
        // Direct map comparison (not just the hash) — fields preserved exactly.
        assert_eq!(tc.fields, map, "fields blob preserved key-for-key");
        assert_eq!(tc.fields_hash(), before_hash, "fieldsHash' == fieldsHash");
    }
}

// =====================================================================
// invariants.intent › GuardResultIsTotal / EvaluateGuard
//   (!recognized) ==> outcome == GuardInvalid ; never panics
// contracts.intent C1 › UpstreamApprovalChecker port; missing checker → Invalid
// =====================================================================

struct AlwaysApprove;
impl UpstreamApprovalChecker for AlwaysApprove {
    fn check_upstream_approved(&self, _doc: &Document) -> GuardResult {
        GuardResult {
            passed: true,
            guard_name: "upstream_approved".into(),
            message: "ok".into(),
        }
    }
}

#[test]
fn unknown_guard_is_invalid_and_total() {
    let d = Document::new("d", "prd", "t");
    let guards = [
        "",
        "   ",
        "bogus",
        "has_sections",        // missing ':' → unknown
        "checklist_complete:", // empty section name is still recognized
        "definitely_not_a_guard:x",
        "; ; ;",                  // only empty fragments
        "has_sections:A;mystery", // unknown fragment in a chain
    ];
    for g in guards {
        // Totality: never panics.
        let outcome = guard_outcome_for(g, &d, None);
        // Each listed guard either is fully unrecognized or contains an
        // unrecognized fragment → propagates InvalidSkillDef (GuardInvalid),
        // EXCEPT "checklist_complete:" which is recognized (and will Fail/Pass).
        if g.trim() == "checklist_complete:" {
            assert_ne!(outcome, GuardOutcome::GuardInvalid);
        } else {
            assert_eq!(
                outcome,
                GuardOutcome::GuardInvalid,
                "unknown guard {g:?} → GuardInvalid"
            );
            assert!(matches!(
                check_guard(g, &d, None),
                Err(GuardError::InvalidSkillDef(_))
            ));
        }
    }
}

#[test]
fn recognized_guards_are_not_invalid() {
    let mut d = Document::new("d", "prd", "t");
    d.body = "## Summary\n\nReal filled content here.\n\n## Tasks\n\n- [x] a\n".into();
    for g in [
        "has_sections:Summary",
        "checklist_complete",
        "checklist_complete:Tasks",
    ] {
        assert!(guard_recognized(g));
        let outcome = guard_outcome_for(g, &d, None);
        assert_ne!(outcome, GuardOutcome::GuardInvalid, "{g} is recognized");
    }
}

#[test]
fn upstream_approved_requires_injected_checker() {
    let d = Document::new("d", "prd", "t");
    // Missing checker → deterministic InvalidSkillDef, never panic (contract C1).
    assert!(matches!(
        check_guard("upstream_approved", &d, None),
        Err(GuardError::InvalidSkillDef(_))
    ));
    // Injected checker → its GuardResult flows back.
    let r = check_guard("upstream_approved", &d, Some(&AlwaysApprove)).unwrap();
    assert!(r.passed);
}

#[test]
fn composed_guard_all_must_pass() {
    let mut d = Document::new("d", "prd", "t");
    d.body = "## Summary\n\nFilled.\n\n## Tasks\n\n- [x] a\n- [x] b\n".into();
    let ok = check_guard("has_sections:Summary;checklist_complete:Tasks", &d, None).unwrap();
    assert!(ok.passed, "both satisfied → pass: {}", ok.message);

    d.body = "## Summary\n\nFilled.\n\n## Tasks\n\n- [x] a\n- [ ] b\n".into();
    let bad = check_guard("has_sections:Summary;checklist_complete:Tasks", &d, None).unwrap();
    assert!(!bad.passed, "one fragment fails → fail");
}
