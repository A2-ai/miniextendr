//! The snapshot encoding's deserializer. Every visit is decided from the R
//! value alone (the module docs' `deserialize_any` table), so a value reads
//! back the same through serde's buffered `Content` as directly.

use super::super::error::RSerdeError;
use super::Kind;
use crate::from_r::charsxp_to_str;
use crate::{SEXP, SEXPTYPE, SexpExt};
use serde::de::{
    self, DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

/// Reads a value of the snapshot encoding.
pub(super) struct ExactDeserializer {
    sexp: SEXP,
}

impl ExactDeserializer {
    pub(super) fn new(sexp: SEXP) -> Self {
        ExactDeserializer { sexp }
    }

    fn kind(&self) -> Result<Option<Kind>, RSerdeError> {
        Kind::of(self.sexp)
    }

    fn has_names(&self) -> bool {
        !self.sexp.get_names().is_nil()
    }
}

/// The error for an R value this encoding doesn't write.
fn not_encoded(sexp: SEXP, kind: Option<Kind>) -> RSerdeError {
    let marker = kind.map_or_else(String::new, |kind| {
        format!(" with `serde = {:?}`", kind.name())
    });
    RSerdeError::Message(format!(
        "not a value of the snapshot encoding: {} of length {}{marker}",
        crate::typed_list::sexptype_name(sexp.type_of()),
        sexp.len(),
    ))
}

/// Element `i` of the atomic vector `sexp` as a scalar: `visit_bool`,
/// `visit_i32`, `visit_f64` or `visit_str`. The encoding writes no `NA`; an
/// `integer` `NA` is `i32::MIN` and a `double` keeps its bits, but a
/// logical or string `NA` has no value to give.
fn visit_scalar<'de, V: Visitor<'de>>(
    sexp: SEXP,
    i: isize,
    visitor: V,
) -> Result<V::Value, RSerdeError> {
    match sexp.type_of() {
        SEXPTYPE::LGLSXP => match sexp.logical_elt(i) {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(RSerdeError::UnexpectedNa),
        },
        SEXPTYPE::INTSXP => visitor.visit_i32(sexp.integer_elt(i)),
        SEXPTYPE::REALSXP => visitor.visit_f64(sexp.real_elt(i)),
        SEXPTYPE::STRSXP => visitor.visit_str(string_elt(sexp, i)?),
        _ => Err(not_encoded(sexp, None)),
    }
}

/// Element `i` of the character vector `sexp`.
fn string_elt(sexp: SEXP, i: isize) -> Result<&'static str, RSerdeError> {
    let charsxp = sexp.string_elt(i);
    if charsxp == SEXP::na_string() {
        return Err(RSerdeError::UnexpectedNa);
    }
    // SAFETY: a CHARSXP other than `NA_character_`, rooted by the input.
    Ok(unsafe { charsxp_to_str(charsxp) })
}

/// An integer written in decimal: visited as the first of `i64`, `u64`,
/// `i128`, `u128` that holds it, as serde's own formats do (and `Content`
/// keeps `i64` and `u64`).
fn visit_integer<'de, V: Visitor<'de>>(decimal: &str, visitor: V) -> Result<V::Value, RSerdeError> {
    if let Ok(v) = decimal.parse::<i64>() {
        visitor.visit_i64(v)
    } else if let Ok(v) = decimal.parse::<u64>() {
        visitor.visit_u64(v)
    } else if let Ok(v) = decimal.parse::<i128>() {
        visitor.visit_i128(v)
    } else if let Ok(v) = decimal.parse::<u128>() {
        visitor.visit_u128(v)
    } else {
        Err(RSerdeError::Message(format!(
            "not a value of the snapshot encoding: {decimal:?} is not an integer"
        )))
    }
}

/// The string a `serde = "str"` raw vector holds.
fn nul_string(sexp: SEXP) -> Result<String, RSerdeError> {
    // SAFETY: a raw vector, rooted by the input.
    let bytes = unsafe { sexp.as_slice::<u8>() };
    String::from_utf8(bytes.to_vec()).map_err(|_| RSerdeError::InvalidUtf8)
}

