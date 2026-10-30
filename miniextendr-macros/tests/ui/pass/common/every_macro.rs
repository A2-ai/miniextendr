// Every derive, attribute and function-like macro, invoked once each.
//
// Included by the pass fixtures that put generated code in a hostile scope:
// `shadowed_standard_names.rs` (std names shadowed, #1991) and
// `glob_import_method_calls.rs` (`use miniextendr_api::*`, #2020). Its own
// code spells every standard-library path out and calls trait methods with
// fully qualified syntax, so only the generated code can fail.

use miniextendr_api::altrep_data::{
    AltComplexData, AltIntegerData, AltListData, AltLogicalData, AltRawData, AltRealData,
    AltStringData, AltrepDataptr, AltrepExtractSubset, AltrepLen, AltrepSerialize, Logical,
};
use miniextendr_api::dots::Dots;
use miniextendr_api::either_impl::Either;
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};
use miniextendr_api::{
    Call, CallerCall, Checked, DataFrame, ExternalPtr, IntoR, LazyDots, List, MatchArg, Missing,
    NArgs, NonNa, Quoted, RCondition, RCow, Rcomplex, RngGuard, SEXP, StrVec, TryFromSexp, Unchecked,
    miniextendr, typed_list,
};

// region: standalone functions

#[miniextendr]
pub fn optional(x: i32) -> ::core::option::Option<i32> {
    ::core::option::Option::Some(x)
}

#[miniextendr]
pub fn fallible(x: i32) -> ::core::result::Result<i32, ::std::string::String> {
    ::core::result::Result::Ok(x)
}

#[miniextendr]
pub fn shapes(
    text: &str,
    values: &[f64],
    owned: ::std::string::String,
    list: ::std::vec::Vec<i32>,
    maybe: ::core::option::Option<f64>,
    missing: Missing<i32>,
) -> ::std::vec::Vec<f64> {
    let _ = (text, owned, list, maybe, missing.is_missing());
    values.to_vec()
}

#[miniextendr]
pub fn rooted_views<'a>(strings: StrVec<'_>, values: RCow<'a, i32>) -> RCow<'a, i32> {
    let _ = strings.len();
    values
}

#[miniextendr]
pub fn rooted_references(
    values: &[i32],
    scale: &f64,
    label: ::core::option::Option<&str>,
    text: ::std::borrow::Cow<'_, str>,
    words: ::std::vec::Vec<::core::option::Option<::std::borrow::Cow<'_, str>>>,
) -> i32 {
    let _ = (values, scale, label, text, words);
    0
}

#[miniextendr(strict, invisible)]
pub fn strict_unit(x: i64) {
    let _ = x;
}

#[miniextendr(strict)]
pub fn strict_array(x: u64) -> ::core::option::Option<[u64; 2]> {
    ::core::option::Option::Some([x, x])
}

#[miniextendr(strict)]
pub fn strict_rows(
    x: i64,
) -> ::std::vec::Vec<::core::option::Option<::std::boxed::Box<[i64]>>> {
    let mut rows = ::std::vec::Vec::new();
    rows.push(::core::option::Option::None);
    rows.push(::core::option::Option::Some(::std::boxed::Box::from([x])));
    rows
}

#[miniextendr(coerce, check_interrupt, rng)]
pub fn coerced(x: u16, xs: ::std::vec::Vec<u8>) -> f64 {
    f64::from(x) + xs.len() as f64
}

#[miniextendr(rng)]
pub fn drawn(n: i32, rng: &RngGuard, scale: f64) -> f64 {
    f64::from(n) * scale * rng.unif_rand()
}

#[miniextendr(rng(seed = seed), s3(generic = "stats::simulate", class = "shadow_fit"))]
pub fn simulate_shadow_fit(
    _object: SEXP,
    #[miniextendr(default = "1")] nsim: usize,
    #[miniextendr(default = "NULL")] seed: ::core::option::Option<i64>,
    _dots: &Dots,
    rng: &RngGuard,
) -> ::std::vec::Vec<f64> {
    let _ = seed;
    (0..nsim).map(|_| rng.norm_rand()).collect()
}

