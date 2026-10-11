//! Owned list construction: growth, both finishes, and GC/drop-order invariants.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::{
    DataFrame, DataFrameError, IntoList, IntoR, NamedListBuilder, OwnedProtect, RootedSentinel,
    SEXP, SEXPTYPE, SexpExt, TryFromSexp, sys,
};

fn with_gctorture(f: impl FnOnce()) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            unsafe { r_eval_str_global("gctorture(FALSE)").unwrap() };
        }
    }
    let _reset = Reset;
    unsafe { r_eval_str_global("gctorture(TRUE)").unwrap() };
    f();
}

fn allocation_burst() {
    for i in 0..32 {
        let _ = vec![i; 64].into_sexp();
    }
    unsafe { sys::R_gc() };
}

fn assert_r_identical(value: SEXP, expected: &str) {
    let reference = RootedSentinel::new(unsafe { r_eval_str_global(expected).unwrap() });
    let identical = bool::from(unsafe { sys::R_compute_identical(value, reference.as_sexp(), 0) });
    if !identical {
        eprintln!(
            "R comparison failed: {expected}; actual names type: {:?}",
            value.get_names().type_of()
        );
    }
    assert!(identical);
}

#[test]
fn mixed_values_raw_sexps_names_and_list_finish() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut list = NamedListBuilder::with_capacity(1);
            assert!(list.is_empty());
            list.push("integer", vec![1, 2, 3]);
            list.push("double", 2.5);
            list.push("logical", true);
            list.push("text", "hello");
            // Fresh, unrooted values, including one that triggers growth.
            list.push_sexp("factor", unsafe {
                r_eval_str_global("factor(c('a', 'b'))").unwrap()
            });
            list.push_sexp("date", unsafe {
                r_eval_str_global("as.Date('2026-10-10')").unwrap()
            });
            list.push_sexp("nested", unsafe {
                r_eval_str_global("list(x = 7L)").unwrap()
            });
            list.push_sexp("null", SEXP::nil());
            list.rename(0, "ids");
            list.names_mut()[3] = "label".into();
            assert_eq!(list.len(), 8);
            assert_eq!(list.names()[0], "ids");
            allocation_burst();
            let value = RootedSentinel::new(list);
            assert_r_identical(
                value.as_sexp(),
                "list(ids = c(1L, 2L, 3L), double = 2.5, logical = TRUE, \
                 label = 'hello', factor = factor(c('a', 'b')), \
                 date = as.Date('2026-10-10'), nested = list(x = 7L), null = NULL)",
            );
        });
    });
}

#[test]
fn data_frame_finish_stays_rooted_and_has_compact_row_names() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut columns = NamedListBuilder::with_capacity(8);
            columns.push("id", vec![1, 2, 3]);
            columns.push("value", vec![0.5, 1.5, 2.5]);
            columns.push("label", vec!["a", "b", "c"]);
            let frame = columns.into_data_frame().unwrap();
            allocation_burst();
            assert_eq!((frame.nrow(), frame.ncol()), (3, 3));
            // Equal to base data.frame(), including its compact row.names.
            assert_r_identical(
                frame.as_sexp(),
                "data.frame(id = c(1L, 2L, 3L), value = c(0.5, 1.5, 2.5), \
                 label = c('a', 'b', 'c'))",
            );
        });
    });
}

#[test]
fn length_error_names_every_mismatched_column() {
    r_test_utils::with_r_thread(|| {
        let mut columns = NamedListBuilder::new();
        columns.push("reference", vec![1, 2]);
        columns.push("short", vec![3]);
        columns.push("valid", vec![4, 5]);
        columns.push("long", vec![6, 7, 8]);
        columns.push("empty", Vec::<i32>::new());
        let error = match columns.into_data_frame() {
            Err(error) => error,
            Ok(_) => panic!("unequal columns must fail"),
        };
        assert_eq!(
            error.to_string(),
            "column \"short\" has length 1 (expected 2); \
             column \"long\" has length 3 (expected 2); \
             column \"empty\" has length 0 (expected 2)"
        );
        let DataFrameError::UnequalLengths { expected, columns } = error else {
            panic!("expected a structured column-length error");
        };
        assert_eq!(expected, 2);
        assert_eq!(
            columns,
            vec![("short".into(), 1), ("long".into(), 3), ("empty".into(), 0)]
        );
    });
}

