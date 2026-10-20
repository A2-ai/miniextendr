//! Fixtures for `ProtectPool` as a per-call root set (#1909).
//!
//! `Columns` owns a pool and roots each column it builds with `root`. A struct
//! that owned a `ProtectScope` instead would unprotect with `UNPROTECT(n)`,
//! which pops the top of R's protect stack whoever pushed it, so dropping it
//! out of order would unprotect another struct's columns. The pool takes no
//! protect-stack entry per value and releases only its own list.

use miniextendr_api::prelude::*;

/// Named columns, each rooted in the struct's own pool until it drops.
struct Columns {
    roots: ProtectPool,
    cols: Vec<(String, SEXP)>,
}

impl Columns {
    fn new() -> Self {
        Self {
            // Capacity 1, so most `root` calls grow the pool: the path that
            // must keep a fresh value reachable while it allocates.
            roots: ProtectPool::new(1),
            cols: Vec::new(),
        }
    }

    /// Convert `value`, root it, and keep it as column `name`.
    fn push(&mut self, name: &str, value: impl IntoR) {
        let sexp = self.roots.root(value);
        self.cols.push((name.to_owned(), sexp));
    }

    /// The column called `name`, rooted while `self` lives.
    fn column(&self, name: &str) -> SEXP {
        self.cols
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, sexp)| *sexp)
            .expect("no such column")
    }

    /// The columns as a named list. They stay rooted by the pool while the
    /// list allocates; the pool drops once the list holds them.
    fn into_list(self) -> NamedListBuilder {
        let mut list = NamedListBuilder::with_capacity(self.cols.len());
        for (name, sexp) in &self.cols {
            list.push_sexp(name.as_str(), *sexp);
        }
        list
    }
}

/// Build four columns, each an allocation, in a struct that owns a pool.
fn columns() -> NamedListBuilder {
    let mut cols = Columns::new();
    cols.push("id", vec![1, 2, 3]);
    cols.push("value", vec![0.5, 1.5, 2.5]);
    cols.push("label", vec!["a", "b", "c"]);
    cols.push("flag", vec![Some(true), Some(false), None]);
    cols.into_list()
}

/// Two column structs, the older dropped first: out of creation order.
fn drop_out_of_order() -> NamedListBuilder {
    let mut older = Columns::new();
    older.push("ints", vec![1, 2, 3]);
    older.push("label", vec!["a", "b", "c"]);
    let mut newer = Columns::new();
    newer.push("reals", vec![0.5, 1.5]);
    // Keep one of the older struct's columns past its drop.
    newer.push("ints", older.column("ints"));
    // A `ProtectScope` in `older` would pop `newer`'s entries here.
    drop(older);
    // Allocate after the drop: `newer`'s columns must still be rooted.
    newer.push("strings", vec!["x", "y", "z"]);
    newer.into_list()
}

/// Columns built by a struct that owns a `ProtectPool`, as a named list.
#[miniextendr(noexport)]
pub fn root_pool_columns() -> NamedListBuilder {
    columns()
}

/// Columns kept by the newer of two pool-owning structs after the older one
/// drops first, out of creation order.
#[miniextendr(noexport)]
pub fn root_pool_drop_out_of_order() -> NamedListBuilder {
    drop_out_of_order()
}

/// Drive both root-pool fixtures, plus a pool that grows many times, under GC
/// pressure. No arguments, so the no-arg GC sweep exercises every path.
#[miniextendr(noexport)]
pub fn gc_stress_root_pool() -> NamedListBuilder {
    let mut many = Columns::new();
    for i in 1..=20 {
        many.push(&format!("c{i}"), vec![i; 3]);
    }
    let mut result = NamedListBuilder::with_capacity(3);
    result.push("columns", columns());
    result.push("out_of_order", drop_out_of_order());
    result.push("many", many.into_list());
    result
}
