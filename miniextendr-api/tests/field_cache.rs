//! `#[r_data(cache)]` (#2271): a cached field's getter returns the R value
//! its first read made until the value is borrowed exclusively, on every
//! path that borrows it so: `&mut self`, a view receiver, a `&mut T`
//! argument, the R setters, consuming methods, `ExternalPtr::borrow_mut`,
//! `downcast_mut`, `into_inner`. A shared borrow keeps it, a clone starts
//! without one, and a failed conversion is not kept.
mod r_test_utils;

use miniextendr_api::condition::RCondition;
use miniextendr_api::externalptr::{ErasedExternalPtr, ExternalPtrMut, RSidecar, TypedExternal};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, SEXP, TryFromSexp, miniextendr};

#[derive(ExternalPtr, Clone)]
pub struct Cached {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache)]
    pub values: Vec<i32>,
    #[r_data(cache, null_if_none)]
    pub label: Option<String>,
    #[r_data]
    pub plain: Vec<i32>,
    #[r_data]
    pub n: i32,
}

#[miniextendr(env, noexport)]
impl Cached {
    pub fn cache_push(&mut self, value: i32) {
        self.values.push(value);
    }
    pub fn cache_len(&self) -> i32 {
        i32::try_from(self.values.len()).unwrap()
    }
    pub fn cache_view_push(mut self: ExternalPtrMut<'_, Self>, value: i32) {
        self.values.push(value);
    }
    pub fn cache_grown(self) -> Self {
        let mut this = self;
        this.values.push(0);
        this
    }
    pub fn cache_checked(self) -> Result<Self, String> {
        let mut this = self;
        this.values.push(-1);
        Ok(this)
    }
}

#[miniextendr(noexport)]
pub fn cache_push_argument(target: &mut Cached, value: i32) {
    target.values.push(value);
}

fn host() -> Cached {
    Cached {
        _r: RSidecar,
        values: vec![1, 2, 3],
        label: None,
        plain: vec![7],
        n: 1,
    }
}

unsafe fn entry(
    suffix: &str,
    arity: i32,
) -> unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .unwrap_or_else(|| panic!("`{suffix}` must be registered"));
    assert_eq!(entry.numArgs, arity);
    entry.fun.unwrap()
}

unsafe fn call2(suffix: &str, a: SEXP, b: SEXP) -> SEXP {
    let call = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(entry(suffix, 2))
    };
    unsafe { call(a, b) }
}

unsafe fn call3(suffix: &str, a: SEXP, b: SEXP, c: SEXP) -> SEXP {
    let call = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP,
        >(entry(suffix, 3))
    };
    unsafe { call(a, b, c) }
}

/// Fails the test on a tagged condition.
fn ok(result: SEXP) -> SEXP {
    if let Some(condition) = unsafe { RCondition::from_tagged_sexp(result) } {
        panic!("unexpected condition: {condition:?}");
    }
    result
}

/// Reads the field `name` of `x` through its R getter.
fn get(x: SEXP, name: &str) -> SEXP {
    ok(unsafe {
        call2(
            &format!("__mx_rdata_get_Cached_{name}"),
            x,
            false.into_sexp(),
        )
    })
}

fn ints(x: SEXP) -> Vec<i32> {
    Vec::<i32>::try_from_sexp(x).unwrap()
}

/// A type without cached fields.
#[derive(ExternalPtr)]
pub struct Uncached {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub values: Vec<i32>,
}

#[test]
fn the_type_counts_its_cached_fields() {
    assert_eq!(<Cached as TypedExternal>::CACHED_FIELD_COUNT, 2);
    assert_eq!(<Uncached as TypedExternal>::CACHED_FIELD_COUNT, 0);
}

