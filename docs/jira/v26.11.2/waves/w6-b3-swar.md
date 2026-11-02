# w6-b3-swar receipt

## Standing

**ALIVE** (scoped to `swar`; foreign-lane failures disclosed below).

## Files written

- `crates/cgap-branchless/src/swar.rs` (182 lines: kernels + cfg(test) naive
  oracles + differential/edge/KAT courts) — the ONLY source file owned/written.
- `docs/jira/v26.11/...` typo guard: this receipt only.

## Kernels + conventions

- `swar_eq_u64(word,target)` — 0xFF per equal byte, 0x00 else. XOR then
  exact borrow-free nonzero-per-byte detect
  `((x & !HI) + ~HI) | x` (& HI), expanded `((nz ^ HI) >> 7) * 0xFF`.
- `swar_has_zero_byte(v)` — `((v & !HI) + ~HI) | v) & HI != HI`.
- `swar_range_mask(val,min,max)` — 0xFF where min <= byte <= max
  inclusive; empty when min > max. Built on internal `swar_lt`
  (per-byte unsigned <, 0x80 per lane), verified 500k+ random pairs and
  byte-edge sweep during design (python /tmp/lt_check.py lineage).
- `swar_splat`, `swar_lt` (pub) helpers.

### Divergence from dispatch text

Dispatch named the "classic `(v - 0x0101..) & !v & 0x8080..`" detect for
`swar_has_zero_byte`. That form FALSE-POSITIVES when a 0x01 byte borrows
into the lane above (witnessed: corpus pair
x=0x46b4ba7a9954746a y=0x4927bb7a2ad6df14 — XOR byte 0x01 flipped a
neighbor lane). Naive oracle kept as contract; implemented the exact
borrow-free form instead. Recorded as an honest divergence, oracle rules.

## Courts (anti-vacuity)

- `swar_differential_property`: 50k splitmix64-chain pairs (corpus seed
  0xA20C_0FFE_E5A7_E000; 0xBF58_476D_1CE4_E5B9 used only as the
  SplitMix64 derive constant) diffed against naive per-byte oracles.
  Mutations witnessed flipping it this session: (a) classic
  subtract-detect → borrow false-positive FAIL; (b) missing `& HI`
  lane mask → FAIL; (c) known-answer target typo → FAIL. The court is
  not vacuous; three distinct mutations each red'd it before green.
- `swar_byte_edge_exhaustive`: uniform {0x00,0x7F,0x80,0xFF} per byte
  lane sweep, all 16 splat pairs, incl. range boundary 0x7F/0x80.
- `swar_known_answer_edges`: incl. min>max emptiness.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/b3-target cargo test -p cgap-branchless swar`
  → 3 passed / 0 failed, exit 0 (final run).
- Full crate `cargo test -p cgap-branchless` → 21 passed / 5 FAILED —
  the 5 are `arithmetic::tests` (B2) and `eytzinger::tests` (B4),
  foreign lanes' in-flight files, not swar. Scoped gate is the
  dispatch gate and is green.
- asm court: `cargo rustc --release -- --emit asm=/tmp/b3/swar.s`
  over /tmp/b3-asm probe crate wrapping the four swar kernels
  (b3_eq/b3_zero/b3_range/b3_lt) → 68-line aarch64 listing, grep
  `\b(b|bl|br|blr|cbz|cbnz|tbz|tbnz)\b` = **0 branch instructions**;
  straight-line with `ret` epilogues. Recorded honestly: probe is
  release codegen on this host (aarch64, darwin), not a bit-by-bit
  budget gate.
- Earlier failing runs (mutation witnesses) same command, exit 1.

## SLA disclosures (disclose-in-both)

- Minimal unblock to `crates/cgap-branchless/src/eytzinger.rs`
  (B4's file): added missing `use std::vec::Vec;` under the existing
  `extern crate std;`. B4 landed the same import concurrently →
  duplicate; I removed MY line, B4's import is the surviving one.
  Net diff to eytzinger.rs from me: zero lines (disclosing because the
  transient edit touched B4's file).