impl<'de> Deserializer<'de> for ExactDeserializer {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        let sexp = self.sexp;
        if sexp.is_nil() {
            return visitor.visit_unit();
        }
        let kind = self.kind()?;
        let len = sexp.len();
        match (sexp.type_of(), kind) {
            (SEXPTYPE::LGLSXP | SEXPTYPE::INTSXP | SEXPTYPE::REALSXP | SEXPTYPE::STRSXP, None)
                if len == 1 =>
            {
                visit_scalar(sexp, 0, visitor)
            }
            // SAFETY: a raw vector, rooted by the input.
            (SEXPTYPE::RAWSXP, None) => visitor.visit_bytes(unsafe { sexp.as_slice::<u8>() }),
            (SEXPTYPE::STRSXP, Some(Kind::Int)) if len == 1 => {
                visit_integer(string_elt(sexp, 0)?, visitor)
            }
            (SEXPTYPE::RAWSXP, Some(Kind::Str)) => visitor.visit_string(nul_string(sexp)?),
            (SEXPTYPE::VECSXP, None) if self.has_names() => {
                visitor.visit_map(NamedAccess::new(sexp)?)
            }
            (SEXPTYPE::VECSXP, Some(Kind::Map)) => visitor.visit_map(PairAccess::new(sexp)?),
            (SEXPTYPE::VECSXP, Some(Kind::Some)) if len == 1 => {
                visitor.visit_some(ExactDeserializer::new(sexp.vector_elt(0)))
            }
            _ => match Items::of(sexp, kind) {
                Some(items) => visitor.visit_seq(items),
                None => Err(not_encoded(sexp, kind)),
            },
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        if self.sexp.is_nil() {
            return visitor.visit_none();
        }
        if self.kind()? == Some(Kind::Some) && self.sexp.len() == 1 {
            return visitor.visit_some(ExactDeserializer::new(self.sexp.vector_elt(0)));
        }
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        let sexp = self.sexp;
        let kind = if sexp.is_nil() { None } else { self.kind()? };
        match (sexp.type_of(), kind) {
            // A unit variant: its name.
            (SEXPTYPE::STRSXP, None) if sexp.len() == 1 => {
                let variant: de::value::StrDeserializer<'_, RSerdeError> =
                    string_elt(sexp, 0)?.into_deserializer();
                variant.deserialize_enum(name, variants, visitor)
            }
            // A variant with a payload: `list(Variant = payload)`.
            (SEXPTYPE::VECSXP, None) if sexp.len() == 1 && self.has_names() => {
                visitor.visit_enum(VariantPayload {
                    variant: string_elt(sexp.get_names(), 0)?,
                    payload: sexp.vector_elt(0),
                })
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "an enum variant: character(1) or list(<variant> = <payload>)",
                actual: not_encoded(sexp, kind).to_string(),
            }),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
    }
}

// region: Elements of a sequence

/// Element `index` of an atomic vector that is a sequence (`serde = "seq"`),
/// or a name of a named list: a scalar.
struct Element {
    sexp: SEXP,
    index: isize,
}

