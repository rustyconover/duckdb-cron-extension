# Building The Extension

Building this DuckDB extension is a bit complicated since DuckDB does not currently have a C API for getting the timestamp value from a `duckdb_value`.

I've created a branch of DuckDB that adds this support.

https://github.com/rustyconover/duckdb/tree/feat_timestamp_duckdb_value

Since Rust is being used to interface with DuckDB, the duckdb-rs crate must also be updated to expose the new functions from the DuckDB C API.

https://github.com/rustyconover/duckdb-rs/tree/feat_add_timestamp_for_duckdb_value