#[test]
fn repeated_reads_return_the_kept_object_until_an_exclusive_borrow() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut owner = ExternalPtr::new(host());
        let x = owner.as_sexp();
        // A value compared with a later read's is protected: once its kept
        // copy is dropped, a collection could hand its address to the next
        // value.
        let scope = ProtectScope::new();

        let first = scope.protect(get(x, "values")).get();
        assert_eq!(ints(first), [1, 2, 3]);
        assert_eq!(get(x, "values"), first);
        // An uncached field converts on every read.
        let plain = scope.protect(get(x, "plain")).get();
        assert_ne!(get(x, "plain"), plain);
        // A shared borrow keeps it.
        assert_eq!(owner.borrow().values, [1, 2, 3]);
        assert_eq!(get(x, "values"), first);

        // `ExternalPtr::borrow_mut`.
        owner.borrow_mut().values.push(4);
        let second = get(x, "values");
        assert_ne!(second, first);
        assert_eq!(ints(second), [1, 2, 3, 4]);
        assert_eq!(get(x, "values"), second);
        // A read of another field keeps it too.
        get(x, "plain");
        assert_eq!(get(x, "values"), second);

        // A `&self` method keeps it; a `&mut self` method drops it.
        let len = ok(call2("Cached__cache_len", SEXP::nil(), x));
        assert_eq!(i32::try_from_sexp(len).unwrap(), 4);
        assert_eq!(get(x, "values"), second);
        let five = scope.protect(5_i32.into_sexp()).get();
        ok(call3("Cached__cache_push", SEXP::nil(), x, five));
        let third = get(x, "values");
        assert_eq!(ints(third), [1, 2, 3, 4, 5]);

        // An `ExternalPtrMut` receiver.
        let six = scope.protect(6_i32.into_sexp()).get();
        ok(call3("Cached__cache_view_push", SEXP::nil(), x, six));
        assert_eq!(ints(get(x, "values")), [1, 2, 3, 4, 5, 6]);

        // A `&mut T` argument.
        let seven = scope.protect(7_i32.into_sexp()).get();
        ok(call3("_cache_push_argument", SEXP::nil(), x, seven));
        assert_eq!(ints(get(x, "values")), [1, 2, 3, 4, 5, 6, 7]);

        // Consuming methods: `self -> Self` and `self -> Result<Self, E>`.
        ok(call2("Cached__cache_grown", SEXP::nil(), x));
        assert_eq!(ints(get(x, "values")), [1, 2, 3, 4, 5, 6, 7, 0]);
        ok(call2("Cached__cache_checked", SEXP::nil(), x));
        assert_eq!(ints(get(x, "values")), [1, 2, 3, 4, 5, 6, 7, 0, -1]);

        // The R setter of the field, and of another field: every exclusive
        // borrow drops what is kept.
        let kept = scope.protect(get(x, "values")).get();
        let replacement = scope.protect(vec![9_i32].into_sexp()).get();
        ok(call3(
            "__mx_rdata_set_Cached_values",
            x,
            replacement,
            false.into_sexp(),
        ));
        let after_set = scope.protect(get(x, "values")).get();
        assert_ne!(after_set, kept);
        assert_eq!(ints(after_set), [9]);
        let n = scope.protect(2_i32.into_sexp()).get();
        ok(call3("__mx_rdata_set_Cached_n", x, n, false.into_sexp()));
        assert_ne!(get(x, "values"), after_set);

        // `downcast_mut` of the erased handle.
        let kept = scope.protect(get(x, "values")).get();
        let mut erased = ErasedExternalPtr::from_sexp(x);
        erased.downcast_mut::<Cached>().unwrap().values.push(8);
        let after_downcast = get(x, "values");
        assert_ne!(after_downcast, kept);
        assert_eq!(ints(after_downcast), [9, 8]);
    });
}

#[test]
fn a_null_is_kept_like_any_value() {
    r_test_utils::with_r_thread(|| {
        let mut owner = ExternalPtr::new(host());
        let x = owner.as_sexp();
        assert!(get(x, "label").is_nil());
        assert!(get(x, "label").is_nil());
        owner.borrow_mut().label = Some("a".to_owned());
        let label = get(x, "label");
        assert_eq!(String::try_from_sexp(label).unwrap(), "a");
        assert_eq!(get(x, "label"), label);
    });
}

#[test]
fn a_clone_and_a_rewrapped_value_start_without_a_kept_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let owner = ExternalPtr::new(host());
        let kept = get(owner.as_sexp(), "values");
        let copy = owner.clone();
        let copied = scope.protect(get(copy.as_sexp(), "values")).get();
        assert_ne!(copied, kept);
        assert_eq!(ints(copied), [1, 2, 3]);
        assert_eq!(get(owner.as_sexp(), "values"), kept);

        let rewrapped = ExternalPtr::new(ExternalPtr::into_inner(copy));
        let again = get(rewrapped.as_sexp(), "values");
        assert_ne!(again, copied);
        assert_eq!(ints(again), [1, 2, 3]);
    });
}

#[test]
fn a_read_under_an_exclusive_borrow_is_refused() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut owner = ExternalPtr::new(host());
        let x = owner.as_sexp();
        get(x, "values");
        let guard = owner.borrow_mut();
        let refused = call2("__mx_rdata_get_Cached_values", x, false.into_sexp());
        let condition = RCondition::from_tagged_sexp(refused).expect("a conflict");
        let (_, parts) = condition.into_parts();
        assert!(
            parts
                .class
                .iter()
                .any(|class| class == "miniextendr_borrow_conflict"),
            "{:?}",
            parts.class
        );
        drop(guard);
        assert_eq!(ints(get(x, "values")), [1, 2, 3]);
    });
}
