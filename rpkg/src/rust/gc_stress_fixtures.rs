//! Fixtures for GC stress tests.
//!
//! Provides `SharedData` (R6 class) and `into_sexp_altrep` for the GC stress
//! and ALTREP serialization test suites.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::SEXPTYPE;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::{IntoRAltrep, miniextendr};
#[cfg(feature = "jiff")]
use miniextendr_api::{JiffZonedVec, Timestamp};

use crate::dataframe_enum_payload_matrix::{
    BTreeMapEvent, Direction, HashMapEvent, NestedFactorEvent, NestedFlattenEvent, NestedListEvent,
    Point, Status, StructFlattenEvent, StructListEvent,
};

/// Simple R6 class for GC stress tests.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SharedData {
    x: f64,
    y: f64,
    label: String,
}

/// @param x Numeric x-coordinate.
/// @param y Numeric y-coordinate.
/// @param label Character label.
#[miniextendr(r6)]
impl SharedData {
    pub fn new(x: f64, y: f64, label: String) -> Self {
        SharedData { x, y, label }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

/// Exercise `Vec<Option<collection>>` conversions under GC pressure.
///
/// Allocates `Vec<Option<Vec<i32>>>`, `Vec<Option<HashSet<String>>>`, and
/// `Vec<Option<BTreeSet<i32>>>` and converts each to SEXP, verifying that the
/// `OwnedProtect` in `vec_option_of_into_r_to_list` keeps the outer list live
/// across inner `into_sexp()` calls.
#[miniextendr]
pub fn gc_stress_vec_option_collection() {
    // Vec<Option<Vec<i32>>>: mix of Some and None
    let vec_opt: Vec<Option<Vec<i32>>> = vec![
        Some(vec![1, 2, 3]),
        None,
        Some(vec![4, 5]),
        None,
        Some(vec![]),
    ];
    let _ = vec_opt.into_sexp();

    // Vec<Option<HashSet<String>>>: some with multiple strings, some None
    let hs_opt: Vec<Option<HashSet<String>>> = vec![
        Some(["a", "b", "c"].iter().map(|s| s.to_string()).collect()),
        None,
        Some(["d"].iter().map(|s| s.to_string()).collect()),
    ];
    let _ = hs_opt.into_sexp();

    // Vec<Option<BTreeSet<i32>>>: sorted elements, some None
    let bt_opt: Vec<Option<BTreeSet<i32>>> = vec![
        Some([3, 1, 2].iter().copied().collect()),
        None,
        Some([5, 4].iter().copied().collect()),
    ];
    let _ = bt_opt.into_sexp();
}

/// Exercise `Vec<Option<&str>>` and `Vec<Option<&[T]>>` conversions under GC pressure.
///
/// Allocates STRSXP + list-column SEXPs with interleaved None/Some values to verify
/// PROTECT discipline across string and slice allocations.
#[miniextendr]
pub fn gc_stress_vec_option_borrowed() {
    // Vec<Option<&str>>: STRSXP with NA_character_
    let str_opt: Vec<Option<&str>> = vec![Some("hello"), None, Some("world"), None];
    let _ = str_opt.into_sexp();

    // Vec<Option<&[f64]>>: list-column, NULL for None
    let a: &[f64] = &[1.0, 2.0, 3.0];
    let b: &[f64] = &[4.0];
    let slice_opt: Vec<Option<&[f64]>> = vec![Some(a), None, Some(b), None];
    let _ = slice_opt.into_sexp();

    // Vec<Option<&[String]>>: list-column (character vector per row)
    let sa: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let sb: Vec<String> = vec!["z".to_string()];
    let str_slice_opt: Vec<Option<&[String]>> =
        vec![Some(sa.as_slice()), None, Some(sb.as_slice())];
    let _ = str_slice_opt.into_sexp();
}

/// Exercise map-field `Vec<Vec<K>>` / `Vec<Vec<V>>` column codegen under GC pressure.
///
/// Synthesizes realistic `HashMapEvent` and `BTreeMapEvent` rows and drives both
/// `to_dataframe` (align) and `to_dataframe_split` (per-variant partition) paths.
/// Verifies that the `ProtectScope::protect_raw` calls in the generated map-column
/// code keep each inner `Vec<K>` / `Vec<V>` SEXP live across the subsequent
/// `into_sexp()` call for the parallel column.
#[miniextendr]
pub fn gc_stress_dataframe_map() {
    // HashMap align path — multiple variants, multiple rows, includes empty map.
    let hm_rows = vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([
                ("x".to_string(), 1i32),
                ("y".to_string(), 2i32),
                ("z".to_string(), 3i32),
            ]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::new(), // empty map → Some(vec![]) in both columns
        },
        HashMapEvent::Tally {
            label: "d".into(),
            tally: HashMap::from([("p".to_string(), 10i32)]),
        },
        HashMapEvent::Empty { label: "e".into() },
    ];
    let _ = HashMapEvent::to_dataframe(hm_rows.clone());
    let _ = HashMapEvent::to_dataframe_split(hm_rows);

    // BTreeMap align path — same shape, sorted key order exercised.
    let bt_rows = vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([
                ("z".to_string(), 3i32),
                ("a".to_string(), 1i32),
                ("m".to_string(), 2i32),
            ]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "d".into(),
            tally: BTreeMap::from([("q".to_string(), 99i32)]),
        },
        BTreeMapEvent::Empty { label: "e".into() },
    ];
    let _ = BTreeMapEvent::to_dataframe(bt_rows.clone());
    let _ = BTreeMapEvent::to_dataframe_split(bt_rows);
}

