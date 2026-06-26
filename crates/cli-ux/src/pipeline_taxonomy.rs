//! Canonical pipeline ids and deprecated aliases (ADR-029).
//!
//! Pattern: `{domain}-{object?}-{phase}` — migration-* uses `slice`; daily work uses `feature-*`.

/// Resolve a pipeline name to its canonical id (accepts deprecated aliases).
pub fn canonical_pipeline_name(name: &str) -> &str {
    PIPELINE_ALIASES
        .iter()
        .find_map(|(old, new)| (*old == name).then_some(*new))
        .unwrap_or(name)
}

/// Deprecated ids that resolve to `canonical`.
pub fn deprecated_aliases_for(canonical: &str) -> Vec<&'static str> {
    PIPELINE_ALIASES
        .iter()
        .filter_map(|(old, new)| (*new == canonical).then_some(*old))
        .collect()
}

/// Deprecated ids that still resolve via [`canonical_pipeline_name`].
pub const PIPELINE_ALIASES: &[(&str, &str)] = &[
    ("slice-spec", "migration-slice-spec"),
    ("slice-delivery", "migration-slice-delivery"),
    ("greenfield-product-spec", "product-greenfield-spec"),
    ("tech-decision", "arch-decision"),
    ("bugfix", "fix-regression"),
    ("weekly-health-check", "doc-sync-weekly"),
];

/// Migration slice delivery pipeline (Strangler cutover chain).
pub fn is_migration_slice_delivery(pipeline: &str) -> bool {
    canonical_pipeline_name(pipeline) == "migration-slice-delivery"
}

/// Regression fix pipeline.
pub fn is_fix_regression(pipeline: &str) -> bool {
    canonical_pipeline_name(pipeline) == "fix-regression"
}

/// UI / catalog grouping label.
pub fn pipeline_domain(name: &str) -> &'static str {
    let name = canonical_pipeline_name(name);
    if name.starts_with("migration-") {
        return "migration";
    }
    if name.starts_with("feature-") {
        return "feature";
    }
    if name.starts_with("product-") {
        return "product";
    }
    if name.starts_with("doc-") {
        return "doc";
    }
    if name.starts_with("fix-") {
        return "fix";
    }
    if name.starts_with("arch-") {
        return "arch";
    }
    if name.starts_with("platform-") {
        return "platform";
    }
    "other"
}

/// If `name` is a deprecated alias, return its canonical id; otherwise `None`.
pub fn canonicalize_if_deprecated(name: &str) -> Option<&'static str> {
    PIPELINE_ALIASES
        .iter()
        .find_map(|(old, new)| (*old == name).then_some(*new))
}

/// Stems of deprecated `.pipeline.yaml` files that may linger in `.popsicle/pipelines/`.
pub fn deprecated_pipeline_install_stems() -> impl Iterator<Item = &'static str> {
    PIPELINE_ALIASES.iter().map(|(old, _)| *old)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_resolve_to_canonical() {
        assert_eq!(
            canonical_pipeline_name("slice-delivery"),
            "migration-slice-delivery"
        );
        assert_eq!(canonical_pipeline_name("bugfix"), "fix-regression");
        assert_eq!(
            canonical_pipeline_name("migration-slice-spec"),
            "migration-slice-spec"
        );
    }

    #[test]
    fn domain_labels_follow_prefix() {
        assert_eq!(pipeline_domain("slice-spec"), "migration");
        assert_eq!(pipeline_domain("feature-delivery"), "feature");
        assert_eq!(pipeline_domain("feature-arch-spec"), "feature");
        assert_eq!(pipeline_domain("doc-sync-weekly"), "doc");
    }

    #[test]
    fn canonicalize_if_deprecated_maps_aliases() {
        assert_eq!(
            canonicalize_if_deprecated("slice-delivery"),
            Some("migration-slice-delivery")
        );
        assert_eq!(canonicalize_if_deprecated("migration-slice-spec"), None);
    }
}
