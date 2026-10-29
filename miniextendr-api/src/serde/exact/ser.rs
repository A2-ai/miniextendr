//! The snapshot encoding's serializer: a value becomes a [`Node`] tree on the
//! Rust side, which [`Node::build`] then makes into R objects.
//!
//! Building the tree first keeps R out of the user's `Serialize` code and
//! lets a sequence of scalars collect into one Rust vector as it arrives, so
//! a long `Vec<f64>` costs one R allocation and no protect-stack entry per
//! element.

use std::borrow::Cow;

use super::super::error::RSerdeError;
use super::{Kind, mark};
use crate::gc_protect::OwnedProtect;
use crate::into_r::{IntoR, alloc_r_vector};
use crate::sys::Rf_allocVector;
use crate::{RLogical, SEXP, SEXPTYPE, SexpExt};
use serde::ser::{self, Serialize};

// region: The tree

/// A value of the encoding, before it is made into R objects. Each variant is
/// one row of the module docs' table.
#[derive(Debug, PartialEq)]
pub(super) enum Node {
    Null,
    Bool(bool),
    Int(i32),
    /// An integer outside `i32`, in decimal.
    Wide(String),
    Real(f64),
    /// A string without a NUL byte.
    Str(String),
    /// The UTF-8 bytes of a string holding a NUL byte.
    NulStr(Vec<u8>),
    Bytes(Vec<u8>),
    Seq(Seq),
    /// A tuple: an unnamed list.
    List(Vec<Node>),
    /// A struct, a map whose keys are all strings, or an enum variant with a
    /// payload: a named list.
    Named(Vec<(Cow<'static, str>, Node)>),
    /// A map whose keys are not all strings.
    Map {
        keys: Seq,
        values: Seq,
    },
    /// `Some(x)` where `x` is `Null` or `Some`.
    Some(Box<Node>),
}

/// A sequence: one atomic vector while its elements are scalars of one kind,
/// a list otherwise.
#[derive(Debug, PartialEq)]
pub(super) enum Seq {
    /// The empty sequence, or one whose elements are not scalars of one kind.
    List(Vec<Node>),
    Bools(Vec<bool>),
    Ints(Vec<i32>),
    Reals(Vec<f64>),
    Strs(Vec<String>),
    /// Integers, at least one outside `i32`, in decimal.
    Wides(Vec<String>),
}

impl Seq {
    fn new() -> Self {
        Seq::List(Vec::new())
    }

    /// The sequence of `nodes`, in order.
    pub(super) fn of(nodes: Vec<Node>) -> Self {
        let mut seq = Seq::new();
        let capacity = nodes.len();
        for node in nodes {
            seq.push(node, capacity);
        }
        seq
    }

    /// Appends `node`; `capacity` sizes the vector the first element starts.
    fn push(&mut self, node: Node, capacity: usize) {
        let node = match (&mut *self, node) {
            (Seq::Bools(v), Node::Bool(x)) => return v.push(x),
            (Seq::Ints(v), Node::Int(x)) => return v.push(x),
            (Seq::Reals(v), Node::Real(x)) => return v.push(x),
            (Seq::Strs(v), Node::Str(x)) => return v.push(x),
            (Seq::Wides(v), Node::Wide(x)) => return v.push(x),
            (Seq::Wides(v), Node::Int(x)) => return v.push(x.to_string()),
            (Seq::List(v), node) if !v.is_empty() => return v.push(node),
            (_, node) => node,
        };
        *self = match (std::mem::replace(self, Seq::new()), node) {
            (Seq::List(_), node) => Seq::first(node, capacity),
            (Seq::Ints(v), Node::Wide(x)) => {
                let mut wides: Vec<String> = Vec::with_capacity(capacity.max(v.len() + 1));
                wides.extend(v.iter().map(i32::to_string));
                wides.push(x);
                Seq::Wides(wides)
            }
            (seq, node) => {
                let mut nodes = seq.into_nodes();
                nodes.push(node);
                Seq::List(nodes)
            }
        };
    }

    /// The sequence that starts with `node`.
    fn first(node: Node, capacity: usize) -> Self {
        fn one<T>(x: T, capacity: usize) -> Vec<T> {
            let mut v = Vec::with_capacity(capacity.max(1));
            v.push(x);
            v
        }
        match node {
            Node::Bool(x) => Seq::Bools(one(x, capacity)),
            Node::Int(x) => Seq::Ints(one(x, capacity)),
            Node::Real(x) => Seq::Reals(one(x, capacity)),
            Node::Str(x) => Seq::Strs(one(x, capacity)),
            Node::Wide(x) => Seq::Wides(one(x, capacity)),
            node => Seq::List(one(node, capacity)),
        }
    }

    fn into_nodes(self) -> Vec<Node> {
        match self {
            Seq::List(v) => v,
            Seq::Bools(v) => v.into_iter().map(Node::Bool).collect(),
            Seq::Ints(v) => v.into_iter().map(Node::Int).collect(),
            Seq::Reals(v) => v.into_iter().map(Node::Real).collect(),
            Seq::Strs(v) => v.into_iter().map(Node::Str).collect(),
            Seq::Wides(v) => v.into_iter().map(integer_node).collect(),
        }
    }
}

/// An integer of `Seq::Wides` as a node of its own.
fn integer_node(decimal: String) -> Node {
    match decimal.parse::<i32>() {
        Ok(x) => Node::Int(x),
        Err(_) => Node::Wide(decimal),
    }
}

fn int(v: i128) -> Node {
    i32::try_from(v).map_or_else(|_| Node::Wide(v.to_string()), Node::Int)
}

fn uint(v: u128) -> Node {
    i32::try_from(v).map_or_else(|_| Node::Wide(v.to_string()), Node::Int)
}

fn string(s: &str) -> Node {
    if s.contains('\0') {
        Node::NulStr(s.as_bytes().to_vec())
    } else {
        Node::Str(s.to_owned())
    }
}

/// A field or variant name, which becomes an R name: a CHARSXP can't hold a
/// NUL byte.
fn name(name: &'static str) -> Result<Cow<'static, str>, RSerdeError> {
    if name.contains('\0') {
        return Err(RSerdeError::Message(format!(
            "the name {name:?} holds a NUL byte, which an R name can't"
        )));
    }
    Ok(Cow::Borrowed(name))
}

fn tagged(variant: &'static str, payload: Node) -> Result<Node, RSerdeError> {
    Ok(Node::Named(vec![(name(variant)?, payload)]))
}

// endregion

// region: Making the R objects

impl Node {
    /// The R object of this node, unprotected. Each child is set into its
    /// parent as soon as it is made, so only the path to the node being made
    /// is on R's protect stack.
    pub(super) fn build(self) -> SEXP {
        match self {
            Node::Null => SEXP::nil(),
            Node::Bool(x) => SEXP::scalar_logical(x),
            Node::Int(x) => SEXP::scalar_integer(x),
            Node::Wide(x) => mark(SEXP::scalar_string_from_str(&x), Kind::Int),
            Node::Real(x) => SEXP::scalar_real(x),
            Node::Str(x) => SEXP::scalar_string_from_str(&x),
            Node::NulStr(x) => mark(x.as_slice().into_sexp(), Kind::Str),
            Node::Bytes(x) => x.as_slice().into_sexp(),
            Node::Seq(seq) => seq.build(),
            Node::List(nodes) => list(nodes),
            Node::Named(fields) => named(fields),
            Node::Map { keys, values } => mark(
                named(vec![
                    (Cow::Borrowed("keys"), Node::Seq(keys)),
                    (Cow::Borrowed("values"), Node::Seq(values)),
                ]),
                Kind::Map,
            ),
            Node::Some(inner) => mark(list(vec![*inner]), Kind::Some),
        }
    }
}

impl Seq {
    fn build(self) -> SEXP {
        match self {
            Seq::List(nodes) => list(nodes),
            Seq::Bools(v) => mark(atomic(v.into_iter().map(RLogical::from)), Kind::Seq),
            Seq::Ints(v) => mark(atomic(v), Kind::Seq),
            Seq::Reals(v) => mark(atomic(v), Kind::Seq),
            Seq::Strs(v) => mark(strings(&v), Kind::Seq),
            Seq::Wides(v) => mark(strings(&v), Kind::Ints),
        }
    }
}

fn r_len(n: usize) -> isize {
    n.try_into().expect("length exceeds isize::MAX")
}

/// An atomic vector of `values`.
fn atomic<T: crate::RNativeType>(values: impl IntoIterator<Item = T>) -> SEXP {
    let values: Vec<T> = values.into_iter().collect();
    // SAFETY: R's main thread; filling the slice allocates nothing.
    let (sexp, slice) = unsafe { alloc_r_vector::<T>(values.len()) };
    slice.copy_from_slice(&values);
    sexp
}

/// A character vector of `values`, none holding a NUL byte.
fn strings(values: &[String]) -> SEXP {
    // SAFETY: R's main thread; each CHARSXP is set as soon as it is made.
    let out = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, r_len(values.len()))) };
    for (i, s) in values.iter().enumerate() {
        out.get().set_string_elt(r_len(i), SEXP::charsxp(s));
    }
    out.get()
}

