//! An explicit, stable index over an owned string-keyed map.
use std::collections::{BTreeMap, HashMap};

use super::{AltrepLen, AltrepSerialize};
use crate::{IntoR, RNativeType, SEXP, TryFromSexp};

/// A named ALTREP vector over a map's values, indexed by sorted keys.
///
/// `From<HashMap<String, T>>` retains the map and builds a sorted key index;
/// reads never depend on hash-table iteration order. Returning this type to R
/// attaches those keys as names. Built-in classes cover native integer, real,
/// raw, complex and logical values. Names are eager, values are read lazily.
///
/// The map is immutable after construction. R writes materialize an independent
/// value cache; subsequent R reads and serialization use that cache. Rust map
/// accessors describe the original map, so this is a value snapshot rather than
/// a shared mutable dictionary. No hash-table bucket pointer is exposed to R.
#[derive(Clone, Debug)]
pub struct AltrepMap<T: RNativeType> {
    values: HashMap<String, T>,
    keys: Vec<String>,
}

impl<T: RNativeType> From<HashMap<String, T>> for AltrepMap<T> {
    fn from(values: HashMap<String, T>) -> Self {
        let mut keys: Vec<_> = values.keys().cloned().collect();
        keys.sort_unstable();
        Self { values, keys }
    }
}
impl<T: RNativeType> From<BTreeMap<String, T>> for AltrepMap<T> {
    fn from(values: BTreeMap<String, T>) -> Self {
        Self::from(values.into_iter().collect::<HashMap<_, _>>())
    }
}
impl<T: RNativeType> AltrepMap<T> {
    /// Keys in R's positional order (Rust string ordering).
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    /// Read an original map value by key.
    pub fn get(&self, key: &str) -> Option<&T> {
        self.values.get(key)
    }
    /// Borrow the original map.
    pub fn as_map(&self) -> &HashMap<String, T> {
        &self.values
    }
    /// Recover the original map before handing ownership to R.
    pub fn into_map(self) -> HashMap<String, T> {
        self.values
    }
    /// Number of indexed values.
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    fn value_at(&self, i: usize) -> T {
        self.values[&self.keys[i]]
    }
}
impl<T: RNativeType> AltrepLen for AltrepMap<T> {
    fn altrep_len(&self) -> usize {
        self.len()
    }
}
impl<T: RNativeType> crate::externalptr::TypedExternal for AltrepMap<T> {
    const TYPE_NAME: &'static str = "AltrepMap";
    const TYPE_NAME_CSTR: &'static [u8] = b"AltrepMap\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr::AltrepMap\0";
}
impl<T: RNativeType + IntoR + TryFromSexp> AltrepSerialize for AltrepMap<T>
where
    HashMap<String, T>: TryFromSexp,
{
    fn serialized_state(&self) -> SEXP {
        // Deterministic serialization too: ordinary HashMap IntoR follows its
        // randomized iteration order, while this state must match our index.
        self.keys
            .iter()
            .map(|key| (key.clone(), self.values[key]))
            .collect::<BTreeMap<_, _>>()
            .into_sexp()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        HashMap::<String, T>::try_from_sexp(state)
            .ok()
            .map(Self::from)
    }
}
impl<T: RNativeType> IntoR for AltrepMap<T>
where
    Self: crate::altrep::RegisterAltrep,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        unsafe {
            let names = crate::OwnedProtect::new(self.keys.clone().into_sexp());
            let vector = crate::OwnedProtect::new(crate::Altrep(self).into_sexp());
            crate::sys::Rf_setAttrib_unchecked(
                vector.get(),
                crate::sys::R_NamesSymbol,
                names.get(),
            );
            Ok(vector.get())
        }
    }
}

macro_rules! native_map {
    ($element:ty, $family:ident) => {
        impl super::$family for AltrepMap<$element> {
            fn elt(&self, i: usize) -> $element {
                self.value_at(i)
            }
        }
    };
}
native_map!(i32, AltIntegerData);
native_map!(f64, AltRealData);
native_map!(u8, AltRawData);
native_map!(crate::Rcomplex, AltComplexData);
impl super::AltLogicalData for AltrepMap<crate::RLogical> {
    fn elt(&self, i: usize) -> super::Logical {
        self.value_at(i).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AltIntegerData;

    #[test]
    fn map_order_is_independent_of_insertion_order() {
        let first = AltrepMap::from(HashMap::from([
            ("z".into(), 3),
            ("a".into(), 1),
            ("m".into(), 2),
        ]));
        let second = AltrepMap::from(HashMap::from([
            ("m".into(), 2),
            ("a".into(), 1),
            ("z".into(), 3),
        ]));
        assert_eq!(first.keys(), &["a", "m", "z"]);
        assert_eq!(first.keys(), second.keys());
        for i in 0..3 {
            assert_eq!(first.elt(i), second.elt(i));
        }
        assert_eq!(first.get("m"), Some(&2));
        assert_eq!(first.get("absent"), None);
    }
}
