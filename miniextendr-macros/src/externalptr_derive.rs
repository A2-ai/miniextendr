//! # `#[derive(ExternalPtr)]` - ExternalPtr Support
//!
//! This module implements the `#[derive(ExternalPtr)]` macro which generates
//! a `TypedExternal` impl for use with `ExternalPtr<T>`.
//!
//! Trait ABI wrapper infrastructure is automatically generated when you use
//! `#[miniextendr]` on `impl Trait for Type` blocks.
//!
//! ## Sidecar codegen vs `#[miniextendr]` fns
//!
//! `#[derive(ExternalPtr)]` with `#[r_data]` fields emits a pair of C/R
//! wrappers per sidecar field (`<Type>_get_<field>` /
//! `<Type>_set_<field>`). **These wrappers do NOT go through
//! `c_wrapper_builder::CWrapperContext`** — they are hand-rolled in this
//! module so they can present the simplest signature R expects from a slot
//! accessor:
//!
//! | Aspect | `#[miniextendr]` fn / method (`c_wrapper_builder`) | Sidecar accessor (here) |
//! |---|---|---|
//! | First C param | `__miniextendr_call: SEXP` | none |
//! | C `numArgs` | 1 + user args | `1` (getter) / `2` (setter) |
//! | R `.Call` | `.Call(C_…, .call = sys.call(), …)` | `.Call(C_…, x)` / `.Call(C_…, x, value)` |
//! | Error transport | tagged-condition SEXP via `with_r_unwind_protect(…, Some(call))` | tagged-condition SEXP via `with_r_unwind_protect(…, None)`; the R guard's `sys.call()` supplies attribution |
//!
//! **A sidecar R wrapper passes no `.call` argument.** The C function doesn't
//! have a slot for it: R would throw "Incorrect number of arguments" at
//! runtime (#344, #348).
//!
//! ## Usage
//!
//! ### Basic (no traits)
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyData {
//!     value: i32,
//! }
//! // Generates: impl TypedExternal for MyData { ... }
//! ```
//!
//! ### With R Sidecar Slots and Class System
//!
//! The `#[r_data]` attribute marks fields that get R accessors. Every public
//! slot gets the standalone `Type_get_field(x)` / `Type_set_field(x, value)`.
//! Use `#[externalptr(...)]` to name the class system; for R6 and S7 the impl's
//! `r_data_accessors` option also wires the slots into the class:
//!
//! | Class System | Attribute | R Accessors |
//! |--------------|-----------|-------------|
//! | Environment | `#[externalptr(env)]` (default) | `Type_get_field()`, `Type_set_field()` |
//! | R6 | `#[externalptr(r6)]` + `#[miniextendr(r6(r_data_accessors))]` | Active bindings, `obj$field` / `obj$field <- value` |
//! | S3 | `#[externalptr(s3)]` | `Type_get_field()`, `Type_set_field()` |
//! | S4 | `#[externalptr(s4)]` | `Type_get_field()`, `Type_set_field()` |
//! | S7 | `#[externalptr(s7)]` + `#[miniextendr(s7(r_data_accessors))]` | Properties, `obj@field` / `obj@field <- value` |
//!
//! Standalone setters and R6 active-binding setters return the receiver invisibly
//! by default. `#[r_data(setter = "visible")]` exposes that return value;
//! `setter = "invisible"` explicitly requests the default. These options also
//! control S7 property setters, whose unmarked return remains visible.
//! R assignment itself stays invisible regardless of the setter's choice.
//!
//! Three field tiers are supported:
//!
//! 1. **Rooted values** (`Sidecar<T>`) - kept in the external pointer's
//!    protection list, which roots them and serializes them with the
//!    pointer. The derive generates typed accessors on the struct, with the
//!    field's visibility: `#[r_data(ref)]` a getter `fn f(&self) -> T`,
//!    `#[r_data(mut)]` a setter `fn set_f(&mut self, value: T)`,
//!    `#[r_data(ref, mut)]` or a bare `#[r_data]` both. The struct keeps a
//!    back-reference to the slot, which the handle writes whenever it hands
//!    the struct out; see `miniextendr_api::externalptr::Sidecar`. A bare
//!    `SEXP` field is refused: the struct can't root it.
//! 2. **Scalars** (`i32`, `f64`, `bool`, `u8`) - struct fields, returned as a
//!    length-1 vector and written with `Rf_as*`
//! 3. **Conversion types** (anything else) - struct fields, converted with the
//!    `IntoR`/`TryFromSexp` traits on every read and write
//!
//! Scalar and conversion values live in the Rust struct, behind the pointer's
//! address, which `saveRDS` does not write. The R accessors of a `Sidecar`
//! field read and write the protection list by slot index: the setter
//! validates the value with `TryFromSexp::<T>` and stores the value R gave,
//! and both keep working on a pointer `readRDS()` brought back without an
//! address (checked by the type ID in `prot[0]`).
//!
//! ### A saved and restored object
//!
//! `saveRDS()` / `serialize()` keep the pointer's `prot` list (the `Sidecar`
//! values and the user slot) but not the Rust value. On the restored pointer
//! the `Sidecar` accessors work; the accessor of a struct field, like every
//! method, raises the classed error `miniextendr_restored_no_value` ("restored
//! from a saved session and has no Rust value; re-create it"). A pointer
//! another version of the crate saved is refused by every accessor with
//! `miniextendr_restored_other_version`, which names both versions. R's
//! `y <- x` shares the one Rust value and the one `prot` list: a write through
//! `y` is visible through `x`.
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! #[externalptr(r6)]
//! pub struct MyType {
//!     pub x: i32,
//!
//!     #[r_data]
//!     r: RSidecar,  // Selector - enables R accessors for this type
//!
//!     #[r_data(ref, mut)]
//!     pub keys: Sidecar<Vec<i32>>,  // In the protection list; `self.keys()` / `self.set_keys(v)`
//!
//!     #[r_data]
//!     pub count: i32,  // Scalar struct field
//!
//!     #[r_data]
//!     pub name: String,  // Conversion: uses IntoR/TryFromSexp
//! }
//!
//! #[miniextendr(r6(r_data_accessors))]
//! impl MyType {
//!     pub fn new(n: i32) -> Self {
//!         MyType { x: 0, r: RSidecar, keys: Sidecar::new((1..=n).collect()), count: n, name: String::new() }
//!     }
//! }
//! // Generates: active bindings `keys`, `count`, `name` in the R6Class
//! ```
//!
//! ### Trait ABI wiring
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyCounter {
//!     value: i32,
//! }
//!
//! #[miniextendr]
//! impl Counter for MyCounter { /* ... */ }
//! ```
//!
//! ## Generated Types (trait impls)
//!
//! ### Wrapper Struct
//!
//! ```ignore
//! #[repr(C)]
//! struct __MxWrapperMyCounter {
//!     erased: mx_erased,  // Must be first field
//!     data: MyCounter,
//! }
//! ```
//!
//! ### Base Vtable
//!
//! ```ignore
//! static __MX_BASE_VTABLE_MYCOUNTER: mx_base_vtable = mx_base_vtable {
//!     drop: __mx_drop_mycounter,
//!     concrete_tag: TAG_MYCOUNTER,
//!     query: __mx_query_mycounter,
//! };
//! ```
//!
//! ### Query Function
//!
//! The query function maps trait tags to vtable pointers:
//!
//! ```ignore
//! unsafe extern "C" fn __mx_query_mycounter(
//!     ptr: *mut mx_erased,
//!     trait_tag: mx_tag,
//! ) -> *const c_void {
//!     if trait_tag == TAG_COUNTER {
//!         return std::ptr::from_ref(&__VTABLE_MYPKG_COUNTER_FOR_MYCOUNTER).cast::<c_void>();
//!     }
//!     std::ptr::null()
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use syn::{DeriveInput, Field, Ident, Visibility};

use crate::miniextendr_impl::ClassSystem;

/// Parse `#[externalptr(...)]` attributes to extract class system.
///
/// Supported forms:
/// - `#[externalptr(env)]` - Environment style (default)
/// - `#[externalptr(r6)]` - R6 class
/// - `#[externalptr(s3)]` - S3 class
/// - `#[externalptr(s4)]` - S4 class
/// - `#[externalptr(s7)]` - S7 class
fn parse_externalptr_attrs(input: &DeriveInput) -> syn::Result<ClassSystem> {
    let mut class_system = ClassSystem::Env; // Default

    for attr in &input.attrs {
        if attr.path().is_ident("externalptr") {
            attr.parse_nested_meta(|meta| {
                let ident_str = meta
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();

                match ident_str.as_str() {
                    "env" => class_system = ClassSystem::Env,
                    "r6" => class_system = ClassSystem::R6,
                    "s3" => class_system = ClassSystem::S3,
                    "s4" => class_system = ClassSystem::S4,
                    "s7" => class_system = ClassSystem::S7,
                    "vctrs" => class_system = ClassSystem::Vctrs,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &meta.path,
                            format!(
                                "unknown class system '{}'; expected one of: env, r6, s3, s4, s7, vctrs",
                                ident_str
                            ),
                        ));
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(class_system)
}

/// Check if a field has the `#[r_data]` attribute.
fn has_r_data_attr(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident("r_data"))
}

/// The last path segment that names the rooted sidecar field type,
/// `miniextendr_api::externalptr::Sidecar<T>`. Its final name is #1857; the
/// derive spells it here only.
const SIDECAR_TYPE_NAME: &str = "Sidecar";

/// The `T` of a `Sidecar<T>` field type, `None` for any other type.
fn sidecar_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let seg = type_path.path.segments.last()?;
    if seg.ident != SIDECAR_TYPE_NAME {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    });
    let inner = types.next()?;
    types.next().is_none().then_some(inner)
}

/// Options for a sidecar field's generated documentation, setters and Rust
/// accessors.
#[derive(Default)]
struct RDataOptions {
    prop_doc: Option<String>,
    setter_invisible: Option<bool>,
    /// `ref`: generate the Rust getter of a `Sidecar<T>` field.
    access_ref: Option<Span>,
    /// `mut`: generate the Rust setter of a `Sidecar<T>` field.
    access_mut: Option<Span>,
}

/// Which Rust accessors a `Sidecar<T>` field gets: `#[r_data(ref)]` the
/// getter, `#[r_data(mut)]` the setter, `#[r_data(ref, mut)]` or a bare
/// `#[r_data]` both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct SidecarAccess {
    get: bool,
    set: bool,
}

/// One item of `#[r_data(...)]`: `ref`, `mut` (both keywords, so
/// `parse_nested_meta` cannot read them) or `key = "value"`.
enum RDataArg {
    Ref(Span),
    Mut(Span),
    KeyValue { key: Ident, value: syn::LitStr },
}

impl syn::parse::Parse for RDataArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![ref]) {
            let token: syn::Token![ref] = input.parse()?;
            Ok(Self::Ref(token.span))
        } else if lookahead.peek(syn::Token![mut]) {
            let token: syn::Token![mut] = input.parse()?;
            Ok(Self::Mut(token.span))
        } else if lookahead.peek(Ident) {
            let key: Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: syn::LitStr = input.parse()?;
            Ok(Self::KeyValue { key, value })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Parse field-level `#[r_data(ref, mut, prop_doc = "...", setter = "visible")]`
/// options.
fn parse_r_data_options(field: &Field) -> syn::Result<RDataOptions> {
    let mut options = RDataOptions::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("r_data") || matches!(attr.meta, syn::Meta::Path(_)) {
            continue;
        }
        let args = attr.parse_args_with(
            syn::punctuated::Punctuated::<RDataArg, syn::Token![,]>::parse_terminated,
        )?;
        for arg in args {
            match arg {
                RDataArg::Ref(span) => {
                    if options.access_ref.replace(span).is_some() {
                        return Err(syn::Error::new(span, "duplicate `ref` option"));
                    }
                }
                RDataArg::Mut(span) => {
                    if options.access_mut.replace(span).is_some() {
                        return Err(syn::Error::new(span, "duplicate `mut` option"));
                    }
                }
                RDataArg::KeyValue { key, value } if key == "prop_doc" => {
                    options.prop_doc = Some(value.value());
                }
                RDataArg::KeyValue { key, value } if key == "setter" => {
                    if options.setter_invisible.is_some() {
                        return Err(syn::Error::new_spanned(key, "duplicate `setter` option"));
                    }
                    options.setter_invisible = Some(match value.value().as_str() {
                        "visible" => false,
                        "invisible" => true,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                value,
                                "setter must be \"visible\" or \"invisible\"",
                            ));
                        }
                    });
                }
                RDataArg::KeyValue { key, .. } => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!(
                            "unknown key `{key}`; supported: `ref`, `mut`, `prop_doc`, `setter`"
                        ),
                    ));
                }
            }
        }
    }
    if options.setter_invisible.is_some()
        && (is_rsidecar_type(field) || !is_pub(field) || field.ident.is_none())
    {
        return Err(syn::Error::new_spanned(
            field,
            "`setter` requires a public named sidecar slot, not the RSidecar selector",
        ));
    }
    if let Some(span) = options.access_ref.or(options.access_mut)
        && sidecar_inner_type(&field.ty).is_none()
    {
        return Err(syn::Error::new(
            span,
            format!(
                "`ref` and `mut` in `#[r_data(...)]` apply to `{SIDECAR_TYPE_NAME}<T>` fields only; \
                 a plain `#[r_data]` field stays a struct field with its R accessors"
            ),
        ));
    }
    Ok(options)
}

