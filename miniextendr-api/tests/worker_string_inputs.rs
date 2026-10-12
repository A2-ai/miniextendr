//! Worker character borrows use Rust-owned storage, including container forms.
#![cfg(feature = "worker-thread")]

mod r_test_utils;

use std::sync::atomic::{AtomicUsize, Ordering};

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{IntoR, RCondition, SEXP, SexpExt, TryFromSexp, miniextendr};

static R_TEXT_ADDRESS: AtomicUsize = AtomicUsize::new(0);

#[miniextendr(worker, noexport)]
pub fn worker_character_inputs<'a>(
    text: &'a str,
    optional: Option<&'a str>,
    values: Vec<&'a str>,
    optional_values: Vec<Option<&'a str>>,
    __owned_text: i32,
) -> String {
    assert!(!miniextendr_api::worker::is_r_main_thread());
    for value in std::iter::once(text)
        .chain(optional)
        .chain(values.iter().copied())
        .chain(optional_values.iter().flatten().copied())
    {
        if value == "owned-text" {
            assert_ne!(
                value.as_ptr().addr(),
                R_TEXT_ADDRESS.load(Ordering::Relaxed)
            );
        }
    }
    // A source parameter can share a generated storage binding's spelling.
    format!("{text}|{optional:?}|{values:?}|{optional_values:?}|{__owned_text}")
}

type Wrapper = unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP;

unsafe fn wrapper() -> Wrapper {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(b"_worker_character_inputs")
        })
        .expect("worker wrapper must be registered");
    assert_eq!(entry.numArgs, 6);
    unsafe {
        std::mem::transmute::<unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void, Wrapper>(
            entry.fun.unwrap(),
        )
    }
}

#[test]
fn worker_string_shapes_copy_bytes_and_keep_na_null_and_empty_semantics() {
    r_test_utils::with_r_thread(|| unsafe {
        let call = wrapper();
        let scope = ProtectScope::new();
        let text = scope.protect("owned-text".into_sexp());
        R_TEXT_ADDRESS.store(text.get().string_elt(0).r_char().addr(), Ordering::Relaxed);
        let values = scope.protect(vec![Some("owned-text"), None, Some("")].into_sexp());
        let extra = scope.protect(17_i32.into_sexp());
        let result = scope.protect(call(
            SEXP::nil(),
            text.get(),
            text.get(),
            values.get(),
            values.get(),
            extra.get(),
        ));
        assert_eq!(
            String::try_from_sexp(result.get()).unwrap(),
            "owned-text|Some(\"owned-text\")|[\"owned-text\", \"\", \"\"]|[Some(\"owned-text\"), None, Some(\"\")]|17",
        );

        let na = scope.protect(vec![None::<String>].into_sexp());
        let empty = scope.protect(Vec::<String>::new().into_sexp());
        for optional in [SEXP::nil(), na.get()] {
            let result = scope.protect(call(
                SEXP::nil(),
                na.get(),
                optional,
                empty.get(),
                empty.get(),
                extra.get(),
            ));
            assert_eq!(
                String::try_from_sexp(result.get()).unwrap(),
                "|None|[]|[]|17"
            );
        }

        // The owned detour must retain the argument conversion condition.
        let refused = scope.protect(call(
            SEXP::nil(),
            text.get(),
            extra.get(),
            values.get(),
            values.get(),
            extra.get(),
        ));
        let (_, parts) = RCondition::from_tagged_sexp(refused.get())
            .unwrap()
            .into_parts();
        assert!(parts.message.contains("optional"), "{}", parts.message);
    });
}
