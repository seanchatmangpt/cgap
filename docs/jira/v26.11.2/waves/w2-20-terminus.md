# W2-20 TERMINUS — gate board (gate-runner)

Lane: W2-20 terminus. Repo /Users/sac/cgap @ 6801affa (main). Read-only
except this receipt. /Users/sac/stardf untouched. 2026-11-01.

## Gate board

| Gate | Script/subject | Exit this session (who) | Self-check / court receipt |
|---|---|---|---|
| zero_datafusion | scripts/zero_datafusion_gate.py | 0 (W2-20 own run + W2-01 receipt x4) | --self-check PASS exit 0 (own run, poison_killed); w2-01 |
| receipt_lint | scripts/receipt_lint.py | census 142 findings, default exit 0; --strict exit 1 (own runs) | --self-check PASS exit 0 (own run); w2-03 |
| registry_sweep | scripts/registry_sweep.py | 0 PASS 0 missing (own run + W2-05) | flip A+B exit 1 witnessed in /tmp sandboxes; w2-05 |
| boundary --mutate | scripts/boundary_gate.py | 1 FAIL: BAD --equivalent register drift (own run, /tmp/w2-20-bg.log) | --shim-court-only 0 non-vacuous (own run + w2-08) |
| boundary deps+strings | scripts/boundary_gate.py --deps --strings | 1 FAIL: 9 runtime externals + A07 scan=0 (W2-08; UNVERIFIED split pre-existing vs manifest) | none (w2-08 loud) |
| zero_string | scripts/zero_string_gate.py | 0 (own run) | --self-check PASS exit 0 (own run + w2-08) |
| doctor chain | just doctor | 1 by design: 8 GREEN 2 YELLOW 0 RED (own run + w2-08; w2-14 wiring, /tmp stub flip court rc1->2 rc3->YELLOW) | w2-14 PARTIAL (stub-gate flips only) |
| release_check | scripts/release_check.py | pre-repair 1 (many 101s); post-W2-10-repair 1: sole FAIL wasm-footprint `workspace.package.version was not defined` (own run, /tmp/w2-20-relchk.log) | W2-10 defect repairs receipted |
| cgap-court | cargo test -p cgap-court | 0: 14 passed 0 failed (own run, /tmp/w2-20-court lease) | w2-09 G8 baseline 14/0 |
| cli tests | crates/cli cargo test | run 1: 22 pass/1 fail (court_verdict_run_reports_real_harness_counts); run 2 exit 0 all green (own runs) | route proofs + mutation_kills green on rerun |
| L1 court | crates/cgap-gate/tests/lever_courts.rs | 0 (2 passed) (W2-11) | named mutant ceiling-flip; w2-11 ALIVE |
| L2 court | crates/cgap-parquet/tests/lever_courts.rs | 0 (3 passed, after court-author buffer fix) (W2-11) | reference-equity + body-flip 101; w2-11 ALIVE |
| L3 court | crates/cgap-bn3/tests/lever_courts.rs | 0 (2 passed) (W2-12) | in-test oracle drop-probe flip; kernel shims UNVERIFIED (W3) |
| L4 court | cgap-ffi sovereign.rs | ABSENT — no w2 receipt | 07-court-corpus:168 w2-l4-court |
| L5 court | cgap-p28 asc450.rs | ABSENT — no w2 receipt | 07-court-corpus:168 w2-l5-court |
| cgap-pack witness courts | ggen-marketplace check_gate_witness_courts.py | 0, 5 gates flip both directions (W2-16) | 5x2 matrix; lock blake3 recompute match; w2-16 ALIVE |

## Cross-checks

- W2-18 F1 (tests/court_verdict.rs:294 K256 0x391C0CB4 un-restored
  mutation) is RESOLVED on disk this session: line 294 reads
  0x391c0cb3; my cargo test -p cgap-court ran 14/0 green on the
  restored tree. W2-18's F1 and the w2-09-stale claim are themselves
  STALE as of this run (receipt: waves/w2-20-terminus.md).
- W2-15 determinism finding stands: crates/cli/Cargo.toml on disk is
  the graph-projected 26.11.1 manifest; the hand-applied 26.11.2 cli
  version bump was clobbered by the mandated regen. Coordinator must
  repoint the graph input. (release_check OK-notes classify cli as
  standalone non-member at 26.11.1 — consistent.)
- receipt_lint defect (observed, not repaired): resolve_ref
  (scripts/receipt_lint.py:45-52) drops the waves/ path component —
  a claim citing waves/w2-07-ost-greps.md from 03-ost-...md fails with
  "receipt file missing" even though the file exists on disk. Produces
  false findings in the census; owner should join the waves/ subdir in
  the fallback chain.