/// Check if a field type is `RSidecar`.
///
/// Returns `true` if the last path segment of the field's type is `RSidecar`,
/// which acts as the selector marker enabling R sidecar accessor generation.
fn is_rsidecar_type(field: &Field) -> bool {
    if let syn::Type::Path(type_path) = &field.ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "RSidecar")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Check if a field is public.
fn is_pub(field: &Field) -> bool {
    matches!(field.vis, Visibility::Public(_))
}

/// The kind of sidecar slot, determining how getter/setter FFI functions are generated.
///
/// Each kind maps to a different codegen strategy for reading and writing the
/// slot through R's `.Call` interface:
/// - `Sidecar`: a value in the external pointer's protection list, validated
///   with `TryFromSexp::<T>` on write and stored as R gave it.
/// - Scalars: a struct field, read into R with `Rf_Scalar*`, written with `Rf_as*`.
/// - Conversion: a struct field, converted with the `IntoR`/`TryFromSexp` traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    /// `Sidecar<T>` field: the value at this position among the type's
    /// `Sidecar` fields, kept in the external pointer's protection list
    /// (getter returns it, setter validates `value` as a `T` and stores it
    /// there).
    Sidecar(usize),
    /// Scalar struct field `i32` (or `i16`/`i8`), returned as a length-1 integer.
    ScalarInt,
    /// Scalar struct field `f64` (or `f32`), returned as a length-1 double.
    ScalarReal,
    /// Scalar struct field `bool` (or `Rbool`), returned as a length-1 logical.
    ScalarLogical,
    /// Scalar struct field `u8`, returned as a length-1 raw.
    ScalarRaw,
    /// Conversion type -- uses `IntoR`/`TryFromSexp` traits for arbitrary Rust types.
    Conversion,
}

/// Information about a single `#[r_data]`-annotated sidecar slot field.
///
/// Collected during struct field parsing and used to generate FFI getter/setter
/// functions and R wrapper code for each public slot.
struct SidecarSlot {
    /// Rust identifier of the field (e.g., `count`, `name`).
    name: Ident,
    /// Rust type of the field, used in conversion-based getter/setter codegen.
    ty: syn::Type,
    /// The field's visibility, which its generated Rust accessors take.
    vis: Visibility,
    /// Whether the field is `pub`. Only public fields get R accessor functions.
    is_public: bool,
    /// Determines the codegen strategy for reading/writing this slot.
    kind: SlotKind,
    /// The Rust accessors of a `Sidecar<T>` field (`#[r_data(ref | mut)]`);
    /// unused by the other kinds.
    access: SidecarAccess,
    /// Optional documentation string for the S7 `@prop` tag.
    /// Sourced from `#[r_data(prop_doc = "...")]`. `None` means no doc was supplied;
    /// a default fallback string is used at emit time.
    prop_doc: Option<String>,
    /// Explicit setter visibility; each generated setter keeps its default when absent.
    setter_invisible: Option<bool>,
}

/// Aggregated sidecar information extracted from struct field analysis.
///
/// Contains everything needed to generate sidecar accessor code: the
/// selector presence, the list of typed slots, and the target class system.
struct SidecarInfo {
    /// Whether the struct contains an `RSidecar`-typed field marked with `#[r_data]`.
    /// At most one selector is allowed per struct.
    has_selector: bool,
    /// The `#[r_data]` slot fields (excluding the RSidecar selector itself),
    /// each carrying its index, kind, and visibility.
    slots: Vec<SidecarSlot>,
    /// The R class system chosen via `#[externalptr(...)]`, controlling the
    /// style of generated R wrapper code.
    class_system: ClassSystem,
}

/// Determine the [`SlotKind`] for a field type by inspecting its last path segment.
///
/// Recognizes `Sidecar<T>`, which takes position `sidecars` (the number of
/// `Sidecar` fields before it), scalar numerics (`i32`, `i16`, `i8`, `f64`,
/// `f32`), booleans (`bool`, `Rbool`), and raw bytes (`u8`). A bare `SEXP` is
/// an error: the struct can't root it. Everything else falls through to
/// [`SlotKind::Conversion`].
fn slot_kind_for_type(ty: &syn::Type, sidecars: usize) -> syn::Result<SlotKind> {
    if sidecar_inner_type(ty).is_some() {
        return Ok(SlotKind::Sidecar(sidecars));
    }
    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        let ident = &seg.ident;
        if ident == SIDECAR_TYPE_NAME {
            return Err(syn::Error::new_spanned(
                ty,
                format!("`{SIDECAR_TYPE_NAME}` takes one type argument: `{SIDECAR_TYPE_NAME}<T>`"),
            ));
        }
        if ident == "SEXP" {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "a `SEXP` sidecar field is not rooted: the GC frees its value once R drops \
                     its own references. Declare the field as `{SIDECAR_TYPE_NAME}<SEXP>`, which \
                     keeps the value in the external pointer's protection list"
                ),
            ));
        }
        if ident == "i32" || ident == "i16" || ident == "i8" {
            return Ok(SlotKind::ScalarInt);
        }
        if ident == "f64" || ident == "f32" {
            return Ok(SlotKind::ScalarReal);
        }
        if ident == "bool" || ident == "Rbool" {
            return Ok(SlotKind::ScalarLogical);
        }
        if ident == "u8" {
            return Ok(SlotKind::ScalarRaw);
        }
    }
    // Everything else uses conversion
    Ok(SlotKind::Conversion)
}

