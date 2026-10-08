//! Aliases for `match_arg` choices (#1843): `#[match_arg(alias = "grey")]`
//! on `Shade::Gray` lets a user write `"grey"` for the choice `"gray"`
//! without making it a fourth choice. The alias is matched only when typed in
//! full (`"gr"` stays the prefix of `"gray"`), on every parameter shape below
//! and by the Rust matchers, and appears nowhere the choices are listed: the
//! formal, the usage, the `@param` line and the errors.

use miniextendr_api::{MatchArg, Missing, SEXP, miniextendr};

/// A colour with an alias on one choice.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum Shade {
    Red,
    #[match_arg(alias = "grey")]
    Gray,
    Blue,
}

/// `color` matched by the generated wrapper.
///
/// @param color A colour.
#[miniextendr(internal)]
pub fn match_arg_alias_shade(#[miniextendr(match_arg)] color: Shade) -> String {
    format!("{color:?}")
}

/// `color` with a `NULL` default: `NULL` is no colour.
///
/// @param color A colour, or `NULL`.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_optional(#[miniextendr(match_arg)] color: Option<Shade>) -> String {
    match color {
        Some(color) => format!("{color:?}"),
        None => "none".to_string(),
    }
}

/// `color` that may be omitted.
///
/// @param color A colour; omitted is reported as such.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_omitted(#[miniextendr(match_arg)] color: Missing<Shade>) -> String {
    match color {
        Missing::Absent => "absent".to_string(),
        Missing::Present(color) => format!("{color:?}"),
    }
}

/// Several colours, each matched on its own.
///
/// @param colors One or more colours.
#[miniextendr(internal)]
pub fn match_arg_alias_shades(#[miniextendr(match_arg, several_ok)] colors: Vec<Shade>) -> String {
    colors
        .iter()
        .map(|color| format!("{color:?}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// `color` whose default is `"blue"`.
///
/// @param color A colour, `"blue"` by default.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_default(
    #[miniextendr(match_arg, default = "\"blue\"")] color: Shade,
) -> String {
    format!("{color:?}")
}

/// `color` next to a parameter named `c`: the choices and the aliases are
/// written with `base::c`.
///
/// @param color A colour.
/// @param c A number that shadows `c()`.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_shadowed(#[miniextendr(match_arg)] color: Shade, c: i32) -> String {
    format!("{color:?}:{c}")
}

/// `color` converted by `Shade`'s own `TryFromSexp`, with no `match_arg`
/// check in the wrapper.
///
/// @param color A colour.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_converted(color: Shade) -> String {
    format!("{color:?}")
}

/// `color` matched by the body with `match_arg_param()`.
///
/// @param color A colour.
#[miniextendr(internal)]
pub fn match_arg_alias_shade_param(color: SEXP) -> String {
    let color: Shade =
        miniextendr_api::match_arg_param(color, "color").unwrap_or_else(|e| e.raise());
    format!("{color:?}")
}

/// A colour or a number.
///
/// @param color A colour, or a number.
#[cfg(feature = "either")]
#[miniextendr(internal)]
pub fn match_arg_alias_shade_or_number(
    #[miniextendr(match_arg)] color: miniextendr_api::either_impl::Either<Shade, f64>,
) -> String {
    match color {
        miniextendr_api::either_impl::Either::Left(color) => format!("{color:?}"),
        miniextendr_api::either_impl::Either::Right(n) => format!("number:{n}"),
    }
}

// region: an impl-block method

/// A palette holding one colour, for the method-level `match_arg(p)`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct AliasPalette {
    color: Shade,
}

#[miniextendr(env)]
impl AliasPalette {
    #[miniextendr(match_arg(color))]
    pub fn new(color: Shade) -> Self {
        Self { color }
    }

    /// Replace the colour and return it.
    #[miniextendr(match_arg(color))]
    pub fn paint(&mut self, color: Shade) -> String {
        self.color = color;
        format!("{:?}", self.color)
    }
}

// endregion
