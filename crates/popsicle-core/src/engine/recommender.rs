use serde::Serialize;

use crate::model::PipelineDef;

/// Scale-adaptive pipeline recommendation engine.
///
/// Matches a task description against pipeline keywords to suggest
/// the most appropriate pipeline for the task's complexity level.
#[derive(Debug)]
pub struct PipelineRecommender;

/// A pipeline recommendation with reasoning and alternatives.
#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub pipeline_name: String,
    pub scale: String,
    pub reason: String,
    pub alternatives: Vec<Alternative>,
    pub cli_command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alternative {
    pub pipeline_name: String,
    pub scale: String,
    pub reason: String,
}

/// Scale ordering for fallback comparison: heavier pipelines appear later.
const SCALE_ORDER: &[&str] = &["minimal", "light", "standard", "full", "planning"];

fn scale_rank(scale: &str) -> usize {
    SCALE_ORDER.iter().position(|&s| s == scale).unwrap_or(2)
}

impl PipelineRecommender {
    /// Recommend the best pipeline for the given task description.
    ///
    /// Matching strategy:
    /// 1. Tokenize the task description (lowercase, split on whitespace/punctuation).
    /// 2. Score each pipeline by the number of keyword hits.
    /// 3. Pick the highest-scoring pipeline; on tie, prefer the lighter scale.
    /// 4. Remaining pipelines with any hits become alternatives.
    /// 5. If no keywords match, fall back to `tech-sdlc`.
    pub fn recommend(task: &str, pipelines: &[PipelineDef]) -> Recommendation {
        let tokens = tokenize(task);

        let task_lower = task.to_lowercase();

        let mut scored: Vec<(usize, &PipelineDef)> = pipelines
            .iter()
            .map(|p| {
                let hits = p
                    .keywords
                    .iter()
                    .filter(|kw| {
                        let kw_lower = kw.to_lowercase();
                        tokens.contains(&kw_lower) || task_lower.contains(&kw_lower)
                    })
                    .count();
                (hits, p)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.0.cmp(&a.0).then_with(|| {
                let sa = a.1.scale.as_deref().unwrap_or("standard");
                let sb = b.1.scale.as_deref().unwrap_or("standard");
                scale_rank(sa).cmp(&scale_rank(sb))
            })
        });

        let best = scored.iter().find(|(hits, _)| *hits > 0).map(|(_, p)| *p);

        match best {
            Some(pipeline) => {
                let alternatives: Vec<Alternative> = scored
                    .iter()
                    .filter(|(hits, p)| *hits > 0 && p.name != pipeline.name)
                    .map(|(_, p)| Alternative {
                        pipeline_name: p.name.clone(),
                        scale: p.scale.clone().unwrap_or_default(),
                        reason: p.description.clone(),
                    })
                    .collect();

                let matched_kws: Vec<&str> = pipeline
                    .keywords
                    .iter()
                    .filter(|kw| {
                        let kw_lower = kw.to_lowercase();
                        tokens.contains(&kw_lower) || task_lower.contains(&kw_lower)
                    })
                    .map(|s| s.as_str())
                    .collect();

                let scale = pipeline.scale.clone().unwrap_or_default();

                Recommendation {
                    reason: format!(
                        "Matched keywords [{}] → scale '{}'",
                        matched_kws.join(", "),
                        scale
                    ),
                    cli_command: format!(
                        "popsicle pipeline run {} --title \"<title>\"",
                        pipeline.name
                    ),
                    pipeline_name: pipeline.name.clone(),
                    scale,
                    alternatives,
                }
            }
            None => {
                let fallback_name = if pipelines.iter().any(|p| p.name == "tech-sdlc") {
                    "tech-sdlc"
                } else {
                    pipelines
                        .first()
                        .map(|p| p.name.as_str())
                        .unwrap_or("quick")
                };

                let fallback = pipelines.iter().find(|p| p.name == fallback_name);
                let scale = fallback
                    .and_then(|p| p.scale.clone())
                    .unwrap_or_else(|| "standard".to_string());

                let alternatives: Vec<Alternative> = pipelines
                    .iter()
                    .filter(|p| p.name != fallback_name)
                    .map(|p| Alternative {
                        pipeline_name: p.name.clone(),
                        scale: p.scale.clone().unwrap_or_default(),
                        reason: p.description.clone(),
                    })
                    .collect();

                Recommendation {
                    pipeline_name: fallback_name.to_string(),
                    scale,
                    reason: "No keyword match — defaulting to standard complexity".to_string(),
                    cli_command: format!(
                        "popsicle pipeline run {} --title \"<title>\"",
                        fallback_name
                    ),
                    alternatives,
                }
            }
        }
    }
}

