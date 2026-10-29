use super::*;

#[test]
fn hint_names_the_parameter_and_line() {
    let src = "use x::Strs;\n\n#[miniextendr]\npub fn f(sources: ..., dosing_type: Strs) {}\n";
    let hint = hint(src).expect("hint");
    assert!(hint.starts_with("line 4: Rust's `...` is only valid as the last parameter"));
    assert!(hint.contains("`sources: &Dots`"));
}

#[test]
fn bare_dots_get_a_placeholder_name() {
    let hint = hint("mod m { fn f(x: i32, ..., y: i32) {} }").expect("hint");
    assert!(hint.contains("`_dots: &Dots`"));
}

/// `_: ...` is unnamed dots too: `_dots: &Dots` works on a method, where
/// `_: &Dots` needs a plain name.
#[test]
fn wild_dots_get_the_placeholder_name() {
    let hint = hint("impl T { fn m(&self, _: ..., n: i32) {} }").expect("hint");
    assert!(hint.contains("`_dots: &Dots`"), "got: {hint}");
    assert!(!hint.contains("`_: &Dots`"), "got: {hint}");
}

#[test]
fn method_in_an_impl_body() {
    assert!(hint("impl T { fn m(&self, rest: ..., n: i32) {} }").is_some());
}

#[test]
fn last_dots_and_comments_get_no_hint() {
    assert_eq!(hint("fn f(x: i32, rest: ...) {}"), None);
    assert_eq!(hint("fn f(x: i32, rest: ...,) {}"), None);
    assert_eq!(hint("fn f(x: i32, _: ...) {}"), None);
    assert_eq!(hint("// fn f(rest: ..., x: i32) {}\nfn g() {}"), None);
    assert_eq!(hint("/// fn f(rest: ..., x: i32) {}\nfn g() {}"), None);
}

#[test]
fn generics_before_the_parameters() {
    assert!(hint("fn f<'a, F: Fn(i32) -> i32>(rest: ..., f: &'a F) {}").is_some());
}
