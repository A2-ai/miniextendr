//! `TypedExternal` implementations for standard library types.
//!
//! This module provides `TypedExternal` implementations for common std types,
//! allowing them to be stored in `ExternalPtr<T>` without manual implementation.
//!
//! # Generic Types
//!
//! For generic types like `Vec<T>`, the type name does not include the type parameter
//! (e.g., `Vec<i32>` and `Vec<String>` both have type name "Vec"). This means type
//! checking at the R level won't distinguish between different instantiations.
//! If you need stricter type safety, create a newtype wrapper and derive `ExternalPtr`.

use crate::externalptr::TypedExternal;

/// Implement TypedExternal for a concrete type.
/// For standard library types, we use the simple name for both display and ID.
macro_rules! impl_te {
    ($ty:ty, $name:literal) => {
        impl TypedExternal for $ty {
            const TYPE_NAME: &'static str = $name;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!("std::", $name, "\0").as_bytes();
        }
    };
}

/// Implement TypedExternal for a generic type with 'static bounds.
/// For standard library types, we prefix the ID with "std::" for clarity.
macro_rules! impl_te_generic {
    (<$($g:ident),+> $ty:ty, $name:literal) => {
        impl<$($g: 'static),+> TypedExternal for $ty {
            const TYPE_NAME: &'static str = $name;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!("std::", $name, "\0").as_bytes();
        }
    };
    (<$($g:ident : $bound:path),+> $ty:ty, $name:literal) => {
        impl<$($g: $bound + 'static),+> TypedExternal for $ty {
            const TYPE_NAME: &'static str = $name;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!("std::", $name, "\0").as_bytes();
        }
    };
}

// =============================================================================
// Primitives
// =============================================================================

impl_te!(bool, "bool");
impl_te!(char, "char");
impl_te!(i8, "i8");
impl_te!(i16, "i16");
impl_te!(i32, "i32");
impl_te!(i64, "i64");
impl_te!(i128, "i128");
impl_te!(isize, "isize");
impl_te!(u8, "u8");
impl_te!(u16, "u16");
impl_te!(u32, "u32");
impl_te!(u64, "u64");
impl_te!(u128, "u128");
impl_te!(usize, "usize");
impl_te!(f32, "f32");
impl_te!(f64, "f64");

// =============================================================================
// Strings
// =============================================================================

impl_te!(String, "String");
impl_te!(std::ffi::CString, "CString");
impl_te!(std::ffi::OsString, "OsString");
impl_te!(std::path::PathBuf, "PathBuf");

// =============================================================================
// Collections
// =============================================================================

impl_te_generic!(<T> Vec<T>, "Vec");
impl_te_generic!(<T> std::collections::VecDeque<T>, "VecDeque");
impl_te_generic!(<T> std::collections::LinkedList<T>, "LinkedList");
impl_te_generic!(<T> std::collections::BinaryHeap<T>, "BinaryHeap");
impl_te_generic!(<K, V> std::collections::HashMap<K, V>, "HashMap");
impl_te_generic!(<K, V> std::collections::BTreeMap<K, V>, "BTreeMap");
impl_te_generic!(<T> std::collections::HashSet<T>, "HashSet");
impl_te_generic!(<T> std::collections::BTreeSet<T>, "BTreeSet");

// =============================================================================
// Smart Pointers
// =============================================================================

impl_te_generic!(<T> Box<T>, "Box");

// Box<[T]> is a special case - it's a fat pointer (Sized) wrapping a DST.
// Unlike Box<T> where T: Sized, Box<[T]> can store dynamically-sized slices.
// This is useful for ALTREP when you want fixed-size heap allocation without
// Vec's capacity overhead.
impl<T: 'static> TypedExternal for Box<[T]> {
    const TYPE_NAME: &'static str = "BoxSlice";
    const TYPE_NAME_CSTR: &'static [u8] = b"BoxSlice\0";
    const TYPE_ID_CSTR: &'static [u8] = b"std::BoxSlice\0";
}

