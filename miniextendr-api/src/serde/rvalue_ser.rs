//! Serializer for converting Rust values to [`RValue`] via serde.
//!
//! The owned, `Send` counterpart of [`super::ser::RSerializer`]: the same type
//! mapping, but the output is an [`RValue`] tree instead of a live `SEXP`, so it
//! can be built on any thread and travel through `panic_any`. This is what
//! condition `data =` payloads and `#[miniextendr(serde_error)]` need: a
//! serialized error's fields have to cross the unwind (and possibly the
//! worker→main thread) boundary before any R object exists.
//!
//! # Type mapping
//!
//! | Rust | [`RValue`] |
//! |---|---|
//! | `bool` | `Logical([Some(b)])` |
//! | `i8` / `i16` / `i32` / `u8` / `u16` | `Integer([Some(v)])` |
//! | `i64` / `u32` / `u64` | `Integer` when the value fits `i32` and is not `NA_integer_`, else `Double` |
//! | `f32` / `f64` | `Double([v])` |
//! | `char` / `str` / `String` | `Character([Some(s)])` |
//! | `serialize_bytes` | `Raw` |
//! | `()` / unit struct / `None` | `Null` |
//! | unit enum variant | `Character([Some("Variant")])` |
//! | newtype struct | transparent |
//! | newtype / tuple / struct enum variant | `List([("Variant", payload)])` |
//! | sequence of same-kind scalars | one atomic vector |
//! | other sequences, tuples | unnamed `List` |
//! | struct, map with string keys | named `List` |
//!
//! `None` becomes `Null`, not a typed `NA`: the serializer never sees the
//! inner type of an absent `Option`. A `Vec<Option<T>>` therefore becomes a
//! list with `NULL` holes, exactly as [`RSerializer::to_sexp`] renders it.
//! For the same reason an empty sequence becomes an empty list. A field under
//! [`as_character`], [`as_integer`], [`as_double`] or [`as_logical`]
//! (`#[serde(serialize_with = ...)]`) is always that atomic vector instead.
//!
//! [`RSerializer::to_sexp`]: super::ser::RSerializer::to_sexp

use super::error::RSerdeError;
use crate::IsNA;
use crate::rvalue::RValue;
use serde::ser::{self, Serialize};

// region: Public entry points

/// Serializer that converts Rust values to an owned [`RValue`] tree.
///
/// ```
/// use miniextendr_api::RValue;
/// use miniextendr_api::serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point { x: f64, tag: String }
///
/// let v = RValue::from_serde(&Point { x: 1.5, tag: "a".into() }).unwrap();
/// match v {
///     RValue::List(fields) => {
///         assert_eq!(fields[0].0.as_deref(), Some("x"));
///         assert!(matches!(&fields[0].1, RValue::Double(d) if d == &[1.5]));
///     }
///     other => panic!("expected a named list, got {other:?}"),
/// }
/// ```
pub struct RValueSerializer;

/// Serialize any `Serialize` value into an [`RValue`].
pub fn to_rvalue<T: ?Sized + Serialize>(value: &T) -> Result<RValue, RSerdeError> {
    value.serialize(RValueSerializer)
}

impl RValue {
    /// Build an [`RValue`] from any `Serialize` value; see [`to_rvalue`] and
    /// the [module docs](self) for the type mapping.
    pub fn from_serde<T: ?Sized + Serialize>(value: &T) -> Result<Self, RSerdeError> {
        to_rvalue(value)
    }
}

// endregion

// region: Scalar and container helpers

/// `i64` policy shared with `IntoR for i64`: `Integer` when the value fits an
/// `i32` other than `NA_integer_`, otherwise `Double` (lossy past 2^53, as R
/// itself is).
fn int_or_double(v: i64) -> RValue {
    match i32::try_from(v) {
        Ok(i) if !i.is_na() => RValue::Integer(vec![Some(i)]),
        // Deliberate widening: R has no 64-bit integer; this mirrors the
        // REALSXP fallback of `i64::into_sexp`.
        _ => RValue::Double(vec![v as f64]),
    }
}

fn unnamed_list(elements: Vec<RValue>) -> RValue {
    RValue::List(elements.into_iter().map(|e| (None, e)).collect())
}

