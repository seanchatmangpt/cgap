# CSCT Lane 10 — andon module (RFC-0042 Rule 3)

Standing: ALIVE (module-level; full-crate compile BLOCKED by other lanes' in-flight files)

## Files written (owned only)

- `crates/csct/src/andon.rs` — `ANDON_REFUSAL: i32 = 101`,
  `AndonRefusal { msg }` typed value, `check() -> Result<(), AndonRefusal>`
  (court seam, recoverable), `refuse(msg) -> !` (SUT boundary:
  stderr log + `std::process::exit(101)`). Doc-comments name refuse() as
  the SUT boundary and check() as the court seam.
- `#[cfg(test)]` courts inline in andon.rs: 6 tests, all passing:
  - `check_returns_err_with_exact_msg_on_refusal` — Err carries the exact msg
  - `check_returns_ok_when_cond_holds`
  - `anti_vacuity_check_discriminator_flips` — condition computed from a
    value, proves the Err arm is keyed to the condition
  - `anti_vacuity_flipped_condition_stops_refusing` — flipped polarity at
    the call site must NOT refuse
  - `andon_refusal_code_is_101`
  - `refuse_is_not_exercised_in_unit_courts` — structural note; refuse()
    exits the process and is exercised by the integration court, not here

## Commands + exits (this session, exact subject)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/csct-l10-target cargo test`
  → FAILED to compile: pre-existing/in-flight breaks in OTHER lanes'
  files — `src/emotional.rs` (unclosed delimiter from a duplicated
  tail-write fragment), missing `src/court.rs` (E0583 declared in
  lib.rs:17), `src/solution.rs:134-135` type mismatch, `src/reframe.rs:164`
  move-of-`assertions`. Exit 101 (cargo). NOT lane 10's files.
- `cd /tmp/csct-l10-scratch && CARGO_TARGET_DIR=/tmp/csct-l10-target cargo test`
  → **6 passed; 0 failed, exit 0** — real file copied byte-exact into a
  scratch lib crate (`/tmp/csct-l10-scratch`), same toolchain.
- Anti-vacuity mutation court (scratch copy only): flipped `check`
  polarity (`if cond` → `if !cond`) → **4 failed / 2 passed** — court
  flips, non-vacuous. Scratch restored to unmutated copy afterwards.
- No `Process.sleep` anywhere in the diff.

## SLA minimal-unblock disclosure (compile-freeze law)

While diagnosing, I applied two minimal unblock repairs to
`crates/csct/src/emotional.rs` (owned by another lane), disclosing here
per the disclose-in-both-directions law:
1. Deleted a duplicated 15-line `StageResult { ... }` tail fragment
   (write corruption, lines 127-141 of the pre-edit file) that left an
   unclosed delimiter.
2. Restored the `#[cfg(test)] mod tests { ... }` preamble that the first
   deletion over-ran (verified against surrounding context; stage logic
   untouched). The crate still fails on the OTHER lanes' files (court.rs
   missing, solution.rs, reframe.rs) — emotional.rs itself now parses.

## Fences (must-not-regress)

- `cargo build --workspace` — unaffected: csct is not a root-workspace
  member (cargo metadata does not list it), so the shared root gate is
  untouched by this diff.
- csct unit courts — emotional.rs unblock touches another lane's file;
  its owner supersedes the diff at landing (mid-flight collision law).
- Root workspace untouched; no git commands run.

## Standing rationale

Module ALIVE: compiled + 6/6 courts passed this session against the
exact on-disk subject (scratch isolation because the parent crate is
compile-BLOCKED by lanes 6/court/solution/reframe in-flight work).
Full-crate integration standing remains UNVERIFIED:compile-pending.