#[miniextendr(unwrap_in_r)]
pub fn unwrapped(x: i32) -> ::core::result::Result<i32, ::std::string::String> {
    ::core::result::Result::Ok(x)
}

#[miniextendr]
pub fn checks(
    #[miniextendr(no_na)] factor: f64,
    #[miniextendr(inherits("data.frame"))] frame: SEXP,
    #[miniextendr(default = "1L")] n: i32,
    marked: Checked<i32>,
    loose: Unchecked<::std::vec::Vec<f64>>,
    present: NonNa<::std::vec::Vec<f64>>,
) -> f64 {
    let _ = frame;
    factor * f64::from(n + *marked) + loose.len() as f64 + present.len() as f64
}

#[miniextendr]
pub fn either(x: Either<i32, ::std::string::String>) -> bool {
    x.is_left()
}

#[miniextendr]
pub fn returns_impl(x: i32) -> impl IntoR {
    x
}

#[miniextendr]
pub fn markers(x: i32, call: Call, rest: LazyDots, n: NArgs) -> i32 {
    let _ = (call.sexp(), rest.len());
    x + i32::try_from(n.get()).unwrap_or(0)
}

#[miniextendr(noexport, call = caller)]
pub fn caller_marked(x: i32, _call: CallerCall) -> i32 {
    x
}

#[miniextendr]
pub fn quoted(expr: Quoted) -> SEXP {
    expr.eval()
}

#[miniextendr(dots = typed_list!(alpha => numeric(2), beta? => character()))]
pub fn typed_dots(_dots: &Dots) -> i32 {
    let _ = dots_typed;
    1
}

#[miniextendr]
pub fn untyped_dots(n: i32, _dots: ...) -> i32 {
    n
}

#[miniextendr(s3(generic = "format", class = "shadow_fmt"))]
pub fn shadow_fmt_format(x: SEXP, _dots: ...) -> SEXP {
    x
}

#[miniextendr]
pub fn list_shapes(x: List, frame: DataFrame) -> List {
    let _ = frame;
    x
}

// R re-exports, written by the wrapper writer.
miniextendr_api::reexport!(generics::tidy, stats::na.omit, magrittr::"%>%");

// endregion

// region: match_arg, choices and factors

#[derive(::core::clone::Clone, ::core::marker::Copy, Debug, MatchArg)]
pub enum Mode {
    Fast,
    #[match_arg(alias = "careful")]
    Slow,
}

#[miniextendr]
pub fn pick(
    #[miniextendr(match_arg)] mode: Mode,
    #[miniextendr(match_arg)] maybe: ::core::option::Option<Mode>,
    #[miniextendr(match_arg, several_ok)] many: ::std::vec::Vec<Mode>,
    #[miniextendr(choices("a", "b"))] letter: &str,
) -> usize {
    let _ = (mode, letter);
    usize::from(maybe.is_some()) + many.len()
}

#[derive(::core::clone::Clone, ::core::marker::Copy, Debug, miniextendr_api::RFactor)]
pub enum Level {
    Low,
    High,
}

#[derive(::core::clone::Clone, ::core::marker::Copy, Debug, miniextendr_api::RFactor)]
#[r_factor(interaction = ["Low", "High"])]
pub enum Staged {
    Before(Level),
    After(Level),
}

#[miniextendr]
pub fn levels(level: Level, staged: Staged) -> Staged {
    let _ = level;
    staged
}

// endregion

// region: conditions

#[derive(Debug, RCondition)]
#[condition(class = "shadow_error", message = "bad row {row}")]
pub struct RowError {
    row: i32,
    #[condition(skip_none)]
    hint: ::core::option::Option<::std::string::String>,
}

#[derive(Debug, RCondition)]
#[condition(class = "shadow_family")]
pub enum Family {
    #[condition(call = none, message = "missing {what}", argument_message = "need {what}")]
    Missing { what: ::std::string::String },
    #[condition(kind = warning, message = "late by {delay}")]
    Late {
        #[condition(rename = "by")]
        delay: i32,
    },
    #[condition(message = "plain")]
    Plain,
}

