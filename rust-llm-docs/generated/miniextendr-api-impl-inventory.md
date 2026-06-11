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
| `Factor<''a>` | `<'a>` | concrete | 2 | miniextendr-api/src/factor.rs:222 |
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
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:116 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:164 |
| `Vec<&''static str>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:197 |
| `Vec<Option<&''static str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:231 |
| `std::collections::HashSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:278 |
| `std::collections::BTreeSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:282 |
| `std::borrow::Cow<''static, [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:34 |
| `Option<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:369 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `Vec<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `Option<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:392 |
| `std::borrow::Cow<''static, str>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:60 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:84 |
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
| `Option<&''static i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `&''static mut i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `&''static i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `Option<&''static mut i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&''static i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&''static i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&''static mut i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&''static mut i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&''static [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&''static [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<&''static mut [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Vec<Option<&''static mut [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:448 |
| `Option<&''static f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `&''static mut f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `&''static f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `Option<&''static mut f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&''static f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&''static f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&''static mut f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&''static mut f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&''static [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&''static [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<&''static mut [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&''static mut [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:449 |
| `&''static mut u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `&''static u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&''static mut u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static mut u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static mut u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static mut [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static mut [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&''static u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&''static mut crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&''static crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&''static crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&''static mut crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&''static mut crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&''static [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&''static [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<&''static mut [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Vec<Option<&''static mut [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:451 |
| `Option<&''static crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&''static mut crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&''static crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
| `&''static crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static mut crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static mut crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static mut crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static mut [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static mut [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `&''static mut crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static str>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:121 |
| `char` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:199 |
| `String` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:245 |
| `Option<String>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:317 |
| `&''static str` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:47 |
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
| `SMatrix<T, {'expr': 'R', 'value': None, 'is_literal': False}, {'expr': 'C', 'value': None, 'is_literal': False}>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:246 |
| `Option<SMatrix<T, {'expr': 'R', 'value': None, 'is_literal': False}, {'expr': 'C', 'value': None, 'is_literal': False}>>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:312 |
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
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:671 |
| `RArray<i8, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:778 |
| `RArray<i16, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:779 |
| `RArray<i64, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:780 |
| `RArray<isize, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:781 |
| `RArray<u16, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:782 |
| `RArray<u32, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:783 |
| `RArray<u64, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:784 |
| `RArray<usize, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:785 |
| `RArray<f32, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:788 |
| `RArray<bool, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:791 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:497 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:508 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:519 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:555 |
| `RCow<''static, T>` | `<T>` | concrete | 3 | miniextendr-api/src/rcow.rs:167 |
| `FromJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:97 |
| `StrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:428 |
| `ProtectedStrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:668 |

### `TryFromSexp` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/from_r/references.rs:449** (12 impls): `Option<&''static f64>`, `&''static mut f64`, `&''static f64`, `Option<&''static mut f64>`, `Vec<&''static f64>`, `Vec<Option<&''static f64>>`, `Vec<&''static mut f64>`, `Vec<Option<&''static mut f64>>`, `Vec<&''static [f64]>`, `Vec<Option<&''static [f64]>>`, `Vec<&''static mut [f64]>`, `Vec<Option<&''static mut [f64]>>`
- **miniextendr-api/src/from_r/references.rs:451** (12 impls): `Option<&''static mut crate::RLogical>`, `Vec<&''static crate::RLogical>`, `Vec<Option<&''static crate::RLogical>>`, `Vec<&''static mut crate::RLogical>`, `Vec<Option<&''static mut crate::RLogical>>`, `Vec<&''static [crate::RLogical]>`, `Vec<Option<&''static [crate::RLogical]>>`, `Vec<&''static mut [crate::RLogical]>`, `Vec<Option<&''static mut [crate::RLogical]>>`, `Option<&''static crate::RLogical>`, `&''static mut crate::RLogical`, `&''static crate::RLogical`
- **miniextendr-api/src/from_r/references.rs:452** (12 impls): `&''static crate::Rcomplex`, `Option<&''static mut crate::Rcomplex>`, `Vec<&''static crate::Rcomplex>`, `Vec<Option<&''static crate::Rcomplex>>`, `Vec<&''static mut crate::Rcomplex>`, `Vec<Option<&''static mut crate::Rcomplex>>`, `Vec<&''static [crate::Rcomplex]>`, `Vec<Option<&''static [crate::Rcomplex]>>`, `Vec<&''static mut [crate::Rcomplex]>`, `Vec<Option<&''static mut [crate::Rcomplex]>>`, `&''static mut crate::Rcomplex`, `Option<&''static crate::Rcomplex>`
- **miniextendr-api/src/from_r/references.rs:448** (12 impls): `Option<&''static i32>`, `&''static mut i32`, `&''static i32`, `Option<&''static mut i32>`, `Vec<&''static i32>`, `Vec<Option<&''static i32>>`, `Vec<&''static mut i32>`, `Vec<Option<&''static mut i32>>`, `Vec<&''static [i32]>`, `Vec<Option<&''static [i32]>>`, `Vec<&''static mut [i32]>`, `Vec<Option<&''static mut [i32]>>`
- **miniextendr-api/src/from_r/references.rs:450** (12 impls): `&''static mut u8`, `&''static u8`, `Option<&''static mut u8>`, `Vec<&''static u8>`, `Vec<Option<&''static u8>>`, `Vec<&''static mut u8>`, `Vec<Option<&''static mut u8>>`, `Vec<&''static [u8]>`, `Vec<Option<&''static [u8]>>`, `Vec<&''static mut [u8]>`, `Vec<Option<&''static mut [u8]>>`, `Option<&''static u8>`
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
| `Vec<std::borrow::Cow<''_, str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1314 |
| `Vec<Option<std::borrow::Cow<''_, str>>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1333 |
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
| `std::borrow::Cow<''_, [T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:752 |
| `std::borrow::Cow<''_, str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:772 |
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
| `SMatrix<T, {'expr': 'R', 'value': None, 'is_literal': False}, {'expr': 'C', 'value': None, 'is_literal': False}>` | `<T, R, C>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:286 |
| `Option<SMatrix<T, {'expr': 'R', 'value': None, 'is_literal': False}, {'expr': 'C', 'value': None, 'is_literal': False}>>` | `<T, R, C>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:335 |
| `DVector<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:924 |
| `DMatrix<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:946 |
| `ArcArray2<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1009 |
| `ArrayView1<''a, T>` | `<'a, T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1030 |
| `ArrayView2<''a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1048 |
| `ArrayView3<''a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1079 |
| `ArrayViewD<''a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1122 |
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
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 5 | miniextendr-api/src/rarray.rs:796 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:408 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:422 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:436 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:465 |
| `RCow<''_, T>` | `<T>` | concrete | 5 | miniextendr-api/src/rcow.rs:185 |
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

## `From` — 255 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsDisplayVec<T>` | `<T>` | blanket | 1 | (no span) |
| `RDebugView` | `<T>` | blanket | 1 | (no span) |
| `CollectNA<I>` | `<T>` | blanket | 1 | (no span) |
| `RNGtype` | `<T>` | blanket | 1 | (no span) |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T>` | blanket | 1 | (no span) |
| `RndVec<T>` | `<T>` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T>` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T>` | blanket | 1 | (no span) |
| `IterLogicalData<I>` | `<T>` | blanket | 1 | (no span) |
| `RWrapperEntry` | `<T>` | blanket | 1 | (no span) |
| `RSerdeError` | `<T>` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T>` | blanket | 1 | (no span) |
| `ThreadLocalArena` | `<T>` | blanket | 1 | (no span) |
| `RBorrow<''a, T>` | `<T>` | blanket | 1 | (no span) |
| `NamedList` | `<T>` | blanket | 1 | (no span) |
| `StrVecBuilder<''a>` | `<T>` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T>` | blanket | 1 | (no span) |
| `DllInfo` | `<T>` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T>` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T>` | blanket | 1 | (no span) |
| `SparseIterIntData<I>` | `<T>` | blanket | 1 | (no span) |
| `MatchArgError` | `<T>` | blanket | 1 | (no span) |
| `TypedListError` | `<T>` | blanket | 1 | (no span) |
| `RCloneView` | `<T>` | blanket | 1 | (no span) |
| `TlsRoot` | `<T>` | blanket | 1 | (no span) |
| `SexpError` | `<T>` | blanket | 1 | (no span) |
| `DataFrameShape` | `<T>` | blanket | 1 | (no span) |
| `Missing<T>` | `<T>` | blanket | 1 | (no span) |
| `FromJson<T>` | `<T>` | blanket | 1 | (no span) |
| `ProtectKey` | `<T>` | blanket | 1 | (no span) |
| `Sortedness` | `<T>` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<T>` | blanket | 1 | (no span) |
| `cetype_t` | `<T>` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T>` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T>` | blanket | 1 | (no span) |
| `IterState<I, T>` | `<T>` | blanket | 1 | (no span) |
| `ThreadLocalHashArena` | `<T>` | blanket | 1 | (no span) |
| `MatchArgChoicesEntry` | `<T>` | blanket | 1 | (no span) |
| `mx_erased` | `<T>` | blanket | 1 | (no span) |
| `JsonOptions` | `<T>` | blanket | 1 | (no span) |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T>` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T>` | blanket | 1 | (no span) |
| `ReprotectSlot<''a>` | `<T>` | blanket | 1 | (no span) |
| `StrVecCowIter` | `<T>` | blanket | 1 | (no span) |
| `SerdeRows<T>` | `<T>` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T>` | blanket | 1 | (no span) |
| `TraitDispatchRow` | `<T>` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T>` | blanket | 1 | (no span) |
| `RSymbol` | `<T>` | blanket | 1 | (no span) |
| `RRng` | `<T>` | blanket | 1 | (no span) |
| `RStringArray` | `<T>` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T>` | blanket | 1 | (no span) |
| `CollectStrings<I>` | `<T>` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T>` | blanket | 1 | (no span) |
| `Borsh<T>` | `<T>` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T>` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T>` | blanket | 1 | (no span) |
| `RDefaultView` | `<T>` | blanket | 1 | (no span) |
| `NaHandling` | `<T>` | blanket | 1 | (no span) |
| `RIteratorView` | `<T>` | blanket | 1 | (no span) |
| `Dots` | `<T>` | blanket | 1 | (no span) |
| `DispatchNames` | `<T>` | blanket | 1 | (no span) |
| `RAllocator` | `<T>` | blanket | 1 | (no span) |
| `MatchArgParamDocEntry` | `<T>` | blanket | 1 | (no span) |
| `SparseIterRealData<I>` | `<T>` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T>` | blanket | 1 | (no span) |
| `WindowedIterIntData<I>` | `<T>` | blanket | 1 | (no span) |
| `Root<''a>` | `<T>` | blanket | 1 | (no span) |
| `VctrsKind` | `<T>` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T>` | blanket | 1 | (no span) |
| `Protected<''a, T>` | `<T>` | blanket | 1 | (no span) |
| `StrVecIter` | `<T>` | blanket | 1 | (no span) |
| `RThreadBuilder` | `<T>` | blanket | 1 | (no span) |
| `TraitDispatchEntry` | `<T>` | blanket | 1 | (no span) |
| `AsJsonPretty<T>` | `<T>` | blanket | 1 | (no span) |
| `NullOnErr` | `<T>` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<T>` | blanket | 1 | (no span) |
| `IterComplexData<I>` | `<T>` | blanket | 1 | (no span) |
| `JiffTimestampVec` | `<T>` | blanket | 1 | (no span) |
| `PanicReport<''a>` | `<T>` | blanket | 1 | (no span) |
| `RCoerceError` | `<T>` | blanket | 1 | (no span) |
| `RawError` | `<T>` | blanket | 1 | (no span) |
| `ResultShape<S>` | `<T>` | blanket | 1 | (no span) |
| `mx_base_vtable` | `<T>` | blanket | 1 | (no span) |
| `ProtectPool` | `<T>` | blanket | 1 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T>` | blanket | 1 | (no span) |
| `ListBuilder<''a>` | `<T>` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T>` | blanket | 1 | (no span) |
| `AltrepRegRow` | `<T>` | blanket | 1 | (no span) |
| `TypeSpec` | `<T>` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T>` | blanket | 1 | (no span) |
| `WorkerPump<T>` | `<T>` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T>` | blanket | 1 | (no span) |
| `ClassNameEntry` | `<T>` | blanket | 1 | (no span) |
| `StreamingRealData<F>` | `<T>` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T>` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T>` | blanket | 1 | (no span) |
| `Collect<I>` | `<T>` | blanket | 1 | (no span) |
| `AltrepSexp` | `<T>` | blanket | 1 | (no span) |
| `StrVec` | `<T>` | blanket | 1 | (no span) |
| `Sampletype` | `<T>` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 1 | (no span) |
| `RErrorView` | `<T>` | blanket | 1 | (no span) |
| `Rboolean` | `<T>` | blanket | 1 | (no span) |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 1 | (no span) |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 1 | (no span) |
| `mx_tag` | `<T>` | blanket | 1 | (no span) |
| `WindowedIterRealData<I>` | `<T>` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T>` | blanket | 1 | (no span) |
| `RSidecar` | `<T>` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T>` | blanket | 1 | (no span) |
| `CoerceError` | `<T>` | blanket | 1 | (no span) |
| `SexpNaError` | `<T>` | blanket | 1 | (no span) |
| `ListAccumulator<''a>` | `<T>` | blanket | 1 | (no span) |
| `RDataFrameBuilder` | `<T>` | blanket | 1 | (no span) |
| `GuardMode` | `<T>` | blanket | 1 | (no span) |
| `SidecarPropEntry` | `<T>` | blanket | 1 | (no span) |
| `RngGuard` | `<T>` | blanket | 1 | (no span) |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 1 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 1 | (no span) |
| `ParseStatus` | `<T>` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<T>` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<T>` | blanket | 1 | (no span) |
| `AsJson<T>` | `<T>` | blanket | 1 | (no span) |
| `AsList<T>` | `<T>` | blanket | 1 | (no span) |
| `IterRawData<I>` | `<T>` | blanket | 1 | (no span) |
| `ExternalSlice<T>` | `<T>` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T>` | blanket | 1 | (no span) |
| `SparseIterComplexData<I>` | `<T>` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T>` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T>` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T>` | blanket | 1 | (no span) |
| `SparseIterState<I, T>` | `<T>` | blanket | 1 | (no span) |
| `StreamingIntData<F>` | `<T>` | blanket | 1 | (no span) |
| `IterIntData<I>` | `<T>` | blanket | 1 | (no span) |
| `SplitShape` | `<T>` | blanket | 1 | (no span) |
| `TypedList` | `<T>` | blanket | 1 | (no span) |
| `WorkerUnprotectGuard` | `<T>` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T>` | blanket | 1 | (no span) |
| `RHashView` | `<T>` | blanket | 1 | (no span) |
| `Factor<''a>` | `<T>` | blanket | 1 | (no span) |
| `TypeMismatchError` | `<T>` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<T>` | blanket | 1 | (no span) |
| `RSerializer` | `<T>` | blanket | 1 | (no span) |
| `SparseIterRawData<I>` | `<T>` | blanket | 1 | (no span) |
| `REnv` | `<T>` | blanket | 1 | (no span) |
| `List` | `<T>` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T>` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T>` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T>` | blanket | 1 | (no span) |
| `RLogical` | `<T>` | blanket | 1 | (no span) |
| `RCopyView` | `<T>` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T>` | blanket | 1 | (no span) |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 1 | (no span) |
| `DataFrame` | `<T>` | blanket | 1 | (no span) |
| `JiffZonedVec` | `<T>` | blanket | 1 | (no span) |
| `AltrepRegistration` | `<T>` | blanket | 1 | (no span) |
| `CollectNAInt<I>` | `<T>` | blanket | 1 | (no span) |
| `DataFrameError` | `<T>` | blanket | 1 | (no span) |
| `Missing<T>` | `<T>` | blanket | 1 | (no span) |
| `ROrdView` | `<T>` | blanket | 1 | (no span) |
| `ProtectScope` | `<T>` | blanket | 1 | (no span) |
| `OwnedProtect` | `<T>` | blanket | 1 | (no span) |
| `RDisplayView` | `<T>` | blanket | 1 | (no span) |
| `IterRealData<I>` | `<T>` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T>` | blanket | 1 | (no span) |
| `WindowedIterState<I, T>` | `<T>` | blanket | 1 | (no span) |
| `IterListData<I>` | `<T>` | blanket | 1 | (no span) |
| `Raw<T>` | `<T>` | blanket | 1 | (no span) |
| `Logical` | `<T>` | blanket | 1 | (no span) |
| `PanicSource` | `<T>` | blanket | 1 | (no span) |
| `TypeSpec` | `<T>` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<T>` | blanket | 1 | (no span) |
| `RCall` | `<T>` | blanket | 1 | (no span) |
| `RndMat<T>` | `<T>` | blanket | 1 | (no span) |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 1 | (no span) |
| `ListMut` | `<T>` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T>` | blanket | 1 | (no span) |
| `AsJsonVec<T>` | `<T>` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T>` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T>` | blanket | 1 | (no span) |
| `N01type` | `<T>` | blanket | 1 | (no span) |
| `ProtectedStrVec` | `<T>` | blanket | 1 | (no span) |
| `SplitResults` | `<T>` | blanket | 1 | (no span) |
| `TypedEntry` | `<T>` | blanket | 1 | (no span) |
| `RBase` | `<T>` | blanket | 1 | (no span) |
| `Rcomplex` | `<T>` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<T>` | blanket | 1 | (no span) |
| `FactorHandling` | `<T>` | blanket | 1 | (no span) |
| `CallDefRow` | `<T>` | blanket | 1 | (no span) |
| `RPartialOrdView` | `<T>` | blanket | 1 | (no span) |
| `RFromStrView` | `<T>` | blanket | 1 | (no span) |
| `IntoRError` | `<T>` | blanket | 1 | (no span) |
| `RawHeader` | `<T>` | blanket | 1 | (no span) |
| `SEXPREC` | `<T>` | blanket | 1 | (no span) |
| `IterStringData<I>` | `<T>` | blanket | 1 | (no span) |
| `Arena<M>` | `<T>` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T>` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T>` | blanket | 1 | (no span) |
| `RDeserializer` | `<T>` | blanket | 1 | (no span) |
| `SEXP` | `<T>` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T>` | blanket | 1 | (no span) |
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
| `ArrayView1<''a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1453 |
| `ArrayView2<''a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1479 |
| `ArrayView3<''a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1491 |
| `ArrayViewD<''a, T>` | `<'a, T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1503 |
| `Array1<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1533 |
| `Array2<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1544 |
| `Array3<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1556 |
| `ArrayD<T>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1569 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:158 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:164 |
| `RCow<''_, T>` | `<T>` | concrete | 1 | miniextendr-api/src/rcow.rs:157 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:255 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:477 |
| `*mut SEXPREC` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:484 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:202 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:330 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:339 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:265 |

