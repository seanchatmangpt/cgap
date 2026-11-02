# research-duckdb-arrow

R3 explore-type research receipt: DuckDB ↔ Arrow/Feather interop mechanics, as of 2026-11-01,
all claims fetched live this session (curl + exit 0 unless noted). No code changes.

## 1. Arrow extension: arrow_scan, pushdown, zero-copy and its limits

**Lifecycle (load-bearing finding).** The `arrow` extension is DEPRECATED. Starting with
DuckDB v1.3 it was archived; its replacement is the community `nanoarrow` extension
(also aliased `INSTALL arrow FROM community`).

- https://duckdb.org/docs/1.2/extensions/arrow.html ("The Arrow extension is deprecated...
  consider using the nanoarrow community extension instead")
- https://raw.githubusercontent.com/duckdb/arrow/master/README.md ("Starting with DuckDB
  v1.3, this extension will be archived... replacement... nanoarrow")

**Surface (both old arrow and new nanoarrow):**

| function | type | role |
|---|---|---|
| `scan_arrow_ipc` | table function | scan a list of pointers to Arrow IPC buffers |
| `to_arrow_ipc` | table in-out function | serialize a table into IPC blobs |
| `read_arrow` | table function (nanoarrow only) | read `.arrow` (IPC file = Feather V2) / `.arrows` (IPC stream) files |
| `arrow_kv_metadata` / `nanoarrow_version` | table / scalar | metadata + version |

- https://duckdb.org/docs/1.2/extensions/arrow.html (functions table)
- https://duckdb.org/community_extensions/extensions/nanoarrow (functions table + install)

**Ingest path (how scan_arrow_ipc works).** The ingest path is the Arrow C Data Interface:
an `ArrowArrayStream` (schema + get_next ArrowArray batches) is handed to DuckDB as pointers
into producer-owned memory. `duckdb_arrow_scan` / `duckdb_arrow_array_scan` in the C API
register an `ArrowArrayStream`/array as a viewable table; the nanoarrow `scan_arrow_ipc`
table function scans "a list of pointers pointing to Arrow IPC buffers". Per the nanoarrow
page: "reading buffers is dangerous, as an incorrect pointer can crash the database system."
Buffer pointers imply NO defensive validation — producer liveness is the caller's contract.

- https://duckdb.org/community_extensions/extensions/nanoarrow
- https://duckdb.org/docs/2.0/clients/c/api (duckdb_arrow_scan, duckdb_arrow_array_scan)

**Pushdown: what actually reaches the Arrow producer.** Tiered by object type (2.0 docs,
SQL on Arrow guide):

- Arrow **Dataset**: DuckDB pushes column projections AND translatable row filters into the
  dataset scan — "only the necessary data is pulled into memory."
- Arrow **Scanner**: pushdown happens at Arrow-compute level (the scanner's own filter is
  applied by Arrow; DuckDB does not push new filters into an existing Scanner).
- Arrow **Table / RecordBatch / RecordBatchReader**: "read in full and filtered afterwards";
  a RecordBatchReader can be scanned only once (single-consumer stream).
- https://duckdb.org/docs/2.0/guides/python/sql_on_arrow (pushdown paragraph + per-object
  sections; note on RecordBatchReader single scan; parallel-scan row-order nondeterminism
  for order-dependent aggregates like `first(i)` without ORDER BY)

**Zero-copy guarantees and their limits.**

- Guarantee (2021 announcement, cross-posted to Arrow blog): DuckDB consumes Arrow batches
  "without requiring any extra data copying" — batches arrive as ArrowArray buffers the
  DuckDB vectors point into; results stream back as RecordBatchReaders.
  https://duckdb.org/2021/12/03/duck-arrow.html
- Limits (same sources):
  1. Zero-copy holds at batch granularity only; DuckDB materializes what its operators
     need (hash tables, spills) downstream of the scan — the scan boundary is what is
     copy-free, not the whole query.
  2. Tables/RecordBatchReaders are fully materialized into the scan before filtering
     (sql_on_arrow guide, above) — zero-copy does not mean lazy.
  3. Single-shot streams: a RecordBatchReader is consumable once (sql_on_arrow guide).
  4. Pointer validity is caller-owned (nanoarrow: wrong pointer crashes the DB).
  5. Type coverage: DuckDB converts Arrow types it cannot natively execute; conversion
     happens at the scan boundary (C API exposes `duckdb_data_chunk_from_arrow` taking a
     `duckdb_arrow_converted_schema` — an explicit convert step exists in the API).
     https://duckdb.org/docs/2.0/clients/c/api

