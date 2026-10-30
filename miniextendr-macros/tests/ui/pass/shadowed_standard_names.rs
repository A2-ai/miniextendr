//! #1991: generated Rust must resolve its own standard-library names.
//!
//! Every macro expands into the module that invokes it, so a bare `Vec`,
//! `Default` or `vec!` in the expansion resolves to whatever that module has
//! under the name. This module shadows every standard-library name generated
//! code could reach for (prelude types, traits and functions, the std macros
//! and the `std` / `core` / `alloc` roots), then invokes every derive,
//! attribute and function-like macro, so it compiles only if every emitted
//! reference is absolute. The invocations live in `common/every_macro.rs`,
//! shared with `glob_import_method_calls.rs`. rpkg's
//! `shadowed_std_names_tests.rs` covers the paths that need features this
//! crate doesn't have (vctrs, rayon, serde).
#![allow(dead_code, unused_macros, non_camel_case_types)]

// region: shadows

// `macro_rules!` scoping is textual: these shadow the std macros for the rest
// of the file. A shadow that expanded to valid code would detect nothing.
macro_rules! vec { ($($t:tt)*) => { ::core::compile_error!("shadowed `vec!` reached") }; }
macro_rules! format { ($($t:tt)*) => { ::core::compile_error!("shadowed `format!` reached") }; }
macro_rules! format_args { ($($t:tt)*) => { ::core::compile_error!("shadowed `format_args!` reached") }; }
macro_rules! panic { ($($t:tt)*) => { ::core::compile_error!("shadowed `panic!` reached") }; }
macro_rules! assert { ($($t:tt)*) => { ::core::compile_error!("shadowed `assert!` reached") }; }
macro_rules! assert_eq { ($($t:tt)*) => { ::core::compile_error!("shadowed `assert_eq!` reached") }; }
macro_rules! assert_ne { ($($t:tt)*) => { ::core::compile_error!("shadowed `assert_ne!` reached") }; }
macro_rules! debug_assert { ($($t:tt)*) => { ::core::compile_error!("shadowed `debug_assert!` reached") }; }
macro_rules! debug_assert_eq { ($($t:tt)*) => { ::core::compile_error!("shadowed `debug_assert_eq!` reached") }; }
macro_rules! debug_assert_ne { ($($t:tt)*) => { ::core::compile_error!("shadowed `debug_assert_ne!` reached") }; }
macro_rules! unreachable { ($($t:tt)*) => { ::core::compile_error!("shadowed `unreachable!` reached") }; }
macro_rules! unimplemented { ($($t:tt)*) => { ::core::compile_error!("shadowed `unimplemented!` reached") }; }
macro_rules! todo { ($($t:tt)*) => { ::core::compile_error!("shadowed `todo!` reached") }; }
macro_rules! write { ($($t:tt)*) => { ::core::compile_error!("shadowed `write!` reached") }; }
macro_rules! writeln { ($($t:tt)*) => { ::core::compile_error!("shadowed `writeln!` reached") }; }
macro_rules! print { ($($t:tt)*) => { ::core::compile_error!("shadowed `print!` reached") }; }
macro_rules! println { ($($t:tt)*) => { ::core::compile_error!("shadowed `println!` reached") }; }
macro_rules! eprint { ($($t:tt)*) => { ::core::compile_error!("shadowed `eprint!` reached") }; }
macro_rules! eprintln { ($($t:tt)*) => { ::core::compile_error!("shadowed `eprintln!` reached") }; }
macro_rules! dbg { ($($t:tt)*) => { ::core::compile_error!("shadowed `dbg!` reached") }; }
macro_rules! concat { ($($t:tt)*) => { ::core::compile_error!("shadowed `concat!` reached") }; }
macro_rules! stringify { ($($t:tt)*) => { ::core::compile_error!("shadowed `stringify!` reached") }; }
macro_rules! matches { ($($t:tt)*) => { ::core::compile_error!("shadowed `matches!` reached") }; }
macro_rules! env { ($($t:tt)*) => { ::core::compile_error!("shadowed `env!` reached") }; }
macro_rules! option_env { ($($t:tt)*) => { ::core::compile_error!("shadowed `option_env!` reached") }; }
macro_rules! include { ($($t:tt)*) => { ::core::compile_error!("shadowed `include!` reached") }; }
macro_rules! include_str { ($($t:tt)*) => { ::core::compile_error!("shadowed `include_str!` reached") }; }
macro_rules! include_bytes { ($($t:tt)*) => { ::core::compile_error!("shadowed `include_bytes!` reached") }; }
macro_rules! line { ($($t:tt)*) => { ::core::compile_error!("shadowed `line!` reached") }; }
macro_rules! column { ($($t:tt)*) => { ::core::compile_error!("shadowed `column!` reached") }; }
macro_rules! file { ($($t:tt)*) => { ::core::compile_error!("shadowed `file!` reached") }; }
macro_rules! module_path { ($($t:tt)*) => { ::core::compile_error!("shadowed `module_path!` reached") }; }
macro_rules! compile_error { ($($t:tt)*) => { ::core::compile_error!("shadowed `compile_error!` reached") }; }

// Unit structs occupy both the type and the value namespace, so a bare
// generated type, variant or constructor resolves to one of these.
struct Some;
struct None;
struct Ok;
struct Err;
struct Option;
struct Result;
struct Box;
struct Vec;
struct String;
fn drop(_: u8) {}

// Prelude traits. A bare `Default::default()` or `impl Iterator for` in an
// expansion names one of these instead.
trait Default {}
trait Clone {}
trait Copy {}
trait Iterator {}
trait IntoIterator {}
trait DoubleEndedIterator {}
trait ExactSizeIterator {}
trait Extend {}
trait From {}
trait Into {}
trait TryFrom {}
trait TryInto {}
trait ToString {}
trait ToOwned {}
trait AsRef {}
trait AsMut {}
trait PartialEq {}
trait Eq {}
trait PartialOrd {}
trait Ord {}
trait Drop {}
trait Send {}
trait Sync {}
trait Sized {}
trait Unpin {}
trait Fn {}
trait FnMut {}
trait FnOnce {}

// A path such as `std::vec::Vec` without the leading `::` starts here.
mod std {}
mod core {}
mod alloc {}

// endregion

// `::core::` because the `include!` shadow above is in scope.
::core::include!("common/every_macro.rs");

miniextendr_api::miniextendr_init!(shadowed);

fn main() {}
