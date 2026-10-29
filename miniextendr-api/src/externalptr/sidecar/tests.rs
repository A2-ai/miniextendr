use super::{no_handle_classes, saved_no_handle_message};

/// A list or environment without a pointer gets the type's no-handle
/// message, else its restored message (#2218); another receiver keeps
/// the receiver refusal's wording.
#[test]
fn saved_no_handle_message_prefers_the_no_handle_reason() {
    assert_eq!(
        saved_no_handle_message(true, Some("no engine"), Some("saved")),
        Some("no engine")
    );
    assert_eq!(
        saved_no_handle_message(true, None, Some("saved")),
        Some("saved")
    );
    assert_eq!(saved_no_handle_message(true, None, None), None);
    assert_eq!(
        saved_no_handle_message(false, Some("no engine"), Some("saved")),
        None
    );
}

/// A parent class both lists share follows both specific classes.
#[test]
fn no_handle_classes_keep_each_class_at_its_last_place() {
    assert_eq!(
        no_handle_classes(&["pkg_saved", "pkg_error"], &["pkg_field", "pkg_error"]),
        [
            "pkg_saved",
            "pkg_field",
            "pkg_error",
            "miniextendr_no_handle"
        ]
    );
    assert_eq!(no_handle_classes(&[], &[]), ["miniextendr_no_handle"]);
    assert_eq!(
        no_handle_classes(&["pkg_error"], &[]),
        ["pkg_error", "miniextendr_no_handle"]
    );
}
