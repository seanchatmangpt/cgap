# DoD audit — cgap-parquet + cgap-arrow (EXP-06, 2026-11-02)

Auditor: EXP-06 read-only lane. Repo /Users/sac/cgap, branch main.
Receipt of record: docs/jira/v26.11.2/audit/dod-cgap-parquet-arrow.md
(this file). No writes other than this receipt. Note: /tmp/exp06-target
could NOT be deleted — `rm -rf /tmp/exp06-target` was denied by the
permission system. Cleanup owed by operator.

## T1 — Zero-copy: coordinate reads are mmap windows + slice views (PASS)

cgap-parquet scan side is genuinely zero-copy on the read path:

- The mapping itself: `Mmap::as_slice` returns
  `core::slice::from_raw_parts(self.ptr, self.len)` over the PROT_READ
  MAP_PRIVATE mapping — no copy, no heap (scan.rs:107-109; mapping
  created by the two raw `mmap` syscalls at scan.rs:99-103, zero deps).
- Whole-column payload: `column_bytes` is a bounds-checked `.get()` of a
  window of that map slice (scan.rs:845-850) — a borrowed view, zero
  bytes moved.
- Scalar coordinate reads: `value_u32` (scan.rs:857-873) and
  `value_u64` (scan.rs:875-890) do checked offset math then
  `u32::from_ne_bytes(b.try_into()...)` on a 4/8-byte `.get()` window of
  the map — no allocation, no serialization; per-row `checked_mul` and
  column-end bounds give typed `ScanError::OffsetMismatch` refusals
  instead of neighbor-column reads.
- FLBA coordinates: `value_flba` aliases the mapping
  (`&[u8]`, scan.rs:899-928) — zero copy by signature.

cgap-arrow read surfaces are slice views, not copies:

- `Buffer::as_slice` / `as_typed<T>` are
  `slice::from_raw_parts(_mut)` views over the owned allocation, with
  alignment checked before the cast (buffer.rs:83-112).
- `slice::slice()` returns a `Sliced<'a, T>` borrow {parent, offset,
  len} — no allocation; `Sliced::get` indexes parent values at
  `offset + i` with parent-bit validity (slice.rs:82-118,
  129-137).
- cgap-arrow `ipc.rs` is the one caveat: `read_sealed`'s body path
  `copy_from_slice(&buf[body_at..body_at + body_len])` into a
  caller-provided `body_out` (ipc.rs:362) is a copy into caller-owned
  memory, and the module is an in-house 64-byte MetaHeader envelope —
  NOT Apache Arrow IPC wire format (see T2).

VERDICT: reading coordinate vectors does not trigger serialization or
heap realloc. Mechanism = raw mmap + from_raw_parts windows +
borrowed `Sliced` views; the only copies are caller-directed
(`ipc.rs` body copy, `decode_page_into` into caller buffers) and the
footer stats helper `stats_alloc` which openly builds an owning Vec
(scan.rs:1032-1044) — non-hot-path, documented.

## T2 — Format compliance: real pyarrow court exists and PASSES in-tree

Strongest courts found and run (CARGO_TARGET_DIR=/tmp/exp06-target,
2026-11-02):

- cgap-parquet, --features std: EXIT 0. Per-binary counts:
  lib 195 passed / 1 ignored (63.25s; the 1 ignored is a
  long-running encode_fast differential parity variant), then
  integration binaries 1 + 4 (aligned_interop) + 4 + 4 + 17
  (mutation_kills) + 3 + 2 + 8 + 3 + 3 + 4 + 2 + 0 = 59 more,
  total 254 passed, 0 failed, 1 ignored.
- cgap-arrow, --all-features: EXIT 0. 139 + 3 + 9 + 1 + 36 + 1 + 4
  + 6 + 0 = 199 passed, 0 failed.
- cgap-arrow, default: EXIT 0. 136 + 3 + 9 + 1 + 36 + 1 + 4 + 6 + 0
  = 196 passed, 0 failed (3 filtered under default = nested-types
  gates).
- aligned_interop, --nocapture, real subprocess:
  `PYARROW-RECEIPT pyarrow=23.0.1 rows=256 type=fixed_size_binary[1]
  fidelity=EXACT` plus the anti-vacuity leg `fidelity=MISMATCH`
  (corrupted page correctly rejected by pyarrow). 4 passed, exit 0.
  This is a REAL external-reader court: hand-assembled thrift-compact
  container read back byte-exact by non-cgap pyarrow
  (tests/aligned_interop.rs:7-20, 162-236); missing pyarrow degrades
  to a LOUD skip, never a silent pass.
- Envelope court: cgap-arrow tests/ipc_courts.rs exercises the
  in-house MetaHeader envelope (seal/read, digest, continuation-byte
  corruption at ipc_courts.rs:192). It courts the in-house format
  only — it is NOT validation against a real Arrow IPC reader; the
  real-reader claim for Arrow side rests on nothing in-tree.

