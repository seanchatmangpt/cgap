# Lane R-03 — cgap-git fence (tollgate wave)

Read-only fence lane. No source edits, no git commands. Wrote only this
receipt.

## W1 Subject

Full-crate `cargo test -p cgap-git` fence over the landed FIX-04/FIX-05/
RC-01/RC-02/RC-12/RC-14 surfaces, plus the riscv32imc no-default-features
build re-witness.

## W2 Commands / exits (this session, /tmp/r03-target lease)

- `cargo test -p cgap-git` (run 1) -> exit 101 — SINGLE anomaly:
  `perf_baseline` binary "could not execute process ... No such file or
  directory (never executed)" at its OUT_DIR path. Zero compile errors in
  the log; no test asserted failure.
- `cargo test -p cgap-git --test perf_baseline` (isolation) -> exit 0
  (0 passed / 1 ignored — the test is `#[ignore]`-gated; binary builds
  and runs).
- `cargo test -p cgap-git` (run 2, rerun after isolation) -> **exit 0**,
  38 `test result: ok` lines (matches RC-14's 38-binary witness exactly),
  1 interleaved `test result: FAILED` line.
- `cargo build -p cgap-git --target riscv32imc-unknown-none-elf
  --no-default-features` -> exit 0.

## W3 Required green set (verified in run-2 log /tmp/r03-full-test2.log)

- ref_store_court 5/5 ok
- ref_store_reentrancy_court 5/5 ok
- ref_store_width_arms_court 5/5 ok
- pack_demux_poison_court 3/3 ok
- posh_alloc_sentry_court 1/1 ok
- quota_alloc_purity_court 1/1 ok
- andon_liveness_court ok (incl. penstock_sieve_court test)
- pack_demux_court 11 passed / 1 ignored ok
- posh_refusal_matrix_court 4/4, posh_wire_court 2/2,
  soc_full_chip_court 3/3, e2e wire courts ok

## W4 Classification of the run-1 anomaly

`perf_baseline` never-executed / file-missing: the OUT_DIR test binary was
absent at execute time despite zero compile errors; the immediate isolated
rerun built and ran it (exit 0) and the full-suite rerun passed it (exit 0
overall). Classified TRANSIENT (binary materialization race at its unusual
OUT_DIR executable path), not a test failure and not a source defect. No
owner; no fix applied (read-only lane). If it recurs, next fence lane
should investigate cargo's OUT_DIR executable placement.

## W5 Known cosmetic (per RC-14 receipt W6-F2, re-witnessed)

The single `test result: FAILED` line at
/tmp/r03-full-test2.log:1501 is the octopus_merge_court expected-fail
child subprocess printing a FAILED block while the parent binary passes
(5 ok). Pre-existing log artifact, confirmed non-blocking; suite exit 0.

## W6 Fences

- Full `cargo test -p cgap-git`: exit 0 this session (38 ok binaries).
- riscv32imc `--no-default-features`: exit 0 (RC-02's divergence note
  about a transient E0599 mid-edit race: not reproduced, clean build).
- No mutations run (fence lane; mutation flips are owned by the landing
  lanes' receipts).

## W7 Standing

**ALIVE** for the full cgap-git fence and the riscv no-default-features
build: commands + exits above, run this session against the exact shared
tree. Zero source edits; lease /tmp/r03-target deleted at lane end.
