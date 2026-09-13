# Trait impl inventory

Source: `target/doc/miniextendr_macros.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 106 | 0 |
| `Borrow` | 106 | 0 |
| `BorrowMut` | 106 | 0 |
| `Freeze` | 106 | 0 |
| `From` | 106 | 0 |
| `Into` | 106 | 0 |
| `RefUnwindSafe` | 106 | 0 |
| `Send` | 106 | 0 |
| `Sync` | 106 | 0 |
| `TryFrom` | 106 | 0 |
| `TryInto` | 106 | 0 |
| `Unpin` | 106 | 0 |
| `UnsafeUnpin` | 106 | 0 |
| `UnwindSafe` | 106 | 0 |
| `Debug` | 33 | 33 |
| `Clone` | 24 | 24 |
| `CloneToUninit` | 24 | 0 |
| `ToOwned` | 24 | 0 |
| `Default` | 23 | 23 |
| `Eq` | 14 | 14 |
| `PartialEq` | 14 | 14 |
| `StructuralPartialEq` | 14 | 14 |
| `Parse` | 13 | 13 |
| `Copy` | 11 | 11 |
| `Display` | 2 | 2 |
| `FromStr` | 2 | 2 |
| `ToString` | 2 | 0 |
| `ParsedImplExt` | 1 | 1 |

## `Debug` — 33 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1371 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:622 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `ParsedMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:426 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:463 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:514 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:536 |
| `ParsedImpl` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:712 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:793 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `TraitConst` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:308 |
| `MethodInfo` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_trait.rs:846 |
| `LowerCall` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:61 |
| `LowerFun` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:70 |
| `LowerArg` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:79 |
| `LowerAtom` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:86 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:274 |

## `Clone` — 24 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1371 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1678 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:622 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:274 |

## `Default` — 23 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ContainerAttrs` | `` | concrete | 1 | miniextendr-macros/src/condition_derive.rs:41 |
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/condition_derive.rs:76 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:128 |
| `RDataOptions` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:196 |
| `RFactorAttrs` | `` | concrete | 1 | miniextendr-macros/src/factor_derive.rs:61 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `MatchArgAttrs` | `` | concrete | 1 | miniextendr-macros/src/match_arg_derive.rs:46 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1241 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1678 |
| `PerParamMiniextendrAttr` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:446 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:622 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:463 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:514 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:536 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `RoxygenBuilder` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:643 |
| `RustConversionBuilder` | `` | concrete | 1 | miniextendr-macros/src/rust_conversion_builder.rs:656 |

## `Eq` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `PartialEq` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `StructuralPartialEq` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1414 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `Parse` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ListInput` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:62 |
| `ListEntry` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:77 |
| `RenamePair` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1613 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1713 |
| `MiniextendrFunctionParsed` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:716 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:865 |
| `TpieInput` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:559 |
| `TpieMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:667 |
| `TypedDataframeField` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:108 |
| `TypedDataframeInput` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:56 |
| `ParsedTypeSpec` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:121 |
| `TypedListInput` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:39 |
| `ParsedEntry` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:79 |

## `Copy` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:281 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ReturnPref` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1678 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |

## `Display` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:50 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:126 |

## `FromStr` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ClassSystem` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:310 |
| `VctrsKind` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:341 |

## `ParsedImplExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::miniextendr_impl::ParsedImpl` | `` | concrete | 6 | miniextendr-macros/src/r_class_formatter.rs:1066 |
