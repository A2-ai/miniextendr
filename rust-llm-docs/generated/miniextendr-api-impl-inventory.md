# Trait impl inventory

Source: `target/doc/miniextendr_api.json`

Traits with impls: 190

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `TryFromSexp` | 434 | 434 |
| `IntoR` | 325 | 325 |
| `From` | 255 | 49 |
| `BorrowMut` | 201 | 1 |
| `Borrow` | 201 | 1 |
| `Tap` | 200 | 0 |
| `TryInto` | 200 | 0 |
| `Same` | 200 | 0 |
| `Pointable` | 200 | 0 |
| `Freeze` | 200 | 0 |
| `Into` | 200 | 0 |
| `Unpin` | 200 | 0 |
| `Pipe` | 200 | 0 |
| `FmtForward` | 200 | 0 |
| `Any` | 200 | 0 |
| `UnsafeUnpin` | 200 | 0 |
| `SupersetOf` | 200 | 0 |
| `Conv` | 200 | 0 |
| `Send` | 200 | 7 |
| `UnwindSafe` | 200 | 0 |
| `Sync` | 200 | 10 |
| `IntoEither` | 200 | 0 |
| `TryConv` | 200 | 0 |
| `TryFrom` | 200 | 0 |
| `RefUnwindSafe` | 200 | 0 |
| `TypedExternal` | 198 | 198 |
| `Allocation` | 137 | 0 |
| `IntoRAs` | 135 | 135 |
| `Equivalent` | 111 | 0 |
| `TryCoerce` | 95 | 93 |
| `RDebug` | 93 | 1 |
| `Debug` | 92 | 92 |
| `RClone` | 87 | 1 |
| `ToOwned` | 86 | 0 |
| `CloneToUninit` | 86 | 0 |
| `Clone` | 86 | 86 |
| `AltVec` | 64 | 64 |
| `InferBase` | 64 | 64 |
| `AltrepLen` | 64 | 64 |
| `Altrep` | 64 | 64 |
| `Coerce` | 53 | 53 |
| `RCopy` | 49 | 1 |
| `Copy` | 48 | 48 |
| `PartialEq` | 39 | 39 |
| `Scalar` | 38 | 0 |
| `StructuralPartialEq` | 38 | 38 |
| `Eq` | 37 | 37 |
| `RegisterAltrep` | 33 | 33 |
| `AltrepDataptr` | 27 | 27 |
| `AltrepSerialize` | 27 | 27 |
| `AltrepExtract` | 22 | 1 |
| `RDisplay` | 22 | 1 |
| `ToString` | 21 | 0 |
| `Display` | 21 | 21 |
| `Deref` | 20 | 20 |
| `Receiver` | 20 | 0 |
| `RDefault` | 20 | 1 |
| `Error` | 20 | 20 |
| `RError` | 19 | 1 |
| `Default` | 19 | 19 |
| `AltIntegerData` | 16 | 16 |
| `AltRealData` | 16 | 16 |
| `AltReal` | 16 | 16 |
| `AltInteger` | 16 | 16 |
| `RHash` | 15 | 1 |
| `Hash` | 14 | 14 |
| `Drop` | 12 | 12 |
| `TraitView` | 11 | 11 |
| `AltString` | 10 | 10 |
| `AltStringData` | 10 | 10 |
| `AtomicElement` | 9 | 9 |
| `DerefMut` | 8 | 8 |
| `AltRaw` | 8 | 8 |
| `AltRawData` | 8 | 8 |
| `IntoIterator` | 7 | 2 |
| `AltLogicalData` | 7 | 7 |
| `AltLogical` | 7 | 7 |
| `WidensToF64` | 7 | 7 |
| `AsRef` | 6 | 6 |
| `Parsable` | 6 | 0 |
| `AltComplexData` | 6 | 6 |
| `Formattable` | 6 | 0 |
| `AltComplex` | 6 | 6 |
| `RNdArrayOps` | 6 | 6 |
| `IteratorRandom` | 5 | 0 |
| `ROrd` | 5 | 1 |
| `Iterator` | 5 | 5 |
| `RPartialOrd` | 5 | 1 |
| `ExactSizeIterator` | 5 | 5 |
| `IntoList` | 5 | 5 |
| `RNativeType` | 5 | 5 |
| `TryFromList` | 5 | 5 |
| `Rng` | 4 | 0 |
| `IntoRecords` | 4 | 0 |
| `TryRng` | 4 | 1 |
| `RngCore` | 4 | 0 |
| `TryRngCore` | 4 | 0 |
| `PartialOrd` | 4 | 4 |
| `WidensToI32` | 4 | 4 |
| `Ord` | 4 | 4 |
| `RngExt` | 4 | 0 |
| `Comparable` | 4 | 0 |
| `TryCryptoRng` | 3 | 0 |
| `AsNamedListExt` | 3 | 3 |
| `IntoRAltrep` | 3 | 1 |
| `AsNamedVectorExt` | 3 | 3 |
| `CryptoRng` | 3 | 0 |
| `AsRNativeExt` | 3 | 1 |
| `ParallelBridge` | 2 | 0 |
| `RNdSlice2D` | 2 | 2 |
| `RSourced` | 2 | 2 |
| `FromDataFrame` | 2 | 2 |
| `ThreadLocalArenaOps` | 2 | 2 |
| `AsMut` | 2 | 2 |
| `Storage` | 2 | 2 |
| `AsDataFrameExt` | 2 | 1 |
| `RNdIndex` | 2 | 2 |
| `IntoDataFrame` | 2 | 2 |
| `RDateTimeFormat` | 2 | 2 |
| `Protector` | 2 | 2 |
| `AltrepClass` | 2 | 2 |
| `RNdSlice` | 2 | 2 |
| `CheckedBitPattern` | 1 | 0 |
| `RDistributions` | 1 | 1 |
| `IsContiguous` | 1 | 1 |
| `RTime` | 1 | 1 |
| `RUuidOps` | 1 | 1 |
| `RSerializeNative` | 1 | 1 |
| `RFromIter` | 1 | 1 |
| `UnitEnumFactor` | 1 | 1 |
| `RSignedDuration` | 1 | 1 |
| `RTimestamp` | 1 | 1 |
| `Zeroable` | 1 | 1 |
| `RComplexOps` | 1 | 1 |
| `IntoRVecElement` | 1 | 1 |
| `Pointer` | 1 | 1 |
| `RVectorOps` | 1 | 1 |
| `RNum` | 1 | 1 |
| `BitAnd` | 1 | 1 |
| `StorageMut` | 1 | 0 |
| `RSpan` | 1 | 1 |
| `DoubleEndedIterator` | 1 | 1 |
| `Not` | 1 | 1 |
| `RBigUintBitOps` | 1 | 1 |
| `AltListData` | 1 | 1 |
| `AltrepSexpExt` | 1 | 1 |
| `RawStorage` | 1 | 1 |
| `RCaptureGroups` | 1 | 1 |
| `RBorshOps` | 1 | 1 |
| `AnyBitPattern` | 1 | 0 |
| `AsListExt` | 1 | 1 |
| `RBigUintOps` | 1 | 1 |
| `AsExternalPtrExt` | 1 | 1 |
| `NoUninit` | 1 | 0 |
| `GlobalAlloc` | 1 | 1 |
| `Pod` | 1 | 1 |
| `RToVec` | 1 | 1 |
| `RDecimalOps` | 1 | 1 |
| `ParCollectR` | 1 | 1 |
| `RBigIntBitOps` | 1 | 1 |
| `RRegexOps` | 1 | 1 |
| `RJsonValueOps` | 1 | 1 |
| `RJsonBridge` | 1 | 1 |
| `RDuration` | 1 | 1 |
| `RSigned` | 1 | 1 |
| `BitXor` | 1 | 1 |
| `RBigIntOps` | 1 | 1 |
| `RFloat` | 1 | 1 |
| `ROrderedFloatOps` | 1 | 1 |
| `RSerialize` | 1 | 1 |
| `RDate` | 1 | 1 |
| `RDeserialize` | 1 | 1 |
| `RMatrixOps` | 1 | 1 |
| `AsVctrsExt` | 1 | 1 |
| `RIndexMapOps` | 1 | 1 |
| `RDeserializeNative` | 1 | 1 |
| `RUrlOps` | 1 | 1 |
| `RTomlOps` | 1 | 1 |
| `RZoned` | 1 | 1 |
| `BitOr` | 1 | 1 |
| `RFromStr` | 1 | 1 |
| `Deserializer` | 1 | 1 |
| `SexpExt` | 1 | 1 |
| `FusedIterator` | 1 | 1 |
| `AltList` | 1 | 1 |
| `RAhoCorasickOps` | 1 | 1 |
| `Serializer` | 1 | 1 |
| `RawStorageMut` | 1 | 1 |
| `RDateTime` | 1 | 1 |
| `MatchArg` | 1 | 1 |

