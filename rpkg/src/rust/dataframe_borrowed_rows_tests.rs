//! Borrowed rows → `data.frame` (#2307).
//!
//! `BorrowedRowsStore` holds `Vec<Row>`s and converts them from `&self`
//! methods, through `rows.as_slice().into_dataframe()` and
//! `(&rows).into_dataframe()`, without cloning the rows. Neither row type is
//! `Clone`. The free functions build the same rows and convert them as an
//! owned `Vec<Row>`, so the tests compare both frames with `identical()`:
//! column names, order and types, factor levels, typed all-`NA` columns and
//! the 0-row frame of an empty store.

use std::collections::BTreeMap;
use std::rc::Rc;

use miniextendr_api::dataframe::{BuiltDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, ExternalPtr, IntoList, miniextendr};

// region: Row types

/// A struct flattened into `inner_*` columns.
#[derive(Clone, DataFrameRow)]
pub struct BorrowedInner {
    pub q: i32,
    pub tag: String,
}

/// A unit enum written as a factor column.
#[derive(Clone, Copy, DataFrameRow)]
pub enum BorrowedLevel {
    Low,
    Mid,
    High,
}

/// A struct written as a list column.
#[derive(Clone, IntoList)]
pub struct BorrowedListed {
    pub p: f64,
}

/// One row with every struct column shape. Not `Clone`; neither is its skipped
/// field.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub struct BorrowedObs {
    pub id: i32,
    pub label: String,
    pub note: Option<String>,
    pub weight: Option<f64>,
    pub flag: Option<bool>,
    pub coords: [f64; 2],
    #[dataframe(width = 2)]
    pub scores: Vec<f64>,
    #[dataframe(expand)]
    pub extra: Vec<i32>,
    #[dataframe(expand)]
    pub words: Vec<String>,
    pub inner: BorrowedInner,
    #[dataframe(as_factor)]
    pub level: BorrowedLevel,
    pub counts: BTreeMap<String, i32>,
    #[dataframe(as_list)]
    pub listed: BorrowedListed,
    #[dataframe(skip)]
    pub cache: Rc<i32>,
}

/// One row per enum column shape. Not `Clone`.
#[derive(DataFrameRow)]
#[dataframe(tag = "variant")]
pub enum BorrowedEvent {
    Point {
        id: i32,
        label: String,
        coords: [f64; 2],
    },
    Tagged {
        id: i32,
        #[dataframe(expand)]
        tags: Vec<String>,
        #[dataframe(as_factor)]
        level: BorrowedLevel,
    },
    Nested {
        inner: BorrowedInner,
        counts: BTreeMap<String, i32>,
    },
    Empty,
}

// endregion

// region: Samples

/// The rows of a `case`: `"full"`, `"na"` (every optional value absent) or
/// `"empty"` (no rows).
pub(crate) fn sample_obs(case: &str) -> Vec<BorrowedObs> {
    let obs = |id: i32, label: &str| BorrowedObs {
        id,
        label: label.to_string(),
        note: None,
        weight: None,
        flag: None,
        coords: [f64::from(id), -f64::from(id)],
        scores: Vec::new(),
        extra: Vec::new(),
        words: Vec::new(),
        inner: BorrowedInner {
            q: id * 10,
            tag: format!("in{id}"),
        },
        level: BorrowedLevel::Low,
        counts: BTreeMap::new(),
        listed: BorrowedListed {
            p: f64::from(id) / 2.0,
        },
        cache: Rc::new(id),
    };
    match case {
        "full" => vec![
            BorrowedObs {
                note: Some("first".to_string()),
                weight: Some(1.5),
                flag: Some(true),
                scores: vec![0.25, 0.5],
                extra: vec![1, 2, 3],
                words: vec!["a".to_string()],
                level: BorrowedLevel::High,
                counts: BTreeMap::from([("x".to_string(), 1), ("y".to_string(), 2)]),
                ..obs(1, "one")
            },
            BorrowedObs {
                note: None,
                weight: Some(-2.0),
                flag: None,
                scores: vec![0.75],
                extra: Vec::new(),
                words: vec!["b".to_string(), "c".to_string()],
                level: BorrowedLevel::Mid,
                ..obs(2, "")
            },
            BorrowedObs {
                note: Some(String::new()),
                flag: Some(false),
                extra: vec![4],
                ..obs(3, "three")
            },
        ],
        "na" => vec![obs(1, "one"), obs(2, "two")],
        "empty" => Vec::new(),
        other => panic!("unknown case {other:?}: use \"full\", \"na\" or \"empty\""),
    }
}