### `From` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (206 impls): `AsDisplayVec<T>`, `RDebugView`, `CollectNA<I>`, `RNGtype`, `IterIntCoerceData<I, T>`, `Coerced<T, R>`, `RndVec<T>`, `FactorVec<T>`, `REncodingInfo`, `IterLogicalData<I>`, `RWrapperEntry`, `RSerdeError`, `SEXPTYPE`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `AsNamedList<T>`, `DllInfo`, `R_CallMethodDef`, `AsDataFrame<T>`, `SparseIterIntData<I>`, `MatchArgError`, `TypedListError`, `RCloneView`, `TlsRoot`, `SexpError`, `DataFrameShape`, `Missing<T>`, `FromJson<T>`, `ProtectKey`, `Sortedness`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `JsonOptions`, `RVecStorage<T, R, C>`, `AsDisplay<T>`, `NamedVector<M>`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `ListFromSexpError`, `TraitDispatchRow`, `ExternalPtr<T>`, `RSymbol`, `RRng`, `RStringArray`, `CaptureGroups`, `CollectStrings<I>`, `AsNamedVector<T>`, `Borsh<T>`, `AsRNative<T>`, `Altrep<T>`, `RDefaultView`, `NaHandling`, `RIteratorView`, `Dots`, `DispatchNames`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `StorageCoerceError`, `WindowedIterIntData<I>`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `Protected<''a, T>`, `StrVecIter`, `RThreadBuilder`, `TraitDispatchEntry`, `AsJsonPretty<T>`, `NullOnErr`, `JiffZonedVecMut`, `IterComplexData<I>`, `JiffTimestampVec`, `PanicReport<''a>`, `RCoerceError`, `RawError`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `ListBuilder<''a>`, `TypedListSpec`, `AltrepRegRow`, `TypeSpec`, `AsRError<E>`, `WorkerPump<T>`, `AsExternalPtr<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `SexpTypeError`, `SpecialFloatHandling`, `Collect<I>`, `AltrepSexp`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `RErrorView`, `Rboolean`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `mx_tag`, `WindowedIterRealData<I>`, `NamedVector<M>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `GuardMode`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsJson<T>`, `AsList<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsVctrs<T>`, `SparseIterComplexData<I>`, `Altrep<T>`, `R_altrep_class_t`, `RawTagged<T>`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `IterIntData<I>`, `SplitShape`, `TypedList`, `WorkerUnprotectGuard`, `RFlags<T>`, `RHashView`, `Factor<''a>`, `TypeMismatchError`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `List`, `VctrsBuildError`, `AsRError<E>`, `AsNamedList<T>`, `RLogical`, `RCopyView`, `LogicalCoerceError`, `IterIntFromBoolData<I>`, `DataFrame`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `DataFrameError`, `Missing<T>`, `ROrdView`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `RWrapperPriority`, `WindowedIterState<I, T>`, `IterListData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `RCall`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsNamedVector<T>`, `AsJsonVec<T>`, `R_CMethodDef`, `FactorOptionVec<T>`, `N01type`, `ProtectedStrVec`, `SplitResults`, `TypedEntry`, `RBase`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `FactorHandling`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `IntoRError`, `RawHeader`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `AltrepGuard`, `SexpLengthError`, `RDeserializer`, `SEXP`, `DuplicateNameError`

## `BorrowMut` — 201 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsJsonPretty<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicReport<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | (no span) |
| `ResultShape<S>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_base_vtable` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectPool` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ClassNameEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingRealData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<T> +1wc` | blanket | 1 | (no span) |
| `Collect<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepSexp` | `<T> +1wc` | blanket | 1 | (no span) |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RErrorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sortedness` | `<T> +1wc` | blanket | 1 | (no span) |
| `cetype_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRowBuilder<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListAccumulator<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDataFrameBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SidecarPropEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RngGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedDataFrameListBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBase` | `<T> +1wc` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NaHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingIntData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerUnprotectGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `RHashView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Factor<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NullOnErr` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REnv` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCopyView` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntFromBoolData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegistration` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNAInt<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sampletype` | `<T> +1wc` | blanket | 1 | (no span) |
| `ROrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rboolean` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectScope` | `<T> +1wc` | blanket | 1 | (no span) |
| `OwnedProtect` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDisplayView` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_tag` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterListData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSidecar` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | (no span) |
| `GuardMode` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCall` | `<T> +1wc` | blanket | 1 | (no span) |
| `ParseStatus` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndMat<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJsonVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitResults` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `CallDefRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPartialOrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFromStrView` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPREC` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterStringData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Arena<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDeserializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDebugView` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNA<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `List` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBorrow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `DllInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `RLogical` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrame` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCloneView` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `FromJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalHashArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgChoicesEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_erased` | `<T> +1wc` | blanket | 1 | (no span) |
| `RVecStorage<T, R, C>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Raw<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Logical` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicSource` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `ReprotectSlot<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecCowIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRows<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSymbol` | `<T> +1wc` | blanket | 1 | (no span) |
| `RRng` | `<T> +1wc` | blanket | 1 | (no span) |
| `RStringArray` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectStrings<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `Borsh<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `N01type` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDefaultView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rcomplex` | `<T> +1wc` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RIteratorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `Dots` | `<T> +1wc` | blanket | 1 | (no span) |
| `RAllocator` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgParamDocEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawHeader` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `Protected<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `RThreadBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXP` | `<T> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RNGtype` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1553 |

### `BorrowMut` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `AsJsonPretty<T>`, `Coerced<T, R>`, `JiffZonedVecMut`, `IterComplexData<I>`, `FactorVec<T>`, `REncodingInfo`, `JiffTimestampVec`, `PanicReport<''a>`, `RSerdeError`, `SEXPTYPE`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `ListBuilder<''a>`, `AsNamedList<T>`, `AltrepRegRow`, `R_CallMethodDef`, `WorkerPump<T>`, `AsDataFrame<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `MatchArgError`, `TypedListError`, `TlsRoot`, `Collect<I>`, `SexpError`, `AltrepSexp`, `Missing<T>`, `ProtectKey`, `ArenaGuard<''a, M>`, `RErrorView`, `Sortedness`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `SerdeRowBuilder<T>`, `JsonOptions`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `AsDisplay<T>`, `ListFromSexpError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `ExternalPtr<T>`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `RBase`, `CaptureGroups`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsNamedVector<T>`, `AsJson<T>`, `IterRawData<I>`, `AsRNative<T>`, `ExternalSlice<T>`, `Altrep<T>`, `SparseIterComplexData<I>`, `NaHandling`, `DispatchNames`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `StorageCoerceError`, `IterIntData<I>`, `SplitShape`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `WorkerUnprotectGuard`, `RHashView`, `Factor<''a>`, `NullOnErr`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `RCoerceError`, `RawError`, `RawSlice<T>`, `TypedListSpec`, `TypeSpec`, `AsRError<E>`, `AsExternalPtr<T>`, `RCopyView`, `SexpTypeError`, `IterIntFromBoolData<I>`, `SpecialFloatHandling`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `StrVec`, `Sampletype`, `ROrdView`, `Rboolean`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `mx_tag`, `IterListData<I>`, `NamedVector<M>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `FactorMut<''a>`, `SexpNaError`, `GuardMode`, `RCall`, `ParseStatus`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsList<T>`, `AsVctrs<T>`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `R_altrep_class_t`, `RawTagged<T>`, `JiffTimestampVecMut`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `TypedList`, `RFlags<T>`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`, `TypeMismatchError`, `RndVec<T>`, `IterLogicalData<I>`, `List`, `RWrapperEntry`, `ThreadLocalArena`, `VctrsBuildError`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `DllInfo`, `RLogical`, `SparseIterIntData<I>`, `LogicalCoerceError`, `DataFrame`, `RCloneView`, `DataFrameShape`, `DataFrameError`, `FromJson<T>`, `JiffTimestampVecRef`, `IterState<I, T>`, `ThreadLocalHashArena`, `RWrapperPriority`, `MatchArgChoicesEntry`, `mx_erased`, `RVecStorage<T, R, C>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `TraitDispatchRow`, `RSymbol`, `RRng`, `RStringArray`, `CollectStrings<I>`, `R_CMethodDef`, `Borsh<T>`, `FactorOptionVec<T>`, `N01type`, `TypedEntry`, `RDefaultView`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `RIteratorView`, `FactorHandling`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `IntoRError`, `SparseIterRealData<I>`, `RawHeader`, `WindowedIterIntData<I>`, `AltrepGuard`, `Protected<''a, T>`, `SexpLengthError`, `StrVecIter`, `RThreadBuilder`, `SEXP`, `DuplicateNameError`, `TraitDispatchEntry`, `AsDisplayVec<T>`, `RNGtype`

## `Borrow` — 201 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `List` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBorrow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `DllInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `RLogical` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrame` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCloneView` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `FromJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalHashArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgChoicesEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_erased` | `<T> +1wc` | blanket | 1 | (no span) |
| `RVecStorage<T, R, C>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Raw<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Logical` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicSource` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `ReprotectSlot<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecCowIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRows<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSymbol` | `<T> +1wc` | blanket | 1 | (no span) |
| `RRng` | `<T> +1wc` | blanket | 1 | (no span) |
| `RStringArray` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectStrings<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `Borsh<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `N01type` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDefaultView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rcomplex` | `<T> +1wc` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RIteratorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `Dots` | `<T> +1wc` | blanket | 1 | (no span) |
| `RAllocator` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgParamDocEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawHeader` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `Protected<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `RThreadBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXP` | `<T> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RNGtype` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJsonPretty<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicReport<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | (no span) |
| `ResultShape<S>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_base_vtable` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectPool` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ClassNameEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingRealData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<T> +1wc` | blanket | 1 | (no span) |
| `Collect<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepSexp` | `<T> +1wc` | blanket | 1 | (no span) |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RErrorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sortedness` | `<T> +1wc` | blanket | 1 | (no span) |
| `cetype_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRowBuilder<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListAccumulator<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDataFrameBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SidecarPropEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBase` | `<T> +1wc` | blanket | 1 | (no span) |
| `RngGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedDataFrameListBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NaHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingIntData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerUnprotectGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `RHashView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Factor<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NullOnErr` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REnv` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCopyView` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntFromBoolData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegistration` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNAInt<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sampletype` | `<T> +1wc` | blanket | 1 | (no span) |
| `ROrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rboolean` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectScope` | `<T> +1wc` | blanket | 1 | (no span) |
| `OwnedProtect` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDisplayView` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_tag` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterListData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSidecar` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | (no span) |
| `GuardMode` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCall` | `<T> +1wc` | blanket | 1 | (no span) |
| `ParseStatus` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndMat<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJsonVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitResults` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `CallDefRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPartialOrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFromStrView` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPREC` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterStringData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Arena<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDeserializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDebugView` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNA<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1546 |

### `Borrow` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `TypeMismatchError`, `RndVec<T>`, `IterLogicalData<I>`, `List`, `RWrapperEntry`, `ThreadLocalArena`, `VctrsBuildError`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `DllInfo`, `RLogical`, `SparseIterIntData<I>`, `LogicalCoerceError`, `DataFrame`, `RCloneView`, `DataFrameShape`, `DataFrameError`, `FromJson<T>`, `JiffTimestampVecRef`, `IterState<I, T>`, `ThreadLocalHashArena`, `RWrapperPriority`, `MatchArgChoicesEntry`, `mx_erased`, `RVecStorage<T, R, C>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `TraitDispatchRow`, `RSymbol`, `RRng`, `RStringArray`, `CollectStrings<I>`, `R_CMethodDef`, `Borsh<T>`, `FactorOptionVec<T>`, `N01type`, `TypedEntry`, `RDefaultView`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `RIteratorView`, `FactorHandling`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `IntoRError`, `SparseIterRealData<I>`, `RawHeader`, `WindowedIterIntData<I>`, `AltrepGuard`, `Protected<''a, T>`, `SexpLengthError`, `StrVecIter`, `RThreadBuilder`, `SEXP`, `DuplicateNameError`, `TraitDispatchEntry`, `AsDisplayVec<T>`, `RNGtype`, `AsJsonPretty<T>`, `Coerced<T, R>`, `JiffZonedVecMut`, `IterComplexData<I>`, `FactorVec<T>`, `REncodingInfo`, `JiffTimestampVec`, `PanicReport<''a>`, `RSerdeError`, `SEXPTYPE`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `ListBuilder<''a>`, `AsNamedList<T>`, `AltrepRegRow`, `R_CallMethodDef`, `WorkerPump<T>`, `AsDataFrame<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `MatchArgError`, `TypedListError`, `TlsRoot`, `Collect<I>`, `SexpError`, `AltrepSexp`, `Missing<T>`, `ProtectKey`, `ArenaGuard<''a, M>`, `RErrorView`, `Sortedness`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `SerdeRowBuilder<T>`, `JsonOptions`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `AsDisplay<T>`, `ListFromSexpError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `ExternalPtr<T>`, `SidecarPropEntry`, `RBase`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `CaptureGroups`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsNamedVector<T>`, `AsJson<T>`, `IterRawData<I>`, `AsRNative<T>`, `ExternalSlice<T>`, `Altrep<T>`, `SparseIterComplexData<I>`, `NaHandling`, `DispatchNames`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `StorageCoerceError`, `IterIntData<I>`, `SplitShape`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `WorkerUnprotectGuard`, `RHashView`, `Factor<''a>`, `NullOnErr`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `RCoerceError`, `RawError`, `RawSlice<T>`, `TypedListSpec`, `TypeSpec`, `AsRError<E>`, `AsExternalPtr<T>`, `RCopyView`, `SexpTypeError`, `IterIntFromBoolData<I>`, `SpecialFloatHandling`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `StrVec`, `Sampletype`, `ROrdView`, `Rboolean`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `mx_tag`, `IterListData<I>`, `NamedVector<M>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `FactorMut<''a>`, `SexpNaError`, `GuardMode`, `RCall`, `ParseStatus`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsList<T>`, `AsVctrs<T>`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `R_altrep_class_t`, `RawTagged<T>`, `JiffTimestampVecMut`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `TypedList`, `RFlags<T>`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`

