use super::parse_feature_rules;

#[test]
fn canonical_rules_are_scoped_and_keep_detection_expressions() {
    let content = r#"rules[["outside"]] <- function() FALSE
## BEGIN RULES (do not edit this line)
rules[["alpha"]] <- function() requireNamespace("pkg", quietly = TRUE)
rules[["beta"]] <- function() FALSE
## END RULES (do not edit this line)
rules[["after"]] <- function() TRUE
"#;
    let rules = parse_feature_rules(content).unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules["alpha"], r#"requireNamespace("pkg", quietly = TRUE)"#);
    assert_eq!(rules["beta"], "FALSE");
}

#[test]
fn legacy_or_incomplete_rules_are_rejected() {
    assert!(parse_feature_rules("detect_features <- function() character()").is_err());
    assert!(parse_feature_rules("## BEGIN RULES\n").is_err());
    assert!(
        parse_feature_rules("## BEGIN RULES\n## END RULES\n")
            .unwrap()
            .is_empty()
    );
}
