//! Tests for adapter traits (RDebug, RDisplay, RHash, ROrd, etc.)
//!
//! Non-generic adapter traits use the trait ABI pattern:
//!   `#[miniextendr] impl RDebug for Point {}`
//! Generic/associated-type traits (RIterator, RExtend, etc.) and traits with
//! &Self parameters (ROrd, RPartialOrd) use manual forwarding.

use miniextendr_api::adapter_traits::{
    RExtend, RFromIter, RIterator, RMakeIter, ROrd, RPartialOrd, RToVec,
};
use miniextendr_api::{ExternalPtr, miniextendr, miniextendr_module};
use std::cell::RefCell;
use std::fmt;
use std::str::FromStr;

// =============================================================================
// Test type: Point
// Implements Debug, Display, Hash, PartialOrd, Ord, Clone, Default, FromStr
// =============================================================================

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

/// @noRd
#[miniextendr]
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

    // ROrd: manual forwarding (trait has &Self parameter, no #[miniextendr] on trait)
    fn cmp_to(&self, other: ExternalPtr<Point>) -> i32 {
        ROrd::cmp(self, &*other)
    }

    fn is_less_than(&self, other: ExternalPtr<Point>) -> bool {
        ROrd::cmp(self, &*other) < 0
    }

    fn is_equal_to(&self, other: ExternalPtr<Point>) -> bool {
        ROrd::cmp(self, &*other) == 0
    }

    fn is_greater_than(&self, other: ExternalPtr<Point>) -> bool {
        ROrd::cmp(self, &*other) > 0
    }
}

// Trait ABI: adapter traits registered via #[miniextendr] impl Trait for Type {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RDebug for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RDisplay for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RHash for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RClone for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RDefault for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RFromStr for Point {}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RCopy for Point {}

// =============================================================================
// Test type: MyFloat
// Demonstrates RPartialOrd (partial ordering with NaN)
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, ExternalPtr)]
pub struct MyFloat(f64);

/// @noRd
#[miniextendr]
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

    // RPartialOrd: manual forwarding (trait has &Self parameter)
    fn partial_cmp_to(&self, other: ExternalPtr<MyFloat>) -> Option<i32> {
        RPartialOrd::partial_cmp(self, &*other)
    }

    fn is_less_than(&self, other: ExternalPtr<MyFloat>) -> bool {
        RPartialOrd::partial_cmp(self, &*other) == Some(-1)
    }

    fn is_equal_to(&self, other: ExternalPtr<MyFloat>) -> bool {
        RPartialOrd::partial_cmp(self, &*other) == Some(0)
    }

    fn is_greater_than(&self, other: ExternalPtr<MyFloat>) -> bool {
        RPartialOrd::partial_cmp(self, &*other) == Some(1)
    }

    fn is_comparable(&self, other: ExternalPtr<MyFloat>) -> bool {
        RPartialOrd::partial_cmp(self, &*other).is_some()
    }
}

// =============================================================================
// Test type: ChainedError
// Demonstrates RError (error chain walking)
// Note: ChainedError wraps OuterError. Since OuterError (not ChainedError)
// implements std::error::Error, we use #[miniextendr] impl RError on a
// delegating newtype that implements Error.
// =============================================================================

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

/// @noRd
#[miniextendr]
impl ChainedError {
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

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RError for ChainedError {}

// =============================================================================
// Test type: IntVecIter
// Demonstrates RIterator (with interior mutability)
// Generic trait — uses manual forwarding
// =============================================================================

#[derive(ExternalPtr)]
pub struct IntVecIter(RefCell<std::vec::IntoIter<i32>>);

impl RIterator for IntVecIter {
    type Item = i32;

