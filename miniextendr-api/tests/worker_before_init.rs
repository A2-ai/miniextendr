//! Keep the pre-initialization check in its own process: unit tests may
//! initialize the global runtime concurrently.

#[test]
fn worker_with_r_thread_panics_before_init() {
    let result = std::panic::catch_unwind(|| {
        miniextendr_api::worker::with_r_thread(|| 42);
    });
    let payload = result.expect_err("with_r_thread must reject an uninitialized runtime");
    let message = miniextendr_api::unwind_protect::panic_payload_to_string(payload.as_ref());
    assert!(
        message.contains("miniextendr_runtime_init"),
        "expected init error message, got: {message}"
    );
}
