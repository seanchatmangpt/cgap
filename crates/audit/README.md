# cgap-audit

**Status: NOT-PUBLISHED** — verified against crates.io this session
(no crate record). Workspace-local audit substrate (note the directory is
`crates/audit/`, the package name is `cgap-audit`).

## What it is

Lane A15: the audit substrate carrying three collaborators, not mocks:

- `src/alloc_counter.rs` — `CountingAlloc`, a real `GlobalAlloc`
  counting allocator; the zero-alloc courts' measuring instrument.
- `src/powl.rs` — POWL (predicate/proof-over-log) surface, including the
  Wave-9 cross-lane integration: the A20 stress at the POWL surface,
  which exposed and killed a real O(n^2) binding projection (D8: the
  projection restarted from row 0 per call).
- `src/ocel2_slab.rs` — OCEL² slabs (object-centric event log storage).

## Test receipt (run this session)

```
cargo test -p cgap-audit   # exit 0
  37 passed; 0 failed
```

Receipt of record: `docs/jira/v26.11.1/waves/a15-finish-receipt.md`
(includes the `--features ocel2` disclosure and the
`cgap-gate`/`cgap-storage` dependency closure from the 9be9109d landing).

## The 1M-row stress

`powl.rs::a20_stress_one_million_rows_powl_verify` runs the REAL public
`verify()` over 1,000,000 event rows and asserts final state — it is the
falsifier that killed the O(n^2) projection (a flat-projection mutation
must flip it). CountingAlloc brackets every zero-alloc court with a
positive control: a deliberate allocation MUST move the counter, or the
court is vacuous.