fn tagged(variant: &str, payload: RValue) -> RValue {
    RValue::List(vec![(Some(variant.to_string()), payload)])
}

/// Which atomic kind a length-1 `RValue` is, for sequence coalescing.
fn scalar_kind(v: &RValue) -> Option<u8> {
    match v {
        RValue::Logical(x) if x.len() == 1 => Some(0),
        RValue::Integer(x) if x.len() == 1 => Some(1),
        RValue::Double(x) if x.len() == 1 => Some(2),
        RValue::Complex(x) if x.len() == 1 => Some(3),
        RValue::Character(x) if x.len() == 1 => Some(4),
        RValue::Raw(x) if x.len() == 1 => Some(5),
        _ => None,
    }
}

/// Homogeneous length-1 scalars become one atomic vector (the `RValue` twin of
/// `List::from_scalars_or_list`); anything else stays an unnamed list.
fn coalesce(elements: Vec<RValue>) -> RValue {
    let Some(kind) = elements.first().and_then(scalar_kind) else {
        return unnamed_list(elements);
    };
    if !elements.iter().all(|e| scalar_kind(e) == Some(kind)) {
        return unnamed_list(elements);
    }
    macro_rules! concat {
        ($variant:path) => {
            $variant(
                elements
                    .into_iter()
                    .flat_map(|e| match e {
                        $variant(v) => v,
                        // `scalar_kind` proved every element is this variant.
                        _ => unreachable!("coalesce: kind checked above"),
                    })
                    .collect(),
            )
        };
    }
    match kind {
        0 => concat!(RValue::Logical),
        1 => concat!(RValue::Integer),
        2 => concat!(RValue::Double),
        3 => concat!(RValue::Complex),
        4 => concat!(RValue::Character),
        _ => concat!(RValue::Raw),
    }
}

/// A map key must serialize to exactly one string.
fn key_string(key: RValue) -> Result<String, RSerdeError> {
    match key {
        RValue::Character(mut v) if v.len() == 1 => {
            v.pop().flatten().ok_or(RSerdeError::NonStringKey)
        }
        _ => Err(RSerdeError::NonStringKey),
    }
}

// endregion

// region: Atomic field helpers (#2164)

/// The atomic R type an `as_*` helper gives a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Atomic {
    Logical,
    Integer,
    Double,
    Character,
}

/// The newtype names the helpers serialize through, one per [`Atomic`].
/// [`RValueSerializer`] and [`RSerializer`](super::RSerializer) recognise
/// them; any other serializer sees a transparent newtype.
const ATOMIC_MARKERS: [(&str, Atomic); 4] = [
    ("$miniextendr::as_logical", Atomic::Logical),
    ("$miniextendr::as_integer", Atomic::Integer),
    ("$miniextendr::as_double", Atomic::Double),
    ("$miniextendr::as_character", Atomic::Character),
];

impl Atomic {
    /// Every type, in the order of `condition::SerdeErrorShape`'s `as_*` lists.
    pub(crate) const ALL: [Atomic; 4] = [
        Atomic::Logical,
        Atomic::Integer,
        Atomic::Double,
        Atomic::Character,
    ];

    /// The type a newtype marker `name` stands for.
    pub(crate) fn from_marker(name: &str) -> Option<Self> {
        ATOMIC_MARKERS
            .iter()
            .find(|(marker, _)| *marker == name)
            .map(|(_, atomic)| *atomic)
    }

