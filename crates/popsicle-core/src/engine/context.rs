/// Attention-aware context assembly for upstream document injection.
///
/// Sorts inputs by relevance (Low → Medium → High) and applies content
/// extraction based on relevance level, optimizing for LLM attention
/// distribution (U-shaped: beginning and end receive most attention).
use serde::Serialize;

use crate::engine::markdown;
use crate::model::Relevance;

/// A single upstream document ready for context injection.
#[derive(Debug, Clone)]
pub struct ContextInput {
    pub artifact_type: String,
    pub title: String,
    pub status: String,
    pub body: String,
    pub relevance: Relevance,
    pub sections: Option<Vec<String>>,
}

/// One processed part of the assembled context.
#[derive(Debug, Clone, Serialize)]
pub struct ContextPart {
    pub artifact_type: String,
    pub title: String,
    pub status: String,
    pub relevance: String,
    pub content: String,
}

/// The fully assembled input context, sorted and extracted.
#[derive(Debug, Clone)]
pub struct AssembledContext {
    pub parts: Vec<ContextPart>,
    pub full_text: String,
}

fn relevance_label(r: Relevance) -> &'static str {
    match r {
        Relevance::Low => "Background",
        Relevance::Medium => "Reference",
        Relevance::High => "Primary",
    }
}

/// Assemble upstream documents into an attention-optimized context string.
///
/// Ordering: Low (background, summaries) → Medium (selected sections) → High (full text).
/// High-relevance content is placed closest to the prompt instruction (end of context block).
pub fn assemble_input_context(mut inputs: Vec<ContextInput>) -> AssembledContext {
    inputs.sort_by_key(|i| i.relevance);

    let parts: Vec<ContextPart> = inputs
        .iter()
        .map(|input| {
            let content = match input.relevance {
                Relevance::Low => markdown::extract_summary(&input.body),
                Relevance::Medium => {
                    if let Some(ref sections) = input.sections {
                        let extracted = markdown::extract_sections(&input.body, sections);
                        if extracted.is_empty() {
                            input.body.trim().to_string()
                        } else {
                            extracted
                        }
                    } else {
                        input.body.trim().to_string()
                    }
                }
                Relevance::High => input.body.trim().to_string(),
            };

            ContextPart {
                artifact_type: input.artifact_type.clone(),
                title: input.title.clone(),
                status: input.status.clone(),
                relevance: input.relevance.to_string(),
                content,
            }
        })
        .collect();

    let text_parts: Vec<String> = parts
        .iter()
        .map(|p| {
            let label = relevance_label(match p.relevance.as_str() {
                "low" => Relevance::Low,
                "medium" => Relevance::Medium,
                _ => Relevance::High,
            });
            format!(
                "### [{}] {} — {} [{}]\n\n{}",
                label, p.artifact_type, p.title, p.status, p.content
            )
        })
        .collect();

    let full_text = text_parts.join("\n\n---\n\n");

    AssembledContext { parts, full_text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_low_medium_high() {
        let inputs = vec![
            ContextInput {
                artifact_type: "rfc".into(),
                title: "RFC".into(),
                status: "accepted".into(),
                body: "## Summary\n\nRFC content.\n".into(),
                relevance: Relevance::High,
                sections: None,
            },
            ContextInput {
                artifact_type: "domain-model".into(),
                title: "Domain".into(),
                status: "approved".into(),
                body: "## Entities\n\nUser, Order.\n\n## Boundaries\n\nCore domain.\n".into(),
                relevance: Relevance::Low,
                sections: None,
            },
            ContextInput {
                artifact_type: "prd".into(),
                title: "PRD".into(),
                status: "approved".into(),
                body: "## Problem\n\nThe problem.\n\n## Goals\n\nGoal 1.\n".into(),
                relevance: Relevance::Medium,
                sections: Some(vec!["Problem".into()]),
            },
        ];

        let result = assemble_input_context(inputs);
        assert_eq!(result.parts.len(), 3);
        assert_eq!(result.parts[0].relevance, "low");
        assert_eq!(result.parts[1].relevance, "medium");
        assert_eq!(result.parts[2].relevance, "high");
    }

    #[test]
    fn test_low_relevance_extracts_summary() {
        let inputs = vec![ContextInput {
            artifact_type: "domain-model".into(),
            title: "Domain".into(),
            status: "approved".into(),
            body:
                "Overview of the domain.\n\n## Entities\n\nUser, Order.\n\n## Boundaries\n\nCore.\n"
                    .into(),
            relevance: Relevance::Low,
            sections: None,
        }];

        let result = assemble_input_context(inputs);
        assert!(result.parts[0].content.contains("Overview of the domain."));
        assert!(result.parts[0].content.contains("- Entities"));
        assert!(!result.parts[0].content.contains("User, Order."));
    }

    #[test]
    fn test_medium_relevance_extracts_sections() {
        let inputs = vec![ContextInput {
            artifact_type: "prd".into(),
            title: "PRD".into(),
            status: "approved".into(),
            body: "## Problem\n\nThe problem statement.\n\n## Goals\n\nGoal list.\n\n## Scope\n\nOut of scope.\n".into(),
            relevance: Relevance::Medium,
            sections: Some(vec!["Problem".into(), "Goals".into()]),
        }];

        let result = assemble_input_context(inputs);
        assert!(result.parts[0].content.contains("The problem statement."));
        assert!(result.parts[0].content.contains("Goal list."));
        assert!(!result.parts[0].content.contains("Out of scope."));
    }

    #[test]
    fn test_medium_no_sections_falls_back_to_full() {
        let inputs = vec![ContextInput {
            artifact_type: "prd".into(),
            title: "PRD".into(),
            status: "approved".into(),
            body: "## Problem\n\nContent.\n".into(),
            relevance: Relevance::Medium,
            sections: None,
        }];

        let result = assemble_input_context(inputs);
        assert!(result.parts[0].content.contains("Content."));
    }

    #[test]
    fn test_high_relevance_full_body() {
        let body = "## Summary\n\nFull RFC content.\n\n## Proposal\n\nDetailed proposal.\n";
        let inputs = vec![ContextInput {
            artifact_type: "rfc".into(),
            title: "RFC".into(),
            status: "accepted".into(),
            body: body.into(),
            relevance: Relevance::High,
            sections: None,
        }];

        let result = assemble_input_context(inputs);
        assert!(result.parts[0].content.contains("Full RFC content."));
        assert!(result.parts[0].content.contains("Detailed proposal."));
    }

    #[test]
    fn test_labels_in_full_text() {
        let inputs = vec![
            ContextInput {
                artifact_type: "domain".into(),
                title: "D".into(),
                status: "approved".into(),
                body: "## X\n\nContent.\n".into(),
                relevance: Relevance::Low,
                sections: None,
            },
            ContextInput {
                artifact_type: "rfc".into(),
                title: "R".into(),
                status: "accepted".into(),
                body: "Full body.\n".into(),
                relevance: Relevance::High,
                sections: None,
            },
        ];

        let result = assemble_input_context(inputs);
        assert!(result.full_text.contains("[Background]"));
        assert!(result.full_text.contains("[Primary]"));
    }

    #[test]
    fn test_empty_inputs() {
        let result = assemble_input_context(vec![]);
        assert!(result.parts.is_empty());
        assert!(result.full_text.is_empty());
    }
}
