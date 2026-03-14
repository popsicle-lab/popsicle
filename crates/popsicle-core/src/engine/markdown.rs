//! Markdown utilities for section extraction and summarization.

/// Extract the content between an H2 header and the next H2 header.
/// `after_header` should be the text immediately following the `## Title` line.
pub fn extract_section_content(after_header: &str) -> String {
    let lines: Vec<&str> = after_header.lines().collect();
    let mut content_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if line.starts_with("## ") {
            break;
        }
        content_lines.push(*line);
    }

    content_lines.join("\n").trim().to_string()
}

/// Heuristic: check if content looks like an unfilled template.
pub fn is_template_placeholder(content: &str) -> bool {
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

/// Extract only the named H2 sections from a Markdown body.
/// Returns a string with the matching sections (header + content) concatenated.
pub fn extract_sections(body: &str, section_names: &[String]) -> String {
    let mut result_parts = Vec::new();

    for name in section_names {
        let header = format!("## {}", name);
        if let Some(pos) = body.find(&header) {
            let after_header = &body[pos + header.len()..];
            let content = extract_section_content(after_header);
            if !content.is_empty() {
                result_parts.push(format!("{}\n\n{}", header, content));
            }
        }
    }

    result_parts.join("\n\n")
}

/// Extract a summary from a Markdown body:
/// - Content before the first H2 (if any substantive text exists), plus
/// - A list of all H2 headings as a table-of-contents.
pub fn extract_summary(body: &str) -> String {
    let mut preamble = String::new();
    let mut headings = Vec::new();

    let mut in_preamble = true;
    for line in body.lines() {
        if line.starts_with("## ") {
            in_preamble = false;
            headings.push(line.trim_start_matches("## ").to_string());
        } else if in_preamble {
            preamble.push_str(line);
            preamble.push('\n');
        }
    }

    let preamble = preamble.trim().to_string();

    let mut parts = Vec::new();
    if !preamble.is_empty() && !is_template_placeholder(&preamble) {
        parts.push(preamble);
    }
    if !headings.is_empty() {
        let toc = headings
            .iter()
            .map(|h| format!("- {}", h))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("**Sections:** \n{}", toc));
    }

    parts.join("\n\n")
}