/// Parse struct fields for sidecar information.
///
/// Iterates over all fields, identifying `#[r_data]` markers. Fields with
/// `RSidecar` type are tracked as selector markers (at most one allowed);
/// all other `#[r_data]` fields become [`SidecarSlot`] entries with their
/// slot kind inferred from the field type.
///
/// Returns `Err` if more than one `RSidecar` field is found.
fn parse_sidecar_info(input: &DeriveInput, class_system: ClassSystem) -> syn::Result<SidecarInfo> {
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Ok(SidecarInfo {
                has_selector: false,
                slots: vec![],
                class_system,
            });
        }
    };

    let mut selector_fields: Vec<&Field> = vec![];
    let mut slots = vec![];
    let mut sidecars = 0usize;

    for field in fields.iter() {
        if !has_r_data_attr(field) {
            if sidecar_inner_type(&field.ty).is_some() {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    format!(
                        "a `{SIDECAR_TYPE_NAME}<T>` field needs `#[r_data]`, `#[r_data(ref)]`, \
                         `#[r_data(mut)]` or `#[r_data(ref, mut)]`: the derive attaches only the \
                         fields it knows to the external pointer"
                    ),
                ));
            }
            continue;
        }

        let options = parse_r_data_options(field)?;
        if is_rsidecar_type(field) {
            // RSidecar is the selector marker, not a slot
            selector_fields.push(field);
        } else if let Some(ref ident) = field.ident {
            // Any other type with #[r_data] becomes a slot
            let kind = slot_kind_for_type(&field.ty, sidecars)?;
            let access = if matches!(kind, SlotKind::Sidecar(_)) {
                sidecars += 1;
                let (get, set) = (options.access_ref.is_some(), options.access_mut.is_some());
                // A bare `#[r_data]` on a `Sidecar` field means `ref, mut`.
                SidecarAccess {
                    get: get || !set,
                    set: set || !get,
                }
            } else {
                SidecarAccess::default()
            };
            slots.push(SidecarSlot {
                name: ident.clone(),
                ty: field.ty.clone(),
                vis: field.vis.clone(),
                is_public: is_pub(field),
                kind,
                access,
                prop_doc: options.prop_doc,
                setter_invisible: options.setter_invisible,
            });
        }
    }

    // Check for multiple selectors
    if selector_fields.len() > 1 {
        return Err(syn::Error::new_spanned(
            selector_fields[1],
            "only one RSidecar field is allowed per struct",
        ));
    }

    Ok(SidecarInfo {
        has_selector: !selector_fields.is_empty(),
        slots,
        class_system,
    })
}

/// Generate the token stream for a sidecar getter function body.
///
/// Reads the slot and returns it as an R SEXP. The strategy depends on the
/// slot kind:
/// - `Sidecar`: returns the value from the external pointer's protection list,
///   after reattaching a live struct and flushing its pending value for the
///   field. It needs no live address, so it also reads a pointer that
///   `readRDS` brought back.
/// - Scalar kinds: reads the struct field (accessed via the external pointer
///   address) and wraps it with `Rf_Scalar*`.
/// - `Conversion`: clones the struct field and calls `IntoR::into_sexp`.
///
/// The body runs inside `with_r_unwind_protect` (see the emission site in
/// [`generate_sidecar_accessors`]); a non-external-pointer argument, a null
/// pointer address, or a wrong stored type panics with the same
/// `expected ExternalPtr<T>` message the main class-method path uses. The
/// panic is transported as a tagged condition and re-raised by the R wrapper.
fn generate_getter_body(struct_name: &syn::Ident, slot: &SidecarSlot) -> TokenStream {
    let field_name = &slot.name;

    // Helper: the live struct behind the pointer. `sidecar_r_struct` panics
    // (caught by the surrounding with_r_unwind_protect and raised as an R
    // error) on a non-pointer, a wrong type, or a pointer without an address
    // (a restored object: the classed errors) instead of silently returning
    // R_NilValue.
    let extract_ref = quote::quote! {
        let data: &#struct_name =
            ::miniextendr_api::externalptr::sidecar_r_struct::<#struct_name>(x);
    };

    match slot.kind {
        SlotKind::Sidecar(index) => {
            quote::quote! {
                unsafe {
                    ::miniextendr_api::externalptr::sidecar_r_get::<#struct_name>(x, #index)
                }
            }
        }
        SlotKind::ScalarInt => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_integer(data.#field_name)
                }
            }
        }
        SlotKind::ScalarReal => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_real(data.#field_name)
                }
            }
        }
        SlotKind::ScalarLogical => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_logical(data.#field_name)
                }
            }
        }
        SlotKind::ScalarRaw => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_raw(data.#field_name)
                }
            }
        }
        SlotKind::Conversion => {
            // Use IntoR trait for conversion (e.g., String -> character)
            let ty = &slot.ty;
            quote::quote! {
                use ::miniextendr_api::into_r::IntoR;
                unsafe {
                    #extract_ref
                    let val: #ty = data.#field_name.clone();
                    <#ty as IntoR>::into_sexp(val)
                }
            }
        }
    }
}

/// Generate the token stream for a sidecar setter function body.
///
/// Stores the incoming R SEXP `value` in the slot. The strategy depends on the
/// slot kind:
/// - `Sidecar`: validates `value` with `TryFromSexp::<T>` (a failure raises
///   the conversion error) and stores it in the external pointer's protection
///   list, which roots it; a live struct is reattached and its pending value
///   for the field dropped.
/// - Scalar kinds: writes the struct field, using `Rf_as*` or coercion for
///   single-element extraction;
///   an input that doesn't reduce to a single non-NA scalar raises a
///   conversion error instead of silently storing an NA/`false` sentinel.
/// - `Conversion`: uses `TryFromSexp::try_from_sexp`; a failed conversion
///   raises a conversion error instead of silently dropping the write.
///
/// Returns the external pointer `x` (for R's invisible return convention).
/// The body runs inside `with_r_unwind_protect` (see the emission site in
/// [`generate_sidecar_accessors`]); a bad receiver panics with the main-path
/// `expected ExternalPtr<T>` message and conversion failures return a tagged
/// condition SEXP with kind `conversion` — both are re-raised by the R
/// wrapper. Sidecar accessors have no `__miniextendr_call` slot (#344/#348),
/// so the tagged conditions carry null call attribution.
fn generate_setter_body(struct_name: &syn::Ident, slot: &SidecarSlot) -> TokenStream {
    let field_name = &slot.name;

    // Helper: the live struct behind the pointer, mutably. Failure modes
    // panic like the getter's (`sidecar_r_struct_mut`) instead of silently
    // no-op'ing.
    let extract_mut = quote::quote! {
        let data: &mut #struct_name =
            ::miniextendr_api::externalptr::sidecar_r_struct_mut::<#struct_name>(x);
    };

    // Conversion-failure condition, the argument error of #1594 on the
    // setter's `value` formal: `'<field>' must be <expected>: <reason>` with
    // the field's R-facing expectation. The message names the field, which is
    // what an R6 active binding (`obj$count <- x`) or an S7 property
    // (`obj@count <- x`) user assigned to: their condition call is the
    // binding's anonymous function. `e$param == "value"` (the formal), the
    // field's Rust type is `e$rust_type`, and the crate's
    // `conversion_error_class` applies. A `Conversion` slot's expectation comes
    // from the same type table as an argument's, else from the error at run
    // time, else `invalid '<field>' argument`
    // (`rust_conversion_builder::conversion_value_tokens`); a scalar
    // slot reads its value with `Rf_as*`, which has no error value, so the
    // reason is worded from the rejected value (`from_r::scalar_rejection_reason`).
    let crate_class = crate::crate_config::conversion_error_class();
    // A `Sidecar<T>` slot converts and reports its `T`.
    let value_ty = sidecar_inner_type(&slot.ty).unwrap_or(&slot.ty);
    let rust_type = crate::type_inspect::type_display(value_ty);
    let field_r_name = crate::naming::ident_name(field_name);
    let scalar_err = |expected: &str| -> proc_macro2::TokenStream {
        let prefix = format!("'{field_r_name}' must be {expected}");
        quote::quote! {
            ::miniextendr_api::error_value::conversion_condition_value(
                #prefix,
                "value",
                ::core::option::Option::Some(#rust_type),
                &[#(#crate_class),*],
                ::miniextendr_api::__mx_conversion_err_parts!(
                    ::miniextendr_api::from_r::scalar_rejection_reason(value),
                    true
                ),
                ::core::option::Option::None,
                ::core::option::Option::None,
            )
        }
    };

    // Conversion-failure value of a `Conversion` or `Sidecar` slot: the field
    // type's expectation, else the one the type declares or the error knows at
    // run time (a newtype, a `match_arg` enum field), as for an argument.
    let conversion_err = |ty: &syn::Type| -> proc_macro2::TokenStream {
        let prefix = crate::r_preconditions::conversion_expectation(ty, false)
            .map(|expected| format!("'{field_r_name}' must be {expected}"));
        let declared = crate::rust_conversion_builder::declared_expectation(ty);
        crate::rust_conversion_builder::conversion_value_tokens(
            &crate::rust_conversion_builder::ConversionSubject {
                expected: prefix.as_deref().map_or(
                    crate::rust_conversion_builder::Expected::FromError(Some(&declared)),
                    crate::rust_conversion_builder::Expected::Literal,
                ),
                quoted: &field_r_name,
                param: "value",
                nullable: crate::type_inspect::is_option_type(ty),
                rust_type: &rust_type,
            },
            &crate_class,
            &quote::quote!(::core::option::Option::None),
            syn::spanned::Spanned::span(ty),
        )
    };

    match slot.kind {
        SlotKind::Sidecar(index) => {
            let err_value = conversion_err(value_ty);
            // The receiver is checked before the value, so a wrong pointer is
            // reported as such whatever the value.
            quote::quote! {
                use ::miniextendr_api::TryFromSexp;
                unsafe {
                    ::miniextendr_api::externalptr::sidecar_r_check::<#struct_name>(x);
                    if let Err(e) = <#value_ty as TryFromSexp>::try_from_sexp(value) {
                        return #err_value;
                    }
                    ::miniextendr_api::externalptr::sidecar_r_set::<#struct_name>(
                        x, #index, value,
                    );
                    x
                }
            }
        }
        SlotKind::ScalarInt => {
            let err_value = scalar_err("a number");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_integer() {
                        Some(v) => v,
                        None => return #err_value,
                    };
                    x
                }
            }
        }
        SlotKind::ScalarReal => {
            let err_value = scalar_err("a number");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_real() {
                        Some(v) => v,
                        None => return #err_value,
                    };
                    x
                }
            }
        }
        SlotKind::ScalarLogical => {
            let err_value = scalar_err("TRUE or FALSE");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_logical() {
                        Some(v) => v,
                        None => return #err_value,
                    };
                    x
                }
            }
        }
        SlotKind::ScalarRaw => {
            let err_value = scalar_err("a raw value");
            quote::quote! {
                use ::miniextendr_api::{SexpExt, SEXPTYPE};
                unsafe {
                    #extract_mut
                    let raw_vec = value.coerce(SEXPTYPE::RAWSXP);
                    if raw_vec.len() == 0 {
                        return #err_value;
                    }
                    data.#field_name = raw_vec.raw_elt(0);
                    x
                }
            }
        }
        SlotKind::Conversion => {
            let ty = &slot.ty;
            let err_value = conversion_err(ty);
            quote::quote! {
                use ::miniextendr_api::TryFromSexp;
                unsafe {
                    #extract_mut
                    data.#field_name = match <#ty as TryFromSexp>::try_from_sexp(value) {
                        Ok(val) => val,
                        Err(e) => return #err_value,
                    };
                    x
                }
            }
        }
    }
}

