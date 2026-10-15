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
- **Context-checked construction** -- `new()` runs on R's main thread or routes
  from an active miniextendr worker context

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

`new()` works on R's main thread and from miniextendr's dedicated worker during
an active `run_on_worker` call; the latter routes its R allocations through
`with_r_thread`. It panics on an arbitrary spawned or Rayon thread rather than
calling R off-main.

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

A borrowed argument keeps the call on R's main thread, like a `&self` method:
under `#[miniextendr(worker)]` or the `worker-default` feature such a function
still runs on the main thread.

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
storage must supply that metadata, as other `TryFromSexp` implementations do.
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
called through a View are handles too. An `ExternalPtr<T>` argument moves to
the worker under `worker`, and is read there with `borrow()`.

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
class (`r_data_accessors = "get"` for the readers only). On an S3 bare
pointer `[[`, `as.list()` and `[[<-` refuse an unused argument, as
`dots = empty` does, and `[[` takes `exact`; so does an env class's `[[`
(#2174). With
`fallback = "Self::f"` next to the option, a name no field has is read by
`fn f(&self, name: &str) -> Option<T>` on the Rust value, and
`fallback_names = "Self::g"` adds `g(&self) -> Vec<String>` to the
completions (#2150). Under that S3, S4
or env field syntax the `MyType_get_*()` / `MyType_set_*()` functions are
internal, for the package's own R code; otherwise they are exported. vctrs
classes get only those functions. See `CLASS_SYSTEMS.md`, "Direct Field
Access via Sidecar".

| Field type | Where the value lives | Getter | Setter |
|---|---|---|---|
| `i32`, `f64`, `bool`, `u8` | the Rust struct | a fresh length-1 vector | reads `value` with `Rf_as*` (so the integer setter truncates `3.5` and accepts `"3"`, #1675) |
| any `Clone + IntoR + TryFromSexp` type | the Rust struct | converts a clone with `IntoR` (the struct is only borrowed) | converts with `TryFromSexp`, or raises `'name' must be ...` |
| `Sidecar<T>` | the pointer's `prot` list | the stored R value | validates `value` with `TryFromSexp::<T>` and stores the value R gave |
| `Computed` with `#[r_data(get = "Self::f")]` | nowhere: `f(&self) -> impl IntoR` computes it | converts the function's value with `IntoR` | none; the field syntax raises `miniextendr_read_only_field` |

`#[r_data(name = "keys")]` sets the field's R name, used by every R-facing
string (the field list, the `$` / `[[` labels, the R6 binding or S7 property,
`MyType_get_keys()` / `MyType_set_keys()`, the setter's message); the Rust
accessors (`MyType::r_keys(&ptr)`) and the C symbols keep the Rust
identifier. The name must be syntactic, unique in the type and not `.ptr`.
A `Sidecar` field's R name is also its name in a save (below), so renaming
the Rust field is safe for saves and renaming the R name is not (#2215). A
`Computed` field must be `pub`, and is refused under `#[externalptr(r6)]` /
`#[externalptr(s7)]`, whose bindings and properties need a setter (#1888).

A struct field is a view of the Rust value: every read converts it afresh, and
every write converts into Rust or fails, so no R copy can drift from it. A
computed field is a view too, through its function.

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
  field is empty. `Sidecar::new` and `Sidecar::replace` run on R's main
  thread, or on the managed worker (a `worker` constructor), which hands
  the conversion to the main thread; any other thread panics.
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
- The accessors run on R's main thread only. Off it (a `worker` method, a
  thread the package spawned) an accessor panics with a message naming the
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
`s3(r_data_accessors)` & co. refuse a restored pointer instead) and a rebuild
hook ("Rebuilding after a reload" below) find each field's slot by that name.
A field the save has no slot for raises `miniextendr_restored_missing_slot`,
with the field's R name in `e$field`.

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
#1872), unless the type rebuilds it (`rebuild = path, any_version`). A save
made before slots were named is read by position.

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
  it.

It does not keep the Rust value. Everything that needs it raises a classed
error on the restored object, so code can tell the case apart and
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
  slots are positional, and another version may lay them out differently.
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
  carrying it in `.ptr`, with `.DollarNames()` returning `character(0)`
  (`CLASS_SYSTEMS.md`, "A restored pointer").
- A receiver of the class that carries no pointer at all (a list without
  `.ptr`, e.g. an older format of the same class) raises
  `miniextendr_no_handle` (``this `myclass` object holds no pointer``)
  from every R accessor, and under `refuse = "every_read"` from every S3
  reader, behind the type's `field_error(class = ...)` classes. A list or
  environment receiver also gets the `restored(class = ...)` classes and
  the `restored(message = ...)` text, the shape a save in an older format
  has; an atomic vector given the class was never saved and gets neither.
  Otherwise `field_error(message = "...")` replaces the message of this
  refusal and of the two above.

A live pointer is checked with `Any::downcast`; one without an address by
the type ID in `prot[0]`.

#### Rebuilding after a reload: `rebuild = path`

A type whose `Sidecar` fields hold everything its Rust value is built from
can be rebuilt instead of refused. `#[externalptr(rebuild = path)]` names a
hook, `fn(&Saved<'_, Self>) -> Result<Self, E>`, which the first use that
needs the value calls on a pointer this version of the crate saved:

```rust
use miniextendr_api::externalptr::{RSidecar, Saved, Sidecar};

#[derive(ExternalPtr)]
#[externalptr(r6, rebuild = Self::from_saved)]
pub struct Model {
    #[r_data]
    _r: RSidecar,
    /// The recipe: a save keeps it.
    #[r_data]
    pub values: Sidecar<Vec<f64>>,
    /// Rust state, rebuilt from the recipe.
    total: f64,
}

impl Model {
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, ModelError> {
        let values = Self::values(saved); // the saved slot
        Ok(Model {
            _r: RSidecar,
            total: values.iter().sum(),
            values: Sidecar::default(), // placeholder: the saved slot stays
        })
    }
}
```

- **Where.** Every path that refuses a restored pointer above rebuilds it
  first: a method (any receiver), an `ExternalPtr<T>` / `&T` / `&mut T`
  argument, the R accessor of a struct field. The framework installs the
  value it returns in the restored pointer itself, finalizer included, so the
  later calls, and every R name bound to the object, see a live pointer. A
  `Sidecar` accessor reads (or writes) the saved slot of its field's name,
  without rebuilding.
- **What the hook reads.** The type's generated `Sidecar` getters take the
  `Saved` handle like a live one (`Self::values(saved)`). Only `Sidecar`
  fields have a slot in the save: a struct field or a `Computed` field is
  gone with the Rust value, so a type whose rebuild needs one keeps it in a
  `Sidecar` instead. `Saved::as_sexp()` is the restored pointer, and
  `Saved::version()` the crate version that saved it.
- **By field name.** A save names its slots after the `Sidecar` fields' R
  names (`#[r_data(name = "...")]`, else the Rust field name), so the hook
  reads each field by name, whatever fields the saving version had or in
  what order (#2187, #2215). A field added since that version has no slot:
  `Self::try_values(saved)` (generated for every `Sidecar` field of a type
  with a hook, named after the Rust field) returns `None`, and the plain
  getter raises `miniextendr_restored_missing_slot`, with the field's R name
  in `e$field`. `Saved::has("values")` says the same for an R name. A slot
  whose field is gone is ignored. A save from before slots were named is
  read by position.
- **What the rebuilt object holds.** The pointer is laid out as this
  version's. A field the save has keeps its saved slot: the hook's
  `Sidecar::new(..)` value for it is a placeholder and is dropped. A field
  the save lacks takes the value the hook put in its struct, so give it the
  real value (`Sidecar::new(Self::try_values(saved).unwrap_or_default())`).
  A hook that recomputes a field and wants the object to hold the new value
  returns it as `Sidecar::replace(value)`, which replaces the saved slot.
- **Nested objects.** A `Sidecar<SEXP>` can hold another object's pointer.
  Converting it in the hook (`ExternalPtr::<Inner>::try_from_sexp(..)`)
  rebuilds that one too, in its own pointer, when its type has a hook. A hook
  that reaches its own object again finds it refused, not rebuilt twice.
- **Failure.** An `Err` (or a condition the hook raises with `error!`)
  installs nothing, and the next use tries again. The caller gets the
  type's restored refusal: the `restored(class = ...)` classes, then
  `miniextendr_restored_no_value` and `miniextendr_restored`, with the
  cause's message appended (``this `Model` object was restored from a saved
  session and rebuilding it failed: <cause>``, or `<restored message>
  (rebuilding it failed: <cause>)`). The cause is the condition object
  `e$parent`, with its own classes and fields, as `rlang::abort(parent = )`
  chains errors. It is built from the `Err` as a `#[miniextendr]` function
  returning `Result<T, E>` builds its error
  ([CONDITIONS.md](CONDITIONS.md#classed-result-errors-with-rconditiondata-and-rerror)):
  an `E` that implements `RConditionData` (a `#[derive(RCondition)]` type,
  `RError`, an `RCondition` itself) keeps its classes, message and fields;
  under the `serde` feature a `Serialize + Display` `E` is classed from its
  serde shape, the one `serde_error_shape!` declares for it if any; any
  other `E` gives its `Debug` text. The cause's classes stay out of the
  refusal's, so a handler for them does not catch a reload failure; branch
  on `inherits(e$parent, "pkg_error_config")`. An R error inside the hook
  carries on as itself.
- **A hook with several kinds of failure** returns `Result<Self,
  RCondition>` (#2217). A condition `catch_condition` caught passes through
  `?`, and `serde_condition!(&err)` turns a serde error into the condition
  a function returning `Err(err)` raises, with the crate's declared shape
  ([CONDITIONS.md](CONDITIONS.md#a-serde-error-as-an-rcondition)):

  ```rust
  fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
      let values = Self::values(saved);
      catch_condition(|| check_recipe(&values))?; // raises error!(class = .., data = ..)
      let engine = engine::Engine::fit(&values)
          .map_err(|e| miniextendr_api::serde_condition!(&e))?;
      Ok(Model { engine, values: Sidecar::default() })
  }
  ```

  Either way `e$parent` has the condition's classes, message and fields.
  It is an error whatever kind the `RCondition` was built with, as any
  `Err` is. A hook that converts another restored pointer inside
  `catch_condition` and returns the refusal it caught gives the whole
  chain: `e$parent` is the inner refusal, `e$parent$parent` its cause.
- **Without raising.** `ExternalPtr::<T>::wrap_sexp_with_error(x)` returns
  a failed rebuild as `Err(TypeMismatchError::RebuildFailed { condition })`:
  the refusal above as a value, its cause in it (#2217). A check that must
  not raise (is the object live? fall back to plain data?) needs no
  `catch_condition`; `condition.raise()` raises it. An `ExternalPtr<T>` /
  `&T` / `&mut T` argument and `ExternalPtr::<T>::try_from_sexp` raise it,
  as a method on the object does.
- **Conditions the hook queues** (`defer_warning!` & co.) are signalled when
  the call that triggered the rebuild returns. A hook that replays the
  construction, whose warnings were signalled once already, drops them with
  `deferred_condition::capture(|| ..).0`.
- **Another version's save** is refused as before
  (`miniextendr_restored_other_version`), and the hook does not run, unless
  the type opts in with `any_version` next to the hook:
  `#[externalptr(r6, rebuild = Self::from_saved, any_version)]`. The hook
  then rebuilds a save by any version of the crate that names its slots,
  on the object's first use of any kind (a `Sidecar` read included, since
  the slots are laid out anew). The hook decides what to accept: it reads
  `saved.version()`, checks the fields, and returns an `Err` to refuse. A
  save by another version without slot names stays refused: its layout is
  unknown.

A type whose `Sidecar` fields hold too little to rebuild from needs the
value itself in the save, a serde snapshot (#2186).

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
  prot  → VECSXP[2 + T::R_SLOT_COUNT], named "", "", then the Sidecar fields' names
            [0] → SYMSXP (TYPE_ID_CSTR, for mismatch diagnostics)
            [1] → user-protected SEXP (set via set_protected)
            [2..] → one value per Sidecar<T> field
```

Internally the value is stored as `Box<Box<dyn Any>>`: the outer `Box` is a thin pointer that fits in R's `EXTPTRSXP` `addr` slot, and the inner `Box<dyn Any>` carries the trait-object vtable needed for `Any::downcast` at retrieval time. This lets one non-generic finalizer (`release_any`) free any `T` without per-type monomorphization. Type safety relies on `Any::downcast`, not on the `prot` symbols. The `Any` value is a `RefCell<T>`, which the generated code borrows (see [Reentrant Calls](#reentrant-calls)).

The `prot` slot holds a list. Slot 0 is the namespaced type ID symbol, retained for display/debug parity; authoritative type checking is `Any::downcast`. Slot 1 is available for user-protected R objects that should be kept alive alongside the pointer. The slots after it hold the type's `Sidecar<T>` values; `TypedExternal::R_SLOT_COUNT`, which the derive sets, gives their number.

## Thread Safety

`ExternalPtr` is `Send` when `T: Send`, allowing it to be transferred between threads. All R API calls are serialized through the main thread. Concurrent access is not supported -- R's runtime is single-threaded. The views `ExternalPtrRef` / `ExternalPtrMut` are `!Send`: a borrow stays on the thread that took it.

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
