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
    pub fn register_record_batch(
        &self,
        name: &str,
        batch: RecordBatch,
    ) -> Result<(), String> {
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
            return Ok(RecordBatch::new_empty(Arc::new(arrow_schema::Schema::empty())));
        }

        if batches.len() == 1 {
            return Ok(batches.into_iter().next().unwrap());
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
