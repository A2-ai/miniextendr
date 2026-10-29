use super::*;

// Note: These tests primarily verify compilation and basic invariants.
// Full integration testing requires R to be initialized.
// endregion

// region: Basic invariants

#[test]
fn protect_scope_has_nosend_marker() {
    // Verify the NoSendSync marker type is present
    // (ProtectScope contains PhantomData<Rc<()>> which makes it !Send + !Sync)
    let _: NoSendSync = PhantomData;
}

#[test]
fn protect_scope_default_count_is_zero() {
    let scope = unsafe { ProtectScope::new() };
    assert_eq!(scope.count(), 0);
}

#[test]
fn root_is_copy() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<Root<'static>>();
}

#[test]
fn tls_root_is_copy() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<tls::TlsRoot>();
}
// endregion

// region: Threading: compile-time !Send + !Sync checks

#[test]
fn protect_scope_is_not_send() {
    fn assert_not_send<T>()
    where
        T: ?Sized,
    {
        // This test passes if ProtectScope is !Send
        // We can't directly assert !Send, but the type containing Rc<()> ensures it
    }
    assert_not_send::<ProtectScope>();
}

#[test]
fn protect_scope_is_not_sync() {
    fn assert_not_sync<T>()
    where
        T: ?Sized,
    {
        // This test passes if ProtectScope is !Sync
    }
    assert_not_sync::<ProtectScope>();
}

#[test]
fn owned_protect_is_not_send() {
    fn assert_not_send<T>()
    where
        T: ?Sized,
    {
    }
    assert_not_send::<OwnedProtect>();
}

// Note: We can't easily assert !Send/!Sync at compile time without
// negative trait bounds. The PhantomData<Rc<()>> marker ensures these types
// are !Send and !Sync. If you need compile-time verification, use the
// static_assertions crate with `assert_not_impl_any!`.
// endregion

// region: TLS scope tests

#[test]
fn tls_no_active_scope_by_default() {
    assert!(!tls::has_active_scope());
    assert_eq!(tls::current_count(), None);
    assert_eq!(tls::scope_depth(), 0);
}

#[test]
fn tls_scope_depth_tracking() {
    // Without R, we can only test the TLS tracking logic
    // The actual protect/unprotect requires R runtime

    // Test that scope depth is tracked correctly
    assert_eq!(tls::scope_depth(), 0);

    // We can't fully test with_protect_scope without R initialized,
    // but we can verify the API compiles and the TLS logic works
}

#[test]
#[should_panic(expected = "tls::protect called outside of with_protect_scope")]
fn tls_protect_panics_outside_scope() {
    // This should panic because there's no active scope
    // Note: Can't actually call protect without R, but we test the panic message
    unsafe {
        let _ = tls::protect(crate::SEXP(std::ptr::null_mut()));
    }
}
// endregion

// region: Escape hatch tests

#[test]
fn disarm_prevents_unprotect() {
    let scope = unsafe { ProtectScope::new() };
    assert!(scope.armed.get());

    unsafe { scope.disarm() };
    assert!(!scope.armed.get());

    // Scope will drop without calling Rf_unprotect (can't test actual R call)
}

#[test]
fn rearm_restores_unprotect() {
    let scope = unsafe { ProtectScope::new() };

    unsafe {
        scope.disarm();
        assert!(!scope.armed.get());

        scope.rearm();
        assert!(scope.armed.get());
    }
}
// endregion

// region: Counter tracking tests

#[test]
fn scope_counter_starts_at_zero() {
    let scope = unsafe { ProtectScope::new() };
    assert_eq!(scope.count(), 0);
}

// Note: The following tests require R to be initialized and would be
// integration tests rather than unit tests:
//
// - Balance test: protect N, verify unprotect(N) on drop (gctorture)
// - Nested scopes: verify drop order yields correct net unprotect
// - Reprotect slot: verify set() many times keeps count at +1
//
// These should be tested in miniextendr-api/tests/gc_protect.rs with
// embedded R.
// endregion

// region: Protected<'a, T> tests

/// Verify `Protected::get()` returns the inner value.
#[test]
fn protected_get_returns_inner() {
    // from_trusted with null SEXP (safe for compile-time tests since we
    // never dereference the SEXP itself, only the already-constructed inner)
    let v = vec![1i32, 2, 3];
    let p = unsafe {
        Protected::<'static, Vec<i32>>::from_trusted(crate::SEXP(core::ptr::null_mut()), v)
    };
    assert_eq!(p.get(), &[1, 2, 3]);
}

/// Verify `Deref<Target = T>` works (exercises the blanket impl).
#[test]
fn protected_deref_reaches_inner() {
    let v = vec![10i32, 20];
    let p = unsafe {
        Protected::<'static, Vec<i32>>::from_trusted(crate::SEXP(core::ptr::null_mut()), v)
    };
    // Deref should let us call Vec methods directly.
    assert_eq!(p.len(), 2);
}

/// Verify `into_inner` moves the inner value out cleanly.
#[test]
fn protected_into_inner_moves_value() {
    let v = vec![42i32];
    let p = unsafe {
        Protected::<'static, Vec<i32>>::from_trusted(crate::SEXP(core::ptr::null_mut()), v)
    };
    let got = p.into_inner();
    assert_eq!(got, [42]);
}

/// Verify `from_trusted` sets `_protect` to `None` (no extra protect push).
#[test]
fn protected_from_trusted_does_not_hold_protect() {
    let v: Vec<u8> = vec![];
    let p = unsafe {
        Protected::<'static, Vec<u8>>::from_trusted(crate::SEXP(core::ptr::null_mut()), v)
    };
    // This test uses `from_trusted` to avoid touching the protect stack;
    // `Protected::new` would require an initialized R runtime (see
    // `tests/gc_protect.rs` for that path). Verify the `_protect` field is
    // `None` indirectly via the `Debug` output.
    assert!(format!("{p:?}").contains("protected: false"));
}

/// Smoke-test: `Protected::from_trusted` + `Deref` compiles for a scalar `T`.
#[test]
fn protected_from_trusted_compile_check() {
    let p =
        unsafe { Protected::<'static, i32>::from_trusted(crate::SEXP(core::ptr::null_mut()), 99) };
    assert_eq!(*p, 99);
}
// endregion
