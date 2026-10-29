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

    fn conflicts_with(self, other: Self) -> bool {
        match (self.shape, other.shape) {
            (BorrowShape::Tuple(fields), _) => {
                fields.iter().flatten().any(|a| a.conflicts_with(other))
            }
            (_, BorrowShape::Tuple(fields)) => {
                fields.iter().flatten().any(|b| self.conflicts_with(*b))
            }
            (
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
            ) => (am || bm) && a == b,
        }
    }

    fn needs_internal_check(self) -> bool {
        if self.list_depth > 0 {
            return self.conflicts_with(self);
        }
        match self.shape {
            BorrowShape::Tuple(fields) => needs_check(fields),
            BorrowShape::Vector { .. } => false,
        }
    }

    fn accepts_length(self, length: R_xlen_t) -> bool {
        match self.shape {
            BorrowShape::Vector { scalar: true, .. } => length == 1,
            BorrowShape::Vector { scalar: false, .. } => length > 0,
            BorrowShape::Tuple(_) => false,
        }
    }
}

/// Whether any conversion pair can alias, without inspecting or allocating in R.
///
/// Generated worker wrappers use this before entering R unwind protection so
/// owned/custom conversions and shared-only signatures need no extra boundary.
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
                let name_a = args[i].1;
                let name_b = args[j].1;
                errors.push(if same_arg {
                    format!("parameter `{name_a}` contains duplicate elements that borrow the same non-empty R vector mutably")
                } else {
                    format!("parameters `{name_a}` and `{name_b}` borrow the same non-empty R object, and at least one borrows it mutably; pass distinct vectors")
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

/// Keep each current ALTREP/list element rooted while the other argument is
/// visited. Protection depth follows type nesting, not the number of elements.
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
