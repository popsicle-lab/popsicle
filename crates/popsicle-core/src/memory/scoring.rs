use super::model::{Memory, MemoryLayer, MemoryType};

/// Default maximum number of memories to inject into a prompt.
pub const DEFAULT_INJECT_LIMIT: usize = 10;

fn type_weight(ty: MemoryType) -> f32 {
    match ty {
        MemoryType::Pattern => 1.0,
        MemoryType::Bug => 0.8,
        MemoryType::Decision => 0.6,
        MemoryType::Gotcha => 0.5,
    }
}

fn layer_weight(layer: MemoryLayer) -> f32 {
    match layer {
        MemoryLayer::LongTerm => 1.0,
        MemoryLayer::ShortTerm => 0.7,
    }
}

fn tag_overlap(memory_tags: &[String], query_tags: &[String]) -> f32 {
    if query_tags.is_empty() {
        return 0.0;
    }
    memory_tags
        .iter()
        .filter(|t| query_tags.iter().any(|q| q.eq_ignore_ascii_case(t)))
        .count() as f32
}

fn file_overlap(memory_files: &[String], query_files: &[String]) -> f32 {
    if query_files.is_empty() {
        return 0.0;
    }
    memory_files
        .iter()
        .filter(|f| {
            query_files
                .iter()
                .any(|q| q.contains(f.as_str()) || f.contains(q.as_str()))
        })
        .count() as f32
}

/// Score a single memory for relevance to the current context.
pub fn score_memory(memory: &Memory, context_tags: &[String], context_files: &[String]) -> f32 {
    let tw = type_weight(memory.memory_type);
    let lw = layer_weight(memory.layer);

    let match_score =
        1.0 + tag_overlap(&memory.tags, context_tags) + file_overlap(&memory.files, context_files);

    let stale_penalty = if memory.stale { 0.5 } else { 1.0 };

    tw * lw * match_score * stale_penalty
}

/// Rank memories by relevance and return the top-N.
///
/// `context_tags` and `context_files` are used for matching; pass empty slices
/// to skip tag/file matching (all memories get a base score from type + layer).
pub fn rank_memories<'a>(
    memories: &'a [Memory],
    context_tags: &[String],
    context_files: &[String],
    limit: usize,
) -> Vec<&'a Memory> {
    let mut scored: Vec<(f32, &Memory)> = memories
        .iter()
        .map(|m| (score_memory(m, context_tags, context_files), m))
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.into_iter().map(|(_, m)| m).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory(
        id: u32,
        ty: MemoryType,
        layer: MemoryLayer,
        tags: &[&str],
        files: &[&str],
        stale: bool,
    ) -> Memory {
        Memory {
            id,
            memory_type: ty,
            summary: format!("Memory {id}"),
            created: "2026-03-14".into(),
            layer,
            refs: 0,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            files: files.iter().map(|s| s.to_string()).collect(),
            run: None,
            stale,
            detail: String::new(),
        }
    }

    #[test]
    fn pattern_ranks_above_gotcha() {
        let memories = vec![
            make_memory(
                1,
                MemoryType::Gotcha,
                MemoryLayer::LongTerm,
                &[],
                &[],
                false,
            ),
            make_memory(
                2,
                MemoryType::Pattern,
                MemoryLayer::LongTerm,
                &[],
                &[],
                false,
            ),
        ];
        let ranked = rank_memories(&memories, &[], &[], 10);
        assert_eq!(ranked[0].id, 2);
        assert_eq!(ranked[1].id, 1);
    }

    #[test]
    fn tag_match_boosts_score() {
        let memories = vec![
            make_memory(
                1,
                MemoryType::Bug,
                MemoryLayer::LongTerm,
                &["serde"],
                &[],
                false,
            ),
            make_memory(
                2,
                MemoryType::Bug,
                MemoryLayer::LongTerm,
                &["other"],
                &[],
                false,
            ),
        ];
        let tags = vec!["serde".to_string()];
        let ranked = rank_memories(&memories, &tags, &[], 10);
        assert_eq!(ranked[0].id, 1);
    }

    #[test]
    fn file_match_boosts_score() {
        let memories = vec![
            make_memory(
                1,
                MemoryType::Bug,
                MemoryLayer::LongTerm,
                &[],
                &["context.rs"],
                false,
            ),
            make_memory(
                2,
                MemoryType::Bug,
                MemoryLayer::LongTerm,
                &[],
                &["other.rs"],
                false,
            ),
        ];
        let files = vec!["context.rs".to_string()];
        let ranked = rank_memories(&memories, &[], &files, 10);
        assert_eq!(ranked[0].id, 1);
    }

    #[test]
    fn stale_is_penalized() {
        let memories = vec![
            make_memory(1, MemoryType::Bug, MemoryLayer::LongTerm, &["x"], &[], true),
            make_memory(
                2,
                MemoryType::Bug,
                MemoryLayer::LongTerm,
                &["x"],
                &[],
                false,
            ),
        ];
        let tags = vec!["x".to_string()];
        let ranked = rank_memories(&memories, &tags, &[], 10);
        assert_eq!(ranked[0].id, 2);
    }

    #[test]
    fn limit_truncates() {
        let memories: Vec<_> = (0..20)
            .map(|i| make_memory(i, MemoryType::Bug, MemoryLayer::LongTerm, &[], &[], false))
            .collect();
        let ranked = rank_memories(&memories, &[], &[], 5);
        assert_eq!(ranked.len(), 5);
    }

    #[test]
    fn long_term_ranks_above_short_term() {
        let memories = vec![
            make_memory(1, MemoryType::Bug, MemoryLayer::ShortTerm, &[], &[], false),
            make_memory(2, MemoryType::Bug, MemoryLayer::LongTerm, &[], &[], false),
        ];
        let ranked = rank_memories(&memories, &[], &[], 10);
        assert_eq!(ranked[0].id, 2);
    }
}
