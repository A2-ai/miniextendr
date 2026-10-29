//! [`RObject`]: an R object inside a serde value (#2256).

use std::cell::{Cell, RefCell};
use std::fmt;

use crate::into_r::IntoR;
use crate::{Preserved, SEXP};
use serde::{de, ser};

/// The newtype name an [`RObject`] serializes and deserializes through.
/// miniextendr's serializers and deserializers act on it; any other format
/// reaches [`RObject`]'s refusal.
pub(crate) const MARKER: &str = "$miniextendr::RObject";

/// An R object inside a serde value, rooted for as long as the value holds
/// it.
///
/// [`to_r`](super::to_r) writes the object itself where the value holds it,
/// and [`from_r`](super::from_r) gives the same object back (not a copy):
///
/// ```ignore
/// use miniextendr_api::serde::RObject;
///
/// #[derive(Serialize, Deserialize)]
/// struct Fit {
///     coefficients: Vec<f64>,
///     data: RObject, // the data frame the model was fitted on
/// }
///
/// let fit = Fit { coefficients, data: RObject::new(df) };
/// to_r(&fit)?; // list(coefficients = c(...), data = <the data frame>)
/// ```
///
/// An [`ExternalPtr`](struct@crate::ExternalPtr) to a serde type holding one
/// saves the object with the rest of its value: R's `serialize()` writes it
/// into the save, and `readRDS()` gives it back to the decoded value. Within
/// one save R shares only environments, external pointers and symbols: an R6
/// object or another `ExternalPtr` held in two places comes back as one
/// object (an `ExternalPtr` to a serde type with its own value, through its
/// own carrier), and a data frame held in two places as two copies.
///
/// - The columnar `data.frame` builders (`vec_to_dataframe` & co.) put it in
///   a list column; the parallel ones refuse list columns, this one too.
/// - Other serializers (`serde_json`,
///   [`RValue::from_serde`](crate::RValue::from_serde)) refuse it with an
///   error: an R object has no other encoding, and no address is written.
/// - An internally tagged enum can't hold it as a newtype variant's payload
///   (`Kind(RObject)`): serde allows only a struct or map there. Put it in a
///   struct variant's field.
/// - Inside an untagged or internally tagged enum, or a flattened struct,
///   serde buffers the value before it knows the target type. A snapshot
///   carries the object through that buffer; `from_r` can't, and refuses
///   with an error.
/// - In `to_r` / `from_r` the general mapping applies around the object: an
///   `Option<RObject>` holding `NULL` or a length-1 `NA` reads back as
///   `None`. A snapshot keeps it.
/// - Make, clone and drop it on R's main thread, as a [`Preserved`]. A type
///   holding one is `!Send`, so a `#[miniextendr]` function takes it as `&T`
///   or `&mut T`, not as an `ExternalPtr<T>`.
#[derive(Clone)]
pub struct RObject(Preserved);

impl RObject {
    /// Convert `value` to an R object and root it; a [`SEXP`] converts to
    /// itself. Call it on R's main thread.
    pub fn new(value: impl IntoR) -> Self {
        RObject(Preserved::new(value))
    }

    /// The object, rooted for as long as this value lives.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.0.as_sexp()
    }

    /// The root this value holds.
    pub fn into_preserved(self) -> Preserved {
        self.0
    }
}

impl From<Preserved> for RObject {
    fn from(root: Preserved) -> Self {
        RObject(root)
    }
}

impl From<RObject> for Preserved {
    fn from(object: RObject) -> Self {
        object.0
    }
}

impl std::ops::Deref for RObject {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &SEXP {
        &self.0
    }
}

impl IntoR for RObject {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.0.try_into_sexp()
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        // SAFETY: the caller's contract, passed on.
        unsafe { self.0.try_into_sexp_unchecked() }
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }
}

impl fmt::Debug for RObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RObject").field(&self.as_sexp()).finish()
    }
}

// region: Handing the object over

thread_local! {
    /// The object an [`RObject`] is serializing, for the miniextendr
    /// serializer its `serialize_newtype_struct` reaches.
    static WRITING: Cell<Option<SEXP>> = const { Cell::new(None) };
    /// The object the last [`written`] call handed over.
    static WRITTEN: Cell<Option<SEXP>> = const { Cell::new(None) };
    /// The object a miniextendr deserializer hands to [`RObject`]'s visitor.
    static READING: Cell<Option<SEXP>> = const { Cell::new(None) };
    /// The objects the snapshot decoder buffered, by token: `Some` while it
    /// runs.
    static BUFFERED: RefCell<Option<Vec<SEXP>>> = const { RefCell::new(None) };
}