#[miniextendr]
pub fn raises(x: i32) -> ::core::result::Result<i32, RowError> {
    if x < 0 {
        return ::core::result::Result::Err(RowError {
            row: x,
            hint: ::core::option::Option::None,
        });
    }
    ::core::result::Result::Ok(x)
}

#[miniextendr]
pub fn condition_macros(x: i32) -> i32 {
    miniextendr_api::defer_warning!("deferred {x}");
    miniextendr_api::defer_message!("deferred {x}");
    miniextendr_api::defer_condition!(class = "shadow_deferred", "deferred {x}");
    miniextendr_api::r_print!("{x}");
    miniextendr_api::r_println!("{x}");
    match x {
        0 => miniextendr_api::warning!(class = "shadow_warn", data = ("x", x), "x is {x}"),
        1 => miniextendr_api::message!("x is {}", x),
        2 => miniextendr_api::condition!(class = "shadow_signal", "signal {x}"),
        3 => miniextendr_api::arg_error!(param = "x", "'x' is {x}"),
        4 => miniextendr_api::error!(class = "shadow_zero", "zero"),
        5 => miniextendr_api::rust_error!("one"),
        6 => miniextendr_api::rust_condition!(class = "shadow_two", "two"),
        _ => x,
    }
}

// endregion

// region: ExternalPtr and the class systems

#[derive(::core::clone::Clone, ExternalPtr)]
pub struct Counter {
    #[r_data]
    pub value: i32,
    #[r_data]
    pub label: ::std::string::String,
}

#[miniextendr(env)]
impl Counter {
    pub fn new(value: i32) -> Self {
        Counter { value, label: ::std::string::String::new() }
    }
    pub fn read(&self) -> ::core::option::Option<i32> {
        ::core::option::Option::Some(self.value)
    }
    pub fn write(&mut self, value: i32) -> ::core::result::Result<(), ::std::string::String> {
        self.value = value;
        ::core::result::Result::Ok(())
    }
    pub fn chain(&mut self, by: i32) -> &mut Self {
        self.value += by;
        self
    }
    pub fn maybe_chain(&mut self, by: i32) -> ::core::option::Option<&mut Self> {
        self.value += by;
        ::core::option::Option::Some(self)
    }
    pub fn references(
        &self,
        values: &[i32],
        scale: &f64,
        words: ::std::vec::Vec<::core::option::Option<::std::borrow::Cow<'_, str>>>,
    ) -> i32 {
        let _ = (values, scale, words);
        self.value
    }
    pub fn finish(self) -> i32 {
        self.value
    }
    pub fn rebuilt(self) -> ::core::result::Result<Self, ::std::string::String> {
        ::core::result::Result::Ok(self)
    }
    pub fn absorb(&mut self, other: &Counter) {
        self.value += other.value;
    }
    pub fn origin() -> Self {
        Counter::new(0)
    }
    #[miniextendr(rng)]
    pub fn jitter(&self, rng: &RngGuard) -> f64 {
        f64::from(self.value) + rng.norm_rand()
    }
    #[miniextendr(rng)]
    pub fn drawn_index(rng: &RngGuard, n: f64) -> f64 {
        rng.unif_index(n)
    }
}

#[miniextendr]
pub trait ReadValue {
    const LIMIT: i32;
    fn read_value(&self) -> i32;
    fn bump(&mut self, by: i32);
    fn merge(&mut self, other: &Self) -> i32;
    fn made() -> i32;
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32;
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32;
}

#[miniextendr(env)]
impl ReadValue for Counter {
    const LIMIT: i32 = 10;
    fn read_value(&self) -> i32 {
        self.value
    }
    fn bump(&mut self, by: i32) {
        self.value += by;
    }
    fn merge(&mut self, other: &Self) -> i32 {
        self.value + other.value
    }
    fn made() -> i32 {
        0
    }
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32 {
        let _ = strings.len();
        values[0]
    }
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32 {
        let _ = (values, label, text);
        0
    }
}

#[miniextendr]
pub trait Convert<T> {
    fn convert(&self, value: T) -> ::std::vec::Vec<T>;
}

#[miniextendr(env)]
impl Convert<f64> for Counter {
    fn convert(&self, value: f64) -> ::std::vec::Vec<f64> {
        let mut out = ::std::vec::Vec::new();
        out.push(value);
        out
    }
}

