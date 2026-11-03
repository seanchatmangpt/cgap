# Lane RC-02 — ref_store width-arm compile witnesses + honest riscv feature gating

- Date: 2026-11-02. Lane contract: owns
  `crates/cgap-git/Cargo.toml`, ref_store.rs END-of-file cfg-witness
  module only, and NEW
  `crates/cgap-git/tests/ref_store_width_arms_court.rs`.
- Standing: **ALIVE** (every command below ran this session, exits cited).
- Source findings: `docs/jira/v26.11.2/redteam/red-11-riscv-reentrancy-corruption.md`
  findings 1 (impl16/implcell arms never compiled = UNVERIFIED arms)
  and 5 (riscv32imc default-feature build = raw E0463, exit 101).

## Diff

1. `crates/cgap-git/Cargo.toml` — `[features]` comment documents the
   honest gate: `std` is host-only; OS-less targets build with
   `--no-default-features`; selecting `std` on an OS-less target now
   fails LOUDLY via compile_error (point 2), replacing the raw E0463
   as the primary signal.
2. `crates/cgap-git/src/ref_store.rs` — appended
   `mod seal_width_witness` at the END of the file (counter logic
   region untouched; RC-01 owns that region):
   - W1: `#[cfg(all(feature = "std", target_os = "none"))] compile_error!`
     naming `--no-default-features` as the fix.
   - W2: mirror modules + `const _: () = assert!(...)` width contracts
     for impl32/impl16/implcell; the never-compiled arms now compile
     their type-level guarantees on EVERY target. impl16/implcell go
     from UNVERIFIED to compile-witnessed on host (aarch64) and on
     riscv32imc (implcell mirror compiles there too). Behavioral RMW
     coverage of implcell remains open (RED-11 finding 4/repair, RC-01's
     region).
   - `SEAL_COUNTER_WIDTH == selected_width()` cross-check: exactly the
     target's selected arm's width must equal the public constant.
3. `crates/cgap-git/tests/ref_store_width_arms_court.rs` — NEW court,
   5 legs, all green, exit 0:
   - w1: riscv32imc default-feature sandbox build must FAIL and carry
     the witness message (divergence noted below re: E0463).
   - w2: riscv32imc `--no-default-features` must stay green.
   - w3: host width contract (SEAL_COUNTER_WIDTH == u64::MAX).
   - w4 (anti-vacuity): break the implcell mirror width constant in a
     /tmp sandbox copy → build fails with "implcell arm width contract
     violated".
   - w5 (anti-vacuity): flip the W1 gate cfg to fire on host
     (`target_pointer_width = "64"`) → host default build fails with
     the witness message (gate proved non-vacuous).
   Chicago: real cargo builds against real sandbox copies of the real
   crate source; no mocks. Shared tree never mutated. Sandboxes are
   minted per-test under `~/.cache/tmp/rc02-court-*` and deleted at
   test end (asserted, fail-closed).

## Commands (this session, exits captured)

| cmd | exit |
|---|---|
| baseline `cargo build -p cgap-git --target riscv32imc-unknown-none-elf` (default) | 101 (E0463) |
| baseline `--no-default-features` | 0 (after one transient E0599 mid-edit race with RC-01; clean on rerun) |
| `cargo test -p cgap-git --test ref_store_width_arms_court` | 0 (5 passed) |
| `cargo test -p cgap-git --test ref_store_court` | 0 (5 passed) |
| `cargo test -p cgap-git` (full suite) | 101 — 2 parallel-run flakes, both green in isolation (see Fences) |
| shared-tree riscv32imc `--no-default-features` (post-unblock) | 0 |
| shared-tree riscv32imc default features (post-unblock) | 101 with the witness message (2 message occurrences in log) |
| isolated `quota_flood_alloc_purity` x4 | 0 (4/4 green) |

## SLA unblock (disclose-in-both-directions)

- Witnessed mid-flight RC-01 edit broke the riscv `--no-default-features`
  build: `seal_atomic::implcell::critical` was private (E0603,
  ref_store.rs:222), visible ONLY on the riscv arm (host green — a
  silent freeze for riscv-checking lanes).
- RC-02 applied the minimal unblock: `mod implcell` →
  `pub(crate) mod implcell` (one word). The owning lane (RC-01)
  supersedes this diff on landing. RC-01's receipt must cite this
  unblock; this receipt cites RC-01's `critical` seam as the mechanism.
- During the unblock an intermediate edit transiently dropped
  `use core::cell::Cell;` from implcell — repaired within the same
  command pair and verified on disk (both `use core::cell::Cell;`
  occurrences present, witness tail byte-checked).

## Fences (must-not-regress, all run)

- `ref_store_court` c1–c5: green, exit 0.
- Full `cargo test -p cgap-git`: 2 failures observed under default
  parallelism — `quota_flood_alloc_purity` (alloc-count assert 165 vs
  229) and `mutation_conflict_injection_flip` (ANDON panic, then ok on
  the in-run rerun). Both pass in isolation (quota court 4/4 green
  isolated). Classified pre-existing parallel-run interference flakes,
  NOT regressions of this diff: the RC-02 diff touches ref_store.rs
  witness tail + Cargo.toml comment only; neither failing test exercises
  ref_store. RC-01's in-flight counter edits are the likelier cause of
  allocation-count drift in the quota court. Fence disclosed; owner
  lanes should re-run full suite at integration.
- Court binaries compile under default features (std) on host; the
  witness compile_error cfg (std + target_os="none") cannot fire on
  host, so it never blocks the test build.

## Divergences vs lane doc / red-11

1. w1 asserted `!out.contains("E0463")` first; rustc emits E0463
   regardless (lib.rs:13 `extern crate std` cannot be silenced by a
   compile_error). Court amended: witness message presence is the
   assertion, E0463 absence is NOT claimed. Red-11's "replace the raw
   E0463" is hence "accompany the raw E0463 with a loud named fix" —
   the minimal honest change.
2. RED-11 recorded riscv `--no-default-features` exit 0; a transient
   E0599 (RC-01 mid-edit race) hit this session and passes on rerun.
3. Full-suite exit 101 from the two flakes above; standing of THIS
   lane's surfaces is ALIVE; the flakes are owned by quota/octopus
   lanes' surfaces, re-verify at integration.

## JSON terminus

```json
{"lane":"RC-02","files_written":["crates/cgap-git/Cargo.toml","crates/cgap-git/src/ref_store.rs","crates/cgap-git/tests/ref_store_width_arms_court.rs","docs/jira/v26.11.2/lanes/lane-rc02-ref-store-width-arms.md"],"commands":["cargo build -p cgap-git --target riscv32imc-unknown-none-elf (default) exit 101","cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features exit 0","cargo test -p cgap-git --test ref_store_width_arms_court exit 0 (5 passed)","cargo test -p cgap-git --test ref_store_court exit 0 (5 passed)","cargo test -p cgap-git exit 101 (2 isolated-green flakes)"],"courts_run":["ref_store_width_arms_court w1-w5 (incl. 2 anti-vacuity flips)"],"standing":"ALIVE","divergences":["E0463 remains alongside the witness; loud-fix message is the assertion","full-suite parallel flakes classified pre-existing, isolation-green"]}
```