/// Generate R wrapper code (roxygen-annotated R functions) for a single sidecar slot.
///
/// Produces getter and setter R functions that call the corresponding C entry
/// points via `.Call`. The generated R code includes roxygen tags (`@rdname`,
/// `@param`, `@return`, `@export`) for documentation.
///
/// All class systems generate the same standalone function pattern
/// (`Type_get_field` / `Type_set_field`); only the roxygen title suffix
/// differs. Class-specific integration (e.g., R6 active bindings, S7
/// properties) is handled separately by [`generate_class_integration_r_code`].
///
/// Each wrapper body carries the shared tagged-condition guard
/// ([`crate::method_return_builder::standalone_body`]) so Rust-origin
/// panics/conversion failures are re-raised as structured R conditions.
/// Note the sidecar C functions have **no** `.call` slot — the `.Call()`
/// passes no `.call` argument (#344/#348); `sys.call()` inside the guard
/// supplies fallback attribution instead.
fn generate_r_wrapper_for_slot(
    class_system: ClassSystem,
    type_name: &str,
    field_name: &str,
    getter_c_name: &str,
    setter_c_name: &str,
    setter_invisible: bool,
) -> String {
    // Only the roxygen title suffix differs between class systems.
    let suffix = match class_system {
        ClassSystem::Env => "",
        ClassSystem::R6 => " (for R6)",
        ClassSystem::S3 => " (for S3)",
        ClassSystem::S4 => " (for S4)",
        ClassSystem::S7 => " (for S7)",
        ClassSystem::Vctrs => " (for vctrs)",
    };
    let r_getter_name = format!("{}_get_{}", type_name, field_name);
    let r_setter_name = format!("{}_set_{}", type_name, field_name);
    let getter_body = crate::method_return_builder::standalone_body(
        &format!(".Call({getter_c_name}, x)"),
        ".val",
        "  ",
    );
    let setter_body = crate::method_return_builder::standalone_body(
        &format!(".Call({setter_c_name}, x, value)"),
        if setter_invisible {
            "invisible(x)"
        } else {
            "x"
        },
        "  ",
    );
    format!(
        r#"
#' Get `{field}` field from {type}{suffix}
#' @rdname {type}
#' @param x The {type} external pointer
#' @return The value of the `{field}` field
#' @export
{r_getter} <- function(x) {{
  {getter_body}
}}

#' Set `{field}` field on {type}{suffix}
#' @rdname {type}
#' @param x The {type} external pointer
#' @param value The new value to set
#' @return The {type} pointer ({visibility})
#' @export
{r_setter} <- function(x, value) {{
  {setter_body}
}}
"#,
        type = type_name,
        field = field_name,
        suffix = suffix,
        r_getter = r_getter_name,
        r_setter = r_setter_name,
        getter_body = getter_body,
        setter_body = setter_body,
        visibility = if setter_invisible { "invisibly" } else { "visibly" },
    )
}

/// Generate class-integrated R code for sidecar fields.
///
/// For R6: generates `Type$set("active", "field", ...)` calls that add active bindings
/// referencing the sidecar getter/setter .Call entrypoints.
///
/// For S7: generates `.rdata_properties_Type <- list(...)` with S7::new_property()
/// definitions that can be spliced into the S7 class's properties list.
///
/// Other class systems return empty strings (their standalone accessors suffice).
fn generate_class_integration_r_code(
    class_system: ClassSystem,
    type_name: &str,
    pub_slots: &[&SidecarSlot],
) -> String {
    if pub_slots.is_empty() {
        return String::new();
    }

    // Shared tagged-condition guard (one line); `sys.call()` supplies fallback
    // attribution because the sidecar C functions carry no `.call` slot
    // (#344/#348 — a sidecar `.Call()` passes no `.call` argument).
    let guard =
        |indent: &str| crate::method_return_builder::condition_check_lines(indent).join("\n");

    match class_system {
        ClassSystem::R6 => {
            // Generate $set("active", ...) calls for each sidecar field.
            // These are appended after the R6Class definition and add active bindings
            // that delegate to the sidecar .Call accessors.
            let mut code = String::new();
            code.push_str(&format!(
                "\n# Auto-generated active bindings for {type} sidecar fields.\n",
                type = type_name,
            ));
            code.push_str(
                "# These are applied when `r_data_accessors` is set on the impl block.\n",
            );
            code.push_str(&format!(
                ".rdata_active_bindings_{type} <- function(cls) {{\n\
                 \x20 # R CMD check: self/private are R6 runtime bindings (set by cls$set)\n\
                 \x20 self <- private <- NULL\n",
                type = type_name,
            ));
            for slot in pub_slots {
                let field = crate::naming::ident_name(&slot.name);
                // Must match the sidecar accessor's actual C symbol exactly (#1273
                // crate-prefixing) — routed through the shared naming.rs helpers.
                let getter_c = crate::naming::sidecar_getter_c_name(type_name, &field);
                let setter_c = crate::naming::sidecar_setter_c_name(type_name, &field);
                code.push_str(&format!(
                    "  cls$set(\"active\", \"{field}\", function(value) {{\n\
                     \x20   if (missing(value)) {{\n\
                     \x20     .val <- .Call({getter_c}, private$.ptr)\n\
                     {getter_guard}\n\
                     \x20     .val\n\
                     \x20   }} else {{\n\
                     \x20     .val <- .Call({setter_c}, private$.ptr, value)\n\
                     {setter_guard}\n\
                     \x20     {setter_return}\n\
                     \x20   }}\n\
                     \x20 }}, overwrite = TRUE)\n",
                    field = field,
                    getter_c = getter_c,
                    setter_c = setter_c,
                    getter_guard = guard("      "),
                    setter_guard = guard("      "),
                    setter_return = if slot.setter_invisible.unwrap_or(true) {
                        "invisible(self)"
                    } else {
                        "self"
                    },
                ));
            }
            code.push_str("}\n");
            code
        }
        ClassSystem::S7 => {
            // Generate a helper list of S7::new_property() definitions.
            // The S7 wrapper generator references this when `r_data_accessors` is set.
            let mut code = String::new();
            code.push_str(&format!(
                "\n# Auto-generated S7 property definitions for {type} sidecar fields.\n",
                type = type_name,
            ));
            code.push_str(&format!(
                ".rdata_properties_{type} <- list(\n",
                type = type_name,
            ));
            for (i, slot) in pub_slots.iter().enumerate() {
                let field = crate::naming::ident_name(&slot.name);
                // Must match the sidecar accessor's actual C symbol exactly (#1273
                // crate-prefixing) — routed through the shared naming.rs helpers.
                let getter_c = crate::naming::sidecar_getter_c_name(type_name, &field);
                let setter_c = crate::naming::sidecar_setter_c_name(type_name, &field);
                let comma = if i < pub_slots.len() - 1 { "," } else { "" };
                code.push_str(&format!(
                    "    {field} = S7::new_property(\n\
                     \x20       getter = function(self) {{\n\
                     \x20         .val <- .Call({getter_c}, self@.ptr)\n\
                     {getter_guard}\n\
                     \x20         .val\n\
                     \x20       }},\n\
                     \x20       setter = function(self, value) {{\n\
                     \x20         .val <- .Call({setter_c}, self@.ptr, value)\n\
                     {setter_guard}\n\
                     \x20         {setter_return}\n\
                     \x20       }}\n\
                     \x20   ){comma}\n",
                    field = field,
                    getter_c = getter_c,
                    setter_c = setter_c,
                    getter_guard = guard("          "),
                    setter_guard = guard("          "),
                    setter_return = if slot.setter_invisible.unwrap_or(false) {
                        "invisible(self)"
                    } else {
                        "self"
                    },
                    comma = comma,
                ));
            }
            code.push_str(")\n");
            code
        }
        // Other class systems don't need class-integrated code
        _ => String::new(),
    }
}

