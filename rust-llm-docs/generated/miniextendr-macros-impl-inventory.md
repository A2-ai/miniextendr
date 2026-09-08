# Trait impl inventory

Source: `target/doc/miniextendr_macros.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 97 | 0 |
| `Borrow` | 97 | 0 |
| `BorrowMut` | 97 | 0 |
| `Freeze` | 97 | 0 |
| `From` | 97 | 0 |
| `Into` | 97 | 0 |
| `RefUnwindSafe` | 97 | 0 |
| `Send` | 97 | 0 |
| `Sync` | 97 | 0 |
| `TryFrom` | 97 | 0 |
| `TryInto` | 97 | 0 |
| `Unpin` | 97 | 0 |
| `UnsafeUnpin` | 97 | 0 |
| `UnwindSafe` | 97 | 0 |
| `Debug` | 30 | 30 |
| `Clone` | 21 | 21 |
| `CloneToUninit` | 21 | 0 |
| `ToOwned` | 21 | 0 |
| `Default` | 19 | 19 |
| `Parse` | 13 | 13 |
| `Eq` | 11 | 11 |
| `PartialEq` | 11 | 11 |
| `StructuralPartialEq` | 11 | 11 |
| `Copy` | 10 | 10 |
| `FromStr` | 2 | 2 |
| `Display` | 1 | 1 |
| `ParsedImplExt` | 1 | 1 |
| `ToString` | 1 | 0 |

## `Debug` — 30 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:139 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:47 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1368 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:619 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `ParsedMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:426 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:458 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:509 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:531 |
| `ParsedImpl` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:702 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:783 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `TraitConst` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:296 |
| `MethodInfo` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_trait.rs:846 |
| `LowerCall` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:61 |
| `LowerFun` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:70 |
| `LowerArg` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:79 |
| `LowerAtom` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:86 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:186 |

## `Clone` — 21 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:139 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:47 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1368 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1675 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:619 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:186 |

## `Default` — 19 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:128 |
| `RFactorAttrs` | `` | concrete | 1 | miniextendr-macros/src/factor_derive.rs:61 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `MatchArgAttrs` | `` | concrete | 1 | miniextendr-macros/src/match_arg_derive.rs:46 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1238 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1675 |
| `PerParamMiniextendrAttr` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:443 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:619 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:458 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:509 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:531 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `RoxygenBuilder` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:643 |
| `RustConversionBuilder` | `` | concrete | 1 | miniextendr-macros/src/rust_conversion_builder.rs:719 |

## `Parse` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ListInput` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:62 |
| `ListEntry` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:77 |
| `RenamePair` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1610 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1710 |
| `MiniextendrFunctionParsed` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:713 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:855 |
| `TpieInput` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:547 |
| `TpieMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:655 |
| `TypedDataframeField` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:108 |
| `TypedDataframeInput` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:56 |
| `ParsedTypeSpec` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:121 |
| `TypedListInput` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:39 |
| `ParsedEntry` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:79 |

## `Eq` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:139 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `PartialEq` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:139 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `StructuralPartialEq` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:139 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1411 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `Copy` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:16 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ReturnPref` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1675 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `FromStr` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ClassSystem` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:310 |
| `VctrsKind` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:341 |

## `Display` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:126 |

## `ParsedImplExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::miniextendr_impl::ParsedImpl` | `` | concrete | 6 | miniextendr-macros/src/r_class_formatter.rs:1066 |