#[derive(ExternalPtr)]
#[externalptr(
    s3,
    restored(
        class = ["shadow_saved"],
        message = "saved",
        no_handle = "no pointer",
        refuse = "every_read"
    ),
    field_error(class = "shadow_field")
)]
pub struct Engine {
    n: i32,
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::doubled")]
    pub doubled: Computed,
    #[r_data(ref, mut)]
    pub keys: Sidecar<::std::vec::Vec<i32>>,
    #[r_data(name = "n.base", setter = "visible")]
    pub base: i32,
    #[r_data]
    pub raw: Sidecar<SEXP>,
}

impl Engine {
    fn doubled(&self) -> i32 {
        self.n * 2
    }
}

#[miniextendr(s3(r_data_accessors))]
impl Engine {
    pub fn new(n: i32) -> Self {
        Engine {
            n,
            _r: RSidecar,
            doubled: Computed,
            keys: Sidecar::new(::std::vec::Vec::new()),
            base: n,
            raw: Sidecar::new(SEXP::nil()),
        }
    }
    pub fn size(&self) -> i32 {
        self.n
    }
    #[miniextendr(s3(generic = "length"))]
    pub fn len(&self) -> i32 {
        self.n
    }
}

#[miniextendr(s3)]
impl ReadValue for Engine {
    const LIMIT: i32 = 1;
    fn read_value(&self) -> i32 {
        self.n
    }
    fn bump(&mut self, by: i32) {
        self.n += by;
    }
    fn merge(&mut self, other: &Self) -> i32 {
        self.n + other.n
    }
    fn made() -> i32 {
        1
    }
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32 {
        let _ = strings.len();
        values[0]
    }
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32 {
        let _ = (values, label, text);
        0
    }
}

#[derive(ExternalPtr)]
#[externalptr(r6)]
pub struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub reading: f64,
    celsius: f64,
}

#[miniextendr(r6(r_data_accessors))]
impl Gauge {
    pub fn new(reading: f64) -> Self {
        Gauge { _r: RSidecar, reading, celsius: 0.0 }
    }
    #[miniextendr(r6(active))]
    pub fn celsius(&self) -> f64 {
        self.celsius
    }
    #[miniextendr(r6(setter, prop = "celsius"))]
    pub fn set_celsius(&mut self, value: f64) {
        self.celsius = value;
    }
    pub fn scaled(&self, by: f64) -> ::core::result::Result<f64, ::std::string::String> {
        ::core::result::Result::Ok(self.reading * by)
    }
}

#[miniextendr(r6)]
impl ReadValue for Gauge {
    const LIMIT: i32 = 2;
    fn read_value(&self) -> i32 {
        self.reading as i32
    }
    fn bump(&mut self, by: i32) {
        self.reading += f64::from(by);
    }
    fn merge(&mut self, other: &Self) -> i32 {
        (self.reading + other.reading) as i32
    }
    fn made() -> i32 {
        2
    }
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32 {
        let _ = strings.len();
        values[0]
    }
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32 {
        let _ = (values, label, text);
        0
    }
}

#[derive(ExternalPtr)]
pub struct Shape4 {
    side: f64,
}

#[miniextendr(s4)]
impl Shape4 {
    pub fn new(side: f64) -> Self {
        Shape4 { side }
    }
    pub fn area(&self) -> f64 {
        self.side * self.side
    }
    pub fn grow(&mut self, by: f64) -> ::core::option::Option<f64> {
        self.side += by;
        ::core::option::Option::Some(self.side)
    }
}

#[miniextendr(s4)]
impl ReadValue for Shape4 {
    const LIMIT: i32 = 4;
    fn read_value(&self) -> i32 {
        self.side as i32
    }
    fn bump(&mut self, by: i32) {
        self.side += f64::from(by);
    }
    fn merge(&mut self, other: &Self) -> i32 {
        (self.side + other.side) as i32
    }
    fn made() -> i32 {
        4
    }
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32 {
        let _ = strings.len();
        values[0]
    }
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32 {
        let _ = (values, label, text);
        0
    }
}

