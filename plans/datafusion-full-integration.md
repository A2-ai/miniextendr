# Plan: Full DataFusion integration (SQL + DataFrame API)

## Goal

Extend the current thin `datafusion` feature (RSessionContext with block_on) into a
complete query interface: register R data, run SQL, use DataFrame API, read/write files.

## Current state (already implemented)

- `arrow` feature: zero-copy R↔Arrow for all types (Float64Array, Int32Array, etc.)
- `datafusion` feature: RSessionContext with `register_record_batch`, `sql_to_record_batch`
- Factor↔DictionaryArray, Date↔Date32Array, POSIXct↔TimestampSecondArray

## Additions

### Phase 1: DataFrame API wrapper
- `RDataFrame` wrapping DataFusion's `DataFrame` with sync methods
- `.filter(expr)`, `.select(cols)`, `.aggregate(group_by, aggs)`, `.limit(n)`
- `.sort(col, asc)`, `.join(other, on, type)`
- `.collect() -> RecordBatch`, `.show() -> String`

### Phase 2: File I/O
- Enable DataFusion's `parquet` feature (optional sub-feature)
- `ctx.read_parquet(path)`, `ctx.read_csv(path)`, `ctx.read_json(path)`
- `df.write_parquet(path)`, `df.write_csv(path)`

### Phase 3: UDF bridge
- Register R functions as DataFusion scalar UDFs
- `ctx.register_r_function("my_func", function(x) x * 2)`
- Requires callback from Rust→R via with_r_thread

### Phase 4: Expression builder
- Type-safe expression construction: `col("x").gt(lit(5))`
- Aggregate expressions: `sum(col("x"))`, `count(col("*"))`
- Window functions

## Dependencies

- `datafusion` with additional features: `parquet` (optional)
- Current: `datafusion = { version = "48", default-features = false }`
- Phase 2: add `features = ["parquet"]` sub-feature

## Deferred

- Streaming execution (would need async R integration)
- Custom table providers (R connections as data sources)
- Catalog/schema management
