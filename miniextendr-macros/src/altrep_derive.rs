//! Derive macros for ALTREP data traits.
//!
//! These macros auto-implement `AltrepLen` and `Alt*Data` traits for simple field-based
//! ALTREP types, reducing boilerplate for users.
//!
//! # Three paths: slice field, length field, manual
//!
//! `#[derive(AltrepInteger)]` (and the `Real` / `Logical` / `Raw` / `String`
//! / `Complex` variants) drive one of three code paths:
//!
//! 1. **Slice field** — a struct with one field the derive reads as a slice
//!    (`[T; N]`, `Vec<T>`, `Box<[T]>`, `&'static [T]`), or the one
//!    `#[altrep(field = "name")]` names (a tuple index works too), gets its
//!    length and elements from that field. Element `i` is
//!    `Coerce::<R>::coerce` of a clone of element `i`, with `R` fixed by the
//!    family (`i32`, `f64`, `Rboolean`, `u8`, `Rcomplex`); the string family
//!    takes `AsRef<str>` elements:
//!
//!    ```ignore
//!    impl Coerce<i32> for Die { fn coerce(self) -> i32 { self.value().into() } }
//!
//!    #[derive(AltrepInteger)]
//!    struct Dice<const N: usize> { value: [Die; N], _secret: () }
//!    ```
//!
//!    An element of the family's own R type (`i32` under `AltrepInteger`)
//!    is read by R in place; R writes through the pointer when the field is
//!    owned, and gets a copy of a `&'static [T]` field before it writes.
//!    The path is taken when `field` is given, or when none of `len`,
//!    `elt`, `elt_delegate` is given and the struct has no `len` / `length`
//!    field.
//!
//! 2. **Length field** — point the derive at struct fields that hold the
//!    vector length and a constant element value:
//!
//!    ```ignore
//!    #[derive(AltrepInteger)]
//!    #[altrep(len = "n", elt = "value", class = "Repeat")]
//!    struct Repeat { n: usize, value: i32 }
//!    ```
//!
//! 3. **Manual** — `#[altrep(manual)]` suppresses `AltrepLen` /
//!    `AltIntegerData` generation so you can write the element-access logic
//!    yourself. The low-level trait impls (`Altrep`, `AltVec`, family
//!    methods, `InferBase`) are still emitted — `#[altrep(no_lowlevel)]` is
//!    the additional escape hatch when you also want to control `Altrep` /
//!    `AltVec` directly. An enum takes only this path.
//!
//! Every path emits `Altrep`, `AltVec`, the family trait, `InferBase`,
//! `RegisterAltrep` and the class registration. `#[derive(Altrep)]` (see
//! `crate::altrep`) is the slice path with the R vector type chosen by the
//! element instead of a family.
//!
//! # Generic structs
//!
//! Const and type parameters are taken (type parameters must be `'static`,
//! which the derive requires): each instantiation is its own R class, named
//! by the class name and the parameters (`Dice_5`) and registered on its
//! first conversion. Lifetime parameters are refused: R keeps the struct in
//! an external pointer.
//!
//! # Guard mode attribute
//!
//! The derive recognises `#[altrep(unsafe)]` / `#[altrep(rust_unwind)]` /
//! `#[altrep(r_unwind)]` on the struct, which sets the runtime
//! `Altrep::GUARD` const that the trampolines in `altrep_bridge` dispatch
//! on. Default is `r_unwind` (safe for callbacks that call R APIs).
//!
//! # Derive validation rules
//!
//! Several `#[altrep(...)]` options conflict; the derive rejects illegal
//! combinations at compile time:
//!
//! | Option | Allowed on families | Conflicts with |
//! |---|---|---|
//! | `subset` | all atomic (integer, real, logical, raw, string, complex) | `dataptr`, `list` |
//! | `dataptr` | integer, real, logical, raw, string, complex | `subset`, `list` |
//! | `serialize` | all | generic structs (#2364) |
//! | `manual` | all | `field` |
//! | `no_lowlevel` | all | `duplicate` |
//! | `field` | integer, real, logical, raw, string, complex | `len`, `elt`, `elt_delegate`, `manual` |
//!
//! The list family rejects `dataptr` and `subset` — list elements are
//! arbitrary SEXPs, so there is no contiguous buffer to expose.
//!
//! On the slice path, `dataptr` is what the derive does anyway (R reads
//! native elements in place and gets a copy of any other), `subset` keeps
//! asking for your `AltrepExtractSubset` impl, and `class`, the guards,
//! `serialize`, `duplicate` and `no_lowlevel` mean what they mean elsewhere.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub(crate) mod shape;

use shape::{Selector, SliceField};

/// The family-specific part of the slice path: what element `i` is, and
/// whether R can read the field in place.
#[derive(Clone, Copy)]
enum SliceFamily {
    Integer,
    Real,
    Logical,
    Raw,
    Complex,
    String,
}

impl SliceFamily {
    /// The element bound, for generic structs and messages.
    fn element_bound(self) -> TokenStream {
        match self {
            Self::Integer => quote! { ::miniextendr_api::Coerce<i32> + ::core::clone::Clone },
            Self::Real => quote! { ::miniextendr_api::Coerce<f64> + ::core::clone::Clone },
            Self::Logical => {
                quote! { ::miniextendr_api::Coerce<::miniextendr_api::Rboolean> + ::core::clone::Clone }
            }
            Self::Raw => quote! { ::miniextendr_api::Coerce<u8> + ::core::clone::Clone },
            Self::Complex => {
                quote! { ::miniextendr_api::Coerce<::miniextendr_api::Rcomplex> + ::core::clone::Clone }
            }
            Self::String => quote! { ::core::convert::AsRef<str> },
        }
    }

