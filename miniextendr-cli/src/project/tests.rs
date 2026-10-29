use super::parse_description_field;

#[test]
fn simple_field() {
    let content = "Package: mypkg\nVersion: 0.1.0\n";
    assert_eq!(
        parse_description_field(content, "Package"),
        Some("mypkg".to_string())
    );
}

#[test]
fn continuation_line_field() {
    let content = "Package: mypkg\n\
             Description: This is a\n    long description that\n    wraps.\n\
             Version: 0.1.0\n";
    assert_eq!(
        parse_description_field(content, "Description"),
        Some("This is a long description that wraps.".to_string())
    );
}

#[test]
fn missing_field() {
    let content = "Package: mypkg\nVersion: 0.1.0\n";
    assert_eq!(parse_description_field(content, "License"), None);
}

#[test]
fn field_name_prefix_of_another() {
    // "Packaged:" must not be mistaken for a match on "Package".
    let content = "Packaged: 2024-01-01\nPackage: mypkg\n";
    assert_eq!(
        parse_description_field(content, "Package"),
        Some("mypkg".to_string())
    );
    // And querying the longer name should not pick up the shorter one.
    let content = "Package: mypkg\nPackaged: 2024-01-01\n";
    assert_eq!(
        parse_description_field(content, "Packaged"),
        Some("2024-01-01".to_string())
    );
}
