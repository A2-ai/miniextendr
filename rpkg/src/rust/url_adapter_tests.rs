//! URL adapter tests
use miniextendr_api::url_impl::{Url, RUrlOps};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn url_roundtrip(url: Url) -> Url {
    url
}

/// @noRd
#[miniextendr]
pub fn url_scheme(url: Url) -> String {
    url.scheme().to_string()
}

/// @noRd
#[miniextendr]
pub fn url_host(url: Url) -> Option<String> {
    RUrlOps::host(&url)
}

/// @noRd
#[miniextendr]
pub fn url_path(url: Url) -> String {
    RUrlOps::path(&url)
}

/// @noRd
#[miniextendr]
pub fn url_roundtrip_vec(urls: Vec<Url>) -> Vec<Url> {
    urls
}

/// @noRd
#[miniextendr]
pub fn url_is_valid(s: String) -> bool {
    miniextendr_api::url_impl::url_helpers::is_valid(&s)
}

/// Extract query string from URL
/// @noRd
#[miniextendr]
pub fn url_query(url: Url) -> Option<String> {
    RUrlOps::query(&url)
}

/// Extract fragment from URL
/// @noRd
#[miniextendr]
pub fn url_fragment(url: Url) -> Option<String> {
    RUrlOps::fragment(&url)
}

/// Get port or known default
/// @noRd
#[miniextendr]
pub fn url_port_or_default(url: Url) -> Option<i32> {
    RUrlOps::port_or_known_default(&url).map(|p| p as i32)
}

/// URL with all components
/// @noRd
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

miniextendr_module! {
    mod url_adapter_tests;
    fn url_roundtrip;
    fn url_scheme;
    fn url_host;
    fn url_path;
    fn url_roundtrip_vec;
    fn url_is_valid;
    fn url_query;
    fn url_fragment;
    fn url_port_or_default;
    fn url_full_components;
}