## 2. Feather read/write in DuckDB

**No first-class "feather" surface exists in current docs (2.0).** Grep of the 2.0 doc
tree for "feather": no matches on the data-overview, parquet-overview, or client-API
pages fetched this session. Feather support arrives disguised as Arrow IPC:

- **Feather V2 == Arrow IPC file format**, so nanoarrow's `FROM 'arrow_file.arrow'` /
  `FROM read_arrow('arrow_file.arrow')` reads Feather V2 files; `.arrows` (IPC stream)
  via the same path. https://duckdb.org/community_extensions/extensions/nanoarrow
- **Write**: `COPY tbl TO 'f.arrows' (FORMAT ARROWS, BATCH_SIZE 100)` (IPC stream) and
  `to_arrow_ipc` producing IPC blobs. https://duckdb.org/community_extensions/extensions/nanoarrow
- **Feather V1**: no documented support in any version fetched this session (1.2/1.4/2.0
  docs checked). Feather V1 is the legacy pre-Arrow-0.15 format; DuckDB has never
  documented a reader for it. (Claim scoped to DuckDB docs, not the codebase — UNVERIFIED
  at source level.)
- **Streaming vs materialization**: `read_arrow` on a file scans lazily per IPC record
  batch (file format supports incremental batch decode); buffer-pointer scans
  (`scan_arrow_ipc`) hand DuckDB already-materialized buffers — the copy-free boundary is
  the pointer handoff, not file decoding. Nanoarrow page notes the buffer-pointer path is
  "temporary and will be deprecated in the future, as clients... will have a function that
  internally extracts these buffers from an Arrow stream."
  https://duckdb.org/community_extensions/extensions/nanoarrow

## 3. Performance characteristics (documented, not measured here)

**Conversion cost (copy vs zero-copy).** The documented picture:

- Scan-boundary Arrow→DuckDB is zero-copy for types DuckDB executes natively (buffers
  shared, not copied) — 2021 announcement: compose queries across engines "without paying
  any cost to (re)serialize the data when you pass it back and forth."
  https://duckdb.org/2021/12/03/duck-arrow.html
- The explicit conversion seam is visible in the C API:
  `duckdb_data_chunk_from_arrow(connection, ArrowArray*, duckdb_arrow_converted_schema,
  duckdb_data_chunk*)` + `duckdb_destroy_arrow_converted_schema` — a nontrivial schema
  convert step exists precisely because not every Arrow type maps 1:1 to a DuckDB physical
  type. https://duckdb.org/docs/2.0/clients/c/api
- Materialization costs appear when DuckDB cannot push down: Arrow Tables /
  RecordBatchReaders are "read in full and filtered afterwards."
  https://duckdb.org/docs/2.0/guides/python/sql_on_arrow

**Documented benchmark points (DuckDB's own post; TARGETs for cgap, not measured):**

- Projection pushdown, lineitem SF1 (`sum(l_extendedprice*l_discount)` over 16 cols):
  DuckDB 0.19 s vs Pandas 2.13 s — ~11x, attributed to DuckDB pushing the 2-column
  projection into the Arrow scan so only needed columns are touched.
- Filter pushdown (adds 4 filter columns): DuckDB similarly an order of magnitude ahead
  of Pandas; pushdown reaches partition elimination in the backing Parquet files.
  https://duckdb.org/2021/12/03/duck-arrow.html (Benchmark Comparison section; hardware:
  SciLens cluster for NYC taxi, M1 MacBook Pro for TPC-H; DuckDB parallel, pandas
  single-threaded — the post discloses this asymmetry itself)

**Consequence for cgap:** the performance story of DuckDB↔Arrow is entirely about what
the scan refuses to touch (columns not projected, partitions not opened), not about the
batch handoff speed. A cgap DuckDB path wins or loses on projection/filter pushdown
support, not on buffer plumbing.

## 4. C API / extension surface for native cgap→DuckDB scan

