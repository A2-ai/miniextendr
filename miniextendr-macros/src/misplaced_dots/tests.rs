use super::*;

fn name_of(src: &str) -> Option<Option<String>> {
    find(src.parse().unwrap()).map(|found| found.name)
}

#[test]
fn named_dots_before_another_parameter() {
    assert_eq!(
        name_of("pub fn f(sources: ..., dosing_type: Strs) {}"),
        Some(Some("sources".to_string()))
    );
}

#[test]
fn bare_dots_before_another_parameter() {
    assert_eq!(name_of("fn f(x: i32, ..., y: i32) {}"), Some(None));
}

/// `_: ...` is unnamed dots too, so the hint names `_dots: &Dots`, which a
/// method accepts (`_: &Dots` there needs a plain name).
#[test]
fn wild_dots_before_another_parameter_are_unnamed() {
    assert_eq!(name_of("fn f(x: i32, _: ..., y: i32) {}"), Some(None));
    assert_eq!(
        name_of("impl T { fn m(&self, _: ..., n: i32) {} }"),
        Some(None)
    );
}

#[test]
fn method_in_an_impl_body() {
    assert_eq!(
        name_of("impl T { pub fn m<'a>(&self, rest: ..., n: &'a str) {} }"),
        Some(Some("rest".to_string()))
    );
}

#[test]
fn generics_with_a_fn_bound() {
    assert_eq!(
        name_of("fn f<F: Fn(i32) -> i32>(rest: ..., f: F) {}"),
        Some(Some("rest".to_string()))
    );
}

#[test]
fn last_dots_are_fine() {
    assert_eq!(name_of("fn f(x: i32, rest: ...) {}"), None);
    assert_eq!(name_of("fn f(x: i32, rest: ...,) {}"), None);
    assert_eq!(name_of("fn f(x: i32, ...) {}"), None);
    assert_eq!(name_of("fn f(x: i32, _: ...) {}"), None);
}

#[test]
fn explicit_dots_type_is_fine() {
    assert_eq!(name_of("fn f(rest: &Dots, x: i32) {}"), None);
}

#[test]
fn message_names_the_parameter() {
    let msg = message(Some("sources"));
    assert!(msg.starts_with("Rust's `...` is only valid as the last parameter"));
    assert!(msg.contains("`sources: &Dots`"));
    assert!(message(None).contains("`_dots: &Dots`"));
}
