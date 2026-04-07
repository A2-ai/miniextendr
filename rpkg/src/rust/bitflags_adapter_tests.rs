//! Bitflags adapter tests
use miniextendr_api::bitflags_impl::{
    RFlags, flags_from_i32_strict, flags_from_i32_truncate, flags_to_i32,
};
use miniextendr_api::miniextendr;

miniextendr_api::bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Perms: u32 {
        const READ    = 0b001;
        const WRITE   = 0b010;
        const EXECUTE = 0b100;
    }
}

/// Test RFlags roundtrip through R.
/// @param flags Permission flags as RFlags wrapper.
#[miniextendr]
pub fn bitflags_roundtrip(flags: RFlags<Perms>) -> RFlags<Perms> {
    flags
}

/// Test strict conversion from integer to flags, rejecting unknown bits.
/// @param value Integer bitmask to convert.
#[miniextendr]
pub fn bitflags_from_strict(value: i32) -> Option<i32> {
    let f: Option<Perms> = flags_from_i32_strict(value);
    f.and_then(flags_to_i32)
}

/// Test truncating conversion from integer to flags, ignoring unknown bits.
/// @param value Integer bitmask to convert.
#[miniextendr]
pub fn bitflags_from_truncate(value: i32) -> i32 {
    let f: Perms = flags_from_i32_truncate(value);
    flags_to_i32(f).unwrap()
}

/// Test checking the READ flag on a permission set.
/// @param flags Permission flags as RFlags wrapper.
#[miniextendr]
pub fn bitflags_has_read(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::READ)
}

/// Test checking the WRITE flag on a permission set.
/// @param flags Permission flags as RFlags wrapper.
#[miniextendr]
pub fn bitflags_has_write(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::WRITE)
}

/// Test computing the union (bitwise OR) of two flag sets.
/// @param a First permission flags.
/// @param b Second permission flags.
#[miniextendr]
pub fn bitflags_union(a: RFlags<Perms>, b: RFlags<Perms>) -> RFlags<Perms> {
    RFlags::from(*a | *b)
}

/// Test computing the intersection (bitwise AND) of two flag sets.
/// @param a First permission flags.
/// @param b Second permission flags.
#[miniextendr]
pub fn bitflags_intersect(a: RFlags<Perms>, b: RFlags<Perms>) -> RFlags<Perms> {
    RFlags::from(*a & *b)
}

/// Test that empty flags (no bits set) convert to zero.
#[miniextendr]
pub fn bitflags_empty() -> i32 {
    flags_to_i32(Perms::empty()).unwrap()
}

/// Test that all flags combined produce the expected bitmask.
#[miniextendr]
pub fn bitflags_all() -> i32 {
    flags_to_i32(Perms::all()).unwrap()
}

/// Test checking the EXECUTE flag on a permission set.
/// @param flags Permission flags as RFlags wrapper.
#[miniextendr]
pub fn bitflags_has_execute(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::EXECUTE)
}