/// Puts `cell`'s value back when dropped, also on a panic.
struct Restore<T: 'static + Copy> {
    cell: &'static std::thread::LocalKey<Cell<T>>,
    value: T,
}

impl<T: 'static + Copy> Drop for Restore<T> {
    fn drop(&mut self) {
        self.cell.set(self.value);
    }
}

/// The object an [`RObject`] is serializing: a miniextendr serializer calls
/// it from `serialize_newtype_struct(MARKER, ..)`. `None` when the marker
/// came from anything else.
pub(crate) fn written() -> Option<SEXP> {
    let sexp = WRITING.take();
    WRITTEN.set(sexp);
    sexp
}

/// Whether `sexp` is the object the last [`written`] call handed over, after
/// [`forget_written`]: a sequence element that is an `RObject` itself.
pub(crate) fn is_written(sexp: SEXP) -> bool {
    WRITTEN.get() == Some(sexp)
}

/// Forgets the object the last [`written`] call handed over.
pub(crate) fn forget_written() {
    WRITTEN.set(None);
}

/// Calls `visit` with `sexp` ready for [`RObject`]'s visitor to take: a
/// miniextendr deserializer's `deserialize_newtype_struct(MARKER, ..)`.
pub(crate) fn reading<T>(sexp: SEXP, visit: impl FnOnce() -> T) -> T {
    let _restore = Restore {
        cell: &READING,
        value: READING.replace(Some(sexp)),
    };
    visit()
}

/// Runs `decode` with an empty buffer for [`buffer`]: the snapshot decoder.
pub(crate) fn with_buffer<T>(decode: impl FnOnce() -> T) -> T {
    struct Outer(Option<Vec<SEXP>>);
    impl Drop for Outer {
        fn drop(&mut self) {
            BUFFERED.set(self.0.take());
        }
    }
    let _outer = Outer(BUFFERED.replace(Some(Vec::new())));
    decode()
}

/// The prefix of a buffered object's token.
const TOKEN: &str = "$miniextendr::RObject#";

/// Buffers `sexp`, which the decoder's input roots, and returns its token,
/// which the decoder visits for serde's `Content` to keep. `None` outside
/// [`with_buffer`].
pub(crate) fn buffer(sexp: SEXP) -> Option<String> {
    BUFFERED.with_borrow_mut(|objects| {
        let objects = objects.as_mut()?;
        objects.push(sexp);
        Some(format!("{TOKEN}{}", objects.len() - 1))
    })
}

/// The object buffered under `token`, while the decoder that buffered it
/// runs.
fn buffered(token: &str) -> Option<SEXP> {
    let index: usize = token.strip_prefix(TOKEN)?.parse().ok()?;
    BUFFERED.with_borrow(|objects| objects.as_ref()?.get(index).copied())
}

// endregion

// region: serde

impl ser::Serialize for RObject {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let _restore = Restore {
            cell: &WRITING,
            value: WRITING.replace(Some(self.as_sexp())),
        };
        serializer.serialize_newtype_struct(MARKER, &Refusal)
    }
}

/// What a serializer other than miniextendr's finds inside the newtype.
struct Refusal;

impl ser::Serialize for Refusal {
    fn serialize<S: ser::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(ser::Error::custom(
            "`serde::RObject` holds an R object, which only `serde::to_r` and an \
             `ExternalPtr` snapshot can write",
        ))
    }
}

impl<'de> de::Deserialize<'de> for RObject {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(MARKER, RObjectVisitor)
    }
}

struct RObjectVisitor;

impl<'de> de::Visitor<'de> for RObjectVisitor {
    type Value = RObject;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an R object (`serde::RObject`)")
    }

    fn visit_newtype_struct<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<RObject, D::Error> {
        if let Some(sexp) = READING.take() {
            return Ok(RObject::new(sexp));
        }
        // Inside serde's buffered `Content`: a snapshot's token.
        <String as de::Deserialize>::deserialize(deserializer)
            .ok()
            .and_then(|token| buffered(&token))
            .map(RObject::new)
            .ok_or_else(|| {
                de::Error::custom(
                    "no R object for `serde::RObject` here: only `serde::from_r` and an \
                     `ExternalPtr` snapshot give one, and inside an untagged or internally \
                     tagged enum or a flattened struct only a snapshot does",
                )
            })
    }
}

// endregion

#[cfg(test)]
mod tests;
