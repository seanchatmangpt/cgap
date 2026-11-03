# P-09 — Impl-Legs Certification: B-Now Court + A-Deferred qemu (PLAN)

PLAN (session-delivered, transcribed 2026-11-02; not yet executed).

Finder: `redteam/red-11-riscv-reentrancy-corruption.md:93`
(implcell RMW path UNVERIFIED); partial state per
`lanes/lane-rc02-ref-store-width-arms.md:25-29` ("behavioral RMW
coverage of implcell remains open").

## 1. Leg C — ALREADY LANDED

- Typed refusal + opt-in for the implcell surface landed this
  cycle (w2/w5 legs of the ref-store width-gated atomics fix,
  commit a0431b70 lineage). Nothing to do here; recorded so the
  three-leg register reads complete.

## 2. Leg B-now — host-side sandbox-forced implcell harness

- Host test harness that FORCES the implcell (16-bit mirror)
  arms to compile and run under host cfg — i.e. build
  `crates/cgap-git/src/ref_store.rs` with the width cfg ladder
  (ref_store.rs:140-166: `target_has_atomic` 64 -> 32 -> 16
  fallback chain) pinned to the 16-bit arm via
  `--cfg target_has_atomic="16"`-style override in a /tmp
  sandbox copy, exercising the mirror types on the host where
  they are observable.
- Contract court h1-h5 (CONTRACT, not behavioral — no claim of
  on-silicon semantics):
  - h1: width contract `const _` assertions hold under mirror
    types.
  - h2: get/set sequencing obeys the width gate.
  - h3: no tearing under the mirror read-modify-write model.
  - h4: no silent clamp at width boundaries.
  - h5: mirror and impl64 agree on all defined-behavior inputs
    via a shared vector table.
- Mutations (each must flip the court; /tmp sandbox, sha256
  restore + mtime bump):
  - M-B1: break the width const-assert in the sandbox copy ->
    court FAILs (assertion leg is live).
  - M-B2: swap a mirror op for the naive RMW -> h3/h5 FAILs.
- HEADER LAW for the receipt: "contract witnessed, silicon
  UNVERIFIED". The court proves the CONTRACT; it makes no
  behavioral claim about real riscv32imc silicon.

## 3. Leg A-deferred — qemu-system-riscv32 bare-metal court

- Behavioral registration per 01-release-gates.md:159-179: a
  qemu-system-riscv32 bare-metal (board + kernel) run reading
  the SRAM high-water mark; exit 2 = subject missing (gate exit
  law). Courts q1-q3 (reentrancy-under-interrupt, RMW tearing
  census, high-water regression) + mutation M-A1 (inject a torn
  RMW in the kernel image -> q-court must flag).
- BLOCKED on firmware port: the board/kernel pipeline that
  binds the REAL critical section does not exist this session.
  Loud BLOCKED, with the h-law header keeping v26.11.2 honest.

## 4. Blast radius

- B-now: new court file under `crates/cgap-git/tests/` (or a
  sandbox-forced harness crate); ref_store is fresh off
  a0431b70 — the court must not perturb the shipped gate (run
  the ref_store test family pre/post).
- A-deferred: harness script + gate wiring near
  01-release-gates.md §159-179.

## 5. Standing

- Leg C: ALREADY LANDED (commit a0431b70 lineage).
- Leg B-now: UNVERIFIED (plan transcribed, not executed).
- Leg A-deferred: BLOCKED (no firmware port binding the real
  critical section this session).
- Cites re-read on disk 2026-11-02: ref_store.rs:140-166 cfg
  ladder, :722-724 `target_os = "none"` gate.
