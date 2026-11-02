# dod-lsp-max-wire

Audit lane AUD-01. Subject: `/Users/sac/lsp-max` crate `branchless-lsp`
(branch main). Re-verified from live commands 2026-11-02, not cited from
G04/G05/G06/G07 receipts. Peer DoD docs: `dod-cgap-blake3.md`,
`dod-cgap-branchless.md`, `dod-cgap-core.md` (cgap-tree crates; no scope
overlap — this doc covers the lsp-max wire surface only). Receipts
re-read at use time:
`docs/jira/v26.11.2/lanes/lane-g04-lspr-attributes.md`,
`lane-g05-lsp-wire-rebuild.md`, `lane-g06-lsp-didsave-court.md`,
`lane-g07-lsp-conformance-seal.md`.

## 1. Full suite — VERIFIED-DONE

Command: `cargo test -p branchless-lsp` — exit 0.

| Suite | Passed | Failed |
|---|---|---|
| lib unittests (src/lib.rs) | 72 | 0 |
| unittests src/main.rs | 0 | 0 |
| code_actions_courts | 5 | 0 |
| did_save_court | 4 | 0 |
| e9_precision_court | 6 | 0 |
| riscv32imc_court | 3 | 0 |
| Doc-tests | 0 | 0 |

Total 90 passed / 0 failed — matches G06's witnessed 90 (lane-g06
receipt:31-38) and G07's per-target lines (lane-g07 receipt:37-38).
G04's receipt cited the same 72-lib count (:54).

## 2. Rebuilt binary ANDON_REFUSED literal — VERIFIED-DONE

G05 witnessed count = 3 on the 12:53 release binary. This audit found
that binary STALE: `src/lib.rs` and `src/conformance.rs` mtimes (12:56)
postdate it (G06 spawn_blocking hunk at lib.rs:186 and G07 module
registration landed after the 12:53 build). Rebuilt:
`cargo build --release -p branchless-lsp` — exit 0 (20.93s), fresh
binary mtime 13:44.

`strings target/release/branchless-lsp | grep -c ANDON_REFUSED` = 3
(exit 0), matching G05's witness on the fresh binary. Debug binary
rebuilt by the test run (mtime 13:02): same count 3.

## 3. WG2 live-wire claim (branchy refused / clean empty) — VERIFIED-DONE

Re-ran the python harness (`python3 /tmp/g05_drain.py`, exit 0) against
the FRESH 13:44 release binary — not merely cited from G05. Observed:

- BRANCHY `rf_branchy` (forward `while`): 2 publishes for the URI;
  publish[0] n=0 (tier-1, did_open); publish[1] n=3 (tier-2, did_save) —
  two `ANDON_REFUSED: fn rf_branchy carries a forward conditional
  branch` diagnostics, severity 1 (ERROR), line 3, plus 1 "blocking
  diagnostic emitted without a corresponding ANDON push event".
  `conformance.refused = [{'custom': 'ANDON_REFUSED'}]`.
- CLEAN `rf_clean`: publish[0] n=0; publish[1] n=0;
  `conformance.refused = []`.

Flip pair held on the rebuilt binary. Matches G05's receipt:31-43 line
for line.

## 4. Known OPEN client bug (masking, not lane-owned) — DEFECT-OPEN

`python/lsp_client/branchless_client.py:263-273`
`wait_for_diagnostics` returns the FIRST publishDiagnostics for a URI
(branchless_client.py:273), so a caller using it observes the tier-1
empty publish and masks the tier-2 verdict publish. G05 finding
(lane-g05 receipt:26-29, :75-77); still unfixed at audit time. The
harness works around it by draining `client._notifications` directly
(/tmp/g05_drain.py:24-34). Server behavior is correct on both
fixtures; this is a client-side masking bug only. Not fixed here
(not a lane-owned file). The related stderr-to-DEVNULL quirk
(branchless_client.py:217, G05 receipt:28-29) is also open.

## 5. G07 80-byte BLAKE3 conformance seal + zero-alloc court — VERIFIED-DONE

