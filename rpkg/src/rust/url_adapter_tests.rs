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

miniextendr_module! {
    mod url_adapter_tests;
    fn url_roundtrip;
    fn url_scheme;
    fn url_host;
    fn url_path;
    fn url_roundtrip_vec;
    fn url_is_valid;
}
