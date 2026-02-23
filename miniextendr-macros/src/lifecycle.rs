//! Lifecycle support for R package deprecation management.
//!
//! This module provides integration with R's `lifecycle` package for managing
//! experimental, deprecated, and superseded functions.
//!
//! # Usage
//!
//! Mark functions with lifecycle attributes:
//!
//! ```rust,ignore
//! // Using Rust's deprecated attribute
//! #[deprecated(since = "0.4.0", note = "Use new_fn() instead")]
//! #[miniextendr]
//! pub fn old_fn(x: i32) -> i32 { x }
//!
//! // Using miniextendr's lifecycle attribute
//! #[miniextendr(lifecycle = "experimental")]
//! pub fn new_fn(x: i32) -> i32 { x * 2 }
//! ```
//!
//! # Lifecycle Stages
//!
//! - `experimental`: Function is experimental and may change without notice
//! - `stable`: Function is stable (default, no badge/warning needed)
//! - `superseded`: Function has a better alternative but will be maintained
//! - `deprecated`: Function should no longer be used and may be removed
//! - `defunct`: Function no longer works and throws an error

use std::fmt;

/// Lifecycle stage for a function, method, or argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifecycleStage {
    /// Function is experimental and may change.
    Experimental,
    /// Function is stable (no badge/warning needed).
    #[default]
    Stable,
    /// Function has a better alternative but will be maintained.
    Superseded,
    /// Function should no longer be used (soft warning first).
    SoftDeprecated,
    /// Function should no longer be used (warning).
    Deprecated,
    /// Function no longer works (error).
    Defunct,
}

impl LifecycleStage {
    /// Parse stage from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "experimental" => Some(Self::Experimental),
            "stable" => Some(Self::Stable),
            "superseded" => Some(Self::Superseded),
            "soft-deprecated" | "soft_deprecated" => Some(Self::SoftDeprecated),
            "deprecated" => Some(Self::Deprecated),
            "defunct" => Some(Self::Defunct),
            _ => None,
        }
    }

    /// Get the lifecycle badge text for roxygen.
    pub fn badge(&self) -> Option<&'static str> {
        match self {
            Self::Experimental => Some(r#"`r lifecycle::badge("experimental")`"#),
            Self::Stable => None, // No badge for stable
            Self::Superseded => Some(r#"`r lifecycle::badge("superseded")`"#),
            Self::SoftDeprecated | Self::Deprecated => {
                Some(r#"`r lifecycle::badge("deprecated")`"#)
            }
            Self::Defunct => Some(r#"`r lifecycle::badge("deprecated")`"#),
        }
    }

    /// Get the lifecycle signal function name.
    pub fn signal_fn(&self) -> Option<&'static str> {
        match self {
            Self::Experimental => Some("lifecycle::signal_stage"),
            Self::Stable => None,
            Self::Superseded => Some("lifecycle::signal_stage"),
            Self::SoftDeprecated => Some("lifecycle::deprecate_soft"),
            Self::Deprecated => Some("lifecycle::deprecate_warn"),
            Self::Defunct => Some("lifecycle::deprecate_stop"),
        }
    }

    /// Get the bare R function name for `@importFrom lifecycle` roxygen tag.
    ///
    /// Returns the function name without the `lifecycle::` prefix.
    pub fn import_from_fn(&self) -> Option<&'static str> {
        match self {
            Self::Experimental | Self::Superseded => Some("signal_stage"),
            Self::Stable => None,
            Self::SoftDeprecated => Some("deprecate_soft"),
            Self::Deprecated => Some("deprecate_warn"),
            Self::Defunct => Some("deprecate_stop"),
        }
    }

    /// Get the roxygen @keywords value (if needed).
    pub fn keywords(&self) -> Option<&'static str> {
        match self {
            Self::Experimental => Some("internal"),
            Self::Stable => None,
            Self::Superseded => None,
            Self::SoftDeprecated | Self::Deprecated | Self::Defunct => None,
        }
    }
}

