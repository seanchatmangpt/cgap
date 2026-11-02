# Lane INL receipt — inline-site census extension (FA6 F6-INL closure)

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. Lane owner file:
`crates/cgap-branchless/tests/branchless_census.rs` (only file written).

## FA6 gap

`#[inline]`/`#[inline(always)]` primitives were censused only as isolated
symbols; branches at inline CALL SITES inside real callers were invisible
to the census. This lane closes the gap with canonical CALLER probes.

## Caller inventory (recorded in census output)

| caller | primitives inlined | cell class |
|---|---|---|
| `caller_filter_lane` | cmov_u64 + pred_inc, 8-lane | HOT (exact 0 branches, both targets) |
| `caller_swar_scan` | swar_eq_u64 + swar_splat, 8-lane | HOT (exact 0 branches, both targets) |
| `caller_eytzinger_probe` | eytzinger_next in-loop + eytzinger_restore tail | documented caller-own residual |

Caller inventory recorded via `[INL-CALLER-INVENTORY]` line per run and
`CALLER_GRID` table (new callers get new rows).

## Cell verdicts (measured this session)

- `caller_filter_lane` host-aarch64: 0 branches; riscv32imc: 0 branches.
  The 8-lane loop unrolls bounds-check-free over `const LANE: usize = 8`,
  so the caller's own documented loop back-edge budget is 0 and the
  inlined primitive bodies contribute 0 branches beyond it.
- `caller_swar_scan` host-aarch64: 0 branches; riscv32imc: 0 branches.
- `caller_eytzinger_probe` host-aarch64 4/4, riscv32imc 9/9 — caller-OWN
  structural branches only (loop compare, bounds-check branch, out-of-bounds
  panic call, plus rv32 eytzinger_restore shift-amount guard). NOT
  primitive branches: eytzinger_next is inline(always) and 0-branch
  isolated; the residual shape matches the isolated-symbol table.
- Existing 16/16 grid unchanged: "census host-aarch64: 10 HOT cells
  0-branch, 3 RESIDUAL cells, refs labeled" (same counts as before).

## Anti-vacuity: M-INL-1

Injection: `if pred { core::hint::black_box(x); }` into the
caller_filter_lane loop body of a /tmp sandbox probe copy (shared tree
untouched; probe source is test-generated, nothing to restore).
Measured flip: host-aarch64 0 -> 15 branches (tbz/tbnz per lane; black_box
is opaque so LLVM cannot select-lower the injected branch), riscv32imc
0 -> 16 (beqz/bnez per lane + j). A caller court that stays 0 under the
mutant panics VACUOUS. Runs in all three test fns (host, riscv, indep).

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/inl-target cargo test -p cgap-branchless` — exit 0
  (32 unit/court tests + 3 census tests, 0 failed).
- `CARGO_TARGET_DIR=/tmp/inl-target cargo test -p cgap-branchless --test
  branchless_census -- --nocapture` — exit 0; caller-cell and M-INL-1
  flip witnesses quoted above.

## Design notes

- Caller probe compiles WITHOUT `--inline-threshold=0` (isolated-symbol
  census keeps threshold=0 on purpose); `#[inline]` hints must fire at
  the call site for the caller census to be meaningful.
- Fixed-width LANE=8 lanes make the loop unroll bounds-check-free, which
  pins the caller's own branch budget to 0 and makes the 0-branch HOT
  cell exact rather than allowlisted. A rolled dynamic-slice filter
  caller was measured first: its baseline (6 host / 2 rv32) was
  loop-shape dependent (autovectorization preamble), so it was rejected
  as an unstable exact cell.
- Residuals are toolchain-pinned (same discipline as the existing
  residual table): a toolchain bump that shifts the caller-eytzinger
  probe counts will fail loudly and be re-witnessed.

## Standing

ALIVE for the caller census extension (commands + exits above, this
session). Divergences: none from the lane doc; the "loop back-edge
budget" for the HOT caller cells is 0 because the representative loops
unroll — recorded here per the doc's "documented loop back-edge" wording.