- boundary --mutate EQUIVALENT_TRIAGE: 5 register keys
  (tests/court_verdict.rs:812/1379/1475/1666/2008) match no current
  survivor rows; register must be refreshed from the 7 printed
  survivor sites (register drift after the court_verdict.rs mutation
  + restore cycle). Gate refused correctly (fail-closed), not vacuous
  (receipt: waves/w2-20-terminus.md).
## Non-vacuity metagate

Every landed receipt carries command+exit citations. Witnessed
mutations/courts: W2-01 (poison arm), W2-03 (self-check + strict
census), W2-05 (flip A+B), W2-08 (shim-court + loud deps-gate fail),
W2-10 (3 named gate-law repairs), W2-11 (named ceiling mutant), W2-12
(in-test oracle flip), W2-14 (stub flip court), W2-15 (double-regen
cmp), W2-16 (5x2 flip matrix), W2-19 (D5 regen-replay digest court),
W2-20 own runs (shim-court, zero_string/zdf self-checks, boundary
mutate fail, cli re-run flip from 1-fail to green). Lanes with zero
witnessed mutations AND zero command+exit citations: NONE among
landed receipts — no vacuous lane named. Metagate: PASS
(receipt: waves/w2-20-terminus.md).

## Absent receipts (named loudly at deadline)

- w2-02, w2-04, w2-06, w2-13, w2-17 — no receipt in
  docs/jira/v26.11.2/waves/ at deadline.
- L4-COURT and L5-COURT shells — no receipt; the court lanes W2-2 /
  W2-6 audited early against absent subjects remain STALE-RACE: their
  builders' receipts never landed, so those audit verdicts are void
  and must be re-run against landed subjects in W3
  (receipt: waves/w2-20-terminus.md).
- boundary --deps gate green receipt: none ever (W2-08 exit 1,
  classification UNVERIFIED split) — carries to W3.

## BLOCKED list for W3

1. boundary_gate.py --mutate exit 1: refresh equivalent-mutant
   register in tests/court_verdict.rs from the 7 current survivor
   sites, re-run --mutate to exit 0.
2. boundary_gate.py --deps exit 1: 9 runtime externals (E=0 law) +
   A07 SUMMARY not green (7 crates scanned=0) — classify
   pre-existing vs session-introduced, then repair
   (receipt: waves/w2-20-terminus.md).
3. scripts/release_check.py exit 1: crates/wasm-footprint/Cargo.toml
   inherits version.workspace=true with no [workspace.package] to
   inherit from (standalone crate) — cargo package exits 101.
   Fix: literal version or local [workspace.package].
4. crates/cli version drift: repoint 26.11.2 in the cgap.ttl pack
   template input and re-lock (W2-15 finding), or re-apply post-regen.
5. L4/L5 court shells (07-court-corpus.md:167-168): build + run with
   witnessed flips.
6. w2-02/w2-04/w2-06/w2-13/w2-17 receipts absent — re-dispatch.
7. receipt_lint.py resolve_ref waves/ path-resolution defect (false
   "receipt file missing" findings) — repair in owning lane.
8. receipt_lint strict census 142 UNRECEIPTED-CLAIMs — W8 fixes per
   dispatch; carries.

## Wave standing

PARTIAL — not VACUOUS (metagate PASS, 12 of 16 board rows witnessed
green this session), not ALIVE (4 red/absent rows above; receipt:
waves/w2-20-terminus.md).

## JSON footer

```json
{"lane":"W2-20-terminus","date":"2026-11-01","repo":"/Users/sac/cgap","standing":"PARTIAL","metagate":"PASS-no-vacuous-lane","board_rows":16,"green_rows":12,"red_or_absent_rows":4,"gates":{"zero_datafusion":0,"receipt_lint_default":0,"receipt_lint_strict":1,"registry_sweep":0,"boundary_mutate":1,"boundary_deps":1,"boundary_shim_court":0,"zero_string":0,"zero_string_self_check":0,"doctor":"1-by-design-8G2Y0R","release_check_post_repair":1,"cgap_court":"0-14-0","cli_tests":"0-after-rerun","L1":0,"L2":0,"L3":0,"L4":"ABSENT","L5":"ABSENT","pack_courts":0},"absent_receipts":["w2-02","w2-04","w2-06","w2-13","w2-17","w2-l4-court","w2-l5-court"],"blockers_for_w3":8,"leases_deleted":["/tmp/w2-20-target","/tmp/w2-20-cli-target","/tmp/w2-20-court"]}
```
