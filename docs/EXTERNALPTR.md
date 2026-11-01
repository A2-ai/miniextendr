# ExternalPtr

`ExternalPtr<T>` is a Box-like owned pointer that wraps R's `EXTPTRSXP`. It lets you hand ownership of Rust-allocated data to R and let R's garbage collector decide when to drop it.

**Source**: `miniextendr-api/src/externalptr.rs`

## Why ExternalPtr Exists

R has no native way to hold arbitrary Rust data. `EXTPTRSXP` is R's mechanism for storing opaque C pointers, but it provides no type safety, no RAII cleanup, and no protection against use-after-free. `ExternalPtr<T>` wraps `EXTPTRSXP` with:

- **Type-safe access** via authoritative `Any::downcast` checks; the
  `TypedExternal` symbols are display and diagnostic metadata
- **Automatic cleanup** via R GC finalizer that calls `Drop`
- **Box-like ownership** (`Clone`, `into_inner`, `into_raw`, `leak`), with
  every read of the value checked: a `#[miniextendr]` function borrows it at
  the call (`&T` / `&mut T` parameters), and Rust code holding a handle
  borrows it with `borrow()` / `borrow_mut()`
- **Context-checked construction** -- `new()` runs on R's main thread and
  panics on any other

Checked conversions and erased-handle accessors reject foreign pointers before
reading their address as Rust storage. They require a symbol tag and an ordinary
protection list with a type-ID symbol, and exclude the reserved trait-ABI tag
(which identifies a different allocation layout). This is a metadata boundary,
not authentication: a foreign address deliberately forged with miniextendr
metadata is unsupported. `Any::downcast` remains the concrete-type check.

## When to Use ExternalPtr

| Strategy | Lifetime | Use Case |
|----------|----------|----------|
| `ExternalPtr` | Until R GCs | Rust data owned by R (structs returned to R) |
| `ProtectScope` | Within `.Call` | Temporary R allocations |
| Preserve list | Across `.Call`s | Long-lived R objects (not Rust values) |

Use `ExternalPtr` when you want R to own a Rust value and drop it when R garbage-collects the pointer. This is the standard mechanism for exposing Rust structs to R code.

## Creating an ExternalPtr

### With `#[derive(ExternalPtr)]` (recommended)

The derive macro implements `TypedExternal` and `IntoExternalPtr`, so returning your struct from a `#[miniextendr]` function automatically wraps it:

```rust
#[derive(ExternalPtr)]
pub struct MyData {
    pub value: f64,
}

#[miniextendr]
pub fn create_data(v: f64) -> MyData {
    MyData { value: v }  // Automatically wrapped in ExternalPtr
}
```

### Manual construction

```rust
let ptr = ExternalPtr::new(MyData { value: 3.14 });
```

`new()` works on R's main thread. It panics on a spawned or Rayon thread rather
than calling R off-main.

### Unchecked construction (ALTREP callbacks, main-thread-only code)

```rust
// SAFETY: must be on R's main thread
let ptr = unsafe { ExternalPtr::new_unchecked(MyData { value: 3.14 }) };
```

Skips main-thread routing and checks for performance-critical paths whose
caller has already established R's main-thread context.

### From raw pointers

```rust
let raw = Box::into_raw(Box::new(MyData { value: 1.0 }));
// SAFETY: raw was allocated by Box, is non-null, caller transfers ownership
let ptr = unsafe { ExternalPtr::from_raw(raw) };
```

## Passing Class Objects to Standalone Functions

A `#[miniextendr]` argument of an external type (`&T`, `&mut T`,
`ExternalPtrRef<'_, T>`, `ExternalPtrMut<'_, T>`, `ExternalPtr<T>`, or an
`Option` / `Vec` of them) accepts either the bare pointer a low-level
constructor returns, **or** the ergonomic class handle that wraps it —
`MyData$new(...)`, `S4Foo(...)`, etc. Conversion unwraps the handle
transparently, per class system, and keeps the pointer it unwrapped rooted
for as long as the Rust handle lives:

| Class system | Where the pointer lives | Shape |
|---|---|---|
| Env, S3 | the object itself | already a classed `EXTPTRSXP` — nothing to unwrap |
| R6 | `private$.ptr` | `.__enclos_env__` -> `private` -> `.ptr` |
| S4 | the `ptr` slot | `methods::slot(x, "ptr")` |
| S7 | the `.ptr` property | stored as a plain attribute (`attr(x, ".ptr")`) |
| hand-built list | the `.ptr` element | `structure(list(.ptr = ptr, log = ...), class = "Foo")` |
| hand-built environment | a `.ptr` binding | any environment binding `.ptr` directly |