#[test]
fn empty_lists_and_frames_keep_empty_names() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            for capacity in [0, 7] {
                let list = RootedSentinel::new(NamedListBuilder::with_capacity(capacity));
                assert_eq!(list.as_sexp().xlength(), 0);
                assert_eq!(list.as_sexp().get_names().type_of(), SEXPTYPE::STRSXP);
                assert_eq!(list.as_sexp().get_names().xlength(), 0);

                let frame = NamedListBuilder::with_capacity(capacity)
                    .into_data_frame()
                    .unwrap();
                allocation_burst();
                assert_eq!((frame.nrow(), frame.ncol()), (0, 0));
                assert_r_identical(frame.as_sexp(), "data.frame()");
                let roundtrip = DataFrame::try_from_sexp(frame.as_sexp()).unwrap();
                assert_eq!((roundtrip.nrow(), roundtrip.ncol()), (0, 0));
            }
            let mut columns = NamedListBuilder::new();
            columns.push("id", Vec::<i32>::new());
            columns.push("label", Vec::<String>::new());
            let frame = columns.into_data_frame().unwrap();
            assert_r_identical(
                frame.as_sexp(),
                "data.frame(id = integer(), label = character())",
            );
        });
    });
}

#[test]
fn moved_builder_survives_collections_and_arbitrary_drop_order() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut older = NamedListBuilder::new();
            older.push("old", "discard");
            let mut held = NamedListBuilder::with_capacity(1);
            for i in 0..16 {
                held.push_sexp(format!("v{i}"), vec![i; 4].into_sexp());
                allocation_burst();
            }
            let held = Box::new(held);
            // A younger stack protection must survive dropping an older builder.
            let younger = unsafe { OwnedProtect::new(vec![42; 8].into_sexp()) };
            drop(older);
            allocation_burst();
            assert_eq!(younger.integer_elt(7), 42);
            let list = RootedSentinel::new(*held);
            allocation_burst();
            for i in 0..16 {
                let value = list.as_sexp().vector_elt(i);
                assert_eq!(value.xlength(), 4);
                assert_eq!(value.integer_elt(3), i32::try_from(i).unwrap());
            }
            assert_eq!(younger.integer_elt(0), 42);
        });
    });
}

#[test]
fn fifty_thousand_pushes_do_not_consume_the_protect_stack() {
    r_test_utils::with_r_thread(|| {
        // R's non-API stack counter is only used in this embedded-R test,
        // never in the library or installed package (like R_CStackLimit in
        // r_test_utils). Assert the invariant directly, as well as exceeding
        // the default ~50k stack limit that per-value protection would hit.
        unsafe extern "C" {
            static R_PPStackTop: std::os::raw::c_int;
        }
        let baseline = unsafe { R_PPStackTop };
        let mut builder = NamedListBuilder::default();
        for i in 0..50_000 {
            if i % 2 == 0 {
                builder.push(i.to_string(), i);
            } else {
                builder.push_sexp(i.to_string(), i.into_sexp());
            }
            if i % 4096 == 0 {
                unsafe { sys::R_gc() };
                assert_eq!(unsafe { R_PPStackTop }, baseline);
            }
        }
        assert_eq!(builder.len(), 50_000);
        assert_eq!(unsafe { R_PPStackTop }, baseline);
        let list = RootedSentinel::new(builder);
        assert_eq!(unsafe { R_PPStackTop }, baseline);
        for i in 0..50_000 {
            assert_eq!(
                list.as_sexp().vector_elt(i).integer_elt(0),
                i32::try_from(i).unwrap()
            );
        }
        drop(list);
        assert_eq!(unsafe { R_PPStackTop }, baseline);
    });
}

#[derive(IntoList)]
struct Named<T> {
    id: i32,
    value: T,
    #[into_list(ignore)]
    ignored: (),
}

#[derive(IntoList)]
struct Tuple(i32, String, #[into_list(ignore)] ());

#[derive(IntoList)]
struct Unit;

#[derive(IntoList)]
struct Empty {}

#[test]
fn into_list_derive_preserves_named_tuple_and_unit_shapes() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let named = Named {
                id: 1,
                value: "hello".to_string(),
                ignored: (),
            };
            let () = named.ignored;
            let list = RootedSentinel::new(named.into_list());
            assert_r_identical(list.as_sexp(), "list(id = 1L, value = 'hello')");
            let list = RootedSentinel::new(Tuple(2, "tuple".into(), ()).into_list());
            assert_r_identical(list.as_sexp(), "list(2L, 'tuple')");
            let list = RootedSentinel::new(Unit.into_list());
            assert_r_identical(list.as_sexp(), "list()");
            let list = RootedSentinel::new(Empty {}.into_list());
            assert_r_identical(list.as_sexp(), "setNames(list(), character())");
        });
    });
}

#[cfg(feature = "serde")]
#[test]
fn serde_dispatch_result_can_be_held_before_into_r() {
    use miniextendr_api::serde::{DispatchNames, dispatch_to_dataframes};

    #[derive(serde::Serialize)]
    struct Row {
        id: i32,
    }

    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let builder = dispatch_to_dataframes(
                [Ok(Row { id: 1 }), Err(Row { id: 2 }), Ok(Row { id: 3 })],
                None,
                DispatchNames::default(),
            )
            .unwrap();
            allocation_burst();
            let value = RootedSentinel::new(builder);
            assert_r_identical(
                value.as_sexp(),
                "list(ok = data.frame(id = c(1L, 3L)), err = data.frame(id = 2L))",
            );
        });
    });
}
