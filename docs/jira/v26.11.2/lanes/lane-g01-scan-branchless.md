# Lane G01 — scan.rs branchless re-lower (v26.11.2)

Standing: **ALIVE** (all commands + exits run this session, 2026-11-02)

## Subject
`crates/cgap-branchless/src/scan.rs` — sole file written in-tree.
Receipt: `docs/jira/v26.11.2/lanes/lane-g01-scan-branchless.md` (this file).

## Diff
1. `prefix_max_u8x8`: `u8::max` chain (lowered to bgeu/bltu cascade on
   riscv32imc, no cmov) -> SWAR parallel-prefix: 8 lanes packed into one
   u64, three fully-unrolled Hillis-Steele steps (d=1,2,4) select
   `max(v[i], v[i-d])` through a per-byte `swar_lt_harvest` mask
   (harvest-cite of `swar.rs::swar_lt`, swar.rs:24-39; scan.rs must stay
   self-contained because the scan_courts census compiles it standalone
   via `#[path]` wrapper — witnessed E0433 otherwise) plus
   `(lt >> 7) * 0xFF` lane expansion. Now `const fn`. Promoted from
   non-const — capability addition, callers unaffected (0 cross-crate
   consumers, grep witnessed).
2. `count_leading_eq_u8`: `mask.trailing_ones()` (beqz-guarded ctz libcall
   on RV32IMC, no Zbb) -> pure-arithmetic run length: scalar equality
   bitmask (xor+seqz lowering unchanged from BEFORE) + lowest-zero-bit
   isolation `z = (~m & (m+1)) & 0xFF` + `popcount8(z-1)` (SWAR popcount,
   wrapping ops throughout — release-identical, debug panic-free).
3. `segmented_prefix_sum_u32x8`: dead `let mut acc: u32 = 0;` initializer
   (scan.rs:122) folded into `let mut acc: u32 = values[0]; out[0] = acc;`
   — the unused-assignment warning is gone from cargo test output.

## Court: AE4 disassembly-invariants (scripts/verify_disassembly_invariants.py)
NOTE: the court script CHANGED MID-SESSION (added arm E, G03's
byte-level binary mutant arm). Divergence recorded; this receipt uses
the on-disk version at use time.

- BEFORE (sandbox replay: /tmp/g01-baseline copy of the crate with the
  pre-edit scan.rs reconstructed from the session's first Read — scan.rs
  is untracked, no HEAD version exists):
  `CGAP_ROOT=/tmp/g01-baseline python3 scripts/verify_disassembly_invariants.py --skip-binmut`
  -> **exit 1**: [FORBIDDEN] count_leading_eq_u8 (beqz),
  prefix_max_u8x8 (bgeu, bltu).
- AFTER (real tree):
  `python3 scripts/verify_disassembly_invariants.py --skip-binmut`
  -> **exit 0**, verdict line: "AE4 ... CLEAN (census + linked witness +
  mutant + self-check + binary mutant all held)"; count_leading_eq_u8 and
  prefix_max_u8x8 both "[PASS] 0 forbidden branches". Anti-vacuity arms
  C/D PASS (M-AE4-1 if-injection flips; branchy naive_if_cmov flagged).
- Full court (arm E included) = exit 3: "[SKIP] arm E binary-mutant court
  did not run (loud)" — G03's arm found no patchable candidate in the
  witness archive. Loud skip per ABI, pre-existing/environmental, not
  caused by this diff (reproduced before/after irrelevant: the arm
  inspects only the witness archive, not scan.rs).

## Instruction-level flip (riscv32imc, --emit=asm opt-level=3)
BEFORE prefix_max_u8x8: bgeu x5 + bltu x1 cascade with store-side-effect
unrolling (sb into a1, the in-place swap shape).
AFTER prefix_max_u8x8: lbu/lui/addi/slli/or/and/sltu/sub/not/xor/srli/sb
— 0 branch mnemonics, `ret`.
BEFORE count_leading_eq_u8: xor/seqz mask build + beqz-guarded ctz tail.
AFTER count_leading_eq_u8: xor/seqz mask build + not/addi/and/addi/
srli/and/sub/andi + ret — 0 branch mnemonics. Full dumps in the
session transcript; also reproducible via the emit-asm build above.
Target ISel: RV32IMC (C extension, no Zbb/Zicond) per the court ABI.

## Test fence (must-not-regress)
`cd crates/cgap-branchless && cargo test` — exit 0, all suites:
34 lib + 3 branchless_census + 8 scan_courts + 7 transpose_courts
(+0 doctests). Includes census_scan_hot_cells_zero_branch (host
aarch64, 0 flow mnemonics per HOT cell, incl. count_leading_eq_u8),
court_count_leading_eq_u8 (exhaustive 256 masks x 4 targets),
court_prefix_max_u8x8 (4096 corpus draws vs naive oracle + monotonic
edge cases), M-TS2-1 mutation anti-vacuity. scan.rs:122 warning gone.

## Divergences / mid-flight findings
- Court script gained arm E mid-session (G03); first two court runs
  used the pre-arm-E script. Recorded as divergence, not errata (no
  prior receipt contradicted).
- Two intermediate defects caught by existing courts, fixed in-lane:
  (a) first SWAR form used `crate::swar::` paths — broke the census
  standalone-compile (E0433); scan.rs must be self-contained.
  (b) first mask-expansion used wrapping_mul(0x0101..01) = 0x01/lane,
  not 0xFF/lane — caught by court_prefix_max_u8x8 corpus draw.
- transpose_64x64 was FORBIDDEN (bne) in the first (pre-arm-E) run and
  PASS in later runs — NOT touched by this lane (transpose.rs not in
  G01 scope); flaky unrolling or script-version difference; REP1/G03
  should re-census transpose ownership. Noted as cross-lane signal.
- scan.rs and the court script are both untracked in git (no HEAD
  baseline exists); baseline replay was a /tmp sandbox reconstruction,
  sha-relevant only to arm-A census semantics.

## Standing
ALIVE — every claim above carries a command + exit run this session.
Perf: no perf claim made or measured (INTEGRATION FENCE in scan.rs
header still applies; no stream-compaction win claimed).