    fn marker(self) -> &'static str {
        ATOMIC_MARKERS
            .iter()
            .find(|(_, atomic)| *atomic == self)
            .map(|(marker, _)| *marker)
            .expect("every Atomic has a marker")
    }

    /// The helper's name, which is also the `serde_error` option's.
    pub(crate) fn helper(self) -> &'static str {
        match self {
            Atomic::Logical => "as_logical",
            Atomic::Integer => "as_integer",
            Atomic::Double => "as_double",
            Atomic::Character => "as_character",
        }
    }

    fn what(self) -> &'static str {
        match self {
            Atomic::Logical => "a logical",
            Atomic::Integer => "an integer that fits an R integer",
            Atomic::Double => "a number",
            Atomic::Character => "a string",
        }
    }

    /// The field's usual serialization, `value`, as one vector of this type:
    /// an atomic vector of it as it is (an integer one widened for
    /// [`Atomic::Double`]), `NULL` (a `None`) as one `NA`, and an unnamed
    /// list (an empty sequence, or one with `None` holes) element by element,
    /// each `NULL` or a scalar of the type.
    pub(crate) fn coerce(self, value: RValue) -> Result<RValue, RSerdeError> {
        self.try_coerce(value).map_err(|element| {
            RSerdeError::Message(format!("`{}`: {}", self.helper(), self.mismatch(element)))
        })
    }

    /// [`Atomic::coerce`], failing with the element that doesn't fit (`None`
    /// for the value as a whole). The `serde_error` boundary options call it
    /// on a payload field that is already serialized.
    pub(crate) fn try_coerce(self, value: RValue) -> Result<RValue, Option<usize>> {
        let mut out = AtomicVec::new(self);
        match value {
            RValue::List(items) => {
                for (i, (name, item)) in items.into_iter().enumerate() {
                    // A struct or map field, or a nested sequence, is no element.
                    let element = matches!(item, RValue::Null) || scalar_kind(&item).is_some();
                    if name.is_some() || !element {
                        return Err(Some(i));
                    }
                    out.push(item).map_err(|()| Some(i))?;
                }
            }
            other => out.push(other).map_err(|()| None)?,
        }
        Ok(out.finish())
    }

    /// Why [`Atomic::try_coerce`] failed: `element` of the field (or the
    /// field) is not of this type.
    pub(crate) fn mismatch(self, element: Option<usize>) -> String {
        let place = match element {
            Some(i) => format!("element {} of the field", i + 1),
            None => "the field".to_string(),
        };
        format!("{place} is not {} or `None`", self.what())
    }
}

/// The vector [`Atomic::coerce`] builds.
enum AtomicVec {
    Logical(Vec<Option<bool>>),
    Integer(Vec<Option<i32>>),
    Double(Vec<f64>),
    Character(Vec<Option<String>>),
}

impl AtomicVec {
    fn new(atomic: Atomic) -> Self {
        match atomic {
            Atomic::Logical => AtomicVec::Logical(Vec::new()),
            Atomic::Integer => AtomicVec::Integer(Vec::new()),
            Atomic::Double => AtomicVec::Double(Vec::new()),
            Atomic::Character => AtomicVec::Character(Vec::new()),
        }
    }

    /// Append `value`'s elements: `NULL` is one `NA`.
    fn push(&mut self, value: RValue) -> Result<(), ()> {
        use crate::altrep_traits::NA_REAL;
        match (self, value) {
            (AtomicVec::Logical(out), RValue::Null) => out.push(None),
            (AtomicVec::Integer(out), RValue::Null) => out.push(None),
            (AtomicVec::Double(out), RValue::Null) => out.push(NA_REAL),
            (AtomicVec::Character(out), RValue::Null) => out.push(None),
            (AtomicVec::Logical(out), RValue::Logical(v)) => out.extend(v),
            (AtomicVec::Integer(out), RValue::Integer(v)) => out.extend(v),
            (AtomicVec::Double(out), RValue::Double(v)) => out.extend(v),
            (AtomicVec::Double(out), RValue::Integer(v)) => {
                out.extend(v.into_iter().map(|x| x.map_or(NA_REAL, f64::from)));
            }
            (AtomicVec::Character(out), RValue::Character(v)) => out.extend(v),
            _ => return Err(()),
        }
        Ok(())
    }

    fn finish(self) -> RValue {
        match self {
            AtomicVec::Logical(v) => RValue::Logical(v),
            AtomicVec::Integer(v) => RValue::Integer(v),
            AtomicVec::Double(v) => RValue::Double(v),
            AtomicVec::Character(v) => RValue::Character(v),
        }
    }
}

/// Serialize a field as an R `logical` vector, for
/// `#[serde(serialize_with = "miniextendr_api::serde::as_logical")]`.
///
/// See [`as_character`]: the same rules, for `bool`s.
pub fn as_logical<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ?Sized + Serialize,
    S: ser::Serializer,
{
    serializer.serialize_newtype_struct(Atomic::Logical.marker(), value)
}