/// Exercise struct-field DataFrameRow flatten + as_list paths under GC pressure.
///
/// Allocates `StructFlattenEvent` and `StructListEvent` rows, calls both
/// `to_dataframe` (align) and `to_dataframe_split`, and converts to SEXP,
/// verifying that `ProtectScope` keeps inner column SEXPs live across scatter
/// allocations.  No arguments required — suitable for the fast gctorture sweep.
#[miniextendr]
pub fn gc_stress_dataframe_struct() {
    use miniextendr_api::into_r::IntoR as _;

    // Flatten path: struct-typed field expands to prefixed columns.
    let flatten_rows = vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Other { id: 2 },
        StructFlattenEvent::Located {
            id: 3,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 4 },
    ];
    let _ = StructFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = StructFlattenEvent::to_dataframe_split(flatten_rows).into_sexp();

    // as_list path: struct-typed field kept as opaque list-column.
    let list_rows = vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Other { id: 2 },
        StructListEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
        StructListEvent::Other { id: 4 },
    ];
    let _ = StructListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = StructListEvent::to_dataframe_split(list_rows).into_sexp();
}

/// Exercise nested-enum field DataFrameRow flatten + as_factor + as_list paths
/// under GC pressure.
///
/// Drives both `to_dataframe` (align) and `to_dataframe_split` paths for all
/// three nested-enum field modes. No arguments — suitable for the fast gctorture
/// sweep.
#[miniextendr]
pub fn gc_stress_dataframe_nested_enum() {
    use miniextendr_api::into_r::IntoR as _;

    // Flatten path: nested payload-bearing DataFrameRow enum expands to prefixed columns.
    // Uses Status (Ok / Err { code }) so status_variant + status_code columns are exercised.
    let flatten_rows = vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 4 },
    ];
    let _ = NestedFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = NestedFlattenEvent::to_dataframe_split(flatten_rows).into_sexp();

    // as_factor path: unit-only enum field stored as factor column.
    let factor_rows = vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Stop { id: 2 },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 4 },
    ];
    let _ = NestedFactorEvent::to_dataframe(factor_rows.clone()).into_sexp();
    let _ = NestedFactorEvent::to_dataframe_split(factor_rows).into_sexp();

    // as_list path: enum field kept as opaque list-column.
    let list_rows = vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::South,
        },
        NestedListEvent::Stop { id: 2 },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 4 },
    ];
    let _ = NestedListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = NestedListEvent::to_dataframe_split(list_rows).into_sexp();

    // Status is already exercised via NestedFlattenEvent above.
}

/// Exercise the native-SEXP ALTREP (`NativeSexpIntAltrep`) under GC pressure.
///
/// Constructs an ALTREP-backed integer vector where `data1` is a plain
/// `INTSXP` (no ExternalPtr).  Exercises both element access (via `Elt`) and
/// the dataptr path (`as.integer`) so that any PROTECT-discipline bug around
/// the `data1` allocation would be caught by `gctorture(TRUE)`.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr]
pub fn gc_stress_native_sexp_altrep() {
    use crate::native_sexp_altrep_fixture::native_sexp_altrep_new;
    use miniextendr_api::prelude::SexpExt as _;

    // Construct a small ALTREP-backed integer vector.
    let values = vec![10i32, 20, 30, 40, 50];
    let sexp = native_sexp_altrep_new(values);

    // Verify that it is indeed ALTREP.
    assert!(
        sexp.is_altrep(),
        "native_sexp_altrep_new should return an ALTREP SEXP"
    );

    // Force element access (exercises the Elt path) via the SexpExt trait.
    let n = sexp.len();
    assert_eq!(n, 5);
    for i in 0..n {
        let v = sexp.integer_elt(i as isize);
        assert_eq!(v, (i as i32 + 1) * 10);
    }

    // Force the Dataptr path (exercises `AltVec::dataptr`).
    let _ptr = unsafe { miniextendr_api::sys::DATAPTR_RO(sexp) };
}

/// Exercise `JiffZonedVec` ALTREP construction and element access under GC pressure.
///
/// Constructs a single-timezone `JiffZonedVec` with several `America/New_York`
/// timestamps, converts it to an SEXP via `into_sexp`, and forces element access
/// to verify that both the `into_sexp_altrep` allocation and the `set_posixct_tz`
/// PROTECT path are GC-safe.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "jiff")]
#[miniextendr]
pub fn gc_stress_jiff_zoned_vec() {
    use miniextendr_api::jiff::tz::TimeZone;

    let tz = TimeZone::get("America/New_York").expect("America/New_York must exist");
    let timestamps = [
        Timestamp::new(1_735_689_600, 0).expect("valid timestamp"), // 2025-01-01 UTC
        Timestamp::new(1_750_000_000, 0).expect("valid timestamp"),
        Timestamp::new(1_760_000_000, 0).expect("valid timestamp"),
    ];
    let data: Vec<miniextendr_api::Zoned> = timestamps
        .iter()
        .map(|ts| ts.to_zoned(tz.clone()))
        .collect::<Vec<_>>();
    let vec = JiffZonedVec::new(data).expect("single-tz JiffZonedVec construction");

    // Convert to SEXP (exercises ALTREP allocation + set_posixct_tz PROTECT path).
    let sexp = vec.into_posixct_sexp();

    // Force element access via the ALTREP Elt path.
    use miniextendr_api::prelude::SexpExt as _;
    let n = sexp.len();
    assert_eq!(n, 3);
    for i in 0..n {
        let _v = sexp.real_elt(i as isize);
    }
}

