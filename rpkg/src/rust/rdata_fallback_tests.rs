//! Fixtures for the field fallback of `r_data_accessors` (#2150):
//! `fallback = "Self::f"` answers, on the bare pointer, the names no
//! `#[r_data]` field has, and `fallback_names = "Self::g"` adds its names to
//! `.DollarNames()`.
//!
//! - `FallbackS3` (`fallback_rec`): the default lookup, with the package's
//!   `field_error` / `restored(refuse = "every_read")` options and a second
//!   impl block that adds a method.
//! - `FallbackList` (`fallback_list`): `lookup = "list"`, a fallback whose
//!   value is a scalar `Option`, and no `every_read`.
//! - `FallbackS4` and `FallbackEnv`: the S4 and env readers.
//!
//! Tests live in `rpkg/tests/testthat/test-sidecar-field-fallback.R`.

use miniextendr_api::externalptr::{Computed, RSidecar};
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

// region: S3, the default lookup

/// A results object whose `#[r_data]` fields are few and whose table's
/// columns `$` / `[[` read through the fallback: `x$alpha` reads the column
/// `alpha` when no field is named `alpha`. The table also has a column named
/// `id`, which the field of that name hides.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(
    s3,
    field_error(
        class = ["fallback_error_field", "fallback_error"],
        no_field_hint = "Its columns are `x$columns`."
    ),
    restored(class = ["fallback_error_saved", "fallback_error"], refuse = "every_read")
)]
pub struct FallbackS3 {
    #[r_data]
    _r: RSidecar,

    /// The id, a struct field.
    #[r_data]
    pub id: i32,

    /// The table's column names, computed.
    #[r_data(get = "Self::column_names")]
    pub columns: Computed,

    /// The table: named numeric columns.
    table: Vec<(String, Vec<f64>)>,
}

impl FallbackS3 {
    /// The column `name`, `None` when the table has none. `boom` raises the
    /// fallback's own classed error.
    fn column(&self, name: &str) -> Option<Vec<f64>> {
        if name == "boom" {
            miniextendr_api::rust_error!(
                class = "fallback_boom",
                "no column of table {} can be named `boom`",
                self.id
            )
        }
        self.table
            .iter()
            .find(|(column, _)| column == name)
            .map(|(_, values)| values.clone())
    }

    /// The table's column names, in order.
    fn column_names(&self) -> Vec<String> {
        self.table
            .iter()
            .map(|(column, _)| column.clone())
            .collect()
    }
}

/// A results object of the table `alpha = c(1, 2)`, `beta = id`,
/// `id = 99` and `aleph = numeric()`; `$` and `[[` read a column no field
/// hides, and `$<-` / `[[<-` write a field only.
#[miniextendr(
    s3(
        r_data_accessors,
        fallback = "Self::column",
        fallback_names = "Self::column_names"
    ),
    class = "fallback_rec"
)]
impl FallbackS3 {
    /// Create a `fallback_rec`.
    /// @param id The id; the column `beta` holds it too.
    pub fn new(id: i32) -> Self {
        FallbackS3 {
            _r: RSidecar,
            id,
            columns: Computed,
            table: vec![
                ("alpha".to_string(), vec![1.0, 2.0]),
                ("beta".to_string(), vec![f64::from(id)]),
                ("id".to_string(), vec![99.0]),
                ("aleph".to_string(), vec![]),
            ],
        }
    }
}

/// A second impl block of `fallback_rec`, without the fallback: the block
/// with the constructor has it, and its readers serve the class.
#[miniextendr(s3(r_data_accessors), class = "fallback_rec")]
impl FallbackS3 {
    /// The number of columns.
    pub fn n_columns(&self) -> i32 {
        i32::try_from(self.table.len()).expect("column count exceeds i32")
    }
}
// endregion

// region: S3, a list's lookup and a scalar fallback

/// A record read with a list's lookup, whose fallback's value is a scalar:
/// `x$double` is twice the level, `x$unset` is `NA` (the fallback's
/// `Some(None)`), and any other name reads `NULL` (its `None`), never `NA`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct FallbackList {
    #[r_data]
    _r: RSidecar,

    /// The level, a struct field.
    #[r_data]
    pub level: i32,
}

