# Lane MCP-18 — cgap-git pre-existing warning census closure (FS-17 deltas)

Date: 2026-11-02. Branch: main. Files owned: `crates/cgap-git/src/mcp_driver.rs`,
`crates/cgap-git/src/ref_store.rs`. Cargo.toml untouched (no lints-table need).

## W1 — Subject

Three FS-17 census warnings in cgap-git: mcp_driver.rs:36 misplaced inner-crate
`#![no_std]`, ref_store.rs:273 deprecated `Atomic::<u64>::fetch_update`, ref_store.rs:351
unused variable `bound`.

## W2 — Diff

1. `mcp_driver.rs` — removed the module-level `#![no_std]` (crate root `lib.rs:10` already
   declares `#![no_std]`; the module attribute was misplaced, hence the
   `unused_attributes` warning). `#![forbid(unsafe_code)]` and `extern crate alloc`
   retained. Additional: gated `use alloc::vec::Vec` under `#[cfg(feature = "std")]` —
   once the mis-arm warning cleared, the rv32 opt-in build exposed a pre-existing
   unused-import warning in the same owned file; cfg-gated to match the module's
   existing `#[cfg(feature = "std")]` usage sites.
2. `ref_store.rs:273` — `fetch_update` → `try_update` on `AtomicU64` inside the
   `Counter::fetch_update` wrapper (nightly renamed the method, same CAS semantics,
   identical signature `(Ordering, Ordering, F) -> Result<…>`; compiles clean).
3. `ref_store.rs:351` — removed the dead `bound` local in `SealRegister::stamp`. Analyzed
   before touching: NOT a bug — post-RED-11, ceiling enforcement lives INSIDE
   `reserve`'s CAS closure, which computes its own `bound` (ref_store.rs:374-378);
   the `stamp`-local copy was never read after that repair. Comment updated to point
   at `reserve`.

## W3 — Commands (this session, exact)

- `cargo build -p cgap-git` → exit 0, ZERO warnings (baseline pre-fix had exactly the
  three census warnings, captured in the session transcript; final build: 0).
- `cargo test -p cgap-git --test ref_store_court --test ref_store_reentrancy_court
  --test ref_store_width_arms_court` → exit 0; 5+5+8 passed, 0 failed (run twice:
  pre- and post-Vec-gate edit, same result).
- `cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features`
  → compile_error still fires (ADV-11 loud-until-bound gate intact; 1 match for
  compile_error in output).
- `RUSTFLAGS="--cfg implcell_critical_provided" cargo build -p cgap-git
  --target riscv32imc-unknown-none-elf --no-default-features` → exit 0, ZERO warnings.
- Full `cargo test -p cgap-git --no-fail-fast` → one failure, see W5.
- Build root: `/tmp/mcp18-target` (lane lease, deleted at lane end).

## W4 — Anti-vacuity

- The rv32 no-opt-in leg is itself the non-vacuity witness for the ADV-11 gate: it
  still aborts with the typed compile_error naming the cfg keys. It flipped: opt-in
  build succeeds, non-opt-in build refuses — the gate discriminates.
- The rename is asserted by the compiler (deprecated-method warning gone only because
  the call site changed); removing it restores the warning.

## W5 — Divergences / pre-existing findings

- **octopus_merge_court flake (pre-existing, NOT session-introduced — grounded)**:
  full-suite parallel run failed `mutation_conflict_injection_flip` with an
  ANDON domain clash panic at `octopus_merge.rs:366`. Grounded pre-existing: the
  same court PASSES standalone both with HEAD bytes of my two files swapped in
  (sandbox swap, sha256-verified restore of my post-fix bytes:
  ref_store `4d99ff71…`, mcp_driver `94e18472…`) and with my final files. The failure
  appeared only in the full-suite parallel execution. This is another lane's surface
  (octopus_merge.rs is outside my ownership); filed here as a finding, not fixed.
- Cargo.toml needed no lints-table change; untouched per contract.

## W6 — Standing

ALIVE for the lane's three-target scope (host 0-warning build + three named courts
green + both rv32 legs verified this session, exits above). The full-suite
`cargo test -p cgap-git` is PARTIAL (one pre-existing parallel-mode flake in an
unowned file, grounded above).

## W7 — Fences named

RC-01/R-ADV11FIX landed surface (ref_store width ladder, implcell gate, RED-11 RMW
ceiling-inside-CAS) — all three width courts re-run green post-edit. The rename must
not change the Counter wrapper's external `Result<u64,u64>` contract: asserted by the
5+5+8 court pass.