impl fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Experimental => write!(f, "experimental"),
            Self::Stable => write!(f, "stable"),
            Self::Superseded => write!(f, "superseded"),
            Self::SoftDeprecated => write!(f, "soft-deprecated"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Defunct => write!(f, "defunct"),
        }
    }
}

/// Full lifecycle specification for a function or method.
#[derive(Debug, Clone, Default)]
pub struct LifecycleSpec {
    /// The lifecycle stage.
    pub stage: LifecycleStage,
    /// Version when the lifecycle change occurred (e.g., "0.4.0").
    pub when: Option<String>,
    /// What is being deprecated (e.g., "old_fn()" or "old_fn(arg)").
    /// Auto-inferred from function name if not provided.
    pub what: Option<String>,
    /// Replacement to suggest (e.g., "new_fn()").
    pub with: Option<String>,
    /// Additional details message.
    pub details: Option<String>,
    /// Unique ID for lifecycle deprecation tracking.
    pub id: Option<String>,
}

impl LifecycleSpec {
    /// Create a new lifecycle spec with a stage.
    pub fn new(stage: LifecycleStage) -> Self {
        Self {
            stage,
            ..Default::default()
        }
    }

    /// Create from a Rust `#[deprecated]` attribute.
    pub fn from_deprecated(since: Option<&str>, note: Option<&str>) -> Self {
        let mut spec = Self::new(LifecycleStage::Deprecated);
        spec.when = since.map(String::from);

        // Try to parse "with" from note if it contains "use X instead" pattern
        if let Some(note) = note {
            if let Some(rest) = note.to_lowercase().strip_prefix("use ") {
                if let Some(end) = rest.find(" instead") {
                    spec.with = Some(rest[..end].to_string());
                } else {
                    spec.with = Some(rest.to_string());
                }
            }
            spec.details = Some(note.to_string());
        }

        spec
    }

    /// Generate the R prelude code for lifecycle signaling.
    ///
    /// Returns R code to insert at the start of the function body.
    pub fn r_prelude(&self, fn_name: &str) -> Option<String> {
        let signal_fn = self.stage.signal_fn()?;

        let what = self.what.as_deref().unwrap_or(fn_name);

        match self.stage {
            LifecycleStage::Experimental | LifecycleStage::Superseded => {
                // lifecycle::signal_stage("experimental", "fn_name()")
                Some(format!("{}(\"{}\", \"{}()\")", signal_fn, self.stage, what))
            }
            LifecycleStage::SoftDeprecated
            | LifecycleStage::Deprecated
            | LifecycleStage::Defunct => {
                // lifecycle::deprecate_*(when, what, with, details, id)
                let when = self.when.as_deref().unwrap_or("0.0.0");
                let what_arg = format!("\"{}()\"", what);
                let with_arg = self
                    .with
                    .as_ref()
                    .map(|w| format!(", \"{}\"", w))
                    .unwrap_or_default();
                let details_arg = self
                    .details
                    .as_ref()
                    .map(|d| format!(", details = \"{}\"", d.replace('"', "\\\"")))
                    .unwrap_or_default();
                let id_arg = self
                    .id
                    .as_ref()
                    .map(|id| format!(", id = \"{}\"", id))
                    .unwrap_or_default();

                Some(format!(
                    "{}(\"{}\", {}{}{}{})",
                    signal_fn, when, what_arg, with_arg, details_arg, id_arg
                ))
            }
            LifecycleStage::Stable => None,
        }
    }
}

/// Collect the `@importFrom lifecycle ...` roxygen tag needed for a set of lifecycle specs.
///
/// This is used by class generators (R6, env, S3, S4, S7) to aggregate lifecycle
/// imports from all methods and include them in the class-level roxygen block.
/// Returns `None` if no lifecycle imports are needed.
pub fn collect_lifecycle_imports<'a>(
    specs: impl Iterator<Item = &'a LifecycleSpec>,
) -> Option<String> {
    let mut fns = std::collections::BTreeSet::new();
    for spec in specs {
        if spec.stage.badge().is_some() {
            fns.insert("badge");
        }
        if let Some(fn_name) = spec.stage.import_from_fn() {
            fns.insert(fn_name);
        }
    }
    if fns.is_empty() {
        None
    } else {
        Some(format!(
            "@importFrom lifecycle {}",
            fns.into_iter().collect::<Vec<_>>().join(" ")
        ))
    }
}