/// Exercise `array_to_sexp` PROTECT discipline for integer, float, and boolean
/// TOML arrays under GC pressure.
///
/// Constructs `TomlValue::Array` values covering the four homogeneous branches
/// of `array_to_sexp` and converts each to SEXP via `IntoR`, verifying that
/// the `OwnedProtect` guards keep each allocation live across the fill loop.
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "toml")]
#[miniextendr]
pub fn gc_stress_toml_array() {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::toml_impl::TomlValue;

    // Integer branch (all fit in i32) → INTSXP
    let int_arr = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(2),
        TomlValue::Integer(i64::from(i32::MAX)),
    ]);
    let _ = int_arr.into_sexp();

    // Integer branch (one value exceeds i32 range) → REALSXP fallback
    let big_int_arr = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(i64::from(i32::MAX) + 1),
        TomlValue::Integer(3),
    ]);
    let _ = big_int_arr.into_sexp();

    // Float branch → REALSXP
    let float_arr = TomlValue::Array(vec![
        TomlValue::Float(1.1),
        TomlValue::Float(2.2),
        TomlValue::Float(3.3),
    ]);
    let _ = float_arr.into_sexp();

    // Boolean branch → LGLSXP
    let bool_arr = TomlValue::Array(vec![
        TomlValue::Boolean(true),
        TomlValue::Boolean(false),
        TomlValue::Boolean(true),
    ]);
    let _ = bool_arr.into_sexp();
}

/// Exercise `NamedDataFrameListBuilder` SEXP protection under GC pressure.
///
/// Pushes two `ColumnarDataFrame`s (synthesised internally) into a
/// `NamedDataFrameListBuilder` and calls `build()`.  Each `push` stores the
/// input SEXP in an internal `Vec<SEXP>` (protected by the builder's
/// `ProtectScope`), making this the canonical gctorture coverage for the
/// SEXP-storage-across-allocations path in the builder.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_named_df_list_builder() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{NamedDataFrameListBuilder, vec_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        msg: String,
    }

    let oks: Vec<OkRow> = (0..50)
        .map(|i| OkRow {
            id: i,
            value: f64::from(i) * 1.5,
        })
        .collect();
    let errs: Vec<ErrRow> = (0..20)
        .map(|i| ErrRow {
            id: i,
            msg: format!("err {i}"),
        })
        .collect();

    NamedDataFrameListBuilder::new()
        .push("results", vec_to_dataframe(&oks).unwrap())
        .push("error", vec_to_dataframe(&errs).unwrap())
        .build()
        .into_sexp()
}

