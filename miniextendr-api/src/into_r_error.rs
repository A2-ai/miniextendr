//! Error types for fallible [`IntoR`](crate::into_r::IntoR) conversions.
//!
//! [`IntoRError`] is returned by `try_into_sexp` when conversion fails
//! (e.g., string exceeds R's `i32` length limit, or fields of a tuple fail).

use crate::RValue;
use crate::condition::{ConditionCall, ConditionData, ErrParts, RConditionData};
use crate::from_r::BatchedErrors;

/// Error returned by [`IntoR::try_into_sexp`](crate::into_r::IntoR::try_into_sexp)
/// for types whose conversion to R can fail.
///
/// # Variants
///
/// - `StringTooLong`: a Rust string exceeds R's `i32` length limit (~2 GB)
/// - `LengthOverflow`: a collection length exceeds R's `R_xlen_t` capacity
/// - `Elements`: elements of a container failed to convert (a tuple's
///   fields, the rows of a `Vec` of tuples), each with its own condition
///
/// It implements [`RConditionData`]. Returned from a `#[miniextendr]`
/// function, an `Elements` error raises the batched message, the class every
/// failing element shares (else none beyond `rust_error`) and `e$errors`,
/// one entry per failing element: see [`FailedElements`].
#[derive(Debug, Clone)]
pub enum IntoRError {
    /// A string's byte length exceeds `i32::MAX`.
    StringTooLong {
        /// Actual byte length of the string.
        len: usize,
    },
    /// A collection's element count exceeds the target R vector capacity.
    LengthOverflow {
        /// Actual element count.
        len: usize,
    },
    /// Elements of a container failed to convert.
    Elements(FailedElements),
}

impl std::fmt::Display for IntoRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntoRError::StringTooLong { len } => {
                write!(
                    f,
                    "string byte length {} exceeds R's i32 limit ({})",
                    len,
                    i32::MAX
                )
            }
            IntoRError::LengthOverflow { len } => {
                write!(f, "collection length {} overflows R vector capacity", len)
            }
            IntoRError::Elements(failed) => f.write_str(&failed.message),
        }
    }
}

impl std::error::Error for IntoRError {}

/// The condition an `IntoRError` raises in R: its message; for `Elements`
/// the shared class of the failing elements and their conditions as
/// `e$errors`.
impl RConditionData for IntoRError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        match self {
            IntoRError::Elements(failed) => failed.class.clone(),
            IntoRError::StringTooLong { .. } | IntoRError::LengthOverflow { .. } => Vec::new(),
        }
    }

    fn data(&self) -> Option<ConditionData> {
        match self {
            IntoRError::Elements(failed) => Some(vec![(
                crate::condition::ERRORS_FIELD.to_owned(),
                failed.errors_rvalue(),
            )]),
            IntoRError::StringTooLong { .. } | IntoRError::LengthOverflow { .. } => None,
        }
    }

    /// The `.mx_errors` field of `Elements` is the framework's own.
    fn __mx_keeps_internal_fields(&self) -> bool {
        true
    }
}

/// The elements of a container whose conversion to R failed (a tuple's
/// fields, the rows of a `Vec` of tuples, the elements of an `AsJsonVec`):
/// the batched message, the class
/// they share and each failing element's own condition with its position.
///
/// Raised in R (`kind = "result_err"`):
///
/// - the message names every failing position, in the grammar of every
///   batched conversion: `map keys must be strings for R named lists
///   (element 2)`;
/// - the class vector is the one every failing element has, when they all
///   have the same, so a handler for the element's class catches the
///   container as it catches the element alone; when they differ (or have
///   none) the error has only the `rust_error` classes;
/// - `e$errors` is a list with one entry per failing element,
///   `list(index = , condition = )`: `index` is its position, numbered as R
///   counts (`2L` for a tuple's second field, `c(3L, 2L)` for the second
///   field of a `Vec` of tuples' third row), and `condition` the condition
///   the element raises as a return value of its own, with its classes,
///   message and fields.
///
/// Like the message, `e$errors` lists the first ten failures; past them the
/// message ends `; and N more` and those failures have no entry. The shared
/// class is decided over every failure.
#[derive(Debug, Clone)]
pub struct FailedElements {
    message: String,
    class: Vec<String>,
    elements: Vec<FailedElement>,
}

impl FailedElements {
    /// The batched message, naming every failing position.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The classes every failing element shares; empty when they differ.
    pub fn class(&self) -> &[String] {
        &self.class
    }

    /// The listed failing elements, in the order they failed.
    pub fn elements(&self) -> &[FailedElement] {
        &self.elements
    }

