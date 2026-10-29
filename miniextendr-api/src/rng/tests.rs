use super::RngGuard;

/// Compiles only for a type that is not `Send`: for a `Send` type both
/// impls apply and the call below is ambiguous.
trait AmbiguousIfSend<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

/// Compiles only for a type that is not `Sync`.
trait AmbiguousIfSync<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSync<()> for T {}
impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}

/// A guard stays on the thread that loaded the state, and so does a
/// borrow of it: a draw can't run on another thread.
#[test]
fn guard_is_neither_send_nor_sync() {
    let _ = <RngGuard as AmbiguousIfSend<_>>::some_item;
    let _ = <RngGuard as AmbiguousIfSync<_>>::some_item;
    let _ = <&RngGuard as AmbiguousIfSend<_>>::some_item;
}