/// Convert an R vector to an ALTREP-backed vector by materializing then re-wrapping.
/// Dispatches on `type_of()`: INTSXP, REALSXP, STRSXP.
/// @param x An integer, numeric, or character vector to convert.
#[miniextendr]
pub fn into_sexp_altrep(x: SEXP) -> SEXP {
    let sxp_type = x.type_of();
    match sxp_type {
        SEXPTYPE::INTSXP => {
            let v: Vec<i32> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::REALSXP => {
            let v: Vec<f64> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::STRSXP => {
            // Use Vec<Option<String>> to preserve NA_character_ values
            let v: Vec<Option<String>> =
                miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        _ => panic!("into_sexp_altrep: unsupported SEXP type {:?}", sxp_type),
    }
}

// region: dataframe_to_vec / with_dataframe_rows GC stress

/// Exercise `dataframe_to_vec` PROTECT discipline under GC pressure.
///
/// Synthesizes 10 rows via `vec_to_dataframe`, then runs `dataframe_to_vec`
/// to reconstruct the original rows. Returns the row count to verify execution.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_dataframe_to_vec() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{dataframe_to_vec, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct GcRow {
        id: i32,
        score: f64,
        label: Option<String>,
    }

    let original: Vec<GcRow> = (0i32..10)
        .map(|i| GcRow {
            id: i,
            score: f64::from(i) * 1.1,
            label: if i % 2 == 0 {
                Some(format!("item_{}", i))
            } else {
                None
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_dataframe_to_vec: vec_to_dataframe failed")
        .into_sexp();

    let back: Vec<GcRow> =
        dataframe_to_vec(sexp).expect("gc_stress_dataframe_to_vec: dataframe_to_vec failed");

    assert_eq!(back.len(), original.len());
    for (a, b) in original.iter().zip(back.iter()) {
        assert_eq!(a, b);
    }

    back.len() as i32
}

/// Exercise `with_dataframe_rows` PROTECT discipline under GC pressure.
///
/// Synthesizes 10 rows, converts to a data.frame, then uses `with_dataframe_rows`
/// to compute a sum via the scoped callback. Returns the sum to verify execution.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_with_dataframe_rows() -> f64 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{vec_to_dataframe, with_dataframe_rows};

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "crate::serde")]
    struct GcRow {
        value: f64,
        tag: String,
    }

    let source: Vec<GcRow> = (0i32..10)
        .map(|i| GcRow {
            value: f64::from(i) * 2.0,
            tag: format!("t{}", i),
        })
        .collect();

    let sexp = vec_to_dataframe(&source)
        .expect("gc_stress_with_dataframe_rows: vec_to_dataframe failed")
        .into_sexp();

    with_dataframe_rows(sexp, |rows: &[GcRow]| {
        // Compute a sum and verify tags are valid.
        let sum: f64 = rows.iter().map(|r| r.value).sum();
        for r in rows {
            assert!(
                r.tag.starts_with('t'),
                "tag should start with 't': {}",
                r.tag
            );
        }
        sum
    })
    .expect("gc_stress_with_dataframe_rows: with_dataframe_rows failed")
}

/// Exercise `dataframe_to_vec` nested-struct un-flattening under GC pressure.
///
/// Synthesizes 10 rows of a nested `Person { name, address: Address { city, zip } }`
/// struct via `vec_to_dataframe`, then reconstructs them via `dataframe_to_vec`
/// using the single-underscore prefix-matching path. Returns the row count.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_dataframe_to_vec_nested() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{dataframe_to_vec, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Address {
        city: String,
        zip: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Person {
        name: String,
        address: Address,
    }

    let original: Vec<Person> = (0i32..10)
        .map(|i| Person {
            name: format!("person_{}", i),
            address: Address {
                city: format!("city_{}", i),
                zip: format!("{:05}", i * 1000),
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_dataframe_to_vec_nested: vec_to_dataframe failed")
        .into_sexp();

    let back: Vec<Person> = dataframe_to_vec(sexp)
        .expect("gc_stress_dataframe_to_vec_nested: dataframe_to_vec failed");

    assert_eq!(back.len(), original.len());
    for (a, b) in original.iter().zip(back.iter()) {
        assert_eq!(a, b);
    }

    back.len() as i32
}

// endregion

// region: Streaming serialize GC stress

/// Exercise `iter_to_dataframe` PROTECT discipline under GC pressure.
///
/// Synthesizes 50 rows via an iterator and runs them through
/// `iter_to_dataframe`, verifying that the per-builder `ProtectScope` keeps
/// every protected SEXP live across the fill loop and `assemble_dataframe`.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_iter_to_dataframe() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::iter_to_dataframe;

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct StreamRow {
        id: i32,
        value: f64,
        label: String,
        flag: Option<i32>,
    }

    let df = iter_to_dataframe(
        (0i32..50).map(|i| StreamRow {
            id: i,
            value: f64::from(i) * 1.5,
            label: format!("row_{i}"),
            flag: if i % 3 == 0 { None } else { Some(i * 2) },
        }),
        None,
    )
    .expect("gc_stress_iter_to_dataframe: iter_to_dataframe failed");
    df.into_sexp()
}

/// Exercise `dispatch_to_dataframes` PROTECT discipline under GC pressure.
///
/// Synthesises a mixed Result-iterator (every third row is an Err with a
/// distinct payload shape) and runs it through the streaming two-builder
/// path. Each builder's `ProtectScope` plus the assembling
/// `NamedDataFrameListBuilder` need to keep every protected SEXP live.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_dispatch_to_dataframes() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{DispatchNames, dispatch_to_dataframes};

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        val: f64,
    }
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let list = dispatch_to_dataframes(
        (0i32..30).map(|i| {
            if i % 3 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("skip_{i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    val: f64::from(i) * 0.5,
                })
            }
        }),
        Some(30),
        DispatchNames::default(),
    )
    .expect("gc_stress_dispatch_to_dataframes: dispatch_to_dataframes failed");
    list.into_sexp()
}

// endregion

// region: DataFrameBuilder with_schema / grow_schema GC stress

/// Exercise [`DataFrameBuilder::with_schema`] PROTECT discipline under GC
/// pressure. Builds a pre-declared schema, pushes 50 rows including a
/// `None`-bearing optional column, then assembles via `finish()`. Verifies
/// the per-builder `ProtectScope` keeps every protected SEXP live across
/// the fill loop and final assembly.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_builder_with_schema() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{DataFrameBuilder, TypeSpec};

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        ratio: f64,
        tag: Option<String>,
    }

    let mut b = DataFrameBuilder::<Row>::with_schema(
        [
            ("id", TypeSpec::Integer),
            ("ratio", TypeSpec::Real),
            // Optional(Character) keeps the column character-typed even if
            // the first row's `tag` is None.
            ("tag", TypeSpec::Optional(Box::new(TypeSpec::Character))),
        ],
        Some(50),
    );
    for i in 0..50i32 {
        b.push(Row {
            id: i,
            ratio: f64::from(i) * 0.25,
            tag: if i % 3 == 0 {
                None
            } else {
                Some(format!("tag_{i}"))
            },
        })
        .expect("gc_stress_builder_with_schema: push failed");
    }
    let df = b
        .finish()
        .expect("gc_stress_builder_with_schema: finish failed");
    df.into_sexp()
}

/// Exercise [`DataFrameBuilder::grow_schema`] PROTECT discipline under GC
/// pressure. Pushes 30 heterogeneous `BTreeMap` rows that progressively
/// introduce new keys, forcing the back-fill path to allocate columns and
/// NA-fill prior rows. Verifies the grown columns stay rooted and
/// length-aligned through `finish()`.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_builder_grow_schema() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::DataFrameBuilder;
    use std::collections::BTreeMap;

    let mut b = DataFrameBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    for i in 0..30i32 {
        let mut row: BTreeMap<String, i32> = BTreeMap::new();
        row.insert("base".into(), i);
        // Introduce a fresh key every 5 rows to drive the back-fill path.
        if i % 5 == 0 {
            row.insert(format!("k_{i}"), i * 10);
        }
        // A common second column that appears from row 1 onward.
        if i >= 1 {
            row.insert("common".into(), i + 100);
        }
        b.push(row).expect("gc_stress_builder_grow_schema: push failed");
    }
    let df = b
        .finish()
        .expect("gc_stress_builder_grow_schema: finish failed");
    df.into_sexp()
}

