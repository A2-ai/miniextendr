use super::*;
use std::borrow::Cow;

struct TestRow {
    name: String,
    value: i32,
}

impl Tabled for TestRow {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.name),
            Cow::Owned(self.value.to_string()),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![Cow::Borrowed("name"), Cow::Borrowed("value")]
    }
}

#[test]
fn test_table_to_string() {
    let rows = vec![
        TestRow {
            name: "foo".into(),
            value: 1,
        },
        TestRow {
            name: "bar".into(),
            value: 2,
        },
    ];
    let table = table_to_string(&rows);
    assert!(table.contains("foo"));
    assert!(table.contains("bar"));
    assert!(table.contains("name"));
    assert!(table.contains("value"));
}

#[test]
fn test_table_to_string_opts_left() {
    let rows = vec![TestRow {
        name: "test".into(),
        value: 42,
    }];
    let table = table_to_string_opts(&rows, None, "left", true);
    assert!(table.contains("test"));
}

#[test]
fn test_table_to_string_opts_center() {
    let rows = vec![TestRow {
        name: "test".into(),
        value: 42,
    }];
    let table = table_to_string_opts(&rows, None, "center", true);
    assert!(table.contains("test"));
}

#[test]
fn test_table_to_string_opts_max_width() {
    let rows = vec![TestRow {
        name: "verylongname".into(),
        value: 42,
    }];
    let table = table_to_string_opts(&rows, Some(5), "left", true);
    // Width truncation should apply
    assert!(!table.is_empty());
}

#[test]
fn test_builder_to_string() {
    let mut builder = Builder::new();
    builder.push_record(["Col1", "Col2"]);
    builder.push_record(["a", "b"]);
    let table = builder_to_string(builder);
    assert!(table.contains("Col1"));
    assert!(table.contains("a"));
}

#[test]
fn test_table_from_vecs() {
    let headers = vec!["H1", "H2"];
    let rows = vec![vec!["r1c1", "r1c2"], vec!["r2c1", "r2c2"]];
    let table = table_from_vecs(&headers, &rows);
    assert!(table.contains("H1"));
    assert!(table.contains("r1c1"));
    assert!(table.contains("r2c2"));
}

#[test]
fn test_table_to_string_styled() {
    let rows = vec![TestRow {
        name: "x".into(),
        value: 1,
    }];

    let ascii = table_to_string_styled(&rows, "ascii");
    assert!(ascii.contains("x"));

    let markdown = table_to_string_styled(&rows, "markdown");
    assert!(markdown.contains("|"));
}

#[test]
fn test_empty_table() {
    let rows: Vec<TestRow> = vec![];
    let table = table_to_string(&rows);
    // Empty table should still have headers
    assert!(table.contains("name"));
}
