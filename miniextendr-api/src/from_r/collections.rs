//! Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet).
//!
//! Named R lists convert to `HashMap<String, V>` / `BTreeMap<String, V>`.
//! Unnamed R vectors convert to `HashSet<T>` / `BTreeSet<T>` for native types.
//! Nested lists convert to `Vec<HashMap<String, V>>` etc.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ffi::{RLogical, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_str};

macro_rules! impl_map_try_from_sexp {
    ($(#[$meta:meta])* $map_ty:ident, $create:expr) => {
        $(#[$meta])*
        impl<V: TryFromSexp> TryFromSexp for $map_ty<String, V>
        where
            V::Error: Into<SexpError>,
        {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                named_list_to_map(sexp, $create)
            }
        }
    };
}

impl_map_try_from_sexp!(
    /// Convert R named list (VECSXP) to HashMap<String, V>.
    ///
    /// See `named_list_to_map` for NA/empty name handling (elements with NA/empty
    /// names map to key `""` and may silently overwrite each other).
    HashMap, HashMap::with_capacity
);
impl_map_try_from_sexp!(
    /// Convert R named list (VECSXP) to BTreeMap<String, V>.
    ///
    /// See `named_list_to_map` for NA/empty name handling (elements with NA/empty
    /// names map to key `""` and may silently overwrite each other).
    BTreeMap, |_| BTreeMap::new()
);

/// Helper to convert R named list to a map type.
///
/// Returns an error if the list has duplicate non-empty, non-NA names.
///
/// # NA and Empty Name Handling
///
/// **Warning:** Elements with NA or empty names are converted with key `""`:
/// - `NA` names become empty string key `""`
/// - Empty string names `""` stay as `""`
/// - If multiple elements have NA/empty names, later ones **silently overwrite** earlier ones
///
/// This means data loss can occur without error if your list has multiple
/// unnamed or NA-named elements.
///
/// **Example of silent data loss:**
/// ```r
/// # In R:
/// x <- list(a = 1, 2, 3)  # Elements 2 and 3 have empty names
/// # After conversion, only one of them survives under key ""
/// ```
///
/// If you need all elements regardless of names, use `Vec<(String, V)>` instead,
/// or convert the list to a vector first.
fn named_list_to_map<V, M, F>(sexp: SEXP, create_map: F) -> Result<M, SexpError>
where
    V: TryFromSexp,
    V::Error: Into<SexpError>,
    M: Extend<(String, V)>,
    F: FnOnce(usize) -> M,
{
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    let mut map = create_map(len);

    // Get names attribute
    let names = sexp.get_names();
    let has_names = names.type_of() == SEXPTYPE::STRSXP && names.len() == len;

    // Single-pass: check duplicates AND convert in one loop
    let mut seen = HashSet::with_capacity(len);

    for i in 0..len {
        let key = if has_names {
            let charsxp = names.string_elt(i as crate::ffi::R_xlen_t);
            if charsxp == SEXP::na_string() {
                String::new()
            } else {
                unsafe { charsxp_to_str(charsxp) }.to_owned()
            }
        } else {
            // Use index as key if no names
            i.to_string()
        };

        // Check duplicate for non-empty keys
        if !key.is_empty() && !seen.insert(key.clone()) {
            return Err(SexpError::DuplicateName(key));
        }

        let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
        let value = V::try_from_sexp(elem).map_err(|e| e.into())?;
        map.extend(std::iter::once((key, value)));
    }

    Ok(map)
}

macro_rules! impl_vec_map_try_from_sexp {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: TryFromSexp> TryFromSexp for Vec<$map_ty<String, V>>
        where
            V::Error: Into<SexpError>,
        {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                list_to_vec_of_maps::<$map_ty<String, V>>(sexp)
            }
        }
    };
}

impl_vec_map_try_from_sexp!(
    /// Convert R list of named lists to `Vec<HashMap<String, V>>`.
    HashMap
);
impl_vec_map_try_from_sexp!(
    /// Convert R list of named lists to `Vec<BTreeMap<String, V>>`.
    BTreeMap
);

/// Helper to convert R list (VECSXP) to `Vec<M>` where each element is
/// converted via `M: TryFromSexp`.
fn list_to_vec_of_maps<M>(sexp: SEXP) -> Result<Vec<M>, SexpError>
where
    M: TryFromSexp,
    M::Error: Into<SexpError>,
{
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
        let map = M::try_from_sexp(elem).map_err(Into::into)?;
        result.push(map);
    }

    Ok(result)
}

macro_rules! impl_set_try_from_sexp_native {
    ($set:ident<$t:ty>) => {
        impl TryFromSexp for $set<$t> {
            type Error = SexpTypeError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let slice: &[$t] = TryFromSexp::try_from_sexp(sexp)?;
                Ok(slice.iter().copied().collect())
            }
        }
    };
}

impl_set_try_from_sexp_native!(HashSet<i32>);
impl_set_try_from_sexp_native!(HashSet<u8>);
impl_set_try_from_sexp_native!(HashSet<RLogical>);
impl_set_try_from_sexp_native!(BTreeSet<i32>);
impl_set_try_from_sexp_native!(BTreeSet<u8>);

macro_rules! impl_vec_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for Vec<$t> {
            type Error = SexpTypeError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let slice: &[$t] = TryFromSexp::try_from_sexp(sexp)?;
                Ok(slice.to_vec())
            }
        }
    };
}

impl_vec_try_from_sexp_native!(i32);
impl_vec_try_from_sexp_native!(f64);
impl_vec_try_from_sexp_native!(u8);
impl_vec_try_from_sexp_native!(RLogical);
impl_vec_try_from_sexp_native!(crate::ffi::Rcomplex);

macro_rules! impl_boxed_slice_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for Box<[$t]> {
            type Error = SexpTypeError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let slice: &[$t] = TryFromSexp::try_from_sexp(sexp)?;
                Ok(slice.into())
            }
        }
    };
}

impl_boxed_slice_try_from_sexp_native!(i32);
impl_boxed_slice_try_from_sexp_native!(f64);
impl_boxed_slice_try_from_sexp_native!(u8);
impl_boxed_slice_try_from_sexp_native!(RLogical);
impl_boxed_slice_try_from_sexp_native!(crate::ffi::Rcomplex);
// endregion
