//! Integration with Apache DataFusion query engine.
//!
//! This module provides a thin sync wrapper around DataFusion's async API,
//! letting users run SQL queries on R data frames from `#[miniextendr]` functions.
//!
//! # Architecture
//!
//! DataFusion is async (Tokio-based). This module uses `tokio::runtime::Runtime::block_on`
//! internally so that `#[miniextendr]` functions remain synchronous. A per-thread Tokio
//! runtime is created lazily on first use.
//!
//! # Example
//!
//! ```rust,ignore
//! use miniextendr_api::datafusion_impl::RSessionContext;
//! use miniextendr_api::arrow_impl::RecordBatch;
//!
//! #[miniextendr]
//! fn query_dataframe(df: RecordBatch, sql: &str) -> RecordBatch {
//!     let ctx = RSessionContext::new();
//!     ctx.register_record_batch("df", df).unwrap();
//!     ctx.sql_to_record_batch(sql).unwrap()
//! }
//! ```
//!
//! # Features
//!
//! Enable with `features = ["datafusion"]` (implies `arrow`):
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["datafusion"] }
//! ```

pub use datafusion;

use datafusion::prelude::*;
use std::sync::Arc;

use super::arrow_impl::RecordBatch;
use crate::ffi::SEXP;
use crate::from_r::{SexpError, TryFromSexp};

/// Get or create a thread-local Tokio runtime for blocking on async DataFusion operations.
fn runtime() -> &'static tokio::runtime::Runtime {
    thread_local! {
        static RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new()
            .expect("failed to create Tokio runtime for DataFusion");
    }
    // SAFETY: The runtime is thread-local and lives for the thread's lifetime.
    RT.with(|rt| unsafe { &*(rt as *const tokio::runtime::Runtime) })
}

/// A synchronous wrapper around DataFusion's `SessionContext`.
///
/// This type manages a DataFusion session with a Tokio runtime for blocking
/// on async operations. It can be stored in an `ExternalPtr` for use across
/// multiple R function calls.
///
/// # Example
///
/// ```rust,ignore
/// let ctx = RSessionContext::new();
/// ctx.register_record_batch("my_table", batch)?;
/// let result = ctx.sql_to_record_batch("SELECT * FROM my_table WHERE x > 5")?;
/// ```
pub struct RSessionContext {
    ctx: SessionContext,
}

impl RSessionContext {
    /// Create a new DataFusion session with default configuration.
    pub fn new() -> Self {
        Self {
            ctx: SessionContext::new(),
        }
    }

    /// Create a session from an existing DataFusion `SessionContext`.
    pub fn from_context(ctx: SessionContext) -> Self {
        Self { ctx }
    }

    /// Get a reference to the underlying `SessionContext`.
    pub fn context(&self) -> &SessionContext {
        &self.ctx
    }

    /// Get a mutable reference to the underlying `SessionContext`.
    pub fn context_mut(&mut self) -> &mut SessionContext {
        &mut self.ctx
    }

