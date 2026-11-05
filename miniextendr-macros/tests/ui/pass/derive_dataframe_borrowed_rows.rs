//! Compile-pass test: borrowed rows (`&[Row]` / `&Vec<Row>`) convert to a
//! data frame wherever every value the fill clones is `Clone`, for every column
//! shape, without the row type being `Clone` (#2307). A row with a non-`Clone`
//! field still derives and converts as an owned `Vec<Row>`.

#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use miniextendr_api::{DataFrameRow, IntoDataFrame, IntoList};

/// `&[T]` and `&Vec<T>` both convert.
fn borrowed<T>()
where
    for<'r> &'r [T]: IntoDataFrame,
    for<'r> &'r Vec<T>: IntoDataFrame,
{
}

#[derive(Clone, DataFrameRow)]
struct Inner {
    q: i32,
    label: String,
}

#[derive(Clone, Copy, DataFrameRow)]
enum Level {
    Low,
    High,
}

#[derive(Clone, IntoList)]
struct Listed {
    p: f64,
}

#[derive(Clone, DataFrameRow)]
#[dataframe(tag = "variant")]
enum Shape {
    Circle { r: f64 },
    Square { side: f64, name: String },
}

/// Every struct column shape. Not `Clone` itself, and neither is its skipped field.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
struct Every<'a> {
    id: i32,
    name: String,
    qualified: ::std::string::String,
    note: Option<String>,
    weight: Option<f64>,
    text: &'a str,
    values: Vec<f64>,
    by_key: HashMap<String, i32>,
    by_int: BTreeMap<i32, f64>,
    pair: [String; 2],
    borrowed_pair: &'a [f64; 2],
    #[dataframe(width = 2)]
    firsts: Vec<String>,
    #[dataframe(width = 2)]
    borrowed_firsts: &'a [f64],
    #[dataframe(expand)]
    rest: Vec<String>,
    #[dataframe(expand)]
    boxed_rest: Box<[i32]>,
    #[dataframe(expand)]
    borrowed_rest: &'a [f64],
    inner: Inner,
    shape: Shape,
    #[dataframe(as_factor)]
    level: Level,
    #[dataframe(as_list)]
    listed: Listed,
    #[dataframe(rename = "type")]
    r#type: String,
    #[dataframe(skip)]
    cache: Rc<i32>,
}

/// Columns named like the fills' own bindings.
#[derive(DataFrameRow)]
struct Shadowing {
    rows: i32,
    len: String,
    row: f64,
    _n_rows: Option<String>,
}

#[derive(DataFrameRow)]
enum ShadowingEvent {
    A { rows: i32, len: String },
    B { row: f64 },
}

#[derive(DataFrameRow)]
struct Tuple(f64, String, Option<String>, #[dataframe(width = 2)] Vec<String>);

#[derive(DataFrameRow)]
struct Unit;

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
struct TagOnly;

/// Every enum column shape.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Event<'a> {
    A {
        id: i32,
        name: String,
        pair: [String; 2],
        borrowed_pair: &'a [f64; 2],
        #[dataframe(width = 2)]
        firsts: Vec<String>,
        #[dataframe(width = 2)]
        borrowed_firsts: &'a [f64],
        #[dataframe(expand)]
        rest: Vec<i32>,
        inner: Inner,
        shape: Shape,
        #[dataframe(as_factor)]
        level: Level,
        #[dataframe(as_list)]
        listed: Listed,
        by_int: BTreeMap<i32, f64>,
        #[dataframe(skip)]
        cache: Rc<i32>,
    },
    B(i32, String),
    C {
        id: i32,
        text: &'a str,
        #[dataframe(expand)]
        rest: Vec<i32>,
    },
    Empty,
}

#[derive(DataFrameRow)]
#[dataframe(conflicts = "string")]
enum Mixed {
    Number { x: f64 },
    Text { x: String },
}

#[derive(DataFrameRow)]
enum Untagged {
    One { name: String },
    Two,
}

/// Field types that are not `Clone`: a flattened struct, a list column and a
/// factor.
#[derive(DataFrameRow)]
struct InnerNotClone {
    q: i32,
}

#[derive(IntoList)]
struct ListedNotClone {
    p: f64,
}

#[derive(DataFrameRow)]
enum LevelNotClone {
    Low,
    High,
}

/// Non-`Clone` fields: no borrowed conversion, but the derive and the owned
/// conversion are unaffected.
#[derive(DataFrameRow)]
struct NotClone {
    id: i32,
    name: String,
    inner: InnerNotClone,
    #[dataframe(as_list)]
    listed: ListedNotClone,
    #[dataframe(as_factor)]
    level: LevelNotClone,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum NotCloneEvent {
    A { inner: InnerNotClone },
    B {
        id: i32,
        #[dataframe(as_list)]
        listed: ListedNotClone,
    },
}

fn owned(rows: Vec<NotClone>, events: Vec<NotCloneEvent>) {
    let _ = rows.into_dataframe();
    let _ = events.into_dataframe();
}

/// From a `&self` method: the slice and the `&Vec` spelling.
struct Store<'a> {
    rows: Vec<Every<'a>>,
    events: Vec<Event<'a>>,
}

impl Store<'_> {
    fn frames(&self) {
        let _ = self.rows.as_slice().into_dataframe();
        let _ = (&self.rows).into_dataframe();
        let _ = self.events.as_slice().into_dataframe();
        let _ = (&self.events).into_dataframe();
    }
}

fn main() {
    borrowed::<Inner>();
    borrowed::<Level>();
    borrowed::<Shape>();
    borrowed::<Every<'static>>();
    borrowed::<Shadowing>();
    borrowed::<ShadowingEvent>();
    borrowed::<Tuple>();
    borrowed::<Unit>();
    borrowed::<TagOnly>();
    borrowed::<Event<'static>>();
    borrowed::<Mixed>();
    borrowed::<Untagged>();
}
