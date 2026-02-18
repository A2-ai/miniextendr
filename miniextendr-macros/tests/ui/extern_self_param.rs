use miniextendr_macros::miniextendr;

// `self` in a free extern function is a syntax error, but the macro has
// a defensive check for FnArg::Receiver.  This fixture documents
// what the compiler reports for this invalid construct.
#[miniextendr]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn bad_extern(self) {
}

fn main() {}
