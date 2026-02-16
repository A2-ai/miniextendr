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

use miniextendr_api::adapter_traits::RIterator;
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
}

// Trait ABI: adapter traits registered via empty impl + TPIE (Trait-Provided Impl Expansion).
// The trait definition exports method metadata; empty impls auto-expand C/R wrappers.

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

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::ROrd for Point {}

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
}

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RPartialOrd for MyFloat {}

// =============================================================================
// Test type: ChainedError
// Demonstrates RError (error chain walking)
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
// Non-blanket trait with associated type — full impl required
// =============================================================================

#[derive(ExternalPtr)]
pub struct IntVecIter(RefCell<std::vec::IntoIter<i32>>);

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RIterator for IntVecIter {
    type Item = i32;

    #[miniextendr(r_name = "next_item")]
    fn next(&self) -> Option<Self::Item> {
        self.0.borrow_mut().next()
    }

    #[miniextendr(skip)]
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

/// @noRd
#[miniextendr]
impl IntVecIter {
    fn new(data: Vec<i32>) -> Self {
        IntVecIter(RefCell::new(data.into_iter()))
    }
}

// =============================================================================
// Test type: GrowableVec
// Demonstrates RExtend (collection extension with interior mutability)
// Non-blanket generic trait — full impl required
// =============================================================================

#[derive(ExternalPtr)]
pub struct GrowableVec(RefCell<Vec<i32>>);

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RExtend<i32> for GrowableVec {
    fn extend_from_vec(&self, items: Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }

    fn len(&self) -> i64 {
        self.0.borrow().len() as i64
    }

    fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
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
// =============================================================================

use std::collections::HashSet;

#[derive(ExternalPtr)]
pub struct IntSet(HashSet<i32>);

// RFromIter: static factory method (IntSet doesn't impl FromIterator directly)
/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RFromIter<i32> for IntSet {
    fn from_vec(items: Vec<i32>) -> Self {
        IntSet(items.into_iter().collect())
    }
}

// RToVec: sorted output for deterministic tests
/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RToVec<i32> for IntSet {
    fn to_vec(&self) -> Vec<i32> {
        let mut v: Vec<_> = self.0.iter().cloned().collect();
        v.sort();
        v
    }

    fn len(&self) -> i64 {
        self.0.len() as i64
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// @noRd
#[miniextendr]
impl IntSet {
    fn contains(&self, value: i32) -> bool {
        self.0.contains(&value)
    }
}

// =============================================================================
// Test type: IterableVec and IterableVecIter
// Demonstrates RMakeIter (iterator factory)
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

/// @noRd
#[miniextendr]
impl miniextendr_api::adapter_traits::RMakeIter<i32, IterableVecIter> for IterableVec {
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
}

/// @noRd
#[miniextendr(blanket)]
impl miniextendr_api::adapter_traits::RIterator for IterableVecIter {
    type Item = i32;

    #[miniextendr(r_name = "next_item")]
    fn next(&self) -> Option<Self::Item> {
        unreachable!()
    }

    #[miniextendr(skip)]
    fn size_hint(&self) -> (i64, Option<i64>) {
        unreachable!()
    }

    fn count(&self) -> i64 {
        unreachable!()
    }

    fn collect_n(&self, n: i32) -> Vec<Self::Item> {
        unreachable!()
    }

    fn skip(&self, n: i32) -> i32 {
        unreachable!()
    }

    fn nth(&self, n: i32) -> Option<Self::Item> {
        unreachable!()
    }
}

/// @noRd
#[miniextendr]
impl IterableVecIter {
    fn collect_all(&self) -> Vec<i32> {
        let mut result = Vec::new();
        while let Some(item) = RIterator::next(self) {
            result.push(item);
        }
        result
    }
}

// =============================================================================
// Test type: ExportControlTraitPoint
// Demonstrates internal/noexport on trait impls
// =============================================================================

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
#[miniextendr(internal)]
impl ExportControlTraitPoint {
    fn new(x: i32) -> Self {
        ExportControlTraitPoint { x }
    }
}

/// Internal trait impl: should have @keywords internal, no @export
#[miniextendr(internal)]
impl miniextendr_api::adapter_traits::RDebug for ExportControlTraitPoint {}

/// Noexport trait impl: should have docs but no @export
#[miniextendr(noexport)]
impl miniextendr_api::adapter_traits::RDisplay for ExportControlTraitPoint {}

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
    impl ExportControlTraitPoint;

    // Non-generic adapter traits (TPIE: empty impl auto-expands wrappers)
    impl miniextendr_api::adapter_traits::RDebug for Point;
    impl miniextendr_api::adapter_traits::RDisplay for Point;
    impl miniextendr_api::adapter_traits::RHash for Point;
    impl miniextendr_api::adapter_traits::RClone for Point;
    impl miniextendr_api::adapter_traits::RDefault for Point;
    impl miniextendr_api::adapter_traits::RFromStr for Point;
    impl miniextendr_api::adapter_traits::RCopy for Point;
    impl miniextendr_api::adapter_traits::RError for ChainedError;
    impl miniextendr_api::adapter_traits::ROrd for Point;
    impl miniextendr_api::adapter_traits::RPartialOrd for MyFloat;

    // Export control on trait impls
    impl miniextendr_api::adapter_traits::RDebug for ExportControlTraitPoint;
    impl miniextendr_api::adapter_traits::RDisplay for ExportControlTraitPoint;

    // Associated-type trait (non-blanket)
    impl miniextendr_api::adapter_traits::RIterator for IntVecIter;
    impl miniextendr_api::adapter_traits::RIterator for IterableVecIter;

    // Generic traits
    impl miniextendr_api::adapter_traits::RExtend<i32> for GrowableVec;
    impl miniextendr_api::adapter_traits::RFromIter<i32> for IntSet;
    impl miniextendr_api::adapter_traits::RToVec<i32> for IntSet;
    impl miniextendr_api::adapter_traits::RMakeIter<i32, IterableVecIter> for IterableVec;
}
