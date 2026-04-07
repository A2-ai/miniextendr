//! URL adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::url_impl::{RUrlOps, Url};

/// Test URL roundtrip through R.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_roundtrip(url: Url) -> Url {
    url
}

/// Test extracting the scheme from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_scheme(url: Url) -> String {
    url.scheme().to_string()
}

/// Test extracting the host from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_host(url: Url) -> Option<String> {
    RUrlOps::host(&url)
}

/// Test extracting the path from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_path(url: Url) -> String {
    RUrlOps::path(&url)
}

/// Test Vec<Url> roundtrip through R character vector.
/// @param urls Character vector of URLs from R.
#[miniextendr]
pub fn url_roundtrip_vec(urls: Vec<Url>) -> Vec<Url> {
    urls
}

/// Test validating whether a string is a valid URL.
/// @param s String to validate as URL.
#[miniextendr]
pub fn url_is_valid(s: String) -> bool {
    miniextendr_api::url_impl::url_helpers::is_valid(&s)
}

/// Test extracting the query string from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_query(url: Url) -> Option<String> {
    RUrlOps::query(&url)
}

/// Test extracting the fragment from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_fragment(url: Url) -> Option<String> {
    RUrlOps::fragment(&url)
}

/// Test extracting the port or known default port from a URL.
/// @param url Parsed URL from R string.
#[miniextendr]
pub fn url_port_or_default(url: Url) -> Option<i32> {
    RUrlOps::port_or_known_default(&url).map(|p| p as i32)
}

/// Test extracting all URL components (scheme, host, path, query, fragment).
#[miniextendr]
pub fn url_full_components() -> Vec<String> {
    let url = Url::parse("https://user:pass@example.com:8080/path?q=1#frag").unwrap();
    vec![
        RUrlOps::scheme(&url),
        RUrlOps::host(&url).unwrap_or_default(),
        RUrlOps::path(&url),
        RUrlOps::query(&url).unwrap_or_default(),
        RUrlOps::fragment(&url).unwrap_or_default(),
    ]
}
