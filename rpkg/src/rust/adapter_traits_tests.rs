//! Tests for adapter traits (RDebug, RDisplay, RHash, ROrd, etc.)
//!
//! All adapter traits use the trait ABI pattern:
//!   `#[miniextendr] impl Trait for Type { ... }`
//!
//! For blanket-impl traits (ROrd, RPartialOrd), use `#[miniextendr(blanket)]`
//! to suppress the impl block (blanket provides it) while still generating
//! C wrappers and R wrappers from the method signatures.
//!
//! For non-blanket traits (RIterator, RExtend, etc.), the impl block IS emitted
//! and contains the actual implementation.
//!
//! Adapter methods named like a std method are `r_`-prefixed in Rust and keep
//! the std name in R (#2020). Every impl, empty or with a body, takes the R
//! names from the trait (#2062), so the impls below carry no `r_name`.

// The impls here are env pinned, except the class-system cells at the end,
// which pin their own system: several adapter traits have multiple impls in
// this file, and env trait attachment requires env inherent impls under
// s7-default. Explicit class systems keep this file meaning what its tests
// assert, independent of *-default features.

use miniextendr_api::adapter_traits::RIterator;
use miniextendr_api::{ExternalPtr, miniextendr};
use std::cell::RefCell;
use std::fmt;
use std::str::FromStr;

// region: Test type: Point (Debug, Display, Hash, PartialOrd, Ord, Clone, Default, FromStr)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, ExternalPtr)]
pub struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl FromStr for Point {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if !s.starts_with('(') || !s.ends_with(')') {
            return Err("Expected format: (x, y)".to_string());
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() != 2 {
            return Err("Expected two comma-separated values".to_string());
        }
        let x: i32 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid x value: {}", parts[0]))?;
        let y: i32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid y value: {}", parts[1]))?;
        Ok(Point { x, y })
    }
}

/// Point inherent methods: constructor and field accessors.
#[miniextendr(env)]
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }
}

// Trait ABI: adapter traits registered via empty impl + TPIE (Trait-Provided Impl Expansion).
// The trait definition exports method metadata; empty impls auto-expand C/R wrappers.

/// RDebug trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RDebug for Point {}

/// RDisplay trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RDisplay for Point {}

/// RHash trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RHash for Point {}

/// RClone trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RClone for Point {}

/// RDefault trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RDefault for Point {}

/// RFromStr trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RFromStr for Point {}

/// RCopy trait ABI registration for Point.
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RCopy for Point {}

/// ROrd trait ABI registration for Point (blanket-impl, total ordering).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::ROrd for Point {}
// endregion

// region: Test type: MyFloat (Demonstrates RPartialOrd (partial ordering with NaN))

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, ExternalPtr)]
pub struct MyFloat(f64);

/// MyFloat inherent methods: constructor and accessor.
#[miniextendr(env)]
impl MyFloat {
    fn new(value: f64) -> Self {
        MyFloat(value)
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn nan() -> Self {
        MyFloat(f64::NAN)
    }
}

/// RPartialOrd trait ABI registration for MyFloat (partial ordering with NaN).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RPartialOrd for MyFloat {}
// endregion

// region: Test type: ChainedError (Demonstrates RError (error chain walking))

#[derive(Debug)]
struct InnerError {
    msg: String,
}

impl fmt::Display for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for InnerError {}

#[derive(Debug)]
struct OuterError {
    msg: String,
    source: Option<InnerError>,
}

impl fmt::Display for OuterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for OuterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// Wrapper to expose OuterError to R.
///
/// Implements Error by delegating to the inner OuterError, so the
/// RError blanket impl applies and we can use trait ABI registration.
#[derive(Debug, ExternalPtr)]
pub struct ChainedError(OuterError);

impl fmt::Display for ChainedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ChainedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

/// ChainedError inherent methods: constructors with and without source errors.
#[miniextendr(env)]
impl ChainedError {
    /// @param outer_msg Character string for the outer error message.
    /// @param inner_msg Character string for the inner (source) error message.
    fn new(outer_msg: &str, inner_msg: &str) -> Self {
        ChainedError(OuterError {
            msg: outer_msg.to_string(),
            source: Some(InnerError {
                msg: inner_msg.to_string(),
            }),
        })
    }

