# A-5 SRAM stack estimate — working receipt (2026-11-01)

Lane: A-5 (SRAM-STACK-ESTIMATE), repo /Users/sac/cgap, branch main.
Lease: docs/jira/v26.11.1/waves/gate-5-silicon.md (APPEND section
"SRAM stack estimate") + this file. All ESTIMATE labels honored.

## Commands + exits (this session, first-hand)

- `rustup target add riscv32imc-unknown-none-elf` -> "up to date" (exit 0;
  nightly rustlib carries the triple).
- `CARGO_TARGET_DIR=/tmp/a5-target cargo build --release -p cgap-bn3
  --target riscv32imc-unknown-none-elf` -> exit 1, 410 errors, E0463
  "can't find crate for `std`" (unify.rs:98) — cgap-bn3 is a std host
  crate. Witnessed BLOCKED leg; never a silent skip.
- `RUSTFLAGS="--emit=asm -C opt-level=s -C panic=abort -C debuginfo=0"
  CARGO_TARGET_DIR=/tmp/a5-target cargo build --release -p cgap-bn3 -p
  cgap-ffi` -> exit 0; asm subjects:
  /tmp/a5-target/release/build/cgap-bn3/34888793bef92df1/out/cgap_bn3-...s
  /tmp/a5-target/release/build/cgap-ffi/fc77ee42aac2a019/out/cgap_ffi.s
- Frame + call-graph parse (python, on-disk asm bytes): 515 bl-edges,
  per-function frame = max `sub sp, sp, #N` in prologue.
- Deletion: `python3 -c "import shutil; shutil.rmtree('/tmp/a5-target')"`
  -> exit 0 (below).

## Estimate table (all ESTIMATE, never measured)

Deepest chain (aarch64 host bound for rv32): cgap_eval_bn3 688 B ->
ConsequentProjector::project 272 B -> ConsequentProjector::emit_into
2144 B -> sha256_padded 432 B = 3,536 B total, 4 frames, 10.8% of the
32,768 B SRAM budget (margin 29,232 B). Alt legs: compile leg 1,632 B
incl. eval; unify leg 1,712 B incl. eval. Assumptions: panic=abort, no
LTO frame shrink, single exception context, allocator heap excluded.

## Standing

§4 SRAM eval-stack term: UNVERIFIED -> PARTIAL (estimate method stated;
hardware/qemu stack-paint witness remains future). Divergence from
dispatch: the anticipated riscv32imc frame measurement is BLOCKED (E0463,
witnessed); the dispatch's sanctioned fn-size heuristic was applied on
the host aarch64 build as the declared rv32 bound instead.
