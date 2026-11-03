# Lane XC-04 — M-CTQ consolidation bench (v26.11.2)

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. No git write commands
run.

## Files written (owned)

- `crates/bench/benches/mctq_consolidation.rs` (NEW, harness=false)
- ONE `[[bench]] name = "mctq_consolidation"` entry in
  `crates/bench/Cargo.toml` (dispatch-owned line)
- `docs/jira/v26.11.2/lanes/lane-xc04-mctq-consolidation.md` (this receipt)

## Method of record

csct N=7 Min/Median/Max (03-msa-gauge.md Sec 2; no p99 — max-of-7 is not a
quantile, FIX-04 law). Batch-amortized windows (gauge law: 41 ns timer
floor). FACT_WINDOW = 1,365 (pack_demux.rs:51). Contention guard is the
chains-ingress.rs pattern reproduced in-file (32 MiB walk, 12 ns/64B-line
ceiling, loud stderr marker); both runs were quiet
(1.02 / 0.93 ns/64B-line).

## Commands + exits (this session)

- `cd crates/bench && CARGO_TARGET_DIR=/tmp/xc04-target cargo build
  --release --bench mctq_consolidation` → exit 0 (after two typed compile
  fixes, below)
- `CARGO_TARGET_DIR=/tmp/xc04-target cargo bench -p bench --bench
  mctq_consolidation` run 1 → exit 0
- Same command run 2 → exit 0 (stability witness)

## Station results (runs 1 / 2, N=7 each)

**M-CTQ-04 seal throughput** (b06 path: 80 B lease frames, real
`cgap_blake3::hash`, monotonic chain, batch-1365 amortized):

- run 1: min 3,529,029 median 3,824,869 max 3,846,425 receipts/s
- run 2: min 4,069,561 median 4,130,106 max 4,135,837 receipts/s
- Run 1's median sat near the b06 1M floor; run 2 at b06's 4.3M. Window
  differs from b06 (1,365 vs 1M receipts per run) — coordinator should
  consolidate with that disclosed, not silently.

**M-CTQ-05 FFI transition cost** (measured route: IN-PROCESS extern "C"
entry into the real `cgap_admit_frame` via the cgap-ffi rlib — NOT the
cdylib/cgo route; disclosed divergence from ipc-03's 85 ns membrane floor,
which prices the cgo trampoline this harness does not):

- null-refusal floor arm: 0.38-0.41 ns/call (in-process route; the null
  short-circuit folds hard in-process — this prices kernel-side cost only;
  the cgo membrane floor 85 ns from ipc-03 remains the wire-level floor)
- admit arm over REAL 152 B A04 frames (batch-1365 amortized):
  run 1: min 123.81 median 124.02 max 125.12 ns/frame
  run 2: min 112.48 median 113.37 max 114.44 ns/frame
  (ipc-03 measured 183-187 ns/frame through cgo; the ~70 ns delta is the
  in-process-vs-cgo membrane delta, consistent with ipc-03's own floor
  arithmetic)
- positive control: admit refused typed before the fixture bug was fixed
  (0x43470002 OABI_MISMATCH); after fix, CGAP_OK witnessed.

## Station 3

**M-CTQ-06 mmap zero-copy read** (ipc-04 mechanism in Rust: temp file,
producer RW MAP_SHARED, consumer SECOND independent PROT_READ mapping,
wire header "CGAPSHM1", 1,365 facts at 24 B stride, raw mmap(2)/munmap(2)
externs — no new crate dependency):

- run 1: min 0.15 median 0.18 max 1.56 ns/op
- run 2: min 0.15 median 0.15 max 1.47 ns/op
- Anti-vacuity flip witnessed BOTH runs: corruption written through the
  producer mapping is visible through the consumer mapping (shared-page
  proof), restore verified byte-exact (`MCTQ-06-FLIP` line). A private
  copy would have failed this court.
- Note: run-1 max 1.56 ns/op is first-run page-table warmup on the consumer
  mapping; min/median stable at 0.15 across runs. This prices the L1-hot
  zero-copy loop (32,760 B working set); ipc-04's 4.52 ns/op included the
  Go harness read call. Different instrument, same mechanism — disclose at
  consolidation.

## Anti-vacuity + flips

- Station 1: final-frame digest re-derivation positive control each run
  (b06 chain law); a zero-rate run is a typed FAIL exit 1.
- Station 2: real admit must return CGAP_OK on a REAL fixture frame —
  witnessed REFUSING (0x43470002) when the fixture was wrong, i.e. the
  control flips, before the fix; typed FAIL on non-OK status.
- Station 3: cross-mapping corruption flip + byte-exact restore witnessed
  both runs.
- All-zero / error measurement = typed `[MCTQ-FAIL]` + exit 1 in every
  station. No station faked a pass.

## Divergences

- Dispatch named the station-1 window "1,365"; b06's receipted 4.35M/s used
  a 1M-receipt chain. Kept 1,365 (dispatch law), disclosed for
  consolidation — the two figures are not the same instrument scale.
- Station 2 measured in-process extern "C" (contract's sanctioned fallback,
  documented in-file and above); ipc-03's cgo floor figures stay the
  wire-level authority.
- Raw mmap via extern "C" declarations (no memmap2 dep — [dependencies]
  outside this lane's ownership). FFI symbols are libc's, linked by std.

## Fences (must-not-regress, checked)

- chains-ingress.rs untouched (RC-11 ownership) — contention guard
  REPRODUCED in-file, not imported.
- Root workspace compile: bench crate built green exit 0 both runs; no
  other crate edited. Existing [[bench]] entries untouched; `cargo bench
  -p bench --bench chains-ingress` NOT run (RC-11's surface, not mine) —
  my manifest edit adds one block only, all prior blocks byte-identical.
- Coordinator promotion pass owns 11-measured-claims.md; no numbers written
  there by this lane (per contract).

## Standing

ALIVE — `cargo bench -p bench --bench mctq_consolidation` exit 0 twice this
session, all three stations measured with positive controls PASS and the
station-3 flip witnessed. /tmp build roots (`/tmp/xc04-target`,
`/tmp/xc04-probe-target`) deleted at lane end.