/// Generate sidecar accessor constants and `extern "C-unwind"` functions.
///
/// For each public `#[r_data]` field, generates:
/// - A getter FFI function (`C_<crate>__mx_rdata_get_Type_field`)
/// - A setter FFI function (`C_<crate>__mx_rdata_set_Type_field`)
/// - `R_CallMethodDef` entries for routine registration
/// - R wrapper function code (roxygen-documented)
/// - Class-integration code for R6 / S7 (active bindings / properties)
///
/// The generated constants are:
/// - `RDATA_CALL_DEFS_{TYPE}`: slice of `R_CallMethodDef` for registration
/// - `R_WRAPPERS_RDATA_{TYPE}`: string literal of R wrapper code
///
/// Returns `Err` if the struct has generic type parameters (`.Call` entrypoints
/// cannot be generic).
fn generate_sidecar_accessors(input: &DeriveInput, info: &SidecarInfo) -> syn::Result<TokenStream> {
    // Reject generic structs — .Call entrypoints cannot be generic
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "ExternalPtr does not support generic structs; \
             .Call entrypoints cannot be generic",
        ));
    }

    // If no selector or no public slots, nothing to register
    let pub_slots: Vec<_> = info.slots.iter().filter(|s| s.is_public).collect();
    if !info.has_selector || pub_slots.is_empty() {
        return Ok(quote::quote! {});
    }

    let name = &input.ident;
    let name_str = name.to_string();
    let name_upper = name_str.to_uppercase();

    // Generate getter/setter functions and R wrappers for each pub slot
    let mut c_functions = vec![];
    let mut r_wrappers = String::new();

    // Add documentation header that establishes the @name topic for sidecar accessors.
    // This ensures @rdname references have a valid target even if the main class
    // definition uses @noRd.
    if !pub_slots.is_empty() {
        let source_line = crate::roxygen::source_tag(format!(
            "Generated by miniextendr from `#[derive(ExternalPtr)]` on `{name_str}`"
        ))
        .map(|line| format!("{line}\n"))
        .unwrap_or_default();
        r_wrappers.push_str(&format!(
            r#"
#' @title {type} Sidecar Accessors
#' @name {type}
#' @description Getter and setter functions for `#[r_data]` fields on `{type}`.
{source_line}NULL

"#,
            type = name_str,
        ));
    }

    for slot in &pub_slots {
        let field_name = &slot.name;
        let field_name_str = crate::naming::ident_name(field_name);

        // C function names (crate-prefixed for webR cross-package symbol
        // uniqueness — #1273, routed through the shared naming.rs helpers)
        let getter_c_name = crate::naming::sidecar_getter_c_name(&name_str, &field_name_str);
        let setter_c_name = crate::naming::sidecar_setter_c_name(&name_str, &field_name_str);
        let getter_fn_name = Ident::new(&getter_c_name, Span::call_site());
        let setter_fn_name = Ident::new(&setter_c_name, Span::call_site());
        let source_location_doc = crate::source_location_doc(field_name.span());
        let getter_doc = format!(
            "Generated sidecar getter for `{}` field on Rust type `{}`.",
            field_name_str, name_str
        );
        let setter_doc = format!(
            "Generated sidecar setter for `{}` field on Rust type `{}`.",
            field_name_str, name_str
        );
        let getter_doc_lit = syn::LitStr::new(&getter_doc, field_name.span());
        let setter_doc_lit = syn::LitStr::new(&setter_doc, field_name.span());

        // Generate getter/setter bodies based on slot kind
        let getter_body = generate_getter_body(name, slot);
        let setter_body = generate_setter_body(name, slot);

        // Generate C getter function.
        //
        // The body runs under `with_r_unwind_protect` so Rust panics (and R
        // longjmps during allocation) become a tagged condition SEXP that the
        // R wrapper re-raises, matching the main call-slot path. Sidecar
        // accessors have no `__miniextendr_call` slot (#344/#348), so the
        // transport uses null call attribution; the R wrapper's guard passes
        // `sys.call()` to `.miniextendr_raise_condition` as the fallback.
        // Conditions queued inside the body (a `defer_warning!` in a field's
        // `IntoR` / `TryFromSexp`) are signalled at this call, through the
        // same `mark()` / `finish()` pair the generated wrappers have.
        c_functions.push(quote::quote! {
            #[doc = #getter_doc_lit]
            #[doc = #source_location_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub unsafe extern "C-unwind" fn #getter_fn_name(
                x: ::miniextendr_api::SEXP
            ) -> ::miniextendr_api::SEXP {
                let __miniextendr_deferred_mark = ::miniextendr_api::deferred_condition::mark();
                let __miniextendr_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || { #getter_body },
                    ::core::option::Option::None,
                );
                unsafe {
                    ::miniextendr_api::deferred_condition::finish(
                        __miniextendr_deferred_mark,
                        __miniextendr_value,
                        ::core::option::Option::None,
                    )
                }
            }
        });

        // Generate C setter function (same unwind/condition transport as the
        // getter above).
        c_functions.push(quote::quote! {
            #[doc = #setter_doc_lit]
            #[doc = #source_location_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub unsafe extern "C-unwind" fn #setter_fn_name(
                x: ::miniextendr_api::SEXP,
                value: ::miniextendr_api::SEXP,
            ) -> ::miniextendr_api::SEXP {
                let __miniextendr_deferred_mark = ::miniextendr_api::deferred_condition::mark();
                let __miniextendr_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || { #setter_body },
                    ::core::option::Option::None,
                );
                unsafe {
                    ::miniextendr_api::deferred_condition::finish(
                        __miniextendr_deferred_mark,
                        __miniextendr_value,
                        ::core::option::Option::None,
                    )
                }
            }
        });

        // Generate R_CallMethodDef entries via distributed slice
        let getter_c_name_cstr = format!("{}\0", getter_c_name);
        let setter_c_name_cstr = format!("{}\0", setter_c_name);
        let getter_cstr_lit =
            syn::LitByteStr::new(getter_c_name_cstr.as_bytes(), Span::call_site());
        let setter_cstr_lit =
            syn::LitByteStr::new(setter_c_name_cstr.as_bytes(), Span::call_site());
        let getter_def_ident = Ident::new(
            &format!(
                "__MX_CALL_DEF_RDATA_GET_{}_{}",
                name_upper,
                field_name_str.to_uppercase()
            ),
            Span::call_site(),
        );
        let setter_def_ident = Ident::new(
            &format!(
                "__MX_CALL_DEF_RDATA_SET_{}_{}",
                name_upper,
                field_name_str.to_uppercase()
            ),
            Span::call_site(),
        );

        c_functions.push(quote::quote! {
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            static #getter_def_ident: ::miniextendr_api::sys::R_CallMethodDef =
                ::miniextendr_api::sys::R_CallMethodDef {
                    name: #getter_cstr_lit.as_ptr().cast(),
                    fun: Some(unsafe { ::std::mem::transmute(#getter_fn_name as unsafe extern "C-unwind" fn(_) -> _) }),
                    numArgs: 1,
                };
        });
        c_functions.push(quote::quote! {
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            static #setter_def_ident: ::miniextendr_api::sys::R_CallMethodDef =
                ::miniextendr_api::sys::R_CallMethodDef {
                    name: #setter_cstr_lit.as_ptr().cast(),
                    fun: Some(unsafe { ::std::mem::transmute(#setter_fn_name as unsafe extern "C-unwind" fn(_, _) -> _) }),
                    numArgs: 2,
                };
        });

        // Generate R wrapper code based on class system
        let field_start = field_name.span().start();
        r_wrappers.push_str(&format!(
            "# Generated from Rust source line {}:{}\n# Wraps sidecar field `{}` on Rust type `{}` via `{}` and `{}`.\n",
            field_start.line,
            field_start.column + 1,
            field_name_str,
            name_str,
            getter_c_name,
            setter_c_name,
        ));
        r_wrappers.push_str(&generate_r_wrapper_for_slot(
            info.class_system,
            &name_str,
            &field_name_str,
            &getter_c_name,
            &setter_c_name,
            slot.setter_invisible.unwrap_or(true),
        ));
    }

    // Generate class-integrated R code for R6 and S7.
    // This code is appended after the standalone accessors so that
    // `r_data_accessors` in the impl block can auto-integrate sidecar fields.
    r_wrappers.push_str(&generate_class_integration_r_code(
        info.class_system,
        &name_str,
        &pub_slots,
    ));

    let const_name_wrappers = Ident::new(
        &format!("R_WRAPPERS_RDATA_{}", name_upper),
        Span::call_site(),
    );
    let source_location_doc = crate::source_location_doc(name.span());
    let source_line_lit = syn::LitInt::new(&name.span().start().line.to_string(), name.span());

    // For S7 class systems, emit MX_S7_SIDECAR_PROPS entries so the S7 codegen
    // can substitute @prop lines for sidecar properties at write time.
    let sidecar_prop_entries = if info.class_system == ClassSystem::S7 {
        let entries: Vec<_> = pub_slots
            .iter()
            .map(|slot| {
                let field_str = crate::naming::ident_name(&slot.name);
                let doc_str = slot
                    .prop_doc
                    .as_deref()
                    .unwrap_or("(undocumented sidecar property)");
                let entry_ident = Ident::new(
                    &format!(
                        "__MX_S7_SIDECAR_PROP_{}_{}",
                        name_upper,
                        field_str.to_uppercase()
                    ),
                    Span::call_site(),
                );
                quote::quote! {
                    #[doc(hidden)]
                    #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_S7_SIDECAR_PROPS), linkme(crate = ::miniextendr_api::linkme))]
                    static #entry_ident: ::miniextendr_api::registry::SidecarPropEntry =
                        ::miniextendr_api::registry::SidecarPropEntry {
                            rust_type: #name_str,
                            field_name: #field_str,
                            prop_doc: #doc_str,
                        };
                }
            })
            .collect();
        quote::quote! { #(#entries)* }
    } else {
        quote::quote! {}
    };

    Ok(quote::quote! {
        #(#c_functions)*

        /// Sidecar accessor R wrapper code via distributed slice.
        #[doc = #source_location_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        static #const_name_wrappers: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Sidecar,
                source_file: file!(),
                source_line: #source_line_lit,
                content: #r_wrappers,
            };

        #sidecar_prop_entries
    })
}

/// The `Sidecar<T>` slots of a type, in protection-list order.
fn sidecar_slots(info: &SidecarInfo) -> impl Iterator<Item = (usize, &SidecarSlot)> {
    info.slots.iter().filter_map(|slot| match slot.kind {
        SlotKind::Sidecar(index) => Some((index, slot)),
        _ => None,
    })
}

/// Generate the `TypedExternal` trait implementation for the derive target.
///
/// Produces the associated constants:
/// - `TYPE_NAME`: the struct name as a `&'static str`
/// - `TYPE_NAME_CSTR`: null-terminated byte string of the struct name
/// - `TYPE_ID_CSTR`: globally unique ID in the format
///   `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`,
///   using `CARGO_PKG_NAME`, `CARGO_PKG_VERSION`, and `module_path!()`.
/// - `R_SLOT_COUNT`: the number of `Sidecar<T>` fields, when there are any,
///   with the `__mx_visit_sidecars` hook that hands each of them, with its
///   slot index and name, to the handle (which writes their back-references,
///   flushes their pending values and detaches them).
///
/// Supports generic structs (generics are forwarded to the impl).
fn generate_typed_external(input: &DeriveInput, info: &SidecarInfo) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name_str = name.to_string();
    let name_lit = syn::LitStr::new(&name_str, name.span());
    let name_cstr = syn::LitByteStr::new(format!("{}\0", name_str).as_bytes(), name.span());
    let sidecars: Vec<_> = sidecar_slots(info).collect();
    let r_slot_count = (!sidecars.is_empty()).then(|| {
        let count = sidecars.len();
        let visits = sidecars.iter().map(|&(index, slot)| {
            let field = &slot.name;
            let field_name = crate::naming::ident_name(field);
            quote::quote! { visit(#index, #field_name, &self.#field); }
        });
        quote::quote! {
            const R_SLOT_COUNT: usize = #count;

            fn __mx_visit_sidecars(
                &self,
                visit: &mut dyn FnMut(usize, &'static str, &dyn ::miniextendr_api::externalptr::SidecarField),
            ) {
                #(#visits)*
            }
        }
    });

    // TYPE_ID_CSTR format: "<crate_name>@<crate_version>::<module_path>::<type_name>\0"
    //
    // Uses env!("CARGO_PKG_NAME") and env!("CARGO_PKG_VERSION") for the crate identifier,
    // ensuring two packages with the same type name from the same crate+version are compatible,
    // while different crate versions are considered distinct types.
    //
    // The module_path!() may include "crate::" prefix when compiled within the crate,
    // but combined with the explicit crate@version prefix, this is unambiguous.
    quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::TypedExternal for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #name_lit;
            const TYPE_NAME_CSTR: &'static [u8] = #name_cstr;
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(
                    env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
                    "::", module_path!(), "::", #name_lit, "\0"
                ).as_bytes();
            #r_slot_count
        }
    }
}

/// Generate the Rust accessors of each `Sidecar<T>` field, as inherent
/// methods with the field's visibility: `fn keys(&self) -> T` for
/// `#[r_data(ref)]`, `fn set_keys(&mut self, value: T)` for `#[r_data(mut)]`,
/// both for `#[r_data(ref, mut)]` or a bare `#[r_data]`.
///
/// A non-`pub` accessor is `#[allow(dead_code)]`: it is generated whether or
/// not the crate calls it. The accessors (and the `TypedExternal` hook) read
/// the fields, so rustc sees them used.
fn generate_sidecar_rust_accessors(input: &DeriveInput, info: &SidecarInfo) -> TokenStream {
    let methods: Vec<TokenStream> = sidecar_slots(info)
        .flat_map(|(_, slot)| {
            let inner = sidecar_inner_type(&slot.ty).expect("a Sidecar slot has an inner type");
            let field = &slot.name;
            let field_name = crate::naming::ident_name(field);
            let vis = &slot.vis;
            let allow = (!matches!(vis, Visibility::Public(_)))
                .then(|| quote::quote! { #[allow(dead_code)] });
            let mut methods = Vec::with_capacity(2);
            if slot.access.get {
                let doc = format!(
                    "The value of the `{field_name}` sidecar field: the R value the external \
                     pointer keeps, converted to its Rust type (a copy for a converting type), \
                     after flushing a pending value; or the pending value of a field not \
                     attached to a pointer. On R's main thread."
                );
                methods.push(quote::quote! {
                    #[doc = #doc]
                    #allow
                    #vis fn #field(&self) -> #inner {
                        self.#field.__mx_get()
                    }
                });
            }
            if slot.access.set {
                let setter = quote::format_ident!("set_{}", field_name);
                let doc = format!(
                    "Stores `value` in the `{field_name}` sidecar field: in the external \
                     pointer's protection list when attached to one, else as the pending value \
                     the next wrap moves there. On R's main thread."
                );
                methods.push(quote::quote! {
                    #[doc = #doc]
                    #allow
                    #vis fn #setter(&mut self, value: #inner) {
                        self.#field.__mx_set(value)
                    }
                });
            }
            methods
        })
        .collect();
    if methods.is_empty() {
        return TokenStream::new();
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote::quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#methods)*
        }
    }
}

/// Generate the `IntoExternalPtr` marker trait impl.
///
/// This marker trait enables the blanket `impl<T: IntoExternalPtr> IntoR for T`
/// in miniextendr-api, allowing the type to be returned directly from functions.
fn generate_into_external_ptr(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::IntoExternalPtr for #name #ty_generics #where_clause {}
    }
}

/// Generate the concrete `IntoRVecElement` impl (issue #1284).
///
/// Lights up the single `impl<T: IntoRVecElement> IntoR for Vec<T>` blanket in
/// miniextendr-api, so `-> Vec<MyType>` returns compile and produce a `VECSXP`
/// of external pointers. The impl must be concrete (not a blanket over
/// `IntoExternalPtr`) because the `MatchArg` bridge already occupies a blanket
/// on `IntoRVecElement` and two blankets would collide (E0119) — the same
/// funnel design `#[derive(IntoR)]` newtypes use (see
/// `miniextendr_api::newtype` module docs).
///
/// GC discipline: `ExternalPtr::collect_into_r_list` allocates each
/// `EXTPTRSXP` directly into the already-protected destination list, so no
/// element is ever live-but-unrooted across a later allocation.
fn generate_into_r_vec_element(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote::quote! {
        impl #impl_generics ::miniextendr_api::IntoRVecElement for #name #ty_generics #where_clause {
            #[inline]
            fn elements_into_sexp(values: ::std::vec::Vec<Self>) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::<Self>::collect_into_r_list(values)
            }
        }
    }
}