// endregion

// region: BorrowedRows GC stress

/// Exercise `dataframe_to_vec_borrowed` PROTECT discipline under GC pressure.
///
/// Synthesises 10 rows via `vec_to_dataframe`, then deserialises through
/// `dataframe_to_vec_borrowed` and reads back via the `BorrowedRows<'_, T>`
/// handle. Verifies the internal `OwnedProtect` keeps the source SEXP rooted
/// while we read each row.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_borrowed_rows() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{BorrowedRows, dataframe_to_vec_borrowed, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        score: f64,
        label: Option<String>,
    }

    let original: Vec<Row> = (0i32..10)
        .map(|i| Row {
            id: i,
            score: f64::from(i) * 1.1,
            label: if i % 2 == 0 {
                Some(format!("item_{i}"))
            } else {
                None
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_borrowed_rows: vec_to_dataframe failed")
        .into_sexp();

    let bundle: BorrowedRows<'_, Row> =
        dataframe_to_vec_borrowed(sexp).expect("gc_stress_borrowed_rows: deserialise failed");

    assert_eq!(bundle.len(), original.len());
    for (a, b) in original.iter().zip(bundle.iter()) {
        assert_eq!(a, b);
    }

    bundle.len() as i32
}

// endregion

// region: STRSXP-building PROTECT discipline GC stress

/// Exercise the PROTECT discipline of the migrated STRSXP-building / factor
/// paths under GC pressure.
///
/// Drives:
/// - `match_arg::choices_sexp` (via a derived `MatchArg` enum's `IntoR for Vec<T>`),
/// - `match_arg::match_arg_vec_into_sexp` (same path),
/// - `factor::build_factor_with_levels` (via `FactorVec` / `FactorOptionVec`
///   blanket impls — bare `RFactor` enum, no caching),
/// - `named_vector::set_names_on_sexp` (via `NamedVector<HashMap<String, i32>>`).
///
/// Each path allocates a fresh STRSXP and reads/writes elements across
/// allocations that can fire GC — exactly the shape that broke in PR #344's
/// `make_rust_condition_value`. No arguments — suitable for the fast
/// gctorture no-arg fixture sweep.
#[miniextendr]
pub fn gc_stress_protect_discipline() {
    use miniextendr_api::factor::{FactorOptionVec, FactorVec};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::match_arg::MatchArg;
    use miniextendr_api::NamedVector;

    // FactorVec<Color> → build_factor_with_levels exercised on a no-cache path
    // (RFactor blanket through FactorVec).
    let fv: FactorVec<crate::factor_tests::Color> = FactorVec(vec![
        crate::factor_tests::Color::Red,
        crate::factor_tests::Color::Green,
        crate::factor_tests::Color::Blue,
        crate::factor_tests::Color::Red,
    ]);
    let _ = fv.into_sexp();

    // FactorOptionVec<Color> with NA → same path, different indices builder.
    let fov: FactorOptionVec<crate::factor_tests::Color> = FactorOptionVec(vec![
        Some(crate::factor_tests::Color::Red),
        None,
        Some(crate::factor_tests::Color::Green),
        None,
    ]);
    let _ = fov.into_sexp();

    // Vec<Mode>: IntoR (Vec<T: MatchArg>) → match_arg_vec_into_sexp,
    // exercising the STRSXP build loop with the R_BlankString short-circuit
    // path skipped (no empty strings in CHOICES here).
    let _ = miniextendr_api::match_arg::match_arg_vec_into_sexp(vec![
        crate::match_arg_tests::Mode::Fast,
        crate::match_arg_tests::Mode::Safe,
        crate::match_arg_tests::Mode::Debug,
    ]);

    // choices_sexp<Mode> → STRSXP build with the same short-circuit branch.
    let _ = miniextendr_api::match_arg::choices_sexp::<crate::match_arg_tests::Mode>();

    // NamedVector<HashMap<String, i32>> → set_names_on_sexp STRSXP build.
    let mut m = HashMap::new();
    m.insert("alpha".to_string(), 1i32);
    m.insert("beta".to_string(), 2);
    m.insert("gamma".to_string(), 3);
    let _ = NamedVector(m).into_sexp();

    // Empty-key edge case: hits the `R_BlankString` short-circuit in
    // `set_names_on_sexp`.
    let mut m2 = HashMap::new();
    m2.insert(String::new(), 42i32);
    let _ = NamedVector(m2).into_sexp();

    // CHOICES read-back to silence unused-warning on the MatchArg trait import
    // (also a sanity check that the enum type is reachable at compile time).
    let _ = <crate::match_arg_tests::Mode as MatchArg>::CHOICES.len();
}

// endregion

// region: map_to_dataframe / result_to_dataframe / vec_to_dataframe_split helpers (#700, #697, #699)

/// Exercise `map_to_dataframe` under GC pressure.
///
/// Allocates a `BTreeMap<i32, KvValue>`, serialises through the helper,
/// converts to SEXP. Touches the same SEXP-storage code paths as
/// `vec_to_dataframe` (its underlying call) — `gctorture(TRUE)` validates
/// the schema-discovery / column-buffer / character-column protections.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_map_to_dataframe() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::map_to_dataframe;
    use std::collections::BTreeMap;

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct KvValue {
        score: f64,
        label: String,
    }

    let mut map: BTreeMap<i32, KvValue> = BTreeMap::new();
    for i in 0..40i32 {
        map.insert(
            i,
            KvValue {
                score: f64::from(i) * 1.25,
                label: format!("entry_{i}"),
            },
        );
    }

    map_to_dataframe(&map, "id")
        .expect("gc_stress_map_to_dataframe: map_to_dataframe failed")
        .into_sexp()
}

/// Exercise `result_to_dataframe(Auto)` under GC pressure with mixed Ok/Err
/// rows. Exercises the split path (intermediate `ColumnarDataFrame` pair +
/// `NamedDataFrameListBuilder` assembly in `IntoR for DataFrameShape`).
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_result_to_dataframe_auto() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..40i32)
        .map(|i| {
            if i % 3 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("err {i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    value: f64::from(i) * 1.5,
                })
            }
        })
        .collect();

    result_to_dataframe(
        &rows,
        ResultShape::Auto {
            empty_ok_sentinel: (),
        },
    )
    .expect("gc_stress_result_to_dataframe_auto: helper failed")
    .into_sexp()
}

