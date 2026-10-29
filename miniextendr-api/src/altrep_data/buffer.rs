//! Native Vec ownership that preserves allocations already exposed to R.
use super::{AltrepDataptr, AltrepLen, AltrepSerialize};
use crate::{IntoR, RNativeType, SEXP, TryFromSexp};

/// A growable native buffer whose exposed allocations survive later edits.
///
/// Wrap in `Altrep` to return it to R. Built-in classes use value semantics:
/// R duplication clones the current values, so a copy R makes keeps its own
/// values. An edit through the owner of an R object changes that object in
/// place, for every binding R has of it.
///
/// [`edit`](Self::edit) detaches first if R has requested a data pointer, and
/// keeps the old allocation until this owner is dropped. Previously returned
/// pointers then refer to the prior values; subsequent callbacks see the new
/// values. R provides no pointer-release notification, so repeated exposure
/// followed by editing retains one buffer per such edit. No buffer is copied
/// when it has only been read through `Elt` or `Get_region`.
///
/// This type deliberately has no `DerefMut` or method returning `&mut Vec<T>`:
/// growth must pass through the edit boundary.
#[derive(Debug)]
pub struct AltrepBuffer<T: RNativeType> {
    values: Vec<T>,
    exposed: bool,
    retired: Vec<Vec<T>>,
}

impl<T: RNativeType> AltrepBuffer<T> {
    /// Take ownership without copying or exposing the allocation.
    pub fn new(values: Vec<T>) -> Self {
        Self {
            values,
            exposed: false,
            retired: Vec::new(),
        }
    }

    /// Read the current values. The borrow prevents concurrent Rust edits.
    pub fn as_slice(&self) -> &[T] {
        &self.values
    }

    /// Edit the Vec, retaining its old allocation if a pointer was exposed.
    /// A panic in `edit` leaves earlier pointers valid and keeps any changes
    /// already made to the new Vec; it does not roll the edit back.
    pub fn edit<R>(&mut self, edit: impl FnOnce(&mut Vec<T>) -> R) -> R {
        if self.exposed {
            let next = self.values.clone();
            self.retired.push(std::mem::replace(&mut self.values, next));
            self.exposed = false;
        }
        edit(&mut self.values)
    }

    /// Current number of elements.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Capacity of the current Vec, excluding retained older allocations.
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }
    /// Whether the current vector is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<T: RNativeType> AsRef<[T]> for AltrepBuffer<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: RNativeType> From<Vec<T>> for AltrepBuffer<T> {
    fn from(values: Vec<T>) -> Self {
        Self::new(values)
    }
}
impl<T: RNativeType> Clone for AltrepBuffer<T> {
    fn clone(&self) -> Self {
        Self::new(self.values.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.edit(|values| values.clone_from(&source.values));
    }
}
impl<T: RNativeType> AltrepLen for AltrepBuffer<T> {
    fn altrep_len(&self) -> usize {
        self.len()
    }
}
impl<T: RNativeType> crate::externalptr::TypedExternal for AltrepBuffer<T> {
    const TYPE_NAME: &'static str = "AltrepBuffer";
    const TYPE_NAME_CSTR: &'static [u8] = b"AltrepBuffer\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr::AltrepBuffer\0";
}
impl<T: RNativeType> AltrepSerialize for AltrepBuffer<T>
where
    Vec<T>: IntoR + TryFromSexp,
{
    fn serialized_state(&self) -> SEXP {
        self.values.clone().into_sexp()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        Vec::<T>::try_from_sexp(state).ok().map(Self::new)
    }
}

macro_rules! native_buffer {
    ($element:ty, $storage:ty, $family:ident) => {
        impl super::$family for AltrepBuffer<$element> {
            fn elt(&self, i: usize) -> $storage {
                self.values[i]
            }
            fn as_slice(&self) -> Option<&[$storage]> {
                Some(&self.values)
            }
        }
        impl AltrepDataptr<$storage> for AltrepBuffer<$element> {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $storage> {
                self.exposed = true;
                Some(self.values.as_mut_ptr().cast())
            }
            // Use the mutable path even for full read-only Dataptr requests,
            // preserving mutable pointer provenance. Pointer peeks stay lazy.
        }
    };
}
native_buffer!(i32, i32, AltIntegerData);
native_buffer!(f64, f64, AltRealData);
native_buffer!(u8, u8, AltRawData);
native_buffer!(crate::Rcomplex, crate::Rcomplex, AltComplexData);

impl super::AltLogicalData for AltrepBuffer<crate::RLogical> {
    fn elt(&self, i: usize) -> super::Logical {
        self.values[i].into()
    }
    fn as_r_slice(&self) -> Option<&[i32]> {
        // RLogical has the same layout and valid bit patterns as i32.
        Some(unsafe { std::slice::from_raw_parts(self.values.as_ptr().cast(), self.values.len()) })
    }
}
impl AltrepDataptr<i32> for AltrepBuffer<crate::RLogical> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        self.exposed = true;
        Some(self.values.as_mut_ptr().cast())
    }
}

#[cfg(test)]
mod tests;