#[derive(ExternalPtr)]
#[externalptr(s7)]
pub struct Range7 {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub label: ::std::string::String,
    start: f64,
    end: f64,
}

#[miniextendr(s7(r_data_accessors))]
impl Range7 {
    pub fn new(start: f64, end: f64) -> Self {
        Range7 { _r: RSidecar, label: ::std::string::String::new(), start, end }
    }
    #[miniextendr(s7(getter))]
    pub fn length(&self) -> f64 {
        self.end - self.start
    }
    #[miniextendr(s7(getter, prop = "midpoint"))]
    pub fn get_midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }
    #[miniextendr(s7(setter, prop = "midpoint"))]
    pub fn set_midpoint(&mut self, value: f64) {
        let half = (self.end - self.start) / 2.0;
        self.start = value - half;
        self.end = value + half;
    }
    pub fn shifted(&self, by: f64) -> ::std::vec::Vec<f64> {
        let mut out = ::std::vec::Vec::new();
        out.push(self.start + by);
        out.push(self.end + by);
        out
    }
}

#[miniextendr(s7)]
impl ReadValue for Range7 {
    const LIMIT: i32 = 7;
    fn read_value(&self) -> i32 {
        self.start as i32
    }
    fn bump(&mut self, by: i32) {
        self.start += f64::from(by);
    }
    fn merge(&mut self, other: &Self) -> i32 {
        (self.start + other.start) as i32
    }
    fn made() -> i32 {
        7
    }
    fn read_views(&self, strings: StrVec<'_>, values: RCow<'_, i32>) -> i32 {
        let _ = strings.len();
        values[0]
    }
    fn read_refs(
        &self,
        values: &[i32],
        label: ::core::option::Option<&str>,
        text: ::std::borrow::Cow<'_, str>,
    ) -> i32 {
        let _ = (values, label, text);
        0
    }
}

#[derive(ExternalPtr)]
pub struct Scale {
    by: f64,
}

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "sc"))]
impl Scale {
    pub fn new(mode: Mode, n: i32) -> ::std::vec::Vec<f64> {
        let _ = mode;
        let mut out = ::std::vec::Vec::new();
        out.push(f64::from(n));
        out
    }
}

// The `TypedExternal` impl for a type the derive can't reach.
pub struct Opaque(i32);
miniextendr_api::impl_typed_external!(Opaque);

// endregion

// region: lists, data frames and newtypes

#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
pub struct Record {
    pub value: i32,
    pub name: ::std::string::String,
    pub tags: ::std::vec::Vec<::std::string::String>,
}

#[derive(::core::clone::Clone, ::core::marker::Copy, Debug, miniextendr_api::DataFrameRow)]
#[dataframe(tag = "variant")]
pub enum Direction {
    North,
    South,
}

#[derive(::core::clone::Clone, Debug, miniextendr_api::DataFrameRow)]
#[dataframe(align, tag = "variant")]
pub enum Status {
    Fine,
    Failed { code: i32 },
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(tag = "_kind")]
pub struct Row {
    pub id: i32,
    #[dataframe(rename = "label")]
    pub name: ::std::string::String,
    #[dataframe(skip)]
    pub cache: i32,
    pub coords: [f64; 2],
    #[dataframe(width = 2)]
    pub scores: ::std::vec::Vec<f64>,
    #[dataframe(expand)]
    pub extra: ::std::vec::Vec<i32>,
    #[dataframe(as_list)]
    pub listed: ::std::vec::Vec<i32>,
    #[dataframe(as_factor)]
    pub heading: Direction,
    pub counts: ::std::collections::HashMap<::std::string::String, i32>,
    pub maybe: ::core::option::Option<f64>,
}

#[derive(::core::clone::Clone, miniextendr_api::DataFrameRow)]
pub struct Inner {
    pub a: i32,
    pub b: f64,
}