/// Serialize a field as an R `integer` vector, for
/// `#[serde(serialize_with = "miniextendr_api::serde::as_integer")]`.
///
/// See [`as_character`]: the same rules, for integers. A value outside R's
/// integer range (or `i32::MIN`, R's `NA`) is an error, not a double.
pub fn as_integer<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ?Sized + Serialize,
    S: ser::Serializer,
{
    serializer.serialize_newtype_struct(Atomic::Integer.marker(), value)
}

/// Serialize a field as an R `double` vector, for
/// `#[serde(serialize_with = "miniextendr_api::serde::as_double")]`.
///
/// See [`as_character`]: the same rules, for numbers; integers widen.
pub fn as_double<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ?Sized + Serialize,
    S: ser::Serializer,
{
    serializer.serialize_newtype_struct(Atomic::Double.marker(), value)
}

/// Serialize a field as an R `character` vector, for
/// `#[serde(serialize_with = "miniextendr_api::serde::as_character")]`.
///
/// serde gives a serializer no element type, so an empty `Vec<String>`
/// becomes `list()` and a `Vec<Option<String>>` with a `None` a list with a
/// `NULL` hole (see the [module docs](self)). With this helper the field is
/// always a character vector:
///
/// | Field | Without | With `as_character` |
/// |---|---|---|
/// | `vec![]` | `list()` | `character(0)` |
/// | `vec!["a", "b"]` | `c("a", "b")` | `c("a", "b")` |
/// | `vec![Some("a"), None]` | `list("a", NULL)` | `c("a", NA)` |
/// | `None` (an `Option` field) | `NULL` | `NA_character_` |
///
/// The field may be a sequence or a single value; each element must be a
/// string (or `char`) or `None`, else serialization fails with an error
/// naming the element. Serializers other than miniextendr's (`serde_json`,
/// ...) see the field as it is: the helper serializes through a newtype
/// they pass through.
///
/// ```
/// use miniextendr_api::RValue;
/// use miniextendr_api::serde::Serialize;
///
/// #[derive(Serialize)]
/// struct NotFound {
///     #[serde(serialize_with = "miniextendr_api::serde::as_character")]
///     nearest: Vec<String>,
/// }
///
/// let v = RValue::from_serde(&NotFound { nearest: vec![] }).unwrap();
/// let RValue::List(fields) = v else { unreachable!() };
/// assert!(matches!(&fields[0].1, RValue::Character(c) if c.is_empty()));
/// ```
pub fn as_character<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ?Sized + Serialize,
    S: ser::Serializer,
{
    serializer.serialize_newtype_struct(Atomic::Character.marker(), value)
}

// endregion

// region: serde::Serializer for RValueSerializer

impl ser::Serializer for RValueSerializer {
    type Ok = RValue;
    type Error = RSerdeError;

    type SerializeSeq = RValueSeq;
    type SerializeTuple = RValueSeq;
    type SerializeTupleStruct = RValueSeq;
    type SerializeTupleVariant = RValueTupleVariant;
    type SerializeMap = RValueMap;
    type SerializeStruct = RValueStruct;
    type SerializeStructVariant = RValueStructVariant;

    fn serialize_bool(self, v: bool) -> Result<RValue, RSerdeError> {
        Ok(RValue::Logical(vec![Some(v)]))
    }