## `Tap` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SparseIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StreamingIntData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `FactorHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SplitShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Root<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StorageCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RHashView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AltrepGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsDisplayVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SEXP` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RNGtype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RSerializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SparseIterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `REncodingInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `REnv` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `FactorVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SEXPTYPE` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RawError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `R_CallMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsDataFrame<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RCopyView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffZonedVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AltrepRegistration` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CollectNAInt<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RBase` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ROrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectScope` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectKey` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffTimestampVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `cetype_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Sortedness` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsFromStr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RDisplayView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterListData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RawSliceTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JsonOptions` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsDisplay<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `NamedVector<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsSerialize<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SexpNaError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RCall` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RndMat<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ListMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RStringArray` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CaptureGroups` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RRng` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsJsonVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectedStrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SplitResults` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsRNative<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `NaHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CallDefRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DispatchNames` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RPartialOrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RFromStrView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SEXPREC` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterStringData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsFromStrVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Arena<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `VctrsKind` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RDeserializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RDebugView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CollectNA<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RFlags<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RawHeader` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffZonedVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RndVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `NullOnErr` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StrVecIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypeMismatchError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RWrapperEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ThreadLocalArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RBorrow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `NamedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypedListSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DllInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsRError<E>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RawSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SparseIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `VctrsBuildError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Altrep<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Coerced<T, R>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RCloneView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SpecialFloatHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `LogicalCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SexpTypeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DataFrameShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `FromJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DataFrameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Sampletype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Rboolean` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ThreadLocalHashArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `mx_erased` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `mx_tag` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SerdeRows<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RSidecar` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TraitDispatchRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StrVecCowIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RSymbol` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `GuardMode` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `CollectStrings<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RCow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffZonedVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ParseStatus` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Borsh<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsVctrs<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `FactorOptionVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RDefaultView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `R_altrep_class_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RIteratorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Rcomplex` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Dots` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RAllocator` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RawTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SparseIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IntoRError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RThreadBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TraitDispatchEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Protected<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SexpLengthError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DuplicateNameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsJsonPretty<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Factor<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffTimestampVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RPrimitive<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `PanicReport<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `List` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ResultShape<S>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `mx_base_vtable` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RSerdeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ProtectPool` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ListBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AltrepRegRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `WorkerPump<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsNamedList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ClassNameEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `StreamingRealData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RLogical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `MatchArgError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypedListError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `DataFrame` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Collect<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AltrepSexp` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TlsRoot` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RErrorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Missing<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `OwnedProtect` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `FactorMut<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ListAccumulator<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RDataFrameBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Raw<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `Logical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RWrapperPriority` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `PanicSource` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ListFromSexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SidecarPropEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RngGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `IterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `ExternalSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `AsNamedVector<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `R_CMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `TypedEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `N01type` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |
| `JiffTimestampVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329 |

### `Tap` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/tap.rs:329** (200 impls): `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `Root<''a>`, `StorageCoerceError`, `RHashView`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `ProtectedStrVecIter<''a>`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `RBase`, `ROrdView`, `ProtectScope`, `ProtectKey`, `JiffTimestampVecRef`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `RVecStorage<T, R, C>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `CoerceError`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `VctrsKind`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `RFlags<T>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `NullOnErr`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `SparseIterIntData<I>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `RCloneView`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Sampletype`, `IterState<I, T>`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `ExternalPtr<T>`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `Protected<''a, T>`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `N01type`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`

## `TryInto` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `TraitDispatchEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJsonPretty<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NullOnErr` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVecMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterComplexData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `PanicReport<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ResultShape<S>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_base_vtable` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectPool` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawSlice<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListBuilder<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedListSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepRegRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsRError<E>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkerPump<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsExternalPtr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ClassNameEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StreamingRealData<F>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpTypeError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SpecialFloatHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Collect<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepSexp` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Sampletype` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ArenaGuard<''a, M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RErrorView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Rboolean` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SerdeRowBuilder<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterLogicalData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_tag` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSidecar` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsSerialize<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpNaError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListAccumulator<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDataFrameBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `GuardMode` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SidecarPropEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RngGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedDataFrameListBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ParseStatus` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCow<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVecRef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJson<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsList<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRawData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ExternalSlice<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsVctrs<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterComplexData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Altrep<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_altrep_class_t` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawTagged<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RBase` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StreamingIntData<F>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SplitShape` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedList` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkerUnprotectGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RFlags<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RHashView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Factor<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeMismatchError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RPrimitive<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSerializer` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterRawData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REnv` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `List` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `VctrsBuildError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsNamedList<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RLogical` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCopyView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LogicalCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntFromBoolData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrame` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepRegistration` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectNAInt<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrameError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Missing<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ROrdView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectScope` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `OwnedProtect` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDisplayView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RWrapperPriority` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterListData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Raw<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Logical` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `PanicSource` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorMut<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCall` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RndMat<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRealCoerceData<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsNamedVector<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJsonVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_CMethodDef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorOptionVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `N01type` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SplitResults` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Rcomplex` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVecMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CallDefRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RPartialOrdView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RFromStrView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IntoRError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawHeader` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXPREC` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterStringData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Arena<M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpLengthError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDeserializer` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXP` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DuplicateNameError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDisplayVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDebugView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectNA<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RNGtype` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntCoerceData<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Coerced<T, R>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RndVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REncodingInfo` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterLogicalData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RWrapperEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSerdeError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXPTYPE` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ThreadLocalArena` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RBorrow<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedList` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecBuilder<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DllInfo` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_CallMethodDef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDataFrame<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedListError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCloneView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TlsRoot` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrameShape` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FromJson<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectKey` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Sortedness` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVecRef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `cetype_t` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawSliceTagged<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsFromStr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ThreadLocalHashArena` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgChoicesEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_erased` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JsonOptions` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RVecStorage<T, R, C>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDisplay<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedVector<M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ReprotectSlot<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecCowIter` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SerdeRows<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListFromSexpError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TraitDispatchRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ExternalPtr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSymbol` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RRng` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RStringArray` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CaptureGroups` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectStrings<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Borsh<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsRNative<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDefaultView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NaHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RIteratorView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Dots` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DispatchNames` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RAllocator` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgParamDocEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StorageCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Root<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `VctrsKind` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsFromStrVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Protected<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecIter` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RThreadBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryInto` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `TraitDispatchEntry`, `AsJsonPretty<T>`, `NullOnErr`, `JiffZonedVecMut`, `IterComplexData<I>`, `JiffTimestampVec`, `PanicReport<''a>`, `RCoerceError`, `RawError`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `ListBuilder<''a>`, `TypedListSpec`, `AltrepRegRow`, `TypeSpec`, `AsRError<E>`, `WorkerPump<T>`, `AsExternalPtr<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `SexpTypeError`, `SpecialFloatHandling`, `Collect<I>`, `AltrepSexp`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `RErrorView`, `Rboolean`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `mx_tag`, `WindowedIterRealData<I>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `GuardMode`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsJson<T>`, `AsList<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsVctrs<T>`, `SparseIterComplexData<I>`, `Altrep<T>`, `R_altrep_class_t`, `RawTagged<T>`, `RBase`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `IterIntData<I>`, `SplitShape`, `TypedList`, `WorkerUnprotectGuard`, `RFlags<T>`, `RHashView`, `Factor<''a>`, `TypeMismatchError`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `List`, `VctrsBuildError`, `AsNamedList<T>`, `RLogical`, `RCopyView`, `LogicalCoerceError`, `IterIntFromBoolData<I>`, `DataFrame`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `DataFrameError`, `Missing<T>`, `ROrdView`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `RWrapperPriority`, `WindowedIterState<I, T>`, `IterListData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `RCall`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsNamedVector<T>`, `AsJsonVec<T>`, `R_CMethodDef`, `FactorOptionVec<T>`, `N01type`, `ProtectedStrVec`, `SplitResults`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `FactorHandling`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `IntoRError`, `RawHeader`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `AltrepGuard`, `SexpLengthError`, `RDeserializer`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RDebugView`, `CollectNA<I>`, `RNGtype`, `IterIntCoerceData<I, T>`, `Coerced<T, R>`, `RndVec<T>`, `FactorVec<T>`, `REncodingInfo`, `IterLogicalData<I>`, `RWrapperEntry`, `RSerdeError`, `SEXPTYPE`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `DllInfo`, `R_CallMethodDef`, `AsDataFrame<T>`, `SparseIterIntData<I>`, `MatchArgError`, `TypedListError`, `RCloneView`, `TlsRoot`, `SexpError`, `DataFrameShape`, `FromJson<T>`, `ProtectKey`, `Sortedness`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `JsonOptions`, `RVecStorage<T, R, C>`, `AsDisplay<T>`, `NamedVector<M>`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `ListFromSexpError`, `TraitDispatchRow`, `ExternalPtr<T>`, `RSymbol`, `RRng`, `RStringArray`, `CaptureGroups`, `CollectStrings<I>`, `Borsh<T>`, `AsRNative<T>`, `RDefaultView`, `NaHandling`, `RIteratorView`, `Dots`, `DispatchNames`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `StorageCoerceError`, `WindowedIterIntData<I>`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `Protected<''a, T>`, `StrVecIter`, `RThreadBuilder`

## `Same` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IntoRError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypedList` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StorageCoerceError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RFlags<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AltrepGuard` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RThreadBuilder` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TraitDispatchEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SexpLengthError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SEXP` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DuplicateNameError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsJsonPretty<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Factor<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RNGtype` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterComplexData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffTimestampVec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RPrimitive<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `PanicReport<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `FactorVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `List` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ResultShape<S>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `mx_base_vtable` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RSerdeError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SEXPTYPE` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectPool` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RawError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ListBuilder<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AltrepRegRow` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `WorkerPump<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsNamedList<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ClassNameEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StreamingRealData<F>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RBase` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `MatchArgError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypedListError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DataFrame` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Collect<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AltrepSexp` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TlsRoot` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SexpError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RErrorView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectKey` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Sortedness` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `OwnedProtect` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RawSliceTagged<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RWrapperPriority` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `FactorMut<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `NamedVector<M>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ListAccumulator<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RDataFrameBuilder` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ListFromSexpError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SidecarPropEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CoerceError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RngGuard` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsJson<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterRawData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ExternalSlice<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsNamedVector<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `R_CMethodDef` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypedEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffTimestampVecMut` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterState<I, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StreamingIntData<F>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `FactorHandling` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterIntData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SplitShape` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Root<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `VctrsKind` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RHashView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsDisplayVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `NullOnErr` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RSerializer` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterRawData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `REncodingInfo` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `REnv` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RCoerceError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Coerced<T, R>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RawSlice<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `VctrsBuildError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `R_CallMethodDef` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypeSpec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsDataFrame<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RCopyView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `LogicalCoerceError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffZonedVec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AltrepRegistration` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CollectNAInt<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ROrdView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Sampletype` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectScope` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Rboolean` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffTimestampVecRef` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `cetype_t` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsFromStr<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RDisplayView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterRealData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterListData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JsonOptions` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `mx_tag` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsDisplay<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RSidecar` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsSerialize<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SexpNaError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RCall` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `GuardMode` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RndMat<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ListMut` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RStringArray` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ParseStatus` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CaptureGroups` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RRng` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsJsonVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ProtectedStrVec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SplitResults` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsRNative<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ExternalPtr<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `NaHandling` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RawTagged<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CallDefRow` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DispatchNames` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RPartialOrdView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RFromStrView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SEXPREC` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterStringData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsFromStrVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Arena<M>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Protected<''a, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RDeserializer` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RDebugView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CollectNA<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RawHeader` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffZonedVecMut` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RndVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StrVecIter` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterLogicalData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypeMismatchError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RWrapperEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ThreadLocalArena` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Altrep<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RBorrow<''a, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `NamedList` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypedListSpec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DllInfo` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsRError<E>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsExternalPtr<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterIntData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RLogical` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RCloneView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SpecialFloatHandling` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SexpTypeError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DataFrameShape` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `FromJson<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StrVec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `DataFrameError` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Missing<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `IterState<I, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ThreadLocalHashArena` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `mx_erased` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Raw<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Logical` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `PanicSource` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SerdeRows<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TypeSpec` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `TraitDispatchRow` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `StrVecCowIter` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RSymbol` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `CollectStrings<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RCow<''a, T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `JiffZonedVecRef` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Borsh<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsList<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `AsVctrs<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `FactorOptionVec<T>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `N01type` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RDefaultView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `R_altrep_class_t` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RIteratorView` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Rcomplex` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `Dots` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `RAllocator` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `SparseIterRealData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34 |

### `Same` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typenum-1.20.0/src/type_operators.rs:34** (200 impls): `IntoRError`, `TypedList`, `StorageCoerceError`, `RFlags<T>`, `AltrepGuard`, `RThreadBuilder`, `TraitDispatchEntry`, `SexpLengthError`, `SEXP`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `RNGtype`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `FactorVec<T>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `SEXPTYPE`, `ProtectPool`, `RawError`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RBase`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `ProtectKey`, `Sortedness`, `OwnedProtect`, `SerdeRowBuilder<T>`, `RawSliceTagged<T>`, `SparseIterLogicalData<I>`, `RWrapperPriority`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `NamedVector<M>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `ListFromSexpError`, `SidecarPropEntry`, `CoerceError`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `Root<''a>`, `VctrsKind`, `RHashView`, `AsDisplayVec<T>`, `NullOnErr`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `RCoerceError`, `Coerced<T, R>`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `VctrsBuildError`, `R_CallMethodDef`, `TypeSpec`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `LogicalCoerceError`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ROrdView`, `Sampletype`, `ProtectScope`, `Rboolean`, `JiffTimestampVecRef`, `cetype_t`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `JsonOptions`, `mx_tag`, `AsDisplay<T>`, `RSidecar`, `AsSerialize<T>`, `RVecStorage<T, R, C>`, `SexpNaError`, `RCall`, `GuardMode`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `ParseStatus`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `ExternalPtr<T>`, `NaHandling`, `RawTagged<T>`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `Protected<''a, T>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `Altrep<T>`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `SparseIterIntData<I>`, `RLogical`, `RCloneView`, `SpecialFloatHandling`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Missing<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `Raw<T>`, `Logical`, `PanicSource`, `SerdeRows<T>`, `TypeSpec`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `N01type`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`

## `Pointable` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Root<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RHashView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RFlags<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AltrepGuard` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SEXP` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsDisplayVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RNGtype` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StrVecIter` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RSerializer` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterRawData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `REncodingInfo` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `REnv` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SEXPTYPE` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RCoerceError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RawError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `R_CallMethodDef` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsDataFrame<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RCopyView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffZonedVec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AltrepRegistration` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CollectNAInt<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectKey` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ROrdView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Sortedness` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectScope` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffTimestampVecRef` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `cetype_t` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RawSliceTagged<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsFromStr<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RDisplayView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterRealData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterListData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JsonOptions` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsDisplay<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `NamedVector<M>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CoerceError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SexpNaError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RCall` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RndMat<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ExternalPtr<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ListMut` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RStringArray` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CaptureGroups` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsJsonVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectedStrVec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SplitResults` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsRNative<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `NaHandling` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CallDefRow` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Altrep<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DispatchNames` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RPartialOrdView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RFromStrView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SEXPREC` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterStringData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `VctrsKind` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsFromStrVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Arena<M>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Protected<''a, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RDeserializer` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RDebugView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CollectNA<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `NullOnErr` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffZonedVecMut` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RBase` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RndVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterLogicalData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypeMismatchError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RWrapperEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ThreadLocalArena` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RBorrow<''a, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `NamedList` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RawSlice<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypedListSpec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `VctrsBuildError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DllInfo` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypeSpec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsRError<E>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsExternalPtr<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterIntData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `LogicalCoerceError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RCloneView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SpecialFloatHandling` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SexpTypeError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DataFrameShape` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `FromJson<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StrVec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Sampletype` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DataFrameError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Rboolean` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterState<I, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ThreadLocalHashArena` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `mx_erased` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `mx_tag` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RSidecar` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsSerialize<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SerdeRows<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TraitDispatchRow` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `GuardMode` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RSymbol` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RRng` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ParseStatus` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `CollectStrings<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RCow<''a, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffZonedVecRef` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Borsh<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsList<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsVctrs<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `FactorOptionVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RDefaultView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `R_altrep_class_t` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RIteratorView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RawTagged<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Dots` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RAllocator` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterRealData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IntoRError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypedList` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RThreadBuilder` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TraitDispatchEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SexpLengthError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DuplicateNameError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsJsonPretty<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Factor<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterComplexData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Coerced<T, R>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffTimestampVec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RPrimitive<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `PanicReport<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `FactorVec<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `List` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ResultShape<S>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `mx_base_vtable` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RSerdeError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ProtectPool` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ListBuilder<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AltrepRegRow` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `WorkerPump<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsNamedList<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ClassNameEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StreamingRealData<F>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RLogical` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `MatchArgError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypedListError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `DataFrame` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Collect<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AltrepSexp` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TlsRoot` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SexpError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RErrorView` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Missing<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `OwnedProtect` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RWrapperPriority` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Raw<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Logical` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `PanicSource` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypeSpec` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `FactorMut<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ListAccumulator<''a>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RDataFrameBuilder` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StrVecCowIter` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ListFromSexpError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SidecarPropEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RngGuard` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsJson<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterRawData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `ExternalSlice<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `AsNamedVector<T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `R_CMethodDef` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `N01type` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `TypedEntry` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `Rcomplex` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `JiffTimestampVecMut` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SparseIterState<I, T>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StreamingIntData<F>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `FactorHandling` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `IterIntData<I>` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `SplitShape` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `StorageCoerceError` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |
| `RawHeader` | `<T>` | blanket | 6 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194 |

### `Pointable` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-epoch-0.9.18/src/atomic.rs:194** (200 impls): `Root<''a>`, `RHashView`, `RFlags<T>`, `AltrepGuard`, `SEXP`, `AsDisplayVec<T>`, `RNGtype`, `StrVecIter`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `SEXPTYPE`, `RCoerceError`, `RawError`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ProtectKey`, `ROrdView`, `Sortedness`, `ProtectScope`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `JsonOptions`, `AsDisplay<T>`, `NamedVector<M>`, `CoerceError`, `SexpNaError`, `RCall`, `RndMat<T>`, `ExternalPtr<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `RStringArray`, `CaptureGroups`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `Altrep<T>`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `VctrsKind`, `AsFromStrVec<T>`, `Arena<M>`, `Protected<''a, T>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`, `NullOnErr`, `JiffZonedVecMut`, `RBase`, `RndVec<T>`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `TypedListSpec`, `VctrsBuildError`, `DllInfo`, `TypeSpec`, `AsRError<E>`, `AsExternalPtr<T>`, `SparseIterIntData<I>`, `LogicalCoerceError`, `RCloneView`, `SpecialFloatHandling`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `DataFrameError`, `Rboolean`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `RVecStorage<T, R, C>`, `mx_tag`, `RSidecar`, `ReprotectSlot<''a>`, `AsSerialize<T>`, `SerdeRows<T>`, `TraitDispatchRow`, `GuardMode`, `RSymbol`, `RRng`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `RawTagged<T>`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `Coerced<T, R>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `FactorVec<T>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `RWrapperPriority`, `WindowedIterRealData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `StrVecCowIter`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `N01type`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `StorageCoerceError`, `WorkerUnprotectGuard`, `RawHeader`

## `Into` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Factor<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypeMismatchError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RSerializer` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterRawData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `REnv` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `List` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RLogical` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RCopyView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterIntFromBoolData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DataFrame` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AltrepRegistration` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CollectNAInt<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ROrdView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectScope` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `OwnedProtect` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RDisplayView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterRealData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WindowedIterState<I, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterListData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Raw<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Logical` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `PanicSource` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RCall` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RndMat<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterRealCoerceData<I, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ListMut` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsJsonVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `N01type` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SplitResults` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypedEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Rcomplex` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FactorHandling` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CallDefRow` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RPartialOrdView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RFromStrView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RawHeader` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SEXPREC` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterStringData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Arena<M>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RDeserializer` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SEXP` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsDisplayVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RDebugView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CollectNA<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RNGtype` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterIntCoerceData<I, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RndVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterLogicalData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RWrapperEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalArena` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RBorrow<''a, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `NamedList` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StrVecBuilder<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DllInfo` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterIntData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RCloneView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DataFrameShape` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Missing<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FromJson<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectKey` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Sortedness` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `cetype_t` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterState<I, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalHashArena` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `MatchArgChoicesEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `mx_erased` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JsonOptions` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RVecStorage<T, R, C>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ReprotectSlot<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StrVecCowIter` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SerdeRows<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchRow` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RSymbol` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RRng` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RStringArray` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CollectStrings<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Borsh<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RDefaultView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `NaHandling` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RIteratorView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Dots` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `DispatchNames` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RAllocator` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `MatchArgParamDocEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterRealData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WindowedIterIntData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `VctrsKind` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Protected<''a, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StrVecIter` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RThreadBuilder` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsJsonPretty<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `NullOnErr` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterComplexData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `PanicReport<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ResultShape<S>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `mx_base_vtable` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectPool` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ListBuilder<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AltrepRegRow` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WorkerPump<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ClassNameEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StreamingRealData<F>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Collect<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AltrepSexp` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StrVec` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Sampletype` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RErrorView` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Rboolean` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SerdeRowBuilder<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterLogicalData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `mx_tag` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WindowedIterRealData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RSidecar` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ListAccumulator<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RDataFrameBuilder` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `GuardMode` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SidecarPropEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RngGuard` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `NamedDataFrameListBuilder` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ParseStatus` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsJson<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsList<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterRawData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ExternalSlice<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RBase` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterComplexData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SparseIterState<I, T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StreamingIntData<F>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `IterIntData<I>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `SplitShape` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `TypedList` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WorkerUnprotectGuard` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RHashView` | `<T, U> +1wc` | blanket | 1 | (no span) |

