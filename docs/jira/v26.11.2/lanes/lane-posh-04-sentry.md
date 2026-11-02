# Lane POSH-4: zero-alloc sentry court (W-POSH)

- Subject: `crates/cgap-git/tests/posh_alloc_sentry_court.rs` (NEW, 362 lines)
- Standing: **ALIVE**
- Command: `CARGO_TARGET_DIR=/tmp/posh4-target cargo test -p cgap-git --test posh_alloc_sentry_court` — exit 0, 1 passed / 0 failed (run this session, 6.3 s)
- Re-run with transcript: `... -- --nocapture` — exit 0 (grep 'POSH-4', exit 0)

## W1 Subject

New Chicago court mirroring the landed soc_full_chip_court.rs:57-104 pattern
exactly (CountingAlloc + scoped window + positive control FIRST), extended
with a bytes ledger (ALLOC_BYTES) for the bytes==0 assertion.

## W2 Authority

Dispatch contract POSH-4; pattern authority
`crates/cgap-git/tests/soc_full_chip_court.rs:57-104`; honesty pin C03
(`crates/cgap-bn3/tests/edge_closure_courts.rs:422-477`, e5).

## W3 Consequence

Courts in one driver fn (allocator window state is process-global):

1. **Positive control FIRST** (the anti-vacuity flip): deliberate 512-B
   heap alloc inside an open zero window MUST be observed — witnessed
   allocs=1 / bytes=512 before the clean run is trusted. A window that
   cannot flip is VACUOUS.
2. **Sentry window**: allocs==0 AND bytes==0 over the full 1,365-fact
   pack-v2 window (46,442 bytes) streamed through a real
   `StdIngressRing` (push/pop, byte fidelity asserted) and sealed with
   real `cgap-blake3::hash`. Observed (0,0). Receipt root non-zero
   ([208,15,81,80] first 4 bytes).
3. **Honesty window (scoping decision)**: bn3 `unify_to_fixpoint` over
   the 1,293-edge window-derived edge corpus. The sentry window never
   covered it — the fixpoint allocates by construction (W6-BFX owned
   delta tails; C03/e5 pinned warmup allocation). Measured, not faked:
   **2,662 allocs / 650,294,994 bytes** (3 batches, 2,929,939 derived
   rows, fixpoint converged). Also asserted fp_allocs>=1 per the C03
   pin — if bn3 ever reaches zero-alloc fixpoint, this court fails
   LOUDLY telling maintainers to tighten scoping.

## W4 Verification ladder

- cargo test exit 0 (above) — courts 1-3 all asserted.
- Ring byte fidelity asserted inside stage fn (total_in == total_out ==
  window len).
- Window byte law asserted: wlen == 12 + 1365*34 + 20 = 46,442.

## W5 Replay

Frozen seeds 0xBF58_476D_1CE4_E5B9 (splitmix64 multiplier) and
0xA20C_0FFE_E5A7_E000 (corpus) untouched; deterministic corpus; single
test binary; command above replays byte-identically.

## W6 Standing grounds

ALIVE per the exit-0 run this session; positive control witnessed
flipping before the clean run; fixpoint allocations measured and printed
as an honest finding, never folded into the zero claim. UNVERIFIED
residue: none within lane scope.

## W7 Fences

- Fence: existing cgap-git test suite — my file is NEW-only (no shared
  lib edits); lib.rs untouched. `#[path]` include of ingress_ring.rs is
  read-only of another module, the landed soc-court pattern.
- Fence run: none of the pre-existing cgap-git tests were modified, and
  I did not run the whole cgap-git suite (out of lane scope; soc court
  is a separate lane's receipt). Cargo warnings emitted are from the
  `#[path]`-included module's unused-methods under this binary only.
- Divergence noted: `cgap-bn3` was already a declared dev-dependency of
  cgap-git, so NO #[path] include of bn3 sources was needed (dispatch
  anticipated it might be; nothing to disclose beyond this line).
- Disclose-in-both-directions: none applied; no other lane's files
  touched.
