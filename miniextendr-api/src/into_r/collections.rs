//! Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet) to R.
//!
//! - `HashMap<String, V>` / `BTreeMap<String, V>` → named R list
//! - `HashSet<T>` / `BTreeSet<T>` → unnamed R vector (via Vec intermediary)

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use crate::ffi::SexpExt;
use crate::into_r::{IntoR, str_to_charsxp, str_to_charsxp_unchecked};

macro_rules! impl_map_into_r {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: IntoR> IntoR for $map_ty<String, V> {
            type Error = crate::into_r_error::IntoRError;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                map_to_named_list(self.into_iter())
            }
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { map_to_named_list_unchecked(self.into_iter()) }
            }
        }
    };
}

impl_map_into_r!(
    /// Convert HashMap<String, V> to R named list (VECSXP).
    HashMap
);
impl_map_into_r!(
    /// Convert BTreeMap<String, V> to R named list (VECSXP).
    BTreeMap
);

/// Helper to convert an iterator of (String, V) pairs to a named R list.
fn map_to_named_list<V: IntoR>(
    iter: impl ExactSizeIterator<Item = (String, V)>,
) -> crate::ffi::SEXP {
    unsafe {
        let n: crate::ffi::R_xlen_t = iter
            .len()
            .try_into()
            .expect("map length exceeds isize::MAX");
        let list = crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n);
        crate::ffi::Rf_protect(list);

        // Allocate names vector
        let names = crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n);
        crate::ffi::Rf_protect(names);

        for (i, (key, value)) in iter.enumerate() {
            let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
            // Set list element
            list.set_vector_elt(idx, value.into_sexp());

            // Set name
            let charsxp = str_to_charsxp(&key);
            crate::ffi::SET_STRING_ELT(names, idx, charsxp);
        }

        // Attach names attribute
        list.set_names(names);

        crate::ffi::Rf_unprotect(2);
        list
    }
}

/// Unchecked version of [`map_to_named_list`].
unsafe fn map_to_named_list_unchecked<V: IntoR>(
    iter: impl ExactSizeIterator<Item = (String, V)>,
) -> crate::ffi::SEXP {
    unsafe {
        let n: crate::ffi::R_xlen_t = iter
            .len()
            .try_into()
            .expect("map length exceeds isize::MAX");
        let list = crate::ffi::Rf_allocVector_unchecked(crate::ffi::SEXPTYPE::VECSXP, n);
        crate::ffi::Rf_protect(list);

        let names = crate::ffi::Rf_allocVector_unchecked(crate::ffi::SEXPTYPE::STRSXP, n);
        crate::ffi::Rf_protect(names);

        for (i, (key, value)) in iter.enumerate() {
            let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
            crate::ffi::SET_VECTOR_ELT_unchecked(list, idx, value.into_sexp_unchecked());

            let charsxp = str_to_charsxp_unchecked(&key);
            crate::ffi::SET_STRING_ELT_unchecked(names, idx, charsxp);
        }

        crate::ffi::Rf_setAttrib_unchecked(list, crate::ffi::R_NamesSymbol, names);

        crate::ffi::Rf_unprotect(2);
        list
    }
}

/// Convert `HashSet<T>` to R vector.
impl<T> IntoR for HashSet<T>
where
    T: crate::ffi::RNativeType + Eq + Hash,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

/// Convert `BTreeSet<T>` to R vector.
impl<T> IntoR for BTreeSet<T>
where
    T: crate::ffi::RNativeType + Ord,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

macro_rules! impl_set_string_into_r {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl IntoR for $set_ty<String> {
            type Error = crate::into_r_error::IntoRError;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<String> = self.into_iter().collect();
                vec.into_sexp()
            }
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let vec: Vec<String> = self.into_iter().collect();
                unsafe { vec.into_sexp_unchecked() }
            }
        }
    };
}

impl_set_string_into_r!(
    /// Convert `HashSet<String>` to R character vector.
    HashSet
);
impl_set_string_into_r!(
    /// Convert `BTreeSet<String>` to R character vector.
    BTreeSet
);
// endregion