/// Main entry point for `#[derive(ExternalPtr)]`.
///
/// Orchestrates the full derive expansion:
/// 1. Parses `#[externalptr(...)]` attributes for class system selection.
/// 2. Analyzes struct fields for `#[r_data]` sidecar slots.
/// 3. Generates `TypedExternal` impl (type identity for `ExternalPtr<T>`).
/// 4. Generates `IntoExternalPtr` marker impl (enables `IntoR` blanket impl)
///    and a concrete `IntoRVecElement` impl (enables the `IntoR for Vec<T>`
///    blanket, so `-> Vec<MyType>` returns a `VECSXP` of external pointers —
///    #1284). Both are suppressed when `emit_into_r_marker` is `false`, which
///    the struct-level `#[miniextendr(prefer = "native")]` path uses so its
///    concrete `AsRNative` `IntoR` does not collide with the blanket
///    `IntoExternalPtr` `IntoR` (E0119, #1283).
/// 5. Generates sidecar accessor FFI functions, registration constants, and R wrappers.
///
/// Returns the combined token stream of all generated items.
pub fn derive_external_ptr(
    input: DeriveInput,
    emit_into_r_marker: bool,
) -> syn::Result<TokenStream> {
    // Parse class system from #[externalptr(...)] attribute
    let class_system = parse_externalptr_attrs(&input)?;

    // Parse sidecar information from struct fields
    let sidecar_info = parse_sidecar_info(&input, class_system)?;

    let typed_external = generate_typed_external(&input, &sidecar_info);
    let sidecar_rust_accessors = generate_sidecar_rust_accessors(&input, &sidecar_info);
    let (into_external_ptr, into_r_vec_element) = if emit_into_r_marker {
        (
            generate_into_external_ptr(&input),
            generate_into_r_vec_element(&input),
        )
    } else {
        (
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        )
    };
    let sidecar_accessors = generate_sidecar_accessors(&input, &sidecar_info)?;
    let erased_wrapper = generate_erased_wrapper(&input);

    Ok(quote::quote! {
        #typed_external
        #sidecar_rust_accessors
        #into_external_ptr
        #into_r_vec_element
        #sidecar_accessors
        #erased_wrapper
    })
}