/// Replace or insert an H2 section in a Markdown document.
///
/// If the section exists, replaces its content (or appends when `append` is true).
/// If it doesn't exist, inserts a new section before `## Notes` (or at the end).
pub fn upsert_section(doc: &str, section_name: &str, content: &str, append: bool) -> String {
    let header = format!("## {}", section_name);
    let lines: Vec<&str> = doc.lines().collect();

    let section_start = lines.iter().position(|l| l.trim() == header);

    if let Some(start_idx) = section_start {
        let content_start = start_idx + 1;
        let section_end = lines[content_start..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map(|p| content_start + p)
            .unwrap_or(lines.len());

        let mut result: Vec<String> = lines[..content_start]
            .iter()
            .map(|s| s.to_string())
            .collect();

        if append {
            let existing_text = lines[content_start..section_end].join("\n");
            let existing_trimmed = existing_text.trim_end();
            if !existing_trimmed.is_empty() {
                result.push(String::new());
                for l in existing_trimmed.lines() {
                    result.push(l.to_string());
                }
            }
            result.push(String::new());
            for l in content.lines() {
                result.push(l.to_string());
            }
        } else {
            result.push(String::new());
            for l in content.lines() {
                result.push(l.to_string());
            }
        }

        result.push(String::new());
        result.extend(lines[section_end..].iter().map(|s| s.to_string()));

        result.join("\n")
    } else {
        let insert_before = lines.iter().position(|l| l.trim() == "## Notes");
        let insert_idx = insert_before.unwrap_or(lines.len());

        let mut result: Vec<String> = lines[..insert_idx].iter().map(|s| s.to_string()).collect();

        if insert_idx > 0 && !lines[insert_idx - 1].trim().is_empty() {
            result.push(String::new());
        }
        result.push(header);
        result.push(String::new());
        for l in content.lines() {
            result.push(l.to_string());
        }
        result.push(String::new());

        result.extend(lines[insert_idx..].iter().map(|s| s.to_string()));

        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section_content_basic() {
        let text = "\n\nSome real content here.\nMore details.\n\n## Next Section\n\nOther stuff.";
        let content = extract_section_content(text);
        assert!(content.contains("Some real content here."));
        assert!(!content.contains("Next Section"));
    }

    #[test]
    fn test_extract_section_content_end_of_doc() {
        let text = "\n\nFinal section content.\nNo more headers.";
        let content = extract_section_content(text);
        assert!(content.contains("Final section content."));
    }

    #[test]
    fn test_extract_sections_multiple() {
        let body = "## Intro\n\nHello world.\n\n## Details\n\nSome details.\n\n## Conclusion\n\nThe end.\n";
        let result = extract_sections(body, &["Intro".into(), "Conclusion".into()]);
        assert!(result.contains("## Intro"));
        assert!(result.contains("Hello world."));
        assert!(result.contains("## Conclusion"));
        assert!(result.contains("The end."));
        assert!(!result.contains("Details"));
    }

    #[test]
    fn test_extract_sections_missing() {
        let body = "## Intro\n\nHello.\n";
        let result = extract_sections(body, &["Nonexistent".into()]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_summary_with_preamble() {
        let body = "This is the overview.\n\n## Goals\n\nGoal 1.\n\n## Scope\n\nScope details.\n";
        let summary = extract_summary(body);
        assert!(summary.contains("This is the overview."));
        assert!(summary.contains("- Goals"));
        assert!(summary.contains("- Scope"));
        assert!(!summary.contains("Goal 1."));
    }

    #[test]
    fn test_extract_summary_no_preamble() {
        let body = "## Goals\n\nGoal 1.\n\n## Scope\n\nScope details.\n";
        let summary = extract_summary(body);
        assert!(!summary.contains("Goal 1."));
        assert!(summary.contains("- Goals"));
        assert!(summary.contains("- Scope"));
    }

    #[test]
    fn test_extract_summary_template_preamble() {
        let body = "Describe the project...\n\n## Goals\n\nGoal 1.\n";
        let summary = extract_summary(body);
        assert!(!summary.contains("Describe the project"));
        assert!(summary.contains("- Goals"));
    }

    #[test]
    fn test_is_template_placeholder() {
        assert!(is_template_placeholder(""));
        assert!(is_template_placeholder("..."));
        assert!(is_template_placeholder("Describe the purpose"));
        assert!(is_template_placeholder("[Name]"));
        assert!(!is_template_placeholder(
            "We use Redis for caching with a 5-minute TTL."
        ));
    }

    #[test]
    fn test_upsert_section_replace_existing() {
        let doc = "## Tech\n\n- Rust\n\n## Notes\n\nUser notes.\n";
        let result = upsert_section(doc, "Tech", "- Go\n- Python", false);
        assert!(result.contains("## Tech\n\n- Go\n- Python"));
        assert!(!result.contains("- Rust"));
        assert!(result.contains("## Notes"));
    }

    #[test]
    fn test_upsert_section_append_existing() {
        let doc = "## Tech\n\n- Rust\n\n## Notes\n\nUser notes.\n";
        let result = upsert_section(doc, "Tech", "- Go", true);
        assert!(result.contains("- Rust"));
        assert!(result.contains("- Go"));
        assert!(result.contains("## Notes"));
    }

    #[test]
    fn test_upsert_section_insert_new_before_notes() {
        let doc = "## Tech\n\n- Rust\n\n## Notes\n\nUser notes.\n";
        let result = upsert_section(doc, "Architecture", "- Layered", false);
        let arch_pos = result.find("## Architecture").unwrap();
        let notes_pos = result.find("## Notes").unwrap();
        assert!(arch_pos < notes_pos);
        assert!(result.contains("- Layered"));
    }

    #[test]
    fn test_upsert_section_insert_new_at_end() {
        let doc = "## Tech\n\n- Rust\n";
        let result = upsert_section(doc, "Architecture", "- Layered", false);
        assert!(result.contains("## Architecture"));
        assert!(result.contains("- Layered"));
    }
}