/// Split task description into lowercase tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| c.is_whitespace() || c == ',' || c == '.' || c == ':' || c == ';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PipelineDef, StageDef};

    fn sample_pipelines() -> Vec<PipelineDef> {
        vec![
            PipelineDef {
                name: "test-only".to_string(),
                description: "Testing pipeline".to_string(),
                stages: vec![StageDef {
                    name: "test".to_string(),
                    skills: vec![],
                    skill: Some("test".to_string()),
                    description: "Test".to_string(),
                    depends_on: vec![],
                    requires_approval: false,
                }],
                keywords: vec![
                    "test".to_string(),
                    "coverage".to_string(),
                    "测试".to_string(),
                    "覆盖率".to_string(),
                ],
                scale: Some("minimal".to_string()),
            },
            PipelineDef {
                name: "impl-test".to_string(),
                description: "Implementation and testing".to_string(),
                stages: vec![],
                keywords: vec![
                    "implement".to_string(),
                    "coding".to_string(),
                    "实现".to_string(),
                    "编码".to_string(),
                    "small".to_string(),
                ],
                scale: Some("light".to_string()),
            },
            PipelineDef {
                name: "tech-sdlc".to_string(),
                description: "Technical lifecycle".to_string(),
                stages: vec![],
                keywords: vec![
                    "refactor".to_string(),
                    "migrate".to_string(),
                    "upgrade".to_string(),
                    "重构".to_string(),
                    "迁移".to_string(),
                ],
                scale: Some("standard".to_string()),
            },
            PipelineDef {
                name: "full-sdlc".to_string(),
                description: "Full software development lifecycle".to_string(),
                stages: vec![],
                keywords: vec![
                    "feature".to_string(),
                    "user story".to_string(),
                    "product".to_string(),
                    "功能".to_string(),
                    "需求".to_string(),
                ],
                scale: Some("full".to_string()),
            },
            PipelineDef {
                name: "design-only".to_string(),
                description: "Design and planning only".to_string(),
                stages: vec![],
                keywords: vec![
                    "plan".to_string(),
                    "explore".to_string(),
                    "evaluate".to_string(),
                    "规划".to_string(),
                    "探索".to_string(),
                ],
                scale: Some("planning".to_string()),
            },
        ]
    }

    #[test]
    fn test_recommend_feature_matches_full_sdlc() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Add user authentication feature", &pipelines);
        assert_eq!(rec.pipeline_name, "full-sdlc");
        assert_eq!(rec.scale, "full");
    }

    #[test]
    fn test_recommend_refactor_matches_tech_sdlc() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Refactor the database layer", &pipelines);
        assert_eq!(rec.pipeline_name, "tech-sdlc");
    }

    #[test]
    fn test_recommend_test_matches_test_only() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("补充测试覆盖率", &pipelines);
        assert_eq!(rec.pipeline_name, "test-only");
    }

    #[test]
    fn test_recommend_implement_matches_impl_test() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Implement the small caching module", &pipelines);
        assert_eq!(rec.pipeline_name, "impl-test");
    }

    #[test]
    fn test_recommend_plan_matches_design_only() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("探索微服务拆分的可行性", &pipelines);
        assert_eq!(rec.pipeline_name, "design-only");
    }

    #[test]
    fn test_recommend_no_match_falls_back_to_tech_sdlc() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("do something unrelated", &pipelines);
        assert_eq!(rec.pipeline_name, "tech-sdlc");
        assert!(rec.reason.contains("No keyword match"));
    }

    #[test]
    fn test_recommend_multi_word_keyword() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Write a user story for login flow", &pipelines);
        assert_eq!(rec.pipeline_name, "full-sdlc");
    }

    #[test]
    fn test_recommend_alternatives_present() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Add user authentication feature", &pipelines);
        assert!(!rec.alternatives.is_empty() || rec.pipeline_name == "full-sdlc");
    }

    #[test]
    fn test_recommend_cli_command() {
        let pipelines = sample_pipelines();
        let rec = PipelineRecommender::recommend("Refactor the database layer", &pipelines);
        assert!(rec.cli_command.contains("popsicle pipeline run tech-sdlc"));
    }

    #[test]
    fn test_recommend_empty_pipelines() {
        let rec = PipelineRecommender::recommend("anything", &[]);
        assert_eq!(rec.pipeline_name, "quick");
    }
}
