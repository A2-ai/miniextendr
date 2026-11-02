//! Preparing an argument whose native borrow depends on the arm its
//! conversion takes (#1504).
//!
//! An `Either` borrows through whichever arm converts first, and only the
//! conversion can tell which. A generated wrapper's alias check first takes
//! both arms ([`NativeBorrow::either`]); when that union meets another
//! argument, the wrapper prepares the argument
//! ([`TryFromSexp::__mx_prepare`]): the conversion runs once, up to the point
//! where it would build a native reference. An owned arm's value is kept, so
//! it converts once. A borrowing arm records what it borrows and the arms
//! that lead to it, and builds its reference only after the check passes
//! ([`TryFromSexp::__mx_materialize`], through [`convert`]).

use crate::SEXP;
use crate::from_r::{NativeBorrow, TryFromSexp};

/// The arms a prepared conversion took through nested `Either`s, outermost
/// first: bit `k` is the arm at depth `k` (set for the right arm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arms {
    bits: u64,
    depth: u32,
}

impl Arms {
    /// No `Either` above the borrowing leaf.
    pub const LEAF: Self = Self { bits: 0, depth: 0 };

    /// This path under one more `Either` whose arm is the right one when
    /// `right`. `None` past 64 levels, which the caller treats as undecided.
    pub const fn under(self, right: bool) -> Option<Self> {
        if self.depth == u64::BITS {
            return None;
        }
        Some(Self {
            bits: (self.bits << 1) | if right { 1 } else { 0 },
            depth: self.depth + 1,
        })
    }

    /// Whether the outermost arm is the right one, and the path below it.
    pub const fn pop(self) -> (bool, Self) {
        (
            self.bits & 1 == 1,
            Self {
                bits: self.bits >> 1,
                depth: self.depth.saturating_sub(1),
            },
        )
    }
}

/// An argument's conversion as a generated wrapper's alias check leaves it:
/// `T`'s, whose conversion error is `E`.
///
/// The type has no `TryFromSexp` bound, so a wrapper can declare and read
/// the state of an argument whose type has no conversion without a second
/// error: [`convert`] reports it, at the type.
#[doc(hidden)]
#[must_use]
pub enum Prepared<T, E = <T as TryFromSexp>::Error> {
    /// Not prepared: it converts after the check, which takes its declared
    /// metadata.
    Unprepared,
    /// Converted during preparation and borrows nothing native: the value
    /// is the argument.
    Ready(T),
    /// Failed during preparation. The error is reported where the argument
    /// converts, so the order of argument errors does not change.
    Failed(E),
    /// A borrowing arm will convert: `borrow` is all it borrows, `arms` the
    /// way to it. Its reference is built after the check.
    Deferred {
        /// What the selected arm borrows.
        borrow: Option<NativeBorrow>,
        /// The arms from the argument's type down to the borrowing leaf.
        arms: Arms,
    },
    /// The arm could not be learned without building a reference: it
    /// converts after the check, which takes `borrow` (every arm it may
    /// still take).
    Undecided(Option<NativeBorrow>),
}

impl<T: TryFromSexp> Prepared<T> {
    /// A borrowing leaf whose checks passed: its conversion will succeed and
    /// borrow what `T::NATIVE_BORROW` says.
    pub fn deferred() -> Self {
        Self::Deferred {
            borrow: T::NATIVE_BORROW,
            arms: Arms::LEAF,
        }
    }
}

impl<T, E> Prepared<T, E> {
    /// An owned conversion's result.
    pub fn from_result(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(error) => Self::Failed(error),
        }
    }

    /// What the argument borrows for the second pass of the check, given its
    /// declared metadata.
    pub fn borrow(&self, declared: Option<NativeBorrow>) -> Option<NativeBorrow> {
        match self {
            Self::Unprepared => declared,
            Self::Ready(_) | Self::Failed(_) => None,
            Self::Deferred { borrow, .. } | Self::Undecided(borrow) => *borrow,
        }
    }

    /// The preparation of a type that wraps `T`: `value` and `error` build
    /// its value and error, and its `__mx_materialize` must follow the same
    /// arms through `T`'s.
    pub fn map<U: TryFromSexp>(
        self,
        value: impl FnOnce(T) -> U,
        error: impl FnOnce(E) -> U::Error,
    ) -> Prepared<U> {
        match self {
            Self::Unprepared => Prepared::Unprepared,
            Self::Ready(v) => Prepared::Ready(value(v)),
            Self::Failed(e) => Prepared::Failed(error(e)),
            Self::Deferred { borrow, arms } => Prepared::Deferred { borrow, arms },
            Self::Undecided(borrow) => Prepared::Undecided(borrow),
        }
    }

    /// This preparation as one arm of an `Either` (`right` for the right
    /// arm), whose own metadata is `either`.
    pub fn arm<U: TryFromSexp>(
        self,
        right: bool,
        value: impl FnOnce(T) -> U,
        either: Option<NativeBorrow>,
    ) -> Prepared<U> {
        match self {
            Self::Ready(v) => Prepared::Ready(value(v)),
            Self::Deferred { borrow, arms } => match arms.under(right) {
                Some(arms) => Prepared::Deferred { borrow, arms },
                None => Prepared::Undecided(either),
            },
            Self::Undecided(borrow) => Prepared::Undecided(borrow),
            Self::Unprepared | Self::Failed(_) => {
                unreachable!("an arm's failure is handled before it is wrapped")
            }
        }
    }
}

/// Convert an argument that a generated wrapper's alias check may have
/// prepared. Call it only after the check: a deferred arm builds its
/// reference here.
#[doc(hidden)]
#[inline]
pub fn convert<T: TryFromSexp>(prepared: Prepared<T>, sexp: SEXP) -> Result<T, T::Error> {
    match prepared {
        Prepared::Unprepared | Prepared::Undecided(_) => T::try_from_sexp(sexp),
        Prepared::Ready(value) => Ok(value),
        Prepared::Failed(error) => Err(error),
        Prepared::Deferred { arms, .. } => T::__mx_materialize(sexp, arms),
    }
}

/// The preparation of a native reference conversion whose checks `check`
/// runs: a passed check defers the reference, a failed one is the error.
pub(crate) fn native_leaf<T: TryFromSexp>(check: Result<(), T::Error>) -> Prepared<T> {
    match check {
        Ok(()) => Prepared::deferred(),
        Err(error) => Prepared::Failed(error),
    }
}

#[cfg(test)]
mod tests;