    fn next(&self) -> Option<Self::Item> {
        self.0.borrow_mut().next()
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

/// @noRd
#[miniextendr]
impl IntVecIter {
    fn new(data: Vec<i32>) -> Self {
        IntVecIter(RefCell::new(data.into_iter()))
    }

    // Renamed from `next` because `next` is a reserved word in R
    fn next_item(&self) -> Option<i32> {
        RIterator::next(self)
    }

    // Returns (lower_bound, upper_bound) where upper_bound is -1 if unknown
    fn size_hint(&self) -> Vec<i32> {
        let (lo, hi) = RIterator::size_hint(self);
        vec![lo as i32, hi.map(|h| h as i32).unwrap_or(-1)]
    }

    fn count(&self) -> i64 {
        RIterator::count(self)
    }

    fn collect_n(&self, n: i32) -> Vec<i32> {
        RIterator::collect_n(self, n)
    }

    fn skip(&self, n: i32) -> i32 {
        RIterator::skip(self, n)
    }

    fn nth(&self, n: i32) -> Option<i32> {
        RIterator::nth(self, n)
    }
}

// =============================================================================
// Test type: GrowableVec
// Demonstrates RExtend (collection extension with interior mutability)
// Generic trait — uses manual forwarding
// =============================================================================

#[derive(ExternalPtr)]
pub struct GrowableVec(RefCell<Vec<i32>>);

impl RExtend<i32> for GrowableVec {
    fn extend_from_vec(&self, items: Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }

    fn len(&self) -> i64 {
        self.0.borrow().len() as i64
    }
}

/// @noRd
#[miniextendr]
impl GrowableVec {
    fn new() -> Self {
        GrowableVec(RefCell::new(Vec::new()))
    }

    fn from_vec(data: Vec<i32>) -> Self {
        GrowableVec(RefCell::new(data))
    }

    fn extend(&self, items: Vec<i32>) {
        RExtend::extend_from_vec(self, items);
    }

    fn len(&self) -> i64 {
        RExtend::<i32>::len(self)
    }

    fn is_empty(&self) -> bool {
        RExtend::<i32>::is_empty(self)
    }

    fn to_vec(&self) -> Vec<i32> {
        Clone::clone(&*self.0.borrow())
    }

    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// =============================================================================
// Test type: IntSet
// Demonstrates RFromIter and RToVec (collection creation and extraction)
// Generic traits — uses manual forwarding
// =============================================================================

use std::collections::HashSet;

#[derive(ExternalPtr)]
pub struct IntSet(HashSet<i32>);

/// @noRd
#[miniextendr]
impl IntSet {
    // RFromIter via wrapper (HashSet implements FromIterator)
    fn from_vec(items: Vec<i32>) -> Self {
        IntSet(RFromIter::from_vec(items))
    }

    fn len(&self) -> i64 {
        RToVec::<i32>::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        RToVec::<i32>::is_empty(&self.0)
    }

    fn to_vec(&self) -> Vec<i32> {
        let mut v = RToVec::to_vec(&self.0);
        v.sort();
        v
    }

    fn contains(&self, value: i32) -> bool {
        self.0.contains(&value)
    }
}

// =============================================================================
// Test type: IterableVec and IterableVecIter
// Demonstrates RMakeIter (iterator factory)
// Generic traits — uses manual forwarding
// =============================================================================

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

impl RMakeIter<i32, IterableVecIter> for IterableVec {
    fn make_iter(&self) -> IterableVecIter {
        IterableVecIter(RefCell::new(Clone::clone(&self.0).into_iter()))
    }
}

/// @noRd
#[miniextendr]
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

    fn make_iter(&self) -> IterableVecIter {
        RMakeIter::make_iter(self)
    }
}

#[miniextendr]
impl IterableVecIter {
    // Renamed from `next` because `next` is a reserved word in R
    fn next_item(&self) -> Option<i32> {
        RIterator::next(self)
    }

    // Returns (lower_bound, upper_bound) where upper_bound is -1 if unknown
    fn size_hint(&self) -> Vec<i32> {
        let (lo, hi) = RIterator::size_hint(self);
        vec![lo as i32, hi.map(|h| h as i32).unwrap_or(-1)]
    }

    fn collect_all(&self) -> Vec<i32> {
        let mut result = Vec::new();
        while let Some(item) = RIterator::next(self) {
            result.push(item);
        }
        result
    }
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod adapter_traits_tests;

    impl Point;
    impl MyFloat;
    impl ChainedError;
    impl IntVecIter;
    impl GrowableVec;
    impl IntSet;
    impl IterableVec;
    impl IterableVecIter;

    // Trait ABI registrations for adapter traits
    impl miniextendr_api::adapter_traits::RDebug for Point;
    impl miniextendr_api::adapter_traits::RDisplay for Point;
    impl miniextendr_api::adapter_traits::RHash for Point;
    impl miniextendr_api::adapter_traits::RClone for Point;
    impl miniextendr_api::adapter_traits::RDefault for Point;
    impl miniextendr_api::adapter_traits::RFromStr for Point;
    impl miniextendr_api::adapter_traits::RCopy for Point;
    impl miniextendr_api::adapter_traits::RError for ChainedError;
}
