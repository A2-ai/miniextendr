//! Nested readers let the child schema decide whether zero columns are valid.
mod r_test_utils;

use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{DataFrameRowConvert, FromDataFrame, IntoDataFrame};
use std::fmt::Debug;

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct DynamicChild {
    #[dataframe(expand)]
    values: Vec<String>,
}

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct Parent {
    id: i32,
    #[dataframe(rename = "payload")]
    child: DynamicChild,
}

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct Deep {
    parent: Parent,
}

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct ZeroChild {
    empty: [i32; 0],
}

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct OnlyChild {
    child: ZeroChild,
}

#[derive(Debug, PartialEq, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum Event {
    First { child: DynamicChild },
    Absent,
    Second { child: DynamicChild },
}

fn parents(n: usize, populated: bool) -> Vec<Parent> {
    (0..n)
        .map(|i| Parent {
            id: i32::try_from(i).unwrap(),
            child: DynamicChild {
                values: if populated && i == 1 {
                    vec!["one".into(), "two".into()]
                } else {
                    vec![]
                },
            },
        })
        .collect()
}

fn events() -> Vec<Event> {
    vec![
        Event::Absent,
        Event::First {
            child: DynamicChild { values: vec![] },
        },
        Event::Absent,
        Event::Second {
            child: DynamicChild { values: vec![] },
        },
        Event::First {
            child: DynamicChild { values: vec![] },
        },
    ]
}

fn check<T: DataFrameRowConvert + Debug + PartialEq>(rows: impl Fn() -> Vec<T>, ncol: usize) {
    let expected = rows();
    let frame = rows().into_dataframe().unwrap();
    assert_eq!((frame.nrow(), frame.ncol()), (expected.len(), ncol));
    assert_eq!(Vec::<T>::from_dataframe(&frame).unwrap(), expected);
    #[cfg(feature = "rayon")]
    {
        assert_eq!(Vec::<T>::from_dataframe_par(&frame).unwrap(), expected);
        let frame = rows().into_dataframe_par().unwrap();
        assert_eq!((frame.nrow(), frame.ncol()), (expected.len(), ncol));
        assert_eq!(Vec::<T>::from_dataframe_par(&frame).unwrap(), expected);
    }
}

#[test]
fn struct_readers_accept_empty_dynamic_children_at_any_depth() {
    r_test_utils::with_r_thread(|| {
        for n in [0, 3] {
            for populated in [false, true] {
                let ncol = if populated && n > 1 { 3 } else { 1 };
                check(|| parents(n, populated), ncol);
                check(
                    || {
                        parents(n, populated)
                            .into_iter()
                            .map(|parent| Deep { parent })
                            .collect()
                    },
                    ncol,
                );
            }
        }
    });
}

#[test]
fn column_free_parent_retains_child_row_count() {
    r_test_utils::with_r_thread(|| {
        for n in [0, 2] {
            check(
                || {
                    (0..n)
                        .map(|_| OnlyChild {
                            child: ZeroChild { empty: [] },
                        })
                        .collect()
                },
                0,
            );
        }
    });
}

#[test]
fn enum_readers_densify_empty_children_for_all_contributing_variants() {
    r_test_utils::with_r_thread(|| {
        check(Vec::<Event>::new, 1);
        check(|| vec![Event::Absent], 1);
        check(events, 1);
    });
}

#[derive(Debug, DataFrameRow)]
pub struct Required {
    value: i32,
}

#[derive(Debug, DataFrameRow)]
pub struct RequiredParent {
    id: i32,
    child: Required,
}

#[derive(Debug, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum RequiredEvent {
    Present { child: Required },
    Absent,
}

#[test]
fn missing_required_child_columns_still_report_the_child_schema() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![RequiredParent {
            id: 1,
            child: Required { value: 7 },
        }]
        .into_dataframe()
        .unwrap()
        .select(&["id"]);
        let error = Vec::<RequiredParent>::from_dataframe(&frame)
            .unwrap_err()
            .to_string();
        assert!(error.contains("struct column `child`"), "{error}");
        assert!(error.contains("column `value` is missing"), "{error}");
        let frame = vec![
            RequiredEvent::Present {
                child: Required { value: 7 },
            },
            RequiredEvent::Absent,
        ]
        .into_dataframe()
        .unwrap()
        .select(&["kind"]);
        let error = Vec::<RequiredEvent>::from_dataframe(&frame)
            .unwrap_err()
            .to_string();
        assert!(error.contains("struct column `child`"), "{error}");
        assert!(error.contains("column `value` is missing"), "{error}");
        #[cfg(feature = "rayon")]
        assert!(Vec::<RequiredEvent>::from_dataframe_par(&frame).is_err());
    });
}

#[derive(Debug, DataFrameRow)]
pub struct Unreadable {
    #[dataframe(skip)]
    token: std::rc::Rc<()>,
}

#[derive(Debug, DataFrameRow)]
pub struct UnreadableParent {
    child: Unreadable,
}

#[test]
fn empty_selection_does_not_invent_a_reader_for_skipped_child_fields() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![UnreadableParent {
            child: Unreadable {
                token: std::rc::Rc::new(()),
            },
        }]
        .into_dataframe()
        .unwrap();
        let error = Vec::<UnreadableParent>::from_dataframe(&frame)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("struct column `child`: nested type has no data.frame reader"),
            "{error}"
        );
    });
}
