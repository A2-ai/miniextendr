//! An alias's error visibility must be a boolean in its own attribute.
use miniextendr_api::MatchArg;

#[derive(Copy, Clone, MatchArg)]
enum StringListed {
    #[match_arg(alias = "oral", listed = "no")]
    Extravascular,
}

#[derive(Copy, Clone, MatchArg)]
enum NumberListed {
    #[match_arg(alias = "oral", listed = 0)]
    Extravascular,
}

#[derive(Copy, Clone, MatchArg)]
enum NoAlias {
    #[match_arg(listed = false)]
    Extravascular,
}

#[derive(Copy, Clone, MatchArg)]
enum DuplicateListed {
    #[match_arg(alias = "oral", listed = true, listed = false)]
    Extravascular,
}

fn main() {}