What a custom table function must expose to make cgap-decoded columns DuckDB-queryable,
from the documented C API table-function contract
(https://duckdb.org/docs/2.0/clients/c/table_functions — full API fetched this session):

1. **Registration**: `duckdb_create_table_function` → `set_name`,
   `add_parameter`/`add_named_parameter` (e.g. a VARCHAR path or BLOB pointer argument),
   `set_extra_info` (+destroy callback), then `duckdb_register_table_function(con, f)`.
   Registration fails (DuckDBError) if incomplete or name-colliding.
2. **bind**: declare output columns via `duckdb_bind_add_result_column(name,
   duckdb_logical_type)`; stash scan state (file handle, footer, row-group index) in
   `duckdb_bind_set_bind_data` (read-only after bind; destroy callback mandatory);
   optionally `duckdb_bind_set_cardinality(is_exact)` for the optimizer, and report
   bind-time failures via `duckdb_bind_set_error`.
3. **init / local_init**: read the projected column list with
   `duckdb_init_get_column_count` / `duckdb_init_get_column_index` (only populated if
   `duckdb_table_function_supports_projection_pushdown(true)` was set — default false
   means DuckDB expects all columns); set parallelism via `duckdb_init_set_max_threads`
   (default 1); allocate per-thread state via `duckdb_init_set_init_data` /
   local_init.
4. **main function**: called repeatedly until it signals exhaustion; fill DuckDB data
   chunks (the chunk surface lives in https://duckdb.org/docs/2.0/clients/c/data_chunk and
   the api ref, including `duckdb_result_arrow_array` to emit a chunk AS an ArrowArray);
   errors via `duckdb_function_set_error`.
5. **Alternative — the Arrow-stream route**: instead of a table function, implement the
   Arrow C Data Interface producer (`ArrowArrayStream`) and register with
   `duckdb_arrow_scan(connection, table_name, duckdb_arrow_stream)` /
   `duckdb_arrow_array_scan(...)` (api ref). This is what the deprecated arrow extension
   and nanoarrow do; it gets DuckDB's Arrow scan machinery for free but inherits the
   materialize-then-filter semantics for non-dataset producers (section 1).

**Implication for stardf DuckDB-queryability:** two concrete, spec-shaped options —
(a) C Data Interface producer + `duckdb_arrow_scan` (cheapest; pushdown limited to what
the Arrow scan path gives), or (b) native table function with
`supports_projection_pushdown(true)` so cgap decodes only requested columns from its
columnar layout (matches the measured driver of DuckDB-Arrow performance, section 3).
Filter pushdown is NOT part of the documented C table-function API fetched this session
(projection pushdown is; filter pushdown appears only at the Arrow-Dataset tier and via
the C++ `TableFunctionRef`/statistics internals — recorded as a gap, not invented).

## Closing: what cgap-parquet / cgap-ffi would implement for native DuckDB scan

Spec-shaped, honest about gaps:

1. **cgap-ffi**: an Arrow C Data Interface producer — export each decoded cgap column
   batch as `ArrowArray` + `ArrowSchema` wrapped in an `ArrowArrayStream` (get_next /
   get_schema / release with the C ABI mandated by A04's CgapFrameHeader lane family).
   This is the minimum for `duckdb_arrow_scan` and for pyarrow/pandas consumers.
2. **cgap-parquet**: an Arrow-IPC-compatible batch decoder path so a DuckDB user can go
   through the (deprecated-but-aliased) `arrow`/`nanoarrow` `read_arrow` on cgap-written
   `.arrows` output — i.e., cgap-parquet's writer emits standard Arrow IPC streams.
3. **Later / optional**: a native DuckDB table function (section 4 list) registering
   `cgap_scan(path)` with projection pushdown so DuckDB queries decode only projected
   cgap columns. Requires the C table-function bind/init/main contract above; filter
   pushdown beyond projection is not reachable through the documented C API surface and
   is recorded here as UNKNOWN, not claimed.
4. **Non-goals found in research**: Feather V1 support (no DuckDB reader exists);
   buffer-pointer scans into cgap-owned memory without a liveness protocol (nanoarrow
   documents crash-on-bad-pointer; cgap would need a pinned-buffer contract first).

## Sources (all fetched 2026-11-01)

- https://duckdb.org/docs/2.0/guides/python/sql_on_arrow
- https://duckdb.org/docs/2.0/guides/python/import_arrow
- https://duckdb.org/docs/1.2/extensions/arrow.html
- https://duckdb.org/community_extensions/extensions/nanoarrow
- https://raw.githubusercontent.com/duckdb/arrow/master/README.md
- https://duckdb.org/2021/12/03/duck-arrow.html
- https://duckdb.org/docs/2.0/clients/c/table_functions
- https://duckdb.org/docs/2.0/clients/c/api
- https://duckdb.org/docs/2.0/data/overview (link discovery)

## Standing

ALIVE as a research receipt (every claim URL-cited, fetched this session via curl exit 0;
benchmark numbers are DuckDB-published, method stated in-source — TARGETs for cgap until
cgap measures its own). Feather-V1-absence claim is UNVERIFIED at codebase level
(docs-only sweep).