fn list(nodes: Vec<Node>) -> SEXP {
    // SAFETY: R's main thread; each child is set as soon as it is made.
    let out = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, r_len(nodes.len()))) };
    for (i, node) in nodes.into_iter().enumerate() {
        out.get().set_vector_elt(r_len(i), node.build());
    }
    out.get()
}

fn named(fields: Vec<(Cow<'static, str>, Node)>) -> SEXP {
    let n = r_len(fields.len());
    // SAFETY: R's main thread; both vectors are rooted, and each child is set
    // as soon as it is made.
    let (out, names) = unsafe {
        (
            OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n)),
            OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n)),
        )
    };
    for (i, (key, node)) in fields.into_iter().enumerate() {
        names.get().set_string_elt(r_len(i), SEXP::charsxp(&key));
        out.get().set_vector_elt(r_len(i), node.build());
    }
    out.get().set_names(names.get());
    out.get()
}

// endregion

// region: serde::Serializer

/// Serializes a value into a [`Node`].
pub(super) struct NodeSerializer;

impl ser::Serializer for NodeSerializer {
    type Ok = Node;
    type Error = RSerdeError;

    type SerializeSeq = SeqNode;
    type SerializeTuple = ListNode;
    type SerializeTupleStruct = ListNode;
    type SerializeTupleVariant = TupleVariantNode;
    type SerializeMap = MapNode;
    type SerializeStruct = StructNode;
    type SerializeStructVariant = StructVariantNode;