No external-tool receipt (pyarrow version, exit) is pinned anywhere in
docs/jira/v26.11.2 — the pyarrow receipt exists only as test stdout;
`grep -rln pyarrow docs/jira/v26.11.2` is empty. Prior in-tree census
(waves/crate-reality/cgap-parquet.md:17,74: aligned_interop 4/0/0,
cargo test exit 0, 222/0/1) matches this run's shape.

VERDICT: Parquet side = FORMAT-COMPLIANT (real pyarrow 23.0.1 EXACT
receipt, live in-tree, anti-vacuity leg present). Arrow side =
ENVELOPE-ONLY: `ipc.rs` implements cgap's own 64-byte sealed envelope,
not Apache Arrow IPC; no real Arrow-IPC reader court exists in-tree.
Any "Arrow IPC compatible" claim would be UNSUPPORTED; the in-tree
claims checked say "cgap Arrow envelope", which is honest.

## T3 — Dependency audit: zero-dep holds for both crates (PASS)

- crates/cgap-parquet/Cargo.toml `[dependencies]` section is EMPTY;
  only dev-dep is workspace-path `cgap-arrow` (Cargo.toml
  [dev-dependencies] block, path = "../cgap-arrow").
- crates/cgap-arrow/Cargo.toml `[dependencies]` EMPTY; dev-deps are
  workspace-path cgap-core, cgap-gate, cgap-storage.
- Cargo.lock confirms no third-party runtime leakage: workspace
  `cgap-parquet 26.11.2` deps = ["cgap-arrow 26.11.2"] only;
  `cgap-arrow 26.11.2` deps = ["cgap-core", "cgap-gate",
  "cgap-storage"] only — all in-workspace, no external names.
- no_std viability: cgap-arrow compiles + 196 tests pass with default
  features (no std) — VIABLE as tested. cgap-parquet lib is no_std
  (scan.rs raw mmap syscalls + alloc-free paths), but see T4: its
  default-feature TEST target does not compile. Lib-only no_std holds
  for cgap-parquet; the test-suite no_std claim does not.

VERDICT: E=0 external runtime deps confirmed for both crates at
Cargo.toml and Cargo.lock level.

## T4 — OPEN DEFECT (confirmed, reproduced): default-feature
`cargo test -p cgap-parquet` exits 101

Reproduced exactly as alleged:

- `CARGO_TARGET_DIR=/tmp/exp06-target cargo test -p cgap-parquet`
  (default features, no std): EXIT 101. 18 compile errors:
  `cannot find type RingSink` (x7), `cannot find trait SpillSync`,
  `cannot find module or crate private`, `cannot find module or crate
  std` (x11) — all in crates/cgap-parquet/src/mmap_writer.rs, at
  1821, 1823, 1825-1827, 1841, 1844-1845, 1852, 1896-1899, 1902-1903,
  1908, 1912, 1914.
- Same command with `--features std`: EXIT 0, 254 passed / 0 failed /
  1 ignored.

Root cause (verified by inspection): production items are correctly
std-gated — `RingSink` at mmap_writer.rs:721
(`#[cfg(feature = "std")]` line 720), `private::Sealed` at
mmap_writer.rs:746 (gate 743-751), `SpillSync` at 752-757, the
`BlockSink` impl at 860-861. But the test module at mmap_writer.rs:899
is gated `#[cfg(test)]` ONLY — not `#[cfg(all(test,
feature = "std"))]` — and its body (1821-1914: `impl private::Sealed
for SharedRec` at 1823, `RingSink::with_dyn` at 1844, `RingSink::new`
at 1902) references the std-only items. Under default features +
cfg(test), the referenced items vanish => 18 E0432/E0433/E0405/E0425
errors => exit 101.

Fix shape (one line): change mmap_writer.rs:899 from `#[cfg(test)]`
to `#[cfg(all(test, feature = "std"))]` (or gate the two File-based
test fns individually).

VERDICT: OPEN defect CONFIRMED. Default-feature test target of
cgap-parquet does not compile; all cgap-parquet test claims are only
valid under `--features std`.

## Court manifest (exact commands, exits, counts)

1. cargo test -p cgap-parquet --features std -> EXIT 0
   (254 passed, 0 failed, 1 ignored; lib 195 in 63.25s)
2. cargo test -p cgap-parquet (default) -> EXIT 101
   (18 compile errors, mmap_writer.rs test module, see T4)
3. cargo test -p cgap-arrow --all-features -> EXIT 0
   (199 passed, 0 failed)
4. cargo test -p cgap-arrow (default) -> EXIT 0
   (196 passed, 0 failed; nested-types tests filtered)
5. cargo test -p cgap-parquet --features std --test aligned_interop
   -- --nocapture -> EXIT 0, 4 passed,
   PYARROW-RECEIPT pyarrow=23.0.1 fidelity=EXACT + MISMATCH legs.

All runs used CARGO_TARGET_DIR=/tmp/exp06-target; deletion denied by
permissions (see header).
