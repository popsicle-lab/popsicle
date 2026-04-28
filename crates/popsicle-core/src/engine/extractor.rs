use regex::Regex;
use serde_json::json;

use crate::model::document::Document;
use crate::model::work_item::{WorkItem, WorkItemKind};

/// Extract user stories from a PRD document as WorkItems with kind=Story.
///
/// `fields` is populated with `persona`, `goal`, `benefit`, and `acceptance` (Vec<String>).
pub fn extract_user_stories(doc: &Document) -> Vec<WorkItem> {
    let body = &doc.body;
    let mut stories = Vec::new();

    let section = extract_section(body, "User Stories & Acceptance Criteria")
        .or_else(|| extract_section(body, "User Stories"));
    let section = match section {
        Some(s) => s,
        None => return stories,
    };

    let h3_re = Regex::new(r"(?m)^###\s+(?:Story\s+\d+:\s*)?(.+)$").unwrap();
    let as_a_re = Regex::new(
        r"(?i)\*\*As a\*\*\s*(.+?)\s*\*\*I want to\*\*\s*(.+?)\s*\*\*So that\*\*\s*(.+)",
    )
    .unwrap();
    let ac_re = Regex::new(r"(?m)^-\s*\[[ x]\]\s*(.+)$").unwrap();

    let h3_matches: Vec<_> = h3_re.find_iter(&section).collect();

    for (i, m) in h3_matches.iter().enumerate() {
        let caps = h3_re.captures(m.as_str()).unwrap();
        let title = caps[1].trim().to_string();

        let subsection_end = h3_matches
            .get(i + 1)
            .map(|n| n.start())
            .unwrap_or(section.len());
        let subsection = &section[m.start()..subsection_end];

        let mut wi = WorkItem::new(String::new(), WorkItemKind::Story, &title);

        let (mut persona, mut goal, mut benefit) = (String::new(), String::new(), String::new());
        if let Some(as_caps) = as_a_re.captures(subsection) {
            persona = as_caps[1].trim().to_string();
            goal = as_caps[2].trim().to_string();
            benefit = as_caps[3].trim().to_string();
        }

        let acceptance: Vec<String> = ac_re
            .captures_iter(subsection)
            .map(|c| c[1].trim().to_string())
            .collect();

        wi.description = format!("As a {persona} I want to {goal} so that {benefit}");
        wi.set_field("persona", json!(persona));
        wi.set_field("goal", json!(goal));
        wi.set_field("benefit", json!(benefit));
        wi.set_field("acceptance", json!(acceptance));

        stories.push(wi);
    }

    stories
}

/// Extract test cases from a test-spec document as WorkItems with kind=TestCase.
///
/// `fields` is populated with `test_type`, `priority_level` (P0/P1/P2), and `steps` (Vec<String>).
pub fn extract_test_cases(doc: &Document, test_type: &str) -> Vec<WorkItem> {
    let body = &doc.body;
    let mut cases = Vec::new();

    let h3_re = Regex::new(r"(?m)^###\s+(.+)$").unwrap();
    let step_re = Regex::new(r"(?m)^-\s*\[[ x]\]\s*(.+)$").unwrap();
    let bullet_re = Regex::new(r"(?m)^-\s+(.+)$").unwrap();

    let h3_matches: Vec<_> = h3_re.find_iter(body).collect();

    let priority_context = detect_priority_context(body);

    for (i, m) in h3_matches.iter().enumerate() {
        let caps = h3_re.captures(m.as_str()).unwrap();
        let title = caps[1].trim().to_string();

        if is_section_header(&title) {
            continue;
        }

        let subsection_end = h3_matches
            .get(i + 1)
            .map(|n| n.start())
            .unwrap_or(body.len());
        let subsection = &body[m.start()..subsection_end];

        let mut wi = WorkItem::new(String::new(), WorkItemKind::TestCase, &title);

        let mut steps: Vec<String> = step_re
            .captures_iter(subsection)
            .map(|c| c[1].trim().to_string())
            .collect();

        if steps.is_empty() {
            steps = bullet_re
                .captures_iter(subsection)
                .map(|c| c[1].trim().to_string())
                .collect();
        }

        let priority_level = infer_priority(m.start(), &priority_context);

        wi.set_field("test_type", json!(test_type));
        wi.set_field("priority_level", json!(priority_level));
        wi.set_field("steps", json!(steps));

        cases.push(wi);
    }

    cases
}

/// Extract bugs from a bug-report document as WorkItems with kind=Bug.
///
/// `fields` is populated with `severity`, `expected_behavior`, `actual_behavior`,
/// `steps_to_reproduce` (Vec<String>).
pub fn extract_bugs(doc: &Document) -> Vec<WorkItem> {
    let body = &doc.body;
    let mut bugs = Vec::new();

    let bug_re = Regex::new(r"(?m)^###\s+(?:BUG-\w+:\s*)?(.+)$").unwrap();
    let severity_re = Regex::new(r"(?i)\*\*Severity\*\*:\s*(\w+)").unwrap();
    let expected_re = Regex::new(r"(?i)\*\*Expected\*\*:\s*(.+)").unwrap();
    let actual_re = Regex::new(r"(?i)\*\*Actual\*\*:\s*(.+)").unwrap();
    let steps_re = Regex::new(r"(?i)\*\*Steps to reproduce\*\*:\s*(.+)").unwrap();

    let section = extract_section(body, "New Bugs").unwrap_or_else(|| body.to_string());

    let h3_matches: Vec<_> = bug_re.find_iter(&section).collect();

    for (i, m) in h3_matches.iter().enumerate() {
        let caps = bug_re.captures(m.as_str()).unwrap();
        let title = caps[1].trim().to_string();

        if title.starts_with('[') || title.contains("Title") {
            continue;
        }

        let subsection_end = h3_matches
            .get(i + 1)
            .map(|n| n.start())
            .unwrap_or(section.len());
        let subsection = &section[m.start()..subsection_end];

        let severity = severity_re
            .captures(subsection)
            .map(|c| c[1].trim().to_lowercase())
            .unwrap_or_else(|| "major".to_string());

        let mut wi = WorkItem::new(String::new(), WorkItemKind::Bug, &title);
        wi.set_field("severity", json!(severity));

        if let Some(caps) = expected_re.captures(subsection) {
            wi.set_field("expected_behavior", json!(caps[1].trim()));
        }
        if let Some(caps) = actual_re.captures(subsection) {
            wi.set_field("actual_behavior", json!(caps[1].trim()));
        }
        if let Some(caps) = steps_re.captures(subsection) {
            let steps: Vec<String> = caps[1]
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            wi.set_field("steps_to_reproduce", json!(steps));
        }

        bugs.push(wi);
    }

    bugs
}