#[derive(miniextendr_api::DataFrameRow)]
pub struct Outer {
    pub id: i32,
    pub inner: Inner,
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum Event {
    Value {
        value: i32,
        #[dataframe(rename = "what")]
        label: ::std::string::String,
    },
    Scores {
        #[dataframe(width = 2)]
        scores: ::std::vec::Vec<f64>,
        #[dataframe(expand)]
        extra: ::std::vec::Vec<i32>,
        coords: [f64; 2],
    },
    Listed {
        #[dataframe(as_list)]
        listed: ::std::vec::Vec<i32>,
        #[dataframe(as_factor)]
        heading: Direction,
        status: Status,
        counts: ::std::collections::BTreeMap<::std::string::String, i32>,
        #[dataframe(skip)]
        cache: i32,
    },
    Empty,
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(conflicts = "string")]
pub enum Mixed {
    Number { x: f64 },
    Text { x: ::std::string::String },
}

#[derive(miniextendr_api::DataFrameRow)]
pub enum Nested {
    Has { id: i32, inner: Inner },
    Without { id: i32 },
}

// Rows with `try_from_dataframe` readers: every field reads back, including
// qualified strings (#2036). Without a nested struct the `rayon` reader
// assembles rows by index; with one it hands off to the sequential reader.
#[derive(miniextendr_api::DataFrameRow)]
pub struct Readable {
    pub id: i32,
    pub label: ::std::string::String,
    pub note: ::core::option::Option<::std::string::String>,
    pub weight: f64,
    pub flag: bool,
    pub maybe: ::core::option::Option<f64>,
    pub coords: [f64; 2],
    #[dataframe(expand)]
    pub corners: [i32; 2],
    #[dataframe(width = 2)]
    pub scores: ::std::vec::Vec<f64>,
    #[dataframe(expand)]
    pub extra: ::std::vec::Vec<i32>,
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum ReadableEvent {
    Point { id: i32, coords: [f64; 2], flag: bool, label: ::std::string::String },
    Scores {
        #[dataframe(width = 2)]
        scores: ::std::vec::Vec<f64>,
        #[dataframe(expand)]
        extra: ::std::vec::Vec<i32>,
    },
    Empty,
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum TaggedNested {
    Has { id: i32, inner: Inner },
    Without { id: i32 },
}

#[miniextendr]
pub fn read_rows(frame: SEXP) -> i32 {
    let rows = Readable::try_from_dataframe(frame).map_or(0, |r| r.len());
    let events = ReadableEvent::try_from_dataframe(frame).map_or(0, |r| r.len());
    let nested = TaggedNested::try_from_dataframe(frame).map_or(0, |r| r.len());
    let outer = Outer::try_from_dataframe(frame).map_or(0, |r| r.len());
    i32::try_from(rows + events + nested + outer).unwrap_or(i32::MAX)
}

miniextendr_api::typed_dataframe! {
    pub ShadowFrame {
        id: i32,
        weight: f64,
        flag: ::core::option::Option<i32>,
    }
}

#[miniextendr]
pub fn typed_frame(df: ShadowFrame) -> i32 {
    let _ = df;
    1
}

#[derive(TryFromSexp, IntoR)]
pub struct Meters(f64);

#[derive(TryFromSexp)]
#[try_from_sexp(validate = not_null)]
pub struct Positive(f64);

fn not_null(x: SEXP) -> ::core::result::Result<(), miniextendr_api::from_r::SexpError> {
    let _ = x;
    ::core::result::Result::Ok(())
}

#[miniextendr]
pub fn newtypes(m: Meters, p: Positive) -> Meters {
    Meters(m.0 + p.0)
}

#[repr(transparent)]
#[derive(::core::clone::Clone, ::core::marker::Copy, miniextendr_api::RNativeType, miniextendr_api::IsNA)]
pub struct Native(pub i32);

#[miniextendr(prefer = "native")]
#[repr(transparent)]
#[derive(::core::clone::Clone, ::core::marker::Copy, miniextendr_api::RNativeType)]
pub struct PreferredNative(pub f64);

#[derive(miniextendr_api::IntoList, miniextendr_api::PreferList)]
pub struct PreferredList {
    pub value: i32,
}

#[repr(transparent)]
#[derive(
    ::core::clone::Clone,
    ::core::marker::Copy,
    miniextendr_api::RNativeType,
    miniextendr_api::PreferRNativeType
)]
pub struct PreferredRNative(pub i32);

#[miniextendr(prefer = "externalptr")]
pub struct PreferredPtr {
    value: i32,
    other: i32,
}

#[miniextendr(list)]
pub struct ListMode {
    pub value: i32,
    pub name: ::std::string::String,
}

#[miniextendr(dataframe)]
pub struct FrameMode {
    pub id: i32,
    pub score: f64,
}

#[miniextendr]
pub struct PointerMode {
    x: f64,
    y: f64,
}

#[miniextendr]
#[derive(::core::clone::Clone, ::core::marker::Copy, Debug)]
pub enum FactorMode {
    Spring,
    Summer,
}

#[miniextendr(match_arg)]
#[derive(::core::clone::Clone, ::core::marker::Copy, Debug)]
pub enum ChoiceMode {
    Up,
    Down,
}

#[miniextendr]
pub fn preferred(a: f64, season: FactorMode, way: ChoiceMode) -> PreferredList {
    let _ = (season, way);
    PreferredList { value: a as i32 }
}

#[miniextendr]
pub fn frame_mode() -> FrameModeDataFrame {
    FrameMode::to_dataframe(::std::vec::Vec::new())
}

fn function_like() -> SEXP {
    let _ = miniextendr_api::list!(a = 1i32, b = "x");
    let _ = miniextendr_api::r_str!("1L + 1L");
    let _ = miniextendr_api::r!(1L + 2L);
    SEXP::nil()
}

// endregion

// region: ALTREP

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value")]
pub struct Repeated {
    len: usize,
    value: i32,
}

// `elt_delegate` forwards `elt` to the field's own `AltIntegerData` impl.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt_delegate = "data")]
pub struct Delegated {
    data: ::std::vec::Vec<i32>,
    len: usize,
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(len = "len", elt = "value", class = "ShadowReal")]
pub struct RepeatedReal {
    len: usize,
    value: f64,
}

#[derive(miniextendr_api::AltrepLogical)]
#[altrep(len = "len", elt = "value")]
pub struct RepeatedLogical {
    len: usize,
    value: Logical,
}

#[derive(miniextendr_api::AltrepRaw)]
#[altrep(len = "len", elt = "value")]
pub struct RepeatedRaw {
    len: usize,
    value: u8,
}

#[derive(miniextendr_api::AltrepString)]
#[altrep(len = "len", elt = "value")]
pub struct RepeatedString {
    len: usize,
    value: ::std::string::String,
}

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(len = "len", elt = "value")]
pub struct RepeatedComplex {
    len: usize,
    value: Rcomplex,
}