impl_te_generic!(<T> std::rc::Rc<T>, "Rc");
impl_te_generic!(<T> std::sync::Arc<T>, "Arc");
impl_te_generic!(<T> std::cell::Cell<T>, "Cell");
impl_te_generic!(<T> std::cell::RefCell<T>, "RefCell");
impl_te_generic!(<T> std::cell::UnsafeCell<T>, "UnsafeCell");
impl_te_generic!(<T> std::sync::Mutex<T>, "Mutex");
impl_te_generic!(<T> std::sync::RwLock<T>, "RwLock");
impl_te_generic!(<T> std::sync::OnceLock<T>, "OnceLock");
impl_te_generic!(<T> std::pin::Pin<T>, "Pin");
// ManuallyDrop<T> shares T's type symbols, allowing ExternalPtr<ManuallyDrop<T>>
// to interoperate with ExternalPtr<T>. This is safe because ManuallyDrop<T> is
// #[repr(transparent)] and has identical memory layout to T.
impl<T: TypedExternal> TypedExternal for std::mem::ManuallyDrop<T> {
    const TYPE_NAME: &'static str = T::TYPE_NAME;
    const TYPE_NAME_CSTR: &'static [u8] = T::TYPE_NAME_CSTR;
    const TYPE_ID_CSTR: &'static [u8] = T::TYPE_ID_CSTR;
}
impl_te_generic!(<T> std::mem::MaybeUninit<T>, "MaybeUninit");
impl_te_generic!(<T> std::marker::PhantomData<T>, "PhantomData");

// =============================================================================
// Option / Result
// =============================================================================

impl_te_generic!(<T> Option<T>, "Option");
impl_te_generic!(<T, E> Result<T, E>, "Result");

// =============================================================================
// Ranges
// =============================================================================

impl_te_generic!(<T> std::ops::Range<T>, "Range");
impl_te_generic!(<T> std::ops::RangeInclusive<T>, "RangeInclusive");
impl_te_generic!(<T> std::ops::RangeFrom<T>, "RangeFrom");
impl_te_generic!(<T> std::ops::RangeTo<T>, "RangeTo");
impl_te_generic!(<T> std::ops::RangeToInclusive<T>, "RangeToInclusive");
impl_te!(std::ops::RangeFull, "RangeFull");

// =============================================================================
// I/O
// =============================================================================

impl_te!(std::fs::File, "File");
impl_te_generic!(<R: std::io::Read> std::io::BufReader<R>, "BufReader");
impl_te_generic!(<W: std::io::Write> std::io::BufWriter<W>, "BufWriter");
impl_te_generic!(<T> std::io::Cursor<T>, "Cursor");

// =============================================================================
// Time
// =============================================================================

impl_te!(std::time::Duration, "Duration");
impl_te!(std::time::Instant, "Instant");
impl_te!(std::time::SystemTime, "SystemTime");

// =============================================================================
// Networking
// =============================================================================

impl_te!(std::net::TcpStream, "TcpStream");
impl_te!(std::net::TcpListener, "TcpListener");
impl_te!(std::net::UdpSocket, "UdpSocket");
impl_te!(std::net::IpAddr, "IpAddr");
impl_te!(std::net::Ipv4Addr, "Ipv4Addr");
impl_te!(std::net::Ipv6Addr, "Ipv6Addr");
impl_te!(std::net::SocketAddr, "SocketAddr");
impl_te!(std::net::SocketAddrV4, "SocketAddrV4");
impl_te!(std::net::SocketAddrV6, "SocketAddrV6");

// =============================================================================
// Threading
// =============================================================================

impl_te!(std::thread::Thread, "Thread");
impl_te_generic!(<T> std::thread::JoinHandle<T>, "JoinHandle");
impl_te_generic!(<T> std::sync::mpsc::Sender<T>, "Sender");
impl_te_generic!(<T> std::sync::mpsc::SyncSender<T>, "SyncSender");
impl_te_generic!(<T> std::sync::mpsc::Receiver<T>, "Receiver");
impl_te!(std::sync::Barrier, "Barrier");
impl_te!(std::sync::BarrierWaitResult, "BarrierWaitResult");

// =============================================================================
// Atomics
// =============================================================================

