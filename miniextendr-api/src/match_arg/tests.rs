use super::escape_r_string;

#[test]
fn escapes_backslash_and_quote() {
    assert_eq!(escape_r_string(r#"say "hi""#), r#"say \"hi\""#);
    assert_eq!(escape_r_string(r"c:\path"), r"c:\\path");
}

#[test]
fn escapes_control_characters() {
    assert_eq!(escape_r_string("line1\nline2"), r"line1\nline2");
    assert_eq!(escape_r_string("tab\there"), r"tab\there");
    assert_eq!(escape_r_string("cr\rlf"), r"cr\rlf");
}

#[test]
fn passes_through_plain_strings() {
    assert_eq!(escape_r_string("Fast"), "Fast");
    assert_eq!(escape_r_string("it's"), "it's");
    assert_eq!(escape_r_string(""), "");
}

/// The expectation of a choice parameter: one or several choices, the
/// other accepted values after them, quoted as `one_of` quotes.
#[test]
fn choice_expectation_names_the_choices_and_the_rest() {
    use super::{choice_expectation, one_of};
    assert_eq!(
        choice_expectation(&["fast"], &[], false, ""),
        r#"one of "fast""#
    );
    assert_eq!(
        choice_expectation(&["fast", "slow"], &[], false, ", or a data frame"),
        r#"one of "fast", "slow", or a data frame"#
    );
    assert_eq!(
        choice_expectation(&["fast", "slow"], &[], true, ", or a number"),
        r#"one or more of "fast", "slow", or a number"#
    );
    assert_eq!(
        choice_expectation(&[r#"say "hi""#, "bye"], &[], false, ""),
        r#"one of "say \"hi\"", "bye""#
    );
    assert_eq!(one_of(&["fast", "slow"], &[]), r#"one of "fast", "slow""#);
}

/// The listed aliases go between the choices and the other accepted
/// values, in the order of their choices, then their own (#2148); an
/// unlisted alias is left out, and a type whose aliases are all unlisted
/// reads as one without aliases.
#[test]
fn choice_expectation_lists_the_listed_aliases() {
    use super::{MatchArgAlias, choice_expectation, one_of};
    let choices = ["extravascular", "iv_bolus", "gray"];
    let aliases = [
        MatchArgAlias::new("grey", "gray"),
        MatchArgAlias::unlisted("ev", "extravascular"),
        MatchArgAlias::new("oral", "extravascular"),
        MatchArgAlias::new("Grau", "gray"),
    ];
    assert_eq!(
        one_of(&choices, &aliases),
        r#"one of "extravascular", "iv_bolus", "gray" (or "oral" for "extravascular", "grey" for "gray", "Grau" for "gray")"#
    );
    assert_eq!(
        choice_expectation(&choices, &aliases[..2], true, ", or a number"),
        r#"one or more of "extravascular", "iv_bolus", "gray" (or "grey" for "gray"), or a number"#
    );
    assert_eq!(
        one_of(&choices, &[MatchArgAlias::unlisted("ev", "extravascular")]),
        one_of(&choices, &[])
    );
    // Quoted as the choices are.
    assert_eq!(
        one_of(
            &[r#"say "hi""#],
            &[MatchArgAlias::new("hello", r#"say "hi""#)]
        ),
        r#"one of "say \"hi\"" (or "hello" for "say \"hi\"")"#
    );
}

/// The `match_arg` check's message quotes the aliases as it quotes the
/// choices, without escapes, and keeps them in the order of `CHOICES`
/// when a `default = "..."` formal moves a choice to the front.
#[test]
fn arg_message_lists_the_listed_aliases() {
    use super::{MatchArgAlias, MatchArgError};
    static ALIASES: [MatchArgAlias; 3] = [
        MatchArgAlias::new("grey", "gray"),
        MatchArgAlias::new("rouge", "red"),
        MatchArgAlias::unlisted("hello", r#"say "hi""#),
    ];
    let e = MatchArgError::NoMatch {
        input: "zzz".to_string(),
        choices: &["red", "gray", r#"say "hi""#],
        aliases: &ALIASES,
    };
    assert_eq!(
        e.arg_message("color", &["gray", "red", r#"say "hi""#]),
        r#"'color' should be one of "gray", "red", "say "hi"" (or "rouge" for "red", "grey" for "gray")"#
    );
    assert_eq!(
        e.to_string(),
        r#"expected one of "red", "gray", "say \"hi\"" (or "rouge" for "red", "grey" for "gray"), got "zzz""#
    );
    let long = MatchArgError::InvalidLength {
        actual: 2,
        choices: &["red"],
        aliases: &ALIASES,
    };
    assert_eq!(
        long.arg_message("color", &["red"]),
        "'color' must be of length 1"
    );
}

/// A `default = "..."` formal: the default first, the rest in their
/// order (not a rotation of the list), and `None` for a default that is
/// no choice (#1767).
#[test]
fn default_first_moves_the_default_to_the_front() {
    use super::default_first;
    let choices = ["Fast", "Safe", "Debug"];
    let is = |default: &'static str| move |c: &str| c == default;
    assert_eq!(
        default_first(&choices, is("Safe")).unwrap(),
        ["Safe", "Fast", "Debug"]
    );
    assert_eq!(
        default_first(&choices, is("Debug")).unwrap(),
        ["Debug", "Fast", "Safe"]
    );
    assert_eq!(default_first(&choices, is("Fast")).unwrap(), choices);
    assert_eq!(default_first(&choices, is("Slow")), None);
    assert_eq!(default_first(&["only"], is("only")).unwrap(), ["only"]);
}
