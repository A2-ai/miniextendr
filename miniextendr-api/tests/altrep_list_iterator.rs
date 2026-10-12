//! List iterators retain fresh R objects through allocation, copy and save.
mod r_test_utils;

use std::cell::Cell;
use std::rc::Rc;

use miniextendr_api::altrep_data::{AltListData, AltrepLen, AltrepSerialize, IterListData};
use miniextendr_api::altrep_traits::Altrep;
use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::{IntoR, OwnedProtect, SEXP, SexpExt};

/// Run R's collector, which also runs the finalizers it made ready.
fn gc() {
    unsafe { r_eval_str_global("invisible(gc())").unwrap() };
}

/// How many environments from [`watched_env`] R has finalized.
fn finalized() -> i32 {
    unsafe { r_eval_str_global("mx_list_iter_finalized").unwrap() }.integer_elt(0)
}

/// A new environment, held by nothing, whose finalizer counts itself.
fn watched_env(_: i32) -> SEXP {
    unsafe {
        r_eval_str_global(
            "local({
                 e <- new.env()
                 reg.finalizer(e, function(e) mx_list_iter_finalized <<- mx_list_iter_finalized + 1L)
                 e
             })",
        )
        .unwrap()
    }
}

#[test]
fn yielded_objects_live_as_long_as_an_adapter_holds_them() {
    r_test_utils::with_r_thread(|| unsafe {
        r_eval_str_global("mx_list_iter_finalized <- 0L").unwrap();
        let data = IterListData::from_iter((0..3).map(watched_env as fn(i32) -> SEXP), 3);
        data.elt(2);
        gc();
        assert_eq!(finalized(), 0, "the cache roots every yielded object");

        let copy = data.clone();
        drop(data);
        gc();
        assert_eq!(finalized(), 0, "the clone's cache holds the same objects");
        drop(copy);
        gc();
        assert_eq!(finalized(), 3, "dropping the last adapter releases them");
        r_eval_str_global("rm(mx_list_iter_finalized)").unwrap();
    });
}

#[test]
fn allocating_prefix_is_rooted_and_clones_advance_independently() {
    r_test_utils::with_r_thread(|| unsafe {
        let calls = Rc::new(Cell::new(0_i32));
        let seen = calls.clone();
        let source = (0..8_i32).map(move |i| {
            seen.set(seen.get() + 1);
            miniextendr_api::sys::R_gc();
            vec![i, seen.get()].into_sexp()
        });
        let data = IterListData::from_iter(source, 6);
        assert_eq!(calls.get(), 0);
        assert_eq!(data.elt(3).integer_elt(0), 3);
        assert_eq!(calls.get(), 4);
        let first = data.elt(0);
        miniextendr_api::sys::R_gc();
        assert_eq!(first.integer_elt(0), 0);
        assert_eq!(data.elt(0), first);
        assert_eq!(calls.get(), 4);

        let copy = data.clone();
        assert_eq!(calls.get(), 4, "Clone must not consume the source");
        assert_eq!(data.elt(4).integer_elt(1), 5);
        assert_eq!(copy.elt(5).integer_elt(1), 7);
        assert_eq!(copy.elt(4).integer_elt(1), 6);
        assert_eq!(data.elt(4).integer_elt(1), 5);
        assert!(data.elt(6).is_nil());
        assert_eq!(calls.get(), 7, "No read past the declared prefix");
        drop(data);
        miniextendr_api::sys::R_gc();
        assert_eq!(copy.elt(0).integer_elt(0), 0);
        assert_eq!(copy.elt(5).integer_elt(0), 5);
    });
}