    /// Register a RecordBatch as a named table.
    pub fn register_record_batch(&self, name: &str, batch: RecordBatch) -> Result<(), String> {
        let schema = batch.schema();
        let table = datafusion::datasource::MemTable::try_new(schema, vec![vec![batch]])
            .map_err(|e| e.to_string())?;
        self.ctx
            .register_table(name, Arc::new(table))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Execute a SQL query and collect results into a single RecordBatch.
    ///
    /// Multiple result batches are concatenated into one.
    pub fn sql_to_record_batch(&self, query: &str) -> Result<RecordBatch, String> {
        let rt = runtime();
        let df = rt
            .block_on(self.ctx.sql(query))
            .map_err(|e| e.to_string())?;
        let batches: Vec<RecordBatch> = rt.block_on(df.collect()).map_err(|e| e.to_string())?;

        if batches.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(
                arrow_schema::Schema::empty(),
            )));
        }

        if batches.len() == 1 {
            return Ok(batches.into_iter().next().expect("len checked == 1"));
        }

        let schema = batches[0].schema();
        arrow_select::concat::concat_batches(&schema, &batches).map_err(|e| e.to_string())
    }

    /// Execute a SQL query and return results as `Vec<RecordBatch>`.
    pub fn sql_to_batches(&self, query: &str) -> Result<Vec<RecordBatch>, String> {
        let rt = runtime();
        let df = rt
            .block_on(self.ctx.sql(query))
            .map_err(|e| e.to_string())?;
        rt.block_on(df.collect()).map_err(|e| e.to_string())
    }

    /// Execute a SQL query and return an `RDataFrame` for further operations.
    ///
    /// The query is not executed until `.collect()` is called.
    pub fn sql(&self, query: &str) -> Result<RDataFrame, String> {
        let rt = runtime();
        let df = rt
            .block_on(self.ctx.sql(query))
            .map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Register a CSV file as a named table.
    pub fn register_csv(&self, name: &str, path: &str) -> Result<(), String> {
        let rt = runtime();
        rt.block_on(self.ctx.register_csv(name, path, Default::default()))
            .map_err(|e| e.to_string())
    }
}