The same resolution applies to the receiver of every class-system instance
method (`&self`, `&mut self`, `self`, and the receivers
`ExternalPtrRef<'_, Self>` / `ExternalPtrMut<'_, Self>` / `ExternalPtr<Self>`):
the generated prelude calls `resolve_receiver` on the object R
dispatched with, so an S3 class whose object is a list with R-side state next
to the handle dispatches into `#[miniextendr(s3)] impl` methods unchanged
(#1469; `S3_METHODS.md`, "Handles with R-visible state"). That shape is for
interop with R-defined objects: state the package owns belongs in sidecar
fields (`#[r_data]`, "RSidecar" below), which travel with the handle in every
class system. A bare pointer is recognised with one
`TYPEOF` compare; a receiver that yields no pointer raises
`expected a `Foo` object (an external pointer, or an R6/S4/S7 handle,
environment, or list carrying one in `.ptr`), got VECSXP`. Type safety is
unchanged: the pointer that comes out still has to `downcast` to `Foo`.

```rust
#[derive(ExternalPtr)]
pub struct MyData { pub value: f64 }

#[miniextendr(r6)]
impl MyData {
    pub fn new(value: f64) -> Self { Self { value } }
}

#[miniextendr]
pub fn total(items: Vec<ExternalPtr<MyData>>) -> f64 {
    items.iter().map(|d| d.borrow().value).sum()
}
```

```r
a <- MyData$new(1.0)
b <- MyData$new(2.0)
total(list(a, b))  # 3 -- no need to reach for a bare-pointer constructor
```

`Any::downcast` remains the type-safety authority: a handle for the *wrong*
Rust type still fails with the usual downcast type-mismatch error — this
only widens the accepted R-side shape, not type safety. Vctrs is the one
class system where this never applies: a vctrs object is a plain classed
vector with no Rust struct behind it (vctrs constructors return a payload
vector, not `Self`), so there's no `ExternalPtr` to unwrap in the first
place.

## Accessing the Value

The value an `ExternalPtr` stores sits in a `RefCell<T>` ("Reentrant Calls"
below), and every read of it is checked. There are two places it happens:

- **At the entry point.** A `#[miniextendr]` function or method takes the
  value as `&T` / `&mut T` (or `&self` / `&mut self`), and the generated
  wrapper borrows the cell for the call. The body works with plain
  references, and Rust's borrow checker covers everything after that.
- **Through a handle.** An `ExternalPtr<T>` that Rust code holds (one it built
  with `new`, a struct field, a `Vec` element, an `ExternalPtr<T>` argument)
  has no `Deref`. Its value is read with `borrow()` / `borrow_mut()`, which
  check the cell the way the wrapper does:

```rust
let mut ptr = ExternalPtr::new(MyData { value: 3.14 });
println!("{}", ptr.borrow().value);
ptr.borrow_mut().value = 2.71;
```

| Method | Returns | Notes |
|--------|---------|-------|
| `borrow()` | `ExternalPtrRef<'_, T>` | Shared borrow; raises `miniextendr_borrow_conflict` while the value is borrowed mutably |
| `borrow_mut()` | `ExternalPtrMut<'_, T>` | Exclusive borrow; raises it while the value is borrowed at all |
| `as_ptr()` | `*const T` | Raw pointer, no borrow, no ownership transfer |
| `as_sexp()` | `SEXP` | The underlying R object |
| `same_object()` | `ExternalPtr<T>` | A second handle to the same R object; no allocation, no copy |

`borrow()` / `borrow_mut()` panic for a type-erased `ExternalPtr<()>`, which
has no value of its own: downcast it first ("Type-Erased Pointers" below).

### Why there is no `Deref`

`RefCell::borrow()` returns a guard, and the value counts as borrowed only
while that guard lives. `Deref::deref(&self) -> &T` must return a bare
reference, so a guard made inside it drops when it returns (returning a
reference into it is E0515). That leaves `deref` three options, none of them
workable: check, then release, so the `&T` outlives the check and a callback can
`borrow_mut()` the same object alongside it; leak the guard, so the object
stays borrowed for good and every later `&mut` call fails; or skip the check
(`RefCell::as_ptr`), the unchecked read this model removes. The compiler can't
stand in for the check either: it tracks borrows of a handle, not of the
object, and two handles to one R object (`f(a, a)`, two `Vec` elements, a
stored copy) are separate Rust values, so `&*a` and `&mut *b` would alias
without an error. That's why std's `RefCell<T>` has no `Deref` either: you
hold the guard, and the guard dereferences. `ExternalPtrRef` /
`ExternalPtrMut` are that guard plus the R object. Take one per scope, not one
per field:

```rust
let c = ptr.borrow();   // one check, held until `c` drops
c.value + c.other       // plain field access through the view
```

`#[miniextendr]` signatures need none of this: for `&T` / `&mut T`
parameters, the generated wrapper holds the guard for the call.

### Views: `ExternalPtrRef` / `ExternalPtrMut`

`ExternalPtrRef<'a, T>` and `ExternalPtrMut<'a, T>` are a borrow of the value
together with the R object that owns it. They `Deref` (and `ExternalPtrMut`
`DerefMut`s) to `T`, can't outlive the borrow, can't consume the value, and
are `!Send`. A view is the form for code that needs the R object as well as
the value:

- `ExternalPtrRef::as_sexp(&view)` / `ExternalPtrMut::as_sexp(&view)` return
  the object (an associated function, so it never shadows a method of `T`);
- the `Sidecar<T>` accessors take a view ("`Sidecar<T>`: values the pointer
  roots" below).

`view.rb()` / `view.rb_mut()` reborrow a view for a helper that takes one,
without moving it:

```rust
fn bump(mut counter: ExternalPtrMut<'_, Counter>) {
    counter.value += 1;
}

let mut view = ptr.borrow_mut();
bump(view.rb_mut());
bump(view.rb_mut());
```

A view is also a parameter and receiver type, borrowed by the wrapper like
`&T` ("Receivers that reach the R object", "Arguments" below).

### `same_object()`: identity-preserving returns

`same_object()` makes a second owned handle to the R object a handle points
at, without allocating, so a function can return the object it was given or
one it stores:

```rust
#[miniextendr(r6)]
impl Holder {
    // `identical(h$inner(), h$inner())` is TRUE: the same R object each time.
    pub fn inner(&self) -> ExternalPtr<Counter> {
        self.inner.same_object()
    }
}
```

`clone()` would allocate a fresh R object with a copy of the `T`.
`same_object()` takes no borrow and gives no access to the value: reads
through the new handle are checked like any other.

## Consuming and Dropping

| Method | Effect |
|--------|--------|
| `into_inner(this)` | Moves value out, deallocates, neutralizes finalizer |
| `into_raw(this)` | Moves the value into a new `Box<T>` and returns its `*mut T` (not the stored address), neutralizes finalizer, caller owns memory |
| `leak(this)` | Returns `&'a mut T`, memory is never freed |

R's GC finalizer handles cleanup when the `ExternalPtr` goes out of scope in Rust without being explicitly consumed. The Rust `Drop` impl is a no-op to avoid double-free.

`into_inner` and `into_raw` raise the `miniextendr_borrow_conflict` error while
a running call (or a view of another handle) holds the value. `from_raw` boxes
the value again, so its address changes too.

### Panicking destructors are not recoverable

The GC finalizer is an `extern "C"` function. Unwinding through it is
undefined behavior. miniextendr wraps every ExternalPtr finalizer in
`drop_catching_panic`, which calls `std::panic::catch_unwind` around the
destructor; if a `Drop` impl panics the helper prints the panic message
to stderr and calls `std::process::abort()`. That's the only safe
response: R's GC cannot resume after a cross-ABI unwind.

This means: a panic in `impl Drop for YourType` crashes the R session.
Treat destructors as infallible. Put anything that can fail (file
writes, network, mutex takedown) behind an explicit `close()` /
`finalize()` method that the user calls, and keep `Drop` limited to
plain memory release.

## Receivers that reach the R object

`&self` / `&mut self` give a method the value. A method that also needs the
R object takes a view as its receiver, which the wrapper borrows for the call
like `&self` / `&mut self`:

```rust
#[miniextendr(env)]
impl MyType {
    pub fn value(&self) -> i32 { self.value }

    // The object itself, for identity
    pub fn object(self: ExternalPtrRef<'_, Self>) -> SEXP {
        ExternalPtrRef::as_sexp(&self)
    }

    // A Sidecar<T> field (`#[r_data] keys: Sidecar<Vec<i32>>`) is read
    // and written through the view
    pub fn key_count(self: ExternalPtrRef<'_, Self>) -> usize {
        Self::keys(&self).len()
    }

    pub fn push_key(mut self: ExternalPtrMut<'_, Self>, key: i32) {
        let mut keys = Self::keys(&self);
        keys.push(key);
        Self::set_keys(&mut self, keys);
    }

    // The handle, which the wrapper does not borrow
    pub fn is_null_ptr(self: ExternalPtr<Self>) -> bool {
        self.is_null()
    }
}
```

The macro rewrites `self` to an internal binding so the pattern compiles on
stable Rust (no `arbitrary_self_types` required).

The view receivers are the form for a method that needs a `Sidecar<T>`
field's value: the accessors the derive generates are associated functions
that take the view (`Self::keys(&self)`, `Self::set_keys(&mut self, v)`),
since the struct holds nothing that points back at its pointer. See
"`Sidecar<T>`: values the pointer roots" below.

`self: ExternalPtr<Self>` is the handle: the wrapper takes no borrow, so the
body reads through `borrow()` / `borrow_mut()`, and may store the handle,
return it, or move the value out with `ExternalPtr::into_inner`, which
leaves the R object without a value.

`self: &ExternalPtr<Self>` and `self: &mut ExternalPtr<Self>` are compile
errors that name the view to use instead.

## Reentrant Calls

A method that calls back into R can meet its own object again: the callback
may read or set a field of it, or call another of its methods, while the first
method still holds `&self` or `&mut self`. So the value an `ExternalPtr` stores
sits in a `RefCell<T>`, and the generated code borrows that cell for the whole
call, through the conversion of its result:

- `&self` methods, `ExternalPtrRef` receivers, `&T` and `ExternalPtrRef`
  arguments (below) and field reads borrow it shared;
- `&mut self` and by-value methods, `ExternalPtrMut` receivers, `&mut T` and
  `ExternalPtrMut` arguments and field writes borrow it exclusively.

Shared borrows nest. A call that conflicts with one still running raises an R
error of class `miniextendr_borrow_conflict` instead of making a second
reference, and leaves the object as it was. A trait method called through a
View, from this package or another, borrows the same cell (see `TRAIT_ABI.md`).

That borrow is the only run-time check a call's own arguments need: the R
object reaches Rust only through a generated wrapper, on R's main thread, so
the wrapper sees every argument and the receiver, and a callback meets its
borrow. Everything after the entry point is ordinary borrowck. The remaining
run-time checks are on handles Rust keeps ("Handles Rust code keeps" below).

### Arguments

A parameter typed `&T` or `&mut T` for an external type `T` (one that
implements `TypedExternal`, as `#[derive(ExternalPtr)]` does), or the view
`ExternalPtrRef<'_, T>` / `ExternalPtrMut<'_, T>`, or an `Option` of either,
holds its object's value for the whole call like a receiver does:
exclusively for `&mut T` / `ExternalPtrMut`, shared otherwise. `NULL` for an
`Option` holds nothing. The borrow is taken after every argument is converted
(and after a trait method's alias checks), so an error that the argument
conversion raises leaves no object borrowed.

```rust
#[miniextendr]
pub fn absorb(target: &mut Counter, source: &Counter) -> i32 {
    target.value += source.value;
    target.value
}
```

```r
absorb(a, b)  # fine
absorb(a, a)  # error of class miniextendr_borrow_conflict, `a` unchanged
```

The same error meets a callback into R that reaches an argument's object
while the function still holds it: reading a field of a `&mut T` argument,
writing one of a `&T` argument, calling a method that conflicts, or passing
the object to another function that does. A trait method's `&Self` /
`&mut Self` arguments borrow the same way, after the check that rejects their
aliasing each other or the receiver.

A `&T` whose `T` isn't `TypedExternal` fails to compile at the parameter
(`` `T` is not a type an `ExternalPtr` stores ``). The reference parameters
the macro converts from R vectors keep their meaning: `&str`, `&[T]` /
`&mut [T]`, `&i32` / `&f64` / `&u8` / `&RLogical` / `&Rcomplex` (a length-one
vector's element), dots, and types with a `TryFromSexp` precondition. A
`'static` lifetime on a borrowed parameter is a compile error: the borrow
ends with the call.

If a reference has its own `TryFromSexp` implementation, select that
conversion explicitly. The macro cannot resolve a type alias or discover
trait implementations while choosing between native and external borrows:

```rust
type Text = str;

#[miniextendr]
pub fn text_length(#[miniextendr(convert)] text: &Text) -> usize {
    text.len()
}

#[miniextendr(env)]
impl MyType {
    #[miniextendr(convert(text))]
    pub fn text_length(&self, text: &Text) -> usize {
        text.len()
    }
}
```

The method-level `convert(text, other)` form also works on trait impl methods.
The option accepts `&T`, `&mut T`, and `Option` of either. Shared references
use `TryFromRoot`, whose blanket implementation delegates to
`TryFromSexp for &'static T` (or `Option<&'static T>`) and shortens the result
to the call's root lifetime. Mutable references use their raw `TryFromSexp`
implementation; a root alone cannot establish exclusive access. Both forms
run on the main thread and participate in native alias checks through the
converter's `NATIVE_BORROW` metadata. Custom converters borrowing native
storage must supply that metadata, as other `TryFromSexp` implementations do;
for a `&mut` parameter its absence (the default `None`, which the alias check
skips) is a compile error.
The option preserves the converter's NULL, NA, and error semantics; an
optional custom reference therefore needs its own corresponding conversion.
It does not apply to receivers or `Self` arguments, and cannot be combined
with a parameter's `coerce`, `match_arg`, or `choices` option. It overrides
function-wide coercion for that parameter.

An `ExternalPtr<T>` parameter is a handle: the wrapper takes no borrow, and
the body reads the value with `borrow()` / `borrow_mut()`, keeps the handle,
returns it, or moves the value out with `ExternalPtr::into_inner` (which
fails only while another running call holds the value, and leaves the R
object without one). Elements of a
`Vec<ExternalPtr<T>>` and the `ExternalPtr<T>` arguments of a trait method
called through a View are handles too.

### Unwinding

The borrows end when the call returns or unwinds. An R error that jumps over
the call's frames never ends them: a bare `Rf_eval`, `R_forceAndCall` or
`Rf_applyClosure` of R code that raises, or a direct `R_CheckUserInterrupt()`
with an interrupt pending, longjmps from inside the method or function
straight to the generated wrapper's `R_UnwindProtect`. No destructor in
between runs, and the objects stay borrowed: every later call on them raises
`miniextendr_borrow_conflict` for the rest of the session (#2093). Wrapping
the call in `with_r_unwind_protect` does not change that, since it resumes
the R error with `R_ContinueUnwind`.

A method or function that calls back into R should go through the
`expression` evaluators, which stop an R error before it crosses a Rust frame:

- `eval_with_handlers` (or `RCall::eval_with_handlers`) keeps the caller's
  condition handlers and carries an R error through the call's frames as an
  unwind, so the borrows are released before R carries on;
- `RCall::eval` and `try_eval_with_handlers` return it as `Err(REvalError)`;
- `Quoted::eval()` evaluates an unevaluated argument.

`#[miniextendr(check_interrupt)]` checks for an interrupt once, before any
borrow is taken; a safe check inside a long loop is #2101. The lint MXL305
flags `Rf_eval`, `R_forceAndCall`, `Rf_applyClosure` and `R_CheckUserInterrupt`
in `#[miniextendr]` bodies.

### Handles Rust code keeps

A handle Rust code keeps (a vector's element, a handle stored in a struct,
one built from a `SEXP`) is read with `borrow()` / `borrow_mut()`. They check
the cell as the generated code does, so they meet a running call's borrow,
or another view of the same object:

```rust
#[miniextendr]
pub fn absorb_all(mut items: Vec<ExternalPtr<Counter>>) -> i32 {
    let (first, rest) = items.split_first_mut().expect("at least one counter");
    let mut first = first.borrow_mut();
    for other in rest {
        // The same counter twice raises `miniextendr_borrow_conflict`.
        first.value += other.borrow().value;
    }
    first.value
}
```

A view lasts as long as the Rust scope that holds it, so a function keeps
one across a call into R only by holding it there; drop it first to let the
callback use the object.

## Display and Diagnostic Metadata with TypedExternal

Every `ExternalPtr<T>` requires `T: TypedExternal`. This trait provides two identifiers stored in the SEXP:

- **`TYPE_NAME_CSTR`** -- Short display name, stored in the `tag` slot (visible when printing in R)
- **`TYPE_ID_CSTR`** -- Namespaced identifier (`crate@version::module::Type`), stored in `prot[0]` for mismatch diagnostics

R interns both strings with `Rf_install`, but concrete type checking uses the
stored `Any` value's Rust `TypeId`. The symbols make R-facing names and errors
readable; they are not a substitute for the downcast.

### Implementing TypedExternal

**Via derive (recommended):**

```rust
#[derive(ExternalPtr)]
pub struct MyData { /* ... */ }
```

**Via macro:**

```rust
impl_typed_external!(MyData);
// also works for generic types:
impl_typed_external!(MyWrapper<i32>);
```

**Manually:**

```rust
impl TypedExternal for MyData {
    const TYPE_NAME: &'static str = "MyData";
    const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
    const TYPE_ID_CSTR: &'static [u8] =
        concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
                "::", module_path!(), "::MyData\0").as_bytes();
}
```

### Built-in TypedExternal Implementations

**Source**: `miniextendr-api/src/externalptr_std.rs`

The following standard library types have built-in `TypedExternal` impls, so they can be stored in `ExternalPtr<T>` without any manual implementation:

| Category | Types |
|----------|-------|
| **Primitives** | `bool`, `char`, `i8`–`i128`, `isize`, `u8`–`u128`, `usize`, `f32`, `f64` |
| **Strings** | `String`, `CString`, `OsString`, `PathBuf` |
| **Collections** | `Vec<T>`, `VecDeque<T>`, `LinkedList<T>`, `BinaryHeap<T>`, `HashMap<K,V>`, `BTreeMap<K,V>`, `HashSet<T>`, `BTreeSet<T>` |
| **Smart pointers** | `Box<T>`, `Box<[T]>`, `Rc<T>`, `Arc<T>`, `Cell<T>`, `RefCell<T>`, `UnsafeCell<T>`, `Mutex<T>`, `RwLock<T>`, `OnceLock<T>`, `Pin<T>`, `ManuallyDrop<T>`, `MaybeUninit<T>`, `PhantomData<T>` |
| **Option/Result** | `Option<T>`, `Result<T, E>` |
| **Ranges** | `Range<T>`, `RangeInclusive<T>`, `RangeFrom<T>`, `RangeTo<T>`, `RangeToInclusive<T>`, `RangeFull` |
| **I/O** | `File`, `BufReader<R>`, `BufWriter<W>`, `Cursor<T>` |
| **Time** | `Duration`, `Instant`, `SystemTime` |
| **Networking** | `TcpStream`, `TcpListener`, `UdpSocket`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6` |
| **Threading** | `Thread`, `JoinHandle<T>`, `Sender<T>`, `SyncSender<T>`, `Receiver<T>`, `Barrier`, `BarrierWaitResult` |
| **Atomics** | `AtomicBool`, `AtomicI8`–`AtomicI64`, `AtomicIsize`, `AtomicU8`–`AtomicU64`, `AtomicUsize` |
| **Numeric wrappers** | `NonZeroI8`–`NonZeroI128`, `NonZeroIsize`, `NonZeroU8`–`NonZeroU128`, `NonZeroUsize`, `Wrapping<T>`, `Saturating<T>` |
| **Tuples** | `(A,)` through `(A, B, C, D, E, F, G, H, I, J, K, L)` (1–12 elements) |
| **Arrays** | `[T; N]` (const generic, any size) |
| **Static slices** | `&'static [T]`, `&'static mut [T]` |

**Note on generic types**: For generic types like `Vec<T>`, the display symbol
does not include the type parameter (for example, `Vec<i32>` and `Vec<String>`
both display as `"Vec"`). `Any::downcast` still distinguishes the full concrete
types. Use a derived newtype when distinct R-facing names and clearer mismatch
diagnostics matter.

**Note on `ManuallyDrop<T>`**: It shares `T`'s display and diagnostic symbols,
but remains a distinct concrete type to `Any::downcast`. Matching symbols do
not make `ExternalPtr<ManuallyDrop<T>>` interchangeable with `ExternalPtr<T>`.

**Note on static slices**: `&'static [T]` and `&'static mut [T]` are fat pointers (ptr + len) that satisfy `'static + Sized`, so they can be stored directly in `ExternalPtr`. Use cases include const arrays (`&DATA`), leaked data (`Box::leak`), and memory-mapped files.

### IntoExternalPtr

The `IntoExternalPtr` marker trait triggers a blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>` when returning from `#[miniextendr]` functions. `#[derive(ExternalPtr)]` implements both `TypedExternal` and `IntoExternalPtr`.

## Concrete Type Safety and Cross-Package Dispatch

`ExternalPtr<T>` stores `Box<Box<dyn Any>>`. `wrap_sexp()` asks that `Any`
value for `T` with `downcast_mut`; `wrap_sexp_with_error()` reports the stored
diagnostic name on failure. `TYPE_ID_CSTR` (`crate@version::module::Type`) makes
that message unambiguous, but does not decide whether the downcast succeeds.

Do not use matching R symbols as a cross-package concrete-type contract. For
cross-package behavior without a shared concrete Rust type, export a trait and
use the stable tag/vtable protocol described in [Trait ABI](TRAIT_ABI.md).

## Type-Erased Pointers (ErasedExternalPtr)

```rust
pub type ErasedExternalPtr = ExternalPtr<()>;
```

`ErasedExternalPtr` opens a miniextendr-created `EXTPTRSXP` without selecting a
concrete pointee type first. Its address must still contain miniextendr's
`Box<Box<dyn Any>>` representation; it is not a wrapper for arbitrary foreign
external pointers. It is useful for:

- Inspecting the stored type name before downcasting
- Inspecting miniextendr external pointers when the concrete Rust type is not
  known statically

```rust
let erased = unsafe { ErasedExternalPtr::from_sexp(some_sexp) };

// Check what type is stored
if erased.is::<MyData>() {
    let data: ExternalPtrRef<'_, MyData> = erased.downcast_ref::<MyData>().unwrap();
}

// Or read the stored type name
if let Some(name) = erased.stored_type_name() {
    println!("stored type: {}", name);
}
```

Methods on `ErasedExternalPtr`:

| Method | Returns |
|--------|---------|
| `is::<T>()` | `bool` -- does the stored type match `T`? |
| `downcast_ref::<T>()` | `Option<ExternalPtrRef<'_, T>>`, a shared borrow like `borrow()` |
| `downcast_mut::<T>()` | `Option<ExternalPtrMut<'_, T>>`, an exclusive borrow like `borrow_mut()` |
| `stored_type_name()` | `Option<&'static str>` |

## ExternalSlice

`ExternalSlice<T>` stores a `Vec<T>` as a raw pointer + length + capacity, suitable for wrapping in `ExternalPtr`:

```rust
impl_typed_external!(ExternalSlice<f64>);

let data = vec![1.0, 2.0, 3.0];
let ptr = ExternalPtr::new(ExternalSlice::new(data));
assert_eq!(ptr.borrow().as_slice(), &[1.0, 2.0, 3.0]);
```

This is useful when you need R to own a Rust slice and access it by index (e.g., in ALTREP `elt` callbacks).

| Method | Returns |
|--------|---------|
| `new(vec)` | Creates from `Vec<T>` |
| `from_boxed(boxed)` | Creates from `Box<[T]>` |
| `as_slice()` | `&[T]` |
| `as_mut_slice()` | `&mut [T]` |
| `len()` / `is_empty()` | Length queries |

## ALTREP data1/data2 Helpers

ALTREP objects have two data slots (`data1`, `data2`). These helpers extract typed `ExternalPtr`s from those slots:

```rust
// In an ALTREP callback:
fn length(x: SEXP) -> R_xlen_t {
    match unsafe { altrep_data1_as::<MyAltrepData>(x) } {
        Some(ext) => ext.borrow().data.len() as R_xlen_t,
        None => 0,
    }
}
```

| Function | Description |
|----------|-------------|
| `altrep_data1_as::<T>(x)` | Extract data1 as `ExternalPtr<T>` with type check |
| `altrep_data1_as_unchecked::<T>(x)` | Same, skips thread safety assertions |
| `altrep_data2_as::<T>(x)` | Extract data2 as `ExternalPtr<T>` with type check |
| `altrep_data2_as_unchecked::<T>(x)` | Same, skips thread safety assertions |
| `altrep_data1_mut::<T>(x)` | Mutable `&'static mut T` reference from data1 |
| `altrep_data1_mut_unchecked::<T>(x)` | Same, skips thread safety assertions |

The `_unchecked` variants are for performance-critical ALTREP callbacks where you are guaranteed to be on the main thread.

`altrep_data1_mut` and the ALTREP derives' data extraction read the value
without borrowing its `RefCell`, so a class method or argument holding the
same object exclusively doesn't stop them (#2103). A handle from
`altrep_data1_as` / `altrep_data2_as` is read with `borrow()`, like any other.

## RSidecar (R Data Fields)

`RSidecar` is a zero-sized marker type that enables R-facing getter/setter generation for struct fields annotated with `#[r_data]`:

```rust
#[derive(ExternalPtr)]
pub struct MyType {
    pub x: i32,

    #[r_data]
    r: RSidecar,          // Enables R wrapper generation

    #[r_data]
    pub count: i32,       // Generates MyType_get_count() / MyType_set_count()

    #[r_data]
    pub name: String,     // Generates MyType_get_name() / MyType_set_name()

    #[r_data(name = "keys")]
    pub r_keys: Sidecar<Vec<i32>>,  // Generates MyType_get_keys() / MyType_set_keys()

    #[r_data(get = "Self::total")]
    pub total: Computed,  // Generates MyType_get_total() only

    #[r_data(read_only)]
    pub id: i32,          // Generates MyType_get_id() only

    #[r_data(null_if_none)]
    pub label: Option<String>,  // NULL for None (a plain Option<String> reads NA)

    #[r_data(cache)]
    pub values: Vec<f64>,  // Converted on the first read, then returned as is
}
```

Only `pub` fields with `#[r_data]` get R wrapper functions. With
`#[miniextendr(r6(r_data_accessors))]` on the impl they are also R6 active
bindings (`obj$count`, `obj$count <- 2L`), with `s7(r_data_accessors)` S7
properties (`obj@count`), and with `s3(r_data_accessors)`,
`s4(r_data_accessors)` or `env(r_data_accessors)` `$` / `[[` methods that read
them (by the exact name, never a prefix; on an S3 bare pointer `[[` also
takes a whole-number position, and an unknown name is an error unless
`lookup = "list"` reads `NULL`), `$<-` / `[[<-` methods that write them in place
and `.DollarNames()` for completion, plus `names()` / `as.list()` on an S3
class (`r_data_accessors = "get"` for the readers only, `r_data_accessors =
"read_only"` for readers and a `$<-` / `[[<-` that refuses every field,
#2268). On an S3 bare
pointer `[[`, `as.list()` and `[[<-` refuse an unused argument, as
`dots = empty` does, and `[[` takes `exact`; so does an env class's `[[`
(#2174). With
`fallback = "Self::f"` next to the option, a name no field has is read by
`fn f(&self, name: &str) -> Option<T>` on the Rust value, and
`fallback_names = "Self::g"` adds `g(&self) -> Vec<String>` to the
completions (#2150). Under that S3, S4
or env field syntax the `MyType_get_*()` / `MyType_set_*()` functions are
internal, for the package's own R code (under `r_data_accessors =
"read_only"` there are no `MyType_set_*()`); otherwise they are exported. vctrs
classes get only those functions. See `CLASS_SYSTEMS.md`, "Direct Field
Access via Sidecar".

| Field type | Where the value lives | Getter | Setter |
|---|---|---|---|
| `i32`, `f64`, `bool`, `u8` | the Rust struct | a fresh length-1 vector | reads `value` with `Rf_as*` (so the integer setter truncates `3.5` and accepts `"3"`, #1675) |
| any `Clone + IntoR + TryFromSexp` type | the Rust struct | converts a clone with `IntoR` (the struct is only borrowed) | converts with `TryFromSexp`, or raises `'name' must be ...` |
| `Option<T>` with `#[r_data(null_if_none)]`, or `Nullable<T>` | the Rust struct | `NULL` for `None` whatever `T` is (a plain `Option<String>` reads `NA_character_`), else the value (#2270) | `NULL` and `NA` store `None`, as for any `Option<T>`; a bad value raises `'name' must be NULL or ...` |
| `Sidecar<T>` | the pointer's `prot` list | the stored R value | validates `value` with `TryFromSexp::<T>` and stores the value R gave |
| `Computed` with `#[r_data(get = "Self::f")]` | nowhere: `f(&self) -> impl IntoR` computes it | converts the function's value with `IntoR`; a `Result` is an error boundary: `Ok` reads the value, `Err` raises the classed error, `Err(())` of a `Result<T, ()>` reads `NULL` | none; the field syntax and an R6 binding raise `miniextendr_read_only_field`, an S7 property has no setter |
| any of the above but `Computed`, with `#[r_data(read_only)]` | as above | as above | none, as for `Computed` (#2268), so a struct field's type needs no `TryFromSexp`; the Rust accessors of a `Sidecar<T>` stay |
| a converted (non-scalar) struct field with `#[r_data(cache)]` | the Rust struct, and the R value the first read made | that same R object, until the value is next borrowed exclusively (#2271) | as without `cache`; the write drops the kept value |

A getter whose value fails to convert raises the error a function's failed
return conversion raises (`e$kind == "result_err"`, the error's
`RConditionData` classes and fields, else its `Display` text; #2272). A
computed getter returning `Result<T, E>` raises `Err(e)` as a `#[miniextendr]`
function returning it would: `E`'s `RConditionData` classes and fields, else
its serde shape (under the `serde` feature), else its `Debug` text (#2269). To
raise a typed error by hand inside a getter, use
`RCondition::from_data(&e).raise()`:

```rust
impl Engine {
    fn frame(&self) -> Result<BuiltDataFrame, EngineError> { self.build_frame() }
    fn rows(&self) -> i32 {
        self.count_rows().unwrap_or_else(|e| RCondition::from_data(&e).raise())
    }
}
```

`#[r_data(name = "keys")]` sets the field's R name, used by every R-facing
string (the field list, the `$` / `[[` labels, the R6 binding or S7 property,
`MyType_get_keys()` / `MyType_set_keys()`, the setter's message); the Rust
accessors (`MyType::r_keys(&ptr)`) and the C symbols keep the Rust
identifier. The name must be syntactic, unique in the type and not `.ptr`.
A `Sidecar` field's R name is also its name in a save (below), so renaming
the Rust field is safe for saves and renaming the R name is not (#2215). A
`Computed` field must be `pub`; under `#[externalptr(r6)]` /
`#[externalptr(s7)]` it is a read-only active binding / property (#1888).
`#[r_data(read_only)]` goes on a `pub` field that isn't `Computed`, without
`setter = ...`; the refusal carries the type's `field_error(class = ...)`
classes and its `field_error(read_only_hint = "...")` sentence (`%s` is the
field's R name). See `CLASS_SYSTEMS.md`, "Read-only fields".
`#[r_data(null_if_none)]` goes on a `pub` `Option<T>` struct field; a
`Sidecar<T>` field keeps the R value it was given, so a `Sidecar<Nullable<T>>`
keeps `NULL` for `None`, and a `Computed` field's function returns
`Nullable<T>`. See `CLASS_SYSTEMS.md`, "Fields that read `NULL` for `None`".

A struct field is a view of the Rust value: every read converts it afresh, and
every write converts into Rust or fails, so no R copy can drift from it. A
computed field is a view too, through its function. `#[r_data(cache)]` trades
the fresh conversion for speed on a large field: the first read keeps the R
object it made, and every later read returns that object, until the value is
borrowed exclusively (a `&mut self` method, a `&mut T` or `ExternalPtrMut`
argument, an R setter of any field, `ExternalPtr::borrow_mut`, a consuming
method, `into_inner`), which drops every kept value of the object. Each type
checks a constant first (`TypedExternal::CACHED_FIELD_COUNT`), so a type
without cached fields pays nothing. The kept objects sit in the pool that
roots owning `ExternalPtr` handles, not in the `prot` list, so a save never
carries them, a `readRDS()`ed or cloned object starts with none, and the
finalizer releases them. A kept object is marked not mutable: R code that
modifies a value it read (`v[1] <- 0`) copies it first. A change no exclusive
borrow covers is not seen, and the field reads its old value: interior
mutability behind `&self` (`Cell`, `RefCell`, `Mutex`, atomics) and the
ALTREP `data1` paths that skip the borrow (`as_mut_unchecked`). The value is
stale, never unsound. See `CLASS_SYSTEMS.md`, "Cached fields".

A bare R view under `#[r_data]` is a compile error: the GC does not trace
Rust fields. Use `Sidecar<List>`, `Sidecar<Option<List>>`, `Sidecar<DataFrame>`
or `Sidecar<SEXP>` to keep the value in the pointer's protection list. The
check also follows known standard containers, arrays and tuples, so
`Vec<Option<List>>` needs `Sidecar<Vec<Option<List>>>` too. Owned handles such
as `BuiltList`, `BuiltDataFrame` and `Preserved` retain their own roots.

The derive checks the written type. It cannot inspect aliases or fields
inside user-defined wrappers; those must provide their own rooting or use
`Sidecar<T>`.

### `Sidecar<T>`: values the pointer roots

A `Sidecar<T>` field's value lives in the pointer's `prot` list ("SEXP Layout"
below), which roots it as long as the pointer is reachable. Rust reads and
writes it through the pointer: the derive generates associated functions on
the struct that take a view or an `ExternalPtr` handle, with the field's
visibility; the `#[r_data(...)]` option selects them:

| `#[r_data(...)]` | Rust accessors |
|---|---|
| `ref` | `fn keys(ptr: &impl SidecarOwner<Self>) -> T` |
| `mut` | `fn set_keys(ptr: &mut impl SidecarOwnerMut<Self>, value: T)` |
| `ref, mut`, or a bare `#[r_data]` | both |

`T` is any `IntoR + TryFromSexp` type. Rust code sees no `SEXP`s or slot
indices: a plain constructor fills the field with `Sidecar::new(v)` (or
`Sidecar::default()`, which is `Sidecar::new(T::default())`), and a method
that needs the value takes a view as its receiver ("Receivers that reach the
R object" above), reads it with `Self::keys(&self)` and writes it with
`Self::set_keys(&mut self, v)`:

```rust
#[miniextendr(r6(r_data_accessors))]
impl MyType {
    pub fn new(n: i32) -> Self {
        MyType {
            x: 0,
            r: RSidecar,
            count: n,
            name: String::new(),
            keys: Sidecar::new((1..=n).collect()),
        }
    }

    pub fn n_keys(self: ExternalPtrRef<'_, Self>) -> usize {
        Self::keys(&self).len()
    }

    pub fn push_key(mut self: ExternalPtrMut<'_, Self>, key: i32) {
        let mut keys = Self::keys(&self);
        keys.push(key);
        Self::set_keys(&mut self, keys);
    }
}
```

Any other Rust code with a handle makes the same calls: `MyType::keys(&ptr)`,
`MyType::set_keys(&mut ptr, v)`. Through a handle, the getter takes a shared
borrow and the setter an exclusive one, for the call.

The struct holds nothing that points back at its pointer, so a plain
`&self` / `&mut self` receiver cannot reach the values. Custom traits can
ask for a borrowed owner explicitly:

```rust
#[miniextendr]
pub trait KeyAccess: TypedExternal {
    fn keys_copy(self: ExternalPtrRef<'_, Self>) -> Vec<i32>;
    fn replace_keys(self: ExternalPtrMut<'_, Self>, keys: Vec<i32>);
}

#[miniextendr]
impl KeyAccess for MyType {
    fn keys_copy(self: ExternalPtrRef<'_, Self>) -> Vec<i32> {
        Self::keys(&self)
    }
    fn replace_keys(mut self: ExternalPtrMut<'_, Self>, keys: Vec<i32>) {
        Self::set_keys(&mut self, keys);
    }
}
```

Both `.Call` wrappers and cross-package trait shims supply the owner. These
views borrow the value and its GC root for the call; they cannot consume
the Rust value or cross threads. The getter accepts either view or an
ordinary `ExternalPtr`; the setter requires `ExternalPtrMut` or a mutable
handle. Native Rust calls use `KeyAccess::keys_copy(ptr.borrow())` or
`KeyAccess::replace_keys(ptr.borrow_mut(), values)`. The macro rewrites the
special receiver to an associated-function parameter on stable Rust, so use
this UFCS form in Rust. Default trait bodies follow the same rule.

A standard trait with a fixed `&self` signature, such as `Display`, still needs
an explicit local adapter carrying the view, or an inherent method with a
view receiver (#1880). High-level ALTREP data traits also receive only
`&T`. ALTREP low-level callbacks can recover the owner explicitly with
`altrep_data1_as_unchecked::<Self>(x)` and call `Self::keys(&owner)`. See
[Sidecars in callbacks](ALTREP.md#sidecars-in-callbacks).

How the value moves:

- `Sidecar::new(v)` converts `v` to its R object at once and holds it as a
  *pending* value in the struct, rooted until a slot holds it (#2216), so a
  fresh `SEXP` or list passed to it is safe from that moment. Wrapping the
  struct (`ExternalPtr::new` and the other constructors, or the trait ABI's
  `mx_wrap` for erased storage) moves every pending value into the
  pointer's `prot` list. From then on the value lives in R only and the
  field is empty. `Sidecar::new` runs on R's main
  thread; any other thread panics.
- Every accessor, Rust or R, first moves a pending value of its field into
  the `prot` list. So `*ptr.borrow_mut() = MyType::new(..)` takes
  effect at the next access from either side; a `saveRDS()` before any access
  writes the old slot values.
- `mem::swap(a, b)` of two `&mut MyType` (arguments, or
  `&mut *ptr.borrow_mut()`) and `*view = value` are plain moves of the
  struct: the sidecar values stay with each pointer.
- `ExternalPtr::into_inner` / `into_raw` read each slot back into its field
  as a pending value, so wrapping the value again keeps the values, straight
  after a swap included. A `self -> Self` method runs with the values left in
  the pointer; its result is written back into the same pointer, and a field
  it filled with `Sidecar::new` replaces that slot.
- `ExternalPtr::clone` and `clone_from` copy the struct and share the slot
  values as R objects, which are immutable: a later write to a field on
  either side replaces that side's slot only. `MyType::clone()` on the bare
  struct shares its pending R objects only.
- A private `Sidecar<T>` gets private accessors and no R accessor. That is how
  Rust keeps an R value (`Sidecar<Option<List>>`, `Sidecar<SEXP>`) alive for
  as long as the object. A `Sidecar<SEXP>` getter returns a view the slot
  roots until the next write to that field.
- The accessors run on R's main thread only. Off it (a thread the package
  spawned, a Rayon task) an accessor panics with a message naming the
  field and the type.
- A conversion failure in a getter (the slot holds an R value that does not
  convert to `T`) is a panic naming the field and the type; the framework
  turns it into an R error.

The type's name is provisional (#1857).

The `Sidecar` values are what a save keeps: `saveRDS()` writes them with
the pointer and the struct's other fields are lost ("Serialization" below).
They are also shared by every R name bound to the pointer ("Copies" below).

#### A save names `Sidecar` values by their R names

A saved object (`saveRDS()`, `serialize()`) stores one slot per `Sidecar`
field, named after the field's R name: the `#[r_data(name = "...")]` value,
else the Rust field name (#2187, #2215). After a reload, the R accessors that
read a save (`Type_get_f()` and the R6 / S7 fields; the `$` / `[[` methods of
`s3(r_data_accessors)` & co. refuse a restored pointer instead) and a serde
type's restored value ("A serde type comes back after `readRDS()`" below)
find each field's slot by that name. A field the save has no slot for raises
`miniextendr_restored_missing_slot` from an R accessor, with the field's R
name in `e$field`; a serde type's restored value takes that field's value
from its decoded struct instead.

So the R name is the save format:

- Reordering, adding or removing `Sidecar` fields keeps earlier saves
  readable: each field finds its own slot, and a slot whose field is gone is
  ignored.
- Renaming the Rust field keeps them readable as long as the R name stays.
  A field without `name = ...` keeps its old R name with
  `#[r_data(name = "old_name")]`, which also keeps its R accessors.
- Renaming the R name orphans the field's slot in every earlier save: the
  field reads as one the save lacks.
- Retyping a field keeps its name, so an earlier save's slot is read as the
  new type. A Rust getter that can't convert it raises an error naming the
  field; the R accessors return the saved value as it is.

A save from another version of the crate (`version` in the package's
`src/rust/Cargo.toml`, which the type ID in `prot[0]` records) is refused
with the `miniextendr_restored_other_version` error ("Serialization" below,
#1872), unless the type derives serde and serde reads the save. A save made
before slots were named is read by position.

### Copies

An external pointer is one R object. `y <- x` (or passing `x` to a
function) copies the handle, not the Rust value: both names reach the same
`EXTPTRSXP`, so a `set_f()` through `y` is visible through `x`, as is a
`Sidecar` write. R's copy-on-modify does not apply to the Rust value, nor
to the values in the pointer's `prot` list. A crate that wants a copy makes
one explicitly, with a method that clones the struct into a new
`ExternalPtr` (`ExternalPtr::new(self.clone())`); the `Sidecar` values are
copied by the struct's `Clone`.

### Comparing two objects

`identical(x, y)` compares the pointers: it is `TRUE` only when `x` and `y`
are the same object. `all.equal(x, y)` can't read the Rust values either, so
without a method of the class it is `TRUE` for any two objects of one class
(`all.equal.default()` compares an external pointer only by its
`data.class()`). `#[derive(AllEqual)]` gives the class an `all.equal()`
method that compares the struct field by field, each field with its own
`all.equal()` method (`CLASS_SYSTEMS.md`, "Comparing Objects").

### Serialization

`saveRDS()` writes an external pointer's tag and `prot` list but not its
address, and `readRDS()` brings it back with a NULL address. `serialize()`
/ `unserialize()` behave the same way, so the same holds for anything built
on them (`save()` / `load()`, a parallel worker's result, a cache). So a
save keeps:

- the `Sidecar<T>` values and the user slot (`protected()`), which come back
  with the pointer: the standalone R accessors of `Sidecar` fields
  (`Type_get_f()` / `Type_set_f()`), the R6 active bindings and the S7
  properties read and write the restored pointer;
- the type ID in `prot[0]`, crate version included, which says what saved
  it. A save is the type's when the ID's crate and type name are the
  type's: the module path is left out, so a type moved to another module of
  its crate keeps its saves (#2186).

Unless the type derives `Serialize` and `Deserialize` ("A serde type comes
back after `readRDS()`" below), it does not keep the Rust value. Everything
that needs it raises a classed error on the restored object, so code can tell the case apart and
re-create the object: a method (any receiver), an `ExternalPtr<T>`
argument, the R accessor of a struct or computed field. The `$` / `[[` /
`$<-` / `[[<-` field methods of `s3(r_data_accessors)`, `s4(...)` and
`env(...)`, and the S3 `as.list()`, raise it for every field, `Sidecar`
fields included: a reloaded object is refused, not recovered
(`CLASS_SYSTEMS.md`, "A restored pointer").

- Saved by this version of the package: class
  `miniextendr_restored_no_value`, message ``this `MyType` object was
  restored from a saved session and has no Rust value; re-create it``.
- Saved by another version: class `miniextendr_restored_other_version`,
  message ``this `MyType` object was saved by mypkg 0.6.0 and can't be read
  by mypkg 0.7.0; re-create it``, with `e$saved_version` and
  `e$current_version`. The `Sidecar` accessors refuse such a pointer too:
  another version may have other fields.
  To test this case without two builds, call
  `miniextendr_api::externalptr::rewrite_stored_version_for_tests(x, "0.0.1")`
  on a restored pointer from a test-only function: it makes the pointer read
  as a save by that version (#2169).
- Both carry the class `miniextendr_restored`. A pointer miniextendr did
  not build (R's `new("externalptr")`, another library's) keeps the plain
  errors, which name the receiver's R class (``this `myclass` object holds
  a null pointer``, ``... holds a pointer of another type``), or the Rust
  type for a receiver with no class.
- `#[externalptr(restored(class = ["pkg_saved", "pkg_error"], message =
  "..."))]` on the type puts the package's classes, in order, in front of
  those (which stay) and replaces the message, in both cases and on every
  path above; `e$saved_version` / `e$current_version` stay. Its
  `refuse = "every_read"` makes every S3 reader refuse first, before the
  name or position is looked at: `names()`, `as.list()`, an unknown name
  and a position too, on the bare pointer and on a list or environment
  carrying it in `.ptr`. `.DollarNames()` still lists the fields, so
  completion never raises (`CLASS_SYSTEMS.md`, "A restored pointer").
- A receiver of the class that carries no pointer at all (a list without
  `.ptr`, e.g. an older format of the same class) raises
  `miniextendr_no_handle` (``this `myclass` object holds no pointer``)
  from every R accessor, and under `refuse = "every_read"` from every S3
  reader, behind the type's `field_error(class = ...)` classes. A list or
  environment receiver also gets the `restored(class = ...)` classes and
  the `restored(message = ...)` text, the shape a save in an older format
  has; an atomic vector given the class was never saved and gets neither.
  `restored(no_handle = "...")` gives that list or environment its own
  message, one that says why it holds no pointer (``this engine was saved
  by an older release, which kept no engine; build a new one``), in place
  of the restored message; the package's own code can reuse it as
  `TypedExternal::NO_HANDLE_MESSAGE`. Otherwise
  `field_error(message = "...")` replaces the message of this refusal and
  of the two above.

A live pointer is checked with `Any::downcast`; one without an address by
the crate, version and type name of the type ID in `prot[0]`, not its
module path.

#### A serde type comes back after `readRDS()`

A type that derives `Serialize` and `Deserialize` comes back after
`readRDS()` with its value, when miniextendr-api's `serde` feature is on
(#2186). There is no attribute: `#[derive(ExternalPtr)]` checks for the two
traits, and every other type keeps the refusal above.

```rust
use miniextendr_api::externalptr::{RSidecar, Sidecar};
use serde::{Deserialize, Serialize};

#[derive(ExternalPtr, Serialize, Deserialize)]
#[externalptr(r6)]
pub struct Model {
    #[serde(skip)]
    #[r_data]
    _r: RSidecar,
    /// A sidecar value: the save keeps its slot, so serde skips it.
    #[serde(skip)]
    #[r_data]
    pub labels: Sidecar<Vec<String>>,
    values: Vec<f64>,
    total: f64,
}
```

```r
m <- Model$new(c(1, 2))
saveRDS(m, "model.rds")
m2 <- readRDS("model.rds") # in another session too: readRDS() loads the package
m2$total()                 # the value came back
```

- **How.** The pointer's `prot` list ends with a carrier: a zero-length
  ALTREP raw vector of a class the package registers for the type when it
  loads (`miniextendr_saved:<crate>::<Type>`). R calls the class
  itself. During `serialize()` it encodes the value into the save as an R
  list, through the exact mode of the crate's serde-to-R path ("The snapshot
  encoding" below). During `unserialize()`, which loads the package first
  when it is not, it decodes the list and keeps the value. R reads the
  `prot` list before the pointer exists, so the value is attached at the
  pointer's first use: a method (any receiver), an `ExternalPtr<T>` / `&T`
  / `&mut T` argument, an R accessor of any field, the `$` / `[[` field
  methods. From then on the pointer is live, for every R name bound to it.
  No user code runs at restore time besides the type's `Deserialize`.
- **`Sidecar` values** come back from their saved slots, by the fields' R
  names (above). A `Sidecar` field is `#[serde(skip)]`; the decoded
  struct's value for it fills only a field the save has no slot for.
- **Cost.** Every `serialize()` of the object encodes the whole value:
  `saveRDS()`, `save()`, an object sent to a parallel worker, a
  `digest`-style hash of it. That cost was accepted; a type that must not
  pay it leaves out serde.
- **A save reads the value through a borrow** (`ExternalPtr::borrow`). R
  code that saves the object while a call holds it mutably (a callback a
  `&mut self` method runs, which calls `saveRDS(self)`) gets the
  `miniextendr_borrow_conflict` error, and the save stops.
- **A value serde can't encode** gives the save no value: R writes the
  pointer as before, and the reload is refused as for a type without serde.
- **The value comes back exactly**, through untagged and internally
  tagged enums and `#[serde(flatten)]` too: one-element `Vec`s, `f64` bits
  (`NA_real_`'s payload, NaN, `-0.0`, `Inf`), `i32::MIN`, integers above
  2^53, maps whose keys are not strings, `Some(None)`, strings holding a
  NUL byte ("The snapshot encoding" below). Two limits: serde's buffered
  `Content` reads no `i128` / `u128`, so one inside an untagged or
  internally tagged enum or a flattened struct fails to decode (#2257,
  serde-rs/serde#1682); and
  `saveRDS(ascii = TRUE)` / `serialize(ascii = TRUE)` write doubles as
  16-digit text, which R itself reads back changed in the last bit and
  without NaN payloads other than `NA`'s (the binary default keeps them).
- **R objects in the value.** A field of type
  [`serde::RObject`](SERDE_R.md#r-objects-inside-a-serde-value)
  holds an R object (a data frame, a formula, another saved object), which
  the save writes with the rest of the value and `readRDS()` gives back,
  rooted, in any context, buffered enums included. Within one save R shares
  only environments, external pointers and symbols: one of those that the
  value holds in two places, or that the value and the rest of the save
  both hold, comes back as one object. Another `ExternalPtr` to a serde type
  comes back once, through its own carrier, with its own value. Any other
  object (a data frame, a vector) comes back as one copy per place. The
  Rust fields are encoded by value, so serde's rules apply to them (an
  `Rc` held twice comes back as two).
- **Another version's save** is decoded too: serde decides. A field added
  since then decodes when it has `#[serde(default)]`; one without a
  default, or a field whose type changed, fails to decode. A missing
  `Option` field decodes as `None` (serde's default), so a save from
  another version, or a damaged one, can come back with that field cleared
  rather than refused.
- **`&'static str` fields** can never be `DeserializeOwned`: serde borrows a
  field written `&'static str` (or `Option<&'static str>`) with
  `'de: 'static`, so the derived impl is `Deserialize<'static>` only, and
  `Vec<&'static str>` has no impl at all. Such a type has no snapshot. Use
  `String`, `Cow<'static, str>`, or a `#[serde(deserialize_with = ...)]`
  that maps the text to the static strings.
- **A save that fails to decode** is refused at the first use, as a
  restored pointer without a value is: the `restored(class = ...)` classes,
  then `miniextendr_restored_no_value` and `miniextendr_restored`, with the
  message ``this `Model` object was restored from a saved session, but its
  saved value could not be read: <cause>; re-create it`` (or
  `<restored message> (its saved value could not be read: <cause>)`). The
  cause is `e$parent`, of class `miniextendr_decode_error` with serde's
  message; a `Deserialize` impl that raises an `RCondition` keeps its own
  classes. The object stays refused, and saving it again writes the saved
  value back unchanged. `unserialize()` itself does not raise, so `load()`
  of an `.RData` file restores the other objects next to one that fails;
  only an R error raised by the type's own `Deserialize` code propagates
  from it (#2244).
- **Without raising.** `ExternalPtr::<T>::wrap_sexp(x)` attaches the value
  too, and returns `None` for a save that fails to decode.
  `wrap_sexp_with_error` and the `TryFromSexp` conversion raise its refusal,
  as a method does.
- **No value comes back** for a save made with `serialize(version = 2)`,
  which writes no ALTREP class, nor when the package or the type's class
  can't be found (R warns `cannot unserialize ALTVEC object of class ...`):
  a type renamed, or one that dropped serde. The reload is refused, as for a
  type without serde. A type whose
  `TypedExternal` impl is hand-written (or `impl_typed_external!`'s), and
  an object behind a trait-ABI handle, have no carrier either (#2243,
  #2242).

- **Moving and renaming a type.** A save is found by the crate's name and
  the type's: the carrier class (`miniextendr_saved:<crate>::<Type>`) and
  the type ID's match both leave the version and the module path out.
  - Moving a type to another module of its crate is safe: earlier saves
    come back, and the first use writes the type's new path into the
    pointer, which its next save keeps.
  - Renaming a type breaks its earlier saves: R doesn't find their class,
    and they are refused as above. Renaming the type renames its R class
    too.
  - Two serde types of one crate can't share a name, even in different
    modules: each would read the other's saves. The package then fails to
    load, with an error naming both types' paths. A type without serde has
    no carrier and is not checked.

##### The snapshot encoding

The save encodes the value with an exact mode of the crate's serde-to-R
path (#2256): the same R list `serde::to_r` writes where that one is exact,
and an attribute `serde` naming the kind where the general mapping would
be ambiguous. `serde::to_r` / `serde::from_r` keep the general mapping,
which reads better in R but is not exact (#1417, #1419). Only the encoding
writes attributes, so every attribute in it is one of these.

| Rust value | R value |
|---|---|
| `bool` | `logical(1)` |
| an integer that fits `i32` (any type, `i32::MIN` included) | `integer(1)` |
| an integer outside `i32` (`i64`, `u64`, `i128`, `u128`, `u32`) | `character(1)` in decimal, `serde = "int"` |
| `f32`, `f64` | `double(1)`, the bits as they are |
| `char`, `str` | `character(1)` |
| a string holding a NUL byte | `raw` of its UTF-8 bytes, `serde = "str"` |
| bytes (`serialize_bytes`, e.g. `serde_bytes`) | `raw` |
| `None`, `()`, unit struct | `NULL` |
| `Some(x)` | `x`'s value; `list(x)` with `serde = "some"` when `x`'s value is `NULL` or such a list (`Some(None)`) |
| newtype struct | its field's value |
| sequence of `bool`s, of integers that fit `i32`, of floats, or of strings (at least one element) | that atomic vector, `serde = "seq"` |
| sequence of integers, at least one outside `i32` | `character` in decimal, `serde = "ints"` |
| any other sequence (empty, a `None`, nested, mixed), tuple, tuple struct | unnamed list |
| struct, map whose keys are all strings | named list (an empty one is `named list()`) |
| any other map | `list(keys = <keys>, values = <values>)`, each a sequence, `serde = "map"` |
| unit variant | `character(1)`, the variant's name |
| newtype, tuple or struct variant | `list(<variant> = <payload>)` |
| [`serde::RObject`](SERDE_R.md#r-objects-inside-a-serde-value) | `list(<the object>)`, `serde = "robject"` |

The list around an `RObject`'s object keeps the object's own attributes
(a `serde` attribute included) from being read as the encoding's.

Untagged and internally tagged enums and `#[serde(flatten)]` decode
through serde's buffered `Content`, which never sees the target type and
records only what `deserialize_any` visits. So every visit is decided from
the R value alone:

| R value | `deserialize_any` visits |
|---|---|
| `NULL` | `visit_unit` (`None` and `()` read it) |
| `logical(1)`, `integer(1)`, `double(1)`, `character(1)` | `visit_bool`, `visit_i32`, `visit_f64`, `visit_str` |
| `raw` | `visit_bytes` |
| `serde = "int"` | `visit_i64`, else `visit_u64`, `visit_i128`, `visit_u128`: the first that holds the value |
| `serde = "str"` | `visit_string` |
| `serde = "seq"` or `"ints"`, any length (one too) | `visit_seq`, each element by the rows above |
| unnamed list | `visit_seq` |
| named list (a struct, a map with string keys) | `visit_map`, the names as string keys |
| `serde = "map"` | `visit_map`, the keys by these rules |
| `serde = "some"` | `visit_some` |
| `serde = "robject"` | `visit_newtype_struct` with a token string for the object |

serde's buffer can't hold an R object, so it keeps the token, and the
`RObject` the buffered value is decoded into looks the object up by it
while the decoder runs. Outside a buffer, `RObject` takes the object
directly.

No visit maps an `NA` to `None`: only `NULL` (and the `"some"` list) is the
`None` encoding, so `Some(NA_real_)` stays `Some`, and an `integer(1)`
holding `NA_integer_` is `i32::MIN`.

**Sequences.** An atomic vector without the attribute is always a scalar,
and one with `serde = "seq"` is always a sequence, of any length: that is
what tells a `Vec` of one element from its element. A sequence whose
elements are all scalars of one kind stays one atomic vector, so a
`Vec<f64>` of a million values is one 8 MB double vector rather than a 56
MB list of scalars, and the encoder collects it in one Rust vector before
it allocates the R one. A sequence with a `None`, a nested value or mixed
kinds is a list, whose elements each carry their own encoding.

A save written before #2256 used the general mapping. Where the two agree
it decodes; otherwise (a one-element `Vec`, an integer the general mapping
wrote as a double) it is refused at the first use, as a save that fails to
decode. Nothing reads the old mapping as a fallback.

**webR:** the round trip has not been tested there. Nothing in it is
platform-specific (R serializes the pointer's tag and `prot` the same way),
but the webR workflow is opt-in and this path has not been run under it.

## SEXP Layout

The internal layout of an `ExternalPtr`-created `EXTPTRSXP`:

```text
EXTPTRSXP
  addr  → *mut Box<dyn Any> (thin pointer → heap-allocated fat pointer)
            └→ RefCell<T> (the Rust value and its borrow flag)
  tag   → SYMSXP (TYPE_NAME_CSTR, for display)
  prot  → VECSXP[2 + T::R_SLOT_COUNT (+ 1 for a serde type)], named "", "",
          then the Sidecar fields' names (and "" for the carrier)
            [0] → SYMSXP (TYPE_ID_CSTR, for mismatch diagnostics)
            [1] → user-protected SEXP (set via set_protected)
            [2..] → one value per Sidecar<T> field
            [last] → a serde type's carrier: the ALTREP that saves its value
```

Internally the value is stored as `Box<Box<dyn Any>>`: the outer `Box` is a thin pointer that fits in R's `EXTPTRSXP` `addr` slot, and the inner `Box<dyn Any>` carries the trait-object vtable needed for `Any::downcast` at retrieval time. This lets one non-generic finalizer (`release_any`) free any `T` without per-type monomorphization; a type with `#[r_data(cache)]` fields registers `release_cached`, which drops the object's kept field values first. Type safety relies on `Any::downcast`, not on the `prot` symbols. The `Any` value is a `RefCell<T>`, which the generated code borrows (see [Reentrant Calls](#reentrant-calls)).

The `prot` slot holds a list. Slot 0 is the namespaced type ID symbol, retained for display/debug parity; authoritative type checking is `Any::downcast`. Slot 1 is available for user-protected R objects that should be kept alive alongside the pointer. The slots after it hold the type's `Sidecar<T>` values; `TypedExternal::R_SLOT_COUNT`, which the derive sets, gives their number. A serde type's list ends with its carrier ("A serde type comes back after `readRDS()`").

## Thread Safety

`ExternalPtr` is `!Send` and `!Sync`: its R object, its root and its finalizer belong to R's main thread, where every handle is made and dropped. Move the value out with `into_inner` to hand it to another thread. A `#[miniextendr]` function takes `ExternalPtr<T>` for any `TypedExternal` `T`, `Send` or not (#2027). The views `ExternalPtrRef` / `ExternalPtrMut` are `!Send` too: a borrow stays on the thread that took it.

## Trait Implementations

`ExternalPtr<T>` mirrors `Box<T>`'s ownership traits; the ones that read the
value take a checked borrow for the call, like `borrow()` / `borrow_mut()`:

- `Clone` / `clone_from` (deep clone, when `T: Clone`)
- `Default` (when `T: Default`)
- `Display`; `Debug`, which prints `<borrowed>` for a value borrowed
  mutably, and `<erased>` for an `ExternalPtr<()>`, rather than raising
- `Pointer` (the value's address, no borrow)
- `PartialEq` / `Eq` / `PartialOrd` / `Ord` / `Hash` (compare pointee values)
- `Iterator` / `DoubleEndedIterator` / `ExactSizeIterator` / `FusedIterator`
- `From<T>` / `From<Box<T>>`

It has no `Deref` / `DerefMut`, `AsRef` / `AsMut` or `Borrow` / `BorrowMut`:
each would hand out a reference with no borrow behind it ("Why there is no
`Deref`" above). There's no `Pin` support either.