/// Parse lifecycle spec from miniextendr attribute arguments.
///
/// Supports:
/// - `lifecycle = "deprecated"` (simple stage)
/// - `lifecycle(stage = "deprecated", when = "0.4.0", with = "new_fn()")` (full spec)
pub fn parse_lifecycle_attr(meta: &syn::Meta) -> syn::Result<Option<LifecycleSpec>> {
    use syn::spanned::Spanned;

    match meta {
        syn::Meta::NameValue(nv) if nv.path.is_ident("lifecycle") => {
            // lifecycle = "stage"
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) = &nv.value
            {
                let stage = LifecycleStage::from_str(&lit.value()).ok_or_else(|| {
                    syn::Error::new_spanned(
                        lit,
                        "invalid lifecycle stage; expected one of: experimental, stable, superseded, soft-deprecated, deprecated, defunct",
                    )
                })?;
                return Ok(Some(LifecycleSpec::new(stage)));
            }
            Err(syn::Error::new_spanned(
                &nv.value,
                "lifecycle expects a string literal",
            ))
        }
        syn::Meta::List(list) if list.path.is_ident("lifecycle") => {
            // lifecycle(stage = "deprecated", when = "0.4.0", ...)
            let mut spec = LifecycleSpec::default();

            let nested: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]> =
                list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;

            for meta in nested {
                if let syn::Meta::NameValue(nv) = meta {
                    let key = nv.path.get_ident().map(|i| i.to_string());
                    let value = match &nv.value {
                        syn::Expr::Lit(expr_lit) => {
                            if let syn::Lit::Str(lit) = &expr_lit.lit {
                                lit.value()
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &nv.value,
                                    "expected string literal",
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "expected string literal",
                            ));
                        }
                    };

                    match key.as_deref() {
                        Some("stage") => {
                            spec.stage = LifecycleStage::from_str(&value).ok_or_else(|| {
                                syn::Error::new(nv.value.span(), "invalid lifecycle stage")
                            })?;
                        }
                        Some("when") => spec.when = Some(value),
                        Some("what") => spec.what = Some(value),
                        Some("with") => spec.with = Some(value),
                        Some("details") => spec.details = Some(value),
                        Some("id") => spec.id = Some(value),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.path,
                                "unknown lifecycle option; expected: stage, when, what, with, details, id",
                            ));
                        }
                    }
                }
            }

            Ok(Some(spec))
        }
        _ => Ok(None),
    }
}

/// Extract lifecycle info from a `#[deprecated]` attribute.
pub fn parse_rust_deprecated(attr: &syn::Attribute) -> Option<LifecycleSpec> {
    if !attr.path().is_ident("deprecated") {
        return None;
    }

    let mut since = None;
    let mut note = None;

    match &attr.meta {
        syn::Meta::Path(_) => {
            // Just #[deprecated]
        }
        syn::Meta::NameValue(nv) => {
            // #[deprecated = "message"]
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) = &nv.value
            {
                note = Some(lit.value());
            }
        }
        syn::Meta::List(list) => {
            // #[deprecated(since = "...", note = "...")]
            if let Ok(nested) = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let syn::Meta::NameValue(nv) = meta
                        && let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) = &nv.value
                    {
                        if nv.path.is_ident("since") {
                            since = Some(lit.value());
                        } else if nv.path.is_ident("note") {
                            note = Some(lit.value());
                        }
                    }
                }
            }
        }
    }

    Some(LifecycleSpec::from_deprecated(
        since.as_deref(),
        note.as_deref(),
    ))
}