### `Into` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `Factor<''a>`, `TypeMismatchError`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `List`, `VctrsBuildError`, `AsRError<E>`, `RLogical`, `RCopyView`, `LogicalCoerceError`, `IterIntFromBoolData<I>`, `DataFrame`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `DataFrameError`, `ROrdView`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `RWrapperPriority`, `WindowedIterState<I, T>`, `IterListData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `RCall`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsJsonVec<T>`, `R_CMethodDef`, `FactorOptionVec<T>`, `N01type`, `ProtectedStrVec`, `SplitResults`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `FactorHandling`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `IntoRError`, `RawHeader`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `AltrepGuard`, `SexpLengthError`, `RDeserializer`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RDebugView`, `CollectNA<I>`, `RNGtype`, `IterIntCoerceData<I, T>`, `Coerced<T, R>`, `RndVec<T>`, `FactorVec<T>`, `REncodingInfo`, `IterLogicalData<I>`, `RWrapperEntry`, `RSerdeError`, `SEXPTYPE`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `AsNamedList<T>`, `DllInfo`, `R_CallMethodDef`, `AsDataFrame<T>`, `SparseIterIntData<I>`, `MatchArgError`, `TypedListError`, `RCloneView`, `TlsRoot`, `SexpError`, `DataFrameShape`, `Missing<T>`, `FromJson<T>`, `ProtectKey`, `Sortedness`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `JsonOptions`, `RVecStorage<T, R, C>`, `AsDisplay<T>`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `ListFromSexpError`, `TraitDispatchRow`, `ExternalPtr<T>`, `RSymbol`, `RRng`, `RStringArray`, `CaptureGroups`, `CollectStrings<I>`, `AsNamedVector<T>`, `Borsh<T>`, `AsRNative<T>`, `Altrep<T>`, `RDefaultView`, `NaHandling`, `RIteratorView`, `Dots`, `DispatchNames`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `StorageCoerceError`, `WindowedIterIntData<I>`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `Protected<''a, T>`, `StrVecIter`, `RThreadBuilder`, `TraitDispatchEntry`, `AsJsonPretty<T>`, `NullOnErr`, `JiffZonedVecMut`, `IterComplexData<I>`, `JiffTimestampVec`, `PanicReport<''a>`, `RCoerceError`, `RawError`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `ListBuilder<''a>`, `TypedListSpec`, `AltrepRegRow`, `TypeSpec`, `WorkerPump<T>`, `AsExternalPtr<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `SexpTypeError`, `SpecialFloatHandling`, `Collect<I>`, `AltrepSexp`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `RErrorView`, `Rboolean`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `mx_tag`, `WindowedIterRealData<I>`, `NamedVector<M>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `GuardMode`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsJson<T>`, `AsList<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsVctrs<T>`, `RBase`, `SparseIterComplexData<I>`, `R_altrep_class_t`, `RawTagged<T>`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `IterIntData<I>`, `SplitShape`, `TypedList`, `WorkerUnprotectGuard`, `RFlags<T>`, `RHashView`

## `Pipe` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SparseIterRealData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `WindowedIterIntData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IntoRError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypedList` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RThreadBuilder` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TraitDispatchEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Protected<''a, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SexpLengthError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsJsonPretty<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Factor<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterComplexData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `PanicReport<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `List` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ResultShape<S>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `mx_base_vtable` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RSerdeError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectPool` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ListBuilder<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AltrepRegRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ClassNameEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StreamingRealData<F>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RLogical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `MatchArgError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypedListError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DataFrame` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Collect<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AltrepSexp` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TlsRoot` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SexpError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RErrorView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Missing<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `OwnedProtect` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SerdeRowBuilder<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SparseIterLogicalData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `WindowedIterRealData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `FactorMut<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ListAccumulator<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RDataFrameBuilder` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Raw<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Logical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RWrapperPriority` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `PanicSource` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SidecarPropEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RngGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `NamedDataFrameListBuilder` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsJson<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterRawData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ExternalSlice<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `R_CMethodDef` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SparseIterComplexData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypedEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `N01type` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffTimestampVecMut` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SparseIterState<I, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StreamingIntData<F>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `FactorHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterIntData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SplitShape` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `WorkerUnprotectGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Root<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RHashView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AltrepGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SEXP` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RNGtype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RSerializer` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SparseIterRawData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `REncodingInfo` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `REnv` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `FactorVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SEXPTYPE` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RawError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectedStrVecIter<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RCopyView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterIntFromBoolData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffZonedVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AltrepRegistration` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CollectNAInt<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ROrdView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectScope` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectKey` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffTimestampVecRef` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `cetype_t` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Sortedness` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RDisplayView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterRealData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `WindowedIterState<I, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterListData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JsonOptions` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RVecStorage<T, R, C>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `NamedVector<M>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SexpNaError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RCall` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RndMat<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterRealCoerceData<I, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ListMut` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectedStrVecCowIter<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RStringArray` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CaptureGroups` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RRng` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsJsonVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ProtectedStrVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SplitResults` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsRNative<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `NaHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CallDefRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DispatchNames` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RPartialOrdView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RFromStrView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SEXPREC` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterStringData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Arena<M>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `VctrsKind` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RDeserializer` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RDebugView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CollectNA<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RFlags<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterIntCoerceData<I, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RawHeader` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffZonedVecMut` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RndVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `NullOnErr` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StrVecIter` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterLogicalData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RWrapperEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ThreadLocalArena` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RBorrow<''a, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `NamedList` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StrVecBuilder<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypedListSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DllInfo` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsRError<E>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RawSlice<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SparseIterIntData<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Altrep<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RCloneView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SexpTypeError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DataFrameShape` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `FromJson<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RBase` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StrVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ArenaGuard<''a, M>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `DataFrameError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Sampletype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `IterState<I, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Rboolean` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ThreadLocalHashArena` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `MatchArgChoicesEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `mx_erased` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ReprotectSlot<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `mx_tag` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `SerdeRows<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RSidecar` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `TraitDispatchRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `StrVecCowIter` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RSymbol` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `GuardMode` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `CollectStrings<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RCow<''a, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `JiffZonedVecRef` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ParseStatus` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Borsh<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsList<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RDefaultView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RIteratorView` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Rcomplex` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `Dots` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RAllocator` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |
| `MatchArgParamDocEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234 |

### `Pipe` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/pipe.rs:234** (200 impls): `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `Protected<''a, T>`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `N01type`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `Root<''a>`, `StorageCoerceError`, `RHashView`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `ProtectedStrVecIter<''a>`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ROrdView`, `ProtectScope`, `ProtectKey`, `JiffTimestampVecRef`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `RVecStorage<T, R, C>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `CoerceError`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `VctrsKind`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `RFlags<T>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `NullOnErr`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `SparseIterIntData<I>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `RCloneView`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `RBase`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Sampletype`, `IterState<I, T>`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `ExternalPtr<T>`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`

## `FmtForward` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CallDefRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DispatchNames` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RPartialOrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RFromStrView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SEXPREC` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterStringData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsFromStrVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Arena<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `VctrsKind` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RDeserializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RDebugView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `CollectNA<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RFlags<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RawHeader` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffZonedVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RndVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `NullOnErr` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StrVecIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypeMismatchError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RWrapperEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ThreadLocalArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RBorrow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `NamedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypedListSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DllInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsRError<E>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RawSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `VctrsBuildError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Altrep<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Coerced<T, R>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RCloneView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SpecialFloatHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `LogicalCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SexpTypeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DataFrameShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `FromJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DataFrameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Sampletype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Rboolean` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ThreadLocalHashArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `mx_erased` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `mx_tag` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SerdeRows<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RSidecar` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TraitDispatchRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StrVecCowIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RSymbol` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `GuardMode` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `CollectStrings<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RCow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffZonedVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ParseStatus` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Borsh<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsVctrs<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `FactorOptionVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RDefaultView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `R_altrep_class_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RIteratorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Rcomplex` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Dots` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RAllocator` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RawTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IntoRError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RThreadBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TraitDispatchEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Protected<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SexpLengthError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DuplicateNameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsJsonPretty<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Factor<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffTimestampVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RPrimitive<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `PanicReport<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `List` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ResultShape<S>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `mx_base_vtable` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RSerdeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectPool` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ListBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AltrepRegRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `WorkerPump<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsNamedList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ClassNameEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StreamingRealData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RLogical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `MatchArgError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypedListError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `DataFrame` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Collect<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AltrepSexp` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TlsRoot` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RErrorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RBase` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Missing<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `OwnedProtect` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `FactorMut<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ListAccumulator<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RDataFrameBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Raw<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Logical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RWrapperPriority` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `PanicSource` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ListFromSexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SidecarPropEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RngGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ExternalSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsNamedVector<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `R_CMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `TypedEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `N01type` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffTimestampVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StreamingIntData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `FactorHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SplitShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Root<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `StorageCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RHashView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AltrepGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsDisplayVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SEXP` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RNGtype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RSerializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SparseIterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `REncodingInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `REnv` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `FactorVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SEXPTYPE` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RawError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `R_CallMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsDataFrame<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RCopyView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffZonedVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AltrepRegistration` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `CollectNAInt<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ROrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectScope` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectKey` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JiffTimestampVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `cetype_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `Sortedness` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsFromStr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RDisplayView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterListData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RawSliceTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `JsonOptions` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsDisplay<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `NamedVector<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsSerialize<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SexpNaError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RCall` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `CoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RndMat<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ListMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RStringArray` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `CaptureGroups` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `RRng` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsJsonVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `ProtectedStrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `SplitResults` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `AsRNative<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |
| `NaHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114 |

### `FmtForward` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wyz-0.5.1/src/fmt.rs:114** (200 impls): `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `VctrsKind`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `RFlags<T>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `NullOnErr`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `SparseIterIntData<I>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `RCloneView`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Sampletype`, `IterState<I, T>`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `ExternalPtr<T>`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `Protected<''a, T>`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `RBase`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `N01type`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `Root<''a>`, `StorageCoerceError`, `RHashView`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `ProtectedStrVecIter<''a>`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ROrdView`, `ProtectScope`, `ProtectKey`, `JiffTimestampVecRef`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `RVecStorage<T, R, C>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `CoerceError`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`

## `Any` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Protected<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDeserializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXP` | `<T> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDebugView` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNA<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RNGtype` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBorrow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `DllInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCloneView` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `FromJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sortedness` | `<T> +1wc` | blanket | 1 | (no span) |
| `cetype_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ThreadLocalHashArena` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgChoicesEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_erased` | `<T> +1wc` | blanket | 1 | (no span) |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | (no span) |
| `RVecStorage<T, R, C>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ReprotectSlot<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecCowIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRows<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSymbol` | `<T> +1wc` | blanket | 1 | (no span) |
| `RRng` | `<T> +1wc` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectStrings<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `Borsh<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDefaultView` | `<T> +1wc` | blanket | 1 | (no span) |
| `NaHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `RIteratorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Dots` | `<T> +1wc` | blanket | 1 | (no span) |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | (no span) |
| `RAllocator` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgParamDocEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVecIter` | `<T> +1wc` | blanket | 1 | (no span) |
| `RThreadBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `TraitDispatchEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJsonPretty<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Factor<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NullOnErr` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicReport<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ResultShape<S>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_base_vtable` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectPool` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListBuilder<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ClassNameEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingRealData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `Collect<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepSexp` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sampletype` | `<T> +1wc` | blanket | 1 | (no span) |
| `RErrorView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rboolean` | `<T> +1wc` | blanket | 1 | (no span) |
| `OwnedProtect` | `<T> +1wc` | blanket | 1 | (no span) |
| `SerdeRowBuilder<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterLogicalData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_tag` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSidecar` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListAccumulator<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDataFrameBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `GuardMode` | `<T> +1wc` | blanket | 1 | (no span) |
| `SidecarPropEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `RngGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedDataFrameListBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ParseStatus` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJson<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterComplexData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `StreamingIntData<F>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitShape` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBase` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkerUnprotectGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `RHashView` | `<T> +1wc` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerializer` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SparseIterRawData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REnv` | `<T> +1wc` | blanket | 1 | (no span) |
| `List` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RLogical` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCopyView` | `<T> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterIntFromBoolData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrame` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffZonedVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepRegistration` | `<T> +1wc` | blanket | 1 | (no span) |
| `CollectNAInt<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ROrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectScope` | `<T> +1wc` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<T> +1wc` | blanket | 1 | (no span) |
| `RDisplayView` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | (no span) |
| `WindowedIterState<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterListData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Raw<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Logical` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicSource` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCall` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RndMat<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterRealCoerceData<I, T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListMut` | `<T> +1wc` | blanket | 1 | (no span) |
| `RStringArray` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsJsonVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `N01type` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectedStrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `SplitResults` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rcomplex` | `<T> +1wc` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `CallDefRow` | `<T> +1wc` | blanket | 1 | (no span) |
| `RPartialOrdView` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFromStrView` | `<T> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawHeader` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPREC` | `<T> +1wc` | blanket | 1 | (no span) |
| `IterStringData<I>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Arena<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T> +1wc` | blanket | 1 | (no span) |

### `Any` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `Protected<''a, T>`, `SexpLengthError`, `RDeserializer`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RDebugView`, `CollectNA<I>`, `RNGtype`, `IterIntCoerceData<I, T>`, `JiffZonedVecMut`, `RndVec<T>`, `REncodingInfo`, `IterLogicalData<I>`, `RWrapperEntry`, `RSerdeError`, `SEXPTYPE`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `DllInfo`, `R_CallMethodDef`, `AsDataFrame<T>`, `SparseIterIntData<I>`, `MatchArgError`, `TypedListError`, `RCloneView`, `SexpError`, `DataFrameShape`, `FromJson<T>`, `ProtectKey`, `ArenaGuard<''a, M>`, `Sortedness`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `JsonOptions`, `RVecStorage<T, R, C>`, `AsDisplay<T>`, `NamedVector<M>`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `ListFromSexpError`, `TraitDispatchRow`, `RSymbol`, `RRng`, `CaptureGroups`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `Borsh<T>`, `AsRNative<T>`, `RDefaultView`, `NaHandling`, `RIteratorView`, `Altrep<T>`, `Dots`, `DispatchNames`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `StorageCoerceError`, `WindowedIterIntData<I>`, `VctrsKind`, `AsFromStrVec<T>`, `RFlags<T>`, `StrVecIter`, `RThreadBuilder`, `TraitDispatchEntry`, `AsJsonPretty<T>`, `Factor<''a>`, `NullOnErr`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `RCoerceError`, `RawError`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `ListBuilder<''a>`, `TypedListSpec`, `AltrepRegRow`, `TypeSpec`, `AsRError<E>`, `WorkerPump<T>`, `AsExternalPtr<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `SexpTypeError`, `SpecialFloatHandling`, `Collect<I>`, `AltrepSexp`, `StrVec`, `Sampletype`, `RErrorView`, `Rboolean`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `mx_tag`, `WindowedIterRealData<I>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `FactorMut<''a>`, `SexpNaError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `GuardMode`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `AsJson<T>`, `AsList<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `SparseIterComplexData<I>`, `R_altrep_class_t`, `RawTagged<T>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `IterIntData<I>`, `SplitShape`, `RBase`, `TypedList`, `WorkerUnprotectGuard`, `RHashView`, `Coerced<T, R>`, `TypeMismatchError`, `RSerializer`, `FactorVec<T>`, `SparseIterRawData<I>`, `REnv`, `List`, `VctrsBuildError`, `AsNamedList<T>`, `RLogical`, `RCopyView`, `LogicalCoerceError`, `IterIntFromBoolData<I>`, `DataFrame`, `JiffZonedVec`, `TlsRoot`, `AltrepRegistration`, `CollectNAInt<I>`, `DataFrameError`, `Missing<T>`, `ROrdView`, `ProtectScope`, `JiffTimestampVecRef`, `RDisplayView`, `IterRealData<I>`, `RWrapperPriority`, `WindowedIterState<I, T>`, `IterListData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `RCall`, `ExternalPtr<T>`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `RStringArray`, `AsNamedVector<T>`, `AsJsonVec<T>`, `R_CMethodDef`, `N01type`, `ProtectedStrVec`, `SplitResults`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `FactorHandling`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `IntoRError`, `RawHeader`, `SEXPREC`, `IterStringData<I>`, `Root<''a>`, `Arena<M>`, `AltrepGuard`

