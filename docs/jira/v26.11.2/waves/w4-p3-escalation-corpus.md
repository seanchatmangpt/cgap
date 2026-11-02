# W4-P3 — escalation-corpus lane: CRITICAL design finding (lane stopped at dispatch step 1)

Lane W4-P3 · repo /Users/sac/cgap · branch main · 2026-11-01

## Verdict (CRITICAL design finding — dispatch step-1 stop fired)

**The escalation lever can NEVER pay BLAKE3 on a chain-admitted frame.
Escalated∧chain-admitted is structurally impossible, not corpus-empty.**
The lever's value is skip-not-pay, and W3-L1's 10.1x HOLDS — but not via
"digest bounded to a suspicious subset"; it holds because the escalated
subset is provably empty for every frame the chain admits.

Predicate identity (proved by execution over all 256 flag values this
session, constants extracted from the on-disk sources):

- `ESCALATE_MASK = 0x00F0` on header byte 5
  (`crates/cgap-gate/src/penstock.rs:269`, read at penstock.rs:613) and
  stage-1's refusal law `FLAG_RESERVED_MASK = 0xF0` on the SAME byte
  (`crates/cgap-core/src/frame.rs:38`, enforced at frame.rs:284 →
  `FrameError::ReservedBitsSet`) are the same predicate over all 256
  flag values: `escalate == stage1_refusal`, 0 exceptions.
- `ingress` admits row-6 frames with CGAP_OK (penstock.rs:511) and mints
  `escalate = true` exactly when stage 1 refuses (frame.rs:284). So the
  ONLY frames the dam marks escalate are frames the chain refuses at
  stage 1. The suspicious-subset digest stage is dead code at the chain
  level: `stage3_escalated`'s BLAKE3 branch (chains-ingress.rs:162-168)
  is unreachable for any chain-admitted frame.
- No legal trigger exists. The dispatch's candidate triggers both fail:
  - distinct reserved sub-bit that admits — impossible: A04 §6.3/§8.3
    refuses ALL of bits 4-7 (`FLAG_RESERVED_MASK = 0xF0`, no admitting
    sub-bit);
  - first-seen-flow — explicitly out of scope for this lever
    (penstock.rs:476-477: "Flag-only policy (no state table;
    first-seen-flow is the next stage, out of scope this lever)").

W3-L1's receipt framed the empty subset as corpus-construction
("stage 1 refuses reserved-flag frames" as if it were about that corpus);
it is stronger than that: it is the admission law itself. The vacuity
was correctly named by W3-L1; this lane corrects the mechanism (law, not
corpus).

## Why the 10.1x still holds (honest re-framing)

102.7/10.20 ns/frame is real (W3-L1, same corpus/method), but the
chain's actual behavior is: digest stage = prefilter fold ALWAYS — the
"two-stage escalation" is a one-stage policy in practice. The honest
claim is: **skip-not-pay with a vacuous escalated branch**, not
"digest bounded to the suspicious subset". The lever spec's suspicion
policy cannot fire without changing one of the two masks (see Repair
surface below). The blended/escalated-corpus numbers (dispatch step 2)
are UNDEFINED-INTEREST: a CORPUS_ESCALATED with 5% row-6 frames would
have 100% of its escalated frames chain-REFUSED at stage 1, so the
"escalated subset's digest cost" measures refuse-path cost, not digest
cost. Per dispatch, step 2/3 on the corpus is not built; the court flip
was still run (below) against the existing corpus court.

## Grounding (all run this session, CARGO_TARGET_DIR=/tmp/w4-p3-target)

- `cargo test -p cgap-gate --test lever_courts` → exit 0, 3/3 green
  (includes `l1_prefilter_escalation_policy_court`: row-6 frame →
  CGAP_OK + escalate=true — the dam-side trigger is ALIVE).
- `cargo test -p cgap-core --lib reserved` → exit 0 (2 passed; includes
  `frame::tests::mutation_reserved_flag_bit_refused` — stage-1 refusal
  is ALIVE).
- Predicate-identity execution (python over constants extracted from
  the on-disk penstock.rs/frame.rs): IDENTITY_HOLD=True over all 256
  flag values, 0 exceptions.
- **Escalate-never court flip** (the dispatch's named mutant):
  /tmp sandbox /tmp/w4-p3-sb (standalone manifest patch: workspace
  fields inlined; sandbox-only). Shared tree never mutated.
  - originals sha256: penstock.rs
    `ffee1f5343d1170405e91c5a5d4ce11abd639b8ecf8815b76be90899fa594c12`;
    lever_courts.rs
    `92f77dc53f69280c7eb55cd7b2f999e4fea90472f86e21c446faaa7d8b9a4792`.
  - MUTANT: `view.escalate = false;` injected (escalate-never):
    `l1_prefilter_escalation_policy_court` FAILED → **FLIPPED**,
    `cargo test --test lever_courts` exit 101 (2 passed / 1 failed).
  - Shared-tree sha256 unchanged (no restore needed; mutation lived
    only in the sandbox copy). Leases removed post-run.

## If the lever is to be repaired (surface named, NOT built — out of lane)

Escalation can only pay on chain-admitted frames if the escalate bit
reads a shape stage 1 admits. Lawful candidates (all require an A04
amendment — A19 register candidate, not a silent local change):

1. Derive escalation from a shape stage 1 does not refuse: e.g. digest
   entropy class or payload-length outlier (computed post-admit), or
   the attestation lineage bit shape (stage 1 admits all three legal
   lineage values).
2. Split the reserved family: A04 §6.3 amendment reserving bits 4-6 as
   "ignored on read" (dam-admitted) and bit 7 as "must be zero"; set
   ESCALATE_MASK = 0x0080. Requires cgap-core frame.rs validate change
   + its 15 frame tests re-greened; cross-lane surface.
3. Keep the flag-only policy and rename the lever honestly: escalation
   marks refuse-path triage, not digest dispatch; the digest leg is
   permanently vacuous and should be deleted (ERRC: delete outranks
   generate) or re-seated on a post-stage-1 signal.

This lane wrote NO code changes: the stop branch fired, and the repair
is a cross-lane (cgap-core + cgap-gate + bench + A04/A19 register)
amendment that outruns the dispatch's file ownership.

## Session-observed, not this lane's defects (recorded, owner unnamed)

- `cargo test -p cgap-gate -p cgap-core` (shared gate): cgap-gate lib
  test target failed to compile (71 errors, `use crate::std::...`
  no_std shim drift in `#[cfg(test)]` code around penstock.rs:700) —
  another lane's in-flight edit; the `--test lever_courts` target
  compiled and passed, so this lane was not blocked.
- `crates/cgap-core/tests/manifest_court.rs:56`
  `dod10_no_spdx_license_identifiers_anywhere` FAILED — flags
  `license-file = "../../LICENSE.md"` as an SPDX-shaped reference in
  cgap-core/Cargo.toml:7. Pre-existing at run time, not this lane's
  surface.
- Transient: `crates/bench/Cargo.toml` referenced `dred_churn` bench
  before the file existed (another lane mid-write); file landed
  in-session, gate re-parsed clean.

## Standing

ALIVE — every gate run, court, flip, and the identity proof ran this
session with exits recorded above. Steps 2/3 of the dispatch (new
corpus, blended numbers, new bench leg) UNBUILT by the dispatch's own
step-1 stop clause; the "corpus spec / blended numbers" receipt rows are
structurally impossible, recorded as such, not skipped silently.
