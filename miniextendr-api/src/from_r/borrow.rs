//! Native vector borrow metadata and pre-conversion alias checks.

use crate::gc_protect::ProtectScope;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

/// The native vector storage borrowed by a conversion.
///
/// Homogeneous containers add list levels; tuples describe each positional
/// field separately. Optional NULLs require no extra level. Owned fields have
/// no descriptor and never participate in alias checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBorrow {
    shape: BorrowShape,
    list_depth: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BorrowShape {
    Vector {
        sexptype: SEXPTYPE,
        mutable: bool,
        scalar: bool,
    },
    Tuple(&'static [Option<NativeBorrow>]),
    /// The arms of an `Either`, left first. Which arm borrows depends on which
    /// conversion succeeds, so the walk takes both (#1504).
    Either(&'static [Option<NativeBorrow>; 2]),
    /// A `match_arg` / `choices` `Either<T, R>`: character or factor input is
    /// the owned choice, any other input converts to `R`. The input's type
    /// alone decides it.
    ChoiceOr(&'static Option<NativeBorrow>),
}

impl NativeBorrow {
    /// A reference to the only element of a native R vector.
    pub const fn scalar(sexptype: SEXPTYPE, mutable: bool) -> Self {
        Self {
            shape: BorrowShape::Vector {
                sexptype,
                mutable,
                scalar: true,
            },
            list_depth: 0,
        }
    }

    /// A slice over a native R vector, with empty vectors excluded from conflicts.
    pub const fn slice(sexptype: SEXPTYPE, mutable: bool) -> Self {
        Self {
            shape: BorrowShape::Vector {
                sexptype,
                mutable,
                scalar: false,
            },
            list_depth: 0,
        }
    }

    /// Borrow metadata for a positional R list with exactly this many fields.
    /// Returns `None` when every field is owned.
    pub const fn tuple(fields: &'static [Option<Self>]) -> Option<Self> {
        let mut index = 0;
        while index < fields.len() {
            if fields[index].is_some() {
                return Some(Self {
                    shape: BorrowShape::Tuple(fields),
                    list_depth: 0,
                });
            }
            index += 1;
        }
        None
    }

    /// Borrow metadata for an `Either` whose arms convert with `arms`, left
    /// first. Returns `None` when neither arm borrows.
    ///
    /// The check treats it as the union of the arms, which is sound without
    /// knowing which arm converts. When that union meets another argument, a
    /// generated wrapper prepares the argument
    /// ([`TryFromSexp::__mx_prepare`](crate::from_r::TryFromSexp)) to learn
    /// the arm its conversion takes, and checks that arm alone.
    pub const fn either(arms: &'static [Option<Self>; 2]) -> Option<Self> {
        if arms[0].is_none() && arms[1].is_none() {
            None
        } else {
            Some(Self {
                shape: BorrowShape::Either(arms),
                list_depth: 0,
            })
        }
    }

    /// Borrow metadata for a `match_arg` / `choices` parameter of type
    /// `Either<T, R>`, where `right` is `R`'s: character or factor input goes
    /// to the owned choice `T`, any other input to `R`. Returns `None` when
    /// `R` borrows nothing.
    pub const fn choice_or(right: &'static Option<Self>) -> Option<Self> {
        if right.is_none() {
            None
        } else {
            Some(Self {
                shape: BorrowShape::ChoiceOr(right),
                list_depth: 0,
            })
        }
    }

    /// Whether `borrow` is an `Either` at the argument's top level: the shape
    /// a generated wrapper prepares to learn which arm converts. An `Either`
    /// inside a list or tuple is checked against both arms.
    pub const fn selects(borrow: Option<Self>) -> bool {
        matches!(
            borrow,
            Some(Self {
                shape: BorrowShape::Either(_),
                list_depth: 0,
            })
        )
    }

    /// Forward an element conversion through one R list level.
    pub const fn in_list(borrow: Option<Self>) -> Option<Self> {
        match borrow {
            Some(mut borrow) => {
                borrow.list_depth += 1;
                Some(borrow)
            }
            None => None,
        }
    }

    /// The metadata this shape is made of: a tuple's fields, the arms of an
    /// `Either`, a choice `Either`'s other arm. Empty for a leaf.
    fn parts(self) -> &'static [Option<Self>] {
        match self.shape {
            BorrowShape::Vector { .. } => &[],
            BorrowShape::Tuple(fields) => fields,
            BorrowShape::Either(arms) => arms,
            BorrowShape::ChoiceOr(right) => core::slice::from_ref(right),
        }
    }

    /// Whether the check of this metadata takes both arms of an `Either`
    /// anywhere in it.
    fn has_either(self) -> bool {
        matches!(self.shape, BorrowShape::Either(_))
            || self.parts().iter().flatten().any(|part| part.has_either())
    }

    /// Whether this and `other` can borrow one R vector, one of them
    /// mutably. Two vectors, the common pair, are compared here; a shape
    /// made of parts goes to the out-of-line [`Self::parts_conflict_with`].
    #[inline]
    fn conflicts_with(self, other: Self) -> bool {
        if let (
            BorrowShape::Vector {
                sexptype: a,
                mutable: am,
                ..
            },
            BorrowShape::Vector {
                sexptype: b,
                mutable: bm,
                ..
            },
        ) = (self.shape, other.shape)
        {
            return (am || bm) && a == b;
        }
        self.parts_conflict_with(other)
    }

    #[inline(never)]
    fn parts_conflict_with(self, other: Self) -> bool {
        if let BorrowShape::Vector { .. } = self.shape {
            other
                .parts()
                .iter()
                .flatten()
                .any(|b| self.conflicts_with(*b))
        } else {
            self.parts()
                .iter()
                .flatten()
                .any(|a| a.conflicts_with(other))
        }
    }

    /// Whether one argument can borrow one R vector twice, once mutably. A
    /// vector at the top level, the common argument, can't.
    #[inline]
    fn needs_internal_check(self) -> bool {
        if self.list_depth == 0 && matches!(self.shape, BorrowShape::Vector { .. }) {
            return false;
        }
        self.nested_needs_internal_check()
    }

    #[inline(never)]
    fn nested_needs_internal_check(self) -> bool {
        if self.list_depth > 0 {
            return self.conflicts_with(self);
        }
        match self.shape {
            BorrowShape::Vector { .. } => false,
            BorrowShape::Tuple(fields) => needs_check(fields),
            // One arm converts, so the arms never borrow together.
            BorrowShape::Either(_) | BorrowShape::ChoiceOr(_) => self
                .parts()
                .iter()
                .flatten()
                .any(|arm| arm.needs_internal_check()),
        }
    }

    fn accepts_length(self, length: R_xlen_t) -> bool {
        match self.shape {
            BorrowShape::Vector { scalar: true, .. } => length == 1,
            BorrowShape::Vector { scalar: false, .. } => length > 0,
            BorrowShape::Tuple(_) | BorrowShape::Either(_) | BorrowShape::ChoiceOr(_) => false,
        }
    }
}

/// Whether any conversion pair can alias, without inspecting or allocating in R.
///
/// Generated wrappers use this to skip the alias check for owned/custom
/// conversions and shared-only signatures.
#[doc(hidden)]
pub fn needs_check(borrows: &[Option<NativeBorrow>]) -> bool {
    for (i, borrow) in borrows.iter().enumerate() {
        let Some(a) = borrow else { continue };
        if a.needs_internal_check() {
            return true;
        }
        if borrows[i + 1..]
            .iter()
            .flatten()
            .any(|b| a.conflicts_with(*b))
        {
            return true;
        }
    }
    false
}

/// Reject every conflicting argument pair before any Rust borrow is constructed.
///
/// An `Either` anywhere in the metadata is checked against both arms. A
/// generated wrapper first calls [`assert_or_select`] and replaces a
/// prepared argument's metadata with the arm its conversion takes.
///
/// # Safety
///
/// Run on R's main thread inside an R unwind-protected wrapper. Argument SEXPs
/// must stay reachable for the whole call. Metadata must describe the actual
/// conversions of the corresponding arguments.
#[doc(hidden)]
pub unsafe fn assert_no_aliases<const N: usize>(
    args: [(SEXP, &str); N],
    borrows: [Option<NativeBorrow>; N],
) {
    let mut errors = Vec::new();
    unsafe {
        for_each_conflict(&args, &borrows, |i, j| {
            errors.push(message(&args, &borrows, i, j))
        })
    };
    assert!(
        errors.is_empty(),
        "aliasing native arguments: {}",
        errors.join("; ")
    );
}

/// The first pass of a generated wrapper's alias check (#1504).
///
/// Checks `borrows` as [`assert_no_aliases`] does, taking both arms of every
/// `Either`. A conflict that involves an argument whose top level is an
/// `Either` ([`NativeBorrow::selects`]) may vanish once that argument's arm is
/// known, so it raises nothing and marks the argument instead. Returns the
/// marked arguments: the wrapper prepares them, replaces their metadata with
/// what the prepared conversion borrows, and calls [`assert_no_aliases`],
/// which then reports every conflict that remains. With nothing marked, every
/// conflict found has already been raised.
///
/// # Safety
///
/// As for [`assert_no_aliases`].
#[doc(hidden)]
pub unsafe fn assert_or_select<const N: usize>(
    args: [(SEXP, &str); N],
    borrows: [Option<NativeBorrow>; N],
) -> [bool; N] {
    let mut select = [false; N];
    let mut errors = Vec::new();
    unsafe {
        for_each_conflict(&args, &borrows, |i, j| {
            let (si, sj) = (
                NativeBorrow::selects(borrows[i]),
                NativeBorrow::selects(borrows[j]),
            );
            if si || sj {
                select[i] |= si;
                select[j] |= sj;
            } else {
                errors.push(message(&args, &borrows, i, j));
            }
        })
    };
    if !select.contains(&true) {
        assert!(
            errors.is_empty(),
            "aliasing native arguments: {}",
            errors.join("; ")
        );
    }
    select
}

/// Call `found(i, j)` (`i <= j`) for every argument pair whose borrows alias,
/// an argument with itself included.
///
/// # Safety
///
/// As for [`assert_no_aliases`].
unsafe fn for_each_conflict<const N: usize>(
    args: &[(SEXP, &str); N],
    borrows: &[Option<NativeBorrow>; N],
    mut found: impl FnMut(usize, usize),
) {
    for i in 0..N {
        let Some(a) = borrows[i] else { continue };
        for j in i..N {
            let Some(b) = borrows[j] else { continue };
            let same_arg = i == j;
            if !a.conflicts_with(b) || (same_arg && !a.needs_internal_check()) {
                continue;
            }
            let mut path_a = Vec::new();
            let mut path_b = Vec::new();
            let aliases = unsafe {
                visit_leaves(
                    args[i].0,
                    a,
                    &mut path_a,
                    &mut |leaf_a, indices_a, borrow_a| {
                        visit_leaves(
                            args[j].0,
                            b,
                            &mut path_b,
                            &mut |leaf_b, indices_b, borrow_b| {
                                (!same_arg || indices_a < indices_b)
                                    && leaf_a == leaf_b
                                    && borrow_a.conflicts_with(borrow_b)
                            },
                        )
                    },
                )
            };
            if aliases {
                found(i, j);
            }
        }
    }
}

/// The error for a conflict between arguments `i` and `j`.
fn message<const N: usize>(
    args: &[(SEXP, &str); N],
    borrows: &[Option<NativeBorrow>; N],
    i: usize,
    j: usize,
) -> String {
    let name_a = args[i].1;
    let name_b = args[j].1;
    let mut message = if i == j {
        format!(
            "parameter `{name_a}` contains duplicate elements that borrow the same non-empty R vector mutably"
        )
    } else {
        format!(
            "parameters `{name_a}` and `{name_b}` borrow the same non-empty R object, and at least one borrows it mutably; pass distinct vectors"
        )
    };
    if [borrows[i], borrows[j]]
        .into_iter()
        .flatten()
        .any(NativeBorrow::has_either)
    {
        message.push_str(
            " (an `Either` whose arm is not known before conversion is checked against both arms)",
        );
    }
    message
}

/// Keep each current ALTREP/list element rooted while the other argument is
/// visited. Protection depth follows type nesting, not the number of elements.
///
/// A plain vector, the common argument, is visited here; every other shape
/// goes to [`visit_nested`], kept out of line so this stays small enough to
/// inline into the check of a call with plain borrowed vectors.
#[inline]
unsafe fn visit_leaves(
    sexp: SEXP,
    borrow: NativeBorrow,
    path: &mut Vec<R_xlen_t>,
    visit: &mut impl FnMut(SEXP, &[R_xlen_t], NativeBorrow) -> bool,
) -> bool {
    if borrow.list_depth == 0
        && let BorrowShape::Vector { sexptype, .. } = borrow.shape
    {
        return sexp.type_of() == sexptype
            && borrow.accepts_length(sexp.xlength())
            && visit(sexp, path, borrow);
    }
    unsafe { visit_nested(sexp, borrow, path, visit) }
}

/// [`visit_leaves`] for a list, a tuple or an `Either`.
#[inline(never)]
unsafe fn visit_nested(
    sexp: SEXP,
    borrow: NativeBorrow,
    path: &mut Vec<R_xlen_t>,
    visit: &mut impl FnMut(SEXP, &[R_xlen_t], NativeBorrow) -> bool,
) -> bool {
    if borrow.list_depth == 0 {
        match borrow.shape {
            BorrowShape::Vector { .. } => unreachable!("visit_leaves visits a plain vector"),
            // Whichever arm converts, its leaves are among these.
            BorrowShape::Either(arms) => {
                return arms
                    .iter()
                    .flatten()
                    .any(|arm| unsafe { visit_leaves(sexp, *arm, path, visit) });
            }
            BorrowShape::ChoiceOr(right) => {
                if sexp.type_of() == SEXPTYPE::STRSXP || sexp.is_factor() {
                    return false;
                }
                return right
                    .is_some_and(|right| unsafe { visit_leaves(sexp, right, path, visit) });
            }
            BorrowShape::Tuple(_) => {}
        }
    }
    if sexp.type_of() != SEXPTYPE::VECSXP {
        return false;
    }
    let length = sexp.xlength();
    if borrow.list_depth == 0
        && let BorrowShape::Tuple(fields) = borrow.shape
        && usize::try_from(length).ok() != Some(fields.len())
    {
        // Conversion will report the shape error before borrowing any field.
        return false;
    }
    for index in 0..length {
        let child_borrow = if borrow.list_depth > 0 {
            NativeBorrow {
                list_depth: borrow.list_depth - 1,
                ..borrow
            }
        } else {
            let BorrowShape::Tuple(fields) = borrow.shape else {
                unreachable!()
            };
            let Some(field) = fields[usize::try_from(index).unwrap()] else {
                continue;
            };
            field
        };
        let scope = unsafe { ProtectScope::new() };
        let child = unsafe { scope.protect(sexp.vector_elt(index)).get() };
        path.push(index);
        let found = unsafe { visit_leaves(child, child_borrow, path, visit) };
        path.pop();
        if found {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
