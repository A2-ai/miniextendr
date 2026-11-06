//! Owned list construction: growth, both finishes (the rooted `BuiltList` and
//! `BuiltDataFrame` handles, and the data frame's editing chain), and
//! GC/drop-order invariants.

mod r_test_utils;

use miniextendr_api::expression::{RCall, r_eval_str_global};
use miniextendr_api::{
    BuiltDataFrame, BuiltList, DataFrame, DataFrameError, IntoList, IntoR, NamedListBuilder,
    OwnedProtect, Preserved, SEXP, SEXPTYPE, SexpExt, TryFromSexp, sys,
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
    let reference = Preserved::new(unsafe { r_eval_str_global(expected).unwrap() });
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
            let value = Preserved::new(list);
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
fn list_finish_stays_rooted_while_attributes_are_set() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut list = NamedListBuilder::new();
            list.push("a", vec![1, 2]);
            list.push_sexp("b", unsafe { r_eval_str_global("list(x = 'y')").unwrap() });
            let built = list.into_list();
            allocation_burst();
            // Each link allocates (the class vector, the symbol, the value)
            // while the handle keeps the list rooted.
            let built = built
                .set_class(&["my_result", "list"])
                .set_attr("note", vec!["kept"; 3]);
            allocation_burst();
            assert_eq!(built.as_sexp().xlength(), 2);
            let value = Preserved::new(built);
            assert_r_identical(
                value.as_sexp(),
                "structure(list(a = 1:2, b = list(x = 'y')), \
                 class = c('my_result', 'list'), note = rep('kept', 3))",
            );
        });
    });
}

#[test]
fn built_list_reads_through_its_list_view_and_is_handed_to_r() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut parts = NamedListBuilder::new();
            parts.push("id", vec![1, 2, 3]);
            parts.push("label", "three");
            parts.push("missing", Option::<f64>::None);
            let built: BuiltList = parts.into_list();
            allocation_burst();
            assert_eq!(format!("{built:?}"), "BuiltList { len: 3 }");

            // Reads go through `Deref<Target = List>`.
            assert_eq!(built.len(), 3);
            assert!(!built.is_empty());
            assert_eq!(built.get_named::<Vec<i32>>("id"), Some(vec![1, 2, 3]));
            assert_eq!(built.get_named::<String>("label").as_deref(), Some("three"));
            assert_eq!(built.get_named::<Option<f64>>("missing"), Some(None));
            assert_eq!(built.get_named::<i32>("absent"), None);
            assert_eq!(built.get_index::<Vec<i32>>(0), Some(vec![1, 2, 3]));
            assert_eq!(built.get_index::<Vec<i32>>(3), None);

            // Handed to R in the middle of a chain: the handle keeps the list
            // alive while R evaluates a call on it, and while the class and the
            // attribute allocate afterwards.
            let built = built.set_class(&["three_parts", "list"]);
            let names: Vec<String> =
                unsafe { RCall::new("names").arg(built.as_sexp()).eval_base_as() }.unwrap();
            assert_eq!(names, ["id", "label", "missing"]);
            let inherits: bool = unsafe {
                RCall::new("inherits")
                    .arg(built.as_sexp())
                    .arg_value("three_parts")
                    .eval_base_as()
            }
            .unwrap();
            assert!(inherits);
            let built = built.set_attr("source", "rust");
            allocation_burst();

            // `IntoR` releases the root and hands the list over.
            let value = Preserved::new(built);
            assert_r_identical(
                value.as_sexp(),
                "structure(list(id = 1:3, label = 'three', missing = NA_real_), \
                 class = c('three_parts', 'list'), source = 'rust')",
            );
        });
    });
}

