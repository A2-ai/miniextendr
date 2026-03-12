//! Test fixtures for macro bidirectional equivalence.
//!
//! These demonstrate that `#[miniextendr]` on structs/enums produces the same
//! trait implementations as the equivalent derive macros.

use miniextendr_api::miniextendr;

// ============================================================================
// 1. #[miniextendr] on multi-field struct → ExternalPtr
// ============================================================================

/// Multi-field struct: `#[miniextendr]` generates ExternalPtr + TypedExternal.
#[miniextendr]
pub struct MxPoint {
    pub x: f64,
    pub y: f64,
}

/// @noRd
#[miniextendr]
pub fn mx_point_new(x: f64, y: f64) -> MxPoint {
    MxPoint { x, y }
}

/// @noRd
#[miniextendr]
pub fn mx_point_sum(p: miniextendr_api::externalptr::ExternalPtr<MxPoint>) -> f64 {
    p.x + p.y
}

// ============================================================================
// 2. #[miniextendr(list)] on struct → IntoList + TryFromList + PreferList
// ============================================================================

/// Struct with list mode: generates IntoList + TryFromList + PreferList.
/// PreferList includes IntoR that routes through IntoList, so this type
/// can be returned directly from #[miniextendr] functions.
#[miniextendr(list)]
pub struct MxRecord {
    pub name: String,
    pub value: i32,
}

/// @noRd
#[miniextendr]
pub fn mx_record_create(name: String, value: i32) -> MxRecord {
    MxRecord { name, value }
}

// ============================================================================
// 3. #[miniextendr(dataframe)] on struct → DataFrameRow + PreferDataFrame
// ============================================================================

/// Struct with dataframe mode: generates IntoList + DataFrameRow + IntoR on companion.
/// The companion MxObsDataFrame type can be returned directly from #[miniextendr] functions.
#[miniextendr(dataframe)]
pub struct MxObs {
    pub id: i32,
    pub score: f64,
}

/// @noRd
#[miniextendr]
pub fn mx_obs_create() -> MxObsDataFrame {
    MxObs::to_dataframe(vec![
        MxObs { id: 1, score: 0.5 },
        MxObs { id: 2, score: 0.8 },
    ])
}

// ============================================================================
// 4. #[miniextendr] on fieldless enum → RFactor
// ============================================================================

/// Fieldless enum: `#[miniextendr]` generates RFactor + IntoR + TryFromSexp.
#[miniextendr]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MxSeason {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// @noRd
#[miniextendr]
pub fn mx_season_summer() -> MxSeason {
    MxSeason::Summer
}

/// @noRd
#[miniextendr]
pub fn mx_season_name(s: MxSeason) -> String {
    format!("{:?}", s)
}

// ============================================================================
// 5. #[miniextendr(match_arg)] on fieldless enum → MatchArg
// ============================================================================

/// Fieldless enum with match_arg: generates MatchArg + IntoR + TryFromSexp.
#[miniextendr(match_arg)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MxVerbosity {
    Quiet,
    Normal,
    Verbose,
}

/// @noRd
#[miniextendr]
pub fn mx_verbosity_check(v: MxVerbosity) -> String {
    format!("{:?}", v)
}

// ============================================================================
// 6. #[derive(Altrep)] on 1-field struct → ALTREP registration
// ============================================================================

/// 1-field struct via derive: generates ALTREP class registration.
#[derive(miniextendr_api::Altrep)]
pub struct MxDerivedInts(Vec<i32>);

/// @noRd
#[miniextendr]
pub fn mx_derived_ints() -> MxDerivedInts {
    MxDerivedInts(vec![10, 20, 30])
}

// ============================================================================
// Module registration
// ============================================================================