/// Inject lifecycle badge into roxygen tags if not already present.
///
/// Modifies the tags in place, prepending the badge to @description if present,
/// or adding a new @description tag with just the badge.
pub fn inject_lifecycle_badge(tags: &mut Vec<String>, spec: &LifecycleSpec) {
    let Some(badge) = spec.stage.badge() else {
        return;
    };

    // Check if there's already a description tag
    let desc_idx = tags.iter().position(|t| t.starts_with("@description"));

    if let Some(idx) = desc_idx {
        // Prepend badge to existing description
        let existing = &tags[idx];
        let desc_content = existing
            .strip_prefix("@description")
            .unwrap_or("")
            .trim_start();
        tags[idx] = format!("@description {} {}", badge, desc_content);
    } else {
        // Insert new description with just the badge at the start
        tags.insert(0, format!("@description {}", badge));
    }

    // Add keywords if needed and not present
    if let Some(keywords) = spec.stage.keywords()
        && !tags.iter().any(|t| t.starts_with("@keywords"))
    {
        tags.push(format!("@keywords {}", keywords));
    }

    // Add @importFrom for the lifecycle signal function and badge
    inject_lifecycle_imports(tags, spec);
}

/// Inject `@importFrom lifecycle` roxygen tags for the signal function and badge.
///
/// Adds the necessary `@importFrom` tag so roxygen2 registers the lifecycle
/// dependency in NAMESPACE. This is only added if not already present.
pub fn inject_lifecycle_imports(tags: &mut Vec<String>, spec: &LifecycleSpec) {
    // Collect the lifecycle functions we need to import
    let mut fns_to_import = Vec::new();

    // The badge inline R code uses lifecycle::badge()
    if spec.stage.badge().is_some() {
        fns_to_import.push("badge");
    }

    // The runtime signal function
    if let Some(fn_name) = spec.stage.import_from_fn() {
        fns_to_import.push(fn_name);
    }

    if fns_to_import.is_empty() {
        return;
    }

    // Deduplicate against already-present @importFrom lifecycle tags
    let already_imported: Vec<&str> = tags
        .iter()
        .filter_map(|t| t.strip_prefix("@importFrom lifecycle "))
        .flat_map(|s| s.split_whitespace())
        .collect();

    fns_to_import.retain(|f| !already_imported.contains(f));

    if !fns_to_import.is_empty() {
        tags.push(format!("@importFrom lifecycle {}", fns_to_import.join(" ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_from_str() {
        assert_eq!(
            LifecycleStage::from_str("experimental"),
            Some(LifecycleStage::Experimental)
        );
        assert_eq!(
            LifecycleStage::from_str("deprecated"),
            Some(LifecycleStage::Deprecated)
        );
        assert_eq!(
            LifecycleStage::from_str("soft-deprecated"),
            Some(LifecycleStage::SoftDeprecated)
        );
        assert_eq!(LifecycleStage::from_str("unknown"), None);
    }

    #[test]
    fn test_r_prelude_deprecated() {
        let spec = LifecycleSpec {
            stage: LifecycleStage::Deprecated,
            when: Some("0.4.0".into()),
            what: None,
            with: Some("new_fn()".into()),
            details: None,
            id: None,
        };
        let prelude = spec.r_prelude("old_fn").unwrap();
        assert!(prelude.contains("lifecycle::deprecate_warn"));
        assert!(prelude.contains("0.4.0"));
        assert!(prelude.contains("old_fn()"));
        assert!(prelude.contains("new_fn()"));
    }

    #[test]
    fn test_r_prelude_experimental() {
        let spec = LifecycleSpec::new(LifecycleStage::Experimental);
        let prelude = spec.r_prelude("my_fn").unwrap();
        assert!(prelude.contains("lifecycle::signal_stage"));
        assert!(prelude.contains("experimental"));
        assert!(prelude.contains("my_fn()"));
    }

    #[test]
    fn test_from_deprecated_note() {
        let spec = LifecycleSpec::from_deprecated(Some("1.0.0"), Some("Use bar() instead"));
        assert_eq!(spec.stage, LifecycleStage::Deprecated);
        assert_eq!(spec.when, Some("1.0.0".into()));
        assert_eq!(spec.with, Some("bar()".into()));
    }

    #[test]
    fn test_import_from_fn() {
        assert_eq!(
            LifecycleStage::Experimental.import_from_fn(),
            Some("signal_stage")
        );
        assert_eq!(LifecycleStage::Stable.import_from_fn(), None);
        assert_eq!(
            LifecycleStage::Superseded.import_from_fn(),
            Some("signal_stage")
        );
        assert_eq!(
            LifecycleStage::SoftDeprecated.import_from_fn(),
            Some("deprecate_soft")
        );
        assert_eq!(
            LifecycleStage::Deprecated.import_from_fn(),
            Some("deprecate_warn")
        );
        assert_eq!(
            LifecycleStage::Defunct.import_from_fn(),
            Some("deprecate_stop")
        );
    }

    #[test]
    fn test_inject_lifecycle_imports_deprecated() {
        let spec = LifecycleSpec::new(LifecycleStage::Deprecated);
        let mut tags = vec!["@title My function".to_string()];
        inject_lifecycle_badge(&mut tags, &spec);
        // Should have @importFrom lifecycle badge deprecate_warn
        let import_tag = tags
            .iter()
            .find(|t| t.starts_with("@importFrom lifecycle"))
            .expect("should have @importFrom lifecycle tag");
        assert!(import_tag.contains("badge"));
        assert!(import_tag.contains("deprecate_warn"));
    }

    #[test]
    fn test_inject_lifecycle_imports_experimental() {
        let spec = LifecycleSpec::new(LifecycleStage::Experimental);
        let mut tags = vec!["@title My function".to_string()];
        inject_lifecycle_badge(&mut tags, &spec);
        let import_tag = tags
            .iter()
            .find(|t| t.starts_with("@importFrom lifecycle"))
            .expect("should have @importFrom lifecycle tag");
        assert!(import_tag.contains("badge"));
        assert!(import_tag.contains("signal_stage"));
    }

    #[test]
    fn test_inject_lifecycle_imports_stable_no_import() {
        let spec = LifecycleSpec::new(LifecycleStage::Stable);
        let mut tags = vec!["@title My function".to_string()];
        inject_lifecycle_badge(&mut tags, &spec);
        // Stable stage should not add any @importFrom
        assert!(!tags.iter().any(|t| t.starts_with("@importFrom")));
    }

    #[test]
    fn test_inject_lifecycle_imports_no_duplicates() {
        let spec = LifecycleSpec::new(LifecycleStage::Deprecated);
        let mut tags = vec![
            "@title My function".to_string(),
            "@importFrom lifecycle deprecate_warn badge".to_string(),
        ];
        inject_lifecycle_badge(&mut tags, &spec);
        // Should not add a duplicate @importFrom tag
        let import_count = tags
            .iter()
            .filter(|t| t.starts_with("@importFrom lifecycle"))
            .count();
        assert_eq!(import_count, 1);
    }

    #[test]
    fn test_collect_lifecycle_imports_mixed_methods() {
        let specs = [
            LifecycleSpec::new(LifecycleStage::Deprecated),
            LifecycleSpec::new(LifecycleStage::Experimental),
            LifecycleSpec::new(LifecycleStage::Stable),
        ];
        let result = collect_lifecycle_imports(specs.iter());
        let import = result.expect("should produce import tag");
        // BTreeSet gives sorted order: badge, deprecate_warn, signal_stage
        assert_eq!(
            import,
            "@importFrom lifecycle badge deprecate_warn signal_stage"
        );
    }

    #[test]
    fn test_collect_lifecycle_imports_no_lifecycle() {
        let specs: Vec<LifecycleSpec> = vec![];
        let result = collect_lifecycle_imports(specs.iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_collect_lifecycle_imports_only_stable() {
        let specs = [LifecycleSpec::new(LifecycleStage::Stable)];
        let result = collect_lifecycle_imports(specs.iter());
        assert!(result.is_none());
    }
}
