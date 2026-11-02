# lane-gap-08-e2e-court

Honesty header — status vocabulary: ALIVE (ran this session, command+exit
cited); UNVERIFIED never rounds up; as-of 2026-11-02; inputs: ship-plan
section B (`docs/jira/v26.11.2/audit/doctrine-ship-plan.md:92-105`),
crates/cgap-git/src/{mcp_driver,admission_gate,octopus_merge,commit_synth}.rs
(read this session), soc_full_chip_court.rs (CountingAlloc pattern).

## W1 Subject

`crates/cgap-git/tests/e2e_wire_substrate_court.rs` (NEW, 668 lines, only
file written in-tree) + this receipt. No src edits, no lib.rs edits, no git
commands.

## W2 Diff

New court file only. Wire->substrate seam path, all via lib-mounted
`cgap_git::<module>` paths (lib.rs untouched):

1. FRAME IN — `Driver::dispatch` (real fs READ + TOOL Push queue) and
   `parse_frame`;
2. STATE — `parse_update_line` -> `RefTable::apply` (typed refusals);
3. MERGE — `octopus_merge` over 3 heads (dedupe + unify, shared rows
   asserted verbatim in output);
4. COMMIT — `LeaseCommitSpec::render` + `LeaseReceipt::seal`;
5. SEAL — `cgap_blake3::hash` + 80-byte seal shape (`RECEIPT_HEX_LEN/2`,
   layout provenance commented at test line ~430);
6. ALL inside ONE CountingAlloc window, positive control FIRST, window
   exits byte-balanced and traffic-positive, step probe asserts S1..S5==5.

## W3 Commands + exits

- `CARGO_TARGET_DIR=/tmp/gap08-target cargo test -p cgap-git --test
  e2e_wire_substrate_court` -> exit 0, `5 passed; 0 failed`; stable across
  8 consecutive re-runs (8/8 green after the fixes below).
- Full suite `cargo test -p cgap-git` -> ran this session; exit varies
  0/101 due to a PRE-EXISTING flake (see W6). All other targets ok (33
  `test result: ok` blocks observed).

## W4 The four flips (transcripts)

- F1 frame byte corrupt -> `ProtocolError::UnknownVerb` (through Driver
  AND parse_frame; clean-frame control parses). ok.
- F2 gate-rule violation -> `GateError::OntologyRuleBroken{77,
  UnboundHeadVariable}`; downstream probe `F2_DOWNSTREAM_STEPS` stays 0 on
  the violation arm; control arm (valid rule, same wire update) ADMITS and
  moves the probe to 1 (probe proven non-vacuous). ok.
- F3 poisoned head -> `MergeError::DomainClash{head_a:0, head_b:1,
  object_a:0xAAAA, object_b:0xBBBB}` with `subject==clash_s`,
  `predicate==clash_p`; `octopus_merge_andon` escalates under
  `catch_unwind`, payload contains "ANDON-REFUSAL". ok.
- F4 trailer mutate -> tag-hex flip => `ReceiptTagMismatch`; payload-hex
  flip (offset 80 in the 160-char receipt hex) => `ReceiptTagMismatch`
  (re-derivation mismatch); direct `expected_tag` API mismatch asserted;
  non-degenerate tag asserted. ok.

Anti-vacuity witnesses: during development the court FLIPPED for real —
S3 caught a wrong pinned count (77 vs 80), the window caught two real
accounting bugs ((1,2) and (145,104) imbalances) before the final
byte-balanced form; F4b initially surfaced `IllegalByte(96)` from an
invalid hex-flip helper, fixed to a valid-hex cycle.

## W5 window_counts divergence (recorded)

Dispatch said `window_counts()==(0,0) at exit`. The seam legitimately
allocates (BTreeMap, Vec, String) and Vec realloc growth makes EVENT
counts unreconcilable, so the G06 pattern is upgraded to BYTE accounting
with realloc folded as a delta and per-thread attribution
(`WINDOW_OWNER` ThreadId) to exclude harness threads. Exit condition
asserted: `alloc_bytes == dealloc_bytes != 0` (byte-balanced), not (0,0).
This is a deliberate, disclosed divergence from the literal dispatch
wording; a literal (0,0) is impossible over an allocating seam.

## W6 Fences run + pre-existing defects (not mine)

Fences: the wider cgap-git suite. Findings this session, both PRE-EXISTING
files (my file is not compiled into those test binaries):

- `tests/octopus_merge_court.rs::mutation_conflict_injection_flip`
  flakes ~50% even filtered solo (`test result: FAILED` then `ok` across
  consecutive runs; panic at src/octopus_merge.rs:366). Owner: whoever
  landed GIT-16 / octopus_merge_court. My diff cannot affect it.
- `tests/pack_demux_court.rs::c7_parse_path_is_zero_alloc` failed once
  under full-suite parallelism, passes filtered solo. Same class.

Both are loud, disclosed, and unowned by this lane. Full suite reached
exit 0 once this session; exit 101 on flake replays.

## W7 object_store exclusion (LOUD)

`object_store` is NOT lib-mounted: `crates/cgap-git/src/lib.rs:159` is
still `// pub mod object_store;` at court-authoring time (GAP-01
concurrent landing in flight). The court exercises the four mounted
modules and EXCLUDES object_store; it is never stubbed. When GAP-01
lands the mount, this court's header must be updated. (Note:
`tests/object_store_court.rs` exists and uses `#[path]` includes, so it
is unaffected by the mount state.)

## Standing

ALIVE for the lane deliverable (court file, command+exit cited above).
The ship-plan B pillar P5 "E2E seam court ABSENT" residue is closed by
this file; the flake ledger in W6 is handed to the coordinator.
