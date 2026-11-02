# A3 — RingSink commit-leg receipt (finishing W3-L4)

Date: 2026-11-01. Lane A3, canonical checkout /Users/sac/cgap, no git
commands run. The killed W3-L4 lane landed the lever (RingSink +
BlockSink in `crates/cgap-parquet/src/mmap_writer.rs`, FFI commit-leg
swap to thread-local `COMMIT_LEASE` in `crates/cgap-ffi/src/sovereign.rs:723-794`,
W3-R2 FrameView init) but died before writing
`waves/w3-l4-ring-sink.md`. This receipt witnesses the landed code.

## 1. Tests (command + exit)

- `CARGO_TARGET_DIR=/tmp/a3-target cargo test -p cgap-ffi -p cgap-parquet`
  — exit 0. cgap-ffi: 45+4 unit + lever_courts 2 (both `l4_*`) + 5 + 1
  (1 ignored, pre-existing) + others; cgap-parquet: 192 unit (191 passed,
  1 ignored, pre-existing) + lever_courts 3 + all integration suites
  green. Zero failures.
- `cargo test -p cgap-ffi --test lever_courts -- --nocapture` — exit 0:
  `l4_committed_header_flip_is_exactly_spoliation` (court A) and
  `l4_status_flip_witness_at_lease_edge` (court B) both pass.

## 2. File::create out of the hot path (real code path)

`crates/cgap-ffi/src/sovereign.rs:733-740`: `File::create` now exists
ONLY inside the lease-grant arm (`slot.is_none()`), executed once per
lane slot; steady-state per-tx = zero-copy u32 reborrow →
`StaticPageBuffer::writer().write_slab` → `seal(None)` →
`sealed.flush_to(&mut lease.ring)` (rotation drains to the pre-opened
spill handle). No per-call scratch or lane Vec alloc. Error mapping:
grant failure / write_slab err / seal err / flush_to err →
CGAP_ERR_LEASE_EXHAUSTED (0x43470004), never silent truncation.

## 3. Bench (divan, /tmp/a3-target, `cargo bench -p bench --bench chains-sovereign`, exit 0)

- `stage_commit_parquet` (FFI ring path): 150.7 µs @256 = **0.589 µs/tx**;
  2.472 ms @4096 = **0.603 µs/tx**.
- vs before 45.8 µs/tx → **>=76x; target <4 µs/tx: MET.**
- Boundary probe (witnessed in bench stdout): 65,376 lanes accepted,
  first refusal at 65,377 lanes (0x43470004); bench batch = 16
  lanes/frame, LEASE_EXHAUSTED never fires in-bench.
- Alloc column: NOT literally 0 — 2 allocs / 36 B per tx remain
  (FooterSealer `spans.to_vec()` + `build_meta_block` Vec,
  `crates/cgap-parquet/src/footer.rs:187,326`; pre-existing, outside
  lane ownership). Steady-state commit leg itself allocates nothing.

## 4. Courts (mutation battery)

Courts A and B are on-disk tests, already witnessed passing this
session (exit 0 above); no additional /tmp mutation was required —
court A mutates a committed header inside a temp-dir sink and asserts
EXACTLY 0x5032_38FF (CGAP_ERR_P28_SPOLIATION) both directions; court B
drives a 70k-lane commit past the 65,376 lane ceiling and asserts
exactly CGAP_ERR_LEASE_EXHAUSTED. Anti-vacuity: both assert a specific
typed word, not merely "an error". Parquet lever_courts (3 tests)
green.

## 5. Standing

ALIVE. Verified this session; before/after 45.8 → 0.589-0.603 µs/tx;
target met. Pre-existing: 1 ignored test per crate (unrelated to the
lever); 2 unused-import warnings in cgap-gate/penstock.rs (not this
lane's files).

---

```json
{"lane":"A3","files_written":["docs/jira/v26.11.2/waves/a3-ring-sink.md","docs/jira/v26.11.2/02-pareto-levers.md"],"commands":["cargo test -p cgap-ffi -p cgap-parquet (exit 0)","cargo test -p cgap-ffi --test lever_courts (exit 0)","cargo bench -p bench --bench chains-sovereign (exit 0)"],"courts_run":["l4_committed_header_flip_is_exactly_spoliation","l4_status_flip_witness_at_lease_edge","cgap-parquet lever_courts (3)"],"standing":"ALIVE","divergences":["receipt named a3-ring-sink.md instead of the w3-l4-ring-sink.md the L4 row cited (that file never existed; row amended)","court A/B flips witnessed via on-disk lever_courts tests rather than fresh /tmp mutations — they already mutate temp-dir artifacts in-process","steady-state alloc column is 2 allocs/36 B per tx, not 0 (pre-existing footer.rs allocs, outside lane ownership)"]}
```
