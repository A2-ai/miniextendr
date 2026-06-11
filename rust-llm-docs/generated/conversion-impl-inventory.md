# Trait impl inventory

Source: `target/doc/miniextendr_api.json`

Traits with impls: 9

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `TryFromSexp` | 434 | 434 |
| `IntoR` | 325 | 325 |
| `IntoRAs` | 135 | 135 |
| `TryCoerce` | 95 | 93 |
| `Coerce` | 53 | 53 |
| `AltrepSerialize` | 27 | 27 |
| `IntoRAltrep` | 3 | 1 |
| `RSerializeNative` | 1 | 1 |
| `RDeserializeNative` | 1 | 1 |

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

## `IntoRAltrep` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2065 |

## `RSerializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:73 |

## `RDeserializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:146 |