impl FallbackList {
    /// `double` and `unset`, else `None`.
    fn setting(&self, name: &str) -> Option<Option<i32>> {
        match name {
            "double" => Some(Some(self.level * 2)),
            "unset" => Some(None),
            _ => None,
        }
    }

    /// The names `setting` answers.
    fn setting_names(&self) -> Vec<String> {
        vec!["double".to_string(), "unset".to_string()]
    }
}

/// The readers of `fallback_list`, with a list's lookup.
#[miniextendr(
    s3(
        r_data_accessors = "get",
        lookup = "list",
        fallback = "Self::setting",
        fallback_names = "Self::setting_names"
    ),
    class = "fallback_list"
)]
impl FallbackList {
    /// Create a `fallback_list`.
    /// @param level The level.
    pub fn new(level: i32) -> Self {
        FallbackList {
            _r: RSidecar,
            level,
        }
    }
}
// endregion

// region: S4

/// An S4 object whose `$` reads `twice` through the fallback, a scalar: its
/// `None` raises `miniextendr_no_field`, as an unknown name does.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s4)]
pub struct FallbackS4 {
    #[r_data]
    _r: RSidecar,

    /// The depth, a struct field.
    #[r_data]
    pub depth: i32,
}

impl FallbackS4 {
    /// `twice`, else `None`.
    fn setting(&self, name: &str) -> Option<i32> {
        (name == "twice").then_some(self.depth * 2)
    }

    /// The names `setting` answers.
    fn setting_names(&self) -> Vec<String> {
        vec!["twice".to_string()]
    }
}

/// The S4 class `FallbackS4`, with field syntax and a fallback.
#[miniextendr(s4(
    r_data_accessors,
    fallback = "Self::setting",
    fallback_names = "Self::setting_names"
))]
impl FallbackS4 {
    /// Create a `FallbackS4`.
    /// @param depth The depth.
    pub fn new(depth: i32) -> Self {
        FallbackS4 {
            _r: RSidecar,
            depth,
        }
    }
}
// endregion

// region: env

/// An env object whose `$` reads a field, then a method, then a word of its
/// dictionary through the fallback, whose `None` reads `NULL`. The
/// dictionary has a `kind` and a `describe`, which the field and the method
/// of those names hide.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct FallbackEnv {
    #[r_data]
    _r: RSidecar,

    /// The kind, a struct field.
    #[r_data]
    pub kind: String,

    /// The dictionary.
    words: Vec<(String, Vec<String>)>,
}

impl FallbackEnv {
    /// The word `name`, `None` when the dictionary has none.
    fn word(&self, name: &str) -> Option<Vec<String>> {
        self.words
            .iter()
            .find(|(word, _)| word == name)
            .map(|(_, values)| values.clone())
    }

    /// The dictionary's words, in order.
    fn word_names(&self) -> Vec<String> {
        self.words.iter().map(|(word, _)| word.clone()).collect()
    }
}

/// The env class `FallbackEnv`, with field syntax and a fallback.
#[miniextendr(env(
    r_data_accessors,
    fallback = "Self::word",
    fallback_names = "Self::word_names"
))]
impl FallbackEnv {
    /// Create a `FallbackEnv` whose dictionary has `greeting`, `kind` and
    /// `describe`.
    /// @param kind The kind.
    pub fn new(kind: String) -> Self {
        FallbackEnv {
            _r: RSidecar,
            kind,
            words: vec![
                ("greeting".to_string(), vec!["hello".to_string()]),
                ("kind".to_string(), vec!["hidden by the field".to_string()]),
                (
                    "describe".to_string(),
                    vec!["hidden by the method".to_string()],
                ),
            ],
        }
    }

    /// The kind and the number of words.
    pub fn describe(&self) -> String {
        format!("{} with {} words", self.kind, self.words.len())
    }

    /// Calls `f()` while this method holds the object mutably: a read of the
    /// object in `f`, the fallback's included, is a borrow conflict.
    /// @param f An R function of no arguments.
    pub fn hold(&mut self, f: SEXP) {
        use miniextendr_api::expression::RCall;
        unsafe {
            RCall::from_sexp(f).eval_with_handlers(miniextendr_api::sys::R_GlobalEnv);
        }
        self.kind.push('!');
    }
}
// endregion