    fn serialize_i8(self, v: i8) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_i16(self, v: i16) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_i32(self, v: i32) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(i64::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(v))
    }

    fn serialize_u8(self, v: u8) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_u16(self, v: u16) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_u32(self, v: u32) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(i64::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<RValue, RSerdeError> {
        Ok(match i64::try_from(v) {
            Ok(i) => int_or_double(i),
            // Same deliberate widening as `int_or_double`.
            Err(_) => RValue::Double(vec![v as f64]),
        })
    }

    fn serialize_f32(self, v: f32) -> Result<RValue, RSerdeError> {
        Ok(RValue::Double(vec![f64::from(v)]))
    }

    fn serialize_f64(self, v: f64) -> Result<RValue, RSerdeError> {
        Ok(RValue::Double(vec![v]))
    }

    fn serialize_char(self, v: char) -> Result<RValue, RSerdeError> {
        Ok(RValue::Character(vec![Some(v.to_string())]))
    }

    fn serialize_str(self, v: &str) -> Result<RValue, RSerdeError> {
        Ok(RValue::Character(vec![Some(v.to_string())]))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<RValue, RSerdeError> {
        Ok(RValue::Raw(v.to_vec()))
    }

    fn serialize_none(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<RValue, RSerdeError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<RValue, RSerdeError> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<RValue, RSerdeError> {
        match Atomic::from_marker(name) {
            Some(atomic) => atomic.coerce(value.serialize(self)?),
            None => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<RValue, RSerdeError> {
        Ok(tagged(variant, to_rvalue(value)?))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len.unwrap_or(0)))
    }

    fn serialize_tuple(self, len: usize) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<RValueTupleVariant, RSerdeError> {
        Ok(RValueTupleVariant {
            variant,
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<RValueMap, RSerdeError> {
        Ok(RValueMap {
            pending_key: None,
            fields: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<RValueStruct, RSerdeError> {
        Ok(RValueStruct {
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<RValueStructVariant, RSerdeError> {
        Ok(RValueStructVariant {
            variant,
            inner: RValueStruct {
                fields: Vec::with_capacity(len),
            },
        })
    }
}

/// Sequence / tuple accumulator: sequences coalesce homogeneous scalars, tuples
/// always stay lists.
pub struct RValueSeq {
    elements: Vec<RValue>,
}

impl RValueSeq {
    fn with_capacity(n: usize) -> Self {
        RValueSeq {
            elements: Vec::with_capacity(n),
        }
    }
}

impl ser::SerializeSeq for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(coalesce(self.elements))
    }
}

impl ser::SerializeTuple for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(unnamed_list(self.elements))
    }
}

impl ser::SerializeTupleStruct for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(unnamed_list(self.elements))
    }
}

/// `Enum::Variant(a, b)` → `List([("Variant", List([a, b]))])`.
pub struct RValueTupleVariant {
    variant: &'static str,
    elements: Vec<RValue>,
}

impl ser::SerializeTupleVariant for RValueTupleVariant {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(tagged(self.variant, unnamed_list(self.elements)))
    }
}

/// Map accumulator; keys must serialize to a single string.
pub struct RValueMap {
    pending_key: Option<String>,
    fields: Vec<(Option<String>, RValue)>,
}

impl ser::SerializeMap for RValueMap {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        self.pending_key = Some(key_string(to_rvalue(key)?)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| ser::Error::custom("serialize_value called before serialize_key"))?;
        self.fields.push((Some(key), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::List(self.fields))
    }
}

/// Struct accumulator → named list in field order.
pub struct RValueStruct {
    fields: Vec<(Option<String>, RValue)>,
}

impl ser::SerializeStruct for RValueStruct {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((Some(key.to_string()), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::List(self.fields))
    }
}

/// `Enum::Variant { a, b }` → `List([("Variant", List([("a", ..), ("b", ..)]))])`.
pub struct RValueStructVariant {
    variant: &'static str,
    inner: RValueStruct,
}

impl ser::SerializeStructVariant for RValueStructVariant {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        ser::SerializeStruct::serialize_field(&mut self.inner, key, value)
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(tagged(self.variant, ser::SerializeStruct::end(self.inner)?))
    }
}

// endregion

// region: Tagged error parts — variant name + payload fields for serde_error

/// Outcome of [`tagged_parts`]: the variant (if the value carried one) and the
/// payload fields in declaration order.
pub(crate) type TaggedParts = (Option<String>, Vec<(String, RValue)>);

/// Split a serialized error into its variant name and its fields.
///
/// - Externally tagged enums (serde's default) report the variant from the
///   `serialize_*_variant` call: a struct variant's fields are the payload, a
///   newtype variant with a struct payload contributes that struct's fields,
///   any other newtype or tuple payload lands under one field named `value`.
/// - Internally tagged enums (`#[serde(tag = "kind")]`) serialize as a struct
///   whose `tag` field holds the variant; that field is consumed, the others
///   are the payload.
/// - Anything else (a plain struct, a map, a string, a sequence) has no variant;
///   struct and map fields become the payload, scalars and sequences contribute
///   nothing.
pub(crate) fn tagged_parts<E: ?Sized + Serialize>(
    e: &E,
    tag: &str,
) -> Result<TaggedParts, RSerdeError> {
    e.serialize(TaggedSerializer { tag })
}

fn payload_fields(inner: RValue) -> Vec<(String, RValue)> {
    match inner {
        RValue::List(pairs)
            if !pairs.is_empty()
                && pairs
                    .iter()
                    .all(|(n, _)| n.as_deref().is_some_and(|n| !n.is_empty())) =>
        {
            pairs
                .into_iter()
                .map(|(name, value)| (name.unwrap_or_default(), value))
                .collect()
        }
        other => vec![("value".to_string(), other)],
    }
}

fn split_tag(tag: &str, fields: Vec<(String, RValue)>) -> TaggedParts {
    let mut variant = None;
    let mut rest = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        match value {
            RValue::Character(mut v) if variant.is_none() && name == tag && v.len() == 1 => {
                variant = v.pop().flatten();
            }
            value => rest.push((name, value)),
        }
    }
    (variant, rest)
}

struct TaggedSerializer<'a> {
    tag: &'a str,
}

/// Compound accumulator for shapes that carry no variant information.
pub(crate) struct TaggedSwallow;

/// `Enum::Variant(a, b)` → variant plus one `value` field holding the tuple.
pub(crate) struct TaggedTupleVariant {
    variant: &'static str,
    elements: Vec<RValue>,
}

/// A struct or map at the top level: fields, with the tag field split off.
pub(crate) struct TaggedFields<'a> {
    tag: &'a str,
    pending_key: Option<String>,
    fields: Vec<(String, RValue)>,
}

