# dod-cgap-court-gate-p28-crypto

SEED-B audit lane, crates/{cgap-court, cgap-gate, cgap-p28, cgap-crypto},
v26.11.2. Only permitted write: this file. No source edits. All builds ran
under CARGO_TARGET_DIR=/tmp/seedb-target (lease, deleted at lane end).

## Task 1 - cargo test counts

All four lanes ran this session, 2026-11-02, exit 0.

- cgap-court: `cargo test -p cgap-court` EXIT=0. Single harness
  `tests/court_verdict.rs` (3313 lines, `[[test]]` at
  crates/cgap-court/Cargo.toml:9-11): 14 passed; 0 failed; 0 ignored;
  finished in 37.11s. No lib/bin targets (crate is a test-only wrapper;
  zero_string_gate confirms: "EXCLUDED crate=cgap-court ALLOWLIST-JUSTIFICATION:
  mechanical test-only classification (no [lib]/[[bin]] targets)").
- cgap-gate: `cargo test -p cgap-gate` EXIT=0. lib unittests 12 passed;
  `tests/lever_courts.rs` 3 passed; 0 failed.
- cgap-p28: `cargo test -p cgap-p28` EXIT=0. lib unittests 38 passed;
  tests/lever_courts.rs 2 passed; tests/p28_invariant_courts.rs 6 passed.
- cgap-crypto: `cargo test -p cgap-crypto` EXIT=0. lib unittests 4 passed;
  tests/chatman_courts.rs 18 passed; tests/lease_courts.rs 6 passed.
- The A17 pinned surface is witnessed: lease.rs:28
  `pub const PHI_LEASE_LEN: usize = 80` (80-byte PhiLease) with 32-B BLAKE3
  issued_root at crates/cgap-crypto/src/lease.rs:53 and 80-byte wire form at
  lease.rs:129/158.

## Task 2 - no_std / riscv32imc

Host `cargo check -p <crate> --no-default-features` EXIT=0 for all four;
`--target riscv32imc-unknown-none-elf --no-default-features` EXIT=0 for all
four. cgap-court compiles as a lib-check despite having no lib target
(zero targets to check = trivially clean; the test binary itself is
std-host-only by design, dev-deps carry `features = ["std"]` for cgap-parquet
at Cargo.toml:50).

## Task 3 - zero-string / boundary gates

- `python3 scripts/zero_string_gate.py` EXIT=0. SUMMARY scanned=92 crates=22
  hits=0 adv_hits=0 allowed=16638 vocab=9 gate=1.4.2. Per-crate scans
  covered all four audit subjects (cgap-court excluded loudly with
  justification; cgap-gate scanned=7, cgap-p28 scanned=4,
  cgap-crypto scanned=4). Self-check class proven separately by the gate's
  `--self-check` mode (repo-documented non-vacuity proof; not re-run this
  session — noted as such, not claimed).

## Task 4 - packaging / publish flags

| crate | publish | metadata vs house style |
|---|---|---|
| cgap-court | `publish = false` (Cargo.toml:7) | registry-absent-by-design
(CLAUDE.md structural `[[test]]`-only). Missing `description` key —
acceptable because publish=false, but house style asks for it; P3 cosmetic. |
| cgap-gate | `publish = true` (Cargo.toml:8) with dated receipt citation
(A14, `a14-finish-receipt.md`) — receipted flip per publish.md law. First
publish queued under the 2026-11-01 FROZEN guard (`publish-halt.md:31-42`);
`.claude/hooks/publish-guard.sh` enforces. description + license-file
present. |
| cgap-p28 | `publish = true` (Cargo.toml:6), description +
`license-file = "../../LICENSE.md"`, workspace-relative per house style.
One-attempt law binding at next unfreeze. |
| cgap-crypto | `publish = true` (Cargo.toml:6), description + license-file
present; P28-SIL URN cited in comment per A16 receipt. |

Divergence note: dispatch said "all publish=false per CLAUDE.md"; disk truth
is 1/4 false (court only). CLAUDE.md's registry-absent list is court/beam/
court/verification/integration — cgap-gate's flip to true is receipted at
Cargo.toml:8 and CLAUDE.md does not list cgap-gate. Doc yields to disk;
recorded here as a dispatch-level divergence, not a defect.

## Task 5 - courts and anti-vacuity mutation arms

- cgap-court (`tests/court_verdict.rs`, 14 courts): per-court mutation
  ledger at :75-98 ("each names the mutation that flips it"); frozen
  KillLedgerRow offsets pinned by `offset_of!` asserts (mutated_sha256 ==
  2292 at :278); §6.3 non-vacuity rows + strict-vs-mutation-mode
  differential (:98). Generator shim non-vacuity at :727.
- cgap-gate (`tests/lever_courts.rs`, 3 courts): named mutant = 80 B
  attestation byte flip escalating CGAP_OK -> CGAP_ERR_P28_SPOLIATION
  (:7-9, :42-73, one flip per attestation slot family); ceiling-law byte
  flip is the named mutant for L1 (:78-83).
- cgap-p28 (`tests/p28_invariant_courts.rs`, 6 courts): exhaustive
  640-bit single-bit flip sweep (:216-276) — exactly the 11 predicted
  CLEAR-flips of the ceiling field survive, all 629 others refused;
  in-file anti-vacuity probes A/B/C (:283-317) kill neutered-halt,
  truncated-sweep, and reject-all harness mutants. Add 2 courts in
  tests/lever_courts.rs.
- cgap-crypto: `tests/chatman_courts.rs` (18) — byte-sweep court flips
  EVERY byte of the receipt encoding, region-appropriate errors (:277-294);
  M1/M5 shim-court flip receipts cited (:11-21, replayable via the shim
  harness, latest wave-7 run exit 0). `tests/lease_courts.rs` (6) —
  80-byte PhiLease Tick monotonicity tier (A16).

## Verdicts

- cgap-court: VERIFIED-DONE (14/14, exit 0, host+riscv clean, gate green).
- cgap-gate: VERIFIED-DONE (15/15, exit 0, receipted publish=true).
- cgap-p28: VERIFIED-DONE (46/46, exit 0, 640-flip sweep alive).
- cgap-crypto: VERIFIED-DONE (28/28, exit 0, byte-sweep court alive).
- Standing caveat: all counts are this-session re-reads from cargo test
  output above; mutation arms are witnessed by in-file flip receipts and
  prior-wave shim-court runs, not re-executed in this read-only lane.
- `python3 scripts/boundary_gate.py` bare invocation: exit nonzero "no mode:
  one of --mutate, --shim-court-only, --deps, --strings required" — the gate
  is mode-driven, bare run is a usage error, not a gate verdict. Modes not in
  this lane's write scope; recorded as OUT-OF-SCOPE, not a pass.
- just doctor: NOT run this session (per-lane scope); status UNVERIFIED here.
