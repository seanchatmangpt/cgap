# Lane CGAP-PRD-6: zero-alloc stage checkpoint (v26.11.2 starDF)

- Subject: `crates/cgap-core/src/checkpoint.rs` (NEW, 119 lines),
  `crates/cgap-core/tests/checkpoint_court.rs` (NEW, 141 lines)
- Standing: **ALIVE**
- Command: `CARGO_TARGET_DIR=/tmp/prd6-target cargo test -p cgap-core --test checkpoint_court` — exit 0, 1 passed / 0 failed (run this session)
- Full-suite fence: `CARGO_TARGET_DIR=/tmp/prd6-target cargo test -p cgap-core` — exit 101, but the ONLY failure is `manifest_court::dod10_no_spdx_license_identifiers_anywhere`, PRE-EXISTING (see Fences below); all other targets green (17+1+8+2+11+4+6+2 passed, 0 failed)

## W1 Subject

Zero-alloc stage checkpoint prototype: snapshot ONLY the zero-alloc-stage
state — ring head/tail (u64 pair), receipt nonce (u64), receipt parent
([u8;32]) — into a caller-provided fixed 88-byte `CheckpointBuf`
(56 B state + 32 B BLAKE3 trailer). Restore validates the digest and
returns typed `CheckpointMismatch` on any corruption. API:
`CheckpointBuf::new/snapshot/restore/bytes/bytes_mut`.

## W2 Authority

Dispatch contract CGAP-PRD-6. Pattern authorities:
`docs/jira/v26.11.2/lanes/lane-posh-04-sentry.md` (2,662-alloc fixpoint
measurement, scoping citation; positive-control-first; single-driver
sequencing law) and `crates/cgap/tests/alloc_boundary.rs` (landed
`#[path]`-include + `EnclaveTrackingAlloc` registration pattern).
Digest injected as `DigestFn = fn(&[u8]) -> [u8; 32]`; court binds real
`cgap_blake3::hash` (cgap-blake3/src/lib.rs:197) via the §3 dev-dep
named exception (cgap-core/Cargo.toml:37-41). cgap-core stays E=0
externals; Cargo.toml and lib.rs UNTOUCHED (lib.rs off-limits, so
checkpoint.rs is mounted only via the court's `#[path]` include —
same E0152 convention as tracking_alloc.rs:14-18).

## W3 Consequence

Single driver fn (allocator window state is process-global):

0. **Positive control FIRST**: a 512-B alloc inside a zero-budget
   window MUST be observed as a sticky breach (site =
   "prd6-positive-control", offending_bytes >= 512). Witnessed
   flipping before the clean run; a window that cannot flip is
   VACUOUS.
1. **Roundtrip**: snapshot->restore field-equal; two snapshots of the
   same state are byte-equal regions.
2. **ANTI-VACUITY (named mutation)**: corrupting ONE region byte
   (state zone, byte 7; and digest trailer zone, byte 87) ->
   `Err(CheckpointMismatch)`. Also asserted: mutating the source
   state changes the region.
3. **Digest determinism/sensitivity**: identical state -> identical
   digest trailer; state change -> different trailer.
4. **Zero-alloc**: window_counts delta == (0 allocs, 0 bytes) across
   snapshot+restore (budget usize::MAX so the window observes without
   breaching; counters read AFTER arming — arm resets them).

Sandbox flip battery (mutation on /tmp sandbox copies, shared tree
untouched — sha256 1e0e72a2278f8ad5... identical pre/post; restore
sha256-exact + mtime-bumped before the post-restore baseline):

| mutation | expected | exit |
|---|---|---|
| (baseline) | pass | 0 |
| MUT1: `if false &&` the digest check (vacuum restore) | COURT2 flip | 101 |
| MUT2: constant `[0u8;32]` digest in snapshot | COURT3 flip | 101 |
| restored | pass | 0 |

## W4 Verification ladder

- Court exit 0 (above); sandbox flip battery (above) — the court is
  non-vacuous on both the restore law (MUT1) and the digest law
  (MUT2).
- Full cgap-core suite run (see Fences for the one pre-existing
  failure).

## W5 Replay

Frozen seeds: court state uses 0x1CE4_E5B9_DEAD_C0DE,
0xA20C_0FFE_E5A7_E001 (corpus seed + 1, derived NOT conflated with the
0xA20C_0FFE_E5A7_E000 constant); splitmix64 multiplier
0xBF58_476D_1CE4_E5B9 untouched. Deterministic; command above replays.

## W6 Standing grounds

ALIVE per the exit-0 court run this session against the exact subject.
UNVERIFIED residue: none in lane scope.

## W7 Fences

- Fence: cgap-core full suite. Result: exit 101 with exactly ONE
  failure, `manifest_court::dod10_no_spdx_license_identifiers_anywhere`
  (manifest_court.rs:56 flagging Cargo.toml:8). PRE-EXISTING and NOT
  lane-introduced: `git diff crates/cgap-core/Cargo.toml` shows an
  uncommitted working-tree edit (pre-dating this lane; Cargo.toml is
  lane off-limits) that moved the "P28-SIL v2.1.0" comment text onto
  the `license-file` line, which the SPDX-shape court refuses. Owner:
  the lane that made that Cargo.toml edit. My diff is NEW-only (two
  new files, nothing shared edited).
- Fence: E=0 externals for the cgap-core lib target — held; checkpoint
  is pure core, no deps added.
- PENDING-design: audit/cgapprd-04-checkpoint-design.md ABSENT at
  write time (PRD-4 in flight). API shape follows the dispatch
  contract; reconcile via errata when PRD-4 lands.
- Disclose-in-both-directions: none applied; no other lane's files
  touched.
