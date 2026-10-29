use super::{claim, class_name_of};

#[test]
fn the_class_name_leaves_out_the_version_and_the_module_path() {
    assert_eq!(
        class_name_of("mypkg@1.2.0::mypkg::models::Model"),
        "miniextendr_saved:mypkg::Model"
    );
    assert_eq!(
        class_name_of("mypkg@0.1.0::mypkg::Model"),
        "miniextendr_saved:mypkg::Model"
    );
    assert_eq!(class_name_of("my::Model"), "miniextendr_saved:my::Model");
}

#[test]
fn two_types_of_one_crate_with_the_same_name_are_refused() {
    let mut claimed = Vec::new();
    let first = "mypkg@0.1.0::mypkg::a::Model";
    let second = "mypkg@0.1.0::mypkg::b::Model";
    assert!(claim(&mut claimed, &class_name_of(first), first).is_ok());
    // The same type again: its registration runs once per class cell,
    // but a test harness may call it more often.
    assert!(claim(&mut claimed, &class_name_of(first), first).is_ok());
    let message = claim(&mut claimed, &class_name_of(second), second)
        .expect_err("the second `Model` would share the first one's class");
    assert!(
        message.contains("`mypkg::a::Model` and `mypkg::b::Model`"),
        "{message}"
    );
    // Another crate's type of that name, and another name, have their own
    // class.
    let other_crate = "otherpkg@0.1.0::otherpkg::a::Model";
    assert!(claim(&mut claimed, &class_name_of(other_crate), other_crate).is_ok());
    let other_name = "mypkg@0.1.0::mypkg::a::Record";
    assert!(claim(&mut claimed, &class_name_of(other_name), other_name).is_ok());
    assert_eq!(claimed.len(), 3);
}