    fn serialize_bool(self, v: bool) -> Result<Node, RSerdeError> {
        Ok(Node::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Node, RSerdeError> {
        Ok(Node::Int(i32::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Node, RSerdeError> {
        Ok(Node::Int(i32::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Node, RSerdeError> {
        Ok(Node::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Node, RSerdeError> {
        Ok(int(i128::from(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Node, RSerdeError> {
        Ok(int(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Node, RSerdeError> {
        Ok(Node::Int(i32::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Node, RSerdeError> {
        Ok(Node::Int(i32::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Node, RSerdeError> {
        Ok(uint(u128::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Node, RSerdeError> {
        Ok(uint(u128::from(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<Node, RSerdeError> {
        Ok(uint(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Node, RSerdeError> {
        Ok(Node::Real(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Node, RSerdeError> {
        Ok(Node::Real(v))
    }

    fn serialize_char(self, v: char) -> Result<Node, RSerdeError> {
        Ok(string(v.encode_utf8(&mut [0u8; 4])))
    }

    fn serialize_str(self, v: &str) -> Result<Node, RSerdeError> {
        Ok(string(v))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Node, RSerdeError> {
        Ok(Node::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Node, RSerdeError> {
        Ok(Node::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Node, RSerdeError> {
        Ok(match value.serialize(self)? {
            // `Some(None)`, `Some(())`, `Some(Some(None))`, ...: the inner
            // value alone would read back as `None`.
            node @ (Node::Null | Node::Some(_)) => Node::Some(Box::new(node)),
            node => node,
        })
    }

    fn serialize_unit(self) -> Result<Node, RSerdeError> {
        Ok(Node::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Node, RSerdeError> {
        Ok(Node::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Node, RSerdeError> {
        Ok(Node::Str(name(variant)?.into_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Node, RSerdeError> {
        // Transparent, the `as_*` helpers' markers included: they choose an R
        // type for reading in R, which a snapshot doesn't need.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Node, RSerdeError> {
        tagged(variant, value.serialize(NodeSerializer)?)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqNode, RSerdeError> {
        Ok(SeqNode {
            seq: Seq::new(),
            capacity: len.unwrap_or(0),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<ListNode, RSerdeError> {
        Ok(ListNode(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<ListNode, RSerdeError> {
        Ok(ListNode(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantNode, RSerdeError> {
        Ok(TupleVariantNode {
            variant,
            elements: ListNode(Vec::with_capacity(len)),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<MapNode, RSerdeError> {
        let capacity = len.unwrap_or(0);
        Ok(MapNode {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<StructNode, RSerdeError> {
        Ok(StructNode(Vec::with_capacity(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructVariantNode, RSerdeError> {
        Ok(StructVariantNode {
            variant,
            fields: StructNode(Vec::with_capacity(len)),
        })
    }
}

/// A sequence, collected as it arrives.
pub(super) struct SeqNode {
    seq: Seq,
    capacity: usize,
}

impl ser::SerializeSeq for SeqNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.seq
            .push(value.serialize(NodeSerializer)?, self.capacity);
        Ok(())
    }

    fn end(self) -> Result<Node, RSerdeError> {
        Ok(Node::Seq(self.seq))
    }
}

/// A tuple or tuple struct: always a list.
pub(super) struct ListNode(Vec<Node>);

impl ser::SerializeTuple for ListNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.0.push(value.serialize(NodeSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Node, RSerdeError> {
        Ok(Node::List(self.0))
    }
}

impl ser::SerializeTupleStruct for ListNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Node, RSerdeError> {
        ser::SerializeTuple::end(self)
    }
}

/// `Enum::Variant(a, b)`: `list(Variant = list(a, b))`.
pub(super) struct TupleVariantNode {
    variant: &'static str,
    elements: ListNode,
}

impl ser::SerializeTupleVariant for TupleVariantNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        ser::SerializeTuple::serialize_element(&mut self.elements, value)
    }

    fn end(self) -> Result<Node, RSerdeError> {
        tagged(self.variant, Node::List(self.elements.0))
    }
}

/// A map: a named list when every key is a string, else its keys and values.
pub(super) struct MapNode {
    keys: Vec<Node>,
    values: Vec<Node>,
}

impl ser::SerializeMap for MapNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        self.keys.push(key.serialize(NodeSerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.values.push(value.serialize(NodeSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Node, RSerdeError> {
        if self.keys.len() != self.values.len() {
            return Err(ser::Error::custom("a map key without a value"));
        }
        if self.keys.iter().all(|key| matches!(key, Node::Str(_))) {
            let fields = self.keys.into_iter().zip(self.values).map(|(key, value)| {
                let Node::Str(key) = key else {
                    unreachable!("every key is a string")
                };
                (Cow::Owned(key), value)
            });
            return Ok(Node::Named(fields.collect()));
        }
        Ok(Node::Map {
            keys: Seq::of(self.keys),
            values: Seq::of(self.values),
        })
    }
}

/// A struct: a named list in field order.
pub(super) struct StructNode(Vec<(Cow<'static, str>, Node)>);

impl ser::SerializeStruct for StructNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.0.push((name(key)?, value.serialize(NodeSerializer)?));
        Ok(())
    }

    fn end(self) -> Result<Node, RSerdeError> {
        Ok(Node::Named(self.0))
    }
}

/// `Enum::Variant { a, b }`: `list(Variant = list(a = .., b = ..))`.
pub(super) struct StructVariantNode {
    variant: &'static str,
    fields: StructNode,
}

impl ser::SerializeStructVariant for StructVariantNode {
    type Ok = Node;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        ser::SerializeStruct::serialize_field(&mut self.fields, key, value)
    }

    fn end(self) -> Result<Node, RSerdeError> {
        tagged(self.variant, Node::Named(self.fields.0))
    }
}

// endregion