fn extract_section(body: &str, heading: &str) -> Option<String> {
    let h2_re = Regex::new(r"(?m)^##\s+(.+)$").unwrap();
    let mut start = None;
    let mut end = body.len();

    for m in h2_re.find_iter(body) {
        let caps = h2_re.captures(m.as_str()).unwrap();
        let h = caps[1].trim();
        if start.is_some() {
            end = m.start();
            break;
        }
        if h == heading {
            start = Some(m.end());
        }
    }

    start.map(|s| body[s..end].to_string())
}

struct PriorityRange {
    start: usize,
    end: usize,
    priority: String,
}

fn detect_priority_context(body: &str) -> Vec<PriorityRange> {
    let mut ranges = Vec::new();
    let h2_re = Regex::new(r"(?m)^##\s+(.+)$").unwrap();
    let p_re = Regex::new(r"(?i)\b(P0|P1|P2)\b").unwrap();

    let h2_matches: Vec<_> = h2_re.find_iter(body).collect();

    for (i, m) in h2_matches.iter().enumerate() {
        let caps = h2_re.captures(m.as_str()).unwrap();
        let heading = caps[1].trim();
        if let Some(p_caps) = p_re.captures(heading) {
            let priority = p_caps[1].to_uppercase();
            let end = h2_matches
                .get(i + 1)
                .map(|n| n.start())
                .unwrap_or(body.len());
            ranges.push(PriorityRange {
                start: m.start(),
                end,
                priority,
            });
        }
    }

    ranges
}

fn infer_priority(pos: usize, ranges: &[PriorityRange]) -> String {
    for r in ranges {
        if pos >= r.start && pos < r.end {
            return r.priority.clone();
        }
    }
    "P1".to_string()
}

fn is_section_header(title: &str) -> bool {
    let lower = title.to_lowercase();
    matches!(
        lower.as_str(),
        "summary"
            | "overview"
            | "background"
            | "scope"
            | "references"
            | "statistics"
            | "bug registry"
            | "new bugs"
            | "appendix"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Document;

    fn make_doc(body: &str) -> Document {
        let mut doc = Document::new("test", "Test Doc", "test-skill", "run-1", "test-spec");
        doc.body = body.to_string();
        doc
    }

    #[test]
    fn test_extract_user_stories() {
        let body = r#"## User Stories & Acceptance Criteria

### Story 1: Create Project

**As a** developer **I want to** initialize a project **So that** I can start quickly

**Acceptance Criteria:**
- [ ] Config file is generated
- [ ] Directory structure is created

### Story 2: Run Tests

**As a** QA engineer **I want to** run tests **So that** I verify quality

- [ ] Tests execute successfully
"#;
        let doc = make_doc(body);
        let stories = extract_user_stories(&doc);
        assert_eq!(stories.len(), 2);
        assert_eq!(stories[0].title, "Create Project");
        assert_eq!(stories[0].field_str("persona"), Some("developer"));
        let acc = stories[0]
            .fields
            .get("acceptance")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(acc.len(), 2);
        assert_eq!(stories[1].title, "Run Tests");
    }

    #[test]
    fn test_extract_test_cases() {
        let body = r#"## P0 Critical Tests

### Login with valid credentials

- [ ] Navigate to login page
- [ ] Enter valid username
- [ ] Click submit
- [ ] Verify dashboard loads

## P1 Important Tests

### Search functionality

- [ ] Enter search term
- [ ] Verify results appear
"#;
        let doc = make_doc(body);
        let cases = extract_test_cases(&doc, "e2e");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].title, "Login with valid credentials");
        assert_eq!(cases[0].field_str("priority_level"), Some("P0"));
        let steps = cases[0]
            .fields
            .get("steps")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(cases[1].field_str("priority_level"), Some("P1"));
    }

    #[test]
    fn test_extract_bugs() {
        let body = r#"## New Bugs

### BUG-001: Login fails on Safari

**Severity**: critical
**Expected**: User logs in successfully
**Actual**: 500 error returned
**Steps to reproduce**: Open Safari; Navigate to /login; Enter credentials; Click submit

### BUG-002: Missing icon on dashboard

**Severity**: minor
**Expected**: Icon displays
**Actual**: Broken image
"#;
        let doc = make_doc(body);
        let bugs = extract_bugs(&doc);
        assert_eq!(bugs.len(), 2);
        assert_eq!(bugs[0].title, "Login fails on Safari");
        assert_eq!(bugs[0].field_str("severity"), Some("critical"));
        let steps = bugs[0]
            .fields
            .get("steps_to_reproduce")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(bugs[1].field_str("severity"), Some("minor"));
    }
}
