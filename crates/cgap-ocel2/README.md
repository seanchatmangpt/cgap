# cgap-ocel2

cgap -> OCEL 2.0 projection membrane (standalone, `crates/cli` pattern:
own `[workspace]`, NOT a root-workspace member, `publish = false`,
CalVer 26.11.3). A typed, refuse-closed projection of the ring-commit
receipt chain (`cgap_ffi::receipt_chain::RingReceiptChain`) into three
OCEL 2.0 tables — events / objects / E2O edges — emitted as real
`cgap-arrow` `Batch` columnar buffers plus a documented SQLite/DuckDB
view DDL (`view` module) so pm4py can consume the result.

## The view module (pm4py SQL view DDL)

Four frozen DDL statements: `EVENTS_DDL`, `OBJECTS_DDL`, `E2O_DDL`,
`OCEL_FLAT_VIEW_DDL` (the pm4py-facing flat view joining events to
objects through E2O). `full_ddl()` joins them in dependency order.
`event_id` is the receipt-chain ordinal (logical sequence), NOT a wall
clock — there is deliberately no `ocel:timestamp` column.

## Dependency surface

- Consumes `cgap-ffi`'s caller-side AoQ event tap rows
  (`cgap-ffi/src/aoq_events.rs` `AoqEventRow`): `TapEvent` maps 1:1
  onto the frozen 64-byte X6 row (`event_id = chain-ordinal` law
  mirrors `AoqEventRow::ordinal`). cgap-ffi does NOT depend on
  cgap-ocel2 — the dependency direction is one-way, ocel2 ->
  cgap-ffi. No other crate consumes cgap-ocel2.
- Real collaborators, Chicago law: the REAL chain, REAL sealed blocks,
  REAL license record, REAL BLAKE3 kernel (path deps only).

## Court surface (13 tests)

- `src/conformance_court.rs` (6): drives the REAL 4x5 AoQ session
  machine, taps only accepted steps into a `TapSink`, replays the
  sidecar chain through the tap oracle; includes the anti-vacuity flip
  (`impossible_row_flips_the_court`).
- `tests/tamper_court.rs` (5): chain tamper legs — flipped block byte,
  mutated receipt field, post-hoc license change each collapse to the
  typed `Ocel2Refusal::Spoliation` refusal; NO tables on refusal.
- `tests/roundtrip_court.rs` (1): full projection round trip (chain ->
  Arrow batches -> re-read) asserts the tables carry exactly the
  chain's links.
- `view.rs` `courts` module (1): the DDL court — every statement
  present, flat view joins all three tables, "NOT a wall clock" note
  is load-bearing; anti-vacuity via the full_ddl joint-count check.