## `SupersetOf` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsFromStrVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Arena<M>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Protected<''a, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RDeserializer` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RDebugView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CollectNA<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterIntCoerceData<I, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffZonedVecMut` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RndVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StrVecIter` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterLogicalData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypeMismatchError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RWrapperEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ThreadLocalArena` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Altrep<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RBorrow<''a, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `NamedList` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StrVecBuilder<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypedListSpec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DllInfo` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsRError<E>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsExternalPtr<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterIntData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RLogical` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RCloneView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SpecialFloatHandling` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SexpTypeError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DataFrameShape` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `FromJson<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StrVec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ArenaGuard<''a, M>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DataFrameError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Missing<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterState<I, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ThreadLocalHashArena` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `MatchArgChoicesEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `mx_erased` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ReprotectSlot<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Raw<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Logical` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `PanicSource` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SerdeRows<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypeSpec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TraitDispatchRow` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StrVecCowIter` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RSymbol` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CollectStrings<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RCow<''a, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffZonedVecRef` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Borsh<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsList<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsVctrs<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `FactorOptionVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `N01type` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RDefaultView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `R_altrep_class_t` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RIteratorView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Rcomplex` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Dots` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RAllocator` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `MatchArgParamDocEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterRealData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `WindowedIterIntData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IntoRError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypedList` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StorageCoerceError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RFlags<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AltrepGuard` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RThreadBuilder` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TraitDispatchEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SexpLengthError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SEXP` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DuplicateNameError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsJsonPretty<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Factor<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RNGtype` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterComplexData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffTimestampVec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RPrimitive<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `PanicReport<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `FactorVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `List` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ResultShape<S>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `mx_base_vtable` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RSerdeError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SEXPTYPE` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectPool` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RawError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ListBuilder<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RBase` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AltrepRegRow` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `WorkerPump<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsNamedList<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ClassNameEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StreamingRealData<F>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `MatchArgError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypedListError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DataFrame` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Collect<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AltrepSexp` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TlsRoot` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SexpError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RErrorView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectKey` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Sortedness` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `OwnedProtect` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SerdeRowBuilder<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RawSliceTagged<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterLogicalData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RWrapperPriority` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `WindowedIterRealData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `FactorMut<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `NamedVector<M>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ListAccumulator<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RDataFrameBuilder` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ListFromSexpError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SidecarPropEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CoerceError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RngGuard` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `NamedDataFrameListBuilder` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsJson<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterRawData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ExternalSlice<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsNamedVector<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `R_CMethodDef` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterComplexData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypedEntry` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffTimestampVecMut` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterState<I, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `StreamingIntData<F>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `FactorHandling` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterIntData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SplitShape` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `WorkerUnprotectGuard` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RawHeader` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Root<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `VctrsKind` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RHashView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsDisplayVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `NullOnErr` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RSerializer` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SparseIterRawData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `REncodingInfo` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `REnv` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RCoerceError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Coerced<T, R>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectedStrVecIter<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RawSlice<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `VctrsBuildError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `R_CallMethodDef` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `TypeSpec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsDataFrame<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RCopyView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterIntFromBoolData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `LogicalCoerceError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffZonedVec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AltrepRegistration` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CollectNAInt<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ROrdView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Sampletype` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectScope` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `Rboolean` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JiffTimestampVecRef` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `cetype_t` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsFromStr<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RDisplayView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterRealData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `WindowedIterState<I, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterListData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `JsonOptions` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `mx_tag` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsDisplay<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RSidecar` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsSerialize<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RVecStorage<T, R, C>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SexpNaError` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RCall` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `GuardMode` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RndMat<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterRealCoerceData<I, T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ListMut` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectedStrVecCowIter<''a>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RStringArray` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ParseStatus` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CaptureGroups` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RRng` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsJsonVec<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ProtectedStrVec` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SplitResults` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `AsRNative<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `ExternalPtr<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `NaHandling` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RawTagged<T>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `CallDefRow` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `DispatchNames` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RPartialOrdView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `RFromStrView` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `SEXPREC` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |
| `IterStringData<I>` | `<SS, SP> +1wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90 |

### `SupersetOf` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/simba-0.9.1/src/scalar/subset.rs:90** (200 impls): `AsFromStrVec<T>`, `Arena<M>`, `Protected<''a, T>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`, `JiffZonedVecMut`, `RndVec<T>`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `Altrep<T>`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `SparseIterIntData<I>`, `RLogical`, `RCloneView`, `SpecialFloatHandling`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Missing<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `Raw<T>`, `Logical`, `PanicSource`, `SerdeRows<T>`, `TypeSpec`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `N01type`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `StorageCoerceError`, `RFlags<T>`, `AltrepGuard`, `RThreadBuilder`, `TraitDispatchEntry`, `SexpLengthError`, `SEXP`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `RNGtype`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `FactorVec<T>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `SEXPTYPE`, `ProtectPool`, `RawError`, `ListBuilder<''a>`, `RBase`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `ProtectKey`, `Sortedness`, `OwnedProtect`, `SerdeRowBuilder<T>`, `RawSliceTagged<T>`, `SparseIterLogicalData<I>`, `RWrapperPriority`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `NamedVector<M>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `ListFromSexpError`, `SidecarPropEntry`, `CoerceError`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `RawHeader`, `Root<''a>`, `VctrsKind`, `RHashView`, `AsDisplayVec<T>`, `NullOnErr`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `RCoerceError`, `Coerced<T, R>`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `VctrsBuildError`, `R_CallMethodDef`, `TypeSpec`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `LogicalCoerceError`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ROrdView`, `Sampletype`, `ProtectScope`, `Rboolean`, `JiffTimestampVecRef`, `cetype_t`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `JsonOptions`, `mx_tag`, `AsDisplay<T>`, `RSidecar`, `AsSerialize<T>`, `RVecStorage<T, R, C>`, `SexpNaError`, `RCall`, `GuardMode`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `ParseStatus`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `ExternalPtr<T>`, `NaHandling`, `RawTagged<T>`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`

## `Conv` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `WorkerUnprotectGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Root<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StorageCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RHashView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AltrepGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsDisplayVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SEXP` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RNGtype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RSerializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `REncodingInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `REnv` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `FactorVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SEXPTYPE` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RawError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `R_CallMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsDataFrame<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RCopyView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffZonedVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AltrepRegistration` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RBase` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CollectNAInt<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ROrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectScope` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectKey` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffTimestampVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `cetype_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Sortedness` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsFromStr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RDisplayView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterListData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RawSliceTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JsonOptions` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsDisplay<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `NamedVector<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsSerialize<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SexpNaError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RCall` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RndMat<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ListMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RStringArray` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CaptureGroups` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RRng` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsJsonVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectedStrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SplitResults` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsRNative<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `NaHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CallDefRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DispatchNames` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RPartialOrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RFromStrView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SEXPREC` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterStringData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsFromStrVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Arena<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `VctrsKind` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RDeserializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RDebugView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CollectNA<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RFlags<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RawHeader` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffZonedVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RndVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `NullOnErr` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StrVecIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypeMismatchError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RWrapperEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ThreadLocalArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RBorrow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `NamedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypedListSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DllInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsRError<E>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RawSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `VctrsBuildError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Altrep<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Coerced<T, R>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RCloneView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SpecialFloatHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `LogicalCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SexpTypeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DataFrameShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `FromJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DataFrameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Sampletype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Rboolean` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ThreadLocalHashArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `mx_erased` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `mx_tag` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SerdeRows<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RSidecar` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TraitDispatchRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StrVecCowIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RSymbol` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `GuardMode` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `CollectStrings<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RCow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffZonedVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ParseStatus` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Borsh<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsVctrs<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `FactorOptionVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RDefaultView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `R_altrep_class_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RIteratorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Rcomplex` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Dots` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RAllocator` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RawTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IntoRError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RThreadBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TraitDispatchEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Protected<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SexpLengthError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DuplicateNameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsJsonPretty<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Factor<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffTimestampVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RPrimitive<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `PanicReport<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `List` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ResultShape<S>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `mx_base_vtable` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RSerdeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ProtectPool` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ListBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AltrepRegRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `WorkerPump<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsNamedList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ClassNameEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StreamingRealData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RLogical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `MatchArgError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypedListError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `DataFrame` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Collect<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AltrepSexp` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TlsRoot` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RErrorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Missing<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `OwnedProtect` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `FactorMut<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ListAccumulator<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RDataFrameBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Raw<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `Logical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RWrapperPriority` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `PanicSource` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ListFromSexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SidecarPropEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RngGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `ExternalSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `AsNamedVector<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `R_CMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `TypedEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `N01type` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `JiffTimestampVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SparseIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `StreamingIntData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `FactorHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `IterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |
| `SplitShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58 |

### `Conv` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:58** (200 impls): `WorkerUnprotectGuard`, `Root<''a>`, `StorageCoerceError`, `RHashView`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `ProtectedStrVecIter<''a>`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `RBase`, `CollectNAInt<I>`, `ROrdView`, `ProtectScope`, `ProtectKey`, `JiffTimestampVecRef`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `RVecStorage<T, R, C>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `CoerceError`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `VctrsKind`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `RFlags<T>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `NullOnErr`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `SparseIterIntData<I>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `RCloneView`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Sampletype`, `IterState<I, T>`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `ExternalPtr<T>`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `Protected<''a, T>`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `N01type`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`

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

## `IntoEither` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RThreadBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TraitDispatchEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SexpLengthError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DuplicateNameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsJsonPretty<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Factor<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Coerced<T, R>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffTimestampVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RPrimitive<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `PanicReport<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `FactorVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `List` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ResultShape<S>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `mx_base_vtable` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RSerdeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectPool` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ListBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AltrepRegRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `WorkerPump<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsNamedList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ClassNameEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StreamingRealData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RLogical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `MatchArgError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypedListError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DataFrame` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Collect<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AltrepSexp` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TlsRoot` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RErrorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Missing<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `OwnedProtect` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RWrapperPriority` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Raw<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Logical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `PanicSource` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `FactorMut<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ListAccumulator<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RDataFrameBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StrVecCowIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ListFromSexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SidecarPropEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RngGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ExternalSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsNamedVector<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `R_CMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `N01type` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypedEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Rcomplex` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffTimestampVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StreamingIntData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `FactorHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SplitShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StorageCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RawHeader` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Root<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RHashView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RFlags<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AltrepGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RBase` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SEXP` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsDisplayVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RNGtype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StrVecIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RSerializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `REncodingInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `REnv` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SEXPTYPE` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RawError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `R_CallMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsDataFrame<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RCopyView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffZonedVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AltrepRegistration` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CollectNAInt<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectKey` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ROrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Sortedness` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectScope` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffTimestampVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `cetype_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RawSliceTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsFromStr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RDisplayView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterListData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JsonOptions` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsDisplay<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `NamedVector<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SexpNaError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RCall` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RndMat<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ListMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RStringArray` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CaptureGroups` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsJsonVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectedStrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SplitResults` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsRNative<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `NaHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CallDefRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Altrep<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DispatchNames` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RPartialOrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RFromStrView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SEXPREC` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterStringData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `VctrsKind` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsFromStrVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Arena<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Protected<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RDeserializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RDebugView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CollectNA<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `NullOnErr` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffZonedVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RndVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypeMismatchError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RWrapperEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ThreadLocalArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RBorrow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `NamedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RawSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypedListSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `VctrsBuildError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DllInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsRError<E>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `LogicalCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RCloneView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SpecialFloatHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SexpTypeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DataFrameShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `FromJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `StrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Sampletype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `DataFrameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Rboolean` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ThreadLocalHashArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `mx_erased` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `mx_tag` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RSidecar` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsSerialize<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SerdeRows<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TraitDispatchRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `GuardMode` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RSymbol` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RRng` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `ParseStatus` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `CollectStrings<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RCow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `JiffZonedVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Borsh<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `AsVctrs<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `FactorOptionVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RDefaultView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `R_altrep_class_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RIteratorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RawTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `Dots` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `RAllocator` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `SparseIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `IntoRError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |
| `TypedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64 |

### `IntoEither` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/either-1.15.0/src/into_either.rs:64** (200 impls): `RThreadBuilder`, `TraitDispatchEntry`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `Coerced<T, R>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `FactorVec<T>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `RWrapperPriority`, `WindowedIterRealData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `StrVecCowIter`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `N01type`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `StorageCoerceError`, `WorkerUnprotectGuard`, `RawHeader`, `Root<''a>`, `RHashView`, `RFlags<T>`, `AltrepGuard`, `RBase`, `SEXP`, `AsDisplayVec<T>`, `RNGtype`, `StrVecIter`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `SEXPTYPE`, `RCoerceError`, `RawError`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ProtectKey`, `ROrdView`, `Sortedness`, `ProtectScope`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `JsonOptions`, `AsDisplay<T>`, `NamedVector<M>`, `CoerceError`, `SexpNaError`, `RCall`, `RndMat<T>`, `ExternalPtr<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `RStringArray`, `CaptureGroups`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `Altrep<T>`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`, `SEXPREC`, `IterStringData<I>`, `VctrsKind`, `AsFromStrVec<T>`, `Arena<M>`, `Protected<''a, T>`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `IterIntCoerceData<I, T>`, `NullOnErr`, `JiffZonedVecMut`, `RndVec<T>`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `TypedListSpec`, `VctrsBuildError`, `DllInfo`, `TypeSpec`, `AsRError<E>`, `AsExternalPtr<T>`, `SparseIterIntData<I>`, `LogicalCoerceError`, `RCloneView`, `SpecialFloatHandling`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `DataFrameError`, `Rboolean`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `RVecStorage<T, R, C>`, `mx_tag`, `RSidecar`, `ReprotectSlot<''a>`, `AsSerialize<T>`, `SerdeRows<T>`, `TraitDispatchRow`, `GuardMode`, `RSymbol`, `RRng`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `RawTagged<T>`, `Dots`, `RAllocator`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`

## `TryConv` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SEXPREC` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterStringData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsFromStrVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Arena<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `VctrsKind` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RDeserializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RDebugView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CollectNA<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RFlags<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterIntCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RawHeader` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffZonedVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RndVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `NullOnErr` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StrVecIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypeMismatchError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RWrapperEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ThreadLocalArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RBorrow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `NamedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StrVecBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypedListSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DllInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsRError<E>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RawSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `VctrsBuildError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Altrep<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Coerced<T, R>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RCloneView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SpecialFloatHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `LogicalCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SexpTypeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DataFrameShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `FromJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ArenaGuard<''a, M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DataFrameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Sampletype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Rboolean` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ThreadLocalHashArena` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `MatchArgChoicesEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `mx_erased` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ReprotectSlot<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `mx_tag` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SerdeRows<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypeSpec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RSidecar` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TraitDispatchRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StrVecCowIter` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RSymbol` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `GuardMode` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CollectStrings<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RCow<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffZonedVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ParseStatus` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Borsh<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsVctrs<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `FactorOptionVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RDefaultView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `R_altrep_class_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RIteratorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Rcomplex` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Dots` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ExternalPtr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RAllocator` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RawTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `MatchArgParamDocEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `WindowedIterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IntoRError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypedList` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RThreadBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TraitDispatchEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Protected<''a, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SexpLengthError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DuplicateNameError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsJsonPretty<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Factor<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffTimestampVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RPrimitive<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `PanicReport<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `List` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ResultShape<S>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `mx_base_vtable` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RSerdeError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectPool` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ListBuilder<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AltrepRegRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `WorkerPump<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsNamedList<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ClassNameEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StreamingRealData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RLogical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `MatchArgError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypedListError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DataFrame` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Collect<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AltrepSexp` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TlsRoot` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RBase` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RErrorView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Missing<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `OwnedProtect` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SerdeRowBuilder<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterLogicalData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `WindowedIterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `FactorMut<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ListAccumulator<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RDataFrameBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Raw<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Logical` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RWrapperPriority` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `PanicSource` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ListFromSexpError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SidecarPropEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RngGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `NamedDataFrameListBuilder` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsJson<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ExternalSlice<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsNamedVector<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `R_CMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterComplexData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `TypedEntry` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `N01type` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffTimestampVecMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StreamingIntData<F>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `FactorHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterIntData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SplitShape` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `WorkerUnprotectGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Root<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `StorageCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RHashView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AltrepGuard` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsDisplayVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SEXP` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RNGtype` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RSerializer` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SparseIterRawData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `REncodingInfo` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `REnv` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `FactorVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RCoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SEXPTYPE` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RawError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectedStrVecIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `R_CallMethodDef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsDataFrame<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RCopyView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterIntFromBoolData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffZonedVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AltrepRegistration` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CollectNAInt<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ROrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectScope` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectKey` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JiffTimestampVecRef` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `cetype_t` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `Sortedness` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsFromStr<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RDisplayView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterRealData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `WindowedIterState<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterListData<I>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RawSliceTagged<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `JsonOptions` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsDisplay<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RVecStorage<T, R, C>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `NamedVector<M>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsSerialize<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SexpNaError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RCall` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CoerceError` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RndMat<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `IterRealCoerceData<I, T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ListMut` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectedStrVecCowIter<''a>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RStringArray` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CaptureGroups` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RRng` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsJsonVec<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `ProtectedStrVec` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `SplitResults` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `AsRNative<T>` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `NaHandling` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `CallDefRow` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `DispatchNames` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RPartialOrdView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |
| `RFromStrView` | `<T>` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87 |

### `TryConv` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tap-1.0.1/src/conv.rs:87** (200 impls): `SEXPREC`, `IterStringData<I>`, `AsFromStrVec<T>`, `Arena<M>`, `VctrsKind`, `RDeserializer`, `RDebugView`, `CollectNA<I>`, `RFlags<T>`, `IterIntCoerceData<I, T>`, `RawHeader`, `JiffZonedVecMut`, `RndVec<T>`, `NullOnErr`, `StrVecIter`, `IterLogicalData<I>`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `SparseIterIntData<I>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `RCloneView`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `StrVec`, `ArenaGuard<''a, M>`, `DataFrameError`, `Sampletype`, `IterState<I, T>`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `ReprotectSlot<''a>`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `StrVecCowIter`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `JiffZonedVecRef`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `RDefaultView`, `R_altrep_class_t`, `RIteratorView`, `Rcomplex`, `Dots`, `ExternalPtr<T>`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `Protected<''a, T>`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `IterComplexData<I>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `ProtectPool`, `ListBuilder<''a>`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `AltrepSexp`, `TlsRoot`, `RBase`, `SexpError`, `RErrorView`, `Missing<T>`, `OwnedProtect`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `FactorMut<''a>`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `AsJson<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsNamedVector<T>`, `R_CMethodDef`, `SparseIterComplexData<I>`, `TypedEntry`, `N01type`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `FactorHandling`, `IterIntData<I>`, `SplitShape`, `WorkerUnprotectGuard`, `Root<''a>`, `StorageCoerceError`, `RHashView`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `SparseIterRawData<I>`, `REncodingInfo`, `REnv`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `ProtectedStrVecIter<''a>`, `R_CallMethodDef`, `AsDataFrame<T>`, `RCopyView`, `IterIntFromBoolData<I>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ROrdView`, `ProtectScope`, `ProtectKey`, `JiffTimestampVecRef`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RDisplayView`, `IterRealData<I>`, `WindowedIterState<I, T>`, `IterListData<I>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `RVecStorage<T, R, C>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `CoerceError`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `ProtectedStrVecCowIter<''a>`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `ProtectedStrVec`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `DispatchNames`, `RPartialOrdView`, `RFromStrView`