    fn without_source(msg: &str) -> Self {
        ChainedError(OuterError {
            msg: msg.to_string(),
            source: None,
        })
    }
}

/// RError trait ABI registration for ChainedError (error chain walking).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RError for ChainedError {}
// endregion

// region: Test type: IntVecIter

#[derive(ExternalPtr)]
pub struct IntVecIter(RefCell<std::vec::IntoIter<i32>>);

/// RIterator trait implementation for IntVecIter (sequential integer iteration).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RIterator for IntVecIter {
    type Item = i32;

    fn next(&self) -> Option<Self::Item> {
        self.0.borrow_mut().next()
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }

    fn count(&self) -> i64 {
        let mut count = 0i64;
        while self.next().is_some() {
            count += 1;
        }
        count
    }

    fn collect_n(&self, n: i32) -> Vec<Self::Item> {
        let mut result = Vec::with_capacity(n.max(0) as usize);
        for _ in 0..n {
            match self.next() {
                Some(item) => result.push(item),
                None => break,
            }
        }
        result
    }

    fn skip(&self, n: i32) -> i32 {
        let mut skipped = 0i32;
        for _ in 0..n {
            if self.next().is_none() {
                break;
            }
            skipped += 1;
        }
        skipped
    }

    fn nth(&self, n: i32) -> Option<Self::Item> {
        if n < 0 {
            return None;
        }
        for _ in 0..n {
            self.next()?;
        }
        self.next()
    }
}

/// IntVecIter inherent methods: constructor from integer vector.
#[miniextendr(env)]
impl IntVecIter {
    /// @param data Integer vector of elements to iterate over.
    fn new(data: Vec<i32>) -> Self {
        IntVecIter(RefCell::new(data.into_iter()))
    }
}
// endregion

// region: Test type: GrowableVec

#[derive(ExternalPtr)]
pub struct GrowableVec(RefCell<Vec<i32>>);

/// RExtend trait implementation for GrowableVec (append integers to a growable buffer).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RExtend<i32> for GrowableVec {
    fn extend_from_vec(&self, items: Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }

    fn r_len(&self) -> i64 {
        i64::try_from(self.0.borrow().len()).expect("length fits i64")
    }

    fn r_is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

/// GrowableVec inherent methods: constructors, conversion, and clearing.
#[miniextendr(env)]
impl GrowableVec {
    fn new() -> Self {
        GrowableVec(RefCell::new(Vec::new()))
    }

    /// @param data Integer vector of initial elements.
    fn from_vec(data: Vec<i32>) -> Self {
        GrowableVec(RefCell::new(data))
    }

    fn to_vec(&self) -> Vec<i32> {
        Clone::clone(&*self.0.borrow())
    }

    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}
// endregion

// region: Test type: IntSet (Demonstrates RFromIter and RToVec (collection creation and extraction))

use std::collections::HashSet;

#[derive(ExternalPtr)]
pub struct IntSet(HashSet<i32>);

// RFromIter: static factory method (IntSet doesn't impl FromIterator directly)
/// RFromIter trait implementation for IntSet (construct a set from integer vector).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RFromIter<i32> for IntSet {
    fn from_vec(items: Vec<i32>) -> Self {
        IntSet(items.into_iter().collect())
    }
}

// RToVec: sorted output for deterministic tests
/// RToVec trait implementation for IntSet (extract sorted integer vector).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RToVec<i32> for IntSet {
    fn r_to_vec(&self) -> Vec<i32> {
        let mut v: Vec<_> = self.0.iter().cloned().collect();
        v.sort();
        v
    }

    fn r_len(&self) -> i64 {
        i64::try_from(self.0.len()).expect("length fits i64")
    }
}

