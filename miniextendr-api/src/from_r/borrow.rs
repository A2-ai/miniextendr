//! Native vector borrow metadata and pre-conversion alias checks.

use crate::gc_protect::ProtectScope;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

/// The native vector storage borrowed by a conversion.
///
/// Containers preserve the leaf type and add one list level when their
/// conversion visits VECSXP elements. Optional NULLs require no extra level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBorrow {
    sexptype: SEXPTYPE,
    mutable: bool,
    scalar: bool,
    list_depth: usize,
}

impl NativeBorrow {
    /// A reference to the only element of a native R vector.
    pub const fn scalar(sexptype: SEXPTYPE, mutable: bool) -> Self {
        Self {
            sexptype,
            mutable,
            scalar: true,
            list_depth: 0,
        }
    }

    /// A slice over a native R vector, with empty vectors excluded from conflicts.
    pub const fn slice(sexptype: SEXPTYPE, mutable: bool) -> Self {
        Self {
            sexptype,
            mutable,
            scalar: false,
            list_depth: 0,
        }
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
        (self.mutable || other.mutable) && self.sexptype == other.sexptype
    }

    fn accepts_length(self, length: R_xlen_t) -> bool {
        if self.scalar { length == 1 } else { length > 0 }
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
        if a.mutable && a.list_depth > 0 {
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
            if !a.conflicts_with(b) || (same_arg && a.list_depth == 0) {
                continue;
            }
            let mut path_a = Vec::new();
            let mut path_b = Vec::new();
            let aliases = unsafe {
                visit_leaves(args[i].0, a, &mut path_a, &mut |leaf_a, indices_a| {
                    visit_leaves(args[j].0, b, &mut path_b, &mut |leaf_b, indices_b| {
                        (!same_arg || indices_a < indices_b) && leaf_a == leaf_b
                    })
                })
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
    visit: &mut impl FnMut(SEXP, &[R_xlen_t]) -> bool,
) -> bool {
    if borrow.list_depth == 0 {
        return sexp.type_of() == borrow.sexptype
            && borrow.accepts_length(sexp.xlength())
            && visit(sexp, path);
    }
    if sexp.type_of() != SEXPTYPE::VECSXP {
        return false;
    }
    for index in 0..sexp.xlength() {
        let scope = unsafe { ProtectScope::new() };
        let child = unsafe { scope.protect(sexp.vector_elt(index)).get() };
        path.push(index);
        let found = unsafe {
            visit_leaves(
                child,
                NativeBorrow {
                    list_depth: borrow.list_depth - 1,
                    ..borrow
                },
                path,
                visit,
            )
        };
        path.pop();
        if found {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_r::TryFromSexp;

    #[test]
    fn native_borrow_metadata_follows_actual_converters() {
        macro_rules! check {
            ($t:ty, $sexptype:ident) => {{
                let shared = Some(NativeBorrow::scalar(SEXPTYPE::$sexptype, false));
                let mutable = Some(NativeBorrow::scalar(SEXPTYPE::$sexptype, true));
                assert_eq!(<&$t>::NATIVE_BORROW, shared);
                assert_eq!(<&mut $t>::NATIVE_BORROW, mutable);
                assert_eq!(<Option<&$t>>::NATIVE_BORROW, shared);
                assert_eq!(<Option<&mut $t>>::NATIVE_BORROW, mutable);
                assert_eq!(
                    <Vec<&mut $t>>::NATIVE_BORROW,
                    NativeBorrow::in_list(mutable)
                );
                assert_eq!(
                    <Box<[Option<&mut $t>]>>::NATIVE_BORROW,
                    NativeBorrow::in_list(mutable)
                );
                assert_eq!(
                    <Option<Vec<Vec<&mut $t>>>>::NATIVE_BORROW,
                    NativeBorrow::in_list(NativeBorrow::in_list(mutable))
                );
                let slice = Some(NativeBorrow::slice(SEXPTYPE::$sexptype, true));
                assert_eq!(<&mut [$t]>::NATIVE_BORROW, slice);
                assert_eq!(
                    <Box<[Option<&mut [$t]>]>>::NATIVE_BORROW,
                    NativeBorrow::in_list(slice)
                );
                assert_eq!(<Vec<$t>>::NATIVE_BORROW, None);
                assert_eq!(<Box<[$t]>>::NATIVE_BORROW, None);
            }};
        }
        check!(i32, INTSXP);
        check!(f64, REALSXP);
        check!(u8, RAWSXP);
        check!(crate::RLogical, LGLSXP);
        check!(crate::Rcomplex, CPLXSXP);
        type HiddenBorrow = Option<Vec<Vec<&'static mut i32>>>;
        assert!(HiddenBorrow::NATIVE_BORROW.is_some());
    }

    #[test]
    fn forwarding_wrappers_preserve_native_borrow_metadata() {
        type Leaf = &'static mut i32;
        let scalar = Leaf::NATIVE_BORROW;
        let list = NativeBorrow::in_list(scalar);
        assert_eq!(<Result<Leaf, ()>>::NATIVE_BORROW, scalar);
        assert_eq!(<crate::Missing<Leaf>>::NATIVE_BORROW, scalar);
        assert_eq!(
            <std::collections::HashMap<String, Leaf>>::NATIVE_BORROW,
            list
        );
        assert_eq!(
            <Option<std::collections::BTreeMap<String, Leaf>>>::NATIVE_BORROW,
            list
        );
        assert_eq!(
            <Vec<std::collections::HashMap<String, Leaf>>>::NATIVE_BORROW,
            NativeBorrow::in_list(list)
        );
        assert_eq!(
            <std::borrow::Cow<'static, [i32]>>::NATIVE_BORROW,
            <&[i32]>::NATIVE_BORROW
        );
        assert_eq!(
            <crate::RCow<'static, i32>>::NATIVE_BORROW,
            <&[i32]>::NATIVE_BORROW
        );
        #[cfg(feature = "indexmap")]
        assert_eq!(<indexmap::IndexMap<String, Leaf>>::NATIVE_BORROW, list);
    }

    #[test]
    fn custom_reference_conversions_default_to_no_native_borrow() {
        struct RLogical;
        impl TryFromSexp for &RLogical {
            type Error = ();
            fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
                unreachable!()
            }
        }
        assert_eq!(<&RLogical>::NATIVE_BORROW, None);
        assert_eq!(<String>::NATIVE_BORROW, None);
    }

    #[test]
    fn metadata_avoids_unneeded_walks_and_preserves_length_rules() {
        let shared = NativeBorrow::scalar(SEXPTYPE::INTSXP, false);
        let mutable = NativeBorrow::scalar(SEXPTYPE::INTSXP, true);
        assert!(!needs_check(&[]));
        assert!(!needs_check(&[None, Some(shared), Some(shared)]));
        assert!(!needs_check(&[Some(mutable)]));
        assert!(!needs_check(&[
            Some(mutable),
            Some(NativeBorrow::slice(SEXPTYPE::REALSXP, true))
        ]));
        assert!(needs_check(&[Some(mutable), Some(shared)]));
        assert!(needs_check(&[NativeBorrow::in_list(Some(mutable))]));
        assert!(mutable.accepts_length(1));
        assert!(!mutable.accepts_length(0));
        assert!(!mutable.accepts_length(2));
        assert!(NativeBorrow::slice(SEXPTYPE::INTSXP, true).accepts_length(2));
        assert!(!NativeBorrow::slice(SEXPTYPE::INTSXP, true).accepts_length(0));
    }
}
