use miniextendr_api::RNativeType;

#[derive(Clone, Copy, RNativeType)]
struct RustLayout(i32);

#[repr(align(64))]
#[derive(Clone, Copy, RNativeType)]
struct Overaligned(i32);

#[repr(packed)]
#[derive(Clone, Copy, RNativeType)]
struct Packed(i32);

fn main() {}