impl_te!(std::sync::atomic::AtomicBool, "AtomicBool");
impl_te!(std::sync::atomic::AtomicI8, "AtomicI8");
impl_te!(std::sync::atomic::AtomicI16, "AtomicI16");
impl_te!(std::sync::atomic::AtomicI32, "AtomicI32");
impl_te!(std::sync::atomic::AtomicI64, "AtomicI64");
impl_te!(std::sync::atomic::AtomicIsize, "AtomicIsize");
impl_te!(std::sync::atomic::AtomicU8, "AtomicU8");
impl_te!(std::sync::atomic::AtomicU16, "AtomicU16");
impl_te!(std::sync::atomic::AtomicU32, "AtomicU32");
impl_te!(std::sync::atomic::AtomicU64, "AtomicU64");
impl_te!(std::sync::atomic::AtomicUsize, "AtomicUsize");

// =============================================================================
// Numeric wrappers
// =============================================================================

// Note: NonZero<T> requires T: ZeroablePrimitive (sealed trait), so we use aliases
impl_te!(std::num::NonZeroI8, "NonZero");
impl_te!(std::num::NonZeroI16, "NonZero");
impl_te!(std::num::NonZeroI32, "NonZero");
impl_te!(std::num::NonZeroI64, "NonZero");
impl_te!(std::num::NonZeroI128, "NonZero");
impl_te!(std::num::NonZeroIsize, "NonZero");
impl_te!(std::num::NonZeroU8, "NonZero");
impl_te!(std::num::NonZeroU16, "NonZero");
impl_te!(std::num::NonZeroU32, "NonZero");
impl_te!(std::num::NonZeroU64, "NonZero");
impl_te!(std::num::NonZeroU128, "NonZero");
impl_te!(std::num::NonZeroUsize, "NonZero");
impl_te_generic!(<T> std::num::Wrapping<T>, "Wrapping");
impl_te_generic!(<T> std::num::Saturating<T>, "Saturating");

// =============================================================================
// Tuples (1-12 elements)
// =============================================================================

impl_te_generic!(<A> (A,), "Tuple1");
impl_te_generic!(<A, B> (A, B), "Tuple2");
impl_te_generic!(<A, B, C> (A, B, C), "Tuple3");
impl_te_generic!(<A, B, C, D> (A, B, C, D), "Tuple4");
impl_te_generic!(<A, B, C, D, E> (A, B, C, D, E), "Tuple5");
impl_te_generic!(<A, B, C, D, E, F> (A, B, C, D, E, F), "Tuple6");
impl_te_generic!(<A, B, C, D, E, F, G> (A, B, C, D, E, F, G), "Tuple7");
impl_te_generic!(<A, B, C, D, E, F, G, H> (A, B, C, D, E, F, G, H), "Tuple8");
impl_te_generic!(<A, B, C, D, E, F, G, H, I> (A, B, C, D, E, F, G, H, I), "Tuple9");
impl_te_generic!(<A, B, C, D, E, F, G, H, I, J> (A, B, C, D, E, F, G, H, I, J), "Tuple10");
impl_te_generic!(<A, B, C, D, E, F, G, H, I, J, K> (A, B, C, D, E, F, G, H, I, J, K), "Tuple11");
impl_te_generic!(<A, B, C, D, E, F, G, H, I, J, K, L> (A, B, C, D, E, F, G, H, I, J, K, L), "Tuple12");

// =============================================================================
// Arrays (const generic)
// =============================================================================

impl<T: 'static, const N: usize> TypedExternal for [T; N] {
    const TYPE_NAME: &'static str = "Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"std::Array\0";
}

// =============================================================================
// Static slices
// =============================================================================
//
// `&'static [T]` is Sized (it's a fat pointer: ptr + len, 2 words) and satisfies
// 'static, so it can be stored directly in ExternalPtr.
//
// Use cases:
// - Const arrays: `static DATA: [i32; 5] = [1, 2, 3, 4, 5]; altrep(&DATA)`
// - Leaked data: `let leaked: &'static [i32] = Box::leak(vec![1, 2, 3].into_boxed_slice());`
// - Memory-mapped files with 'static lifetime
//
// Note: The data must genuinely live forever. If using Box::leak, the memory
// is never freed (intentional memory leak for the lifetime of the process).

impl<T: 'static> TypedExternal for &'static [T] {
    const TYPE_NAME: &'static str = "StaticSlice";
    const TYPE_NAME_CSTR: &'static [u8] = b"StaticSlice\0";
    const TYPE_ID_CSTR: &'static [u8] = b"std::StaticSlice\0";
}
