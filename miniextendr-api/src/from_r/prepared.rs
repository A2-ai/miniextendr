//! Conversion preparation for branch-dependent native borrows (#1504).
//!
//! A successful preparation fixes the conversion result without retaining a
//! native Rust reference. Owned converters run once; native converters retain
//! their original raw pointer and root, and form references only in `finish`.
//! This is the runtime contract for the forthcoming generated preflight path.
//! Existing wrappers still use their static metadata; they do not use this
//! protocol yet.

use crate::Root;

/// A native byte range that a prepared conversion will borrow.
///
/// The address is only for overlap comparison. Materializers retain their
/// original pointers, including mutable provenance.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct BorrowRange {
    address: usize,
    bytes: usize,
    mutable: bool,
}

impl BorrowRange {
    fn overlaps(self, other: Self) -> bool {
        if self.bytes == 0 || other.bytes == 0 || !(self.mutable || other.mutable) {
            return false;
        }
        if self.address <= other.address {
            other.address - self.address < self.bytes
        } else {
            self.address - other.address < other.bytes
        }
    }
}

/// A validated conversion whose native references have not been constructed.
///
/// The source root lives through preparation and materialization. Call
/// `assert_no_aliases` with all arguments' ranges before `finish`.
///
/// Custom borrowing converters implement `TryFromSexp::__mx_prepare` and
/// build their deferred result with `native`. The default hook accepts only
/// converters with no native borrow metadata, so it never guesses that a
/// borrowing converter can run safely before preflight.
#[doc(hidden)]
pub struct Prepared<'root, T> {
    input: Root<'root>,
    borrows: Vec<BorrowRange>,
    value: Value<'root, T>,
}

enum Value<'root, T> {
    Ready(T),
    Deferred(Box<dyn FnOnce() -> T + 'root>),
}

impl<'root, T: 'root> Prepared<'root, T> {
    pub(crate) fn ready(input: Root<'root>, value: T) -> Self {
        Self {
            input,
            borrows: Vec::new(),
            value: Value::Ready(value),
        }
    }

    /// Prepare one native range without constructing a Rust reference.
    ///
    /// # Safety
    ///
    /// `pointer` and `len` describe storage kept live by `input` for this
    /// preparation and the returned borrow's lifetime. `mutable` describes
    /// the single native reference `make` will construct. `make` must retain
    /// the original pointer's provenance, perform no conversion checks or R API
    /// calls, and introduce no borrows outside this range. It may run only
    /// after all conflicting arguments have been rejected.
    pub unsafe fn native<E>(
        input: Root<'root>,
        pointer: *const E,
        len: usize,
        mutable: bool,
        make: impl FnOnce() -> T + 'root,
    ) -> Self {
        Self {
            input,
            borrows: vec![BorrowRange {
                address: pointer.addr(),
                bytes: len * core::mem::size_of::<E>(),
                mutable,
            }],
            value: Value::Deferred(Box::new(make)),
        }
    }

    /// The actual ranges selected by this conversion, not its unused alternatives.
    pub fn borrows(&self) -> &[BorrowRange] {
        &self.borrows
    }

    /// Wrap a prepared value without changing its selected native borrows.
    ///
    /// # Safety
    ///
    /// `map` must perform no further conversion checks or R API calls and
    /// introduce no additional native references. Wrapping in a newtype,
    /// `Option`, `Result` or an `Either` variant satisfies this contract.
    pub unsafe fn map<U: 'root>(self, map: impl FnOnce(T) -> U + 'root) -> Prepared<'root, U> {
        let Self {
            input,
            borrows,
            value,
        } = self;
        Prepared {
            input,
            borrows,
            value: match value {
                Value::Ready(value) => Value::Ready(map(value)),
                Value::Deferred(make) => Value::Deferred(Box::new(move || map(make()))),
            },
        }
    }

    /// Compose already validated children without materializing their borrows.
    ///
    /// # Safety
    ///
    /// `borrows` must contain every range selected by `make`'s children.
    /// `make` may only finish those children and assemble their Rust values;
    /// it must not perform further validation or R API calls.
    pub(crate) unsafe fn deferred(
        input: Root<'root>,
        borrows: Vec<BorrowRange>,
        make: impl FnOnce() -> T + 'root,
    ) -> Self {
        Self {
            input,
            borrows,
            value: Value::Deferred(Box::new(make)),
        }
    }

    /// Construct the selected Rust value after preflight.
    ///
    /// # Safety
    ///
    /// All participating arguments' ranges must have passed `assert_no_aliases`
    /// together. No conflicting references may exist outside those arguments;
    /// no intervening mutation may invalidate their storage. A raw
    /// `TryFromSexp` result still requires its caller to keep the R source alive
    /// for the result's lifetime.
    pub unsafe fn finish(self) -> T {
        match self.value {
            Value::Ready(value) => value,
            Value::Deferred(make) => make(),
        }
    }
}

