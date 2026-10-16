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

#[derive(DataFrameRow)]
pub struct OptionalFactorRow {
    pub id: i32,
    #[dataframe(as_factor, rename = "stage")]
    pub level: Option<ReadLevel>,
    pub label: String,
}

#[derive(DataFrameRow)]
pub struct OptionalFactorTuple(
    #[dataframe(as_factor, rename = "stage")] Option<ReadLevel>,
    i32,
);

#[derive(DataFrameRow)]
pub struct ExpandedOptionalFactorRow {
    #[dataframe(as_factor)]
    pub level: ::core::option::Option<ReadLevel>,
    #[dataframe(expand)]
    pub values: Vec<i32>,
}

#[derive(DataFrameRow)]
pub struct NestedOptionalFactorRow {
    pub inner: OptionalFactorRow,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub struct TaggedOptionalFactorRow {
    #[dataframe(as_factor)]
    pub level: Option<ReadLevel>,
}

pub fn optional_factor_rows(empty: bool) -> Vec<OptionalFactorRow> {
    let levels = if empty {
        vec![]
    } else {
        vec![Some(ReadLevel::Low), None, Some(ReadLevel::High)]
    };
    levels
        .into_iter()
        .zip(1..)
        .map(|(level, id)| OptionalFactorRow {
            id,
            level,
            label: format!("row {id}"),
        })
        .collect()
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

/// Build named, tuple, nested, expanded and tagged struct factor columns.
#[miniextendr]
pub fn df_struct_factor(shape: &str, empty: bool) -> BuiltDataFrame {
    if let Some(shape) = shape.strip_prefix("optional-") {
        let rows = optional_factor_rows(empty);
        return match shape {
            "named" => rows.into_dataframe().unwrap(),
            "tuple" => rows
                .into_iter()
                .map(|r| OptionalFactorTuple(r.level, r.id))
                .collect::<Vec<_>>()
                .into_dataframe()
                .unwrap(),
            "nested" => rows
                .into_iter()
                .map(|inner| NestedOptionalFactorRow { inner })
                .collect::<Vec<_>>()
                .into_dataframe()
                .unwrap(),
            "expanded" => rows
                .into_iter()
                .map(|r| ExpandedOptionalFactorRow {
                    level: r.level,
                    values: vec![r.id, r.id * 10],
                })
                .collect::<Vec<_>>()
                .into_dataframe()
                .unwrap(),
            "tagged" => rows
                .into_iter()
                .map(|r| TaggedOptionalFactorRow { level: r.level })
                .collect::<Vec<_>>()
                .into_dataframe()
                .unwrap(),
            _ => panic!("unknown optional fixture shape: {shape}"),
        };
    }
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
        "optional-named" => OptionalFactorRow::try_from_dataframe(df)?.into_dataframe(),
        "optional-nested" => NestedOptionalFactorRow::try_from_dataframe(df)?.into_dataframe(),
        "optional-expanded" => ExpandedOptionalFactorRow::try_from_dataframe(df)?.into_dataframe(),
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
        "optional-named" => OptionalFactorRow::try_from_dataframe_par(df)?.into_dataframe_par(),
        "optional-nested" => {
            NestedOptionalFactorRow::try_from_dataframe_par(df)?.into_dataframe_par()
        }
        "optional-expanded" => {
            ExpandedOptionalFactorRow::try_from_dataframe_par(df)?.into_dataframe_par()
        }
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

    #[test]
    fn optional_factor_companion_preserves_owned_rows_and_missing_cells() {
        let companion = OptionalFactorRowDataFrame::from_rows(optional_factor_rows(false));
        let rows = companion.into_rows();
        assert_eq!(rows.len(), 3);
        assert!(matches!(rows[0].level, Some(ReadLevel::Low)));
        assert!(rows[1].level.is_none());
        assert!(matches!(rows[2].level, Some(ReadLevel::High)));
        assert_eq!(rows[1].label, "row 2");
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn optional_factor_parallel_fill_needs_no_default_or_clone() {
        let companion = OptionalFactorRowDataFrame::from_rows_par(optional_factor_rows(false));
        assert_eq!(companion.id, [1, 2, 3]);
        assert!(matches!(companion.stage[0], Some(ReadLevel::Low)));
        assert!(companion.stage[1].is_none());
        assert!(matches!(companion.stage[2], Some(ReadLevel::High)));
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn struct_factor_parallel_fill_needs_no_default_or_clone() {
        let companion = FactorRowDataFrame::from_rows_par(factor_rows(false));
        assert_eq!(companion.id, [1, 2, 3]);
        assert!(matches!(companion.stage[1], ReadLevel::High));
    }
}