    /// `e$errors` as the tagged values the R raise helper builds into
    /// `list(index = , condition = )` entries.
    fn errors_rvalue(&self) -> RValue {
        RValue::List(
            self.elements
                .iter()
                .map(|element| (None, element.entry_rvalue()))
                .collect(),
        )
    }
}

/// One failing element of a [`FailedElements`]: its position and the parts
/// of its condition.
#[derive(Debug, Clone)]
pub struct FailedElement {
    /// Position path, numbered as R counts (1-based).
    index: Vec<usize>,
    message: String,
    class: Vec<String>,
    data: Option<ConditionData>,
    call: ConditionCall,
}

impl FailedElement {
    fn new(index: usize, parts: ErrParts) -> Self {
        FailedElement {
            index: vec![index + 1],
            message: parts.message,
            class: parts.class,
            data: parts.data,
            call: parts.call,
        }
    }

    /// The element's position, numbered as R counts: one number for a field
    /// or an element, the row then the field for a `Vec` of tuples.
    pub fn index(&self) -> &[usize] {
        &self.index
    }

    /// The element's own message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The element's own classes.
    pub fn class(&self) -> &[String] {
        &self.class
    }

    /// The element's own fields.
    pub fn data(&self) -> Option<&ConditionData> {
        self.data.as_ref()
    }

    /// `list(index = , condition = <tagged value>)`.
    fn entry_rvalue(&self) -> RValue {
        let parts = ErrParts {
            message: self.message.clone(),
            class: self.class.clone(),
            data: self.data.clone(),
            call: self.call,
        };
        RValue::List(vec![
            (Some("index".to_owned()), index_rvalue(&self.index)),
            (
                Some("condition".to_owned()),
                crate::condition::tagged_rvalue(crate::error_value::kind::RESULT_ERR, parts),
            ),
        ])
    }
}

/// A position path as R numbers it: an integer vector, or a double vector
/// for a position past R's integer range (a row of a long list).
fn index_rvalue(index: &[usize]) -> RValue {
    match index
        .iter()
        .map(|&position| i32::try_from(position).map(Some))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(positions) => RValue::Integer(positions),
        // Positions of an R long vector stay below 2^52, so a double holds
        // them exactly.
        Err(_) => RValue::Double(index.iter().map(|&position| position as f64).collect()),
    }
}

/// The failures of a container's elements, folded into one
/// [`IntoRError::Elements`].
///
/// Each failure's message goes into a [`BatchedErrors`], so the message is
/// the one every batched conversion writes; the listed failures (the first
/// [`BATCHED_ERROR_CAP`](crate::from_r::BATCHED_ERROR_CAP)) keep their
/// condition parts for `e$errors`; every failure takes part in the shared
/// class.
#[derive(Default)]
pub(crate) struct ElementFailures {
    batch: BatchedErrors,
    elements: Vec<FailedElement>,
    /// The class every failure so far has; `None` before the first, empty
    /// once two differ.
    class: Option<Vec<String>>,
}

impl ElementFailures {
    /// Record the failure of the element at 0-based `index`.
    #[track_caller]
    pub(crate) fn push<E: RConditionData + ?Sized>(&mut self, index: usize, error: &E) {
        self.share(error.class());
        let elements = &mut self.elements;
        self.batch.push(index, || {
            let parts = crate::condition::into_r_err_parts(error);
            let reason = parts.message.clone();
            elements.push(FailedElement::new(index, parts));
            reason
        });
    }

    /// Record the failure of the row at 0-based `row`, itself a container
    /// (a tuple): its failed elements are listed with the row in front of
    /// their positions, its message is the row's reason.
    #[track_caller]
    pub(crate) fn push_row(&mut self, row: usize, error: IntoRError) {
        let IntoRError::Elements(failed) = error else {
            return self.push(row, &error);
        };
        self.share(failed.class);
        let elements = &mut self.elements;
        self.batch.push(row, || {
            elements.extend(failed.elements.into_iter().map(|mut element| {
                element.index.insert(0, row + 1);
                element
            }));
            failed.message
        });
    }

    /// Fold one failure's classes into the shared class.
    fn share(&mut self, class: Vec<String>) {
        match &mut self.class {
            None => self.class = Some(class),
            Some(shared) if *shared != class => shared.clear(),
            Some(_) => {}
        }
    }

    /// Whether any failure has been recorded.
    pub(crate) fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    /// The one error for every failure recorded.
    pub(crate) fn into_error(self) -> IntoRError {
        IntoRError::Elements(FailedElements {
            message: self.batch.element_message(),
            class: self.class.unwrap_or_default(),
            elements: self.elements,
        })
    }
}

#[cfg(test)]
mod tests;