## `TryFrom` — 200 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Factor<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeMismatchError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RPrimitive<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSerializer` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterRawData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REnv` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `List` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `VctrsBuildError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsNamedList<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RLogical` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCopyView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LogicalCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntFromBoolData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrame` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepRegistration` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectNAInt<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrameError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Missing<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ROrdView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectScope` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `OwnedProtect` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDisplayView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RWrapperPriority` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterListData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Raw<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Logical` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `PanicSource` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorMut<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCall` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RndMat<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRealCoerceData<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsNamedVector<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJsonVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_CMethodDef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorOptionVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `N01type` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SplitResults` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Rcomplex` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVecMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CallDefRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RPartialOrdView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RFromStrView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IntoRError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawHeader` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXPREC` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterStringData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Arena<M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpLengthError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDeserializer` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXP` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DuplicateNameError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDisplayVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDebugView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectNA<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RNGtype` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntCoerceData<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Coerced<T, R>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RndVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FactorVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REncodingInfo` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterLogicalData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RWrapperEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSerdeError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SEXPTYPE` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ThreadLocalArena` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RBorrow<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedList` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecBuilder<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DllInfo` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_CallMethodDef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDataFrame<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedListError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCloneView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TlsRoot` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DataFrameShape` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FromJson<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectKey` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Sortedness` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVecRef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `cetype_t` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawSliceTagged<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsFromStr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ThreadLocalHashArena` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgChoicesEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_erased` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JsonOptions` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RVecStorage<T, R, C>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsDisplay<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedVector<M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ReprotectSlot<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecCowIter` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SerdeRows<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListFromSexpError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TraitDispatchRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ExternalPtr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSymbol` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RRng` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RStringArray` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CaptureGroups` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CollectStrings<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Borsh<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsRNative<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDefaultView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NaHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RIteratorView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Dots` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `DispatchNames` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RAllocator` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MatchArgParamDocEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RBase` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StorageCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Root<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `VctrsKind` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsFromStrVec<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Protected<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVecIter` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RThreadBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TraitDispatchEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJsonPretty<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NullOnErr` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVecMut` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterComplexData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffTimestampVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `PanicReport<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ResultShape<S>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_base_vtable` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectPool` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVecIter<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawSlice<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListBuilder<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedListSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepRegRow` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypeSpec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsRError<E>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkerPump<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsExternalPtr<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ClassNameEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StreamingRealData<F>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpTypeError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SpecialFloatHandling` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Collect<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AltrepSexp` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StrVec` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Sampletype` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ArenaGuard<''a, M>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RErrorView` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Rboolean` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SerdeRowBuilder<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterLogicalData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `mx_tag` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WindowedIterRealData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RSidecar` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsSerialize<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CoerceError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SexpNaError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ListAccumulator<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RDataFrameBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `GuardMode` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SidecarPropEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RngGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `NamedDataFrameListBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ParseStatus` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RCow<''a, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `JiffZonedVecRef` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsJson<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsList<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterRawData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ExternalSlice<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AsVctrs<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterComplexData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Altrep<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `R_altrep_class_t` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RawTagged<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SparseIterState<I, T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StreamingIntData<F>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `IterIntData<I>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `SplitShape` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `TypedList` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkerUnprotectGuard` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RFlags<T>` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RHashView` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryFrom` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (200 impls): `Factor<''a>`, `TypeMismatchError`, `RPrimitive<T>`, `RSerializer`, `SparseIterRawData<I>`, `REnv`, `List`, `VctrsBuildError`, `AsNamedList<T>`, `RLogical`, `RCopyView`, `LogicalCoerceError`, `IterIntFromBoolData<I>`, `DataFrame`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `DataFrameError`, `Missing<T>`, `ROrdView`, `ProtectScope`, `OwnedProtect`, `RDisplayView`, `IterRealData<I>`, `RWrapperPriority`, `WindowedIterState<I, T>`, `IterListData<I>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `FactorMut<''a>`, `RCall`, `RndMat<T>`, `IterRealCoerceData<I, T>`, `ListMut`, `AsNamedVector<T>`, `AsJsonVec<T>`, `R_CMethodDef`, `FactorOptionVec<T>`, `N01type`, `ProtectedStrVec`, `SplitResults`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `JiffTimestampVecMut`, `FactorHandling`, `CallDefRow`, `RPartialOrdView`, `RFromStrView`, `IntoRError`, `RawHeader`, `SEXPREC`, `IterStringData<I>`, `Arena<M>`, `AltrepGuard`, `SexpLengthError`, `RDeserializer`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RDebugView`, `CollectNA<I>`, `RNGtype`, `IterIntCoerceData<I, T>`, `Coerced<T, R>`, `RndVec<T>`, `FactorVec<T>`, `REncodingInfo`, `IterLogicalData<I>`, `RWrapperEntry`, `RSerdeError`, `SEXPTYPE`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `StrVecBuilder<''a>`, `DllInfo`, `R_CallMethodDef`, `AsDataFrame<T>`, `SparseIterIntData<I>`, `MatchArgError`, `TypedListError`, `RCloneView`, `TlsRoot`, `SexpError`, `DataFrameShape`, `FromJson<T>`, `ProtectKey`, `Sortedness`, `JiffTimestampVecRef`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `IterState<I, T>`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_erased`, `JsonOptions`, `RVecStorage<T, R, C>`, `AsDisplay<T>`, `NamedVector<M>`, `ReprotectSlot<''a>`, `StrVecCowIter`, `SerdeRows<T>`, `ListFromSexpError`, `TraitDispatchRow`, `ExternalPtr<T>`, `RSymbol`, `RRng`, `RStringArray`, `CaptureGroups`, `CollectStrings<I>`, `Borsh<T>`, `AsRNative<T>`, `RDefaultView`, `NaHandling`, `RIteratorView`, `Dots`, `DispatchNames`, `RAllocator`, `MatchArgParamDocEntry`, `RBase`, `SparseIterRealData<I>`, `StorageCoerceError`, `WindowedIterIntData<I>`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`, `Protected<''a, T>`, `StrVecIter`, `RThreadBuilder`, `TraitDispatchEntry`, `AsJsonPretty<T>`, `NullOnErr`, `JiffZonedVecMut`, `IterComplexData<I>`, `JiffTimestampVec`, `PanicReport<''a>`, `RCoerceError`, `RawError`, `ResultShape<S>`, `mx_base_vtable`, `ProtectPool`, `ProtectedStrVecIter<''a>`, `RawSlice<T>`, `ListBuilder<''a>`, `TypedListSpec`, `AltrepRegRow`, `TypeSpec`, `AsRError<E>`, `WorkerPump<T>`, `AsExternalPtr<T>`, `ClassNameEntry`, `StreamingRealData<F>`, `SexpTypeError`, `SpecialFloatHandling`, `Collect<I>`, `AltrepSexp`, `StrVec`, `Sampletype`, `ArenaGuard<''a, M>`, `RErrorView`, `Rboolean`, `SerdeRowBuilder<T>`, `SparseIterLogicalData<I>`, `mx_tag`, `WindowedIterRealData<I>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `ListAccumulator<''a>`, `RDataFrameBuilder`, `GuardMode`, `SidecarPropEntry`, `RngGuard`, `NamedDataFrameListBuilder`, `ProtectedStrVecCowIter<''a>`, `ParseStatus`, `RCow<''a, T>`, `JiffZonedVecRef`, `AsJson<T>`, `AsList<T>`, `IterRawData<I>`, `ExternalSlice<T>`, `AsVctrs<T>`, `SparseIterComplexData<I>`, `Altrep<T>`, `R_altrep_class_t`, `RawTagged<T>`, `SparseIterState<I, T>`, `StreamingIntData<F>`, `IterIntData<I>`, `SplitShape`, `TypedList`, `WorkerUnprotectGuard`, `RFlags<T>`, `RHashView`

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
| `std::borrow::Cow<''static, [T]>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:101 |
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
| `&''static [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:255 |
| `&''static mut [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:261 |
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
| `SVector<f64, {'expr': '2', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:396 |
| `SVector<f64, {'expr': '3', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:397 |
| `SVector<f64, {'expr': '4', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:398 |
| `SVector<i32, {'expr': '2', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:399 |
| `SVector<i32, {'expr': '3', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:400 |
| `SVector<i32, {'expr': '4', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:401 |
| `SMatrix<f64, {'expr': '2', 'value': None, 'is_literal': True}, {'expr': '2', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:405 |
| `SMatrix<f64, {'expr': '3', 'value': None, 'is_literal': True}, {'expr': '3', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:406 |
| `SMatrix<f64, {'expr': '4', 'value': None, 'is_literal': True}, {'expr': '4', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:407 |
| `SMatrix<i32, {'expr': '2', 'value': None, 'is_literal': True}, {'expr': '2', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:408 |
| `SMatrix<i32, {'expr': '3', 'value': None, 'is_literal': True}, {'expr': '3', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:409 |
| `SMatrix<i32, {'expr': '4', 'value': None, 'is_literal': True}, {'expr': '4', 'value': None, 'is_literal': True}>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:410 |
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

## `Allocation` — 137 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Rcomplex` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Dots` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RAllocator` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `MatchArgParamDocEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `IntoRError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypedList` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RThreadBuilder` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TraitDispatchEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SexpLengthError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsJsonPretty<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Factor<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `PanicReport<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `List` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ResultShape<S>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `mx_base_vtable` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RSerdeError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AltrepRegRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ClassNameEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RLogical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `MatchArgError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypedListError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DataFrame` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Collect<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TlsRoot` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SexpError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Missing<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `FactorMut<''a>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Raw<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Logical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RWrapperPriority` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `PanicSource` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SidecarPropEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RngGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsJson<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypedEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `N01type` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `FactorHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SplitShape` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `WorkerUnprotectGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AltrepGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SEXP` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RNGtype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RSerializer` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `REncodingInfo` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `REnv` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `StrVecIter` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `FactorVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SEXPTYPE` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RawError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `JiffZonedVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AltrepRegistration` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CollectNAInt<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ProtectKey` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `cetype_t` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Sortedness` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `JsonOptions` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `NamedVector<M>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SexpNaError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RCall` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `StrVecCowIter` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ListMut` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RStringArray` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CaptureGroups` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RRng` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsJsonVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SplitResults` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsRNative<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `NaHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CallDefRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DispatchNames` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SEXPREC` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `VctrsKind` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RDeserializer` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CollectNA<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RFlags<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RawHeader` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `NullOnErr` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RWrapperEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ThreadLocalArena` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RBorrow<''a, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `NamedList` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypedListSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DllInfo` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsRError<E>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RawSlice<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Altrep<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SexpTypeError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DataFrameShape` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `FromJson<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `DataFrameError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RBase` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Sampletype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `StrVec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Rboolean` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ThreadLocalHashArena` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `MatchArgChoicesEntry` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `mx_tag` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `SerdeRows<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RSidecar` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `TraitDispatchRow` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RSymbol` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `GuardMode` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `CollectStrings<I>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `RCow<''a, T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `ParseStatus` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `Borsh<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsList<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33 |

### `Allocation` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/arrow-buffer-55.2.0/src/alloc/mod.rs:33** (137 impls): `Rcomplex`, `Dots`, `RAllocator`, `RawTagged<T>`, `MatchArgParamDocEntry`, `IntoRError`, `TypedList`, `RThreadBuilder`, `TraitDispatchEntry`, `SexpLengthError`, `DuplicateNameError`, `AsJsonPretty<T>`, `Factor<''a>`, `JiffTimestampVec`, `RPrimitive<T>`, `PanicReport<''a>`, `List`, `ResultShape<S>`, `mx_base_vtable`, `RSerdeError`, `AltrepRegRow`, `WorkerPump<T>`, `AsNamedList<T>`, `ClassNameEntry`, `RLogical`, `MatchArgError`, `TypedListError`, `DataFrame`, `Collect<I>`, `TlsRoot`, `SexpError`, `Missing<T>`, `FactorMut<''a>`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `ListFromSexpError`, `SidecarPropEntry`, `RngGuard`, `AsJson<T>`, `AsNamedVector<T>`, `TypedEntry`, `N01type`, `FactorHandling`, `SplitShape`, `WorkerUnprotectGuard`, `StorageCoerceError`, `AltrepGuard`, `AsDisplayVec<T>`, `SEXP`, `RNGtype`, `RSerializer`, `REncodingInfo`, `REnv`, `StrVecIter`, `FactorVec<T>`, `RCoerceError`, `SEXPTYPE`, `RawError`, `R_CallMethodDef`, `AsDataFrame<T>`, `JiffZonedVec`, `AltrepRegistration`, `CollectNAInt<I>`, `ProtectKey`, `cetype_t`, `Sortedness`, `AsFromStr<T>`, `RawSliceTagged<T>`, `JsonOptions`, `AsDisplay<T>`, `NamedVector<M>`, `AsSerialize<T>`, `SexpNaError`, `RCall`, `StrVecCowIter`, `CoerceError`, `ListMut`, `RStringArray`, `CaptureGroups`, `RRng`, `AsJsonVec<T>`, `SplitResults`, `AsRNative<T>`, `NaHandling`, `CallDefRow`, `DispatchNames`, `SEXPREC`, `AsFromStrVec<T>`, `VctrsKind`, `RDeserializer`, `CollectNA<I>`, `RFlags<T>`, `RawHeader`, `NullOnErr`, `TypeMismatchError`, `RWrapperEntry`, `ThreadLocalArena`, `RBorrow<''a, T>`, `NamedList`, `TypedListSpec`, `DllInfo`, `AsRError<E>`, `AsExternalPtr<T>`, `RawSlice<T>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `SpecialFloatHandling`, `LogicalCoerceError`, `SexpTypeError`, `DataFrameShape`, `FromJson<T>`, `DataFrameError`, `RBase`, `Sampletype`, `StrVec`, `Rboolean`, `ThreadLocalHashArena`, `MatchArgChoicesEntry`, `mx_tag`, `SerdeRows<T>`, `TypeSpec`, `RSidecar`, `TraitDispatchRow`, `RSymbol`, `GuardMode`, `CollectStrings<I>`, `RCow<''a, T>`, `ParseStatus`, `Borsh<T>`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `R_altrep_class_t`

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

## `Equivalent` — 111 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RLogical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Missing<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Raw<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Logical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `PanicSource` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `N01type` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `StorageCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RFlags<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `AltrepGuard` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `SEXP` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RNGtype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `SEXPTYPE` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RawError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `ProtectKey` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Sortedness` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RawSliceTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RWrapperPriority` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `NamedVector<M>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `CoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `VctrsKind` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `NullOnErr` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Coerced<T, R>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RawSlice<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `VctrsBuildError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `TypeSpec` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RBase` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `LogicalCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Sampletype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `Rboolean` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `mx_tag` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RSidecar` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `AsSerialize<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `GuardMode` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `ParseStatus` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `ExternalPtr<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `RawTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82 |
| `StorageCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `AltrepGuard` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `SEXP` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RNGtype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `SEXPTYPE` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RawError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `ProtectKey` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Sortedness` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RawSliceTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `NamedVector<M>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `AsSerialize<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `CoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `VctrsKind` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RFlags<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `NullOnErr` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RawSlice<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `VctrsBuildError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Altrep<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Coerced<T, R>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `TypeSpec` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `LogicalCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Sampletype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Rboolean` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `mx_tag` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RSidecar` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `GuardMode` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `ParseStatus` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `ExternalPtr<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RawTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RLogical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Missing<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RBase` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Raw<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `Logical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `RWrapperPriority` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `PanicSource` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `N01type` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166 |
| `VctrsKind` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `NullOnErr` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Coerced<T, R>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RawSlice<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `VctrsBuildError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `TypeSpec` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RBase` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `LogicalCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Sampletype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Rboolean` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `mx_tag` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RSidecar` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `AsSerialize<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `GuardMode` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `ParseStatus` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `ExternalPtr<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RawTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Altrep<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RLogical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Missing<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Raw<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Logical` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `PanicSource` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `N01type` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `StorageCoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RFlags<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `AltrepGuard` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `SEXP` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RNGtype` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `SEXPTYPE` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RawError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `ProtectKey` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `Sortedness` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RawSliceTagged<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `RWrapperPriority` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `NamedVector<M>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |
| `CoerceError` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151 |

### `Equivalent` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.0/src/lib.rs:151** (37 impls): `VctrsKind`, `NullOnErr`, `Coerced<T, R>`, `RawSlice<T>`, `VctrsBuildError`, `TypeSpec`, `RBase`, `LogicalCoerceError`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `AsSerialize<T>`, `GuardMode`, `ParseStatus`, `ExternalPtr<T>`, `RawTagged<T>`, `Altrep<T>`, `RLogical`, `Missing<T>`, `Raw<T>`, `Logical`, `PanicSource`, `N01type`, `StorageCoerceError`, `RFlags<T>`, `AltrepGuard`, `SEXP`, `RNGtype`, `SEXPTYPE`, `RawError`, `ProtectKey`, `Sortedness`, `RawSliceTagged<T>`, `RWrapperPriority`, `NamedVector<M>`, `CoerceError`
- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.5/src/lib.rs:166** (37 impls): `StorageCoerceError`, `AltrepGuard`, `SEXP`, `RNGtype`, `SEXPTYPE`, `RawError`, `ProtectKey`, `Sortedness`, `RawSliceTagged<T>`, `NamedVector<M>`, `AsSerialize<T>`, `CoerceError`, `VctrsKind`, `RFlags<T>`, `NullOnErr`, `RawSlice<T>`, `VctrsBuildError`, `Altrep<T>`, `Coerced<T, R>`, `TypeSpec`, `LogicalCoerceError`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `GuardMode`, `ParseStatus`, `ExternalPtr<T>`, `RawTagged<T>`, `RLogical`, `Missing<T>`, `RBase`, `Raw<T>`, `Logical`, `RWrapperPriority`, `PanicSource`, `N01type`
- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:82** (37 impls): `Altrep<T>`, `RLogical`, `Missing<T>`, `Raw<T>`, `Logical`, `PanicSource`, `N01type`, `StorageCoerceError`, `RFlags<T>`, `AltrepGuard`, `SEXP`, `RNGtype`, `SEXPTYPE`, `RawError`, `ProtectKey`, `Sortedness`, `RawSliceTagged<T>`, `RWrapperPriority`, `NamedVector<M>`, `CoerceError`, `VctrsKind`, `NullOnErr`, `Coerced<T, R>`, `RawSlice<T>`, `VctrsBuildError`, `TypeSpec`, `RBase`, `LogicalCoerceError`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `AsSerialize<T>`, `GuardMode`, `ParseStatus`, `ExternalPtr<T>`, `RawTagged<T>`

## `TryCoerce` — 95 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T, R> +1wc` | concrete | 2 | miniextendr-api/src/coerce.rs:112 |
| `Rcomplex` | `<T, R> +1wc` | blanket | 2 | miniextendr-api/src/coerce.rs:112 |
| `Rboolean` | `<T, R> +1wc` | blanket | 2 | miniextendr-api/src/coerce.rs:112 |
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
- **miniextendr-api/src/coerce.rs:112** (3 impls): `T`, `Rcomplex`, `Rboolean`

## `RDebug` — 93 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `FactorVec<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `REncodingInfo` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RCoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RawError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `List` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RPrimitive<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SexpTypeError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `DataFrame` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Sampletype` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Rboolean` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `mx_tag` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Raw<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Logical` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `PanicSource` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `CaptureGroups` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ListMut` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `R_CMethodDef` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ProtectedStrVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Rcomplex` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `FactorHandling` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AltrepGuard` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SEXPREC` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RawSlice<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypedListSpec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypeSpec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RLogical` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsRError<E>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `DllInfo` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `DataFrameError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Sortedness` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `JsonOptions` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RSidecar` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `CoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `NamedVector<M>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SexpNaError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RStringArray` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `N01type` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsRNative<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `NaHandling` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `DispatchNames` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `IntoRError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Dots` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RFlags<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RAllocator` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypedList` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SEXP` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Protected<''a, T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RNGtype` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `NullOnErr` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SEXPTYPE` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RSerdeError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `MatchArgError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypedListError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SexpError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Missing<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ProtectKey` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `StrVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AltrepSexp` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RBase` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypeSpec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `GuardMode` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `ParseStatus` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RRng` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsList<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `Altrep<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `TypedEntry` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `VctrsKind` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `SexpLengthError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:63 |

### `RDebug` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:63** (93 impls): `FactorVec<T>`, `REncodingInfo`, `RCoerceError`, `RawError`, `List`, `RPrimitive<T>`, `SexpTypeError`, `AsNamedList<T>`, `AsDataFrame<T>`, `DataFrame`, `Sampletype`, `Rboolean`, `RawSliceTagged<T>`, `AsFromStr<T>`, `AsSerialize<T>`, `mx_tag`, `Raw<T>`, `Logical`, `PanicSource`, `CaptureGroups`, `ListMut`, `AsNamedVector<T>`, `R_CMethodDef`, `ProtectedStrVec`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `FactorHandling`, `AsFromStrVec<T>`, `AltrepGuard`, `SEXPREC`, `TypeMismatchError`, `VctrsBuildError`, `RawSlice<T>`, `TypedListSpec`, `R_CallMethodDef`, `TypeSpec`, `RLogical`, `AsRError<E>`, `DllInfo`, `DataFrameError`, `Sortedness`, `JsonOptions`, `RSidecar`, `AsDisplay<T>`, `CoerceError`, `NamedVector<M>`, `SexpNaError`, `RStringArray`, `N01type`, `AsRNative<T>`, `AsVctrs<T>`, `NaHandling`, `T`, `DispatchNames`, `IntoRError`, `Dots`, `RFlags<T>`, `RAllocator`, `TypedList`, `SEXP`, `DuplicateNameError`, `Protected<''a, T>`, `Coerced<T, R>`, `RNGtype`, `NullOnErr`, `SEXPTYPE`, `RSerdeError`, `AsExternalPtr<T>`, `LogicalCoerceError`, `MatchArgError`, `TypedListError`, `SpecialFloatHandling`, `SexpError`, `Missing<T>`, `ProtectKey`, `StrVec`, `AltrepSexp`, `RBase`, `ExternalPtr<T>`, `ListFromSexpError`, `TypeSpec`, `GuardMode`, `ParseStatus`, `RRng`, `AsList<T>`, `Altrep<T>`, `FactorOptionVec<T>`, `TypedEntry`, `RawTagged<T>`, `StorageCoerceError`, `VctrsKind`, `SexpLengthError`, `AsDisplayVec<T>`

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
| `Protected<''a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1048 |
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
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:880 |
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

## `RClone` — 87 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RawError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RawSlice<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypedListSpec` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TlsRoot` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Sortedness` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `cetype_t` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `NamedVector<M>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsRNative<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `NaHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Root<''a>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypedList` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SEXP` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `NullOnErr` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `StrVec` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Sampletype` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Rboolean` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `mx_tag` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RBase` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `CoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `GuardMode` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `ParseStatus` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsList<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypedEntry` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `IntoRError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `FactorVec<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `REncodingInfo` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `List` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RLogical` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `TypedListError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `DataFrame` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SexpError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RSidecar` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Raw<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Logical` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `PanicSource` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `CaptureGroups` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `R_CMethodDef` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `N01type` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `Rcomplex` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RawHeader` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `AltrepGuard` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |
| `RNGtype` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:373 |

### `RClone` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:373** (87 impls): `SEXPTYPE`, `RCoerceError`, `RawError`, `T`, `RawSlice<T>`, `TypedListSpec`, `R_CallMethodDef`, `TypeSpec`, `TlsRoot`, `ProtectKey`, `Sortedness`, `cetype_t`, `JsonOptions`, `AsDisplay<T>`, `NamedVector<M>`, `ExternalPtr<T>`, `AsRNative<T>`, `AsVctrs<T>`, `Altrep<T>`, `NaHandling`, `DispatchNames`, `Root<''a>`, `TypedList`, `SEXP`, `Coerced<T, R>`, `NullOnErr`, `TypeMismatchError`, `VctrsBuildError`, `AsExternalPtr<T>`, `SexpTypeError`, `SpecialFloatHandling`, `Missing<T>`, `StrVec`, `Sampletype`, `DataFrameError`, `Rboolean`, `mx_tag`, `RBase`, `TypeSpec`, `CoerceError`, `SexpNaError`, `GuardMode`, `ParseStatus`, `AsList<T>`, `FactorOptionVec<T>`, `TypedEntry`, `R_altrep_class_t`, `RawTagged<T>`, `IntoRError`, `VctrsKind`, `RFlags<T>`, `DuplicateNameError`, `AsDisplayVec<T>`, `FactorVec<T>`, `REncodingInfo`, `List`, `RSerdeError`, `AsNamedList<T>`, `RLogical`, `AsDataFrame<T>`, `LogicalCoerceError`, `MatchArgError`, `TypedListError`, `DataFrame`, `SexpError`, `RawSliceTagged<T>`, `AsFromStr<T>`, `RWrapperPriority`, `RSidecar`, `AsSerialize<T>`, `Raw<T>`, `Logical`, `PanicSource`, `ListFromSexpError`, `CaptureGroups`, `AsNamedVector<T>`, `R_CMethodDef`, `N01type`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `FactorHandling`, `StorageCoerceError`, `RawHeader`, `AsFromStrVec<T>`, `AltrepGuard`, `SexpLengthError`, `RNGtype`

## `ToOwned` — 86 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RBase` | `<T> +1wc` | blanket | 3 | (no span) |
| `NullOnErr` | `<T> +1wc` | blanket | 3 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 3 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 3 | (no span) |
| `RawSlice<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypedListSpec` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 3 | (no span) |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 3 | (no span) |
| `StrVec` | `<T> +1wc` | blanket | 3 | (no span) |
| `Sampletype` | `<T> +1wc` | blanket | 3 | (no span) |
| `Rboolean` | `<T> +1wc` | blanket | 3 | (no span) |
| `mx_tag` | `<T> +1wc` | blanket | 3 | (no span) |
| `RSidecar` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 3 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 3 | (no span) |
| `GuardMode` | `<T> +1wc` | blanket | 3 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `ParseStatus` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsList<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 3 | (no span) |
| `RawTagged<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypedList` | `<T> +1wc` | blanket | 3 | (no span) |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypeMismatchError` | `<T> +1wc` | blanket | 3 | (no span) |
| `FactorVec<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `List` | `<T> +1wc` | blanket | 3 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `RLogical` | `<T> +1wc` | blanket | 3 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 3 | (no span) |
| `DataFrame` | `<T> +1wc` | blanket | 3 | (no span) |
| `TlsRoot` | `<T> +1wc` | blanket | 3 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 3 | (no span) |
| `Missing<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `RWrapperPriority` | `<T> +1wc` | blanket | 3 | (no span) |
| `Raw<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `Logical` | `<T> +1wc` | blanket | 3 | (no span) |
| `PanicSource` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `R_CMethodDef` | `<T> +1wc` | blanket | 3 | (no span) |
| `N01type` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypedEntry` | `<T> +1wc` | blanket | 3 | (no span) |
| `Rcomplex` | `<T> +1wc` | blanket | 3 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 3 | (no span) |
| `FactorHandling` | `<T> +1wc` | blanket | 3 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 3 | (no span) |
| `RawHeader` | `<T> +1wc` | blanket | 3 | (no span) |
| `Root<''a>` | `<T> +1wc` | blanket | 3 | (no span) |
| `AltrepGuard` | `<T> +1wc` | blanket | 3 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 3 | (no span) |
| `SEXP` | `<T> +1wc` | blanket | 3 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `RNGtype` | `<T> +1wc` | blanket | 3 | (no span) |
| `REncodingInfo` | `<T> +1wc` | blanket | 3 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 3 | (no span) |
| `SEXPTYPE` | `<T> +1wc` | blanket | 3 | (no span) |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 3 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 3 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 3 | (no span) |
| `ProtectKey` | `<T> +1wc` | blanket | 3 | (no span) |
| `Sortedness` | `<T> +1wc` | blanket | 3 | (no span) |
| `cetype_t` | `<T> +1wc` | blanket | 3 | (no span) |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `JsonOptions` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `NamedVector<M>` | `<T> +1wc` | blanket | 3 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 3 | (no span) |
| `CaptureGroups` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsRNative<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `NaHandling` | `<T> +1wc` | blanket | 3 | (no span) |
| `Altrep<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `DispatchNames` | `<T> +1wc` | blanket | 3 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 3 | (no span) |
| `VctrsKind` | `<T> +1wc` | blanket | 3 | (no span) |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 3 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 3 | (no span) |

### `ToOwned` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (86 impls): `RBase`, `NullOnErr`, `RCoerceError`, `RawError`, `RawSlice<T>`, `TypedListSpec`, `TypeSpec`, `AsExternalPtr<T>`, `SexpTypeError`, `SpecialFloatHandling`, `StrVec`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `GuardMode`, `ExternalPtr<T>`, `ParseStatus`, `AsList<T>`, `AsVctrs<T>`, `FactorOptionVec<T>`, `R_altrep_class_t`, `RawTagged<T>`, `TypedList`, `Coerced<T, R>`, `TypeMismatchError`, `FactorVec<T>`, `List`, `VctrsBuildError`, `AsNamedList<T>`, `RLogical`, `LogicalCoerceError`, `DataFrame`, `TlsRoot`, `DataFrameError`, `Missing<T>`, `RWrapperPriority`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `AsNamedVector<T>`, `R_CMethodDef`, `N01type`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `FactorHandling`, `IntoRError`, `RawHeader`, `Root<''a>`, `AltrepGuard`, `SexpLengthError`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RNGtype`, `REncodingInfo`, `RSerdeError`, `SEXPTYPE`, `R_CallMethodDef`, `AsDataFrame<T>`, `MatchArgError`, `TypedListError`, `SexpError`, `ProtectKey`, `Sortedness`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `JsonOptions`, `AsDisplay<T>`, `NamedVector<M>`, `ListFromSexpError`, `CaptureGroups`, `AsRNative<T>`, `NaHandling`, `Altrep<T>`, `DispatchNames`, `StorageCoerceError`, `VctrsKind`, `AsFromStrVec<T>`, `RFlags<T>`

## `CloneToUninit` — 86 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `NullOnErr` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSlice<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `StrVec` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sampletype` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rboolean` | `<T> +1wc` | blanket | 1 | (no span) |
| `mx_tag` | `<T> +1wc` | blanket | 1 | (no span) |
| `NamedVector<M>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSidecar` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | (no span) |
| `GuardMode` | `<T> +1wc` | blanket | 1 | (no span) |
| `ParseStatus` | `<T> +1wc` | blanket | 1 | (no span) |
| `RBase` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsVctrs<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedList` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | (no span) |
| `List` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RLogical` | `<T> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrame` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | (no span) |
| `Raw<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Logical` | `<T> +1wc` | blanket | 1 | (no span) |
| `PanicSource` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypeSpec` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `N01type` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `Rcomplex` | `<T> +1wc` | blanket | 1 | (no span) |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawHeader` | `<T> +1wc` | blanket | 1 | (no span) |
| `AltrepGuard` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXP` | `<T> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplayVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RNGtype` | `<T> +1wc` | blanket | 1 | (no span) |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `REncodingInfo` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedList<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDataFrame<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | (no span) |
| `Sortedness` | `<T> +1wc` | blanket | 1 | (no span) |
| `cetype_t` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `CaptureGroups` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsNamedVector<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRNative<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `NaHandling` | `<T> +1wc` | blanket | 1 | (no span) |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsFromStrVec<T>` | `<T> +1wc` | blanket | 1 | (no span) |

### `CloneToUninit` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (86 impls): `NullOnErr`, `RCoerceError`, `RawError`, `RawSlice<T>`, `TypedListSpec`, `TypeSpec`, `AsExternalPtr<T>`, `SexpTypeError`, `SpecialFloatHandling`, `StrVec`, `Sampletype`, `Rboolean`, `mx_tag`, `NamedVector<M>`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `GuardMode`, `ParseStatus`, `RBase`, `AsList<T>`, `AsVctrs<T>`, `R_altrep_class_t`, `RawTagged<T>`, `TypedList`, `RFlags<T>`, `TypeMismatchError`, `List`, `VctrsBuildError`, `RLogical`, `LogicalCoerceError`, `DataFrame`, `DataFrameError`, `RWrapperPriority`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `R_CMethodDef`, `FactorOptionVec<T>`, `N01type`, `TypedEntry`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `FactorHandling`, `IntoRError`, `RawHeader`, `AltrepGuard`, `SexpLengthError`, `SEXP`, `DuplicateNameError`, `AsDisplayVec<T>`, `RNGtype`, `Coerced<T, R>`, `FactorVec<T>`, `REncodingInfo`, `RSerdeError`, `SEXPTYPE`, `AsNamedList<T>`, `R_CallMethodDef`, `AsDataFrame<T>`, `MatchArgError`, `TypedListError`, `TlsRoot`, `SexpError`, `Missing<T>`, `ProtectKey`, `Sortedness`, `cetype_t`, `RawSliceTagged<T>`, `AsFromStr<T>`, `JsonOptions`, `AsDisplay<T>`, `ListFromSexpError`, `ExternalPtr<T>`, `CaptureGroups`, `AsNamedVector<T>`, `AsRNative<T>`, `Altrep<T>`, `NaHandling`, `DispatchNames`, `StorageCoerceError`, `Root<''a>`, `VctrsKind`, `AsFromStrVec<T>`

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
| `Root<''a>` | `<'a>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:815 |
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
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:120 |
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
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&''static [f64]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:140 |
| `&''static [bool]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:243 |
| `&''static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:296 |
| `&''static [i32]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:31 |
| `&''static [String]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:361 |
| `&''static [&''static str]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:401 |
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
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&''static [i32]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:129 |
| `&''static [f64]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:231 |
| `&''static [bool]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:285 |
| `&''static [u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:346 |
| `&''static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:386 |
| `&''static [&''static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:426 |
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
| `Cow<''static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1186 |
| `Cow<''static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1191 |
| `Cow<''static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1196 |
| `Cow<''static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1201 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:495 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:499 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:503 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:507 |
| `Vec<Option<String>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:519 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:531 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:547 |
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
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&''static [f64]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:132 |
| `&''static [i32]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:23 |
| `&''static [bool]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:234 |
| `&''static [u8]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:288 |
| `&''static [String]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:349 |
| `&''static [&''static str]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:389 |
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

## `RCopy` — 49 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsExternalPtr<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `SexpTypeError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `StrVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Sampletype` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Rboolean` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `mx_tag` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RSidecar` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `CoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `SexpNaError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `GuardMode` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `ParseStatus` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `AsList<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `R_altrep_class_t` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RFlags<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `List` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RLogical` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `DataFrame` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RWrapperPriority` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Raw<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Logical` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `PanicSource` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `R_CMethodDef` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `N01type` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Rcomplex` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RawHeader` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `AltrepGuard` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `SexpLengthError` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `SEXP` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RNGtype` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `SEXPTYPE` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `R_CallMethodDef` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `TlsRoot` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Missing<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `ProtectKey` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Sortedness` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `cetype_t` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `AsDisplay<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `RBase` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `AsRNative<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Altrep<T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `Root<''a>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `VctrsKind` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `NullOnErr` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/adapter_traits.rs:467 |
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:467 |

### `RCopy` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:467** (49 impls): `AsExternalPtr<T>`, `SexpTypeError`, `StrVec`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `AsSerialize<T>`, `CoerceError`, `SexpNaError`, `GuardMode`, `ParseStatus`, `AsList<T>`, `R_altrep_class_t`, `RawTagged<T>`, `RFlags<T>`, `List`, `RLogical`, `LogicalCoerceError`, `DataFrame`, `RWrapperPriority`, `Raw<T>`, `Logical`, `PanicSource`, `R_CMethodDef`, `N01type`, `Rcomplex`, `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>`, `RawHeader`, `AltrepGuard`, `SexpLengthError`, `SEXP`, `RNGtype`, `Coerced<T, R>`, `SEXPTYPE`, `R_CallMethodDef`, `TlsRoot`, `Missing<T>`, `ProtectKey`, `Sortedness`, `cetype_t`, `AsDisplay<T>`, `RBase`, `AsRNative<T>`, `Altrep<T>`, `Root<''a>`, `VctrsKind`, `NullOnErr`, `T`

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
| `Root<''a>` | `<'a>` | concrete | 0 | miniextendr-api/src/gc_protect.rs:815 |
| `TlsRoot` | `` | concrete | 0 | miniextendr-api/src/gc_protect/tls.rs:202 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:119 |
| `List` | `` | concrete | 0 | miniextendr-api/src/list.rs:40 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 0 | miniextendr-api/src/rarray.rs:120 |
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

## `Scalar` — 38 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VctrsKind` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `NullOnErr` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RBase` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RawSlice<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Sampletype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Rboolean` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `mx_tag` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RSidecar` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `GuardMode` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `ParseStatus` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RawTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Altrep<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RLogical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Missing<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Raw<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Logical` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `PanicSource` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `TypeSpec` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `N01type` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Rcomplex` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RFlags<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `AltrepGuard` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `SEXP` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RNGtype` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `SEXPTYPE` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RawError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `ProtectKey` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `Sortedness` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `RawSliceTagged<T>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `NamedVector<M>` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |
| `CoerceError` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8 |

### `Scalar` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/scalar.rs:8** (38 impls): `VctrsKind`, `NullOnErr`, `RBase`, `Coerced<T, R>`, `RawSlice<T>`, `VctrsBuildError`, `TypeSpec`, `LogicalCoerceError`, `Sampletype`, `Rboolean`, `mx_tag`, `RSidecar`, `AsSerialize<T>`, `GuardMode`, `ParseStatus`, `ExternalPtr<T>`, `RawTagged<T>`, `Altrep<T>`, `RLogical`, `Missing<T>`, `Raw<T>`, `Logical`, `PanicSource`, `TypeSpec`, `N01type`, `Rcomplex`, `StorageCoerceError`, `RFlags<T>`, `AltrepGuard`, `SEXP`, `RNGtype`, `SEXPTYPE`, `RawError`, `ProtectKey`, `Sortedness`, `RawSliceTagged<T>`, `NamedVector<M>`, `CoerceError`

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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:576 |
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:577 |
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:578 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:579 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:582 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:583 |
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
| `Cow<''static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1188 |
| `Cow<''static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1193 |
| `Cow<''static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1198 |
| `Cow<''static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1203 |
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
| `Cow<''static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1189 |
| `Cow<''static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1194 |
| `Cow<''static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1199 |
| `Cow<''static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1204 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:390 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:391 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:392 |
| `Vec<bool>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:393 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:394 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:395 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:396 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:399 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:400 |
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

## `AltrepExtract` — 22 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JiffTimestampVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `SparseIterRawData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterIntFromBoolData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterRealData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterListData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterRealCoerceData<I, T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterStringData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterIntCoerceData<I, T>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterLogicalData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `SparseIterIntData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `JiffZonedVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `SparseIterRealData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `WindowedIterIntData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterComplexData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `StreamingRealData<F>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `SparseIterLogicalData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `WindowedIterRealData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterRawData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `SparseIterComplexData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `StreamingIntData<F>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |
| `IterIntData<I>` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/altrep_data/core.rs:336 |

### `AltrepExtract` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/altrep_data/core.rs:336** (22 impls): `JiffTimestampVec`, `SparseIterRawData<I>`, `T`, `IterIntFromBoolData<I>`, `IterRealData<I>`, `IterListData<I>`, `IterRealCoerceData<I, T>`, `IterStringData<I>`, `IterIntCoerceData<I, T>`, `IterLogicalData<I>`, `SparseIterIntData<I>`, `JiffZonedVec`, `SparseIterRealData<I>`, `WindowedIterIntData<I>`, `IterComplexData<I>`, `StreamingRealData<F>`, `SparseIterLogicalData<I>`, `WindowedIterRealData<I>`, `IterRawData<I>`, `SparseIterComplexData<I>`, `StreamingIntData<F>`, `IterIntData<I>`

## `RDisplay` — 22 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `AsRError<E>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `CoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `IntoRError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `TypedListError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `SexpError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `RawError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:102 |

### `RDisplay` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:102** (22 impls): `VctrsBuildError`, `AsRError<E>`, `T`, `DataFrameError`, `CoerceError`, `SexpNaError`, `IntoRError`, `RFlags<T>`, `DuplicateNameError`, `RSerdeError`, `SexpTypeError`, `LogicalCoerceError`, `MatchArgError`, `TypedListError`, `SexpError`, `ExternalPtr<T>`, `ListFromSexpError`, `StorageCoerceError`, `SexpLengthError`, `RCoerceError`, `RawError`, `TypeMismatchError`

## `ToString` — 21 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `TypeMismatchError` | `<T> +1wc` | blanket | 1 | (no span) |
| `VctrsBuildError` | `<T> +1wc` | blanket | 1 | (no span) |
| `AsRError<E>` | `<T> +1wc` | blanket | 1 | (no span) |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DataFrameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `IntoRError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpLengthError` | `<T> +1wc` | blanket | 1 | (no span) |
| `DuplicateNameError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RSerdeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `MatchArgError` | `<T> +1wc` | blanket | 1 | (no span) |
| `TypedListError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpTypeError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ListFromSexpError` | `<T> +1wc` | blanket | 1 | (no span) |
| `StorageCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | (no span) |
| `RCoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `RawError` | `<T> +1wc` | blanket | 1 | (no span) |
| `CoerceError` | `<T> +1wc` | blanket | 1 | (no span) |
| `SexpNaError` | `<T> +1wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | (no span) |

### `ToString` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (21 impls): `TypeMismatchError`, `VctrsBuildError`, `AsRError<E>`, `LogicalCoerceError`, `DataFrameError`, `IntoRError`, `SexpLengthError`, `DuplicateNameError`, `RSerdeError`, `MatchArgError`, `TypedListError`, `SexpTypeError`, `SexpError`, `ListFromSexpError`, `StorageCoerceError`, `RFlags<T>`, `RCoerceError`, `RawError`, `CoerceError`, `SexpNaError`, `ExternalPtr<T>`

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
| `Factor<''_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:213 |
| `FactorMut<''_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:309 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:488 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:546 |
| `Protected<''a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:1039 |
| `Root<''a>` | `<'a>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:837 |
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
| `RCow<''_, T>` | `<T>` | concrete | 2 | miniextendr-api/src/rcow.rs:146 |
| `ArenaGuard<''_, M>` | `<M>` | concrete | 2 | miniextendr-api/src/refcount_protect.rs:788 |

### `Deref` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/optionals/jiff_impl.rs:923** (2 impls): `JiffZonedVecMut`, `JiffZonedVecRef`
- **miniextendr-api/src/optionals/jiff_impl.rs:882** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`

## `Receiver` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `FactorVec<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `TlsRoot` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `JiffTimestampVecRef` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `ExternalPtr<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `RStringArray` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `Altrep<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `Root<''a>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `Protected<''a, T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `JiffZonedVecMut` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `ArenaGuard<''a, M>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `RCow<''a, T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `JiffZonedVecRef` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `RFlags<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `Factor<''a>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `RPrimitive<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `OwnedProtect` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `FactorMut<''a>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `FactorOptionVec<T>` | `<P, T> +2wc` | blanket | 1 | (no span) |
| `JiffTimestampVecMut` | `<P, T> +2wc` | blanket | 1 | (no span) |

### `Receiver` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (20 impls): `Coerced<T, R>`, `FactorVec<T>`, `TlsRoot`, `JiffTimestampVecRef`, `ExternalPtr<T>`, `RStringArray`, `Altrep<T>`, `Root<''a>`, `Protected<''a, T>`, `JiffZonedVecMut`, `ArenaGuard<''a, M>`, `RCow<''a, T>`, `JiffZonedVecRef`, `RFlags<T>`, `Factor<''a>`, `RPrimitive<T>`, `OwnedProtect`, `FactorMut<''a>`, `FactorOptionVec<T>`, `JiffTimestampVecMut`

## `RDefault` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SpecialFloatHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `Missing<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `ProtectScope` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `VctrsKind` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `Arena<M>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `RSidecar` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `RRng` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `FactorHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `SEXP` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `RThreadBuilder` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `WorkerPump<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `JsonOptions` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `RngGuard` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `NamedDataFrameListBuilder` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `NaHandling` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `DispatchNames` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:412 |

### `RDefault` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:412** (20 impls): `SpecialFloatHandling`, `Missing<T>`, `ProtectScope`, `T`, `VctrsKind`, `Arena<M>`, `RSidecar`, `AsSerialize<T>`, `RRng`, `FactorHandling`, `SEXP`, `RThreadBuilder`, `WorkerPump<T>`, `JsonOptions`, `ExternalPtr<T>`, `RngGuard`, `NamedDataFrameListBuilder`, `NaHandling`, `DispatchNames`, `Coerced<T, R>`

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

## `RError` — 19 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RSerdeError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `LogicalCoerceError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `MatchArgError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `TypedListError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `SexpError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `ListFromSexpError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `StorageCoerceError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `SexpLengthError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `RCoerceError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `RawError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `SexpTypeError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `TypeMismatchError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `VctrsBuildError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `DataFrameError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `CoerceError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `SexpNaError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `IntoRError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |
| `DuplicateNameError` | `<T> +1wc` | blanket | 3 | miniextendr-api/src/adapter_traits.rs:275 |

### `RError` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:275** (19 impls): `RSerdeError`, `LogicalCoerceError`, `MatchArgError`, `TypedListError`, `SexpError`, `ListFromSexpError`, `StorageCoerceError`, `SexpLengthError`, `RCoerceError`, `RawError`, `T`, `SexpTypeError`, `TypeMismatchError`, `VctrsBuildError`, `DataFrameError`, `CoerceError`, `SexpNaError`, `IntoRError`, `DuplicateNameError`

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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1187 |
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
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1192 |
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
| `std::borrow::Cow<''static, [f64]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `&''static [f64]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:162 |
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
| `std::borrow::Cow<''static, [i32]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `&''static [i32]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:54 |
| `Int32Array` | `` | concrete | 14 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `DVector<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/ndarray_impl.rs:3573 |

## `RHash` — 15 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SEXPTYPE` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `RLogical` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `RSidecar` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `N01type` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `RNGtype` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `ProtectKey` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `Altrep<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `Sampletype` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `Rboolean` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `mx_tag` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:137 |
| `RFlags<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:137 |

### `RHash` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:137** (15 impls): `SEXPTYPE`, `RLogical`, `RSidecar`, `ExternalPtr<T>`, `N01type`, `Coerced<T, R>`, `RNGtype`, `ProtectKey`, `Altrep<T>`, `Sampletype`, `Rboolean`, `AsSerialize<T>`, `mx_tag`, `T`, `RFlags<T>`

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
| `ArenaGuard<''_, M>` | `<M>` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:782 |
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
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Box<[String]>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `&''static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:363 |
| `&''static [&''static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:403 |
| `StringArray` | `` | concrete | 5 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |

## `AltStringData` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1086 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:509 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:521 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:537 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:553 |
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
| `FactorMut<''_>` | `` | concrete | 1 | miniextendr-api/src/factor.rs:318 |
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
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `&''static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:318 |
| `UInt8Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |

## `AltRawData` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[u8; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1065 |
| `std::borrow::Cow<''static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1197 |
| `Vec<u8>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:504 |
| `Box<[u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:589 |
| `&[u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:962 |
| `SparseIterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:518 |
| `IterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:561 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1674 |

## `IntoIterator` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `StrVecIter` | `<I> +1wc` | blanket | 3 | (no span) |
| `ProtectedStrVecIter<''a>` | `<I> +1wc` | blanket | 3 | (no span) |
| `ProtectedStrVecCowIter<''a>` | `<I> +1wc` | blanket | 3 | (no span) |
| `ExternalPtr<T>` | `<I> +1wc` | blanket | 3 | (no span) |
| `StrVecCowIter` | `<I> +1wc` | blanket | 3 | (no span) |
| `StrVec` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:295 |
| `&''a ProtectedStrVec` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:644 |

### `IntoIterator` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (5 impls): `StrVecIter`, `ProtectedStrVecIter<''a>`, `ProtectedStrVecCowIter<''a>`, `ExternalPtr<T>`, `StrVecCowIter`

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
| `&''static [bool]` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/static_slices.rs:245 |
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

## `Parsable` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Protected<''a, T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |
| `Altrep<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |
| `FactorVec<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |
| `RFlags<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |
| `Coerced<T, R>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |
| `ExternalPtr<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30 |

### `Parsable` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/parsing/parsable.rs:30** (6 impls): `Protected<''a, T>`, `Altrep<T>`, `FactorVec<T>`, `RFlags<T>`, `Coerced<T, R>`, `ExternalPtr<T>`

## `AltComplexData` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1096 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1103 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1111 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1202 |
| `IterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:747 |
| `SparseIterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:628 |

## `Formattable` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |
| `Coerced<T, R>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |
| `ExternalPtr<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |
| `Protected<''a, T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |
| `FactorVec<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |
| `Altrep<T>` | `<T> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33 |

### `Formattable` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.45/src/formatting/formattable.rs:33** (6 impls): `RFlags<T>`, `Coerced<T, R>`, `ExternalPtr<T>`, `Protected<''a, T>`, `FactorVec<T>`, `Altrep<T>`

## `AltComplex` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterComplexData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/coerce.rs:808 |
| `SparseIterComplexData<I>` | `<I>` | concrete | 4 | miniextendr-api/src/altrep_data/iter/sparse.rs:689 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `Vec<crate::Rcomplex>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<''static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |

## `RNdArrayOps` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array1<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1728 |
| `Array2<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1786 |
| `ArrayD<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1844 |
| `Array1<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1904 |
| `Array2<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:1969 |
| `ArrayD<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2034 |

## `IteratorRandom` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectedStrVecIter<''a>` | `<I> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285 |
| `ProtectedStrVecCowIter<''a>` | `<I> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285 |
| `StrVecCowIter` | `<I> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285 |
| `ExternalPtr<T>` | `<I> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285 |
| `StrVecIter` | `<I> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285 |

### `IteratorRandom` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/seq/iterator.rs:285** (5 impls): `ProtectedStrVecIter<''a>`, `ProtectedStrVecCowIter<''a>`, `StrVecCowIter`, `ExternalPtr<T>`, `StrVecIter`

## `ROrd` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:174 |
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:174 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:174 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:174 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:174 |

### `ROrd` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:174** (5 impls): `RWrapperPriority`, `AsSerialize<T>`, `T`, `ExternalPtr<T>`, `Coerced<T, R>`

## `Iterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/externalptr.rs:1632 |
| `StrVecIter` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:234 |
| `StrVecCowIter` | `` | concrete | 3 | miniextendr-api/src/strvec.rs:269 |
| `ProtectedStrVecIter<''a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:591 |
| `ProtectedStrVecCowIter<''a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:622 |

## `RPartialOrd` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsSerialize<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:215 |
| `ExternalPtr<T>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:215 |
| `Coerced<T, R>` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:215 |
| `RWrapperPriority` | `<T> +1wc` | blanket | 1 | miniextendr-api/src/adapter_traits.rs:215 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:215 |

### `RPartialOrd` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/adapter_traits.rs:215** (5 impls): `AsSerialize<T>`, `ExternalPtr<T>`, `Coerced<T, R>`, `RWrapperPriority`, `T`

## `ExactSizeIterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1660 |
| `StrVecIter` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:258 |
| `StrVecCowIter` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:293 |
| `ProtectedStrVecIter<''_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:613 |
| `ProtectedStrVecCowIter<''_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:642 |

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

## `Rng` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<R> +1wc` | blanket | 3 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:65 |
| `Coerced<T, R>` | `<R> +1wc` | blanket | 3 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:65 |
| `ExternalPtr<T>` | `<R> +1wc` | blanket | 3 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:65 |
| `RRng` | `<R> +1wc` | blanket | 3 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:65 |

### `Rng` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:65** (4 impls): `Altrep<T>`, `Coerced<T, R>`, `ExternalPtr<T>`, `RRng`

## `IntoRecords` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/papergrid-0.17.0/src/records/into_records.rs:16 |
| `StrVec` | `<T> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/papergrid-0.17.0/src/records/into_records.rs:16 |
| `StrVecIter` | `<T> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/papergrid-0.17.0/src/records/into_records.rs:16 |
| `StrVecCowIter` | `<T> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/papergrid-0.17.0/src/records/into_records.rs:16 |

### `IntoRecords` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/papergrid-0.17.0/src/records/into_records.rs:16** (4 impls): `ExternalPtr<T>`, `StrVec`, `StrVecIter`, `StrVecCowIter`

## `TryRng` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<R> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:203 |
| `Coerced<T, R>` | `<R> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:203 |
| `ExternalPtr<T>` | `<R> +2wc` | blanket | 4 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:203 |
| `RRng` | `` | concrete | 4 | miniextendr-api/src/optionals/rand_impl.rs:128 |

### `TryRng` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:203** (3 impls): `Altrep<T>`, `Coerced<T, R>`, `ExternalPtr<T>`

## `RngCore` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:259 |
| `RRng` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:259 |
| `Coerced<T, R>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:259 |
| `ExternalPtr<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:259 |

### `RngCore` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:259** (4 impls): `Altrep<T>`, `RRng`, `Coerced<T, R>`, `ExternalPtr<T>`

## `TryRngCore` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<R> +1wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:270 |
| `RRng` | `<R> +1wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:270 |
| `Coerced<T, R>` | `<R> +1wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:270 |
| `ExternalPtr<T>` | `<R> +1wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:270 |

### `TryRngCore` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:270** (4 impls): `Altrep<T>`, `RRng`, `Coerced<T, R>`, `ExternalPtr<T>`

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

## `RngExt` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RRng` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/rng.rs:317 |
| `Coerced<T, R>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/rng.rs:317 |
| `ExternalPtr<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/rng.rs:317 |
| `Altrep<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/rng.rs:317 |

### `RngExt` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand-0.10.1/src/rng.rs:317** (4 impls): `RRng`, `Coerced<T, R>`, `ExternalPtr<T>`, `Altrep<T>`

## `Comparable` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:104 |
| `RWrapperPriority` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:104 |
| `AsSerialize<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:104 |
| `ExternalPtr<T>` | `<Q, K> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:104 |

### `Comparable` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/equivalent-1.0.2/src/lib.rs:104** (4 impls): `Coerced<T, R>`, `RWrapperPriority`, `AsSerialize<T>`, `ExternalPtr<T>`

## `TryCryptoRng` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<R> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:251 |
| `ExternalPtr<T>` | `<R> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:251 |
| `Altrep<T>` | `<R> +2wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:251 |

### `TryCryptoRng` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:251** (3 impls): `Coerced<T, R>`, `ExternalPtr<T>`, `Altrep<T>`

## `AsNamedListExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:882 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:883 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:884 |

## `IntoRAltrep` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JiffZonedVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/into_r.rs:2065 |
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2065 |
| `JiffTimestampVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/into_r.rs:2065 |

### `IntoRAltrep` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r.rs:2065** (3 impls): `JiffZonedVec`, `T`, `JiffTimestampVec`

## `AsNamedVectorExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:901 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:902 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:903 |

## `CryptoRng` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Altrep<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:97 |
| `Coerced<T, R>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:97 |
| `ExternalPtr<T>` | `<R> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:97 |

### `CryptoRng` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_core-0.10.1/src/lib.rs:97** (3 impls): `Altrep<T>`, `Coerced<T, R>`, `ExternalPtr<T>`

## `AsRNativeExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:838 |
| `RLogical` | `<T> +1wc` | blanket | 0 | miniextendr-api/src/convert.rs:838 |
| `Rcomplex` | `<T> +1wc` | blanket | 0 | miniextendr-api/src/convert.rs:838 |

### `AsRNativeExt` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/convert.rs:838** (3 impls): `T`, `RLogical`, `Rcomplex`

## `ParallelBridge` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `StrVecIter` | `<T> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.12.0/src/iter/par_bridge.rs:58 |
| `StrVecCowIter` | `<T> +2wc` | blanket | 1 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.12.0/src/iter/par_bridge.rs:58 |

### `ParallelBridge` — for-types sharing a source span (likely macro-expanded / co-located)

- **/Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.12.0/src/iter/par_bridge.rs:58** (2 impls): `StrVecIter`, `StrVecCowIter`

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

## `AsDataFrameExt` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SerdeRows<T>` | `<T> +1wc` | blanket | 0 | miniextendr-api/src/convert.rs:851 |
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:851 |

### `AsDataFrameExt` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/convert.rs:851** (2 impls): `SerdeRows<T>`, `T`

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

## `CheckedBitPattern` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `<T> +1wc` | blanket | 2 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytemuck-1.25.0/src/checked.rs:143 |

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

## `StorageMut` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, R, C>` | `<S, T, R, C> +4wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nalgebra-0.34.2/src/base/storage.rs:275 |

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

## `AnyBitPattern` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytemuck-1.25.0/src/anybitpattern.rs:56 |

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

## `NoUninit` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `<T> +1wc` | blanket | 0 | /Users/elea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bytemuck-1.25.0/src/no_uninit.rs:72 |

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
