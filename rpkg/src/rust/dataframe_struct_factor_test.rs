//! Struct `as_factor` columns: both assembly paths and readers (#2025).

use crate::dataframe_bounds_test::{ReadInner, ReadLevel, ReadRow};
use miniextendr_api::{BuiltDataFrame, DataFrameRow, IntoDataFrame, SEXP, miniextendr};

// ReadLevel is neither Clone, Copy nor Default.
#[derive(DataFrameRow)]
pub struct FactorRow {
    pub id: i32,
    #[dataframe(as_factor, rename = "stage")]
    pub level: ReadLevel,
    pub label: String,
}

#[derive(DataFrameRow)]
pub struct FactorTuple(#[dataframe(as_factor, rename = "stage")] ReadLevel, i32);

#[derive(DataFrameRow)]
pub struct ExpandedFactorRow {
    #[dataframe(as_factor)]
    pub level: ReadLevel,
    #[dataframe(expand)]
    pub values: Vec<i32>,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub struct TaggedFactorRow {
    #[dataframe(as_factor)]
    pub level: ReadLevel,
}

/// A struct factor inside an enum variant: the parent pads `Stop` rows with NA.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum FactorEvent {
    Step { step: FactorRow },
    Stop { id: i32 },
}

pub fn factor_rows(empty: bool) -> Vec<FactorRow> {
    let levels = if empty {
        vec![]
    } else {
        vec![ReadLevel::Low, ReadLevel::High, ReadLevel::Low]
    };
    levels
        .into_iter()
        .zip(1..)
        .map(|(level, id)| FactorRow {
            id,
            level,
            label: format!("row {id}"),
        })
        .collect()
}

/// Build named, tuple, nested, expanded, tagged and enum-variant struct factor
/// columns.
#[miniextendr]
pub fn df_struct_factor(shape: &str, empty: bool) -> BuiltDataFrame {
    let rows = factor_rows(empty);
    match shape {
        "named" => rows.into_dataframe().unwrap(),
        "tuple" => rows
            .into_iter()
            .map(|r| FactorTuple(r.level, r.id))
            .collect::<Vec<_>>()
            .into_dataframe()
            .unwrap(),
        "nested" => rows
            .into_iter()
            .map(|r| ReadRow {
                id: r.id,
                inner: ReadInner { q: r.id * 10 },
                level: r.level,
            })
            .collect::<Vec<_>>()
            .into_dataframe()
            .unwrap(),
        "expanded" => rows
            .into_iter()
            .map(|r| ExpandedFactorRow {
                level: r.level,
                values: vec![r.id, r.id * 10],
            })
            .collect::<Vec<_>>()
            .into_dataframe()
            .unwrap(),
        "tagged" => rows
            .into_iter()
            .map(|r| TaggedFactorRow { level: r.level })
            .collect::<Vec<_>>()
            .into_dataframe()
            .unwrap(),
        "enum" => rows
            .into_iter()
            .map(|r| match r.id {
                3 => FactorEvent::Stop { id: r.id },
                _ => FactorEvent::Step { step: r },
            })
            .collect::<Vec<_>>()
            .into_dataframe()
            .unwrap(),
        _ => panic!("unknown fixture shape: {shape}"),
    }
}

/// Read struct factor columns back into owned, non-Clone enum values.
#[miniextendr]
pub fn df_struct_factor_roundtrip(df: SEXP, shape: &str) -> Result<BuiltDataFrame, String> {
    match shape {
        "named" => FactorRow::try_from_dataframe(df)?.into_dataframe(),
        "nested" => ReadRow::try_from_dataframe(df)?.into_dataframe(),
        "expanded" => ExpandedFactorRow::try_from_dataframe(df)?.into_dataframe(),
        "enum" => FactorEvent::try_from_dataframe(df)?.into_dataframe(),
        _ => panic!("unknown readable fixture shape: {shape}"),
    }
    .map_err(|e| e.to_string())
}

/// Parallel readers retain the same non-Clone factor semantics.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn df_struct_factor_roundtrip_par(df: SEXP, shape: &str) -> Result<BuiltDataFrame, String> {
    match shape {
        "named" => FactorRow::try_from_dataframe_par(df)?.into_dataframe_par(),
        "nested" => ReadRow::try_from_dataframe_par(df)?.into_dataframe_par(),
        "expanded" => ExpandedFactorRow::try_from_dataframe_par(df)?.into_dataframe_par(),
        "enum" => FactorEvent::try_from_dataframe_par(df)?.into_dataframe_par(),
        _ => panic!("unknown readable fixture shape: {shape}"),
    }
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use miniextendr_api::ColumnarFrame;

    #[test]
    fn struct_factor_companion_preserves_owned_rows() {
        let companion = FactorRowDataFrame::from_rows(factor_rows(false));
        let rows = companion.into_rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].id, 2);
        assert!(matches!(rows[1].level, ReadLevel::High));
        assert_eq!(rows[1].label, "row 2");
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn struct_factor_parallel_fill_needs_no_default_or_clone() {
        let companion = FactorRowDataFrame::from_rows_par(factor_rows(false));
        assert_eq!(companion.id, [1, 2, 3]);
        assert!(matches!(companion.stage[1], ReadLevel::High));
    }
}
