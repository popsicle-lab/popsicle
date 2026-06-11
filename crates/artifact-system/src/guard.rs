//! Guard checks over a [`Document`], plus the upstream-approval **port**.
//!
//! Mirrors:
//! - `acceptance.intent` › `GuardChecklistCompleteIffNoUnchecked`: the pure
//!   checklist outcome is `GuardPass` iff `checkedBoxes == totalBoxes`.
//! - `invariants.intent` › `GuardResultIsTotal` / `EvaluateGuard`: any
//!   **unknown** guard string evaluates to `GuardInvalid` deterministically and
//!   never panics (`(!recognized) ==> outcome == GuardInvalid`).
//! - ADR-004 contract 1: [`UpstreamApprovalChecker`] is defined here and its
//!   signature references only artifact-owned types (`Document` / `GuardResult`),
//!   so `skill-runtime → artifact-system` stays acyclic. A missing checker yields
//!   a deterministic `InvalidSkillDef`, never a panic.

use crate::document::Document;

/// Tri-state guard outcome. Mirrors `enum GuardOutcome`/`GOutcome` in the intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardOutcome {
    GuardPass,
    GuardFail,
    GuardInvalid,
}

/// Outcome of a guard evaluation (legacy `GuardResult` shape, sans the engine
/// types the port must not reference).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardResult {
    pub passed: bool,
    pub guard_name: String,
    pub message: String,
}

/// Guard evaluation error. The single variant mirrors legacy
/// `PopsicleError::InvalidSkillDef` (guard.rs:92-95) — returned (never panicked)
/// for any unrecognized guard string or a missing upstream checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardError {
    InvalidSkillDef(String),
}

/// Pure checklist-completeness model. Mirrors `type GuardCheck` in
/// `acceptance.intent` (`totalBoxes`, `checkedBoxes`, `outcome`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardCheck {
    pub total_boxes: u32,
    pub checked_boxes: u32,
}

/// Artifact-owned **port** for upstream-approval guards (ADR-004 contract 1).
///
/// The implementation (which needs pipeline/run/registry state) lives in
/// `skill-runtime` and is injected; defining the trait here — over only
/// `Document` / `GuardResult` — keeps the dependency arrow one-directional.
pub trait UpstreamApprovalChecker {
    fn check_upstream_approved(&self, doc: &Document) -> GuardResult;
}

/// Pure checklist outcome: `GuardPass` iff every box is checked.
///
/// This is the logical core verified by `GuardChecklistCompleteIffNoUnchecked`
/// (`(outcome == GuardPass) == (checked == total)`), independent of the guard's
/// orthogonal "a section with zero boxes is not complete" policy (see
/// [`check_guard`]). Saturates `checked` to `total` to keep `checked <= total`.
pub fn checklist_outcome(total: u32, checked: u32) -> GuardOutcome {
    let checked = checked.min(total);
    if checked == total {
        GuardOutcome::GuardPass
    } else {
        GuardOutcome::GuardFail
    }
}

/// Whether `guard` names a recognized guard type. Mirrors the `recognized`
/// field of `GuardEval` in `invariants.intent`.
/// Whether a single guard fragment (no `;`) names a recognized guard.
fn fragment_recognized(g: &str) -> bool {
    let g = g.trim();
    g == "upstream_approved"
        || g.starts_with("has_sections:")
        || g == "checklist_complete"
        || g.starts_with("checklist_complete:")
}

/// Whether `guard` is a recognized guard expression. A composed expression
/// (`A;B`) is recognized iff it has at least one non-empty fragment and **every**
/// non-empty fragment is individually recognized — mirroring [`check_guard`],
/// which returns `InvalidSkillDef` as soon as any fragment is unknown.
pub fn guard_recognized(guard: &str) -> bool {
    let mut any = false;
    for frag in guard.split(';') {
        if frag.trim().is_empty() {
            continue;
        }
        any = true;
        if !fragment_recognized(frag) {
            return false;
        }
    }
    any
}

/// Collapse [`check_guard`] into a [`GuardOutcome`] for the totality invariant:
/// `Ok(passed)` → `GuardPass`/`GuardFail`, `Err(InvalidSkillDef)` → `GuardInvalid`.
/// By construction `(!guard_recognized(g)) ==> GuardInvalid`, and it never panics.
pub fn guard_outcome_for(
    guard: &str,
    doc: &Document,
    upstream: Option<&dyn UpstreamApprovalChecker>,
) -> GuardOutcome {
    match check_guard(guard, doc, upstream) {
        Ok(r) if r.passed => GuardOutcome::GuardPass,
        Ok(_) => GuardOutcome::GuardFail,
        Err(GuardError::InvalidSkillDef(_)) => GuardOutcome::GuardInvalid,
    }
}