/// Reject every conflicting pair before any prepared native reference exists.
///
/// This only compares byte ranges and allocates Rust diagnostics. Empty ranges
/// and shared/shared overlaps are valid. Different R/ALTREP objects exposing
/// overlapping storage are rejected too.
#[doc(hidden)]
pub fn assert_no_aliases(args: &[(&str, &[BorrowRange])]) {
    let mut errors = Vec::new();
    for (i, &(name, ranges)) in args.iter().enumerate() {
        for (j, &(other_name, other_ranges)) in args.iter().enumerate().skip(i) {
            let same = i == j;
            let overlaps = ranges.iter().enumerate().any(|(a, range)| {
                other_ranges
                    .iter()
                    .enumerate()
                    .any(|(b, other)| (!same || a < b) && range.overlaps(*other))
            });
            if overlaps {
                errors.push(if same {
                    format!("parameter `{name}` contains overlapping native borrows, at least one mutable")
                } else {
                    format!("parameters `{name}` and `{other_name}` have overlapping native borrows, at least one mutable")
                });
            }
        }
    }
    assert!(
        errors.is_empty(),
        "aliasing native arguments: {}",
        errors.join("; ")
    );
}

/// Preserve left-first conversion, including each arm's own validation error.
#[cfg(feature = "either")]
pub(crate) fn either<'root, L: 'root, R: 'root, LE, RE>(
    left: impl FnOnce() -> Result<Prepared<'root, L>, LE>,
    right: impl FnOnce() -> Result<Prepared<'root, R>, RE>,
) -> Result<Prepared<'root, either::Either<L, R>>, super::SexpError>
where
    LE: Into<super::SexpError>,
    RE: Into<super::SexpError>,
{
    let left_error = match left() {
        Ok(value) => return Ok(unsafe { value.map(either::Either::Left) }),
        Err(error) => error.into(),
    };
    match right() {
        Ok(value) => Ok(unsafe { value.map(either::Either::Right) }),
        Err(error) => Err(super::SexpError::EitherConversion {
            left_error: Box::new(left_error),
            right_error: Box::new(error.into()),
        }),
    }
}

/// Keep all selected element ranges until the whole collection is checked.
pub(crate) fn sequence<'root, T: 'root>(
    root: Root<'root>,
    values: Vec<Prepared<'root, T>>,
) -> Prepared<'root, Vec<T>> {
    let borrows = values
        .iter()
        .flat_map(|v| v.borrows().iter().copied())
        .collect();
    unsafe {
        Prepared::deferred(root, borrows, move || {
            values.into_iter().map(|value| value.finish()).collect()
        })
    }
}

/// Read and retain each list child before the next allocating ALTREP accessor.
pub(crate) fn list<'root, T: 'root>(
    root: Root<'root>,
    mut prepare: impl FnMut(Root<'root>) -> Result<Prepared<'root, T>, super::SexpError>,
) -> Result<Prepared<'root, Vec<T>>, super::SexpError> {
    let values = super::map_vecsxp_batched(root.get(), |value| {
        prepare(unsafe { root.protect_child(value) })
    })?;
    Ok(sequence(root, values))
}