#[test]
fn short_nonfused_source_saves_nulls_and_never_resumes() {
    r_test_utils::with_r_thread(|| unsafe {
        let calls = Rc::new(Cell::new(0));
        let seen = calls.clone();
        let source = std::iter::from_fn(move || {
            seen.set(seen.get() + 1);
            match seen.get() {
                1 => Some(vec![7_i32].into_sexp()),
                2 => None,
                _ => Some(vec![99_i32].into_sexp()),
            }
        });
        let data = IterListData::from_iter(source, 3);
        assert!(data.elt(2).is_nil());
        assert!(data.elt(1).is_nil());
        let state = OwnedProtect::new(data.serialized_state());
        assert_eq!(state.get().len(), 3);
        assert_eq!(state.get().vector_elt(0).integer_elt(0), 7);
        assert!(state.get().vector_elt(1).is_nil());
        assert_eq!(calls.get(), 2);
        // The state is a list of its own: changing it leaves the cache alone.
        state.get().set_vector_elt(2, vec![8_i32].into_sexp());
        assert!(data.elt(2).is_nil());

        type Restored = IterListData<std::iter::Empty<SEXP>>;
        let restored = Restored::unserialize(state.get()).unwrap();
        drop(data);
        miniextendr_api::sys::R_gc();
        assert_eq!(restored.len(), 3);
        assert_eq!(restored.elt(0).integer_elt(0), 7);
        assert!(restored.elt(1).is_nil());
        assert_eq!(restored.elt(2).integer_elt(0), 8);
        assert!(Restored::unserialize(SEXP::nil()).is_none());
    });
}

#[test]
fn an_empty_declared_prefix_does_not_poll_the_source() {
    r_test_utils::with_r_thread(|| unsafe {
        let source = std::iter::from_fn(|| -> Option<SEXP> { panic!("empty source polled") });
        let data = IterListData::from_iter(source, 0);
        assert!(data.elt(0).is_nil());
        let state = OwnedProtect::new(data.serialized_state());
        assert!(state.get().is_list());
        assert_eq!(state.get().len(), 0);
    });
}

type Source = std::iter::Map<std::ops::Range<i32>, fn(i32) -> SEXP>;

fn element(i: i32) -> SEXP {
    if i == 1 {
        SEXP::nil()
    } else {
        vec![i, i + 10].into_sexp()
    }
}

#[derive(Clone, miniextendr_api::AltrepList)]
#[altrep(manual, serialize, duplicate = "clone", class = "ConcreteListIterator")]
struct ListIterator(IterListData<Source>);

impl AltrepLen for ListIterator {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl AltListData for ListIterator {
    fn elt(&self, i: usize) -> SEXP {
        self.0.elt(i)
    }
}
impl AltrepSerialize for ListIterator {
    fn serialized_state(&self) -> SEXP {
        self.0.serialized_state()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        IterListData::unserialize(state).map(Self)
    }
}

#[test]
fn concrete_list_class_duplicates_and_serializes_its_rooted_cache() {
    r_test_utils::with_r_thread(|| unsafe {
        let source = (0..5).map(element as fn(i32) -> SEXP);
        let value =
            OwnedProtect::new(ListIterator(IterListData::from_exact_iter(source)).into_sexp());
        assert_eq!(value.get().vector_elt(2).integer_elt(0), 2);
        let copy = OwnedProtect::new(<ListIterator as Altrep>::duplicate(value.get(), true));
        assert!(copy.get().is_altrep());
        assert_eq!(copy.get().vector_elt(4).integer_elt(1), 14);
        let bytes = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("serialize")
                .arg(value.get())
                .arg(SEXP::nil())
                .eval_base()
                .unwrap(),
        );
        let restored = OwnedProtect::new(
            miniextendr_api::expression::RCall::new("unserialize")
                .arg(bytes.get())
                .eval_base()
                .unwrap(),
        );
        assert!(restored.get().is_altrep());
        miniextendr_api::sys::R_gc();
        for i in 0..5 {
            if i == 1 {
                assert!(restored.get().vector_elt(i).is_nil());
            } else {
                assert_eq!(
                    restored.get().vector_elt(i).integer_elt(0),
                    i32::try_from(i).unwrap()
                );
            }
        }
    });
}