/// The events of a `case`: `"full"` (every variant), `"na"` (only `Empty`, so
/// every payload column is all `NA`) or `"empty"` (no rows).
pub(crate) fn sample_events(case: &str) -> Vec<BorrowedEvent> {
    match case {
        "full" => vec![
            BorrowedEvent::Point {
                id: 1,
                label: "p".to_string(),
                coords: [1.0, 2.0],
            },
            BorrowedEvent::Tagged {
                id: 2,
                tags: vec!["t1".to_string(), "t2".to_string()],
                level: BorrowedLevel::High,
            },
            BorrowedEvent::Nested {
                inner: BorrowedInner {
                    q: 7,
                    tag: "nested".to_string(),
                },
                counts: BTreeMap::from([("k".to_string(), 3)]),
            },
            BorrowedEvent::Empty,
            BorrowedEvent::Tagged {
                id: 5,
                tags: Vec::new(),
                level: BorrowedLevel::Low,
            },
        ],
        "na" => vec![BorrowedEvent::Empty, BorrowedEvent::Empty],
        "empty" => Vec::new(),
        other => panic!("unknown case {other:?}: use \"full\", \"na\" or \"empty\""),
    }
}

// endregion

// region: The store and the owned conversions

/// Rows held by an R object, converted to data frames by reference.
#[derive(ExternalPtr)]
pub struct BorrowedRowsStore {
    obs: Vec<BorrowedObs>,
    events: Vec<BorrowedEvent>,
}

#[miniextendr(env)]
impl BorrowedRowsStore {
    /// Create a store holding the sample rows of one case.
    /// @param case `"full"`, `"na"` (every optional value absent) or `"empty"`.
    pub fn new(case: &str) -> Self {
        BorrowedRowsStore {
            obs: sample_obs(case),
            events: sample_events(case),
        }
    }

    /// The struct rows as a data frame, converted from `&[Row]`.
    pub fn obs_slice(&self) -> BuiltDataFrame {
        self.obs.as_slice().into_dataframe().unwrap()
    }

    /// The struct rows as a data frame, converted from `&Vec<Row>`.
    pub fn obs_vec(&self) -> BuiltDataFrame {
        (&self.obs).into_dataframe().unwrap()
    }

    /// The enum rows as a data frame, converted from `&[Row]`.
    pub fn events_slice(&self) -> BuiltDataFrame {
        self.events.as_slice().into_dataframe().unwrap()
    }

    /// The enum rows as a data frame, converted from `&Vec<Row>`.
    pub fn events_vec(&self) -> BuiltDataFrame {
        (&self.events).into_dataframe().unwrap()
    }

    /// Number of struct rows the store holds (the borrowed conversions leave
    /// them in place).
    pub fn n_obs(&self) -> i32 {
        i32::try_from(self.obs.len()).unwrap()
    }
}

/// The struct rows of a case as a data frame, converted as an owned `Vec<Row>`.
/// @param case `"full"`, `"na"` or `"empty"`.
#[miniextendr]
pub fn df_borrowed_obs_owned(case: &str) -> BuiltDataFrame {
    sample_obs(case).into_dataframe().unwrap()
}

/// The enum rows of a case as a data frame, converted as an owned `Vec<Row>`.
/// @param case `"full"`, `"na"` or `"empty"`.
#[miniextendr]
pub fn df_borrowed_events_owned(case: &str) -> BuiltDataFrame {
    sample_events(case).into_dataframe().unwrap()
}

// endregion
