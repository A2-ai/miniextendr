//! Bitflags adapter tests
use miniextendr_api::bitflags_impl::{
    RFlags, flags_from_i32_strict, flags_from_i32_truncate, flags_to_i32,
};
use miniextendr_api::{miniextendr, miniextendr_module};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Perms: u32 {
        const READ    = 0b001;
        const WRITE   = 0b010;
        const EXECUTE = 0b100;
    }
}

/// @noRd
#[miniextendr]
pub fn bitflags_roundtrip(flags: RFlags<Perms>) -> RFlags<Perms> {
    flags
}

/// @noRd
#[miniextendr]
pub fn bitflags_from_strict(value: i32) -> Option<i32> {
    let f: Option<Perms> = flags_from_i32_strict(value);
    f.and_then(flags_to_i32)
}

/// @noRd
#[miniextendr]
pub fn bitflags_from_truncate(value: i32) -> i32 {
    let f: Perms = flags_from_i32_truncate(value);
    flags_to_i32(f).unwrap()
}

/// @noRd
#[miniextendr]
pub fn bitflags_has_read(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::READ)
}

/// @noRd
#[miniextendr]
pub fn bitflags_has_write(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::WRITE)
}

/// @noRd
#[miniextendr]
pub fn bitflags_union(a: RFlags<Perms>, b: RFlags<Perms>) -> RFlags<Perms> {
    RFlags::from(*a | *b)
}

/// Intersect two flags (bitwise AND)
/// @noRd
#[miniextendr]
pub fn bitflags_intersect(a: RFlags<Perms>, b: RFlags<Perms>) -> RFlags<Perms> {
    RFlags::from(*a & *b)
}

/// Empty flags (no bits set)
/// @noRd
#[miniextendr]
pub fn bitflags_empty() -> i32 {
    flags_to_i32(Perms::empty()).unwrap()
}

/// All flags combined
/// @noRd
#[miniextendr]
pub fn bitflags_all() -> i32 {
    flags_to_i32(Perms::all()).unwrap()
}

/// Check EXECUTE flag
/// @noRd
#[miniextendr]
pub fn bitflags_has_execute(flags: RFlags<Perms>) -> bool {
    flags.contains(Perms::EXECUTE)
}

miniextendr_module! {
    mod bitflags_adapter_tests;
    fn bitflags_roundtrip;
    fn bitflags_from_strict;
    fn bitflags_from_truncate;
    fn bitflags_has_read;
    fn bitflags_has_write;
    fn bitflags_union;
    fn bitflags_intersect;
    fn bitflags_empty;
    fn bitflags_all;
    fn bitflags_has_execute;
}