    /// What an element must implement, for messages.
    fn requirement(self) -> &'static str {
        match self {
            Self::Integer => "Coerce<i32>",
            Self::Real => "Coerce<f64>",
            Self::Logical => "Coerce<Rboolean>",
            Self::Raw => "Coerce<u8>",
            Self::Complex => "Coerce<Rcomplex>",
            Self::String => "AsRef<str>",
        }
    }

    /// The R storage type R can read an element of in place: the family's
    /// own type. `None` for logical (no `Coerce<Rboolean>` element has R's
    /// layout) and string (R stores CHARSXPs).
    fn native_storage(self) -> Option<TokenStream> {
        match self {
            Self::Integer => Some(quote! { i32 }),
            Self::Real => Some(quote! { f64 }),
            Self::Raw => Some(quote! { u8 }),
            Self::Complex => Some(quote! { ::miniextendr_api::Rcomplex }),
            Self::Logical | Self::String => None,
        }
    }

    /// The data trait's `elt` (and zero-copy `as_slice`) over the field.
    fn data_methods(self, slice: &SliceField) -> TokenStream {
        let elem = &slice.elem;
        let values = slice.values();
        let span = elem.span();
        let element = quote! { ::core::clone::Clone::clone(&#values[i]) };
        let coerce = |target: TokenStream| {
            quote_spanned! {span=>
                <#elem as ::miniextendr_api::Coerce<#target>>::coerce(#element)
            }
        };
        let as_slice = |storage: TokenStream| {
            quote! {
                fn as_slice(&self) -> ::core::option::Option<&[#storage]> {
                    ::miniextendr_api::altrep_data::native_view::<#elem, #storage>(#values)
                }
            }
        };
        match self {
            Self::Integer => {
                let value = coerce(quote! { i32 });
                let as_slice = as_slice(quote! { i32 });
                quote! {
                    fn elt(&self, i: usize) -> i32 { #value }
                    #as_slice
                }
            }
            Self::Real => {
                let value = coerce(quote! { f64 });
                let as_slice = as_slice(quote! { f64 });
                quote! {
                    fn elt(&self, i: usize) -> f64 { #value }
                    #as_slice
                }
            }
            Self::Raw => {
                let value = coerce(quote! { u8 });
                let as_slice = as_slice(quote! { u8 });
                quote! {
                    fn elt(&self, i: usize) -> u8 { #value }
                    #as_slice
                }
            }
            Self::Complex => {
                let value = coerce(quote! { ::miniextendr_api::Rcomplex });
                let as_slice = as_slice(quote! { ::miniextendr_api::Rcomplex });
                quote! {
                    fn elt(&self, i: usize) -> ::miniextendr_api::Rcomplex { #value }
                    #as_slice
                }
            }
            Self::Logical => {
                let value = coerce(quote! { ::miniextendr_api::Rboolean });
                quote! {
                    fn elt(&self, i: usize) -> ::miniextendr_api::altrep_data::Logical {
                        <::miniextendr_api::altrep_data::Logical as ::core::convert::From<
                            ::miniextendr_api::Rboolean,
                        >>::from(#value)
                    }
                }
            }
            Self::String => {
                let value = quote_spanned! {span=>
                    ::core::convert::AsRef::<str>::as_ref(&#values[i])
                };
                quote! {
                    fn elt(&self, i: usize) -> ::core::option::Option<&str> {
                        ::core::option::Option::Some(#value)
                    }
                }
            }
        }
    }
}

/// Per-family configuration controlling low-level code generation for an ALTREP type family.
///
/// Each ALTREP family (integer, real, logical, raw, string, complex) has a distinct
/// set of runtime macros for trait implementations and different capabilities (e.g., only
/// some families support `dataptr` or `subset`). This struct captures those differences
/// so [`AltrepAttrs::generate_lowlevel`] can emit the correct code.
struct AltrepFamilyConfig<'a> {
    /// The derive's name, for messages (e.g., `"AltrepInteger"`).
    family_name: &'a str,
    /// The data trait's name, for messages (e.g., `"AltIntegerData"`).
    data_trait: &'a str,
    /// The element type of `AltrepDataptr<T>` that `#[altrep(dataptr)]` asks
    /// the type to implement (`i32` for logical too); `None` for string,
    /// whose `dataptr` materializes a STRSXP (`__impl_altvec_string_dataptr`).
    dataptr_elem: Option<TokenStream>,
    /// The R storage type the default `Dataptr` copies the elements into
    /// (`__impl_altvec_materializing_dataptr`); `None` for string, which
    /// materializes through `__impl_altvec_string_dataptr` too.
    materialize_elem: Option<TokenStream>,
    /// Whether this family supports the `subset` option for `Extract_subset` method
    /// registration.
    subset: bool,
    /// Name of the internal macro for type-specific method implementations
    /// (e.g., `"__impl_altinteger_methods"` for integer).
    methods_macro: &'a str,
    /// Name of the `impl_inferbase_*!` macro that provides `InferBase` for this family
    /// (e.g., `"impl_inferbase_integer"`).
    inferbase_macro: &'a str,
    /// Default guard mode for this family when no explicit guard is specified.
    /// All families default to `RUnwind`: string elt/dataptr call R APIs
    /// (`Rf_mkCharLenCE`), and the others share the default.
    default_guard: &'a str,
    /// The slice path's element conversion.
    slice: SliceFamily,
}

/// Parsed `#[altrep(...)]` attributes controlling ALTREP derive code generation.
///
/// These attributes are placed on the struct and parsed by all ALTREP family derives
/// (`AltrepInteger`, `AltrepReal`, etc.) to customize the generated trait implementations.
///
/// # Supported `#[altrep(...)]` keys
///
/// | Key | Type | Description |
/// |-----|------|-------------|
/// | `field = "field"` | `String` | The slice field (`[T; N]`, `Vec<T>`, `Box<[T]>`, `&'static [T]`) to read the length and elements from; a tuple index (`"0"`) works. Needed only when the struct has several. |
/// | `len = "field"` | `String` | Name (or tuple index) of the struct field that holds the vector length. Auto-detected if a field is named `len` or `length`. |
/// | `elt = "field"` | `String` | Name (or tuple index) of the struct field to return as the element value (produces a constant-value vector). If omitted, the default `elt()` returns `NA` / `NaN` / `0` / `None` depending on the family. |
/// | `elt_delegate = "field"` | `String` | Name (or tuple index) of a field whose own `elt(i)` gives each element. |
/// | `manual` | Flag | Skip automatic `AltrepLen` and `Alt*Data` trait generation. Use when you want to write those trait impls by hand (e.g., for custom `elt` logic, `no_na`, `sum`, etc.). The low-level trait impls (`Altrep`, `AltVec`, family methods, `InferBase`) are still emitted automatically — you do **not** need to write them yourself. Use `no_lowlevel` as an additional escape hatch if you also want to suppress those. |
/// | `no_lowlevel` | Flag | Suppress the automatic low-level trait impls. Use this when you want to provide your own `Altrep`, `AltVec`, and family-specific trait implementations. |
/// | `dataptr` | Flag | Enable `Dataptr` method registration, allowing R to get a direct pointer to the underlying data. Mutually exclusive with `subset`. Not supported for List. |
/// | `serialize` | Flag | Enable `Serialized_state` and `Unserialize` method registration for ALTREP serialization support. Not on generic structs (#2364). |
/// | `duplicate = "clone"` | String | Keep ALTREP on duplication: clone the owner (requires `Clone`) and copy the R cache. Omission retains R's default, which materializes an ordinary vector. |
/// | `subset` | Flag | Enable `Extract_subset` method registration. Mutually exclusive with `dataptr`. Supported on all atomic families. Not supported for List. |
/// | `unsafe` | Flag | Set native-read guard to `Unsafe` -- no panic protection. Allocating bridges stay protected. |
/// | `rust_unwind` | Flag | Set native-read guard to `RustUnwind` -- `catch_unwind` only; reads must not call fallible R APIs. |
/// | `r_unwind` | Flag | Set guard mode to `RUnwind` (default) -- uses `with_r_unwind_protect` for safe R API calls. |
/// | `class = "name"` | `String` | Override the ALTREP class name (default: struct name). |
struct AltrepAttrs {
    /// The slice field, set via `#[altrep(field = "field")]`.
    field: Option<Selector>,
    /// Field containing the vector length, set via `#[altrep(len = "field")]`.
    /// If `None`, auto-detection looks for fields named `len` or `length`.
    len_field: Option<Selector>,
    /// Field for constant-value element access, set via `#[altrep(elt = "field")]`.
    /// When set, `elt()` returns `self.{field}` for every index.
    elt_field: Option<Selector>,
    /// Field for delegated element access, set via `#[altrep(elt_delegate = "field")]`.
    /// When set, `elt()` calls `self.{field}.elt(i)`, delegating to the inner type's
    /// `AltIntegerData`/`AltRealData`/etc. implementation. Useful for wrapper types
    /// around `StreamingIntData`, `StreamingRealData`, etc.
    elt_delegate: Option<Selector>,
    /// Whether to generate the low-level trait impls (`Altrep`, `AltVec`,
    /// family methods, `InferBase`). Defaults to `true`. Set to `false` by
    /// `#[altrep(no_lowlevel)]`.
    generate_lowlevel: bool,
    /// Collected option flags (`dataptr`, `serialize`, `subset`) passed to the runtime macro.
    lowlevel_options: Vec<syn::Ident>,
    /// Guard mode override for ALTREP trampoline callbacks. Maps to `AltrepGuard` variants:
    /// - `Unsafe` -- no protection
    /// - `RustUnwind` -- `catch_unwind` only
    /// - `RUnwind` -- `with_r_unwind_protect` (default)
    guard: Option<syn::Ident>,
    /// Override the ALTREP class name. Default: struct name.
    class_name: Option<String>,
    /// Manual mode: skip `AltrepLen` and `Alt*Data` generation. User provides their own.
    /// Set by `#[altrep(manual)]`. Lowlevel traits + registration still generated.
    manual: bool,
    /// Opt-in duplicate policy; absent retains R's materializing default.
    duplicate: Option<syn::LitStr>,
}

impl AltrepAttrs {
    /// Parses all `#[altrep(...)]` attributes from a derive input struct.
    ///
    /// Multiple `#[altrep(...)]` attributes are supported and their contents are merged.
    /// Unknown keys produce a compile error, and so does a field selector
    /// naming no field of the struct.
    ///
    /// # Errors
    ///
    /// Returns `Err` if an `#[altrep(...)]` attribute has malformed syntax or contains
    /// an unknown key.
    fn parse(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut field = None;
        let mut len_field = None;
        let mut elt_field = None;
        let mut elt_delegate = None;
        let mut generate_lowlevel = true; // Default: generate
        let mut lowlevel_options = Vec::new();
        let mut guard = None;
        let mut class_name = None;
        let mut manual = false;
        let mut duplicate = None;

        for attr in &input.attrs {
            if !attr.path().is_ident("altrep") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let selector = |meta: &syn::meta::ParseNestedMeta| -> syn::Result<Selector> {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    Selector::parse(&lit)
                };
                if meta.path.is_ident("field") {
                    field = Some(selector(&meta)?);
                } else if meta.path.is_ident("len") {
                    len_field = Some(selector(&meta)?);
                } else if meta.path.is_ident("elt") {
                    elt_field = Some(selector(&meta)?);
                } else if meta.path.is_ident("elt_delegate") {
                    elt_delegate = Some(selector(&meta)?);
                } else if meta.path.is_ident("duplicate") {
                    if duplicate.is_some() {
                        return Err(meta.error("duplicate `duplicate` policy"));
                    }
                    let policy: syn::LitStr = meta.value()?.parse()?;
                    if policy.value() != "clone" {
                        return Err(syn::Error::new(
                            policy.span(),
                            "duplicate policy must be `clone`",
                        ));
                    }
                    duplicate = Some(policy);
                } else if meta.path.is_ident("no_lowlevel") {
                    generate_lowlevel = false;
                } else if meta.path.is_ident("manual") {
                    manual = true;
                } else if meta.path.is_ident("class") {
                    let name: syn::LitStr = meta.value()?.parse()?;
                    class_name = Some(name.value());
                } else if meta.path.is_ident("dataptr") {
                    lowlevel_options.push(syn::Ident::new("dataptr", meta.path.span()));
                } else if meta.path.is_ident("serialize") {
                    lowlevel_options.push(syn::Ident::new("serialize", meta.path.span()));
                } else if meta.path.is_ident("subset") {
                    lowlevel_options.push(syn::Ident::new("subset", meta.path.span()));
                } else if meta.path.is_ident("r#unsafe") || meta.path.is_ident("unsafe") {
                    guard = Some(syn::Ident::new("Unsafe", meta.path.span()));
                } else if meta.path.is_ident("rust_unwind") {
                    guard = Some(syn::Ident::new("RustUnwind", meta.path.span()));
                } else if meta.path.is_ident("r_unwind") {
                    guard = Some(syn::Ident::new("RUnwind", meta.path.span()));
                } else {
                    return Err(meta.error(
                        "unknown #[altrep(...)] attribute; expected one of: \
                         `field`, `len`, `elt`, `elt_delegate`, `manual`, `no_lowlevel`, \
                         `class`, `dataptr`, `serialize`, `subset`, `duplicate`, `unsafe`, \
                         `rust_unwind`, `r_unwind`",
                    ));
                }
                Ok(())
            })?;
        }

        if !generate_lowlevel && let Some(policy) = &duplicate {
            return Err(syn::Error::new(
                policy.span(),
                "`duplicate` requires generated low-level traits; remove `no_lowlevel`",
            ));
        }
        let attrs = Self {
            field,
            len_field,
            elt_field,
            elt_delegate,
            generate_lowlevel,
            lowlevel_options,
            guard,
            class_name,
            manual,
            duplicate,
        };
        for selector in [
            &attrs.field,
            &attrs.len_field,
            &attrs.elt_field,
            &attrs.elt_delegate,
        ]
        .into_iter()
        .flatten()
        {
            selector.check(input)?;
        }
        Ok(attrs)
    }

    /// The option flag `name`, if given.
    fn option(&self, name: &str) -> Option<&syn::Ident> {
        self.lowlevel_options.iter().find(|o| *o == name)
    }

    /// Refuse `serialize` on a generic struct: its classes are registered on
    /// first use, so a fresh session reading the save may not have the class.
    fn check_serialize(&self, generics: &syn::Generics) -> syn::Result<()> {
        match self.option("serialize") {
            Some(serialize) if shape::is_generic(generics) => Err(syn::Error::new(
                serialize.span(),
                "`serialize` is not available on a generic struct: each instantiation's ALTREP \
                 class is registered when it is first converted, so a fresh R session reading \
                 the save may not have it yet; without `serialize` R saves the plain vector \
                 (#2364)",
            )),
            _ => Ok(()),
        }
    }

    /// The slice field the derive reads, if it takes the slice path.
    fn slice_field(
        &self,
        input: &syn::DeriveInput,
        family: &AltrepFamilyConfig,
    ) -> syn::Result<Option<SliceField>> {
        if let Some(field) = &self.field {
            if self.manual {
                return Err(syn::Error::new(
                    field.span,
                    "`field` generates `AltrepLen` and the data trait, and `manual` means you \
                     write them: use one",
                ));
            }
            if self.len_field.is_some() || self.elt_field.is_some() || self.elt_delegate.is_some() {
                return Err(syn::Error::new(
                    field.span,
                    "`field` reads the length and the elements from the slice: drop `len`, \
                     `elt` and `elt_delegate`",
                ));
            }
            return shape::slice_field(input, Some(field), family.family_name);
        }
        if self.manual
            || self.len_field.is_some()
            || self.elt_field.is_some()
            || self.elt_delegate.is_some()
            || named_len_field(input).is_some()
        {
            return Ok(None);
        }
        shape::slice_field(input, None, family.family_name)
    }

    /// Extra methods in the existing base impl; R's DuplicateEX still copies
    /// attributes and object flags around this Duplicate hook.
    fn duplicate_methods(&self) -> TokenStream {
        if self.duplicate.is_none() {
            return quote! {};
        }
        quote! {
            , extra = {
                const HAS_DUPLICATE: bool = true;
                fn duplicate(x: ::miniextendr_api::SEXP, _deep: bool) -> ::miniextendr_api::SEXP {
                    unsafe { ::miniextendr_api::altrep_impl::duplicate_altrep_owner::<Self>(x) }
                }
            }
        }
    }

    /// Returns the length field, either from the explicit `len = "..."` attribute
    /// or by auto-detecting a field named `len` or `length` on the struct.
    ///
    /// # Errors
    ///
    /// Returns `Err` if no length field was specified and auto-detection fails.
    fn get_len_field(&self, input: &syn::DeriveInput) -> syn::Result<syn::Member> {
        if let Some(field) = &self.len_field {
            return Ok(field.member.clone());
        }
        named_len_field(input)
            .map(|ident| syn::Member::Named(ident.clone()))
            .ok_or_else(|| {
                syn::Error::new(
                    input.ident.span(),
                    format!(
                        "no length field found; specify with #[altrep(len = \"field_name\")], or \
                         give the struct a slice field ({}) to read the length and elements from",
                        shape::SLICE_KINDS
                    ),
                )
            })
    }

    /// Validates that the requested `#[altrep(...)]` option flags are compatible with
    /// the given ALTREP type family.
    ///
    /// Enforces two rules:
    /// 1. `subset` is only valid for families where `supports_subset` is `true`.
    /// 2. `dataptr` and `subset` are mutually exclusive.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a span pointing to the offending option identifier.
    fn validate_options(&self, family: &str, supports_subset: bool) -> syn::Result<()> {
        if let Some(subset) = self.option("subset") {
            if !supports_subset {
                return Err(syn::Error::new(
                    subset.span(),
                    format!("`subset` is not supported for {family}"),
                ));
            }
            if self.option("dataptr").is_some() {
                return Err(syn::Error::new(
                    subset.span(),
                    "`dataptr` and `subset` are mutually exclusive",
                ));
            }
        }
        Ok(())
    }

    /// Generates low-level ALTREP trait implementation code for a given type family.
    ///
    /// Emits the underlying trait-impl macros directly (Path (a) from #682,
    /// landed via #711/#933), each in its generic form
    /// (`{params} Type {predicates}`), which a concrete type passes with
    /// empty braces. The four items every arm produces are:
    ///
    /// 1. `__impl_altrep_base!` — `impl Altrep` (guard, `with_serialize`, `duplicate`).
    /// 2. an `AltVec` impl, one of:
    ///    - slice field, native family: an `AltrepDataptr` over the field
    ///      (R reads an element of the family's own type in place) and
    ///      `__impl_altvec_dataptr!`,
    ///    - direct dataptr (`__impl_altvec_dataptr!(.., <elem>)`; string routes
    ///      through `__impl_altvec_string_dataptr!`) — `dataptr` option,
    ///    - subset (`__impl_altvec_extract_subset!`) — `subset` option,
    ///    - materializing (`__impl_altvec_materializing_dataptr!(.., <storage>)`
    ///      or the string one) — otherwise,
    /// 3. `methods_macro!` — the family `Alt<Family>` impl,
    /// 4. `inferbase_macro!` — `impl InferBase`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if option validation fails (e.g., `subset` on an
    /// unsupported family, or `dataptr` combined with `subset`).
    fn generate_lowlevel(
        &self,
        name: &syn::Ident,
        generics: &syn::Generics,
        family: &AltrepFamilyConfig,
        slice: Option<&SliceField>,
    ) -> syn::Result<TokenStream> {
        let AltrepFamilyConfig {
            family_name,
            ref dataptr_elem,
            ref materialize_elem,
            subset,
            methods_macro,
            inferbase_macro,
            default_guard,
            slice: slice_family,
            ..
        } = *family;

        if !self.generate_lowlevel {
            return Ok(quote! {});
        }

        self.validate_options(family_name, subset)?;

        let target = shape::brace_target(name, generics);
        let guard = self
            .guard
            .clone()
            .unwrap_or_else(|| syn::Ident::new(default_guard, proc_macro2::Span::call_site()));

        // 1. Altrep base, with an explicit guard.
        let duplicate = self.duplicate_methods();
        let base_impl = if self.option("serialize").is_some() {
            quote! { ::miniextendr_api::__impl_altrep_base!(#target, #guard, with_serialize #duplicate); }
        } else {
            quote! { ::miniextendr_api::__impl_altrep_base!(#target, #guard #duplicate); }
        };

        // 2. AltVec.
        let string_dataptr = quote! { ::miniextendr_api::__impl_altvec_string_dataptr!(#target); };
        let materializing = || match materialize_elem {
            Some(storage) => quote! {
                ::miniextendr_api::__impl_altvec_materializing_dataptr!(#target, #storage);
            },
            None => string_dataptr.clone(),
        };
        let vec_impl = if self.option("subset").is_some() {
            quote! { ::miniextendr_api::__impl_altvec_extract_subset!(#target); }
        } else if let Some(slice) = slice {
            match slice_family.native_storage() {
                Some(storage) => {
                    let dataptr = slice_dataptr(name, generics, slice, &storage);
                    quote! {
                        #dataptr
                        ::miniextendr_api::__impl_altvec_dataptr!(#target, #storage);
                    }
                }
                None => materializing(),
            }
        } else if self.option("dataptr").is_some() {
            match dataptr_elem {
                Some(elem) => quote! { ::miniextendr_api::__impl_altvec_dataptr!(#target, #elem); },
                None => string_dataptr.clone(),
            }
        } else {
            materializing()
        };

        // 3. Family method impl (`impl Alt<Family>`).
        let methods_ident = syn::Ident::new(methods_macro, proc_macro2::Span::call_site());
        let methods_impl = quote! { ::miniextendr_api::#methods_ident!(#target); };

        // 4. InferBase.
        let inferbase_ident = syn::Ident::new(inferbase_macro, proc_macro2::Span::call_site());
        let inferbase_impl = quote! { ::miniextendr_api::#inferbase_ident!(#target); };

        Ok(quote! {
            #base_impl
            #vec_impl
            #methods_impl
            #inferbase_impl
        })
    }
}

/// A field named `len` or `length`, which the length-field path reads.
fn named_len_field(input: &syn::DeriveInput) -> Option<&syn::Ident> {
    let syn::Data::Struct(data) = &input.data else {
        return None;
    };
    data.fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .find(|ident| *ident == "len" || *ident == "length")
}

/// The slice path's `AltrepDataptr<storage>`: R reads an element of the
/// family's own type in place, and writes through it when the field is
/// owned; any other element (and a write to borrowed data) gets a copy.
fn slice_dataptr(
    name: &syn::Ident,
    generics: &syn::Generics,
    slice: &SliceField,
    storage: &TokenStream,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let elem = &slice.elem;
    let values = slice.values();
    let dataptr = match slice.values_mut() {
        Some(values_mut) => quote! {
            ::miniextendr_api::altrep_data::native_view_mut::<#elem, #storage>(#values_mut)
                .map(<[#storage]>::as_mut_ptr)
        },
        None => quote! { ::core::option::Option::None },
    };
    quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltrepDataptr<#storage>
            for #name #ty_generics #where_clause
        {
            fn dataptr(&mut self, _writable: bool) -> ::core::option::Option<*mut #storage> {
                #dataptr
            }

            fn dataptr_or_null(&self) -> ::core::option::Option<*const #storage> {
                ::miniextendr_api::altrep_data::native_view::<#elem, #storage>(#values)
                    .map(<[#storage]>::as_ptr)
            }
        }
    }
}

/// Generates an `impl AltrepLen for T` block that delegates to a named struct field.
fn generate_altrep_len(
    name: &syn::Ident,
    generics: &syn::Generics,
    len_field: &syn::Member,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltrepLen for #name #ty_generics #where_clause {
            fn altrep_len(&self) -> usize {
                self.#len_field
            }
        }
    }
}

/// The struct's shape checks shared by every family derive: no enum
/// without `manual` (the derive can't read one), no union.
fn check_data(
    input: &syn::DeriveInput,
    manual: bool,
    family_name: &str,
    data_trait: &str,
    requirement: Option<&str>,
) -> syn::Result<()> {
    match &input.data {
        syn::Data::Struct(_) => Ok(()),
        syn::Data::Enum(_) if manual => Ok(()),
        syn::Data::Enum(_) => {
            let ident = &input.ident;
            let element = requirement.map_or_else(String::new, |requirement| {
                format!(
                    "; to make a vector of `{ident}` values, put them in a struct field (`[{ident}; N]`, \
                     `Vec<{ident}>`) and implement `{requirement}` for `{ident}`"
                )
            });
            Err(syn::Error::new(
                crate::util::data_keyword_span(&input.data),
                format!(
                    "`#[derive({family_name})]` on an enum needs `#[altrep(manual)]`: the derive \
                     can't read a length or elements from an enum, so you implement `AltrepLen` \
                     and `{data_trait}`{element}"
                ),
            ))
        }
        syn::Data::Union(_) => Err(syn::Error::new(
            crate::util::data_keyword_span(&input.data),
            format!(
                "`#[derive({family_name})]` takes a struct, or an enum with `#[altrep(manual)]`"
            ),
        )),
    }
}

/// Shared implementation for all non-list ALTREP derive macros.
///
/// Generates three items:
/// 1. `impl AltrepLen` and `impl Alt*Data` -- from the slice field, or the
///    length field and `gen_elt_impl`; nothing under `manual`
/// 2. Low-level trait impls via [`AltrepAttrs::generate_lowlevel`]
/// 3. The registration (`crate::altrep::generate_direct_altrep_registration`)
///
/// # Arguments
///
/// * `input` -- The `DeriveInput` from the proc-macro.
/// * `data_trait_path` -- The fully qualified path to the data trait
///   (e.g., `::miniextendr_api::altrep_data::AltIntegerData`).
/// * `gen_elt_impl` -- A closure that receives the optional `elt_field` and
///   `elt_delegate` and returns a token stream for the `fn elt(...)` method
///   of the length-field path. If `elt_field` is `Some`, the closure
///   typically returns `self.{field}`; if `None`, it returns a
///   family-appropriate default (`NA_INTEGER`, `f64::NAN`, `0u8`,
///   `Logical::Na`, `None`, or `Rcomplex { NAN, NAN }`).
/// * `family` -- The [`AltrepFamilyConfig`] for this type family.
///
/// # Errors
///
/// Returns `Err` if attribute parsing fails, the struct's shape doesn't fit,
/// no length field can be found, or option validation fails.
fn derive_altrep_generic(
    mut input: syn::DeriveInput,
    data_trait_path: TokenStream,
    gen_elt_impl: impl FnOnce(Option<&syn::Member>, Option<&syn::Member>) -> TokenStream,
    family: &AltrepFamilyConfig,
) -> syn::Result<TokenStream> {
    let attrs = AltrepAttrs::parse(&input)?;
    check_data(
        &input,
        attrs.manual,
        family.family_name,
        family.data_trait,
        Some(family.slice.requirement()),
    )?;
    input.generics = shape::altrep_generics(&input.generics)?;
    attrs.check_serialize(&input.generics)?;
    let slice = attrs.slice_field(&input, family)?;
    if let Some(slice) = &slice
        && shape::has_type_params(&input.generics)
    {
        shape::add_bound(
            &mut input.generics,
            &slice.elem,
            family.slice.element_bound(),
        );
    }

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data_traits = if attrs.manual {
        quote! {}
    } else if let Some(slice) = &slice {
        let elem = &slice.elem;
        let values = slice.values();
        let methods = family.slice.data_methods(slice);
        quote! {
            impl #impl_generics ::miniextendr_api::altrep_data::AltrepLen for #name #ty_generics #where_clause {
                fn altrep_len(&self) -> usize {
                    <[#elem]>::len(#values)
                }
            }
            impl #impl_generics #data_trait_path for #name #ty_generics #where_clause {
                #methods
            }
        }
    } else {
        let len_field = attrs.get_len_field(&input)?;
        let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
        let elt_impl = gen_elt_impl(
            attrs.elt_field.as_ref().map(|s| &s.member),
            attrs.elt_delegate.as_ref().map(|s| &s.member),
        );
        quote! {
            #altrep_len_impl
            impl #impl_generics #data_trait_path for #name #ty_generics #where_clause {
                #elt_impl
            }
        }
    };

    let lowlevel_impl = attrs.generate_lowlevel(name, generics, family, slice.as_ref())?;

    // Generate full registration (TypedExternal, AltrepClass, RegisterAltrep, IntoR,
    // linkme entry, Ref/Mut).
    let class_name = attrs
        .class_name
        .clone()
        .unwrap_or_else(|| crate::naming::ident_name(name));
    let registration_impl =
        crate::altrep::generate_direct_altrep_registration(&input, &class_name, false)?;

    Ok(quote! {
        #data_traits
        #lowlevel_impl
        #registration_impl
    })
}

/// Derive macro entry point for `AltrepInteger`.
///
/// On the slice path, element `i` is `Coerce::<i32>::coerce` of element `i`
/// of the field, and an `i32` field is read in place. On the length-field
/// path, `elt()` returns `self.{elt_field}` as `i32` if `#[altrep(elt = "...")]`
/// is specified, or `NA_INTEGER` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `i32` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_integer(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltIntegerData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> i32 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> i32 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> i32 { ::miniextendr_api::altrep_traits::NA_INTEGER } }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepInteger",
            data_trait: "AltIntegerData",
            dataptr_elem: Some(quote! { i32 }),
            materialize_elem: Some(quote! { i32 }),
            subset: true,
            methods_macro: "__impl_altinteger_methods",
            inferbase_macro: "impl_inferbase_integer",
            default_guard: "RUnwind",
            slice: SliceFamily::Integer,
        },
    )
}

/// Derive macro entry point for `AltrepReal`.
///
/// On the slice path, element `i` is `Coerce::<f64>::coerce` of element `i`
/// of the field, and an `f64` field is read in place. On the length-field
/// path, `elt()` returns `self.{elt_field}` as `f64` if `#[altrep(elt = "...")]`
/// is specified, or `f64::NAN` (R's `NA_real_`) by default.
///
/// Supports `#[altrep(dataptr)]` for direct `f64` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_real(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRealData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> f64 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> f64 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> f64 { f64::NAN } }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepReal",
            data_trait: "AltRealData",
            dataptr_elem: Some(quote! { f64 }),
            materialize_elem: Some(quote! { f64 }),
            subset: true,
            methods_macro: "__impl_altreal_methods",
            inferbase_macro: "impl_inferbase_real",
            default_guard: "RUnwind",
            slice: SliceFamily::Real,
        },
    )
}

/// Derive macro entry point for `AltrepLogical`.
///
/// On the slice path, element `i` is `Coerce::<Rboolean>::coerce` of element
/// `i` of the field (R gets a copy when it asks for a pointer). On the
/// length-field path, `elt()` returns `self.{elt_field}.into()` as `Logical`
/// if `#[altrep(elt = "...")]` is specified, or `Logical::Na` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `i32` data pointer access (logicals are
/// stored as `i32` in R) and `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_logical(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltLogicalData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> ::miniextendr_api::altrep_data::Logical { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { self.#f.into() } }
            } else {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { ::miniextendr_api::altrep_data::Logical::Na } }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepLogical",
            data_trait: "AltLogicalData",
            dataptr_elem: Some(quote! { i32 }),
            materialize_elem: Some(quote! { ::miniextendr_api::RLogical }),
            subset: true,
            methods_macro: "__impl_altlogical_methods",
            inferbase_macro: "impl_inferbase_logical",
            default_guard: "RUnwind",
            slice: SliceFamily::Logical,
        },
    )
}

/// Derive macro entry point for `AltrepRaw`.
///
/// On the slice path, element `i` is `Coerce::<u8>::coerce` of element `i`
/// of the field, and a `u8` field is read in place. On the length-field
/// path, `elt()` returns `self.{elt_field}` as `u8` if `#[altrep(elt = "...")]`
/// is specified, or `0u8` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `u8` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_raw(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRawData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> u8 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> u8 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> u8 { 0 } }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepRaw",
            data_trait: "AltRawData",
            dataptr_elem: Some(quote! { u8 }),
            materialize_elem: Some(quote! { u8 }),
            subset: true,
            methods_macro: "__impl_altraw_methods",
            inferbase_macro: "impl_inferbase_raw",
            default_guard: "RUnwind",
            slice: SliceFamily::Raw,
        },
    )
}

/// Derive macro entry point for `AltrepString`.
///
/// On the slice path, element `i` is `AsRef::<str>::as_ref` of element `i`
/// of the field. On the length-field path, `elt()` returns
/// `Some(self.{elt_field}.as_ref())` as `Option<&str>` if
/// `#[altrep(elt = "...")]` is specified, or `None` (R's `NA_character_`) by default.
///
/// String ALTREP supports `#[altrep(dataptr)]` for materialized `STRSXP` dataptr
/// (via `__impl_altvec_string_dataptr`) and `#[altrep(subset)]` for `Extract_subset`.
/// Note: String dataptr materializes the entire vector into a cached `STRSXP` in the
/// data2 slot.
pub fn derive_altrep_string(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltStringData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> ::core::option::Option<&str> { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::core::option::Option<&str> { ::core::option::Option::Some(self.#f.as_ref()) } }
            } else {
                quote! { fn elt(&self, _i: usize) -> ::core::option::Option<&str> { ::core::option::Option::None } }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepString",
            data_trait: "AltStringData",
            dataptr_elem: None,
            materialize_elem: None,
            subset: true,
            methods_macro: "__impl_altstring_methods",
            inferbase_macro: "impl_inferbase_string",
            // String elt calls Rf_mkCharLenCE; dataptr calls Rf_allocVector + SET_STRING_ELT.
            // These R API calls can longjmp — must use RUnwind.
            default_guard: "RUnwind",
            slice: SliceFamily::String,
        },
    )
}

/// Derive macro entry point for `AltrepComplex`.
///
/// On the slice path, element `i` is `Coerce::<Rcomplex>::coerce` of element
/// `i` of the field, and an `Rcomplex` field is read in place. On the
/// length-field path, `elt()` returns `self.{elt_field}` as `Rcomplex` if
/// `#[altrep(elt = "...")]` is specified, or `Rcomplex { r: NAN, i: NAN }` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `Rcomplex` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_complex(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltComplexData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> ::miniextendr_api::Rcomplex { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::Rcomplex { self.#f } }
            } else {
                quote! {
                    fn elt(&self, _i: usize) -> ::miniextendr_api::Rcomplex {
                        ::miniextendr_api::Rcomplex { r: f64::NAN, i: f64::NAN }
                    }
                }
            }
        },
        &AltrepFamilyConfig {
            family_name: "AltrepComplex",
            data_trait: "AltComplexData",
            dataptr_elem: Some(quote! { ::miniextendr_api::Rcomplex }),
            materialize_elem: Some(quote! { ::miniextendr_api::Rcomplex }),
            subset: true,
            methods_macro: "__impl_altcomplex_methods",
            inferbase_macro: "impl_inferbase_complex",
            default_guard: "RUnwind",
            slice: SliceFamily::Complex,
        },
    )
}