#[test]
fn empty_built_list_keeps_empty_names_and_takes_attributes() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let built: BuiltList = NamedListBuilder::with_capacity(5).into_list();
            allocation_burst();
            assert_eq!(format!("{built:?}"), "BuiltList { len: 0 }");
            assert!(built.is_empty());
            assert_eq!(built.get_index::<i32>(0), None);
            assert_eq!(built.get_named::<i32>(""), None);
            let names = built.names().expect("an empty finish has names");
            assert_eq!(names.type_of(), SEXPTYPE::STRSXP);
            assert_eq!(names.xlength(), 0);

            let value = Preserved::new(built.set_class(&["empty_result"]));
            assert_r_identical(
                value.as_sexp(),
                "structure(setNames(list(), character()), class = 'empty_result')",
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
fn built_data_frame_editing_chain_is_rooted_at_every_link() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut columns = NamedListBuilder::new();
            columns.push("x_id", vec![1, 2, 3]);
            columns.push("x_value", vec![0.5, 1.5, 2.5]);
            columns.push("x_label", vec!["a", "b", "c"]);
            columns.push("note", vec!["n1", "n2", "n3"]);
            let frame: BuiltDataFrame = columns.into_data_frame().unwrap();
            allocation_burst();
            assert_eq!(format!("{frame:?}"), "BuiltDataFrame { nrow: 3, ncol: 4 }");

            // The added columns are the caller's to keep reachable.
            let flag = Preserved::new(vec![true, false, true]);
            let row = Preserved::new(vec!["r1", "r2", "r3"]);
            // Each link allocates (new lists, names, CHARSXPs) while only the
            // previous link's handle roots its input.
            let edited: BuiltDataFrame = frame
                .strip_prefix("x_")
                .rename("value", "estimate")
                .drop("label")
                .with_column("flag", flag.as_sexp())
                .prepend_column("row", row.as_sexp())
                .select(&["row", "id", "estimate", "flag"]);
            allocation_burst();
            assert_eq!((edited.nrow(), edited.ncol()), (3, 4));
            assert_eq!(edited.names().unwrap(), ["row", "id", "estimate", "flag"]);
            // Reads go through `Deref<Target = DataFrame>`.
            assert_eq!(edited.column::<Vec<i32>>("id"), Some(vec![1, 2, 3]));
            assert_eq!(edited.column::<Vec<f64>>("value"), None);
            assert!(!edited.contains_column("note"));

            let rows: BuiltDataFrame = edited.select_rows(&[2, 0]);
            allocation_burst();
            assert_r_identical(
                edited.as_sexp(),
                "data.frame(row = c('r1', 'r2', 'r3'), id = 1:3, \
                 estimate = c(0.5, 1.5, 2.5), flag = c(TRUE, FALSE, TRUE))",
            );
            assert_r_identical(
                rows.as_sexp(),
                "data.frame(row = c('r3', 'r1'), id = c(3L, 1L), \
                 estimate = c(2.5, 0.5), flag = c(TRUE, TRUE))",
            );
            drop(edited);
            allocation_burst();
            // The row subset owns its own root.
            assert_eq!(rows.column::<Vec<String>>("row").unwrap(), ["r3", "r1"]);
        });
    });
}

#[test]
fn built_data_frame_no_op_and_empty_edits() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            let mut columns = NamedListBuilder::new();
            columns.push("id", vec![1, 2]);
            columns.push("label", vec!["a", "b"]);
            let frame = columns.into_data_frame().unwrap();
            let original = frame.as_sexp();

            // Dropping or renaming a column that isn't there keeps the frame.
            let same = frame.drop("absent").rename("absent", "other");
            assert_eq!(same.as_sexp(), original);
            assert_eq!(same.names().unwrap(), ["id", "label"]);

            // Replacing a column writes into the same frame.
            let replacement = Preserved::new(vec![10, 20]);
            let replaced = same.with_column("id", replacement.as_sexp());
            assert_eq!(replaced.as_sexp(), original);
            allocation_burst();
            assert_eq!(replaced.column::<Vec<i32>>("id"), Some(vec![10, 20]));

            // No rows: every column keeps its type.
            let no_rows = replaced.select_rows(&[]);
            allocation_burst();
            assert_eq!((no_rows.nrow(), no_rows.ncol()), (0, 2));
            assert_r_identical(
                no_rows.as_sexp(),
                "data.frame(id = integer(), label = character())",
            );

            // No columns: unknown names are skipped, the rows stay.
            let no_columns = replaced.select(&["absent"]);
            allocation_burst();
            assert_eq!((no_columns.nrow(), no_columns.ncol()), (2, 0));
            assert_r_identical(
                no_columns.as_sexp(),
                "data.frame(id = 1:2)[, character(0), drop = FALSE]",
            );
        });
    });
}

#[test]
fn adopted_frames_are_rooted_by_their_handle() {
    r_test_utils::with_r_thread(|| {
        with_gctorture(|| {
            // A frame R made, rooted only by the handle once adopted.
            let adopted = unsafe {
                BuiltDataFrame::adopt_sexp(
                    r_eval_str_global("data.frame(a = 1:2, b = c('x', 'y'))").unwrap(),
                )
            };
            allocation_burst();
            // Two handles over one frame each hold their own root.
            let second = unsafe { BuiltDataFrame::adopt(*adopted) };
            drop(adopted);
            allocation_burst();
            assert_r_identical(second.as_sexp(), "data.frame(a = 1:2, b = c('x', 'y'))");
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
                let list = Preserved::new(NamedListBuilder::with_capacity(capacity));
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
            let list = Preserved::new(*held);
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
        let list = Preserved::new(builder);
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
            let list = Preserved::new(named.into_list());
            assert_r_identical(list.as_sexp(), "list(id = 1L, value = 'hello')");
            let list = Preserved::new(Tuple(2, "tuple".into(), ()).into_list());
            assert_r_identical(list.as_sexp(), "list(2L, 'tuple')");
            let list = Preserved::new(Unit.into_list());
            assert_r_identical(list.as_sexp(), "list()");
            let list = Preserved::new(Empty {}.into_list());
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
            let value = Preserved::new(builder);
            assert_r_identical(
                value.as_sexp(),
                "list(ok = data.frame(id = c(1L, 3L)), err = data.frame(id = 2L))",
            );
        });
    });
}
