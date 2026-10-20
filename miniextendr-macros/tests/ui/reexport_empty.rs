//! Test: `reexport!` needs at least one `pkg::name` item (#1910).

use miniextendr_api::reexport;

reexport!();

fn main() {}