/// Exercise `result_to_dataframe(Collated)` under GC pressure. Union-schema
/// emission goes through the `TaggedVariantRow`/`MapForwarder` path.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_result_to_dataframe_collated() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..30i32)
        .map(|i| {
            if i % 4 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("err {i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    value: f64::from(i) * 2.0,
                })
            }
        })
        .collect();

    result_to_dataframe::<_, _, ()>(&rows, ResultShape::Collated)
        .expect("gc_stress_result_to_dataframe_collated: helper failed")
        .into_sexp()
}

/// Exercise `result_to_dataframe(Split)` with all-Err input → sentinel
/// path. Validates that the user-supplied sentinel SEXP is rooted while
/// the outer named list is assembled.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_result_to_dataframe_split_sentinel() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..25i32)
        .map(|i| {
            Err(ErrRow {
                id: i,
                reason: format!("err {i}"),
            })
        })
        .collect();

    // Sentinel is a String — ends up as an STRSXP and exercises the
    // protect-around-CHARSXP-allocation path inside DataFrameShape::IntoR.
    result_to_dataframe(
        &rows,
        ResultShape::Split {
            empty_ok_sentinel: String::from("no ok rows"),
        },
    )
    .expect("gc_stress_result_to_dataframe_split_sentinel: helper failed")
    .into_sexp()
}

/// Exercise `vec_to_dataframe_split(PerVariantListWithTag)` under GC
/// pressure. The tag-column prepend allocates a per-partition STRSXP that
/// must be protected through the `prepend_column` VECSXP reshuffle.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_split_with_tag() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{SplitShape, vec_to_dataframe_split};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    enum Event {
        Click { x: f64, y: f64 },
        Scroll { delta: f64 },
        KeyPress { code: String },
    }

    let rows: Vec<Event> = (0..30i32)
        .map(|i| match i % 3 {
            0 => Event::Click {
                x: f64::from(i),
                y: f64::from(i) * 2.0,
            },
            1 => Event::Scroll {
                delta: f64::from(i) * -0.5,
            },
            _ => Event::KeyPress {
                code: format!("k{i}"),
            },
        })
        .collect();

    vec_to_dataframe_split(
        &rows,
        SplitShape::PerVariantListWithTag {
            column: "variant".into(),
        },
    )
    .expect("gc_stress_split_with_tag: helper failed")
    .into_sexp()
}

/// Exercise `vec_to_dataframe_split(Collated)` under GC pressure. Drives
/// the `TaggedVariantRow` / `MapForwarder` collation path through the
/// schema-union machinery in `ColumnarDataFrame::from_rows`.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_split_collated() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{SplitShape, vec_to_dataframe_split};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    enum Event {
        Click { x: f64, y: f64 },
        Scroll { delta: f64 },
        KeyPress { code: String },
    }

    let rows: Vec<Event> = (0..30i32)
        .map(|i| match i % 3 {
            0 => Event::Click {
                x: f64::from(i),
                y: f64::from(i) * 2.0,
            },
            1 => Event::Scroll {
                delta: f64::from(i) * -0.5,
            },
            _ => Event::KeyPress {
                code: format!("k{i}"),
            },
        })
        .collect();

    vec_to_dataframe_split(
        &rows,
        SplitShape::Collated {
            column: "kind".into(),
        },
    )
    .expect("gc_stress_split_collated: helper failed")
    .into_sexp()
}

// endregion

// region: factor-label dataframe_to_vec GC stress (issue #689)

