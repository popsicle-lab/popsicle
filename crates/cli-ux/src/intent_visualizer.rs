//! Render product intent graphs via intent-lang-visualizer (UI feature).

use intent_lang_syntax::parse;
use intent_lang_visualizer::{render_mermaid_raw, unfence_mermaid, VisKind};

use crate::workspace_readers::IntentDiagramView;

const DIAGRAMS: &[(&str, &str, &str, VisKind)] = &[
    (
        "goal-graph",
        "目标追溯",
        "goal → safety / intent / theorem 的 realized_by 依赖",
        VisKind::GoalGraph,
    ),
    (
        "intent-graph",
        "Intent 关系",
        "intent 之间的数据流与 @tobe / @asis 关系",
        VisKind::IntentGraph,
    ),
    (
        "safety-network",
        "Safety 网络",
        "safety 规则及其约束的类型",
        VisKind::SafetyNetwork,
    ),
    (
        "coverage-matrix",
        "覆盖矩阵",
        "coverage 场景维度与未覆盖组合",
        VisKind::CoverageMatrix,
    ),
];

pub fn render_product_diagrams(source: &str) -> Result<Vec<IntentDiagramView>, String> {
    let program = parse(source).map_err(|e| e.message)?;
    let mut out = Vec::with_capacity(DIAGRAMS.len());
    for (id, label, description, kind) in DIAGRAMS {
        let raw = render_mermaid_raw(&program, *kind);
        let mermaid = crate::mermaid_sanitize::sanitize_mermaid_for_render(&unfence_mermaid(&raw));
        if mermaid.lines().count() <= 2 {
            continue;
        }
        out.push(IntentDiagramView {
            id: (*id).into(),
            label: (*label).into(),
            description: (*description).into(),
            mermaid,
        });
    }
    if out.is_empty() {
        return Err("no diagram content (empty or trivial graphs)".into());
    }
    Ok(out)
}
