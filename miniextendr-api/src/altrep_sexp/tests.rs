use super::*;

/// Verify AltrepSexp is !Send and !Sync at compile time.
/// SEXP IS Send + Sync.
fn _assert_send_sync_properties() {
    fn requires_send<T: Send>() {}
    fn requires_sync<T: Sync>() {}

    // These must NOT compile — uncomment to verify:
    // requires_send::<AltrepSexp>();
    // requires_sync::<AltrepSexp>();

    // SEXP IS Send + Sync:
    requires_send::<SEXP>();
    requires_sync::<SEXP>();
}
