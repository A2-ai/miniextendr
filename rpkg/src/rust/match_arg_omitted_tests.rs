//! Omittable choice parameters: `match_arg` / `choices` on `Missing<..>`
//! (#1551). The formal keeps the choice vector and an omitted argument reaches
//! Rust as `Missing::Absent`.
//!
//! These fixtures have their own file, so their own help page: on
//! `match_arg_tests.Rd` other functions document `mode` / `modes` themselves,
//! and an author description beats a generated `@param` line there (#1590),
//! so the omission wording would never show.

use crate::match_arg_tests::Mode;
use miniextendr_api::Missing;

// region: Omitted choice — Missing<T> (#1551)

/// Omittable optional mode: the formal keeps the choice vector, an omitted
/// argument is `Absent` and `NULL` is `Present(None)`.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_omitted_mode(#[miniextendr(match_arg)] mode: Missing<Option<Mode>>) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(m)) => format!("{m:?}"),
    }
}

/// Omittable mode without `Option`: `NULL` selects the first choice, as for a
/// plain `Mode`.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_omitted_plain(#[miniextendr(match_arg)] mode: Missing<Mode>) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(m) => format!("{m:?}"),
    }
}

/// Omittable several_ok list: an omitted argument is `Absent`, `NULL` still
/// selects every choice.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_omitted_modes(
    #[miniextendr(match_arg, several_ok)] modes: Missing<Vec<Mode>>,
) -> String {
    match modes {
        Missing::Absent => "absent".to_string(),
        Missing::Present(v) => v
            .iter()
            .map(|m| format!("{m:?}"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Omittable several_ok list in a boxed slice.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn match_arg_omitted_modes_boxed(
    #[miniextendr(match_arg, several_ok)] modes: Missing<Box<[Mode]>>,
) -> String {
    match modes {
        Missing::Absent => "absent".to_string(),
        Missing::Present(v) => format!("{} modes", v.len()),
    }
}

/// Omittable optional inline choice on a `Missing<Option<String>>`.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn choices_omitted_color(
    #[miniextendr(choices("red", "green", "blue"))] color: Missing<Option<String>>,
) -> String {
    match color {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(c)) => c,
    }
}

/// Omittable several_ok inline choice on a `Missing<Vec<String>>`.
///
/// @export
#[miniextendr_api::miniextendr]
pub fn choices_omitted_colors(
    #[miniextendr(choices("red", "green", "blue"), several_ok)] colors: Missing<Vec<String>>,
) -> String {
    match colors {
        Missing::Absent => "absent".to_string(),
        Missing::Present(v) => v.join(", "),
    }
}
// endregion
