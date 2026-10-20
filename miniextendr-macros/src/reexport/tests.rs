use super::*;

/// The `(package, name, literal)` of each item of `input`.
fn parsed(input: &str) -> Vec<(String, String, bool)> {
    let tokens: TokenStream = input.parse().expect("tokens");
    match parse_items(tokens) {
        Ok(items) => items
            .into_iter()
            .map(|item| (item.package, item.name, item.literal))
            .collect(),
        Err(err) => panic!("expected `{input}` to parse: {err}"),
    }
}

/// The messages of every error of `input`, in order.
fn errors(input: &str) -> Vec<String> {
    let tokens: TokenStream = input.parse().expect("tokens");
    match parse_items(tokens) {
        Ok(_) => panic!("expected `{input}` to be refused"),
        Err(err) => err.into_iter().map(|err| err.to_string()).collect(),
    }
}

fn item(package: &str, name: &str, literal: bool) -> (String, String, bool) {
    (package.to_string(), name.to_string(), literal)
}

#[test]
fn items_are_dotted_identifiers_or_a_string_literal_name() {
    assert_eq!(
        parsed("generics::tidy, generics::glance, data.table::fread, stats::na.omit"),
        [
            item("generics", "tidy", false),
            item("generics", "glance", false),
            item("data.table", "fread", false),
            item("stats", "na.omit", false),
        ]
    );
    assert_eq!(
        parsed(r#"magrittr::"%>%", base::"[.data.frame", R.utils::"tidy","#),
        [
            item("magrittr", "%>%", true),
            item("base", "[.data.frame", true),
            item("R.utils", "tidy", true),
        ]
    );
}

#[test]
fn rust_keywords_and_raw_identifiers_are_read_as_written() {
    assert_eq!(
        parsed("base::as.numeric, base::r#type.convert, methods::is"),
        [
            item("base", "as.numeric", false),
            item("base", "type.convert", false),
            item("methods", "is", false),
        ]
    );
}

#[test]
fn a_pkg_name_repeated_is_kept_for_the_writer_to_collapse() {
    assert_eq!(
        parsed("generics::tidy, generics::tidy"),
        [
            item("generics", "tidy", false),
            item("generics", "tidy", false)
        ]
    );
}

#[test]
fn an_empty_invocation_is_refused() {
    assert_eq!(
        errors(""),
        ["`reexport!` takes one or more `pkg::name` items, e.g. `reexport!(generics::tidy)`"]
    );
}

#[test]
fn a_package_must_be_an_r_package_name() {
    let message = |package: &str| {
        format!(
            "`{package}` is not an R package name: ASCII letters, digits and `.`, at least two \
             characters, starting with a letter and not ending in `.`"
        )
    };
    assert_eq!(errors("g::tidy"), [message("g")]);
    assert_eq!(errors("my_pkg::tidy"), [message("my_pkg")]);
}

#[test]
fn an_identifier_name_must_be_syntactic() {
    assert_eq!(
        errors("pkg::_hidden"),
        [
            "`_hidden` is not a syntactic R name; write a non-syntactic name as a string \
             literal, `pkg::\"_hidden\"`"
        ]
    );
    assert_eq!(
        errors("base::function"),
        [
            "`function` is not a syntactic R name; write a non-syntactic name as a string \
             literal, `base::\"function\"`"
        ]
    );
}

#[test]
fn a_string_literal_name_must_be_writable() {
    assert_eq!(errors(r#"pkg::"""#), ["a re-exported name can't be empty"]);
    assert_eq!(
        errors(r#"pkg::"caf\u{e9}""#),
        ["`café` is not ASCII; R code in a package must be ASCII"]
    );
    let quoting = |name: &str| {
        format!(
            "the re-exported name `{name}` can't hold whitespace, control characters, `` ` ``, \
             `\\` or `\"`"
        )
    };
    assert_eq!(errors(r#"pkg::"a b""#), [quoting("a b")]);
    assert_eq!(errors(r#"pkg::"a`b""#), [quoting("a`b")]);
    assert_eq!(errors(r#"pkg::"a\\b""#), [quoting("a\\b")]);
    assert_eq!(
        errors(r#"pkg::"x"suffix"#),
        ["a re-exported name written as a string literal takes no suffix"]
    );
}

#[test]
fn an_item_is_one_package_and_one_name() {
    assert_eq!(
        errors("generics"),
        ["expected `::` after the package `generics`: an item is `pkg::name`"]
    );
    assert_eq!(
        errors("generics:tidy"),
        ["expected `::` after the package `generics`: an item is `pkg::name`"]
    );
    assert_eq!(
        errors("generics::tidy::x"),
        ["an item names a package and one of its exports, `pkg::name`; separate items with `,`"]
    );
    assert_eq!(
        errors("generics::tidy generics::glance"),
        ["unexpected token after `pkg::name`; separate items with `,`"]
    );
    assert_eq!(
        errors("generics::tidy,, generics::glance"),
        ["expected a `pkg::name` item before this `,`"]
    );
    assert_eq!(
        errors("\"generics\"::tidy"),
        ["expected the package name of `pkg::name`"]
    );
    assert_eq!(
        errors("pkg::name.2"),
        [
            "expected an identifier after `.`; write a name that is not identifiers joined by \
             `.` as a string literal, `pkg::\"name\"`"
        ]
    );
}

#[test]
fn one_name_from_two_packages_is_refused() {
    assert_eq!(
        errors("generics::tidy, broom::tidy"),
        [
            "`tidy` is already re-exported from `generics` in this `reexport!`; a package \
             exports one object under a name"
        ]
    );
}

/// Every item's problems are reported, each item's on its own tokens.
#[test]
fn every_problem_of_an_invocation_is_reported() {
    let tokens: TokenStream = "x::tidy, generics::_x, generics::glance, broom::glance, g:"
        .parse()
        .expect("tokens");
    let Err(err) = parse_items(tokens) else {
        panic!("expected the invocation to be refused");
    };
    let located: Vec<(String, usize)> = err
        .into_iter()
        .map(|err| (err.to_string(), err.span().start().column))
        .collect();
    assert_eq!(located.len(), 4, "{located:?}");
    assert!(located[0].0.starts_with("`x` is not an R package name"));
    assert_eq!(located[0].1, 0);
    assert!(located[1].0.starts_with("`_x` is not a syntactic R name"));
    assert_eq!(located[1].1, "x::tidy, generics::".len());
    assert!(
        located[2]
            .0
            .starts_with("`glance` is already re-exported from `generics`")
    );
    assert_eq!(
        located[2].1,
        "x::tidy, generics::_x, generics::glance, ".len()
    );
    assert!(
        located[3]
            .0
            .starts_with("expected `::` after the package `g`")
    );
}

#[test]
fn each_item_is_one_host_only_registry_entry() {
    let tokens: TokenStream = "generics::tidy,\n  magrittr::\"%>%\""
        .parse()
        .expect("tokens");
    let expanded = expand(tokens).to_string();
    assert!(
        expanded.starts_with("# [cfg (not (target_arch = \"wasm32\"))] const _ : () = {"),
        "{expanded}"
    );
    assert_eq!(expanded.matches("registry :: MX_REEXPORTS").count(), 2);
    for (package, name, line, column) in [("generics", "tidy", 1, 1), ("magrittr", "%>%", 2, 3)] {
        let entry = format!(
            "ReexportEntry {{ package : \"{package}\" , name : \"{name}\" , source_file : :: \
             core :: file ! () , source_line : {line} , source_column : {column} , }}"
        );
        assert!(expanded.contains(&entry), "{entry}\nnot in\n{expanded}");
    }
}

#[test]
fn a_refused_invocation_expands_to_its_errors_only() {
    let tokens: TokenStream = "x::tidy".parse().expect("tokens");
    let expanded = expand(tokens).to_string();
    assert!(expanded.contains("compile_error"), "{expanded}");
    assert!(!expanded.contains("MX_REEXPORTS"), "{expanded}");
}