#[derive(::core::clone::Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, dataptr, serialize, duplicate = "clone")]
pub struct Ints {
    data: ::std::vec::Vec<i32>,
    #[r_data(ref, mut)]
    pub source: Sidecar<SEXP>,
}

impl AltrepLen for Ints {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltIntegerData for Ints {
    fn elt(&self, i: usize) -> i32 {
        self.data[i]
    }
}

impl AltrepDataptr<i32> for Ints {
    fn dataptr(&mut self, _writable: bool) -> ::core::option::Option<*mut i32> {
        ::core::option::Option::Some(self.data.as_mut_ptr())
    }
}

impl AltrepSerialize for Ints {
    fn serialized_state(&self) -> SEXP {
        SEXP::nil()
    }
    fn unserialize(_state: SEXP) -> ::core::option::Option<Self> {
        ::core::option::Option::None
    }
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(manual, subset, rust_unwind)]
pub struct Reals {
    data: ::std::vec::Vec<f64>,
}

impl AltrepLen for Reals {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltRealData for Reals {
    fn elt(&self, i: usize) -> f64 {
        self.data[i]
    }
}

impl AltrepExtractSubset for Reals {
    fn extract_subset(&self, _indices: &[i32]) -> ::core::option::Option<SEXP> {
        ::core::option::Option::None
    }
}

#[derive(miniextendr_api::AltrepString)]
#[altrep(manual, dataptr, serialize)]
pub struct Strings {
    data: ::std::vec::Vec<::core::option::Option<::std::string::String>>,
}

impl AltrepLen for Strings {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltStringData for Strings {
    fn elt(&self, i: usize) -> ::core::option::Option<&str> {
        self.data[i].as_deref()
    }
}

impl AltrepSerialize for Strings {
    fn serialized_state(&self) -> SEXP {
        SEXP::nil()
    }
    fn unserialize(_state: SEXP) -> ::core::option::Option<Self> {
        ::core::option::Option::None
    }
}

#[derive(miniextendr_api::AltrepLogical)]
#[altrep(manual, serialize)]
pub struct Logicals {
    data: ::std::vec::Vec<Logical>,
}

impl AltrepLen for Logicals {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltLogicalData for Logicals {
    fn elt(&self, i: usize) -> Logical {
        self.data[i]
    }
}

impl AltrepSerialize for Logicals {
    fn serialized_state(&self) -> SEXP {
        SEXP::nil()
    }
    fn unserialize(_state: SEXP) -> ::core::option::Option<Self> {
        ::core::option::Option::None
    }
}

#[derive(::core::clone::Clone, miniextendr_api::AltrepRaw)]
#[altrep(manual, duplicate = "clone")]
pub struct Raws {
    data: ::std::vec::Vec<u8>,
}

impl AltrepLen for Raws {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltRawData for Raws {
    fn elt(&self, i: usize) -> u8 {
        self.data[i]
    }
}

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(manual)]
pub struct Complexes {
    data: ::std::vec::Vec<Rcomplex>,
}

impl AltrepLen for Complexes {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltComplexData for Complexes {
    fn elt(&self, i: usize) -> Rcomplex {
        self.data[i]
    }
}

#[derive(miniextendr_api::AltrepList)]
#[altrep(manual)]
pub struct Lists;

impl AltrepLen for Lists {
    fn altrep_len(&self) -> usize {
        0
    }
}

impl AltListData for Lists {
    fn elt(&self, _: usize) -> SEXP {
        SEXP::nil()
    }
}

#[derive(miniextendr_api::AltrepList)]
#[altrep(manual, serialize)]
pub struct SavedLists;

impl AltrepLen for SavedLists {
    fn altrep_len(&self) -> usize {
        0
    }
}

impl AltListData for SavedLists {
    fn elt(&self, _: usize) -> SEXP {
        SEXP::nil()
    }
}

impl AltrepSerialize for SavedLists {
    fn serialized_state(&self) -> SEXP {
        SEXP::nil()
    }
    fn unserialize(_state: SEXP) -> ::core::option::Option<Self> {
        ::core::option::Option::None
    }
}

// `derive(Altrep)` registers the class; the `impl_alt*_from_data!` macros
// supply the low-level traits.
#[derive(miniextendr_api::Altrep)]
pub struct Registered {
    data: ::std::vec::Vec<i32>,
}

impl AltrepLen for Registered {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltIntegerData for Registered {
    fn elt(&self, i: usize) -> i32 {
        self.data[i]
    }
}

impl AltrepSerialize for Registered {
    fn serialized_state(&self) -> SEXP {
        SEXP::nil()
    }
    fn unserialize(_state: SEXP) -> ::core::option::Option<Self> {
        ::core::option::Option::None
    }
}

miniextendr_api::impl_altinteger_from_data!(Registered, serialize);

#[derive(miniextendr_api::Altrep)]
#[altrep(class = "ShadowRegisteredStrings")]
pub struct RegisteredStrings {
    data: ::std::vec::Vec<::std::string::String>,
}

impl AltrepLen for RegisteredStrings {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltStringData for RegisteredStrings {
    fn elt(&self, i: usize) -> ::core::option::Option<&str> {
        ::core::option::Option::Some(self.data[i].as_str())
    }
}

miniextendr_api::impl_altstring_from_data!(RegisteredStrings);

#[miniextendr]
pub fn altrep_values(n: i32) -> Repeated {
    Repeated { len: usize::try_from(n).unwrap_or(0), value: n }
}

// endregion

#[miniextendr_api::r_ffi_checked]
unsafe extern "C-unwind" {
    pub fn Rf_xlength(x: SEXP) -> miniextendr_api::R_xlen_t;
}
