# Trait impl inventory

Source: `target/doc/miniextendr_api.json`

Traits with impls: 9

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `TryFromSexp` | 439 | 439 |
| `IntoR` | 322 | 322 |
| `IntoRAs` | 135 | 135 |
| `TryCoerce` | 95 | 93 |
| `Coerce` | 53 | 53 |
| `AltrepSerialize` | 27 | 27 |
| `IntoRAltrep` | 3 | 1 |
| `RDeserializeNative` | 1 | 1 |
| `RSerializeNative` | 1 | 1 |

## `TryFromSexp` — 439 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AltrepSexp` | `` | concrete | 3 | miniextendr-api/src/altrep_sexp.rs:282 |
| `AsFromStr<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:757 |
| `AsFromStrVec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:799 |
| `DataFrame` | `` | concrete | 2 | miniextendr-api/src/dataframe.rs:485 |
| `Factor<''a>` | `<'a>` | concrete | 2 | miniextendr-api/src/factor.rs:222 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:488 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:538 |
| `Vec<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1092 |
| `Vec<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1093 |
| `Vec<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1094 |
| `Vec<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1095 |
| `Vec<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1096 |
| `Vec<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1097 |
| `Vec<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1098 |
| `Vec<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1099 |
| `Vec<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1100 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1103 |
| `Box<[bool]>` | `` | concrete | 2 | miniextendr-api/src/from_r.rs:1124 |
| `std::collections::HashSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1168 |
| `std::collections::HashSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1169 |
| `std::collections::HashSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1170 |
| `std::collections::HashSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1171 |
| `std::collections::HashSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1172 |
| `std::collections::HashSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1173 |
| `std::collections::HashSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1174 |
| `std::collections::HashSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1175 |
| `std::collections::BTreeSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1177 |
| `std::collections::BTreeSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1178 |
| `std::collections::BTreeSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1179 |
| `std::collections::BTreeSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1180 |
| `std::collections::BTreeSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1181 |
| `std::collections::BTreeSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1182 |
| `std::collections::BTreeSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1183 |
| `std::collections::BTreeSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1184 |
| `std::collections::HashSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1203 |
| `std::collections::BTreeSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1204 |
| `crate::externalptr::ExternalPtr<T>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1226 |
| `Option<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1273 |
| `i32` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:399 |
| `f64` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:475 |
| `u8` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:476 |
| `crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:477 |
| `crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:478 |
| `crate::SEXP` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:504 |
| `Option<crate::SEXP>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:524 |
| `&[T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:559 |
| `&mut [T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:597 |
| `Option<&[T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:633 |
| `Option<&mut [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:660 |
| `Result<T, ()>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:691 |
| `[T; N]` | `<T, N> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:728 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:766 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:791 |
| `Option<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:817 |
| `Option<std::collections::HashMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:868 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:872 |
| `Option<std::collections::HashSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:901 |
| `Option<std::collections::BTreeSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:905 |
| `Vec<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:916 |
| `crate::coerce::Coerced<T, R>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/from_r.rs:987 |
| `i8` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:263 |
| `i16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:277 |
| `u16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:291 |
| `u32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:305 |
| `f32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:319 |
| `Option<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:383 |
| `Option<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:397 |
| `Option<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:411 |
| `Option<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:425 |
| `Option<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:439 |
| `i64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:564 |
| `u64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:582 |
| `Option<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:597 |
| `Option<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:611 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/from_r/coerced_scalars.rs:625 |
| `Option<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:673 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/from_r/coerced_scalars.rs:739 |
| `Option<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:782 |
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
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:232 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:233 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:234 |
| `Box<[crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:235 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:236 |
| `std::collections::HashMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:38 |
| `std::collections::BTreeMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:45 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:116 |
| `Box<[std::borrow::Cow<''static, str>]>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:149 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:176 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:209 |
| `Vec<&''static str>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:221 |
| `Vec<Option<&''static str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:255 |
| `std::collections::HashSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:302 |
| `std::collections::BTreeSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:306 |
| `std::borrow::Cow<''static, [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:34 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:393 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:393 |
| `Vec<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:393 |
| `Option<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:393 |
| `Option<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:416 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:416 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:416 |
| `Vec<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:416 |
| `std::borrow::Cow<''static, str>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:60 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:84 |
| `Option<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:112 |
| `Option<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:136 |
| `Option<i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:158 |
| `Option<f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:190 |
| `Option<u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:222 |
| `Option<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:244 |
| `crate::Rboolean` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:26 |
| `Option<crate::Rboolean>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:56 |
| `bool` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:86 |
| `Box<[Option<bool>]>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:126 |
| `Vec<crate::Rboolean>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:136 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:166 |
| `Vec<crate::altrep_data::Logical>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:197 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:220 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:245 |
| `Box<[Option<String>]>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:276 |
| `Vec<Option<u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:286 |
| `Vec<Option<i8>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:383 |
| `Vec<Option<i16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:384 |
| `Vec<Option<u16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:385 |
| `Vec<Option<u32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:386 |
| `Vec<Option<i64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:387 |
| `Vec<Option<u64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:388 |
| `Vec<Option<isize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:389 |
| `Vec<Option<usize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:390 |
| `Vec<Option<f32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:391 |
| `Vec<Option<f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:55 |
| `Vec<Option<i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:56 |
| `Box<[Option<f64>]>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:87 |
| `Box<[Option<i32>]>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:88 |
| `Vec<Option<bool>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:91 |
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
| `Option<&''static i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
| `&''static mut i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:448 |
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
| `Option<&''static f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `&''static mut f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:449 |
| `Vec<Option<&''static mut u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static mut [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static mut [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&''static u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `&''static mut u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `&''static u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Option<&''static mut u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<Option<&''static u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `Vec<&''static mut u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:450 |
| `&''static crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:451 |
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
| `Vec<&''static mut [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static mut [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `&''static mut crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `&''static crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static mut crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static mut crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static mut crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<&''static [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Vec<Option<&''static [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:452 |
| `Option<&''static str>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:121 |
| `char` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:199 |
| `String` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:245 |
| `Option<String>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:317 |
| `&''static str` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:47 |
| `List` | `` | concrete | 2 | miniextendr-api/src/list.rs:1184 |
| `Option<List>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1239 |
| `Option<ListMut>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1251 |
| `ListMut` | `` | concrete | 2 | miniextendr-api/src/list.rs:1263 |
| `NamedList` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:163 |
| `Option<NamedList>` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:173 |
| `Missing<T>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/missing.rs:253 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:328 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:343 |
| `Option<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:102 |
| `Vec<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:103 |
| `Vec<Option<AhoCorasick>>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:104 |
| `AhoCorasick` | `` | concrete | 2 | miniextendr-api/src/optionals/aho_corasick_impl.rs:60 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1002 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1070 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1257 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1297 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1310 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1325 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:445 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:489 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:532 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:569 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:601 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:663 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:716 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:172 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:216 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:261 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:305 |
| `BitVec<u8, Msb0>` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:140 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:171 |
| `RBitVec` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:62 |
| `Option<RBitVec>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:93 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:56 |
| `Bytes` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:378 |
| `BytesMut` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:412 |
| `Option<Bytes>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:427 |
| `Option<BytesMut>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:448 |
| `Either<L, R>` | `<L, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:96 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:62 |
| `Vec<SignedDuration>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1015 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1066 |
| `Option<Timestamp>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:113 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1493 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1493 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1534 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1534 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:1605 |
| `Vec<Timestamp>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:174 |
| `Vec<Option<Timestamp>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:233 |
| `Date` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:297 |
| `Option<Date>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:358 |
| `Vec<Date>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:421 |
| `Vec<Option<Date>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:482 |
| `Timestamp` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:54 |
| `Zoned` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:546 |
| `Option<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:629 |
| `Vec<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:685 |
| `Vec<Option<Zoned>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:794 |
| `SignedDuration` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:906 |
| `Option<SignedDuration>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:961 |
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
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:2866 |
| `ArrayD<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:300 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3011 |
| `Array1<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array4<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `ArrayD<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array3<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array6<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array2<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array5<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:665 |
| `Array3<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array6<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array2<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array5<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array1<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array4<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `ArrayD<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:666 |
| `Array3<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array6<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array2<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array5<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array1<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `Array4<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
| `ArrayD<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:667 |
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
| `Array1<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array4<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `ArrayD<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array3<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array6<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array2<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array5<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array3<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array6<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array2<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array5<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array1<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array4<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `ArrayD<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array6<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array2<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array5<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array1<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array4<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `ArrayD<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array3<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
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
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:194 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:206 |
| `Option<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:218 |
| `Option<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:230 |
| `Vec<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:242 |
| `Vec<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:259 |
| `Vec<Option<BigInt>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:276 |
| `Vec<Option<BigUint>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:291 |
| `Option<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:140 |
| `Vec<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:193 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:249 |
| `Complex<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:92 |
| `OrderedFloat<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:118 |
| `OrderedFloat<f32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:126 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:134 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:143 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:152 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:170 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:188 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:197 |
| `Vec<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:102 |
| `Vec<Option<Regex>>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:119 |
| `Regex` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:68 |
| `Option<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:85 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:146 |
| `Option<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:179 |
| `Vec<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:206 |
| `Vec<Option<Decimal>>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:251 |
| `JsonValue` | `` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:573 |
| `Option<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:585 |
| `Vec<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:586 |
| `Vec<Option<JsonValue>>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:587 |
| `Option<OffsetDateTime>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:140 |
| `Vec<OffsetDateTime>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:208 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:278 |
| `Date` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:350 |
| `Option<Date>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:415 |
| `Vec<Date>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:482 |
| `Vec<Option<Date>>` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:547 |
| `OffsetDateTime` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:67 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:102 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:178 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:227 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:305 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:339 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:78 |
| `TomlValue` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:117 |
| `Option<TomlValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/toml_impl.rs:162 |
| `Vec<TomlValue>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:165 |
| `Vec<Option<TomlValue>>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:188 |
| `Option<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:100 |
| `Vec<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:155 |
| `Vec<Option<Url>>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:213 |
| `Url` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:51 |
| `Vec<Option<Uuid>>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:159 |
| `Uuid` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:44 |
| `Option<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:69 |
| `Vec<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:99 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:670 |
| `RArray<i8, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:777 |
| `RArray<i16, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:778 |
| `RArray<i64, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:779 |
| `RArray<isize, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:780 |
| `RArray<u16, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:781 |
| `RArray<u32, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:782 |
| `RArray<u64, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:783 |
| `RArray<usize, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:784 |
| `RArray<f32, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:787 |
| `RArray<bool, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:790 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:492 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:503 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:514 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:550 |
| `FromJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:97 |
| `StrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:428 |
| `ProtectedStrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:668 |

### `TryFromSexp` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/from_r/references.rs:450** (12 impls): `Vec<Option<&''static mut u8>>`, `Vec<&''static [u8]>`, `Vec<Option<&''static [u8]>>`, `Vec<&''static mut [u8]>`, `Vec<Option<&''static mut [u8]>>`, `Option<&''static u8>`, `&''static mut u8`, `&''static u8`, `Option<&''static mut u8>`, `Vec<&''static u8>`, `Vec<Option<&''static u8>>`, `Vec<&''static mut u8>`
- **miniextendr-api/src/from_r/references.rs:452** (12 impls): `Vec<&''static mut [crate::Rcomplex]>`, `Vec<Option<&''static mut [crate::Rcomplex]>>`, `Option<&''static crate::Rcomplex>`, `&''static mut crate::Rcomplex`, `&''static crate::Rcomplex`, `Option<&''static mut crate::Rcomplex>`, `Vec<&''static crate::Rcomplex>`, `Vec<Option<&''static crate::Rcomplex>>`, `Vec<&''static mut crate::Rcomplex>`, `Vec<Option<&''static mut crate::Rcomplex>>`, `Vec<&''static [crate::Rcomplex]>`, `Vec<Option<&''static [crate::Rcomplex]>>`
- **miniextendr-api/src/from_r/references.rs:449** (12 impls): `&''static f64`, `Option<&''static mut f64>`, `Vec<&''static f64>`, `Vec<Option<&''static f64>>`, `Vec<&''static mut f64>`, `Vec<Option<&''static mut f64>>`, `Vec<&''static [f64]>`, `Vec<Option<&''static [f64]>>`, `Vec<&''static mut [f64]>`, `Vec<Option<&''static mut [f64]>>`, `Option<&''static f64>`, `&''static mut f64`
- **miniextendr-api/src/from_r/references.rs:448** (12 impls): `&''static i32`, `Option<&''static mut i32>`, `Vec<&''static i32>`, `Vec<Option<&''static i32>>`, `Vec<&''static mut i32>`, `Vec<Option<&''static mut i32>>`, `Vec<&''static [i32]>`, `Vec<Option<&''static [i32]>>`, `Vec<&''static mut [i32]>`, `Vec<Option<&''static mut [i32]>>`, `Option<&''static i32>`, `&''static mut i32`
- **miniextendr-api/src/from_r/references.rs:451** (12 impls): `&''static crate::RLogical`, `Option<&''static mut crate::RLogical>`, `Vec<&''static crate::RLogical>`, `Vec<Option<&''static crate::RLogical>>`, `Vec<&''static mut crate::RLogical>`, `Vec<Option<&''static mut crate::RLogical>>`, `Vec<&''static [crate::RLogical]>`, `Vec<Option<&''static [crate::RLogical]>>`, `Vec<&''static mut [crate::RLogical]>`, `Vec<Option<&''static mut [crate::RLogical]>>`, `Option<&''static crate::RLogical>`, `&''static mut crate::RLogical`
- **miniextendr-api/src/optionals/ndarray_impl.rs:667** (7 impls): `Array3<i64>`, `Array6<i64>`, `Array2<i64>`, `Array5<i64>`, `Array1<i64>`, `Array4<i64>`, `ArrayD<i64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:670** (7 impls): `Array1<u32>`, `Array4<u32>`, `ArrayD<u32>`, `Array3<u32>`, `Array6<u32>`, `Array2<u32>`, `Array5<u32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:672** (7 impls): `Array6<usize>`, `Array2<usize>`, `Array5<usize>`, `Array1<usize>`, `Array4<usize>`, `ArrayD<usize>`, `Array3<usize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:665** (7 impls): `Array1<i8>`, `Array4<i8>`, `ArrayD<i8>`, `Array3<i8>`, `Array6<i8>`, `Array2<i8>`, `Array5<i8>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:676** (7 impls): `Array2<f32>`, `Array5<f32>`, `Array1<f32>`, `Array4<f32>`, `ArrayD<f32>`, `Array3<f32>`, `Array6<f32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:668** (7 impls): `Array2<isize>`, `Array5<isize>`, `Array1<isize>`, `Array4<isize>`, `ArrayD<isize>`, `Array3<isize>`, `Array6<isize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:671** (7 impls): `Array3<u64>`, `Array6<u64>`, `Array2<u64>`, `Array5<u64>`, `Array1<u64>`, `Array4<u64>`, `ArrayD<u64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:680** (7 impls): `Array1<bool>`, `Array4<bool>`, `ArrayD<bool>`, `Array3<bool>`, `Array6<bool>`, `Array2<bool>`, `Array5<bool>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:666** (7 impls): `Array3<i16>`, `Array6<i16>`, `Array2<i16>`, `Array5<i16>`, `Array1<i16>`, `Array4<i16>`, `ArrayD<i16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:669** (7 impls): `Array1<u16>`, `Array4<u16>`, `ArrayD<u16>`, `Array3<u16>`, `Array6<u16>`, `Array2<u16>`, `Array5<u16>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:393** (4 impls): `Vec<Option<std::path::PathBuf>>`, `std::path::PathBuf`, `Vec<std::path::PathBuf>`, `Option<std::path::PathBuf>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:416** (4 impls): `Option<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `std::ffi::OsString`, `Vec<std::ffi::OsString>`
- **miniextendr-api/src/optionals/jiff_impl.rs:1534** (2 impls): `JiffZonedVecRef`, `JiffZonedVecMut`
- **miniextendr-api/src/optionals/jiff_impl.rs:1493** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`

## `IntoR` — 322 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CollectNAInt<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:1003 |
| `AsExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:263 |
| `AsRNative<T>` | `<T>` | concrete | 5 | miniextendr-api/src/convert.rs:315 |
| `AsNamedList<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:387 |
| `AsNamedList<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:408 |
| `AsNamedList<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:429 |
| `AsNamedVector<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:482 |
| `AsNamedVector<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:498 |
| `AsNamedVector<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:514 |
| `AsDisplay<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:696 |
| `AsDisplayVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:723 |
| `Collect<I>` | `<I, T> +2wc` | concrete | 5 | miniextendr-api/src/convert.rs:868 |
| `CollectStrings<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:923 |
| `AsList<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:94 |
| `CollectNA<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:961 |
| `DataFrame` | `` | concrete | 4 | miniextendr-api/src/dataframe.rs:493 |
| `FactorVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:472 |
| `FactorOptionVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:571 |
| `Option<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1012 |
| `Option<std::collections::HashMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1039 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1066 |
| `Option<std::collections::HashSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1093 |
| `Option<std::collections::BTreeSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1120 |
| `Option<std::collections::HashSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1177 |
| `Option<std::collections::BTreeSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1181 |
| `Vec<String>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1226 |
| `&[String]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1244 |
| `Box<[String]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1262 |
| `&[&str]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1279 |
| `Vec<std::borrow::Cow<''_, str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1297 |
| `crate::SEXP` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:130 |
| `Box<[std::borrow::Cow<''_, str>]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1314 |
| `Vec<Option<std::borrow::Cow<''_, str>>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1333 |
| `Vec<Option<&str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1386 |
| `Vec<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1437 |
| `Vec<Vec<T>>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:1458 |
| `Vec<&[T]>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1506 |
| `Vec<&[String]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1546 |
| `Vec<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1584 |
| `Vec<Option<f64>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1665 |
| `Vec<Option<i32>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1666 |
| `()` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:169 |
| `Vec<Option<i64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1713 |
| `Vec<Option<u64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1714 |
| `Vec<Option<isize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1715 |
| `Vec<Option<usize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1717 |
| `Vec<Option<i8>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1744 |
| `Vec<Option<i16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1745 |
| `Vec<Option<u16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1746 |
| `Vec<Option<u32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1747 |
| `Vec<Option<f32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1748 |
| `Vec<bool>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1781 |
| `Box<[bool]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1801 |
| `&[bool]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1820 |
| `std::convert::Infallible` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:185 |
| `Vec<Option<bool>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1863 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1872 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1880 |
| `Vec<Option<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1892 |
| `(A, B)` | `<A, B>` | concrete | 5 | miniextendr-api/src/into_r.rs:2004 |
| `(A, B, C)` | `<A, B, C>` | concrete | 5 | miniextendr-api/src/into_r.rs:2005 |
| `(A, B, C, D)` | `<A, B, C, D>` | concrete | 5 | miniextendr-api/src/into_r.rs:2006 |
| `(A, B, C, D, E)` | `<A, B, C, D, E>` | concrete | 5 | miniextendr-api/src/into_r.rs:2007 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F>` | concrete | 5 | miniextendr-api/src/into_r.rs:2008 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G>` | concrete | 5 | miniextendr-api/src/into_r.rs:2009 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H>` | concrete | 5 | miniextendr-api/src/into_r.rs:2010 |
| `Vec<Box<[T]>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2113 |
| `Vec<Box<[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2143 |
| `Vec<[T; N]>` | `<T, N> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2171 |
| `Vec<Option<Vec<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2239 |
| `Vec<Option<Vec<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2256 |
| `i32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:226 |
| `f64` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:227 |
| `Vec<Option<std::collections::HashSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2270 |
| `u8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:228 |
| `Vec<Option<std::collections::HashSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2287 |
| `crate::Rcomplex` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:229 |
| `Vec<Option<std::collections::BTreeSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2301 |
| `Option<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:231 |
| `Vec<Option<std::collections::BTreeSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2318 |
| `Vec<Option<std::collections::HashMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2332 |
| `Vec<Option<std::collections::BTreeMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2346 |
| `Vec<Option<&[T]>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r.rs:2362 |
| `Vec<Option<&[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2378 |
| `Vec<std::collections::HashSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2395 |
| `Vec<std::collections::BTreeSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2414 |
| `Vec<std::collections::HashSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2432 |
| `Vec<std::collections::BTreeSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2448 |
| `Vec<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2481 |
| `Vec<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2485 |
| `i8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:291 |
| `i16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:292 |
| `u16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:293 |
| `f32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:296 |
| `u32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:297 |
| `Vec<i32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:336 |
| `Vec<f64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:337 |
| `Vec<u8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:338 |
| `Vec<crate::RLogical>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:339 |
| `Vec<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:340 |
| `&[T]` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:342 |
| `Box<[T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:365 |
| `Vec<i8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:526 |
| `&[i8]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:526 |
| `Vec<i16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:527 |
| `&[i16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:527 |
| `Vec<u16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:528 |
| `&[u16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:528 |
| `Vec<f32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:531 |
| `&[f32]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:531 |
| `Vec<i64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:589 |
| `Vec<u64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:590 |
| `Vec<isize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:591 |
| `Vec<usize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:593 |
| `[T; N]` | `<T, N>` | concrete | 5 | miniextendr-api/src/into_r.rs:609 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:635 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:665 |
| `std::borrow::Cow<''_, [T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:712 |
| `std::borrow::Cow<''_, str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:738 |
| `std::path::PathBuf` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:905 |
| `&std::path::Path` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:905 |
| `Option<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:905 |
| `Vec<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:905 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:905 |
| `Option<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:921 |
| `Vec<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:921 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:921 |
| `std::ffi::OsString` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:921 |
| `&std::ffi::OsStr` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:921 |
| `HashSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:974 |
| `BTreeSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:974 |
| `BTreeSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:975 |
| `HashSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:975 |
| `HashSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:976 |
| `BTreeSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:976 |
| `Option<Vec<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
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
| `T` | `<T>` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:401 |
| `String` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:446 |
| `char` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:466 |
| `&str` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:491 |
| `i64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:50 |
| `Option<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:516 |
| `Option<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:553 |
| `Option<&T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:580 |
| `u64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:85 |
| `Result<T, NullOnErr>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/result.rs:126 |
| `Result<T, E>` | `<T, E> +2wc` | concrete | 4 | miniextendr-api/src/into_r/result.rs:69 |
| `List` | `` | concrete | 4 | miniextendr-api/src/list.rs:1053 |
| `ListMut` | `` | concrete | 4 | miniextendr-api/src/list.rs:1067 |
| `Vec<List>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1086 |
| `Vec<Option<List>>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1115 |
| `NamedList` | `` | concrete | 4 | miniextendr-api/src/list/named.rs:149 |
| `Vec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/match_arg.rs:276 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:287 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:306 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1095 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1134 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1170 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1202 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1226 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1291 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1292 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1293 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1338 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1432 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1483 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:796 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:843 |
| `TimestampSecondArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:874 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:352 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:371 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:386 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:402 |
| `Option<RBitVec>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:121 |
| `BitVec<u8, Msb0>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:173 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:196 |
| `RBitVec` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:98 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:38 |
| `Bytes` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:359 |
| `BytesMut` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:393 |
| `Option<Bytes>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:469 |
| `Option<BytesMut>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:493 |
| `Either<L, R>` | `<L, R> +2wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:145 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:127 |
| `Vec<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:1044 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:1093 |
| `Option<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:147 |
| `JiffTimestampVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:1493 |
| `JiffZonedVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:1534 |
| `Vec<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:207 |
| `Vec<Option<Timestamp>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:266 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:332 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:394 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:455 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:514 |
| `Zoned` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:600 |
| `Option<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:658 |
| `Vec<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:739 |
| `Vec<Option<Zoned>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:847 |
| `Timestamp` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:87 |
| `SignedDuration` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:937 |
| `Option<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:992 |
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
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:2878 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3023 |
| `Array0<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:325 |
| `Array1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:342 |
| `Array2<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:689 |
| `Array3<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:731 |
| `Array4<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:778 |
| `Array5<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:823 |
| `Array6<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `ArrayD<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:910 |
| `ArcArray1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:992 |
| `BigInt` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:306 |
| `BigUint` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:319 |
| `Option<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:332 |
| `Option<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:345 |
| `Vec<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:358 |
| `Vec<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:374 |
| `Vec<Option<BigInt>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:390 |
| `Vec<Option<BigUint>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:406 |
| `Complex<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:124 |
| `Option<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:174 |
| `Vec<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:223 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:278 |
| `OrderedFloat<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:209 |
| `OrderedFloat<f32>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:222 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:235 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:248 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:261 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:277 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:293 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:309 |
| `Decimal` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:287 |
| `Option<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:300 |
| `Vec<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:313 |
| `Vec<Option<Decimal>>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:329 |
| `JsonValue` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:592 |
| `Option<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:605 |
| `Vec<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:621 |
| `Vec<Option<JsonValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:648 |
| `Table` | `` | concrete | 4 | miniextendr-api/src/optionals/tabled_impl.rs:196 |
| `OffsetDateTime` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:110 |
| `Option<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:182 |
| `Vec<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:248 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:315 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:386 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:454 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:519 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:581 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:140 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:159 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:202 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:251 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:380 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:402 |
| `TomlValue` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:209 |
| `Option<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:231 |
| `Vec<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:247 |
| `Vec<Option<TomlValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:274 |
| `Option<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:138 |
| `Vec<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:195 |
| `Vec<Option<Url>>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:251 |
| `Url` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:84 |
| `Vec<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:140 |
| `Vec<Option<Uuid>>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:177 |
| `Uuid` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:53 |
| `Option<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:83 |
| `RArray<T, {'expr': 'NDIM', 'value': None, 'is_literal': False}>` | `<T, NDIM>` | concrete | 5 | miniextendr-api/src/rarray.rs:795 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:403 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:417 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:431 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:460 |
| `DataFrameShape` | `` | concrete | 3 | miniextendr-api/src/serde/columnar.rs:2567 |
| `AsJsonVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:131 |
| `AsJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:34 |
| `AsJsonPretty<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:58 |
| `AsSerialize<T>` | `<T>` | concrete | 2 | miniextendr-api/src/serde/traits.rs:262 |
| `StrVec` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:414 |
| `ProtectedStrVec` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:654 |

### `IntoR` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r.rs:905** (5 impls): `std::path::PathBuf`, `&std::path::Path`, `Option<std::path::PathBuf>`, `Vec<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`
- **miniextendr-api/src/into_r.rs:921** (5 impls): `Option<std::ffi::OsString>`, `Vec<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `std::ffi::OsString`, `&std::ffi::OsStr`
- **miniextendr-api/src/into_r.rs:974** (2 impls): `HashSet<i8>`, `BTreeSet<i8>`
- **miniextendr-api/src/into_r.rs:975** (2 impls): `BTreeSet<i16>`, `HashSet<i16>`
- **miniextendr-api/src/into_r.rs:976** (2 impls): `HashSet<u16>`, `BTreeSet<u16>`
- **miniextendr-api/src/into_r.rs:526** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r.rs:527** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r.rs:528** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r.rs:531** (2 impls): `Vec<f32>`, `&[f32]`

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
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:372 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:386 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:400 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:407 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:414 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:421 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:428 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:435 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:443 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:444 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:445 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:446 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:449 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:472 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:480 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:481 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:482 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:483 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:484 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:485 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:486 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:487 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:488 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:489 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:490 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:495 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:502 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:510 |
| `String` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:529 |
| `&str` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:536 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:544 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:562 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:569 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:576 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:583 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:590 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:634 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:641 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:648 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:648 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:649 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:649 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:650 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:650 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:651 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:651 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:652 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:652 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:653 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:653 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:654 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:654 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:655 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:655 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:656 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:656 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:657 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:657 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:658 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:658 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:661 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:668 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:707 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:721 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:736 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:752 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:787 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:787 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:788 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:788 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:789 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:789 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:790 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:790 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:791 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:791 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:792 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:792 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:795 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:795 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:796 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:796 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:797 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:797 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:798 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:798 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:801 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:811 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:850 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:857 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:864 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:864 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:865 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:865 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:866 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:866 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:867 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:867 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:868 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:868 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:869 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:869 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:870 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:870 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:871 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:871 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:872 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:872 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:873 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:873 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:874 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:874 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:879 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:886 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:896 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:903 |
| `Vec<&str>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:910 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:917 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:925 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:947 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:954 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:961 |

### `IntoRAs` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r_as.rs:871** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:789** (2 impls): `Vec<i32>`, `&[i32]`
- **miniextendr-api/src/into_r_as.rs:873** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r_as.rs:790** (2 impls): `&[u8]`, `Vec<u8>`
- **miniextendr-api/src/into_r_as.rs:648** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r_as.rs:874** (2 impls): `&[f64]`, `Vec<f64>`
- **miniextendr-api/src/into_r_as.rs:792** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:649** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:795** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:651** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:797** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:652** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:798** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:654** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:655** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:864** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:657** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r_as.rs:866** (2 impls): `Vec<i32>`, `&[i32]`
- **miniextendr-api/src/into_r_as.rs:658** (2 impls): `&[f64]`, `Vec<f64>`
- **miniextendr-api/src/into_r_as.rs:867** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:869** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:870** (2 impls): `&[u32]`, `Vec<u32>`
- **miniextendr-api/src/into_r_as.rs:788** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:872** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:791** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:650** (2 impls): `Vec<u8>`, `&[u8]`
- **miniextendr-api/src/into_r_as.rs:796** (2 impls): `Vec<u64>`, `&[u64]`
- **miniextendr-api/src/into_r_as.rs:653** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:656** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:865** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:868** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:787** (2 impls): `Vec<i8>`, `&[i8]`

## `TryCoerce` — 95 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:1005 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:1013 |
| `T` | `<T, R> +1wc` | concrete | 2 | miniextendr-api/src/coerce.rs:112 |
| `Rcomplex` | `<T, R> +1wc` | blanket | 2 | miniextendr-api/src/coerce.rs:112 |
| `Rboolean` | `<T, R> +1wc` | blanket | 2 | miniextendr-api/src/coerce.rs:112 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:315 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:325 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:335 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:366 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:367 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:368 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:369 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:370 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:371 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:372 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:373 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:374 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:375 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:378 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:388 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:398 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:409 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:420 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:431 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:442 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:453 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:464 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:497 |
| `crate::Rboolean` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:512 |
| `crate::RLogical` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:521 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:545 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:546 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:547 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:548 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:549 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:566 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:567 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:568 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:569 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:570 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:571 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:572 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:573 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:574 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:629 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:630 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:631 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:632 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:633 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:634 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:635 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:636 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:653 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:654 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:655 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:656 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:657 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:658 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:659 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:676 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:677 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:678 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:679 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:680 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:681 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:682 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:683 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:684 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:689 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:710 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:731 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:755 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:776 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:796 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:817 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:829 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:869 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:891 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:912 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:934 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:975 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:992 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:111 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:122 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:133 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:144 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:155 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:57 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:83 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:40 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:78 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:104 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:119 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:74 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:91 |

### `TryCoerce` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/coerce.rs:112** (3 impls): `T`, `Rcomplex`, `Rboolean`

## `Coerce` — 53 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `&[T]` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:1097 |
| `Vec<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:1105 |
| `Box<[T]>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:1113 |
| `std::collections::VecDeque<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:1121 |
| `tinyvec::TinyVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1139 |
| `tinyvec::ArrayVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1154 |
| `(A, B)` | `<A, B, RA, RB> +2wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1184 |
| `(A, B, C)` | `<A, B, C, RA, RB, RC> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1185 |
| `(A, B, C, D)` | `<A, B, C, D, RA, RB, RC, RD> +4wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1186 |
| `(A, B, C, D, E)` | `<A, B, C, D, E, RA, RB, RC, RD, RE> +5wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1187 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F, RA, RB, RC, RD, RE, RF> +6wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1188 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G, RA, RB, RC, RD, RE, RF, RG> +7wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1193 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H, RA, RB, RC, RD, RE, RF, RG, RH> +8wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1198 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:142 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:143 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:144 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:145 |
| `crate::Rcomplex` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:146 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:155 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:166 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:176 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:183 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:190 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:197 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:204 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:211 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:221 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:232 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:239 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:246 |
| `Option<f64>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:257 |
| `Option<i32>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:265 |
| `Option<bool>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:273 |
| `Option<crate::Rboolean>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:285 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:299 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:307 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:579 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:586 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:593 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:600 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:607 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:786 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:25 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:33 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:41 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:49 |
| `OrderedFloat<f32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:101 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:24 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:32 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:70 |
| `OrderedFloat<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:93 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:58 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:66 |

## `AltrepSerialize` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:122 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:123 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:124 |
| `Vec<bool>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:125 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:126 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:127 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:128 |
| `Vec<std::borrow::Cow<''static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:131 |
| `Vec<Option<std::borrow::Cow<''static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:132 |
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:140 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:154 |
| `Cow<''static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1563 |
| `Cow<''static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1649 |
| `Cow<''static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1673 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:168 |
| `Cow<''static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1697 |
| `Box<[bool]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:182 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:196 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:210 |
| `std::ops::Range<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:718 |
| `std::ops::Range<i64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:731 |
| `std::ops::Range<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:754 |
| `Float64Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1727 |
| `Int32Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1750 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1773 |
| `BooleanArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1792 |
| `StringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1801 |

## `IntoRAltrep` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JiffTimestampVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/into_r.rs:2095 |
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2095 |
| `JiffZonedVec` | `<T> +1wc` | blanket | 2 | miniextendr-api/src/into_r.rs:2095 |

### `IntoRAltrep` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r.rs:2095** (3 impls): `JiffTimestampVec`, `T`, `JiffZonedVec`

## `RDeserializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:146 |

## `RSerializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:73 |
