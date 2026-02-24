//! Unified trait for enum types with string-based choice sets.
//!
//! Both [`RFactor`](crate::RFactor) and [`MatchArg`](crate::MatchArg) represent
//! enums with a fixed set of string labels. `EnumChoices` extracts the common
//! interface so generic code can operate on either.

/// Trait for enum types that map variants to a fixed set of string labels.
///
/// This is automatically implemented by `#[derive(RFactor)]` and `#[derive(MatchArg)]`.
pub trait EnumChoices: Copy + 'static {
    /// The canonical choice strings, in variant declaration order.
    const CHOICES: &'static [&'static str];

    /// Convert a choice string to the corresponding variant.
    ///
    /// Returns `None` if the string doesn't match any choice exactly.
    fn from_str(s: &str) -> Option<Self>;

    /// Convert the variant to its canonical choice string.
    fn to_str(self) -> &'static str;
}