/// IntSet inherent methods: membership test.
#[miniextendr(env)]
impl IntSet {
    /// @param value Integer value to check for membership.
    fn contains(&self, value: i32) -> bool {
        self.0.contains(&value)
    }
}
// endregion

// region: Test type: IterableVec and IterableVecIter (Demonstrates RMakeIter (iterator factory))

#[derive(ExternalPtr)]
pub struct IterableVec(Vec<i32>);

#[derive(ExternalPtr)]
pub struct IterableVecIter(RefCell<std::vec::IntoIter<i32>>);

impl RIterator for IterableVecIter {
    type Item = i32;

    fn next(&self) -> Option<i32> {
        self.0.borrow_mut().next()
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

/// RMakeIter trait implementation for IterableVec (create an iterator over the vector).
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RMakeIter<i32, IterableVecIter> for IterableVec {
    fn make_iter(&self) -> IterableVecIter {
        IterableVecIter(RefCell::new(Clone::clone(&self.0).into_iter()))
    }
}

/// IterableVec inherent methods: constructor, length, and conversion.
#[miniextendr(env)]
impl IterableVec {
    fn new(data: Vec<i32>) -> Self {
        IterableVec(data)
    }

    fn len(&self) -> i64 {
        self.0.len() as i64
    }

    fn to_vec(&self) -> Vec<i32> {
        Clone::clone(&self.0)
    }
}

/// RIterator blanket trait ABI registration for IterableVecIter.
///
/// Only the required methods are spelled; the omitted defaults (`count`,
/// `collect_n`, `skip`, `nth`) take their wrappers from the declaration.
#[miniextendr(env, blanket)]
impl miniextendr_api::adapter_traits::RIterator for IterableVecIter {
    type Item = i32;

    fn next(&self) -> Option<Self::Item> {
        unreachable!()
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        unreachable!()
    }
}

/// IterableVecIter blanket impl: collect all remaining items.
#[miniextendr(env, blanket)]
impl IterableVecIter {
    fn collect_all(&self) -> Vec<i32> {
        let mut result = Vec::new();
        while let Some(item) = RIterator::next(self) {
            result.push(item);
        }
        result
    }
}
// endregion

// region: Test type: ExportControlTraitPoint (Demonstrates internal/noexport on trait impls)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, ExternalPtr)]
pub struct ExportControlTraitPoint {
    x: i32,
}

impl fmt::Display for ExportControlTraitPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.x)
    }
}

/// @keywords internal
#[miniextendr(env, internal)]
impl ExportControlTraitPoint {
    fn new(x: i32) -> Self {
        ExportControlTraitPoint { x }
    }
}

/// Internal trait impl: should have @keywords internal, no @export
#[miniextendr(env, internal)]
impl miniextendr_api::adapter_traits::RDebug for ExportControlTraitPoint {}

/// Noexport trait impl: no Rd contribution at all (no alias, no usage, no
/// @export) — distinct from `internal` above, which stays documented under
/// `@keywords internal`. See audit A10.
#[miniextendr(env, noexport)]
impl miniextendr_api::adapter_traits::RDisplay for ExportControlTraitPoint {}
// endregion

// region: Renamed adapter methods keep their R names in every class system (#2020)
//
// `RHash::r_hash`, `RClone::r_clone`, `RDefault::r_default` and
// `RFromStr::r_from_str` are `hash()`, `clone()`, `default()` and `from_str()`
// in R. `Point` above is the env cell; these are the other four, all through
// empty (TPIE) impls, which take the R names from the trait definition.

/// Parses a bare integer; shared by the class-system cells below.
fn parse_adapter_value(s: &str) -> Result<i32, String> {
    s.trim().parse().map_err(|_| format!("not an integer: {s}"))
}

#[derive(Clone, Debug, Default, Hash, ExternalPtr)]
pub struct AdapterNamesS3 {
    value: i32,
}

