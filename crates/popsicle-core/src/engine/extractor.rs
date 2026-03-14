use regex::Regex;

use crate::model::bug::{Bug, BugSeverity};
use crate::model::document::Document;
use crate::model::story::{AcceptanceCriterion, UserStory};
use crate::model::testcase::{TestCase, TestPriority, TestType};

/// Extract user stories from a PRD document.
///
/// Parses the "User Stories & Acceptance Criteria" section, looking for:
/// - `### Story N: [Title]` or `### [Title]` headings
/// - `**As a** ... **I want to** ... **So that** ...` pattern
/// - `- [ ]` checklist items as acceptance criteria
pub fn extract_user_stories(doc: &Document) -> Vec<UserStory> {
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

        let mut story = UserStory::new(String::new(), &title);

        if let Some(as_caps) = as_a_re.captures(subsection) {
            story.persona = as_caps[1].trim().to_string();
            story.goal = as_caps[2].trim().to_string();
            story.benefit = as_caps[3].trim().to_string();
        }

        for ac_cap in ac_re.captures_iter(subsection) {
            let desc = ac_cap[1].trim().to_string();
            story
                .acceptance_criteria
                .push(AcceptanceCriterion::new(&desc));
        }

        story.description = format!(
            "As a {} I want to {} so that {}",
            story.persona, story.goal, story.benefit
        );

        stories.push(story);
    }

    stories
}

/// Extract test cases from a test-spec document.
///
/// Parses H3 headings as test case titles and checklist items as steps.
/// Attempts to infer priority from P0/P1/P2 section context.
pub fn extract_test_cases(doc: &Document, test_type: TestType) -> Vec<TestCase> {
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

        let mut tc = TestCase::new(String::new(), &title, test_type.clone());

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

        tc.steps = steps;
        tc.priority_level = infer_priority(m.start(), &priority_context);

        cases.push(tc);
    }

    cases
}

/// Extract bugs from a bug-report document.
///
/// Parses entries matching `### BUG-XXXX: [Title]` pattern with structured fields.
pub fn extract_bugs(doc: &Document) -> Vec<Bug> {
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
            .and_then(|c| c[1].trim().parse::<BugSeverity>().ok())
            .unwrap_or_default();

        let mut bug = Bug::new(String::new(), &title, severity);

        if let Some(caps) = expected_re.captures(subsection) {
            bug.expected_behavior = caps[1].trim().to_string();
        }
        if let Some(caps) = actual_re.captures(subsection) {
            bug.actual_behavior = caps[1].trim().to_string();
        }
        if let Some(caps) = steps_re.captures(subsection) {
            bug.steps_to_reproduce = caps[1]
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        bugs.push(bug);
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
    priority: TestPriority,
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
            let priority = p_caps[1].to_uppercase().parse().unwrap_or(TestPriority::P1);
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

fn infer_priority(pos: usize, ranges: &[PriorityRange]) -> TestPriority {
    for r in ranges {
        if pos >= r.start && pos < r.end {
            return r.priority.clone();
        }
    }
    TestPriority::P1
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
        let mut doc = Document::new("test", "Test Doc", "test-skill", "run-1");
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
        assert_eq!(stories[0].persona, "developer");
        assert_eq!(stories[0].acceptance_criteria.len(), 2);
        assert_eq!(stories[1].title, "Run Tests");
        assert_eq!(stories[1].acceptance_criteria.len(), 1);
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
        let cases = extract_test_cases(&doc, TestType::E2e);
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].title, "Login with valid credentials");
        assert_eq!(cases[0].priority_level, TestPriority::P0);
        assert_eq!(cases[0].steps.len(), 4);
        assert_eq!(cases[1].priority_level, TestPriority::P1);
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
        assert_eq!(bugs[0].severity, BugSeverity::Critical);
        assert_eq!(bugs[0].steps_to_reproduce.len(), 4);
        assert_eq!(bugs[1].severity, BugSeverity::Minor);
    }
}
