//! Test fixtures for DataFusion integration (RSessionContext + RDataFrame).

use miniextendr_api::miniextendr;
use miniextendr_api::optionals::arrow_impl::RecordBatch;
use miniextendr_api::optionals::datafusion_impl::RSessionContext;

// region: RSessionContext — SQL queries

/// @export
#[miniextendr]
pub fn test_df_sql_query(df: RecordBatch, sql: &str) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("df", df).unwrap();
    ctx.sql_to_record_batch(sql).unwrap()
}

// endregion

// region: RDataFrame — lazy operations

/// @export
#[miniextendr]
pub fn test_df_select(df: RecordBatch, cols: Vec<String>) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
    ctx.sql("SELECT * FROM t")
        .unwrap()
        .select(&col_refs)
        .unwrap()
        .collect()
        .unwrap()
}

/// @export
#[miniextendr]
pub fn test_df_sort_limit(df: RecordBatch, col: &str, asc: bool, n: i32) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t")
        .unwrap()
        .sort(col, asc)
        .unwrap()
        .limit(n as usize)
        .unwrap()
        .collect()
        .unwrap()
}

/// @export
#[miniextendr]
pub fn test_df_columns(df: RecordBatch) -> Vec<String> {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t").unwrap().columns()
}

/// @export
#[miniextendr]
pub fn test_df_chain(df: RecordBatch) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t WHERE x > 2")
        .unwrap()
        .sort("x", true)
        .unwrap()
        .limit(3)
        .unwrap()
        .collect()
        .unwrap()
}

/// @export
#[miniextendr]
pub fn test_df_aggregate(df: RecordBatch) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t")
        .unwrap()
        .aggregate(&["name"], &[("total", "sum", "y"), ("cnt", "count", "y")])
        .unwrap()
        .sort("name", true)
        .unwrap()
        .collect()
        .unwrap()
}

/// @export
#[miniextendr]
pub fn test_df_global_agg(df: RecordBatch) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t")
        .unwrap()
        .aggregate(&[], &[("avg_y", "avg", "y"), ("max_x", "max", "x")])
        .unwrap()
        .collect()
        .unwrap()
}

/// @export
#[miniextendr]
pub fn test_df_count(df: RecordBatch) -> i32 {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql("SELECT * FROM t WHERE x > 2")
        .unwrap()
        .count()
        .unwrap() as i32
}

// endregion

// region: Upstream example-derived fixtures

/// JOIN two record batches via SQL.
/// @param left Left RecordBatch.
/// @param right Right RecordBatch.
/// @param sql SQL query with tables "left_t" and "right_t".
/// @export
#[miniextendr]
pub fn test_df_join(left: RecordBatch, right: RecordBatch, sql: &str) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("left_t", left).unwrap();
    ctx.register_record_batch("right_t", right).unwrap();
    ctx.sql_to_record_batch(sql).unwrap()
}

/// SQL window function query.
/// @param df RecordBatch to query.
/// @param sql SQL query using window functions.
/// @export
#[miniextendr]
pub fn test_df_window(df: RecordBatch, sql: &str) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql_to_record_batch(sql).unwrap()
}

/// SQL subquery.
/// @param df RecordBatch to query.
/// @param sql SQL query containing a subquery.
/// @export
#[miniextendr]
pub fn test_df_subquery(df: RecordBatch, sql: &str) -> RecordBatch {
    let ctx = RSessionContext::new();
    ctx.register_record_batch("t", df).unwrap();
    ctx.sql_to_record_batch(sql).unwrap()
}

// endregion
