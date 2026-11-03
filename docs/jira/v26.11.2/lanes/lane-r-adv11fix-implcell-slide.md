# Lane R-ADV11FIX — toolchain-slide cfg regression repair (ref_store)

- Lane: R-ADV11FIX over `/Users/sac/cgap` (branch main). Owned files:
  `crates/cgap-git/src/ref_store.rs` (cfg ladder, critical seam,
  witness), `crates/cgap-git/tests/ref_store_width_arms_court.rs`.
  No git write commands; coordinator owns transitions.
- Subject: ADV-11 spoiler (`redteam/corrosion/adv-11-ahdclt-concurrency-hole.md`)
  — legacy `target_has_atomic` cfg ladder silently selecting implcell
  on riscv32imc with a pass-through `critical`.
- Standing: **ALIVE** — every command below ran this session with the
  exit shown.

## W1 Empirical cfg probe (the slide, quantified)

`rustc --version` → 1.101.0-nightly (12433c307, 2026-10-24).

| key | host (aarch64) | riscv32imc |
|---|---|---|
| `target_has_atomic="64"` (legacy, CAS) | present | ABSENT |
| `target_has_atomic="32"` (legacy, CAS) | present | ABSENT |
| `target_has_atomic_load_store="32"` (new) | present | present |
| `target_has_atomic_load_store="64"` (new) | present | ABSENT |

(`rustc --print cfg --target riscv32imc-unknown-none-elf`, exit 0.)

Correction to the spoiler's framing: the legacy key was NOT "removed"
globally — it is present on host on the SAME nightly. The slide is that
rv32imc no longer reports CAS it does not have; the new load_store
family correctly reports load/store-only for "32". The legacy
`target_has_atomic` keys remain the only CAS signal, so the ladder keeps
them, and the mis-arm (impl32 without CAS) is now impossible: rv32imc
falls to implcell and REFUSES loudly.

## W2 The fix (three layers, ref_store.rs)

1. Ladder keyed on legacy CAS keys only; implcell arm condition widened
   to `not(64)+not(32)+not(16)+not(8)`.
2. implcell::critical is now TYPED: pass-through `f()` exists only
   behind `#[cfg(implcell_critical_provided)]`; without the opt-in, a
   compile_error names both cfg families and both lawful exits. rv32imc
   builds are loud-until-bound, never silently-unsynchronized.
3. Independent arm discriminator in `seal_width_witness`: SELECTED_ARM
   (legacy CAS keys) vs EXPECTED_ARM (independent
   `target_pointer_width` family); mismatch = compile_error
   "cfg-arm discriminator" — kills the self-consistent-vacuity channel.

## W3 Divergences (dispatch vs reality)

1. Dispatch named a Cargo feature `implcell-critical-provided`;
   Cargo.toml is outside this lane's file ownership, so the opt-in is
   realized as `RUSTFLAGS="--cfg implcell_critical_provided"` — same
   typed contract, acceptance visible in the build command itself.
2. Dispatch's "current cfg(target_atomic...)" keys: the
   `target_has_atomic_load_store` family is UNSTABLE (E0658,
   feature-gated `cfg_target_has_atomic`) and unusable from cfg
   predicates without a crate-level feature gate (lib.rs off-limits).
   The independent discriminator therefore probes the stable
   `target_pointer_width` family instead of the load_store family. The
   empirical probe table (W1) is retained as the ground-truth record.
3. Impl16/implcell arms' not-64 addition: cosmetic cfg tightening; the
   ladder ordering is unchanged.

## W4 Commands/exits (this session)

- `cargo build -p cgap-git` → exit 0.
- `cargo test -p cgap-git --test ref_store_width_arms_court` → exit 0,
  **8/8 passed** (w1 loud-std-gate, w2 loud-no-CAS-refusal, w2b opt-in
  green, w3 host width, w4 width-mutation flip, w5 std-gate flip,
  w6 discriminator identity, w7 discriminator-mutation flip).
- `cargo build -p cgap-git --target riscv32imc-unknown-none-elf
  --no-default-features` → **exit 101** with the typed refusal naming
  `implcell_critical_provided` and "no CAS-capable atomic".
- Same build with `RUSTFLAGS="--cfg implcell_critical_provided"` →
  exit 0.
- `cargo test -p cgap-git --no-fail-fast` (full crate): 63/0 lib,
  all suites green EXCEPT
  `octopus_merge_court::mutation_conflict_injection_flip` — verified
  PRE-EXISTING via a /tmp baseline sandbox with HEAD `ref_store.rs`
  (/tmp/radv11-headcheck): identical failure → foreign, not
  session-introduced. Transient parallel-suite failures
  (andon/breach/quota) passed on isolated rerun in the shared tree.
- Anti-vacuity flip battery (court legs w4/w5/w7, /tmp sandboxes, real
  cargo builds): each mutation applied (assert_ne) → build fails with
  the specific witness message. Mutations never touch the shared tree.

## W5 Fences run (must-not-regress surfaces)

- `ref_store_width_arms_court` (owned): 8/8.
- `ref_store_reentrancy_court` (RC-01, host): green within full suite.
- Full `cargo test -p cgap-git`: green except the pre-existing foreign
  octopus failure (W4). RC-01's reserve/stamp semantics untouched —
  diff regions are cfg/witness only.
- NOT run: `make gate0`, publish steps (freeze in force); rv32imc
  builds elsewhere in the workspace that compile cgap-git now REQUIRE
  the opt-in flag — downstream fence owners (make gate0, any rv32imc
  workspace build lanes) must add `RUSTFLAGS="--cfg
  implcell_critical_provided"` or accept the loud refusal. This is the
  intended, disclosed loud-until-bound change.

## W6 Terminus for coordinator

ADV-11 spoiler: **REPAIRED-RECEIPTED** (this receipt). Follow-ups NOT
in this lane's files: PY-5 row (`dflss/08-ax-pfmea.md:128`) must drop
the `riscv32imc=impl32` control claim (now: rv32imc = implcell,
loud-until-bound, requires opt-in); red-11 §1 hazard row superseded by
this receipt; the module doc (ref_store.rs:30-39) still says rv32imc
selects impl32 — corrected by W2's ladder but the stale prose survives
at ref_store.rs:30-39 for the doc lane. Errata law respected: this
receipt corrects the ADV-11 spoiler's "legacy key removed" framing
(its W2/W3) — spoiler file untouched.

## W7 Replay

- Shared tree only carries the two owned-file diffs; mutations lived in
  /tmp sandboxes (rc02-court-*), restored/deleted per-leg.
- Baseline sandbox /tmp/radv11-headcheck: HEAD ref_store.rs + current
  foreign-lane src; used to prove the octopus failure pre-existing.
- Lease cleanup: /tmp/radv11fix-target,
  /tmp/radv11-headcheck-target, /tmp/radv11-headcheck deleted at lane
  end via python shutil.rmtree.