Symbols confirmed on disk:
- Flip court `perturbing_one_counter_flips_seal_bytes` at
  `crates/branchless-lsp/src/conformance.rs:173` — perturb ONE counter,
  seal bytes flip + tampered-prefix decode refuses BadDigest
  (lane-g07 receipt:51-53).
- Zero-alloc instrument: counting global allocator in conformance.rs
  with `pub(crate)` watch helpers `alloc_watch_reset`/`alloc_watch_get`
  (conformance.rs:236, :241); receipts.rs
  `serialize_loop_is_zero_allocation` (receipts.rs:606) rewired to them
  (receipts.rs:615, :617, :627). Both ran green inside the 72-lib suite
  (exit 0, section 1). Positive control (real Vec alloc witness,
  conformance.rs:254-256) guards vacuity.
- Sealed layout (magic "CLSP", version 1, 5x u64 totals, BLAKE3-256
  over bytes 0..48) per lane-g07 receipt:24-29 — layout constants live
  in conformance.rs, exercised by the round-trip/accumulation/saturation
  courts in the same suite.

## 6. G04 parse_attrs key-value form — VERIFIED-DONE

`parse_attrs` at `crates/branchless-lsp/src/attributes.rs:196`.
Witness tests confirmed on disk and green in the suite run:
- `parameterized_attr_parses_full_payload` (attributes.rs:654)
- `key_value_form_round_trips_through_reason_note` (attributes.rs:796,
  G04 gap-fill, lane-g04 receipt:30-32)
- Typed-refusal mutants (`unknown_param_key_is_typed_refused`,
  `malformed_and_duplicate_params_are_typed_refused`, plus G04's
  `mutant_non_numeric_max_branches_flips_happy_path` /
  `mutant_duplicate_key_flips_happy_path`) — all within the 72-lib pass.

## 7. G06 did_save court — VERIFIED-DONE

`crates/branchless-lsp/tests/did_save_court.rs` — 4 passed / 0 failed
in the suite run (section 1). BRANCHY fixture pins `g06_branchy`
(did_save_court.rs:33); assertions include message contains
`g06_branchy` (:117) and code == ANDON-REFUSED-001. The G06
spawn_blocking hunk is live in the shared-tree source
(`crates/branchless-lsp/src/lib.rs:186`) and compiled into the fresh
release binary rebuilt for section 2. Suites riscv32imc (3) and e9
(6) mutation courts also green — anti-vacuity teeth present
(`riscv32imc_opcode_mutation_flips_the_court`,
`e9_opcode_mutation_flips_classification`).

## 8. Verdict table

| Area | Verdict |
|---|---|
| Full suite (90 passed) | VERIFIED-DONE |
| Rebuilt binary ANDON_REFUSED literal (=3) | VERIFIED-DONE |
| WG2 live-wire branchy/clean flip | VERIFIED-DONE |
| Client wait_for_diagnostics masking bug | DEFECT-OPEN |
| G07 seal + zero-alloc court | VERIFIED-DONE |
| G04 parse_attrs key-value form | VERIFIED-DONE |
| G06 did_save court | VERIFIED-DONE |

## 9. Fences touched

`cargo build/test -p branchless-lsp` and `cargo build --release` were
run in the shared lsp-max tree (audit is read-only for source; cargo
wrote target/ artifacts only). No source, python, or cgap-tree files
modified by this lane. No git commands run.

## 10. Divergences

1. The release binary G05 cited was stale relative to post-12:53
   source edits (G06 lib.rs hunk, G07 registration); rebuilt fresh
   before the literal count and wire probe. This is an mtime-freshness
   correction, not a defect in G05's witness (its binary matched the
   then-current source).
2. G05's receipt cites `src/rule_pack.rs`; actual path
   `crates/branchless-lsp/src/rule_pack.rs` (already a recorded G05
   divergence — confirmed, not re-litigated).
3. Wire codeAction remains UNVERIFIED (client lacks the request
   method; G05 receipt:48-49, :68) — carried forward, unchanged.
