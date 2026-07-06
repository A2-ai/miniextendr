//! Fixture for `Option<Self>` static returns (#1164).
//!
//! `try_find` is the lookup-shaped fallible constructor: `Some(Self)` wraps as
//! a class object exactly like `$new()`, while `None` raises an R error (the
//! normal `Option` error path). Symmetric with the `Result<Self, E>` fixture
//! (`SerdeRPoint$from_r` in `serde_r_tests.rs`, audit A4).

use miniextendr_api::miniextendr;

/// A small in-memory registry entry, looked up by id.
#[derive(Debug, Clone, PartialEq, miniextendr_api::ExternalPtr)]
pub struct OptionSelfLookup {
    id: i32,
    label: String,
}

/// @name rpkg_option_self_lookup
/// @aliases OptionSelfLookup
#[miniextendr]
impl OptionSelfLookup {
    /// Create a new entry directly.
    /// @param id Integer id.
    /// @param label Entry label.
    pub fn new(id: i32, label: String) -> Self {
        OptionSelfLookup { id, label }
    }

    /// Look up a known entry by id — the `Option<Self>` lookup-shaped
    /// fallible constructor (#1164). Returns the entry for ids 1..=3;
    /// `None` (raised as an R error) otherwise.
    /// @param id Integer id to look up.
    pub fn try_find(id: i32) -> Option<Self> {
        let label = match id {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => return None,
        };
        Some(OptionSelfLookup {
            id,
            label: label.to_string(),
        })
    }

    /// The entry's id.
    pub fn id(&self) -> i32 {
        self.id
    }

    /// The entry's label.
    pub fn label(&self) -> String {
        self.label.clone()
    }
}
