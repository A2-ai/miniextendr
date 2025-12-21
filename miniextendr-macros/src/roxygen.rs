use std::collections::HashSet;

/// Extract roxygen tag lines (starting with '@') from Rust doc attributes.
///
/// Handles multiline tags: continuation lines (not starting with '@') are
/// appended to the previous tag with a newline separator.
pub(crate) fn roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut tags = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };
        for line in lit.value().lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('@') {
                // New tag starts
                tags.push(trimmed.to_string());
            } else if !trimmed.is_empty() {
                // Continuation line - append to last tag if one exists
                if let Some(last) = tags.last_mut() {
                    last.push('\n');
                    last.push_str(trimmed);
                }
            }
        }
    }
    tags
}

/// Render roxygen tag lines as "#' ..." comment lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn format_roxygen_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for tag in tags {
        for line in tag.lines() {
            out.push_str("#' ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Push roxygen tag lines into a vector of R wrapper lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn push_roxygen_tags(lines: &mut Vec<String>, tags: &[String]) {
    for tag in tags {
        for line in tag.lines() {
            lines.push(format!("#' {}", line));
        }
    }
}

/// Return true if the tag list contains a specific roxygen tag.
pub(crate) fn has_roxygen_tag(tags: &[String], tag: &str) -> bool {
    tag_names(tags).contains(tag)
}

fn tag_names(tags: &[String]) -> HashSet<&str> {
    let mut names = HashSet::new();
    for tag in tags {
        let trimmed = tag.trim_start();
        let name = trimmed
            .strip_prefix('@')
            .and_then(|rest| rest.split_whitespace().next());
        if let Some(name) = name {
            names.insert(name);
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single_line_tags() {
        let tags = vec!["@param x An input".to_string(), "@return Output".to_string()];
        let formatted = format_roxygen_tags(&tags);
        assert_eq!(formatted, "#' @param x An input\n#' @return Output\n");
    }

    #[test]
    fn test_format_multiline_tag() {
        // Simulates: @description First line\nSecond line
        let tags = vec!["@description First line\nSecond line".to_string()];
        let formatted = format_roxygen_tags(&tags);
        assert_eq!(formatted, "#' @description First line\n#' Second line\n");
    }

    #[test]
    fn test_push_multiline_tag() {
        let tags = vec!["@description Line one\nLine two\nLine three".to_string()];
        let mut lines = Vec::new();
        push_roxygen_tags(&mut lines, &tags);
        assert_eq!(
            lines,
            vec![
                "#' @description Line one",
                "#' Line two",
                "#' Line three"
            ]
        );
    }

    #[test]
    fn test_has_roxygen_tag_multiline() {
        // Tag name detection should work even with multiline content
        let tags = vec!["@description First\nSecond".to_string()];
        assert!(has_roxygen_tag(&tags, "description"));
        assert!(!has_roxygen_tag(&tags, "param"));
    }
}
