---
name: miniextendr-dots
description: "Use when the user asks about handling R's ... (dots/variadic) arguments in Rust, the Dots type, LazyDots (dots passed unforced: empty arguments, refusing extra arguments before evaluating them), the typed_list! macro, #[miniextendr(dots = typed_list!(...))] attribute sugar, custom dots binding names with name: ..., formals after ... (a &Dots parameter at any position), optional vs required fields in typed lists, or TypedList accessors."
---

# miniextendr Dots and `typed_list!`

R's `...` (dots) passes an untyped sequence of named or unnamed arguments through a call stack. miniextendr maps `...` to a `&Dots` parameter in Rust and provides the `typed_list!` macro for compile-time-specified runtime validation.

## When to use this skill

- "How do I accept `...` in my Rust function?"
- "What is the `Dots` type?"
- "How do I use `typed_list!`?"
- "What is `name: ...` syntax?"
- "Can a parameter come after `...`?"
- "How do I see `f(a = )` / `x[1, , , ]`, or refuse extra arguments without evaluating them?" (`LazyDots`)
- "How do I make a dots field optional?"
- "What does `#[miniextendr(dots = typed_list!(...))]` do?"
- "What error does a mismatched typed list produce?"

## Key concepts

### The `Dots` type

When a `#[miniextendr]` function has `name: ...` in its Rust signature, the macro transforms that position into a trailing `name: &Dots` parameter. For dots the body ignores, write `_: ...`: it binds the synthetic `__miniextendr_dots`, which user code should not rely on; name the dots (`args: ...`) to read them in the body. Don't write a bare `...`: rustc rejects it before the macro runs (`varargs_without_pattern`, deny by default, rust-lang/rust#145544) and suggests `_: ...`.

`Dots` provides three accessors:

| Method | Returns | Description |
|--------|---------|-------------|
| `as_list()` | `List` | Fast unchecked conversion to a `List` |
| `try_list()` | `Result<List, SexpTypeError>` | Type-checked conversion (names not checked) |
| `typed(spec)` | `Result<TypedList, TypedListError>` | Validate against a `TypedListSpec` |

### Custom binding name with `name: ...`

To give the dots parameter a name, use the `name: ...` syntax, or write the parameter as `name: &Dots` yourself:

```rust
#[miniextendr]
pub fn my_func(args: ...) -> i32 {
    args.as_list().len() as i32
}
```

The generated R wrapper still uses `...`; the Rust binding name is local only.

### Formals after `...`

The parameter of type `&Dots` is R's `...` at its own position, so a formal can follow it. Rust's `...` parses only last, so spell that with an explicit `&Dots` parameter:

```rust
#[miniextendr]
pub fn write_all(x: i32, rest: &Dots, #[miniextendr(default = "FALSE")] overwrite: bool) -> String {
    format!("{x} {} {overwrite}", rest.len())
}
// R: write_all <- function(x, ..., overwrite = FALSE)
```

R matches a formal after `...` by its exact name only (`write_all(1L, over = TRUE)` puts `over` in the dots). One `...` per function: a second `&Dots`, or `...` next to an explicit `&Dots`, is a compile error. Methods follow the same rule in every class system; on a `call = caller` wrapper `.call` stays last.

### Dots left unforced: `LazyDots`

`&Dots` gets `list(...)`, forced before the body runs (an empty element stops the call with "argument is missing"). A parameter typed `LazyDots` (taken by value) is R's `...` too, but the wrapper passes `environment()` instead and evaluates nothing (#1892):

| Method | R equivalent | Forces |
|--------|--------------|--------|
| `len()` / `is_empty()` | `...length()` | nothing |
| `names()` | `...names()` (`None` unnamed) | nothing |
| `is_missing_arg(i)` | element of `substitute(list(...))` is the empty argument | nothing |
| `expr(i)` | element of `substitute(list(...))` | nothing |
| `force(i)` | `..k` via `eval_with_handlers` | element `i` |
| `try_force(i)` | `..k` via `try_eval_with_handlers` (R error → `Err(REvalError)`) | element `i` |