/// `Enum::Variant { a, b }` → variant plus its fields.
pub(crate) struct TaggedStructVariant {
    variant: &'static str,
    fields: Vec<(String, RValue)>,
}

const NO_VARIANT: TaggedParts = (None, Vec::new());

impl<'a> ser::Serializer for TaggedSerializer<'a> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    type SerializeSeq = TaggedSwallow;
    type SerializeTuple = TaggedSwallow;
    type SerializeTupleStruct = TaggedSwallow;
    type SerializeTupleVariant = TaggedTupleVariant;
    type SerializeMap = TaggedFields<'a>;
    type SerializeStruct = TaggedFields<'a>;
    type SerializeStructVariant = TaggedStructVariant;

    fn serialize_bool(self, _: bool) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i8(self, _: i8) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i16(self, _: i16) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i32(self, _: i32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i64(self, _: i64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u8(self, _: u8) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u16(self, _: u16) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u32(self, _: u32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u64(self, _: u64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_f32(self, _: f32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_f64(self, _: f64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_char(self, _: char) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_str(self, _: &str) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_none(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<TaggedParts, RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(variant.to_string()), Vec::new()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<TaggedParts, RSerdeError> {
        // Transparent, like the value serializer: keep intercepting.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(variant.to_string()), payload_fields(to_rvalue(value)?)))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }
    fn serialize_tuple(self, _len: usize) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TaggedTupleVariant, RSerdeError> {
        Ok(TaggedTupleVariant {
            variant,
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<TaggedFields<'a>, RSerdeError> {
        Ok(TaggedFields {
            tag: self.tag,
            pending_key: None,
            fields: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<TaggedFields<'a>, RSerdeError> {
        Ok(TaggedFields {
            tag: self.tag,
            pending_key: None,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TaggedStructVariant, RSerdeError> {
        Ok(TaggedStructVariant {
            variant,
            fields: Vec::with_capacity(len),
        })
    }
}

impl ser::SerializeSeq for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTuple for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTupleStruct for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTupleVariant for TaggedTupleVariant {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok((
            Some(self.variant.to_string()),
            vec![("value".to_string(), unnamed_list(self.elements))],
        ))
    }
}

impl ser::SerializeMap for TaggedFields<'_> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        self.pending_key = Some(key_string(to_rvalue(key)?)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| ser::Error::custom("serialize_value called before serialize_key"))?;
        self.fields.push((key, to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(split_tag(self.tag, self.fields))
    }
}

impl ser::SerializeStruct for TaggedFields<'_> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((key.to_string(), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(split_tag(self.tag, self.fields))
    }
}

impl ser::SerializeStructVariant for TaggedStructVariant {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((key.to_string(), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(self.variant.to_string()), self.fields))
    }
}

// endregion

// region: Tests (pure Rust, no R runtime)

#[cfg(test)]
mod tests;

// endregion