impl<'de> Deserializer<'de> for Element {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visit_scalar(self.sexp, self.index, visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        if self.sexp.type_of() != SEXPTYPE::STRSXP {
            return self.deserialize_any(visitor);
        }
        // A unit variant (a map key, or an element of a sequence of them).
        let variant: de::value::StrDeserializer<'_, RSerdeError> =
            string_elt(self.sexp, self.index)?.into_deserializer();
        variant.deserialize_enum(name, variants, visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
    }
}

/// One integer of a `serde = "ints"` sequence.
struct Integer(&'static str);

impl<'de> Deserializer<'de> for Integer {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visit_integer(self.0, visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, RSerdeError> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct enum identifier
    }
}

/// How the elements of a sequence are stored.
#[derive(Clone, Copy)]
enum Layout {
    /// An unnamed list: each element a value of its own.
    List,
    /// An atomic vector (`serde = "seq"`): each element a scalar.
    Atomic,
    /// A character vector of integers (`serde = "ints"`).
    Integers,
}

/// The elements of a sequence, in order.
struct Items {
    sexp: SEXP,
    layout: Layout,
    index: isize,
    len: isize,
}

impl Items {
    /// The elements of `sexp`, whose `serde` attribute is `kind`, when it is
    /// a sequence.
    fn of(sexp: SEXP, kind: Option<Kind>) -> Option<Self> {
        let layout = match (sexp.type_of(), kind) {
            (SEXPTYPE::VECSXP, None) if sexp.get_names().is_nil() => Layout::List,
            (
                SEXPTYPE::LGLSXP | SEXPTYPE::INTSXP | SEXPTYPE::REALSXP | SEXPTYPE::STRSXP,
                Some(Kind::Seq),
            ) => Layout::Atomic,
            (SEXPTYPE::STRSXP, Some(Kind::Ints)) => Layout::Integers,
            _ => return None,
        };
        Some(Items {
            sexp,
            layout,
            index: 0,
            len: sexp.xlength(),
        })
    }

    /// The elements of a map's keys or values.
    fn of_part(sexp: SEXP) -> Result<Self, RSerdeError> {
        let kind = if sexp.is_nil() { None } else { Kind::of(sexp)? };
        Items::of(sexp, kind).ok_or_else(|| not_encoded(sexp, kind))
    }
}

impl<'de> SeqAccess<'de> for Items {
    type Error = RSerdeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, RSerdeError> {
        if self.index >= self.len {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        match self.layout {
            Layout::List => seed.deserialize(ExactDeserializer::new(self.sexp.vector_elt(index))),
            Layout::Atomic => seed.deserialize(Element {
                sexp: self.sexp,
                index,
            }),
            Layout::Integers => seed.deserialize(Integer(string_elt(self.sexp, index)?)),
        }
        .map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        usize::try_from(self.len - self.index).ok()
    }
}

// endregion

// region: Maps

/// A named list: a struct, or a map whose keys are all strings.
struct NamedAccess {
    names: SEXP,
    values: Items,
}

impl NamedAccess {
    fn new(sexp: SEXP) -> Result<Self, RSerdeError> {
        let names = sexp.get_names();
        if names.type_of() != SEXPTYPE::STRSXP || names.xlength() != sexp.xlength() {
            return Err(not_encoded(sexp, None));
        }
        Ok(NamedAccess {
            names,
            values: Items {
                sexp,
                layout: Layout::List,
                index: 0,
                len: sexp.xlength(),
            },
        })
    }
}

impl<'de> MapAccess<'de> for NamedAccess {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, RSerdeError> {
        if self.values.index >= self.values.len {
            return Ok(None);
        }
        seed.deserialize(Element {
            sexp: self.names,
            index: self.values.index,
        })
        .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, RSerdeError> {
        self.values
            .next_element_seed(seed)?
            .ok_or_else(|| de::Error::custom("a map value without a key"))
    }

    fn size_hint(&self) -> Option<usize> {
        self.values.size_hint()
    }
}

/// `list(keys, values)` (`serde = "map"`): a map whose keys are not all
/// strings.
struct PairAccess {
    keys: Items,
    values: Items,
}

impl PairAccess {
    fn new(sexp: SEXP) -> Result<Self, RSerdeError> {
        if sexp.len() != 2 {
            return Err(not_encoded(sexp, Some(Kind::Map)));
        }
        let keys = Items::of_part(sexp.vector_elt(0))?;
        let values = Items::of_part(sexp.vector_elt(1))?;
        if keys.len != values.len {
            return Err(RSerdeError::LengthMismatch {
                expected: usize::try_from(keys.len).unwrap_or(usize::MAX),
                actual: usize::try_from(values.len).unwrap_or(usize::MAX),
            });
        }
        Ok(PairAccess { keys, values })
    }
}

impl<'de> MapAccess<'de> for PairAccess {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, RSerdeError> {
        self.keys.next_element_seed(seed)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, RSerdeError> {
        self.values
            .next_element_seed(seed)?
            .ok_or_else(|| de::Error::custom("a map value without a key"))
    }

    fn size_hint(&self) -> Option<usize> {
        self.keys.size_hint()
    }
}

// endregion

// region: Enum variants with a payload

/// `list(Variant = payload)`.
struct VariantPayload {
    variant: &'static str,
    payload: SEXP,
}

impl<'de> EnumAccess<'de> for VariantPayload {
    type Error = RSerdeError;
    type Variant = ExactDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, ExactDeserializer), RSerdeError> {
        let variant: de::value::StrDeserializer<'_, RSerdeError> = self.variant.into_deserializer();
        Ok((
            seed.deserialize(variant)?,
            ExactDeserializer::new(self.payload),
        ))
    }
}

impl<'de> VariantAccess<'de> for ExactDeserializer {
    type Error = RSerdeError;

    fn unit_variant(self) -> Result<(), RSerdeError> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, RSerdeError> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        self.deserialize_any(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, RSerdeError> {
        self.deserialize_any(visitor)
    }
}

// endregion