/// Derive macro entry point for `AltrepList`.
///
/// Auto-implements `AltrepLen` and `AltListData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}[i]` as `SEXP` if
/// `#[altrep(elt = "...")]` is specified (the field should be indexable and return `SEXP`),
/// or `R_NilValue` by default.
///
/// List ALTREP does **not** support `#[altrep(dataptr)]`, `#[altrep(subset)]`
/// or `#[altrep(field)]` -- all are rejected at compile time.
/// `#[altrep(serialize)]` is supported on non-generic structs. The class
/// takes R's writes through `altrep_impl::altlist_set_elt`, which copies the
/// elements into data2 on the first one; `Elt` reads data2 from then on.
pub fn derive_altrep_list(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let attrs = AltrepAttrs::parse(&input)?;
    check_data(&input, attrs.manual, "AltrepList", "AltListData", None)?;
    input.generics = shape::altrep_generics(&input.generics)?;
    attrs.check_serialize(&input.generics)?;

    // List does not support dataptr, subset or a slice field.
    for opt in &attrs.lowlevel_options {
        if opt == "dataptr" || opt == "subset" {
            return Err(syn::Error::new(
                opt.span(),
                format!("`{opt}` is not supported for AltrepList"),
            ));
        }
    }
    if let Some(field) = &attrs.field {
        return Err(syn::Error::new(
            field.span,
            "`field` is not supported for AltrepList: name the length with `len` and the \
             SEXP elements with `elt`",
        ));
    }

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // In manual mode, skip AltrepLen + AltListData generation.
    let data_traits = if attrs.manual {
        quote! {}
    } else {
        let len_field = attrs.get_len_field(&input)?;
        let altrep_len_impl = generate_altrep_len(name, generics, &len_field);

        let elt_impl = if let Some(elt_field) = &attrs.elt_field {
            let elt_field = &elt_field.member;
            quote! {
                fn elt(&self, i: usize) -> ::miniextendr_api::SEXP {
                    self.#elt_field[i]
                }
            }
        } else {
            quote! {
                fn elt(&self, _i: usize) -> ::miniextendr_api::SEXP {
                    ::miniextendr_api::SEXP::nil()
                }
            }
        };

        quote! {
            #altrep_len_impl
            impl #impl_generics ::miniextendr_api::altrep_data::AltListData for #name #ty_generics #where_clause {
                #elt_impl
            }
        }
    };

    let has_serialize = attrs.option("serialize").is_some();

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else {
        // Direct emit (#933). The non-serialize arm inlines the body of the
        // retired `impl_altlist_from_data!` delegation: `__impl_altrep_base!`
        // with an explicit guard is token-identical to that macro's
        // hand-written `Altrep` impl, and the `elt` body matches its
        // `altrep_extract_ref` path. The serialize arm keeps its historical
        // `altrep_data1_as` elt body — switching it to `AltrepExtract` would
        // change behaviour for types that override extraction.
        let target = shape::brace_target(name, generics);
        let guard = attrs
            .guard
            .clone()
            .unwrap_or_else(|| syn::Ident::new("RUnwind", proc_macro2::Span::call_site()));
        let duplicate = attrs.duplicate_methods();
        let base_impl = if has_serialize {
            quote! { ::miniextendr_api::__impl_altrep_base!(#target, #guard, with_serialize #duplicate); }
        } else {
            quote! { ::miniextendr_api::__impl_altrep_base!(#target, #guard #duplicate); }
        };
        // Once `Set_elt` has copied the values into data2, they are R's.
        let written_elt = quote! {
            if let ::core::option::Option::Some(data2) = unsafe { ::miniextendr_api::altrep_data::materialized_altrep_data2(x) } {
                return ::miniextendr_api::SexpExt::vector_elt(&data2, i);
            }
        };
        let elt_impl = if has_serialize {
            quote! {
                fn elt(x: ::miniextendr_api::SEXP, i: ::miniextendr_api::R_xlen_t) -> ::miniextendr_api::SEXP {
                    #written_elt
                    // No `RefCell` borrow for the ALTREP data1 read, until #2103.
                    unsafe { ::miniextendr_api::altrep_data1_as::<#name #ty_generics>(x) }
                        .map(|d| <#name #ty_generics as ::miniextendr_api::altrep_data::AltListData>::elt(unsafe { d.as_ref_unchecked() }, i.max(0) as usize))
                        .unwrap_or(::miniextendr_api::SEXP::nil())
                }
            }
        } else {
            quote! {
                fn elt(x: ::miniextendr_api::SEXP, i: ::miniextendr_api::R_xlen_t) -> ::miniextendr_api::SEXP {
                    #written_elt
                    let data = unsafe {
                        <#name #ty_generics as ::miniextendr_api::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                    };
                    <#name #ty_generics as ::miniextendr_api::altrep_data::AltListData>::elt(data, i.max(0) as usize)
                }
            }
        };
        quote! {
            #base_impl
            impl #impl_generics ::miniextendr_api::altrep_traits::AltVec for #name #ty_generics #where_clause {}
            impl #impl_generics ::miniextendr_api::altrep_traits::AltList for #name #ty_generics #where_clause {
                #elt_impl

                const HAS_SET_ELT: bool = true;

                fn set_elt(x: ::miniextendr_api::SEXP, i: ::miniextendr_api::R_xlen_t, v: ::miniextendr_api::SEXP) {
                    unsafe { ::miniextendr_api::altrep_impl::altlist_set_elt::<#name #ty_generics>(x, i, v) }
                }
            }
            ::miniextendr_api::impl_inferbase_list!(#target);
        }
    };

    // Generate full registration (TypedExternal, AltrepClass, RegisterAltrep, IntoR,
    // linkme entry, Ref/Mut) — same as derive_altrep_generic.
    let class_name = attrs
        .class_name
        .clone()
        .unwrap_or_else(|| crate::naming::ident_name(name));
    let registration_impl =
        crate::altrep::generate_direct_altrep_registration(&input, &class_name, false)?;

    Ok(quote! {
        #data_traits
        #lowlevel_impl
        #registration_impl
    })
}

#[cfg(test)]
mod tests;