/// Evaluate a guard expression against `doc`.
///
/// Multiple guards may be combined with `;` (all must pass). Supported types:
/// `upstream_approved`, `has_sections:<A>,<B>`, `checklist_complete`,
/// `checklist_complete:<Section>`. Any other token (or an empty expression)
/// yields `Err(InvalidSkillDef)`. **Total**: never panics for any input.
pub fn check_guard(
    guard: &str,
    doc: &Document,
    upstream: Option<&dyn UpstreamApprovalChecker>,
) -> Result<GuardResult, GuardError> {
    let parts: Vec<&str> = guard
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(GuardError::InvalidSkillDef(format!(
            "Unknown guard type: {guard}"
        )));
    }

    if parts.len() > 1 {
        let mut failed = Vec::new();
        for part in &parts {
            let result = check_single_guard(part, doc, upstream)?;
            if !result.passed {
                failed.push(result.message);
            }
        }
        return Ok(if failed.is_empty() {
            GuardResult {
                passed: true,
                guard_name: guard.to_string(),
                message: "All guards passed.".to_string(),
            }
        } else {
            GuardResult {
                passed: false,
                guard_name: guard.to_string(),
                message: failed.join(". "),
            }
        });
    }

    check_single_guard(parts[0], doc, upstream)
}

fn check_single_guard(
    guard: &str,
    doc: &Document,
    upstream: Option<&dyn UpstreamApprovalChecker>,
) -> Result<GuardResult, GuardError> {
    let guard = guard.trim();

    if guard == "upstream_approved" {
        return match upstream {
            Some(checker) => Ok(checker.check_upstream_approved(doc)),
            None => Err(GuardError::InvalidSkillDef(
                "upstream_approved requires an UpstreamApprovalChecker".to_string(),
            )),
        };
    }

    if let Some(sections_str) = guard.strip_prefix("has_sections:") {
        let required: Vec<&str> = sections_str.split(',').map(|s| s.trim()).collect();
        return Ok(check_has_sections(doc, &required));
    }

    if guard == "checklist_complete" {
        return Ok(check_checklist_complete(doc, None));
    }

    if let Some(section) = guard.strip_prefix("checklist_complete:") {
        return Ok(check_checklist_complete(doc, Some(section.trim())));
    }

    Err(GuardError::InvalidSkillDef(format!(
        "Unknown guard type: {guard}"
    )))
}

fn check_has_sections(doc: &Document, required: &[&str]) -> GuardResult {
    let mut missing = Vec::new();
    let mut empty = Vec::new();

    for &section in required {
        let header = format!("## {section}");
        if let Some(pos) = doc.body.find(&header) {
            let after = &doc.body[pos + header.len()..];
            if is_template_placeholder(&extract_section_content(after)) {
                empty.push(section.to_string());
            }
        } else {
            missing.push(section.to_string());
        }
    }

    let name = format!("has_sections:{}", required.join(","));
    if missing.is_empty() && empty.is_empty() {
        GuardResult {
            passed: true,
            guard_name: name,
            message: "All required sections present and filled.".to_string(),
        }
    } else {
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!("Missing sections: {}", missing.join(", ")));
        }
        if !empty.is_empty() {
            reasons.push(format!(
                "Sections still have template placeholders: {}",
                empty.join(", ")
            ));
        }
        GuardResult {
            passed: false,
            guard_name: name,
            message: reasons.join(". "),
        }
    }
}

fn check_checklist_complete(doc: &Document, section: Option<&str>) -> GuardResult {
    let (text, name) = match section {
        Some(name) => {
            let header = format!("## {name}");
            match doc.body.find(&header) {
                Some(pos) => {
                    let after = &doc.body[pos + header.len()..];
                    (
                        extract_section_content(after),
                        format!("checklist_complete:{name}"),
                    )
                }
                None => {
                    return GuardResult {
                        passed: false,
                        guard_name: format!("checklist_complete:{name}"),
                        message: format!("Section '{name}' not found in document."),
                    }
                }
            }
        }
        None => (doc.body.clone(), "checklist_complete".to_string()),
    };

    let (checked, unchecked) = count_checkboxes(&text);
    let total = checked + unchecked;

    // Legacy policy: a section with zero checkboxes is not "complete". This is an
    // emptiness gate orthogonal to the pure `checklist_outcome` logic.
    if total == 0 {
        return GuardResult {
            passed: false,
            guard_name: name,
            message: "No checklist items found.".to_string(),
        };
    }

    match checklist_outcome(total as u32, checked as u32) {
        GuardOutcome::GuardPass => GuardResult {
            passed: true,
            guard_name: name,
            message: format!("All {total} checklist items complete."),
        },
        _ => GuardResult {
            passed: false,
            guard_name: name,
            message: format!("{unchecked}/{total} checklist items still unchecked."),
        },
    }
}

/// Count `- [x]`/`- [X]` (checked) and `- [ ]` (unchecked) checkboxes.
/// Mirrors legacy `count_checkboxes` (guard.rs:270-282).
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

/// Content between an `## H2` header line and the next `## ` header.
/// Mirrors legacy `markdown::extract_section_content`.
fn extract_section_content(after_header: &str) -> String {
    let mut content = Vec::new();
    for (i, line) in after_header.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if line.starts_with("## ") {
            break;
        }
        content.push(line);
    }
    content.join("\n").trim().to_string()
}

/// Heuristic: does `content` look like an unfilled template?
/// Mirrors legacy `markdown::is_template_placeholder`.
fn is_template_placeholder(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return true;
    }
    const PATTERNS: [&str; 9] = [
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
    let substantive = trimmed.lines().filter(|l| {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') {
            return false;
        }
        !PATTERNS.iter().any(|p| l.contains(p))
    });
    substantive.count() == 0
}