0-based indices (an index past the end panics before anything is forced). Only a literally empty slot is `is_missing_arg`; an element forwarded from a missing argument is a promise (present), and forcing it raises R's missing-argument error. Each element is forced in its own environment, also through forwarded dots. Accepted on standalone fns, `s3(...)` fns and impl methods of env/R6/S3/S4/S7/vctrs; refused on trait methods and `extern`; no per-parameter options, no `typed_list!`, no `&LazyDots` / `Option` / `Missing`, never next to `&Dots`. Main thread only. Docs: `docs/DOTS_TYPED_LIST.md#dots-left-unforced-lazydots`; fixtures `rpkg/src/rust/lazy_dots_tests.rs`.

### `typed_list!` macro

`typed_list!` creates a `TypedListSpec` describing the structure expected in dots. Validation happens at R call time.

Syntax:

```
typed_list!(
    field_name => type_spec,
    optional_field? => type_spec,   // ? marks optional
)
```

Type specifiers:

| Syntax | Matches |
|--------|---------|
| `numeric()` | Real/double vector, any length |
| `numeric(4)` | Real/double vector, exactly 4 elements |
| `integer()` | Integer vector |
| `logical()` | Logical vector |
| `character()` | Character vector |
| `raw()` | Raw vector |
| `complex()` | Complex vector |
| `list()` | List (VECSXP) or `NULL`; a pairlist is refused, as `List` refuses one |
| `"data.frame"` | Object with that class |
| `"my_class"` | Any class name as a string literal |

By default, extra fields in dots are allowed. Use `@exact;` at the start of the spec for strict mode:

```rust
typed_list!(@exact;
    x => numeric(),
    y => numeric()
)
```

### Attribute sugar

The most ergonomic pattern is `#[miniextendr(dots = typed_list!(...))]`. The macro injects validation at the top of the function body and binds the result to `dots_typed`:

```rust
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn compute(_: ...) -> f64 {
    let x: Vec<f64> = dots_typed.get("x").expect("x");
    let y: Vec<f64> = dots_typed.get("y").expect("y");
    x.iter().zip(y.iter()).map(|(a, b)| a + b).sum()
}
```

The macro expands this to:

```rust
let dots_typed = __miniextendr_dots.typed(typed_list!(x => numeric(), y => numeric()))
    .unwrap_or_else(|e| panic!("dots validation failed: {e}"));
```