impl FromStr for AdapterNamesS3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_adapter_value(s).map(|value| Self { value })
    }
}

#[miniextendr(s3)]
impl AdapterNamesS3 {
    /// @param value Initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// RHash trait ABI registration for AdapterNamesS3.
#[miniextendr(s3)]
impl miniextendr_api::RHash for AdapterNamesS3 {}

/// RClone trait ABI registration for AdapterNamesS3.
#[miniextendr(s3)]
impl miniextendr_api::RClone for AdapterNamesS3 {}

/// RDefault trait ABI registration for AdapterNamesS3.
#[miniextendr(s3)]
impl miniextendr_api::RDefault for AdapterNamesS3 {}

/// RFromStr trait ABI registration for AdapterNamesS3.
#[miniextendr(s3)]
impl miniextendr_api::RFromStr for AdapterNamesS3 {}

#[derive(Clone, Debug, Default, Hash, ExternalPtr)]
pub struct AdapterNamesS4 {
    value: i32,
}

impl FromStr for AdapterNamesS4 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_adapter_value(s).map(|value| Self { value })
    }
}

#[miniextendr(s4, internal)]
impl AdapterNamesS4 {
    /// @param value Initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// RHash trait ABI registration for AdapterNamesS4.
#[miniextendr(s4, internal)]
impl miniextendr_api::adapter_traits::RHash for AdapterNamesS4 {}

/// RClone trait ABI registration for AdapterNamesS4.
#[miniextendr(s4, internal)]
impl miniextendr_api::adapter_traits::RClone for AdapterNamesS4 {}

/// RDefault trait ABI registration for AdapterNamesS4.
#[miniextendr(s4, internal)]
impl miniextendr_api::adapter_traits::RDefault for AdapterNamesS4 {}

/// RFromStr trait ABI registration for AdapterNamesS4.
#[miniextendr(s4, internal)]
impl miniextendr_api::adapter_traits::RFromStr for AdapterNamesS4 {}

#[derive(Clone, Debug, Default, Hash, ExternalPtr)]
pub struct AdapterNamesS7 {
    value: i32,
}

impl FromStr for AdapterNamesS7 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_adapter_value(s).map(|value| Self { value })
    }
}

#[miniextendr(s7, internal)]
impl AdapterNamesS7 {
    /// @param value Initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// RHash trait ABI registration for AdapterNamesS7.
#[miniextendr(s7, internal)]
impl miniextendr_api::adapter_traits::RHash for AdapterNamesS7 {}

/// RClone trait ABI registration for AdapterNamesS7.
#[miniextendr(s7, internal)]
impl miniextendr_api::adapter_traits::RClone for AdapterNamesS7 {}

/// RDefault trait ABI registration for AdapterNamesS7.
#[miniextendr(s7, internal)]
impl miniextendr_api::adapter_traits::RDefault for AdapterNamesS7 {}

/// RFromStr trait ABI registration for AdapterNamesS7.
#[miniextendr(s7, internal)]
impl miniextendr_api::adapter_traits::RFromStr for AdapterNamesS7 {}

#[derive(Clone, Debug, Default, Hash, ExternalPtr)]
pub struct AdapterNamesR6 {
    value: i32,
}

impl FromStr for AdapterNamesR6 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_adapter_value(s).map(|value| Self { value })
    }
}

#[miniextendr(r6)]
impl AdapterNamesR6 {
    /// Create a new AdapterNamesR6.
    /// @param value Initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// RHash trait ABI registration for AdapterNamesR6.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RHash for AdapterNamesR6 {}

/// RClone trait ABI registration for AdapterNamesR6.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RClone for AdapterNamesR6 {}

/// RDefault trait ABI registration for AdapterNamesR6.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RDefault for AdapterNamesR6 {}

/// RFromStr trait ABI registration for AdapterNamesR6.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RFromStr for AdapterNamesR6 {}
// endregion

// region: Module registration
// endregion
