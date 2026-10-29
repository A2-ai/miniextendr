use super::*;

#[test]
fn test_url_parse() {
    let url = Url::parse("https://example.com/path?query=1#frag").unwrap();
    assert_eq!(url.scheme(), "https");
    assert_eq!(url.host_str(), Some("example.com"));
    assert_eq!(url.path(), "/path");
    assert_eq!(url.query(), Some("query=1"));
    assert_eq!(url.fragment(), Some("frag"));
}

#[test]
fn test_url_helpers_parse() {
    assert!(url_helpers::parse("https://example.com").is_ok());
    assert!(url_helpers::parse("not a url").is_err());
}

#[test]
fn test_url_helpers_join() {
    let base = Url::parse("https://example.com/base/").unwrap();
    let joined = url_helpers::join(&base, "path").unwrap();
    assert_eq!(joined.as_str(), "https://example.com/base/path");
}

#[test]
fn test_url_helpers_is_valid() {
    assert!(url_helpers::is_valid("https://example.com"));
    assert!(url_helpers::is_valid("http://localhost:8080"));
    assert!(!url_helpers::is_valid("not a url"));
    assert!(!url_helpers::is_valid(""));
}

#[test]
fn test_rurlops_basics() {
    let url = Url::parse("https://user:pass@example.com:8080/path?q=1#frag").unwrap();

    assert_eq!(RUrlOps::scheme(&url), "https");
    assert_eq!(RUrlOps::host(&url), Some("example.com".to_string()));
    assert_eq!(RUrlOps::port(&url), Some(8080));
    assert_eq!(RUrlOps::path(&url), "/path");
    assert_eq!(RUrlOps::query(&url), Some("q=1".to_string()));
    assert_eq!(RUrlOps::fragment(&url), Some("frag".to_string()));
    assert_eq!(RUrlOps::username(&url), "user");
    assert_eq!(RUrlOps::password(&url), Some("pass".to_string()));
}

#[test]
fn test_rurlops_defaults() {
    let url = Url::parse("https://example.com").unwrap();

    assert_eq!(RUrlOps::port(&url), None);
    assert_eq!(RUrlOps::port_or_known_default(&url), Some(443));
    assert_eq!(RUrlOps::query(&url), None);
    assert_eq!(RUrlOps::fragment(&url), None);
    assert_eq!(RUrlOps::password(&url), None);
}
