# Trait impl inventory

Source: `target/doc/miniextendr_macros.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 108 | 0 |
| `Borrow` | 108 | 0 |
| `BorrowMut` | 108 | 0 |
| `Freeze` | 108 | 0 |
| `From` | 108 | 0 |
| `Into` | 108 | 0 |
| `RefUnwindSafe` | 108 | 0 |
| `Send` | 108 | 0 |
| `Sync` | 108 | 0 |
| `TryFrom` | 108 | 0 |
| `TryInto` | 108 | 0 |
| `Unpin` | 108 | 0 |
| `UnsafeUnpin` | 108 | 0 |
| `UnwindSafe` | 108 | 0 |
| `Debug` | 36 | 36 |
| `Clone` | 27 | 27 |
| `CloneToUninit` | 27 | 0 |
| `ToOwned` | 27 | 0 |
| `Default` | 22 | 22 |
| `Eq` | 16 | 16 |
| `PartialEq` | 16 | 16 |
| `StructuralPartialEq` | 16 | 16 |
| `Copy` | 13 | 13 |
| `Parse` | 13 | 13 |
| `Display` | 2 | 2 |
| `FromStr` | 2 | 2 |
| `ToString` | 2 | 0 |
| `ParsedImplExt` | 1 | 1 |

## `Debug` — 36 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1380 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:628 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `ParsedMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:426 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:465 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:516 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:538 |
| `ParsedImpl` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:718 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:799 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `TraitConst` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:312 |
| `MethodInfo` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_trait.rs:846 |
| `LowerCall` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:61 |
| `LowerFun` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:70 |
| `LowerArg` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:79 |
| `LowerAtom` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:86 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:16 |
| `ReturnWrap` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:22 |
| `Container` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:9 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:307 |

## `Clone` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1380 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1687 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:628 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:16 |
| `ReturnWrap` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:22 |
| `Container` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:9 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:307 |

## `Default` — 22 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ContainerAttrs` | `` | concrete | 1 | miniextendr-macros/src/condition_derive.rs:41 |
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/condition_derive.rs:76 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:128 |
| `RFactorAttrs` | `` | concrete | 1 | miniextendr-macros/src/factor_derive.rs:61 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `MatchArgAttrs` | `` | concrete | 1 | miniextendr-macros/src/match_arg_derive.rs:46 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1247 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1687 |
| `PerParamMiniextendrAttr` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:452 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:628 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:465 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:516 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:538 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `RoxygenBuilder` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:643 |
| `RustConversionBuilder` | `` | concrete | 1 | miniextendr-macros/src/rust_conversion_builder.rs:656 |

## `Eq` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:16 |
| `Container` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:9 |

## `PartialEq` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 1 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:16 |
| `Container` | `` | concrete | 1 | miniextendr-macros/src/return_wrap.rs:9 |

## `StructuralPartialEq` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `CrateConfig` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:34 |
| `CrateConfigError` | `` | concrete | 0 | miniextendr-macros/src/crate_config.rs:44 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1423 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:16 |
| `Container` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:9 |

## `Copy` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ReturnPref` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1687 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:323 |
| `ConversionKind` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:16 |
| `Container` | `` | concrete | 0 | miniextendr-macros/src/return_wrap.rs:9 |

## `Parse` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ListInput` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:62 |
| `ListEntry` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:77 |
| `RenamePair` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1622 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1722 |
| `MiniextendrFunctionParsed` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:722 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:871 |
| `TpieInput` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:563 |
| `TpieMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:671 |
| `TypedDataframeField` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:108 |
| `TypedDataframeInput` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:56 |
| `ParsedTypeSpec` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:121 |
| `TypedListInput` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:39 |
| `ParsedEntry` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:79 |

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
| `crate::miniextendr_impl::ParsedImpl` | `` | concrete | 6 | miniextendr-macros/src/r_class_formatter.rs:1073 |