impl Default for RSessionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// TryFromSexp: create RSessionContext from R NULL (new session).
///
/// This allows `#[miniextendr]` functions to accept `RSessionContext`
/// parameters — passing NULL creates a fresh session.
impl TryFromSexp for RSessionContext {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{SEXPTYPE, SexpExt};
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(Self::new())
        } else {
            Err(SexpError::InvalidValue(
                "RSessionContext can only be created from NULL (use ExternalPtr for existing sessions)".into(),
            ))
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// TypedExternal for ExternalPtr support
use crate::externalptr::TypedExternal;

impl TypedExternal for RSessionContext {
    const TYPE_NAME: &'static str = "datafusion::RSessionContext";
    const TYPE_NAME_CSTR: &'static [u8] = b"datafusion::RSessionContext\0";
    const TYPE_ID_CSTR: &'static [u8] = b"datafusion::RSessionContext\0";
}

// region: RDataFrame — sync wrapper around DataFusion DataFrame

/// A synchronous wrapper around DataFusion's `DataFrame`.
///
/// Returned by `RSessionContext::sql()`. Provides chainable operations
/// (select, sort, limit) that build a query plan. The plan is executed
/// lazily — only when `.collect()` is called.
///
/// For filtering, use SQL in `ctx.sql("SELECT * FROM t WHERE x > 5")`.
///
/// # Example
///
/// ```rust,ignore
/// let ctx = RSessionContext::new();
/// ctx.register_record_batch("t", batch)?;
/// let result = ctx.sql("SELECT * FROM t WHERE x > 5")?
///     .select(&["x", "y"])?
///     .sort("x", true)?
///     .limit(10)?
///     .collect()?;
/// ```
pub struct RDataFrame {
    df: datafusion::dataframe::DataFrame,
}

impl RDataFrame {
    /// Create from a DataFusion DataFrame.
    pub fn from_inner(df: datafusion::dataframe::DataFrame) -> Self {
        Self { df }
    }

    /// Select columns by name.
    pub fn select(self, columns: &[&str]) -> Result<Self, String> {
        let exprs: Vec<datafusion::prelude::Expr> = columns
            .iter()
            .map(|c| datafusion::prelude::col(*c))
            .collect();
        let df = self.df.select(exprs).map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Sort by a column.
    pub fn sort(self, column: &str, ascending: bool) -> Result<Self, String> {
        let sort_expr = datafusion::prelude::col(column).sort(ascending, !ascending);
        let df = self.df.sort(vec![sort_expr]).map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Limit the number of rows returned.
    pub fn limit(self, n: usize) -> Result<Self, String> {
        let df = self.df.limit(0, Some(n)).map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Execute the query plan and collect all results into a single RecordBatch.
    pub fn collect(self) -> Result<RecordBatch, String> {
        let rt = runtime();
        let batches: Vec<RecordBatch> =
            rt.block_on(self.df.collect()).map_err(|e| e.to_string())?;

        if batches.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(
                arrow_schema::Schema::empty(),
            )));
        }

        if batches.len() == 1 {
            return Ok(batches.into_iter().next().expect("len checked == 1"));
        }

        let schema = batches[0].schema();
        arrow_select::concat::concat_batches(&schema, &batches).map_err(|e| e.to_string())
    }

    /// Get column names.
    pub fn columns(&self) -> Vec<String> {
        self.df
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }

    /// Aggregate with group-by columns and aggregate expressions.
    ///
    /// `group_by` — column names to group by (empty for global aggregation).
    /// `aggr` — aggregate expressions as `("output_name", "function", "column")` tuples.
    ///
    /// Supported functions: `"sum"`, `"avg"`, `"min"`, `"max"`, `"count"`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // SELECT name, SUM(y) as total FROM t GROUP BY name
    /// df.aggregate(&["name"], &[("total", "sum", "y")])?;
    /// ```
    pub fn aggregate(self, group_by: &[&str], aggr: &[(&str, &str, &str)]) -> Result<Self, String> {
        use datafusion::functions_aggregate::expr_fn;
        use datafusion::prelude::*;

        let group_exprs: Vec<Expr> = group_by.iter().map(|c| col(*c)).collect();

        let aggr_exprs: Vec<Expr> = aggr
            .iter()
            .map(|(alias, func, column)| {
                let expr = match *func {
                    "sum" => expr_fn::sum(col(*column)),
                    "avg" => expr_fn::avg(col(*column)),
                    "min" => expr_fn::min(col(*column)),
                    "max" => expr_fn::max(col(*column)),
                    "count" => expr_fn::count(col(*column)),
                    other => {
                        return Err(format!(
                            "unknown aggregate function '{other}' (supported: sum, avg, min, max, count)"
                        ))
                    }
                };
                Ok(expr.alias(*alias))
            })
            .collect::<Result<_, String>>()?;

        let df = self
            .df
            .aggregate(group_exprs, aggr_exprs)
            .map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Join with another RDataFrame.
    ///
    /// `right` — the other DataFrame to join.
    /// `on` — column names to join on (must exist in both).
    /// `join_type` — one of `"inner"`, `"left"`, `"right"`, `"full"`.
    pub fn join(self, right: RDataFrame, on: &[&str], join_type: &str) -> Result<Self, String> {
        use datafusion::prelude::JoinType;

        let jt = match join_type {
            "inner" => JoinType::Inner,
            "left" => JoinType::Left,
            "right" => JoinType::Right,
            "full" => JoinType::Full,
            other => {
                return Err(format!(
                    "unknown join type '{other}' (supported: inner, left, right, full)"
                ));
            }
        };

        let join_cols: Vec<&str> = on.to_vec();
        let df = self
            .df
            .join(right.df, jt, &join_cols, &join_cols, None)
            .map_err(|e| e.to_string())?;
        Ok(RDataFrame { df })
    }

    /// Return the number of rows (executes the query).
    pub fn count(self) -> Result<usize, String> {
        let rt = runtime();
        rt.block_on(self.df.count()).map_err(|e| e.to_string())
    }

    /// Get the underlying DataFusion DataFrame (for advanced use).
    pub fn into_inner(self) -> datafusion::dataframe::DataFrame {
        self.df
    }
}

impl TypedExternal for RDataFrame {
    const TYPE_NAME: &'static str = "datafusion::RDataFrame";
    const TYPE_NAME_CSTR: &'static [u8] = b"datafusion::RDataFrame\0";
    const TYPE_ID_CSTR: &'static [u8] = b"datafusion::RDataFrame\0";
}

// endregion