/// Exercise the factor-label path through `dataframe_to_vec` under GC
/// pressure.
///
/// The factor branch in `CellDeserializer::deserialize_str/string/char/any`
/// reads `levels` (via `Rf_getAttrib`) and dereferences `STRING_ELT` on the
/// levels SEXP — a path that crosses a GC barrier. This fixture synthesises
/// a one-column factor data.frame internally, then runs `dataframe_to_vec`
/// to reconstruct `Vec<Row { status: String }>`. The intermediate factor +
/// data.frame SEXPs are protected via `OwnedProtect`; the deserialiser must
/// keep them rooted across the per-row label lookups.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr]
pub fn gc_stress_factor_labels() -> i32 {
    use crate::serde::Deserialize;
    use miniextendr_api::factor::{build_factor, build_levels_sexp};
    use miniextendr_api::prelude::{SEXP, SexpExt};
    use miniextendr_api::SEXPTYPE;
    use miniextendr_api::sys::Rf_allocVector;
    use miniextendr_api::gc_protect::OwnedProtect;
    use miniextendr_api::serde::dataframe_to_vec;

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Row {
        status: Option<String>,
    }

    let levels = ["active", "pending", "archived"];
    // Build 30 rows cycling through codes 1..=3 to exercise multiple label
    // hits per call, plus a few NA cells (NA_INTEGER → `status: None`).
    const NA: i32 = i32::MIN; // NA_INTEGER
    let codes: Vec<i32> = (0..30)
        .map(|i| if i % 7 == 0 { NA } else { (i % 3) as i32 + 1 })
        .collect();

    // SAFETY: all R API calls happen on the worker thread (this fixture is
    // invoked via the #[miniextendr] wrapper), and every transient
    // allocation is protected before the next allocation can run GC.
    let df = unsafe {
        let levels_sexp = build_levels_sexp(&levels);
        let _levels_guard = OwnedProtect::new(levels_sexp);
        let col = build_factor(&codes, levels_sexp);
        let _col_guard = OwnedProtect::new(col);

        let list = Rf_allocVector(SEXPTYPE::VECSXP, 1);
        let _list_guard = OwnedProtect::new(list);
        list.set_vector_elt(0, col);

        let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        let _names_guard = OwnedProtect::new(names_sexp);
        names_sexp.set_string_elt(0, SEXP::charsxp("status"));
        list.set_names(names_sexp);

        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        let _class_guard = OwnedProtect::new(class_sexp);
        class_sexp.set_string_elt(0, SEXP::charsxp("data.frame"));
        list.set_class(class_sexp);

        let row_names = Rf_allocVector(SEXPTYPE::INTSXP, 2);
        let _rn_guard = OwnedProtect::new(row_names);
        let rn = row_names.as_mut_slice::<i32>();
        rn[0] = i32::MIN;
        rn[1] = -(codes.len() as i32);
        list.set_row_names(row_names);

        // Hand `list` to dataframe_to_vec while every transient is still
        // protected — the inner guards drop at the end of this block, but
        // by then the only live SEXP we care about (`list`) is unprotected
        // and immediately consumed.
        dataframe_to_vec::<Row>(list).expect("gc_stress_factor_labels: dataframe_to_vec failed")
    };

    assert_eq!(df.len(), codes.len());
    let mut na_count = 0;
    for (i, row) in df.iter().enumerate() {
        match (&row.status, codes[i]) {
            (None, NA) => na_count += 1,
            (Some(label), code) if code != NA => assert!(
                levels.contains(&label.as_str()),
                "unexpected label at row {i}: {label}"
            ),
            other => panic!("row {i} mismatch: {:?} vs code {}", other, codes[i]),
        }
    }
    assert!(na_count > 0, "fixture should exercise NA rows");
    df.len() as i32
}

// endregion

// region: typed_dataframe! (#698)

