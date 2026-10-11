//! Fallible tuple output preserves errors through optional and row containers.
mod r_test_utils;

use std::cell::RefCell;
use std::rc::Rc;

use miniextendr_api::into_r_error::IntoRError;
use miniextendr_api::{IntoR, SEXP};

struct Field {
    index: usize,
    refuse: bool,
    visits: Rc<RefCell<Vec<(usize, bool)>>>,
}

impl Field {
    fn convert(self, unchecked: bool) -> Result<SEXP, &'static str> {
        self.visits.borrow_mut().push((self.index, unchecked));
        if self.refuse {
            Err("field refused")
        } else {
            let value = format!("field {}", self.index);
            Ok(if unchecked {
                unsafe { value.into_sexp_unchecked() }
            } else {
                value.into_sexp()
            })
        }
    }
}

impl IntoR for Field {
    type Error = &'static str;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.convert(false)
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.convert(true)
    }
}

// A heterogeneous tuple must accept different error types without requiring
// every custom field error to implement Into<IntoRError>.
struct OtherFailure;

impl IntoR for OtherFailure {
    type Error = std::io::Error;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Err(std::io::Error::other("another refusal"))
    }
}

fn convert<T: IntoR>(value: T, unchecked: bool) -> Result<SEXP, T::Error> {
    if unchecked {
        // The callers all run inside the embedded R thread.
        unsafe { value.try_into_sexp_unchecked() }
    } else {
        value.try_into_sexp()
    }
}

fn assert_error(result: Result<SEXP, IntoRError>, expected: &str) {
    match result.unwrap_err() {
        IntoRError::Inner(message) => assert_eq!(message, expected),
        other => panic!("expected a batched field error, got {other:?}"),
    }
}

#[test]
fn tuple_output_batches_heterogeneous_errors_and_converts_later_fields() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            for optional in [false, true] {
                let visits = Rc::new(RefCell::new(Vec::new()));
                let field = |index, refuse| Field {
                    index,
                    refuse,
                    visits: Rc::clone(&visits),
                };
                let tuple = (
                    field(1, true),
                    "valid".to_string(),
                    OtherFailure,
                    field(4, true),
                    field(5, false),
                );
                let result = if optional {
                    convert(Some(tuple), unchecked)
                } else {
                    convert(tuple, unchecked)
                };
                assert_error(
                    result,
                    "field refused (elements 1, 4); another refusal (element 3)",
                );
                assert_eq!(
                    *visits.borrow(),
                    [(1, unchecked), (4, unchecked), (5, unchecked)]
                );
            }
        }
    });
}

#[test]
fn tuple_row_output_batches_every_row_through_vec_option_and_box() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            for shape in 0..3 {
                let visits = Rc::new(RefCell::new(Vec::new()));
                let field = |index, refuse| Field {
                    index,
                    refuse,
                    visits: Rc::clone(&visits),
                };
                let rows = vec![
                    (field(1, false), field(2, false)),
                    (field(3, true), field(4, false)),
                    (field(5, false), field(6, true)),
                ];
                let result = match shape {
                    0 => convert(rows, unchecked),
                    1 => {
                        let mut rows: Vec<_> = rows.into_iter().map(Some).collect();
                        rows.insert(1, None);
                        convert(rows, unchecked)
                    }
                    _ => convert(rows.into_boxed_slice(), unchecked),
                };
                assert_error(
                    result,
                    if shape == 1 {
                        "field refused (element 1) (element 3); field refused (element 2) (element 4)"
                    } else {
                        "field refused (element 1) (element 2); field refused (element 2) (element 3)"
                    },
                );
                assert_eq!(
                    *visits.borrow(),
                    (1..=6).map(|index| (index, unchecked)).collect::<Vec<_>>()
                );
            }
        }
    });
}
