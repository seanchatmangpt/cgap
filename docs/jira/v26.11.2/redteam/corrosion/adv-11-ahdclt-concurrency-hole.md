# ADV-11 — AHDCLT moat collapse & concurrency residual exposure (corrosion wave)

- Lane: ADV-11, READ-ONLY over `crates/`; sole file written: this receipt.
- Subject: `crates/cgap-git/src/ref_store.rs` (seal_atomic cfg ladder, implcell
  `critical` seam, `reserve` RMW), PFMEA `docs/jira/v26.11.2/dflss/08-ax-pfmea.md`
  (PY-5 row), `docs/jira/v26.10.31/16-ahdclt.md` (surviving moat), receipts
  `lanes/lane-rc01-ref-store-reentrancy.md` and `lanes/lane-rc02-ref-store-width-arms.md`.
- Standing: **ALIVE** (every command below ran this session with the exit shown).
- Verdict: **SPOILER-CONFIRMED, elevated** — the registered residual ("impl16/
  implcell compile-witnessed only, not execution-certified") is no longer dormant:
  a nightly-toolchain slide has silently MOVED riscv32imc onto the implcell arm.
  The fleet's only no_std target now executes the never-court-executed,
  unsynchronized Cell RMW behind a pass-through `critical`, while PFMEA PY-5
  still asserts `riscv32imc=impl32`.

## W1 Attack 1 — is `implcell::critical` a real fence or a silent no-op?

**Silent no-op at execution, loud in prose. SPOILER-CONFIRMED.**
`crates/cgap-git/src/ref_store.rs:177-180`:

```rust
pub fn critical<R>(f: impl FnOnce() -> R) -> R {
    f()
}
```

The doc comment (:163-176) states firmware MUST bind a real enter/exit-critical
body and that running unbound with interrupts enabled is "a REENTRANCY VIOLATION
by contract, not a silent one" — but nothing enforces it. There is no
`compile_error` for the unbound state, no trait/const-eval hook firmware must
satisfy, no loud failure of any kind. The unbound fallback compiles, runs, and is
unsynchronized. Exactly the false-confidence shape the dispatch named: the seam
provides a typed *document*, not a typed *contract*. The only loudness available
is a human reading the comment.

## W2 Attack 2 — the rv32imc object-level probe, and what it found

Commands/exits (all this session):

| # | cmd | exit |
|---|---|---|
| 1 | `CARGO_TARGET_DIR=/tmp/adv11-target cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 0 |
| 2 | `/opt/homebrew/opt/llvm/bin/llvm-objdump -d <rlib>` -> /tmp/adv11-disasm.txt (131,012 lines) | 0 |
| 3 | `grep -c "amo\.\|lr\.w\|sc\.w" /tmp/adv11-disasm.txt` | 0 matches |
| 4 | disasm symbol sweep: `seal_atomic::implcell::critical` instantiations for BOTH `stamp` and `reserve` closures | present |
| 5 | compile_error probe: `#[cfg(target_has_atomic = "32")] pub const OK: () = ();` + `#[cfg(not(...))] compile_error!(...)`, built `--target riscv32imc-unknown-none-elf` | compile_error FIRES: "LEGACY target_has_atomic=32 ABSENT on riscv32imc" |
| 6 | same probe shape for `target_has_atomic = "64"` on host (aarch64) | 0 grep hits = legacy key PRESENT |
| 7 | `cargo test -p cgap-git --test ref_store_reentrancy_court` (host fence) | 0 (5 passed) |
| 8 | impl32-width assert probe (`SEAL_COUNTER_WIDTH == u32::MAX`) on rv32imc | 0 — non-discriminating (usize::MAX == u32::MAX on rv32; red-11 §1 noted this) |

**Finding: the cfg ladder silently degraded.** The pinned toolchain is a
FLOATING nightly (rust-toolchain.toml: "channel = nightly", 2026-10-31
posture comment); current rustc 1.101.0-nightly (12433c307, 2026-10-24) has
REMOVED the legacy `target_has_atomic` cfg key (only
`target_has_atomic_load_store` remains — probe 5, corroborated by
`rustc --print cfg`). Consequences:

- Host (aarch64): legacy key present -> impl64 fires (courts green, cmd 7).
- riscv32imc: legacy key absent -> EVERY ladder arm condition
  (`target_has_atomic = "N"`) is false; the implcell arm's `not(...)`
  chain is vacuously true -> **implcell fires on riscv32imc**.
- The RMW on rv32imc is therefore a plain `Cell::get`/`Cell::set`
  read-modify-write wrapped in pass-through `critical(|| ...)` — there is
  NO compare-branch-retry loop in the rv32imc object code at all. The
  dispatch asked me to confirm "the RMW goes through the fetch_update
  guard loop": on the current toolchain it does NOT. red-11's impl32
  `__atomic_*` libcall codegen is gone not because atomics arrived but
  because the target silently moved to the weakest arm.
- The dispatch's disclosure question ("fetch_update != interrupt-safe,
  only correctness-sequential"): the honest answer is the source-level
  guarantee ("indivisible check-and-increment", ref_store.rs:313-318) is
  **not delivered on rv32imc** — indivisibility requires either a real
  CAS retry loop (absent: no AMO instructions, no __atomic libcall) or a
  bound critical section (absent: `critical = f()`). red-11 §2's breach
  shape (torn RMW under preemption) is back on the target as-built.

## W3 Toolchain-slide provenance

red-11 §1 explicitly flagged this exact hazard ("the ladder still resolves
impl32 because the legacy `target_has_atomic` key is accepted. Noted as an
upstream-cfg-migration hazard, not a current defect") and pinned nightly
only by CHANNEL, not by date. The hazard has now fired. This is a
toolchain-drift regression, not an RC-01/RC-02 diff defect: their courts,
mirrors, and witnesses were correct on the nightly that built them.

## W4 Why every existing witness missed it (non-vacuity analysis of the witnesses)

1. The RC-02 width witness (`seal_width_witness`, ref_store.rs:679-760) is
   **self-consistent under the slide**: `selected_width()` uses the SAME
   legacy cfg keys as the ladder, so both sides fall to usize::MAX
   together and the cross-check asserts vacuously. A witness derived from
   the same broken premise cannot catch the premise moving.
2. The impl32 mirror is `#[cfg(target_has_atomic = "32")]` — also
   silently cfg-false on the new nightly, so the mirror that would have
   asserted impl32's width contract is itself compiled out.
3. Host courts execute impl64 only; no court executes implcell's RMW
   (RC-01 W6.2 kept that leg REGISTERED). The arm that slid into service
   is precisely the never-executed one.
4. Probe 8 shows the width assert alone is non-discriminating on rv32
   (usize::MAX == u32::MAX numerically), so even a fresh width probe
   cannot catch the slide; a cfg-presence probe (probe 5) can.

## W5 Attack 3 — AHDCLT tie-in verdict

**No overconnection. The doc does NOT claim hardware/microcontroller
security.** `docs/jira/v26.10.31/16-ahdclt.md` states the surviving moat is
"state-chaining + adversarial courts + C-ABI integration cost" (:5-6, :31-52),
scopes it to "a DOWNSTREAM consumer running cgap's courts against a provider,
not a hostile rewriter outside the court perimeter" (:55-58), and already
RETIRED-REFUTED the compute-kernel half and Pillar 2 (:69-73, per RED-01/
RED-02). PY-5's residual therefore does not undermine AHDCLT; the doctrine
already fails closed to UNKNOWN. The defect is not in 16-ahdclt.md — it is in
the **PFMEA PY-5 row (08-ax-pfmea.md:128)**, which asserts
`riscv32imc=impl32` as a measured control on the floating nightly; that
control premise is now false and the row needs a re-derive (owner: PFMEA lane).

## W6 Disclosure honesty

- Honest: red-11/RC-01/PFMEA all keep the impl16/implcell execution
  certification leg REGISTERED and never claim execution certification.
- Stale: the same docs assert `riscv32imc=which arm` as a settled fact —
  true at receipt time, false today. Receipt-cited facts re-read at use
  time; the arm selection is a floating-nightly fact and must be
  re-derived per build, not receipted once.
- The `critical` doc comment's claim "not a silent one" is refuted by the
  execution behavior: unbound `critical = f()` IS silent at runtime. The
  prose overclaims loudness.

## W7 Standing summary

- ALIVE claims (this session, exits above): rv32imc build exit 0; disasm
  exit 0 with zero AMO/lr/sc and implcell critical instantiations;
  compile_error probe fires on rv32imc legacy-key absence; host probe
  shows key present; host reentrancy court exit 0 (5 passed).
- The residual is no longer "REGISTERED, dormant": it is
  **MATERIALIZED-ON-TARGET** (rv32imc binary executes unsynchronized RMW).
  SPOILER-CONFIRMED, elevated from REGISTERED.
- Fences run: ref_store_reentrancy_court (host) exit 0. Fences NOT run:
  full `cargo test -p cgap-git` (known foreign-lane breaks
  quota_alloc_purity_court.rs / egress_ring.rs:650, pre-existing per
  RC-01 W4 — same state this session, not re-run to avoid duplicate
  disclosure; RC-01 owns that fence re-run).
- Suggested owners for the fix (not this lane — read-only): the ladder
  must migrate to `target_has_atomic_load_store` (the new cfg family) or
  pin nightly by date; either closes the slide. A cfg-presence
  compile_error guard (probe 5 shape) belongs in ref_store.rs so the
  slide fails loud instead of silently selecting implcell.

## Replay

- Probe crate: /tmp/adv11-probe (includes the real ref_store.rs via
  #[path] for probes 5/8); disasm retained at /tmp/adv11-disasm.txt.
- Deterministic core: rebuild cgap-git for riscv32imc on any nightly that
  emits the legacy key -> impl32 (old behavior); on nightly 12433c307+
  -> implcell (this finding). Distinguish via the probe-5 cfg-presence
  compile_error, never via width asserts (non-discriminating on rv32).
- Lease cleanup: /tmp/adv11-target and /tmp/adv11-probe-t deleted at lane
  end via python shutil.rmtree; /tmp/adv11-probe retained for the
  coordinator replay window.