/// Exercise the `typed_dataframe!`-generated `TryFromSexp` impl under GC
/// pressure.
///
/// Synthesises an R `data.frame` from scratch, drives it through
/// `TheophDf::try_from_sexp`, and then forces every per-column borrowed
/// accessor to read its slice. The accessors call
/// `RNativeType::dataptr_mut` which is the path most likely to surface
/// PROTECT-discipline bugs (each column is allocated then promoted into a
/// generic VECSXP via `from_raw_pairs`, then attribute-tagged into a
/// data.frame).
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr]
pub fn gc_stress_typed_dataframe() {
    use crate::typed_dataframe_tests::TheophDf;
    use miniextendr_api::IntoR as _;
    use miniextendr_api::from_r::TryFromSexp as _;
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::list::List;

    // Construct a Theoph-shaped data.frame internally.
    //
    // Each column SEXP must be PROTECTed before `List::from_raw_pairs`
    // takes ownership — the docs say "The input SEXPs should already
    // be protected." Without this, gctorture(TRUE) (and even normal
    // R-devel runs) can reap unprotected column SEXPs while the names
    // STRSXP is being built.
    let nrow = 12usize;
    let subject: Vec<i32> = (0..nrow as i32).collect();
    let weight: Vec<f64> = (0..nrow).map(|i| 60.0 + i as f64).collect();
    let dose: Vec<f64> = vec![320.0; nrow];
    let time: Vec<f64> = (0..nrow).map(|i| i as f64 * 0.5).collect();
    let conc: Vec<f64> = (0..nrow).map(|i| 1.0 + i as f64 * 0.1).collect();

    let scope = unsafe { ProtectScope::new() };
    let subject_sexp = unsafe { scope.protect_raw(subject.into_sexp()) };
    let weight_sexp = unsafe { scope.protect_raw(weight.into_sexp()) };
    let dose_sexp = unsafe { scope.protect_raw(dose.into_sexp()) };
    let time_sexp = unsafe { scope.protect_raw(time.into_sexp()) };
    let conc_sexp = unsafe { scope.protect_raw(conc.into_sexp()) };

    let list = List::from_raw_pairs(vec![
        ("subject", subject_sexp),
        ("weight", weight_sexp),
        ("dose", dose_sexp),
        ("time", time_sexp),
        ("conc", conc_sexp),
    ]);
    let df = list
        .as_data_frame()
        .expect("synthetic data.frame promotion should succeed");
    let sexp = df.as_sexp();

    // Drive the validating TryFromSexp path. The result stores per-column
    // SEXPs as plain fields; the surrounding data.frame SEXP is owned by
    // `df` (held alive by `sexp`).
    let theoph = TheophDf::try_from_sexp(sexp).expect("TheophDf validation should succeed");

    // Force every accessor to read its slice — this calls
    // `dataptr_mut` on every column SEXP, exercising any PROTECT
    // problems in storage.
    let subj_sum: i32 = theoph.subject().iter().copied().sum();
    let weight_sum: f64 = theoph.weight().iter().copied().sum();
    let dose_sum: f64 = theoph.dose().iter().copied().sum();
    let time_sum: f64 = theoph.time().iter().copied().sum();
    let conc_sum: f64 = theoph.conc().iter().copied().sum();
    assert_eq!(theoph.nrow(), nrow);
    assert!(subj_sum >= 0);
    assert!(weight_sum > 0.0);
    assert!(dose_sum > 0.0);
    assert!(time_sum >= 0.0);
    assert!(conc_sum > 0.0);

    // Optional column was absent in this fixture.
    assert!(theoph.flag().is_none());

    drop(scope);

    // Now repeat the dance with the optional column populated, so the
    // `Option<SEXP>` storage path is also stressed.
    let subject2: Vec<i32> = (0..nrow as i32).collect();
    let weight2: Vec<f64> = (0..nrow).map(|i| 50.0 + i as f64).collect();
    let dose2: Vec<f64> = vec![160.0; nrow];
    let time2: Vec<f64> = (0..nrow).map(|i| i as f64).collect();
    let conc2: Vec<f64> = (0..nrow).map(|i| 2.0 + i as f64 * 0.25).collect();
    let flag2: Vec<i32> = (0..nrow as i32).map(|i| i % 2).collect();

    let scope2 = unsafe { ProtectScope::new() };
    let subject2_sexp = unsafe { scope2.protect_raw(subject2.into_sexp()) };
    let weight2_sexp = unsafe { scope2.protect_raw(weight2.into_sexp()) };
    let dose2_sexp = unsafe { scope2.protect_raw(dose2.into_sexp()) };
    let time2_sexp = unsafe { scope2.protect_raw(time2.into_sexp()) };
    let conc2_sexp = unsafe { scope2.protect_raw(conc2.into_sexp()) };
    let flag2_sexp = unsafe { scope2.protect_raw(flag2.into_sexp()) };

    let list2 = List::from_raw_pairs(vec![
        ("subject", subject2_sexp),
        ("weight", weight2_sexp),
        ("dose", dose2_sexp),
        ("time", time2_sexp),
        ("conc", conc2_sexp),
        ("flag", flag2_sexp),
    ]);
    let df2 = list2
        .as_data_frame()
        .expect("synthetic data.frame promotion should succeed");
    let sexp2 = df2.as_sexp();

    let theoph2 =
        TheophDf::try_from_sexp(sexp2).expect("TheophDf validation should succeed (with flag)");
    let flag_sum: i32 = theoph2
        .flag()
        .expect("flag column should be present")
        .iter()
        .copied()
        .sum();
    assert!(flag_sum >= 0);
}

// endregion

// region: rayon RDataFrameBuilder

/// Exercise `RDataFrameBuilder` parallel column-fill under GC pressure.
///
/// Builds a heterogeneous data.frame (numeric `x`, integer `y`, character
/// `label` with interspersed `NA`) via the parallel builder. The builder
/// allocates each column SEXP serially, holds them protected across subsequent
/// column / names / row.names / class allocations, and assembles the parent
/// `VECSXP`. This is exactly the SEXP-across-allocation path that needs a
/// gctorture pass. No arguments — suitable for the fast gctorture sweep.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn gc_stress_dataframe_rayon() {
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::rayon_bridge::RDataFrameBuilder;

    let nrow = 200usize;
    let df = RDataFrameBuilder::new(nrow)
        .column::<f64>("x", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = ((offset + i) as f64).sqrt();
            }
        })
        .column::<i32>("y", |chunk: &mut [i32], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as i32 * 3;
            }
        })
        .column_str("label", |i: usize| {
            if i % 7 == 6 {
                None
            } else {
                Some(format!("row_{i}"))
            }
        })
        .build();

    // Force materialization of every column under the same protect scope so any
    // stale/freed column surfaces. The builder leaves `df` unprotected, so
    // protect it before reading.
    let scope = unsafe { ProtectScope::new() };
    let df = unsafe { scope.protect_raw(df) };
    assert!(df.is_data_frame());
    assert_eq!(df.xlength(), 3);

    let x = df.vector_elt(0);
    let y = df.vector_elt(1);
    let label = df.vector_elt(2);
    assert_eq!(x.xlength() as usize, nrow);
    assert_eq!(y.xlength() as usize, nrow);
    assert_eq!(label.xlength() as usize, nrow);

    let x_slice: &[f64] = unsafe { x.as_slice() };
    let y_slice: &[i32] = unsafe { y.as_slice() };
    let x_sum: f64 = x_slice.iter().copied().sum();
    let y_sum: i32 = y_slice.iter().copied().sum();
    assert!(x_sum > 0.0);
    assert!(y_sum > 0);

    // Touch each CHARSXP, including the NA slots.
    let mut na_count = 0usize;
    for i in 0..nrow as isize {
        if label.string_elt_str(i).is_none() {
            na_count += 1;
        }
    }
    assert!(na_count > 0);

    drop(scope);
}

// endregion
