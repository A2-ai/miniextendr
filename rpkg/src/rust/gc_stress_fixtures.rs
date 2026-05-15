//! Fixtures for GC stress tests.
//!
//! Provides `SharedData` (R6 class) and `into_sexp_altrep` for the GC stress
//! and ALTREP serialization test suites.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::ffi::{SEXP, SEXPTYPE, SexpExt};
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
        StructFlattenEvent::Located { id: 1, origin: Point { x: 1.0, y: 2.0 } },
        StructFlattenEvent::Other { id: 2 },
        StructFlattenEvent::Located { id: 3, origin: Point { x: 3.0, y: 4.0 } },
        StructFlattenEvent::Other { id: 4 },
    ];
    let _ = StructFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = StructFlattenEvent::to_dataframe_split(flatten_rows).into_sexp();

    // as_list path: struct-typed field kept as opaque list-column.
    let list_rows = vec![
        StructListEvent::Located { id: 1, origin: Point { x: 1.0, y: 2.0 } },
        StructListEvent::Other { id: 2 },
        StructListEvent::Located { id: 3, origin: Point { x: 5.0, y: 6.0 } },
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
        NestedFlattenEvent::Tracked { id: 1, status: Status::Ok },
        NestedFlattenEvent::Other { id: 2 },
        NestedFlattenEvent::Tracked { id: 3, status: Status::Err { code: 404 } },
        NestedFlattenEvent::Other { id: 4 },
    ];
    let _ = NestedFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = NestedFlattenEvent::to_dataframe_split(flatten_rows).into_sexp();

    // as_factor path: unit-only enum field stored as factor column.
    let factor_rows = vec![
        NestedFactorEvent::Move { id: 1, dir: Direction::North },
        NestedFactorEvent::Stop { id: 2 },
        NestedFactorEvent::Move { id: 3, dir: Direction::East },
        NestedFactorEvent::Stop { id: 4 },
    ];
    let _ = NestedFactorEvent::to_dataframe(factor_rows.clone()).into_sexp();
    let _ = NestedFactorEvent::to_dataframe_split(factor_rows).into_sexp();

    // as_list path: enum field kept as opaque list-column.
    let list_rows = vec![
        NestedListEvent::Move { id: 1, dir: Direction::South },
        NestedListEvent::Stop { id: 2 },
        NestedListEvent::Move { id: 3, dir: Direction::West },
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
    use miniextendr_api::ffi::SexpExt as _;

    // Construct a small ALTREP-backed integer vector.
    let values = vec![10i32, 20, 30, 40, 50];
    let sexp = native_sexp_altrep_new(values);

    // Verify that it is indeed ALTREP.
    assert!(sexp.is_altrep(), "native_sexp_altrep_new should return an ALTREP SEXP");

    // Force element access (exercises the Elt path) via the SexpExt trait.
    let n = sexp.len();
    assert_eq!(n, 5);
    for i in 0..n {
        let v = sexp.integer_elt(i as isize);
        assert_eq!(v, (i as i32 + 1) * 10);
    }

    // Force the Dataptr path (exercises `AltVec::dataptr`).
    let _ptr = unsafe { miniextendr_api::ffi::DATAPTR_RO(sexp) };
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
    use miniextendr_api::ffi::SexpExt as _;
    let n = sexp.len();
    assert_eq!(n, 3);
    for i in 0..n {
        let _v = sexp.real_elt(i as isize);
    }
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