/// Generate the type-erased wrapper infrastructure for trait ABI dispatch.
///
/// For `#[derive(ExternalPtr)] struct MyCounter { ... }` generates:
/// - `__MxWrapperMyCounter` - repr(C) wrapper with mx_erased header + data
/// - `__MX_TAG_MYCOUNTER` - concrete type tag (FNV-1a hash of module_path + type)
/// - `__mx_drop_mycounter` - destructor for R GC
/// - `__MX_BASE_VTABLE_MYCOUNTER` - base vtable with universal_query
/// - `__mx_wrap_mycounter` - constructor returning `*mut mx_erased`
fn generate_erased_wrapper(input: &DeriveInput) -> TokenStream {
    let type_ident = &input.ident;

    // Only for non-generic types (generics can't participate in trait dispatch)
    if !input.generics.params.is_empty() {
        return quote::quote! {};
    }

    let type_upper = type_ident.to_string().to_uppercase();
    let type_lower = type_ident.to_string().to_lowercase();

    let wrapper_name = quote::format_ident!("__MxWrapper{}", type_ident);
    let base_vtable_name = quote::format_ident!("__MX_BASE_VTABLE_{}", type_upper);
    let concrete_tag_name = quote::format_ident!("__MX_TAG_{}", type_upper);
    let drop_fn_name = quote::format_ident!("__mx_drop_{}", type_lower);
    let wrap_fn_name = quote::format_ident!("__mx_wrap_{}", type_lower);
    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let tag_path = format!("::{}", type_ident);

    quote::quote! {
        #[doc = concat!(
            "Type-erased wrapper for `",
            stringify!(#type_ident),
            "` with trait dispatch support."
        )]
        #[doc = "Generated by `#[derive(ExternalPtr)]`."]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[repr(C)]
        #[doc(hidden)]
        struct #wrapper_name {
            pub erased: ::miniextendr_api::abi::mx_erased,
            pub data: #type_ident,
        }

        #[doc(hidden)]
        const #concrete_tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        #[doc(hidden)]
        unsafe extern "C" fn #drop_fn_name(ptr: *mut ::miniextendr_api::abi::mx_erased) {
            if ptr.is_null() {
                return;
            }
            let wrapper = ptr.cast::<#wrapper_name>();
            // A panicking Drop impl must not unwind across the C-ABI boundary.
            // `drop_catching_panic` catches any panic and aborts instead.
            ::miniextendr_api::externalptr::drop_catching_panic(|| {
                unsafe { drop(Box::from_raw(wrapper)); }
            });
        }

        #[doc(hidden)]
        static #base_vtable_name: ::miniextendr_api::abi::mx_base_vtable =
            ::miniextendr_api::abi::mx_base_vtable {
                drop: #drop_fn_name,
                concrete_tag: #concrete_tag_name,
                query: ::miniextendr_api::registry::universal_query,
                data_offset: ::std::mem::offset_of!(#wrapper_name, data),
            };

        #[doc(hidden)]
        fn #wrap_fn_name(data: #type_ident) -> *mut ::miniextendr_api::abi::mx_erased {
            let wrapper = Box::new(#wrapper_name {
                erased: ::miniextendr_api::abi::mx_erased {
                    base: &#base_vtable_name,
                },
                data,
            });
            Box::into_raw(wrapper).cast::<::miniextendr_api::abi::mx_erased>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClassSystem, generate_class_integration_r_code, generate_r_wrapper_for_slot};

    const ALL_CLASS_SYSTEMS: [ClassSystem; 6] = [
        ClassSystem::Env,
        ClassSystem::R6,
        ClassSystem::S3,
        ClassSystem::S4,
        ClassSystem::S7,
        ClassSystem::Vctrs,
    ];

    /// Sidecar accessor C functions take only `x` (getter) or `x, value` (setter) —
    /// no `__miniextendr_call` parameter. The R wrappers must pass no `.call` argument
    /// because it would be counted as an extra positional argument by `.Call()`, causing
    /// "Incorrect number of arguments" errors at runtime. Cover every class system variant.
    #[test]
    fn sidecar_accessors_pass_no_call_slot() {
        let getter_c = "C__mx_rdata_get_T_f";
        let setter_c = "C__mx_rdata_set_T_f";

        for cs in ALL_CLASS_SYSTEMS {
            let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, setter_c, true);
            // Correct form: no .call argument — the C function only accepts x (getter) or x, value (setter).
            assert!(
                out.contains(&format!(".Call({getter_c}, x)")),
                "{cs:?} getter should call without .call:\n{out}"
            );
            assert!(
                out.contains(&format!(".Call({setter_c}, x, value)")),
                "{cs:?} setter should call without .call:\n{out}"
            );
            // No `.call =` in either spelling.
            assert!(
                !out.contains(".call ="),
                "{cs:?} sidecar wrapper must not pass a .call argument:\n{out}"
            );
        }
    }

    /// Every sidecar R wrapper (getter and setter, all class systems) must
    /// carry the tagged-condition guard so Rust panics / conversion failures
    /// transported by `with_r_unwind_protect(…, None)` are re-raised as
    /// structured R conditions instead of leaking the tagged SEXP to the user.
    #[test]
    fn sidecar_accessors_reraise_tagged_conditions() {
        let getter_c = "C__mx_rdata_get_T_f";
        let setter_c = "C__mx_rdata_set_T_f";

        for cs in ALL_CLASS_SYSTEMS {
            let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, setter_c, true);
            assert_eq!(
                out.matches(".miniextendr_raise_condition(.val, sys.call())")
                    .count(),
                2,
                "{cs:?} getter+setter should each carry the condition guard:\n{out}"
            );
        }
    }

    /// A sidecar setter's conversion failure is the argument error of #1594
    /// on its `value` formal (`e$param`), naming the field: `'f' must be
    /// <expected>`, the field's Rust type as `e$rust_type`, and the reason probed from the error (a
    /// `Conversion` slot) or worded from the rejected value (a scalar slot).
    #[test]
    fn sidecar_setter_raises_the_argument_error() {
        let body = |ty: syn::Type, kind: super::SlotKind| {
            let slot = super::SidecarSlot {
                name: syn::Ident::new("f", proc_macro2::Span::call_site()),
                ty,
                vis: syn::parse_quote!(pub),
                is_public: true,
                kind,
                access: super::SidecarAccess::default(),
                prop_doc: None,
                setter_invisible: None,
            };
            super::generate_setter_body(
                &syn::Ident::new("T", proc_macro2::Span::call_site()),
                &slot,
            )
            .to_string()
        };

        let s = body(syn::parse_quote!(i32), super::SlotKind::ScalarInt);
        assert!(s.contains("\"'f' must be a number\" , \"value\""), "{s}");
        assert!(s.contains("scalar_rejection_reason (value)"), "{s}");
        assert!(s.contains("Some (\"i32\")"), "{s}");
        assert!(s.contains("__mx_conversion_err_parts ! ("), "{s}");
        assert!(!s.contains("failed to convert"), "{s}");

        let s = body(syn::parse_quote!(bool), super::SlotKind::ScalarLogical);
        assert!(s.contains("\"'f' must be TRUE or FALSE\""), "{s}");

        let s = body(syn::parse_quote!(Vec<String>), super::SlotKind::Conversion);
        assert!(s.contains("\"'f' must be character\" , \"value\""), "{s}");
        assert!(s.contains("Some (\"Vec<String>\")"), "{s}");
        assert!(s.contains(", true)"), "{s}");

        // No static expectation: the error may supply one at run time (a
        // `match_arg` enum field), else `invalid 'f' argument`.
        let s = body(syn::parse_quote!(MyType), super::SlotKind::Conversion);
        assert!(s.contains("__mx_conversion_expectation ! (e)"), "{s}");
        assert!(s.contains("conversion_prefix (\"f\" , false ,"), "{s}");
        assert!(
            s.contains(") , \"value\" , :: core :: option :: Option :: Some (\"MyType\")"),
            "{s}"
        );
        assert!(s.contains("Some (\"MyType\")"), "{s}");
    }

    /// The R6 active-binding and S7 property integration code also call the
    /// sidecar C entrypoints directly; they need the same guard (with
    /// `sys.call()` fallback attribution) and must not pass `.call =`.
    #[test]
    fn sidecar_class_integration_reraises_tagged_conditions() {
        let slot = super::SidecarSlot {
            name: syn::Ident::new("f", proc_macro2::Span::call_site()),
            ty: syn::parse_quote!(i32),
            vis: syn::parse_quote!(pub),
            is_public: true,
            kind: super::SlotKind::ScalarInt,
            access: super::SidecarAccess::default(),
            prop_doc: None,
            setter_invisible: None,
        };
        let slots = [&slot];

        for cs in [ClassSystem::R6, ClassSystem::S7] {
            let out = generate_class_integration_r_code(cs, "T", &slots);
            assert_eq!(
                out.matches(".miniextendr_raise_condition(.val, sys.call())")
                    .count(),
                2,
                "{cs:?} integration getter+setter should each carry the condition guard:\n{out}"
            );
            assert!(
                !out.contains(".call ="),
                "{cs:?} integration code must not pass a .call argument:\n{out}"
            );
        }
    }

    #[test]
    fn field_setter_visibility_reaches_standalone_and_integrated_wrappers() {
        for (attribute, explicit) in [
            ("", None),
            ("setter = \"visible\"", Some(false)),
            ("setter = \"invisible\"", Some(true)),
        ] {
            let input: syn::DeriveInput = syn::parse_str(&format!(
                "struct Example {{ #[r_data({attribute})] pub value: i32 }}"
            ))
            .unwrap();
            for system in ALL_CLASS_SYSTEMS {
                let info = super::parse_sidecar_info(&input, system).unwrap();
                let slot = &info.slots[0];
                assert_eq!(slot.setter_invisible, explicit);
                let standalone = generate_r_wrapper_for_slot(
                    system,
                    "Example",
                    "value",
                    "get_value",
                    "set_value",
                    slot.setter_invisible.unwrap_or(true),
                );
                assert_eq!(
                    standalone.contains("invisible(x)"),
                    explicit.unwrap_or(true)
                );
                assert!(standalone.contains(if explicit == Some(false) {
                    "pointer (visibly)"
                } else {
                    "pointer (invisibly)"
                }));
                let integrated = generate_class_integration_r_code(system, "Example", &[slot]);
                match system {
                    ClassSystem::R6 => assert_eq!(
                        integrated.contains("invisible(self)"),
                        explicit.unwrap_or(true)
                    ),
                    ClassSystem::S7 => assert_eq!(
                        integrated.contains("invisible(self)"),
                        explicit.unwrap_or(false)
                    ),
                    _ => assert!(integrated.is_empty()),
                }
            }
        }
    }

    #[test]
    fn setter_visibility_and_property_docs_can_share_or_split_attributes() {
        for attrs in [
            "#[r_data(prop_doc = \"A counter\", setter = \"visible\")]",
            "#[r_data] #[r_data(setter = \"visible\", prop_doc = \"A counter\")]",
        ] {
            let input: syn::DeriveInput =
                syn::parse_str(&format!("struct Example {{ {attrs} pub value: i32 }}")).unwrap();
            let info = super::parse_sidecar_info(&input, ClassSystem::S7).unwrap();
            assert_eq!(info.slots[0].prop_doc.as_deref(), Some("A counter"));
            assert_eq!(info.slots[0].setter_invisible, Some(false));
        }
    }

    /// `Sidecar` fields count among themselves: the scalars and conversion
    /// fields between them take no protection-list position. The derive
    /// emits the count, the visitor hook, and typed accessors with each
    /// field's visibility; the R accessors read and write the protection
    /// list by index.
    #[test]
    fn sidecar_fields_number_their_own_positions() {
        let input: syn::DeriveInput = syn::parse_str(
            "struct Engine { #[r_data] _r: RSidecar, #[r_data] pub a: i32, \
             #[r_data] pub keys: Sidecar<Vec<i32>>, #[r_data] pub c: String, \
             #[r_data] steps: Sidecar<Option<List>> }",
        )
        .unwrap();
        let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
        let kinds: Vec<_> = info.slots.iter().map(|slot| slot.kind).collect();
        assert_eq!(
            kinds,
            [
                super::SlotKind::ScalarInt,
                super::SlotKind::Sidecar(0),
                super::SlotKind::Conversion,
                super::SlotKind::Sidecar(1),
            ]
        );
        // A bare `#[r_data]` means `ref, mut`.
        assert_eq!(
            info.slots[1].access,
            super::SidecarAccess {
                get: true,
                set: true
            }
        );

        let out = super::derive_external_ptr(input, true).unwrap().to_string();
        assert!(
            out.contains("const R_SLOT_COUNT : usize = 2usize ;"),
            "{out}"
        );
        assert!(
            out.contains("visit (0usize , \"keys\" , & self . keys) ; visit (1usize , \"steps\" , & self . steps) ;"),
            "{out}"
        );
        assert!(
            out.contains("pub fn keys (& self) -> Vec < i32 > { self . keys . __mx_get () }"),
            "{out}"
        );
        assert!(
            out.contains("pub fn set_keys (& mut self , value : Vec < i32 >) { self . keys . __mx_set (value) }"),
            "{out}"
        );
        // A private field gets private accessors (allowed to be unused) and
        // no R accessor.
        assert!(
            out.contains("# [allow (dead_code)] fn steps (& self) -> Option < List >"),
            "{out}"
        );
        assert!(
            out.contains(
                "# [allow (dead_code)] fn set_steps (& mut self , value : Option < List >)"
            ),
            "{out}"
        );
        assert!(!out.contains("get_steps"), "{out}");

        // The R accessors read and write the protection list, not the struct.
        let keys = &info.slots[1];
        let name = syn::Ident::new("Engine", proc_macro2::Span::call_site());
        let getter = super::generate_getter_body(&name, keys).to_string();
        assert!(
            getter.contains("sidecar_r_get :: < Engine > (x , 0usize)"),
            "{getter}"
        );
        assert!(!getter.contains("R_ExternalPtrAddr"), "{getter}");
        let setter = super::generate_setter_body(&name, keys).to_string();
        assert!(
            setter.contains("sidecar_r_check :: < Engine > (x)"),
            "{setter}"
        );
        // The value is validated as the field's `T` and reported as it.
        assert!(
            setter.contains("< Vec < i32 > as TryFromSexp > :: try_from_sexp (value)"),
            "{setter}"
        );
        assert!(setter.contains("\"'keys' must be integer\""), "{setter}");
        assert!(setter.contains("Some (\"Vec<i32>\")"), "{setter}");
        assert!(
            setter.contains("sidecar_r_set :: < Engine > (x , 0usize , value ,)"),
            "{setter}"
        );
    }

    /// `ref` generates only the getter, `mut` only the setter.
    #[test]
    fn sidecar_access_options_select_the_accessors() {
        let input: syn::DeriveInput = syn::parse_str(
            "struct Engine { #[r_data(ref)] pub keys: Sidecar<Vec<i32>>, \
             #[r_data(mut)] pub(crate) note: Sidecar<String> }",
        )
        .unwrap();
        let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
        assert_eq!(
            info.slots[0].access,
            super::SidecarAccess {
                get: true,
                set: false
            }
        );
        assert_eq!(
            info.slots[1].access,
            super::SidecarAccess {
                get: false,
                set: true
            }
        );
        let out = super::derive_external_ptr(input, true).unwrap().to_string();
        assert!(out.contains("pub fn keys (& self) -> Vec < i32 >"), "{out}");
        assert!(!out.contains("fn set_keys"), "{out}");
        assert!(
            out.contains(
                "# [allow (dead_code)] pub (crate) fn set_note (& mut self , value : String)"
            ),
            "{out}"
        );
        assert!(!out.contains("fn note ("), "{out}");
        // Both options together, in either order, and the hook still lists
        // every field.
        let input: syn::DeriveInput =
            syn::parse_str("struct Engine { #[r_data(mut, ref)] pub keys: Sidecar<Vec<i32>> }")
                .unwrap();
        let info = super::parse_sidecar_info(&input, ClassSystem::Env).unwrap();
        assert_eq!(
            info.slots[0].access,
            super::SidecarAccess {
                get: true,
                set: true
            }
        );
    }

    /// `ref` / `mut` are refused on a plain struct field, a `Sidecar` field
    /// needs `#[r_data]`, and a duplicate option is an error.
    #[test]
    fn sidecar_access_options_are_checked() {
        let refused = |source: &str| {
            let input: syn::DeriveInput = syn::parse_str(source).unwrap();
            super::parse_sidecar_info(&input, ClassSystem::Env)
                .err()
                .map(|err| err.to_string())
                .unwrap_or_else(|| panic!("`{source}` must be refused"))
        };
        let message = refused("struct E { #[r_data(ref)] pub count: i32 }");
        assert!(
            message.contains("apply to `Sidecar<T>` fields only"),
            "{message}"
        );
        let message = refused("struct E { pub keys: Sidecar<Vec<i32>> }");
        assert!(message.contains("needs `#[r_data]`"), "{message}");
        let message = refused("struct E { #[r_data(ref, ref)] pub keys: Sidecar<Vec<i32>> }");
        assert!(message.contains("duplicate `ref`"), "{message}");
        let message = refused("struct E { #[r_data] pub keys: Sidecar }");
        assert!(message.contains("takes one type argument"), "{message}");
    }

    /// A struct without `Sidecar` fields keeps the trait's default count and
    /// hook, and gets no accessors.
    #[test]
    fn no_sidecar_fields_emit_no_count_or_hook() {
        let input: syn::DeriveInput =
            syn::parse_str("struct Plain { #[r_data] _r: RSidecar, #[r_data] pub a: i32 }")
                .unwrap();
        let out = super::derive_external_ptr(input, true).unwrap().to_string();
        assert!(!out.contains("R_SLOT_COUNT"), "{out}");
        assert!(!out.contains("__mx_visit_sidecars"), "{out}");
        assert!(!out.contains("__mx_get"), "{out}");
    }

    /// A bare `SEXP` under `#[r_data]` is refused: nothing would root it.
    #[test]
    fn bare_sexp_sidecar_field_is_refused() {
        let input: syn::DeriveInput =
            syn::parse_str("struct Engine { #[r_data] _r: RSidecar, #[r_data] pub keys: SEXP }")
                .unwrap();
        let Err(err) = super::parse_sidecar_info(&input, ClassSystem::Env) else {
            panic!("a SEXP sidecar field must be refused");
        };
        let message = err.to_string();
        assert!(message.contains("is not rooted"), "{message}");
        assert!(
            message.contains("Declare the field as `Sidecar<SEXP>`"),
            "{message}"
        );
    }
}
