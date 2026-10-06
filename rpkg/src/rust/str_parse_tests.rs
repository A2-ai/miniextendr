//! Fixtures for `try_from_sexp_via_str_parse!` on a type defined in this crate
//! (#1766): `T`, `Option<T>`, `Vec<T>` and `Vec<Option<T>>` arguments, with
//! the NA policy and batched element errors of the built-in uuid / url /
//! regex / num-bigint parsers.

use miniextendr_api::miniextendr;

/// A `major.minor.patch` version, parsed from an R string.
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl std::str::FromStr for Version {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.').map(str::parse::<u32>);
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch)), None) => Ok(Version {
                major,
                minor,
                patch,
            }),
            _ => Err("expected major.minor.patch"),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

miniextendr_api::try_from_sexp_via_str_parse!(Version, "version", |s| s.parse::<Version>());

/// The version, normalised.
#[miniextendr(noexport)]
pub fn str_parse_version(x: Version) -> String {
    x.to_string()
}

/// The version, normalised; `NA` / `NULL` gives `NA`.
#[miniextendr(noexport)]
pub fn str_parse_version_opt(x: Option<Version>) -> Option<String> {
    x.map(|v| v.to_string())
}

/// The versions, normalised.
#[miniextendr(noexport)]
pub fn str_parse_version_vec(x: Vec<Version>) -> Vec<String> {
    x.iter().map(Version::to_string).collect()
}

/// The versions, normalised, keeping `NA`.
#[miniextendr(noexport)]
pub fn str_parse_version_vec_opt(x: Vec<Option<Version>>) -> Vec<Option<String>> {
    x.iter()
        .map(|v| v.as_ref().map(Version::to_string))
        .collect()
}