(For named dots, or an explicit `&Dots` parameter at any position, the binding is that parameter's name.)

### `TypedList` accessors

After validation, `dots_typed` (a `TypedList`) provides:

| Method | Returns |
|--------|---------|
| `get::<T>(name)` | `Result<T, TypedListError>` — required field |
| `get_opt::<T>(name)` | `Result<Option<T>, TypedListError>` — optional field |
| `get_raw(name)` | `Result<SEXP, TypedListError>` — raw SEXP |
| `as_list()` | `List` — underlying list |

`get::<T>()` / `get_opt::<T>()` accept **both** scalar and vector target types:
`T` may be a scalar (`f64`, `i32`, `String`, `bool`) **or** a vector/collection
(`Vec<f64>`, `Vec<i32>`, `Vec<String>`, …) — any `TryFromSexp` with a `Display`
error type. So `let x: Vec<f64> = dots_typed.get("x")?` (above) compiles for a
`numeric()` field, and `dots_typed.get::<Vec<i32>>("survive")` for an
`integer()` field.

### Error types

`TypedListError` variants emitted on validation failure:

| Variant | Cause |
|---------|-------|
| `NotList` | Input was not a list |
| `Missing { name }` | Required field absent |
| `WrongType { name, expected, actual }` | Field type mismatch |
| `WrongLen { name, expected, actual }` | Field length mismatch |
| `ExtraFields { names }` | Extra fields in strict (`@exact`) mode |
| `DuplicateNames { name }` | Duplicate field names |

## How it works

### R wrapper generation

When `#[miniextendr]` sees `...` in the Rust signature, the generated R wrapper function includes `...` in its formals. The `.Call` invocation collects dots with `list(...)` (a list, `VECSXP`, not a pairlist) and passes it as the dots argument to the C wrapper, which wraps it as the `Dots` value. For a `LazyDots` parameter it passes `environment()` (the wrapper's frame, with `...` still unforced) instead.

### Manual validation

Call `.typed()` directly in the function body instead of using the attribute sugar:

```rust
use miniextendr_api::typed_list;

#[miniextendr]
pub fn configure_model(dots: ...) -> String {
    let spec = typed_list!(
        learning_rate => numeric(),
        epochs => integer(),
        verbose? => logical()
    );
    let args = match dots.typed(spec) {
        Ok(a) => a,
        Err(e) => panic!("{e}"),
    };
    let lr: f64 = args.get("learning_rate").expect("learning_rate");
    format!("lr={lr}")
}
```

## Decision trees

### Do I need positional dots or named/typed dots?

- I just want to forward dots to another R function or count how many arguments were passed:
  - Name the dots (`dots: ...`) and use `dots.as_list()` (unchecked) or `dots.try_list()` (type-checked). Neither checks names: `f(a = 1, a = 2)` reaches the body; refuse it with `List::first_duplicate_name` if the function needs unique names.
- I know the exact structure: specific named fields each with known types:
  - Use `typed_list!` either as attribute sugar or manually.
- I want optional fields mixed with required ones:
  - Mark optional fields with `?`: `field? => type_spec`.
  - Use `get_opt` in the function body.

### Unnamed `_: ...` or a named binding?

- `_: ...` is fine when the body ignores the dots (rustc rejects a bare `...` by default).
- Use `name: ...` (or `name: &Dots`) when the body reads them, e.g. `options: ...`.
- Use an explicit `name: &Dots` parameter when a formal must follow the dots.

### `&Dots` or `LazyDots`?

- `&Dots` when every element should be evaluated anyway (a list of values, `typed_list!` validation).
- `LazyDots` when the body must see empty arguments (`update(x, select = )`, `x[1, , , ]`), refuse extra arguments before they are evaluated (`subset(x, TRUE, select = ID)`), read what was written (`expr(i)`), or force only some elements.

## Key files

- `docs/DOTS_TYPED_LIST.md` — full documentation with examples.
- `miniextendr-api/src/dots.rs` — `Dots` type, `TypedList`, `TypedListSpec`, `TypedListError`.
- `miniextendr-api/src/lazy_dots.rs` — `LazyDots`.
- `miniextendr-macros/src/typed_list.rs` — `typed_list!` macro implementation.

## Common pitfalls

- **`match.arg` returning the full vector**: `match.arg(arg = default_choices_vector, several.ok = TRUE)` returns the entire choices vector when called with a default multi-element argument. If you use `match.arg` in an R wrapper alongside dots, check that the defaults are not inadvertently the full choices list. This is a broader `match.arg` gotcha documented in `miniextendr-macros`.

- **Using `as_list()` when you need validation**: `as_list()` is unchecked. If R calls the function with the wrong types, you get a runtime error deep inside your Rust code instead of a clean validation error. Prefer `typed_list!` when the schema is known at compile time.

- **Strict mode rejecting intended extras**: `@exact` causes any unnamed or extra fields to be rejected. If you want to accept caller-defined extras and only validate required fields, omit `@exact`.

- **`get` vs `get_opt` mismatch**: calling `get` for a field marked `?` (optional) panics if the field is absent. Use `get_opt` for optional fields; it returns `None` when the field is missing rather than an error.

- **Duplicate field names in the R call**: `DuplicateNames` fires if the R caller passes the same name twice in `...`. This is usually an R-side mistake; the error message includes the duplicated name.

## Related skills

- `miniextendr-macros` — broader `#[miniextendr]` attribute parsing, codegen, `match.arg` integration.
- `miniextendr-conversions` — `TryFromSexp` and `IntoR` for field types used inside `TypedList::get`.