## `TryFromSexp` — 434 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AltrepSexp` | `` | concrete | 3 | miniextendr-api/src/altrep_sexp.rs:282 |
| `AsFromStrVec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1030 |
| `AsFromStr<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:988 |
| `DataFrame` | `` | concrete | 2 | miniextendr-api/src/dataframe.rs:555 |
| `Factor<'a>` | `<'a>` | concrete | 2 | miniextendr-api/src/factor.rs:222 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:517 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:570 |
| `crate::coerce::Coerced<T, R>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/from_r.rs:1014 |
| `Vec<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1153 |
| `Vec<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1154 |
| `Vec<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1155 |
| `Vec<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1156 |
| `Vec<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1157 |
| `Vec<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1158 |
| `Vec<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1159 |
| `Vec<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1160 |
| `Vec<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1161 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1164 |
| `std::collections::HashSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1220 |
| `std::collections::HashSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1221 |
| `std::collections::HashSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1222 |
| `std::collections::HashSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1223 |
| `std::collections::HashSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1224 |
| `std::collections::HashSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1225 |
| `std::collections::HashSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1226 |
| `std::collections::HashSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1227 |
| `std::collections::BTreeSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1229 |
| `std::collections::BTreeSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1230 |
| `std::collections::BTreeSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1231 |
| `std::collections::BTreeSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1232 |
| `std::collections::BTreeSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1233 |
| `std::collections::BTreeSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1234 |
| `std::collections::BTreeSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1235 |
| `std::collections::BTreeSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1236 |
| `std::collections::HashSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1255 |
| `std::collections::BTreeSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1256 |
| `crate::externalptr::ExternalPtr<T>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1278 |
| `Option<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1325 |
| `Vec<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1354 |
| `Vec<Option<crate::externalptr::ExternalPtr<T>>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1399 |
| `Box<[T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:337 |
| `i32` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:426 |
| `f64` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:502 |
| `u8` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:503 |
| `crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:504 |
| `crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:505 |
| `crate::SEXP` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:531 |
| `Option<crate::SEXP>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:551 |
| `&[T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:586 |
| `&mut [T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:624 |
| `Option<&[T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:660 |
| `Option<&mut [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:687 |
| `Result<T, ()>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:718 |
| `[T; N]` | `<T, N> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:755 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:793 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:818 |
| `Option<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:844 |
| `Option<std::collections::HashMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:895 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:899 |
| `Option<std::collections::HashSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:928 |
| `Option<std::collections::BTreeSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:932 |
| `Vec<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:943 |
| `i8` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:269 |
| `i16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:283 |
| `u16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:297 |
| `u32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:311 |
| `f32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:325 |
| `Option<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:389 |
| `Option<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:403 |
| `Option<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:417 |
| `Option<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:431 |
| `Option<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:445 |
| `i64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:570 |
| `u64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:588 |
| `Option<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:603 |
| `Option<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:617 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/from_r/coerced_scalars.rs:631 |
| `Option<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:679 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/from_r/coerced_scalars.rs:745 |
| `Option<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:788 |
| `Vec<std::collections::HashMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:144 |
| `Vec<std::collections::BTreeMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:148 |
| `std::collections::HashSet<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:194 |
| `std::collections::HashSet<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:195 |
| `std::collections::HashSet<crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:196 |
| `std::collections::BTreeSet<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:197 |
| `std::collections::BTreeSet<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:198 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:213 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:214 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:215 |
| `Vec<crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:216 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:217 |
| `std::collections::HashMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:38 |
| `std::collections::BTreeMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:45 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:116 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:164 |
| `Vec<&'static str>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:197 |
| `Vec<Option<&'static str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:231 |
| `std::collections::HashSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:278 |
| `std::collections::BTreeSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:282 |
| `std::borrow::Cow<'static, [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:34 |
| `Option<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `Vec<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `Option<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `std::borrow::Cow<'static, str>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:60 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:84 |
| `Option<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:118 |
| `Option<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:142 |
| `Option<i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:164 |
| `Option<f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:196 |
| `Option<u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:228 |
| `Option<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:250 |
| `crate::Rboolean` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:32 |
| `Option<crate::Rboolean>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:62 |
| `bool` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:92 |
| `Vec<crate::Rboolean>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:101 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:131 |
| `Vec<crate::altrep_data::Logical>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:162 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:185 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:210 |
| `Vec<Option<u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:241 |
| `Vec<Option<i8>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:314 |
| `Vec<Option<i16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:315 |
| `Vec<Option<u16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:316 |
| `Vec<Option<u32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:317 |
| `Vec<Option<i64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:318 |
| `Vec<Option<u64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:319 |
| `Vec<Option<isize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:320 |
| `Vec<Option<usize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:321 |
| `Vec<Option<f32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:322 |
| `Vec<Option<f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:62 |
| `Vec<Option<i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:63 |
| `Vec<Option<bool>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:66 |
| `Option<&'static i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `&'static mut i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `&'static i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `Option<&'static mut i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&'static i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&'static i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&'static mut i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&'static mut i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&'static [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&'static [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&'static mut [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&'static mut [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Option<&'static f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `&'static mut f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `&'static f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `Option<&'static mut f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&'static f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&'static f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&'static mut f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&'static mut f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&'static [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&'static [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&'static mut [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&'static mut [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `&'static mut u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `&'static u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&'static mut u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&'static u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&'static u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&'static mut u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&'static mut u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&'static [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&'static [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&'static mut [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&'static mut [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&'static u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&'static mut crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&'static crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&'static crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&'static mut crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&'static mut crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&'static [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&'static [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&'static mut [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&'static mut [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Option<&'static crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&'static mut crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&'static crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&'static crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&'static mut crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&'static crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&'static crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&'static mut crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&'static mut crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&'static [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&'static [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&'static mut [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&'static mut [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `&'static mut crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&'static crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&'static str>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:121 |
| `char` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:199 |
| `String` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:245 |
| `Option<String>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:317 |
| `&'static str` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:47 |
| `List` | `` | concrete | 2 | miniextendr-api/src/list.rs:1202 |
| `Option<List>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1257 |
| `Option<ListMut>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1269 |
| `ListMut` | `` | concrete | 2 | miniextendr-api/src/list.rs:1281 |
| `NamedList` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:165 |
| `Option<NamedList>` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:175 |
| `Missing<T>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/missing.rs:253 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:328 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:343 |
| `Vec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:120 |
| `Option<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:145 |
| `Vec<Option<T>>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:165 |
| `Option<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:102 |
| `Vec<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:103 |
| `Vec<Option<AhoCorasick>>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:104 |
| `AhoCorasick` | `` | concrete | 2 | miniextendr-api/src/optionals/aho_corasick_impl.rs:60 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1062 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1130 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1311 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1351 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1364 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1379 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:487 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:534 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:578 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:616 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:650 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:712 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:765 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:176 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:220 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:265 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:309 |
| `BitVec<u8, Msb0>` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:142 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:172 |
| `RBitVec` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:65 |
| `Option<RBitVec>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:95 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:56 |
| `Bytes` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:381 |
| `BytesMut` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:415 |
| `Option<Bytes>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:430 |
| `Option<BytesMut>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:451 |
| `Either<L, R>` | `<L, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:96 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:62 |
| `Zoned` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:129 |
| `Option<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:209 |
| `Vec<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:265 |
| `Vec<Option<Zoned>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:372 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `SignedDuration` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Vec<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Option<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Option<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Option<Timestamp>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `Vec<Option<Date>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:994 |
| `log::LevelFilter` | `` | concrete | 2 | miniextendr-api/src/optionals/log_impl.rs:319 |
| `RDVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1307 |
| `RDMatrix<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1320 |
| `SMatrix<T, R, C>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:246 |
| `Option<SMatrix<T, R, C>>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:312 |
| `DVector<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:78 |
| `DVector<crate::coerce::Coerced<T, R>>` | `<T, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:854 |
| `DMatrix<crate::coerce::Coerced<T, R>>` | `<T, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:885 |
| `DMatrix<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:96 |
| `Array0<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:119 |
| `Array1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:140 |
| `Array2<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:155 |
| `Array3<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:176 |
| `Array4<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:197 |
| `Array5<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:230 |
| `Array6<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:265 |
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:2920 |
| `ArrayD<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:300 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3065 |
| `Array4<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `ArrayD<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array3<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array6<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array2<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array5<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array1<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array3<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array6<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array2<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array5<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array1<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array4<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `ArrayD<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array2<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array5<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array1<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array4<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `ArrayD<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array3<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array6<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array2<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array5<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array1<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array4<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `ArrayD<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array3<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array6<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array1<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array4<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `ArrayD<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array3<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array6<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array2<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array5<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array4<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `ArrayD<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array3<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array6<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array2<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array5<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array1<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array3<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array6<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array2<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array5<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array1<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array4<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `ArrayD<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array2<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array5<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array1<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array4<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `ArrayD<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array3<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array6<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array2<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array5<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array1<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array4<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `ArrayD<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array3<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array6<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:676 |
| `Array1<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `Array4<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `ArrayD<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `Array3<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `Array6<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `Array2<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `Array5<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:680 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:202 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:214 |
| `Option<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:226 |
| `Option<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:238 |
| `Vec<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:250 |
| `Vec<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:267 |
| `Vec<Option<BigInt>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:284 |
| `Vec<Option<BigUint>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:299 |
| `Option<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:129 |
| `Vec<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:171 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:218 |
| `Complex<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:92 |
| `OrderedFloat<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:128 |
| `OrderedFloat<f32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:136 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:144 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:153 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:164 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:182 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:203 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:212 |
| `Vec<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:102 |
| `Vec<Option<Regex>>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:119 |
| `Regex` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:68 |
| `Option<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:85 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:146 |
| `Option<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:179 |
| `Vec<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:206 |
| `Vec<Option<Decimal>>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:251 |
| `JsonValue` | `` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:578 |
| `Option<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:590 |
| `Vec<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:591 |
| `Vec<Option<JsonValue>>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:592 |
| `OffsetDateTime` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Option<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<Date>>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:102 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:178 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:227 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:305 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:339 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:78 |
| `TomlValue` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:122 |
| `Option<TomlValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/toml_impl.rs:167 |
| `Vec<TomlValue>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:170 |
| `Vec<Option<TomlValue>>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:193 |
| `Vec<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:135 |
| `Vec<Option<Url>>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:185 |
| `Url` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:51 |
| `Option<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:89 |
| `Vec<Option<Uuid>>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:127 |
| `Uuid` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:44 |
| `Option<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:58 |
| `Vec<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:77 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:671 |
| `RArray<i8, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:778 |
| `RArray<i16, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:779 |
| `RArray<i64, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:780 |
| `RArray<isize, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:781 |
| `RArray<u16, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:782 |
| `RArray<u32, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:783 |
| `RArray<u64, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:784 |
| `RArray<usize, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:785 |
| `RArray<f32, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:788 |
| `RArray<bool, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:791 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:497 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:508 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:519 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:555 |
| `RCow<'static, T>` | `<T>` | concrete | 3 | miniextendr-api/src/rcow.rs:167 |
| `FromJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:97 |
| `StrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:428 |
| `ProtectedStrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:668 |

### `TryFromSexp` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/from_r/references.rs:449** (12 impls): `Option<&'static f64>`, `&'static mut f64`, `&'static f64`, `Option<&'static mut f64>`, `Vec<&'static f64>`, `Vec<Option<&'static f64>>`, `Vec<&'static mut f64>`, `Vec<Option<&'static mut f64>>`, `Vec<&'static [f64]>`, `Vec<Option<&'static [f64]>>`, `Vec<&'static mut [f64]>`, `Vec<Option<&'static mut [f64]>>`
- **miniextendr-api/src/from_r/references.rs:451** (12 impls): `Option<&'static mut crate::RLogical>`, `Vec<&'static crate::RLogical>`, `Vec<Option<&'static crate::RLogical>>`, `Vec<&'static mut crate::RLogical>`, `Vec<Option<&'static mut crate::RLogical>>`, `Vec<&'static [crate::RLogical]>`, `Vec<Option<&'static [crate::RLogical]>>`, `Vec<&'static mut [crate::RLogical]>`, `Vec<Option<&'static mut [crate::RLogical]>>`, `Option<&'static crate::RLogical>`, `&'static mut crate::RLogical`, `&'static crate::RLogical`
- **miniextendr-api/src/from_r/references.rs:452** (12 impls): `&'static crate::Rcomplex`, `Option<&'static mut crate::Rcomplex>`, `Vec<&'static crate::Rcomplex>`, `Vec<Option<&'static crate::Rcomplex>>`, `Vec<&'static mut crate::Rcomplex>`, `Vec<Option<&'static mut crate::Rcomplex>>`, `Vec<&'static [crate::Rcomplex]>`, `Vec<Option<&'static [crate::Rcomplex]>>`, `Vec<&'static mut [crate::Rcomplex]>`, `Vec<Option<&'static mut [crate::Rcomplex]>>`, `&'static mut crate::Rcomplex`, `Option<&'static crate::Rcomplex>`
- **miniextendr-api/src/from_r/references.rs:448** (12 impls): `Option<&'static i32>`, `&'static mut i32`, `&'static i32`, `Option<&'static mut i32>`, `Vec<&'static i32>`, `Vec<Option<&'static i32>>`, `Vec<&'static mut i32>`, `Vec<Option<&'static mut i32>>`, `Vec<&'static [i32]>`, `Vec<Option<&'static [i32]>>`, `Vec<&'static mut [i32]>`, `Vec<Option<&'static mut [i32]>>`
- **miniextendr-api/src/from_r/references.rs:450** (12 impls): `&'static mut u8`, `&'static u8`, `Option<&'static mut u8>`, `Vec<&'static u8>`, `Vec<Option<&'static u8>>`, `Vec<&'static mut u8>`, `Vec<Option<&'static mut u8>>`, `Vec<&'static [u8]>`, `Vec<Option<&'static [u8]>>`, `Vec<&'static mut [u8]>`, `Vec<Option<&'static mut [u8]>>`, `Option<&'static u8>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:670** (7 impls): `Array4<u32>`, `ArrayD<u32>`, `Array3<u32>`, `Array6<u32>`, `Array2<u32>`, `Array5<u32>`, `Array1<u32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:676** (7 impls): `Array2<f32>`, `Array5<f32>`, `Array1<f32>`, `Array4<f32>`, `ArrayD<f32>`, `Array3<f32>`, `Array6<f32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:665** (7 impls): `Array4<i8>`, `ArrayD<i8>`, `Array3<i8>`, `Array6<i8>`, `Array2<i8>`, `Array5<i8>`, `Array1<i8>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:668** (7 impls): `Array2<isize>`, `Array5<isize>`, `Array1<isize>`, `Array4<isize>`, `ArrayD<isize>`, `Array3<isize>`, `Array6<isize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:671** (7 impls): `Array3<u64>`, `Array6<u64>`, `Array2<u64>`, `Array5<u64>`, `Array1<u64>`, `Array4<u64>`, `ArrayD<u64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:680** (7 impls): `Array1<bool>`, `Array4<bool>`, `ArrayD<bool>`, `Array3<bool>`, `Array6<bool>`, `Array2<bool>`, `Array5<bool>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:666** (7 impls): `Array3<i16>`, `Array6<i16>`, `Array2<i16>`, `Array5<i16>`, `Array1<i16>`, `Array4<i16>`, `ArrayD<i16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:669** (7 impls): `Array1<u16>`, `Array4<u16>`, `ArrayD<u16>`, `Array3<u16>`, `Array6<u16>`, `Array2<u16>`, `Array5<u16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:672** (7 impls): `Array2<usize>`, `Array5<usize>`, `Array1<usize>`, `Array4<usize>`, `ArrayD<usize>`, `Array3<usize>`, `Array6<usize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:667** (7 impls): `Array2<i64>`, `Array5<i64>`, `Array1<i64>`, `Array4<i64>`, `ArrayD<i64>`, `Array3<i64>`, `Array6<i64>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:392** (4 impls): `Vec<Option<std::ffi::OsString>>`, `std::ffi::OsString`, `Vec<std::ffi::OsString>`, `Option<std::ffi::OsString>`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Vec<Option<Date>>`, `Date`, `Vec<Date>`, `Option<Date>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `OffsetDateTime`, `Vec<OffsetDateTime>`, `Option<OffsetDateTime>`, `Vec<Option<OffsetDateTime>>`
- **miniextendr-api/src/optionals/jiff_impl.rs:484** (4 impls): `Vec<Option<SignedDuration>>`, `SignedDuration`, `Vec<SignedDuration>`, `Option<SignedDuration>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Vec<Option<Date>>`, `Date`, `Vec<Date>`, `Option<Date>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:369** (4 impls): `Option<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`, `std::path::PathBuf`, `Vec<std::path::PathBuf>`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Option<Timestamp>`, `Vec<Option<Timestamp>>`, `Timestamp`, `Vec<Timestamp>`
- **miniextendr-api/src/optionals/jiff_impl.rs:882** (2 impls): `JiffTimestampVecRef`, `JiffTimestampVecMut`
- **miniextendr-api/src/optionals/jiff_impl.rs:923** (2 impls): `JiffZonedVecMut`, `JiffZonedVecRef`

## `IntoR` — 325 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsList<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:109 |
| `Collect<I>` | `<I, T> +2wc` | concrete | 5 | miniextendr-api/src/convert.rs:1115 |
| `CollectStrings<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1171 |
| `CollectNA<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:1212 |
| `CollectNAInt<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:1257 |
| `AsExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:371 |
| `AsRNative<T>` | `<T>` | concrete | 5 | miniextendr-api/src/convert.rs:423 |
| `AsDataFrame<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:493 |
| `AsVctrs<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:541 |
| `AsNamedList<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:591 |
| `AsNamedList<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:612 |
| `AsNamedList<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:633 |
| `AsNamedVector<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:686 |
| `AsNamedVector<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:702 |
| `AsNamedVector<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:718 |
| `AsDisplay<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:927 |
| `AsDisplayVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:954 |
| `DataFrame` | `` | concrete | 4 | miniextendr-api/src/dataframe.rs:563 |
| `FactorVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:501 |
| `FactorOptionVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:610 |
| `HashSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1008 |
| `BTreeSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1008 |
| `HashSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1009 |
| `BTreeSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1009 |
| `BTreeSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1010 |
| `HashSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1010 |
| `Option<Vec<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1019 |
| `Option<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1046 |
| `Option<std::collections::HashMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1073 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1100 |
| `Option<std::collections::HashSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1127 |
| `Option<std::collections::BTreeSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1154 |
| `Option<std::collections::HashSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1211 |
| `Option<std::collections::BTreeSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1215 |
| `Vec<String>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1260 |
| `&[String]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1278 |
| `&[&str]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1296 |
| `crate::SEXP` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:130 |
| `Vec<std::borrow::Cow<'_, str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1314 |
| `Vec<Option<std::borrow::Cow<'_, str>>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1333 |
| `Vec<Option<&str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1386 |
| `Vec<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1437 |
| `Vec<Vec<T>>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:1458 |
| `Vec<&[T]>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1506 |
| `Vec<&[String]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1546 |
| `Vec<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1584 |
| `Vec<Option<f64>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1671 |
| `Vec<Option<i32>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1672 |
| `()` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:169 |
| `Vec<Option<i64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1710 |
| `Vec<Option<u64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1711 |
| `Vec<Option<isize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1712 |
| `Vec<Option<usize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1714 |
| `Vec<Option<i8>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1733 |
| `Vec<Option<i16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1734 |
| `Vec<Option<u16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1735 |
| `Vec<Option<u32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1736 |
| `Vec<Option<f32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1737 |
| `Vec<bool>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1770 |
| `&[bool]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1790 |
| `Vec<Option<bool>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1833 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1842 |
| `std::convert::Infallible` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:185 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1850 |
| `Vec<Option<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1862 |
| `(A, B)` | `<A, B>` | concrete | 5 | miniextendr-api/src/into_r.rs:1974 |
| `(A, B, C)` | `<A, B, C>` | concrete | 5 | miniextendr-api/src/into_r.rs:1975 |
| `(A, B, C, D)` | `<A, B, C, D>` | concrete | 5 | miniextendr-api/src/into_r.rs:1976 |
| `(A, B, C, D, E)` | `<A, B, C, D, E>` | concrete | 5 | miniextendr-api/src/into_r.rs:1977 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F>` | concrete | 5 | miniextendr-api/src/into_r.rs:1978 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G>` | concrete | 5 | miniextendr-api/src/into_r.rs:1979 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H>` | concrete | 5 | miniextendr-api/src/into_r.rs:1980 |
| `Vec<Box<[T]>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2083 |
| `Vec<Box<[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2113 |
| `Vec<[T; N]>` | `<T, N> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2130 |
| `Vec<Option<Vec<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2198 |
| `Vec<Option<Vec<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2215 |
| `Vec<Option<std::collections::HashSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2220 |
| `Vec<Option<std::collections::HashSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2237 |
| `Vec<Option<std::collections::BTreeSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2242 |
| `Vec<Option<std::collections::BTreeSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2259 |
| `i32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:226 |
| `Vec<Option<std::collections::HashMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2264 |
| `f64` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:227 |
| `Vec<Option<std::collections::BTreeMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2278 |
| `u8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:228 |
| `crate::Rcomplex` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:229 |
| `Vec<Option<&[T]>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r.rs:2294 |
| `Vec<Option<&[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2309 |
| `Vec<std::collections::HashSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2317 |
| `Vec<std::collections::BTreeSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2336 |
| `Vec<std::collections::HashSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2354 |
| `Vec<std::collections::BTreeSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2360 |
| `Vec<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2383 |
| `Vec<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2387 |
| `Option<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:272 |
| `i8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:332 |
| `i16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:333 |
| `u16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:334 |
| `f32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:337 |
| `u32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:338 |
| `Vec<i32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:380 |
| `Vec<f64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:381 |
| `Vec<u8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:382 |
| `Vec<crate::RLogical>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:383 |
| `Vec<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:384 |
| `&[T]` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:386 |
| `Box<[T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:416 |
| `Vec<i8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:577 |
| `&[i8]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:577 |
| `Vec<i16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:578 |
| `&[i16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:578 |
| `Vec<u16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:579 |
| `&[u16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:579 |
| `Vec<f32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:582 |
| `&[f32]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:582 |
| `Vec<i64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:640 |
| `Vec<u64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:641 |
| `Vec<isize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:642 |
| `Vec<usize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:644 |
| `[T; N]` | `<T, N>` | concrete | 5 | miniextendr-api/src/into_r.rs:660 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:686 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:716 |
| `std::borrow::Cow<'_, [T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:752 |
| `std::borrow::Cow<'_, str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:772 |
| `Option<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:939 |
| `Vec<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:939 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:939 |
| `std::path::PathBuf` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:939 |
| `&std::path::Path` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:939 |
| `Vec<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:955 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:955 |
| `std::ffi::OsString` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:955 |
| `&std::ffi::OsStr` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:955 |
| `Option<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:955 |
| `Altrep<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/altrep.rs:152 |
| `crate::altrep_sexp::AltrepSexp` | `` | concrete | 4 | miniextendr-api/src/into_r/altrep.rs:192 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:116 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:138 |
| `std::collections::HashSet<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:182 |
| `std::collections::BTreeSet<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:186 |
| `std::collections::HashMap<String, V>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:44 |
| `std::collections::BTreeMap<String, V>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:48 |
| `isize` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:117 |
| `usize` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:141 |
| `bool` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:186 |
| `crate::Rboolean` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:187 |
| `crate::RLogical` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:188 |
| `Option<i32>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:190 |
| `Option<f64>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:216 |
| `Option<crate::Rboolean>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:242 |
| `Option<crate::RLogical>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:269 |
| `Option<bool>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:295 |
| `Option<i64>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:348 |
| `Option<u64>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:349 |
| `Option<isize>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:350 |
| `Option<usize>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:352 |
| `Option<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:375 |
| `Option<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:376 |
| `Option<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:377 |
| `Option<u32>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:378 |
| `Option<f32>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:379 |
| `crate::externalptr::ExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:381 |
| `Vec<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:404 |
| `Vec<Option<crate::externalptr::ExternalPtr<T>>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:419 |
| `T` | `<T>` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:493 |
| `i64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:50 |
| `String` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:538 |
| `char` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:558 |
| `&str` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:583 |
| `Option<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:608 |
| `Option<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:645 |
| `Option<&T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:672 |
| `u64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:85 |
| `Result<T, NullOnErr>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/result.rs:126 |
| `Result<T, E>` | `<T, E> +2wc` | concrete | 4 | miniextendr-api/src/into_r/result.rs:69 |
| `List` | `` | concrete | 4 | miniextendr-api/src/list.rs:1071 |
| `ListMut` | `` | concrete | 4 | miniextendr-api/src/list.rs:1085 |
| `Vec<List>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1104 |
| `Vec<Option<List>>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1133 |
| `NamedList` | `` | concrete | 4 | miniextendr-api/src/list/named.rs:151 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:287 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:306 |
| `Vec<Option<T>>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/newtype.rs:203 |
| `Vec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/newtype.rs:97 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1155 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1191 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1224 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1253 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1277 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1345 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1346 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1347 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1392 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1486 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1537 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:855 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:902 |
| `TimestampSecondArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:934 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:356 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:375 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:390 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:406 |
| `RBitVec` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:100 |
| `Option<RBitVec>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:123 |
| `BitVec<u8, Msb0>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:174 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:197 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:38 |
| `Bytes` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:362 |
| `BytesMut` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:396 |
| `Option<Bytes>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:472 |
| `Option<BytesMut>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:496 |
| `Either<L, R>` | `<L, R> +2wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:145 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:127 |
| `Zoned` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:180 |
| `Option<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:238 |
| `Vec<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:316 |
| `Vec<Option<Zoned>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:422 |
| `Option<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Vec<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `SignedDuration` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:484 |
| `Vec<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Option<Timestamp>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Option<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `log::LevelFilter` | `` | concrete | 4 | miniextendr-api/src/optionals/log_impl.rs:332 |
| `RDVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1334 |
| `RDMatrix<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1347 |
| `DVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:137 |
| `DMatrix<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:157 |
| `SMatrix<T, R, C>` | `<T, R, C>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:286 |
| `Option<SMatrix<T, R, C>>` | `<T, R, C>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:335 |
| `DVector<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:924 |
| `DMatrix<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:946 |
| `ArcArray2<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1009 |
| `ArrayView1<'a, T>` | `<'a, T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1030 |
| `ArrayView2<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1048 |
| `ArrayView3<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1079 |
| `ArrayViewD<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1122 |
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:2932 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3077 |
| `Array0<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:325 |
| `Array1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:342 |
| `Array2<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:689 |
| `Array3<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:731 |
| `Array4<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:778 |
| `Array5<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:823 |
| `Array6<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `ArrayD<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:910 |
| `ArcArray1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:992 |
| `BigInt` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:314 |
| `BigUint` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:315 |
| `Option<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:316 |
| `Option<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:319 |
| `Vec<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:322 |
| `Vec<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:327 |
| `Vec<Option<BigInt>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:332 |
| `Vec<Option<BigUint>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:337 |
| `Complex<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:124 |
| `Option<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:163 |
| `Vec<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:201 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:247 |
| `OrderedFloat<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:227 |
| `OrderedFloat<f32>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:228 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:229 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:232 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:235 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:240 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:245 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:250 |
| `Decimal` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:287 |
| `Option<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:288 |
| `Vec<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:291 |
| `Vec<Option<Decimal>>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:296 |
| `JsonValue` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:597 |
| `Option<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:610 |
| `Vec<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:626 |
| `Vec<Option<JsonValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:653 |
| `Table` | `` | concrete | 4 | miniextendr-api/src/optionals/tabled_impl.rs:196 |
| `Option<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `OffsetDateTime` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:140 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:159 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:202 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:251 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:380 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:402 |
| `TomlValue` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:214 |
| `Option<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:236 |
| `Vec<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:252 |
| `Vec<Option<TomlValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:279 |
| `Option<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:128 |
| `Vec<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:176 |
| `Vec<Option<Url>>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:224 |
| `Url` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:84 |
| `Vec<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:118 |
| `Vec<Option<Uuid>>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:145 |
| `Uuid` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:53 |
| `Option<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:72 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 5 | miniextendr-api/src/rarray.rs:796 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:408 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:422 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:436 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:465 |
| `RCow<'_, T>` | `<T>` | concrete | 5 | miniextendr-api/src/rcow.rs:185 |
| `DataFrameShape` | `` | concrete | 3 | miniextendr-api/src/serde/columnar.rs:3280 |
| `AsJsonVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:131 |
| `AsJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:34 |
| `AsJsonPretty<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:58 |
| `AsSerialize<T>` | `<T>` | concrete | 2 | miniextendr-api/src/serde/traits.rs:262 |
| `StrVec` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:414 |
| `ProtectedStrVec` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:654 |

### `IntoR` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r.rs:955** (5 impls): `Vec<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `std::ffi::OsString`, `&std::ffi::OsStr`, `Option<std::ffi::OsString>`
- **miniextendr-api/src/into_r.rs:939** (5 impls): `Option<std::path::PathBuf>`, `Vec<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`, `std::path::PathBuf`, `&std::path::Path`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Date`, `Vec<Option<Date>>`, `Option<Date>`, `Vec<Date>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `Option<OffsetDateTime>`, `Vec<OffsetDateTime>`, `OffsetDateTime`, `Vec<Option<OffsetDateTime>>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Date`, `Vec<Option<Date>>`, `Option<Date>`, `Vec<Date>`
- **miniextendr-api/src/optionals/jiff_impl.rs:484** (4 impls): `Option<SignedDuration>`, `Vec<SignedDuration>`, `SignedDuration`, `Vec<Option<SignedDuration>>`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Vec<Timestamp>`, `Timestamp`, `Vec<Option<Timestamp>>`, `Option<Timestamp>`
- **miniextendr-api/src/into_r.rs:577** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r.rs:1009** (2 impls): `HashSet<i16>`, `BTreeSet<i16>`
- **miniextendr-api/src/into_r.rs:578** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r.rs:1010** (2 impls): `BTreeSet<u16>`, `HashSet<u16>`
- **miniextendr-api/src/into_r.rs:579** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r.rs:582** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r.rs:1008** (2 impls): `HashSet<i8>`, `BTreeSet<i8>`

## `From` — 49 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:107 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:114 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:121 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:168 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:182 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:85 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:96 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:543 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:103 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:365 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:417 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:487 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:534 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:585 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:680 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:103 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1670 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1677 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:482 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:540 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:272 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:278 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:284 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:125 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1196 |
| `crate::from_r::SexpError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:101 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:221 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:228 |
| `Option<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:238 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:201 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:126 |
| `ArrayView1<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1453 |
| `ArrayView2<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1479 |
| `ArrayView3<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1491 |
| `ArrayViewD<'a, T>` | `<'a, T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1503 |
| `Array1<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1533 |
| `Array2<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1544 |
| `Array3<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1556 |
| `ArrayD<T>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1569 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:158 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:164 |
| `RCow<'_, T>` | `<T>` | concrete | 1 | miniextendr-api/src/rcow.rs:157 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:255 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:477 |
| `*mut SEXPREC` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:484 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:202 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:330 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:339 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:265 |

## `BorrowMut` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1553 |

## `Borrow` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1546 |

## `Send` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:406 |
| `WorkerUnprotectGuard` | `` | concrete | 0 | miniextendr-api/src/gc_protect.rs:1383 |
| `TraitDispatchEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:327 |
| `AltrepRegistration` | `` | concrete | 0 | miniextendr-api/src/registry.rs:344 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:70 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1153 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:197 |

## `Sync` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RWrapperEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:236 |
| `MatchArgChoicesEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:253 |
| `MatchArgParamDocEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:270 |
| `ClassNameEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:288 |
| `SidecarPropEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:306 |
| `TraitDispatchEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:326 |
| `AltrepRegistration` | `` | concrete | 0 | miniextendr-api/src/registry.rs:343 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:71 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1152 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:198 |

## `TypedExternal` — 198 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:250 |
| `IterIntFromBoolData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:391 |
| `IterStringData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:531 |
| `IterListData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:653 |
| `IterComplexData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:767 |
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:94 |
| `SparseIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:230 |
| `SparseIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:331 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:430 |
| `SparseIterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:532 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:648 |
| `IterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:276 |
| `IterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:376 |
| `IterLogicalData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:475 |
| `IterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:575 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:274 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:380 |
| `StreamingRealData<F>` | `<F>` | concrete | 3 | miniextendr-api/src/altrep_data/stream.rs:107 |
| `StreamingIntData<F>` | `<F>` | concrete | 3 | miniextendr-api/src/altrep_data/stream.rs:267 |
| `()` | `` | concrete | 3 | miniextendr-api/src/externalptr.rs:329 |
| `std::borrow::Cow<'static, [T]>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:101 |
| `std::rc::Rc<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:107 |
| `std::sync::Arc<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:108 |
| `std::cell::Cell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:109 |
| `std::cell::RefCell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:110 |
| `std::cell::UnsafeCell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:111 |
| `std::sync::Mutex<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:112 |
| `std::sync::RwLock<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:113 |
| `std::sync::OnceLock<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:114 |
| `std::pin::Pin<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:115 |
| `std::mem::ManuallyDrop<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:119 |
| `std::mem::MaybeUninit<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:124 |
| `std::marker::PhantomData<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:125 |
| `Option<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:130 |
| `Result<T, E>` | `<T, E>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:131 |
| `std::ops::Range<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:136 |
| `std::ops::RangeInclusive<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:137 |
| `std::ops::RangeFrom<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:138 |
| `std::ops::RangeTo<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:139 |
| `std::ops::RangeToInclusive<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:140 |
| `std::ops::RangeFull` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:141 |
| `std::fs::File` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:146 |
| `std::io::BufReader<R>` | `<R>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:147 |
| `std::io::BufWriter<W>` | `<W>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:148 |
| `std::io::Cursor<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:149 |
| `std::time::Duration` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:154 |
| `std::time::Instant` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:155 |
| `std::time::SystemTime` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:156 |
| `std::net::TcpStream` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:161 |
| `std::net::TcpListener` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:162 |
| `std::net::UdpSocket` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:163 |
| `std::net::IpAddr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:164 |
| `std::net::Ipv4Addr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:165 |
| `std::net::Ipv6Addr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:166 |
| `std::net::SocketAddr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:167 |
| `std::net::SocketAddrV4` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:168 |
| `std::net::SocketAddrV6` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:169 |
| `std::thread::Thread` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:174 |
| `std::thread::JoinHandle<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:175 |
| `std::sync::mpsc::Sender<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:176 |
| `std::sync::mpsc::SyncSender<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:177 |
| `std::sync::mpsc::Receiver<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:178 |
| `std::sync::Barrier` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:179 |
| `std::sync::BarrierWaitResult` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:180 |
| `std::sync::atomic::AtomicBool` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:185 |
| `std::sync::atomic::AtomicI8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:186 |
| `std::sync::atomic::AtomicI16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:187 |
| `std::sync::atomic::AtomicI32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:188 |
| `std::sync::atomic::AtomicI64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:189 |
| `std::sync::atomic::AtomicIsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:190 |
| `std::sync::atomic::AtomicU8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:191 |
| `std::sync::atomic::AtomicU16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:192 |
| `std::sync::atomic::AtomicU32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:193 |
| `std::sync::atomic::AtomicU64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:194 |
| `std::sync::atomic::AtomicUsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:195 |
| `std::num::NonZeroI8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:201 |
| `std::num::NonZeroI16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:202 |
| `std::num::NonZeroI32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:203 |
| `std::num::NonZeroI64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:204 |
| `std::num::NonZeroI128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:205 |
| `std::num::NonZeroIsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:206 |
| `std::num::NonZeroU8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:207 |
| `std::num::NonZeroU16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:208 |
| `std::num::NonZeroU32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:209 |
| `std::num::NonZeroU64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:210 |
| `std::num::NonZeroU128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:211 |
| `std::num::NonZeroUsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:212 |
| `std::num::Wrapping<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:213 |
| `std::num::Saturating<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:214 |
| `(A)` | `<A>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:219 |
| `(A, B)` | `<A, B>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:220 |
| `(A, B, C)` | `<A, B, C>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:221 |
| `(A, B, C, D)` | `<A, B, C, D>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:222 |
| `(A, B, C, D, E)` | `<A, B, C, D, E>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:223 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:224 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:225 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:226 |
| `(A, B, C, D, E, F, G, H, I)` | `<A, B, C, D, E, F, G, H, I>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:227 |
| `(A, B, C, D, E, F, G, H, I, J)` | `<A, B, C, D, E, F, G, H, I, J>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:228 |
| `(A, B, C, D, E, F, G, H, I, J, K)` | `<A, B, C, D, E, F, G, H, I, J, K>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:229 |
| `(A, B, C, D, E, F, G, H, I, J, K, L)` | `<A, B, C, D, E, F, G, H, I, J, K, L>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:230 |
| `[T; N]` | `<T, N>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:235 |
| `&'static [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:255 |
| `&'static mut [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:261 |
| `bool` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:48 |
| `char` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:49 |
| `i8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:50 |
| `i16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:51 |
| `i32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:52 |
| `i64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:53 |
| `i128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:54 |
| `isize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:55 |
| `u8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:56 |
| `u16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:57 |
| `u32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:58 |
| `u64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:59 |
| `u128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:60 |
| `usize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:61 |
| `f32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:62 |
| `f64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:63 |
| `String` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:68 |
| `std::ffi::CString` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:69 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:70 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:71 |
| `Vec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:76 |
| `std::collections::VecDeque<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:77 |
| `std::collections::LinkedList<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:78 |
| `std::collections::BinaryHeap<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:79 |
| `std::collections::HashMap<K, V>` | `<K, V>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:80 |
| `std::collections::BTreeMap<K, V>` | `<K, V>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:81 |
| `std::collections::HashSet<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:82 |
| `std::collections::BTreeSet<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:83 |
| `Box<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:88 |
| `Box<[T]>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:94 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1565 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1571 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1577 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1583 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1589 |
| `JiffTimestampVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:385 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:386 |
| `DVector<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:387 |
| `DMatrix<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:390 |
| `DMatrix<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:391 |
| `DMatrix<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:392 |
| `SVector<f64, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:396 |
| `SVector<f64, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:397 |
| `SVector<f64, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:398 |
| `SVector<i32, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:399 |
| `SVector<i32, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:400 |
| `SVector<i32, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:401 |
| `SMatrix<f64, 2, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:405 |
| `SMatrix<f64, 3, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:406 |
| `SMatrix<f64, 4, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:407 |
| `SMatrix<i32, 2, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:408 |
| `SMatrix<i32, 3, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:409 |
| `SMatrix<i32, 4, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:410 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1610 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1611 |
| `Array1<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1612 |
| `Array1<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1613 |
| `Array1<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1614 |
| `Array2<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1617 |
| `Array2<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1618 |
| `Array2<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1619 |
| `Array2<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1620 |
| `Array2<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1621 |
| `Array3<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1624 |
| `Array3<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1625 |
| `Array3<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1626 |
| `Array3<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1627 |
| `Array3<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1628 |
| `Array4<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1631 |
| `Array4<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1632 |
| `Array4<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1633 |
| `Array4<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1634 |
| `Array4<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1635 |
| `Array5<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1638 |
| `Array5<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1639 |
| `Array5<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1640 |
| `Array5<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1641 |
| `Array5<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1642 |
| `Array6<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1645 |
| `Array6<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1646 |
| `Array6<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1647 |
| `Array6<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1648 |
| `Array6<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1649 |
| `ArrayD<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1652 |
| `ArrayD<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1653 |
| `ArrayD<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1654 |
| `ArrayD<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1655 |
| `ArrayD<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1656 |
| `ArcArray1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1659 |
| `ArcArray1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1660 |
| `ArcArray2<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1661 |
| `ArcArray2<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1662 |

## `IntoRAs` — 135 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:324 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:332 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:333 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:334 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:335 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:338 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:339 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:340 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:341 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:342 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:345 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:346 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:349 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:360 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:374 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:388 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:395 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:402 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:418 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:425 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:432 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:440 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:441 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:442 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:443 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:446 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:457 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:465 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:466 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:467 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:468 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:469 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:470 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:471 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:472 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:473 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:474 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:475 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:480 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:487 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:495 |
| `String` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:514 |
| `&str` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:521 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:529 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:547 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:554 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:561 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:568 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:575 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:619 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:626 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:633 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:633 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:634 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:634 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:635 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:635 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:636 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:636 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:637 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:637 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:638 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:638 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:639 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:639 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:640 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:640 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:641 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:641 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:642 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:642 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:643 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:643 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:646 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:653 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:664 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:678 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:693 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:709 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:744 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:744 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:745 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:745 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:746 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:746 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:747 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:747 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:748 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:748 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:753 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:769 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:786 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:786 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:787 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:787 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:788 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:788 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:789 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:789 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:792 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:802 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:813 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:820 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:827 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:827 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:828 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:828 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:829 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:829 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:830 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:830 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:831 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:831 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:832 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:832 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:833 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:833 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:834 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:834 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:835 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:835 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:836 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:836 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:837 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:837 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:842 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:849 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:859 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:866 |
| `Vec<&str>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:873 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:880 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:888 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:910 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:917 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:924 |

### `IntoRAs` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r_as.rs:641** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:828** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:642** (2 impls): `&[f32]`, `Vec<f32>`
- **miniextendr-api/src/into_r_as.rs:829** (2 impls): `&[i32]`, `Vec<i32>`
- **miniextendr-api/src/into_r_as.rs:831** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:832** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r_as.rs:744** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r_as.rs:834** (2 impls): `Vec<u64>`, `&[u64]`
- **miniextendr-api/src/into_r_as.rs:745** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:835** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:747** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:837** (2 impls): `Vec<f64>`, `&[f64]`
- **miniextendr-api/src/into_r_as.rs:748** (2 impls): `&[u32]`, `Vec<u32>`
- **miniextendr-api/src/into_r_as.rs:634** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:786** (2 impls): `Vec<i64>`, `&[i64]`
- **miniextendr-api/src/into_r_as.rs:635** (2 impls): `&[u8]`, `Vec<u8>`
- **miniextendr-api/src/into_r_as.rs:787** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:637** (2 impls): `Vec<i64>`, `&[i64]`
- **miniextendr-api/src/into_r_as.rs:789** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:638** (2 impls): `&[isize]`, `Vec<isize>`
- **miniextendr-api/src/into_r_as.rs:640** (2 impls): `Vec<u64>`, `&[u64]`
- **miniextendr-api/src/into_r_as.rs:827** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r_as.rs:643** (2 impls): `Vec<f64>`, `&[f64]`
- **miniextendr-api/src/into_r_as.rs:830** (2 impls): `Vec<i64>`, `&[i64]`
- **miniextendr-api/src/into_r_as.rs:833** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:746** (2 impls): `Vec<u8>`, `&[u8]`
- **miniextendr-api/src/into_r_as.rs:836** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r_as.rs:633** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r_as.rs:636** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:788** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:639** (2 impls): `Vec<u32>`, `&[u32]`

## `TryCoerce` — 93 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T, R> +1wc` | concrete | 2 | miniextendr-api/src/coerce.rs:112 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:318 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:328 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:338 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:369 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:370 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:371 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:372 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:373 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:374 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:375 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:376 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:377 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:378 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:381 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:391 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:401 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:412 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:423 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:434 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:445 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:456 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:467 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:500 |
| `crate::Rboolean` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:515 |
| `crate::RLogical` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:524 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:566 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:587 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:608 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:632 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:653 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:673 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:694 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:706 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:746 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:768 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:789 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:811 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:852 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:869 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:882 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:890 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:119 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:130 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:141 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:152 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:163 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:57 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:87 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:40 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:81 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:104 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:119 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:74 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:91 |

### `TryCoerce` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/coerce.rs:561** (9 impls): `usize`, `isize`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- **miniextendr-api/src/coerce.rs:541** (9 impls): `i8`, `i16`, `i32`, `i64`, `u16`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:551** (8 impls): `i8`, `i16`, `i32`, `i64`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:556** (7 impls): `i32`, `i64`, `u16`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:536** (5 impls): `u32`, `u64`, `usize`, `i64`, `isize`

## `RDebug` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:63 |

## `Debug` — 92 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `RAllocator` | `` | concrete | 1 | miniextendr-api/src/allocator.rs:142 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepSexp` | `` | concrete | 1 | miniextendr-api/src/altrep_sexp.rs:300 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:566 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:100 |
| `AsFromStrVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1027 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:414 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:484 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:530 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:582 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:677 |
| `AsDisplay<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:924 |
| `AsDisplayVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:951 |
| `AsFromStr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:985 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:47 |
| `DataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:898 |
| `Dots` | `` | concrete | 1 | miniextendr-api/src/dots.rs:42 |
| `REncodingInfo` | `` | concrete | 1 | miniextendr-api/src/encoding.rs:23 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1429 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1584 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:467 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:525 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:151 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:172 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:193 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:208 |
| `Protected<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1048 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:119 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:58 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:14 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1162 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1177 |
| `List` | `` | concrete | 1 | miniextendr-api/src/list.rs:40 |
| `ListMut` | `` | concrete | 1 | miniextendr-api/src/list.rs:47 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:56 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:172 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:256 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:132 |
| `RRng` | `` | concrete | 1 | miniextendr-api/src/optionals/rand_impl.rs:108 |
| `CaptureGroups` | `` | concrete | 1 | miniextendr-api/src/optionals/regex_impl.rs:253 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:101 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:880 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1037 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:924 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:12 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXPREC` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:25 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `StrVec` | `` | concrete | 1 | miniextendr-api/src/strvec.rs:17 |
| `ProtectedStrVec` | `` | concrete | 1 | miniextendr-api/src/strvec.rs:686 |
| `DllInfo` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1085 |
| `R_CMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1113 |
| `R_CallMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1138 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:967 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:191 |
| `TypedList` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:277 |
| `TypedListSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:47 |
| `TypedEntry` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:75 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:526 |

## `RClone` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:373 |

## `Clone` — 86 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:100 |
| `AsFromStrVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1027 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:414 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:484 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:530 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:582 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:677 |
| `AsDisplay<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:924 |
| `AsDisplayVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:951 |
| `AsFromStr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:985 |
| `DataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:134 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:47 |
| `REncodingInfo` | `` | concrete | 1 | miniextendr-api/src/encoding.rs:23 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1429 |
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1560 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:467 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:525 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:151 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:172 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:193 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:208 |
| `Root<'a>` | `<'a>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:815 |
| `TlsRoot` | `` | concrete | 1 | miniextendr-api/src/gc_protect/tls.rs:202 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:119 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:58 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:14 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1162 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1177 |
| `List` | `` | concrete | 1 | miniextendr-api/src/list.rs:40 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:56 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `CaptureGroups` | `` | concrete | 1 | miniextendr-api/src/optionals/regex_impl.rs:253 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:101 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:120 |
| `RawHeader` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:115 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1037 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:924 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:12 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `cetype_t` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:353 |
| `StrVec` | `` | concrete | 1 | miniextendr-api/src/strvec.rs:17 |
| `R_CMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1113 |
| `R_CallMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1138 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:967 |
| `R_altrep_class_t` | `` | concrete | 1 | miniextendr-api/src/sys/altrep.rs:187 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:191 |
| `TypedList` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:277 |
| `TypedListSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:47 |
| `TypedEntry` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:75 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:526 |

## `AltVec` — 64 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:138 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:294 |
| `IterIntFromBoolData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:425 |
| `IterStringData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:566 |
| `IterListData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:685 |
| `IterComplexData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/coerce.rs:803 |
| `SparseIterIntData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/sparse.rs:262 |
| `SparseIterRealData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/sparse.rs:365 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/sparse.rs:464 |
| `SparseIterRawData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/sparse.rs:564 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/sparse.rs:684 |
| `IterIntData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/state.rs:308 |
| `IterRealData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/state.rs:408 |
| `IterLogicalData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/state.rs:507 |
| `IterRawData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/state.rs:607 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/windowed.rs:308 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 0 | miniextendr-api/src/altrep_data/iter/windowed.rs:414 |
| `StreamingRealData<F>` | `<F>` | concrete | 0 | miniextendr-api/src/altrep_data/stream.rs:143 |
| `StreamingIntData<F>` | `<F>` | concrete | 0 | miniextendr-api/src/altrep_data/stream.rs:303 |
| `[i32; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 0 | miniextendr-api/src/altrep_impl/arrays.rs:160 |
| `[String; N]` | `<N>` | concrete | 0 | miniextendr-api/src/altrep_impl/arrays.rs:218 |
| `Vec<i32>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [f64]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:140 |
| `&'static [bool]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:243 |
| `&'static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:296 |
| `&'static [i32]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:31 |
| `&'static [String]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:361 |
| `&'static [&'static str]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:401 |
| `Float64Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 4 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/ndarray_impl.rs:3572 |
| `Array1<i32>` | `` | concrete | 4 | miniextendr-api/src/optionals/ndarray_impl.rs:3573 |

## `InferBase` — 64 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:104 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:260 |
| `IterIntFromBoolData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:399 |
| `IterStringData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:537 |
| `IterListData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:659 |
| `IterComplexData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:775 |
| `SparseIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:236 |
| `SparseIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:339 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:438 |
| `SparseIterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:538 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:656 |
| `IterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:282 |
| `IterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:382 |
| `IterLogicalData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:481 |
| `IterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:581 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:282 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:388 |
| `StreamingRealData<F>` | `<F>` | concrete | 3 | miniextendr-api/src/altrep_data/stream.rs:115 |
| `StreamingIntData<F>` | `<F>` | concrete | 3 | miniextendr-api/src/altrep_data/stream.rs:275 |
| `[i32; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:182 |
| `[String; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:231 |
| `Vec<i32>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [i32]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:129 |
| `&'static [f64]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:231 |
| `&'static [bool]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:285 |
| `&'static [u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:346 |
| `&'static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:386 |
| `&'static [&'static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:426 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3572 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3573 |

## `AltrepLen` — 64 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1001 |
| `[f64; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1026 |
| `[bool; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1051 |
| `[u8; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1063 |
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1084 |
| `Vec<crate::Rcomplex>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1095 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1102 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1109 |
| `Cow<'static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1186 |
| `Cow<'static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1191 |
| `Cow<'static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1196 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1201 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:495 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:499 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:503 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:507 |
| `Vec<Option<String>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:519 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:531 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:547 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:563 |
| `Box<[i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:580 |
| `Box<[f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:584 |
| `Box<[u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:588 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:592 |
| `Box<[String]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:595 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:663 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:760 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:901 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:955 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:958 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:961 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:964 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:967 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:975 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:218 |
| `IterIntFromBoolData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:361 |
| `IterStringData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:510 |
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:61 |
| `IterListData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:634 |
| `IterComplexData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:738 |
| `SparseIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:207 |
| `SparseIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:311 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:411 |
| `SparseIterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:512 |
| `SparseIterComplexData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:619 |
| `IterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:254 |
| `IterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:356 |
| `IterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:456 |
| `IterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:555 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:252 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:360 |
| `StreamingIntData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:237 |
| `StreamingRealData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:77 |
| `Float64Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1599 |
| `Int32Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1605 |
| `UInt8Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1611 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1617 |
| `StringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1949 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:898 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:961 |
| `DVector<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1613 |
| `DVector<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1619 |
| `Array1<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3487 |
| `Array1<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3493 |

## `Altrep` — 64 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:127 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:283 |
| `IterIntFromBoolData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:418 |
| `IterStringData<I>` | `<I>` | concrete | 2 | miniextendr-api/src/altrep_data/iter/coerce.rs:556 |
| `IterListData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:678 |
| `IterComplexData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:794 |
| `SparseIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:255 |
| `SparseIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:358 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:457 |
| `SparseIterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:557 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:675 |
| `IterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:301 |
| `IterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:401 |
| `IterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:500 |
| `IterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:600 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:301 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:407 |
| `StreamingRealData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:134 |
| `StreamingIntData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:294 |
| `[i32; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:152 |
| `[String; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:208 |
| `Vec<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [f64]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:132 |
| `&'static [i32]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:23 |
| `&'static [bool]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:234 |
| `&'static [u8]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:288 |
| `&'static [String]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:349 |
| `&'static [&'static str]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:389 |
| `Float64Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3572 |
| `Array1<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3573 |

## `Coerce` — 53 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `tinyvec::TinyVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1016 |
| `tinyvec::ArrayVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1031 |
| `(A, B)` | `<A, B, RA, RB> +2wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1061 |
| `(A, B, C)` | `<A, B, C, RA, RB, RC> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1062 |
| `(A, B, C, D)` | `<A, B, C, D, RA, RB, RC, RD> +4wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1063 |
| `(A, B, C, D, E)` | `<A, B, C, D, E, RA, RB, RC, RD, RE> +5wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1064 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F, RA, RB, RC, RD, RE, RF> +6wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1065 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G, RA, RB, RC, RD, RE, RF, RG> +7wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1070 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H, RA, RB, RC, RD, RE, RF, RG, RH> +8wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1075 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:142 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:143 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:144 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:145 |
| `crate::Rcomplex` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:146 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:191 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:202 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:214 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:224 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:235 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:242 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:249 |
| `Option<f64>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:260 |
| `Option<i32>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:268 |
| `Option<bool>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:276 |
| `Option<crate::Rboolean>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:288 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:302 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:310 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:663 |
| `&[T]` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:974 |
| `Vec<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:982 |
| `Box<[T]>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:990 |
| `std::collections::VecDeque<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:998 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:25 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:33 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:41 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:49 |
| `OrderedFloat<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:100 |
| `OrderedFloat<f32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:108 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:24 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:32 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:73 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:58 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:66 |

### `Coerce` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/coerce.rs:212** (5 impls): `u8`, `u8`, `u8`, `u8`, `u8`
- **miniextendr-api/src/coerce.rs:546** (5 impls): `u8`, `u8`, `i8`, `u16`, `u8`

## `RCopy` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:467 |

## `Copy` — 48 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `AsList<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:100 |
| `AsExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:414 |
| `AsDisplay<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:924 |
| `DataFrame` | `` | concrete | 0 | miniextendr-api/src/dataframe.rs:134 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:151 |
| `SexpLengthError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:172 |
| `SexpNaError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:193 |
| `Root<'a>` | `<'a>` | concrete | 0 | miniextendr-api/src/gc_protect.rs:815 |
| `TlsRoot` | `` | concrete | 0 | miniextendr-api/src/gc_protect/tls.rs:202 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:119 |
| `List` | `` | concrete | 0 | miniextendr-api/src/list.rs:40 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 0 | miniextendr-api/src/rarray.rs:120 |
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `cetype_t` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:353 |
| `StrVec` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:17 |
| `R_CMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1113 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1138 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:967 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:187 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:526 |

## `PartialEq` — 39 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1602 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:119 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:58 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:924 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:967 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:526 |

## `StructuralPartialEq` — 38 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:119 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:58 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 0 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:924 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:967 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/typed_list.rs:119 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:526 |

## `Eq` — 37 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:1609 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:119 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:58 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 0 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:924 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:64 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1538 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:967 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:526 |

## `RegisterAltrep` — 33 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:554 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:555 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:556 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:557 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:558 |
| `Vec<Option<String>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:559 |
| `Vec<crate::Rcomplex>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:560 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:563 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:564 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:565 |
| `Box<[i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:568 |
| `Box<[f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:569 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:570 |
| `Box<[u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:571 |
| `Box<[String]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:572 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:573 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:576 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:577 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:578 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:579 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:582 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:583 |
| `Float64Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1871 |
| `Int32Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1890 |
| `UInt8Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1909 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1928 |
| `StringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1971 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1678 |
| `DVector<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1695 |
| `Array1<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3577 |
| `Array1<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3594 |

## `AltrepDataptr` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1097 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1104 |
| `Cow<'static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1188 |
| `Cow<'static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1193 |
| `Cow<'static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1198 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1203 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:497 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:501 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:505 |
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:582 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:586 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:590 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Float64Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1708 |
| `Int32Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1728 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1746 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1653 |
| `DVector<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1663 |
| `Array1<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3536 |
| `Array1<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3554 |

## `AltrepSerialize` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cow<'static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1189 |
| `Cow<'static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1194 |
| `Cow<'static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1199 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1204 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:390 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:391 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:392 |
| `Vec<bool>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:393 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:394 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:395 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:396 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:399 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:400 |
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:408 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:422 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:436 |
| `Box<[bool]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:450 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:478 |
| `std::ops::Range<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:611 |
| `std::ops::Range<i64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:624 |
| `std::ops::Range<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:647 |
| `Float64Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1781 |
| `Int32Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1804 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1827 |
| `BooleanArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1846 |
| `StringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1855 |

## `AltrepExtract` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/altrep_data/core.rs:336 |

## `RDisplay` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:102 |

## `Display` — 21 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:489 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:97 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:550 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:74 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1444 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1590 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:160 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:181 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:200 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:235 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:111 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:30 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1168 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1185 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:73 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:138 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:138 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:84 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:80 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:230 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:78 |

## `Deref` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 2 | miniextendr-api/src/coerce.rs:954 |
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1516 |
| `Factor<'_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:213 |
| `FactorMut<'_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:309 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:488 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:546 |
| `Protected<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:1039 |
| `Root<'a>` | `<'a>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:837 |
| `OwnedProtect` | `` | concrete | 2 | miniextendr-api/src/gc_protect.rs:925 |
| `TlsRoot` | `` | concrete | 2 | miniextendr-api/src/gc_protect/tls.rs:221 |
| `Altrep<T>` | `<T>` | concrete | 2 | miniextendr-api/src/into_r/altrep.rs:132 |
| `RPrimitive<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:150 |
| `RStringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:234 |
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:117 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `RCow<'_, T>` | `<T>` | concrete | 2 | miniextendr-api/src/rcow.rs:146 |
| `ArenaGuard<'_, M>` | `<M>` | concrete | 2 | miniextendr-api/src/refcount_protect.rs:788 |

### `Deref` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/optionals/jiff_impl.rs:923** (2 impls): `JiffZonedVecMut`, `JiffZonedVecRef`
- **miniextendr-api/src/optionals/jiff_impl.rs:882** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`

## `RDefault` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:412 |

## `Error` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:108 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:498 |
| `DataFrameError` | `` | concrete | 0 | miniextendr-api/src/dataframe.rs:100 |
| `TypeMismatchError` | `` | concrete | 0 | miniextendr-api/src/externalptr.rs:1460 |
| `SexpTypeError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:170 |
| `SexpLengthError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:191 |
| `SexpNaError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:206 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:257 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:175 |
| `IntoRError` | `` | concrete | 0 | miniextendr-api/src/into_r_error.rs:49 |
| `DuplicateNameError` | `` | concrete | 0 | miniextendr-api/src/list.rs:1174 |
| `ListFromSexpError` | `` | concrete | 0 | miniextendr-api/src/list.rs:1194 |
| `MatchArgError` | `` | concrete | 0 | miniextendr-api/src/match_arg.rs:99 |
| `RCoerceError` | `` | concrete | 0 | miniextendr-api/src/r_coerce.rs:155 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:107 |
| `RSerdeError` | `` | concrete | 0 | miniextendr-api/src/serde/error.rs:120 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:68 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:74 |
| `TypedListError` | `` | concrete | 0 | miniextendr-api/src/typed_list.rs:263 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:123 |

## `RError` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/adapter_traits.rs:275 |

## `Default` — 19 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1576 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `ProtectScope` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:789 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:213 |
| `RRng` | `` | concrete | 1 | miniextendr-api/src/optionals/rand_impl.rs:108 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `WorkerPump<T>` | `<T>` | concrete | 1 | miniextendr-api/src/pump.rs:181 |
| `Arena<M>` | `<M>` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:719 |
| `RngGuard` | `` | concrete | 1 | miniextendr-api/src/rng.rs:160 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1043 |
| `NamedDataFrameListBuilder` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:265 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:470 |
| `RThreadBuilder` | `` | concrete | 1 | miniextendr-api/src/thread.rs:304 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:526 |

## `AltIntegerData` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_data/builtins.rs:1003 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1187 |
| `Vec<i32>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:496 |
| `Box<[i32]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:581 |
| `std::ops::Range<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:673 |
| `std::ops::Range<i64>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:770 |
| `&[i32]` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:956 |
| `IterIntFromBoolData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:370 |
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:71 |
| `SparseIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:213 |
| `IterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:260 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:258 |
| `StreamingIntData<F>` | `<F>` | concrete | 2 | miniextendr-api/src/altrep_data/stream.rs:243 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1652 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1639 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3518 |

## `AltRealData` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[f64; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_data/builtins.rs:1028 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1192 |
| `Vec<f64>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:500 |
| `Box<[f64]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:585 |
| `std::ops::Range<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:912 |
| `&[f64]` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:959 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:228 |
| `SparseIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:317 |
| `IterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:362 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:366 |
| `StreamingRealData<F>` | `<F>` | concrete | 2 | miniextendr-api/src/altrep_data/stream.rs:83 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1627 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:904 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:967 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1625 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3499 |

## `AltReal` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 4 | miniextendr-api/src/altrep_data/iter/coerce.rs:301 |
| `SparseIterRealData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:367 |
| `IterRealData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/state.rs:410 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/windowed.rs:416 |
| `StreamingRealData<F>` | `<F>` | concrete | 4 | miniextendr-api/src/altrep_data/stream.rs:148 |
| `[f64; N]` | `<N>` | concrete | 6 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `Vec<f64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `std::ops::Range<f64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[f64]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `&'static [f64]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:162 |
| `Float64Array` | `` | concrete | 14 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `JiffTimestampVec` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:923 |
| `DVector<f64>` | `` | concrete | 14 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `Array1<f64>` | `` | concrete | 14 | miniextendr-api/src/optionals/ndarray_impl.rs:3572 |

## `AltInteger` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 4 | miniextendr-api/src/altrep_data/iter/coerce.rs:145 |
| `IterIntFromBoolData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/coerce.rs:427 |
| `SparseIterIntData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:264 |
| `IterIntData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/state.rs:310 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/windowed.rs:310 |
| `StreamingIntData<F>` | `<F>` | concrete | 4 | miniextendr-api/src/altrep_data/stream.rs:308 |
| `[i32; N]` | `<N>` | concrete | 6 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `Vec<i32>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `std::ops::Range<i32>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `Box<[i32]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `&'static [i32]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:54 |
| `Int32Array` | `` | concrete | 14 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `DVector<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/ndarray_impl.rs:3573 |

## `RHash` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:137 |

## `Hash` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1625 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1494 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1518 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1538 |

## `Drop` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1693 |
| `ExternalSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1862 |
| `WorkerUnprotectGuard` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1372 |
| `ProtectScope` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:776 |
| `OwnedProtect` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:916 |
| `RVecStorage<T, R, C>` | `<T, R, C>` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1183 |
| `RndVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:2914 |
| `RndMat<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3059 |
| `ProtectPool` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:301 |
| `Arena<M>` | `<M>` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:709 |
| `ArenaGuard<'_, M>` | `<M>` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:782 |
| `RngGuard` | `` | concrete | 1 | miniextendr-api/src/rng.rs:166 |

## `TraitView` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RHashView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:131 |
| `ROrdView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:163 |
| `RPartialOrdView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:203 |
| `RErrorView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:263 |
| `RFromStrView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:327 |
| `RCloneView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:367 |
| `RDefaultView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:406 |
| `RCopyView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:453 |
| `RIteratorView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:539 |
| `RDebugView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:54 |
| `RDisplayView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:96 |

## `AltString` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterStringData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:568 |
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:220 |
| `Vec<String>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Box<[String]>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `&'static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:363 |
| `&'static [&'static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:403 |
| `StringArray` | `` | concrete | 5 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |

## `AltStringData` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1086 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:509 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:521 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:537 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:553 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:597 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:969 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:977 |
| `IterStringData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:519 |
| `StringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1955 |

## `AtomicElement` — 9 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `bool` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:106 |
| `String` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:118 |
| `Option<i32>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:130 |
| `Option<f64>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:140 |
| `Option<bool>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:150 |
| `Option<String>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:160 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:47 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:66 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:85 |

## `DerefMut` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:963 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1525 |
| `FactorMut<'_>` | `` | concrete | 1 | miniextendr-api/src/factor.rs:318 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:495 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:553 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:141 |
| `JiffTimestampVecMut` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVecMut` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:923 |

## `AltRaw` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SparseIterRawData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:566 |
| `IterRawData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/state.rs:609 |
| `[u8; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `Vec<u8>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Box<[u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `&'static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:318 |
| `UInt8Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |

## `AltRawData` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[u8; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1065 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1197 |
| `Vec<u8>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:504 |
| `Box<[u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:589 |
| `&[u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:962 |
| `SparseIterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:518 |
| `IterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:561 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1674 |

## `IntoIterator` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `StrVec` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:295 |
| `&'a ProtectedStrVec` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:644 |

## `AltLogicalData` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[bool; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1053 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:564 |
| `Box<[bool]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:593 |
| `&[bool]` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:965 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 2 | miniextendr-api/src/altrep_data/iter/sparse.rs:417 |
| `IterLogicalData<I>` | `<I>` | concrete | 2 | miniextendr-api/src/altrep_data/iter/state.rs:462 |
| `BooleanArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1688 |

## `AltLogical` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SparseIterLogicalData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:466 |
| `IterLogicalData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/state.rs:509 |
| `[bool; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:162 |
| `Vec<bool>` | `` | concrete | 10 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Box<[bool]>` | `` | concrete | 10 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `&'static [bool]` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/static_slices.rs:245 |
| `BooleanArray` | `` | concrete | 10 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |

## `WidensToF64` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `f32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:219 |
| `i8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:220 |
| `i16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:221 |
| `i32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:222 |
| `u8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:223 |
| `u16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:224 |
| `u32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:225 |

## `AsRef` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1532 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:158 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:165 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:242 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:249 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:241 |

## `AltComplexData` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1096 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1103 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1111 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1202 |
| `IterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:747 |
| `SparseIterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:628 |

## `AltComplex` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterComplexData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/coerce.rs:808 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:689 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `Vec<crate::Rcomplex>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |

## `RNdArrayOps` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array1<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1728 |
| `Array2<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1786 |
| `ArrayD<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1844 |
| `Array1<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1904 |
| `Array2<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1969 |
| `ArrayD<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2034 |

## `ROrd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:174 |

## `Iterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/externalptr.rs:1632 |
| `StrVecIter` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:234 |
| `StrVecCowIter` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:269 |
| `ProtectedStrVecIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:591 |
| `ProtectedStrVecCowIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:622 |

## `RPartialOrd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:215 |

## `ExactSizeIterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1660 |
| `StrVecIter` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:258 |
| `StrVecCowIter` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:293 |
| `ProtectedStrVecIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:613 |
| `ProtectedStrVecCowIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:642 |

## `IntoList` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/list.rs:671 |
| `std::collections::HashMap<K, V>` | `<K, V> +2wc` | concrete | 1 | miniextendr-api/src/list.rs:722 |
| `std::collections::BTreeMap<K, V>` | `<K, V> +2wc` | concrete | 1 | miniextendr-api/src/list.rs:779 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/list.rs:836 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/list.rs:861 |

## `RNativeType` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i32` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:209 |
| `f64` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:230 |
| `u8` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:251 |
| `RLogical` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:273 |
| `Rcomplex` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:295 |

## `TryFromList` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:692 |
| `std::collections::HashMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:733 |
| `std::collections::BTreeMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:790 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:846 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:871 |

## `TryRng` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RRng` | `` | concrete | 4 | miniextendr-api/src/optionals/rand_impl.rs:128 |

## `PartialOrd` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1611 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |

## `WidensToI32` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:214 |
| `i16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:215 |
| `u8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:216 |
| `u16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:217 |

## `Ord` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1618 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |

## `AsNamedListExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:882 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:883 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:884 |

## `IntoRAltrep` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2065 |

## `AsNamedVectorExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:901 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:902 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:903 |

## `AsRNativeExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:838 |

## `RNdSlice2D` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array2<f64>` | `` | concrete | 7 | miniextendr-api/src/optionals/ndarray_impl.rs:2273 |
| `Array2<i32>` | `` | concrete | 7 | miniextendr-api/src/optionals/ndarray_impl.rs:2321 |

## `RSourced` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RPrimitive<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:141 |
| `RStringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:223 |

## `FromDataFrame` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/dataframe.rs:758 |
| `SerdeRows<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:256 |

## `ThreadLocalArenaOps` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ThreadLocalArena` | `` | concrete | 2 | miniextendr-api/src/refcount_protect.rs:1111 |
| `ThreadLocalHashArena` | `` | concrete | 2 | miniextendr-api/src/refcount_protect.rs:1121 |

## `AsMut` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1539 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:248 |

## `Storage` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::Dyn>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1251 |
| `RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::U1>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1276 |

## `AsDataFrameExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:851 |

## `RNdIndex` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ArrayD<f64>` | `` | concrete | 10 | miniextendr-api/src/optionals/ndarray_impl.rs:2447 |
| `ArrayD<i32>` | `` | concrete | 10 | miniextendr-api/src/optionals/ndarray_impl.rs:2601 |

## `IntoDataFrame` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/dataframe.rs:747 |
| `SerdeRows<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:241 |

## `RDateTimeFormat` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `OffsetDateTime` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:287 |
| `Date` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:299 |

## `Protector` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectScope` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:216 |
| `crate::protect_pool::ProtectPool` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:223 |

## `AltrepClass` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JiffTimestampVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:882 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:923 |

## `RNdSlice` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array1<f64>` | `` | concrete | 5 | miniextendr-api/src/optionals/ndarray_impl.rs:2160 |
| `Array1<i32>` | `` | concrete | 5 | miniextendr-api/src/optionals/ndarray_impl.rs:2193 |

## `RDistributions` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RRng` | `` | concrete | 4 | miniextendr-api/src/optionals/rand_impl.rs:231 |

## `IsContiguous` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 0 | miniextendr-api/src/optionals/nalgebra_impl.rs:1301 |

## `RTime` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Time` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:711 |

## `RUuidOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Uuid` | `` | concrete | 8 | miniextendr-api/src/optionals/uuid_impl.rs:206 |

## `RSerializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:73 |

## `RFromIter` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `C` | `<C, T> +1wc` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:738 |

## `UnitEnumFactor` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/factor.rs:560 |

## `RSignedDuration` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SignedDuration` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:531 |

## `RTimestamp` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Timestamp` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:744 |

## `Zeroable` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |

## `RComplexOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Complex<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/num_complex_impl.rs:329 |

## `IntoRVecElement` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/match_arg.rs:281 |

## `Pointer` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1596 |

## `RVectorOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DVector<f64>` | `` | concrete | 18 | miniextendr-api/src/optionals/nalgebra_impl.rs:495 |

## `RNum` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/optionals/num_traits_impl.rs:78 |

## `BitAnd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:145 |

## `RSpan` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Span` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:595 |

## `DoubleEndedIterator` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1648 |

## `Not` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:166 |

## `RBigUintBitOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigUint` | `` | concrete | 10 | miniextendr-api/src/optionals/num_bigint_impl.rs:809 |

## `AltListData` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterListData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:643 |

## `AltrepSexpExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::SEXP` | `` | concrete | 4 | miniextendr-api/src/altrep_ext.rs:54 |

## `RawStorage` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 7 | miniextendr-api/src/optionals/nalgebra_impl.rs:1191 |

## `RCaptureGroups` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CaptureGroups` | `` | concrete | 5 | miniextendr-api/src/optionals/regex_impl.rs:314 |

## `RBorshOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:91 |

## `AsListExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:793 |

## `RBigUintOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigUint` | `` | concrete | 13 | miniextendr-api/src/optionals/num_bigint_impl.rs:567 |

## `AsExternalPtrExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:817 |

## `GlobalAlloc` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RAllocator` | `` | concrete | 4 | miniextendr-api/src/allocator.rs:145 |

## `Pod` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |

## `RToVec` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `C` | `<C, T> +3wc` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:809 |

## `RDecimalOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Decimal` | `` | concrete | 22 | miniextendr-api/src/optionals/rust_decimal_impl.rs:400 |

## `ParCollectR` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/rayon_bridge.rs:1768 |

## `RBigIntBitOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigInt` | `` | concrete | 11 | miniextendr-api/src/optionals/num_bigint_impl.rs:703 |

## `RRegexOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Regex` | `` | concrete | 7 | miniextendr-api/src/optionals/regex_impl.rs:214 |

## `RJsonValueOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JsonValue` | `` | concrete | 15 | miniextendr-api/src/optionals/serde_impl.rs:937 |

## `RJsonBridge` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:1049 |

## `RDuration` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Duration` | `` | concrete | 10 | miniextendr-api/src/optionals/time_impl.rs:189 |

## `RSigned` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/optionals/num_traits_impl.rs:139 |

## `BitXor` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:159 |

## `RBigIntOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigInt` | `` | concrete | 17 | miniextendr-api/src/optionals/num_bigint_impl.rs:427 |

## `RFloat` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 41 | miniextendr-api/src/optionals/num_traits_impl.rs:354 |

## `ROrderedFloatOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `OrderedFloat<T>` | `<T> +1wc` | concrete | 16 | miniextendr-api/src/optionals/ordered_float_impl.rs:344 |

## `RSerialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:228 |

## `RDate` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Date` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:836 |

## `RDeserialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:275 |

## `RMatrixOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DMatrix<f64>` | `` | concrete | 24 | miniextendr-api/src/optionals/nalgebra_impl.rs:687 |

## `AsVctrsExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:865 |

## `RIndexMapOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IndexMap<String, T>` | `<T>` | concrete | 9 | miniextendr-api/src/optionals/indexmap_impl.rs:218 |

## `RDeserializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:146 |

## `RUrlOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Url` | `` | concrete | 12 | miniextendr-api/src/optionals/url_impl.rs:300 |

## `RTomlOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `TomlValue` | `` | concrete | 16 | miniextendr-api/src/optionals/toml_impl.rs:624 |

## `RZoned` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Zoned` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:782 |

## `BitOr` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:152 |

## `RFromStr` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:336 |

## `Deserializer` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RDeserializer` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/de.rs:89 |

## `SexpExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::SEXP` | `` | concrete | 85 | miniextendr-api/src/sexp_ext.rs:457 |

## `FusedIterator` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:1668 |

## `AltList` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterListData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:687 |

## `RAhoCorasickOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AhoCorasick` | `` | concrete | 6 | miniextendr-api/src/optionals/aho_corasick_impl.rs:305 |

## `Serializer` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RSerializer` | `` | concrete | 37 | miniextendr-api/src/serde/ser.rs:51 |

## `RawStorageMut` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1230 |

## `RDateTime` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DateTime` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:663 |

## `MatchArg` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `log::LevelFilter` | `` | concrete | 3 | miniextendr-api/src/optionals/log_impl.rs:283 |
