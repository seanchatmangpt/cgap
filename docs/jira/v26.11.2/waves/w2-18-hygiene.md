# W2-18: hygiene census (gate-runner, read-only except receipt)

Date: 2026-11-01. Repo: /Users/sac/cgap, branch main @ 6801affa.
Commands (this session): `git status --porcelain` (56 entries, exit 0),
`git diff` credential sweep (exit 0, zero hits), receipt greps (exit 0),
`ls /tmp` census (exit 0).

## 1. Working-tree classification

| File(s) | Status | Owner | Evidence |
|---|---|---|---|
| Cargo.toml (license-file + version 26.11.2) | M | coordinator | diff: license-file, version bump |
| Cargo.lock | M | coordinator (byproduct of version bump) | 138-line lock churn |
| crates/*/Cargo.toml (14 files) + ontologies/{cloud,fibo}/Cargo.toml | M | coordinator | license/repository/workspace metadata additions |
| crates/cgap-gpu/Cargo.toml | M | coordinator | `publish = true` |
| crates/integration/Cargo.toml, crates/wasm-footprint/Cargo.toml | M | coordinator | integration version-pin change + wasm-footprint metadata |
| scripts/release_check.py | M | W1-16 (release pin repair) | w1-16 receipt names it sole write |
| docs/jira/v26.11.2/00-ard-prd.md | M | W1-07 | receipt files_written |
| docs/jira.v26.11.2/03-ost-design-around-register.md | M | W2-07 | w2-07 receipt files_written |
| docs/jira/v26.11.2/05-wave-plan.md | M | W1-08 | receipt "Files written" |
| docs/jira/v26.11.2/OVERNIGHT-LOOP.md | M | W1-09 | receipt sole-write claim |
| docs/jira/v26.11.2/RESTART-NOW.md | M | W1-10 | receipt files_written |
| docs/jira/v26.11.2/{06-bench-protocol,07-court-corpus,08-registry-sweep,09-overnight-mutation-set}.md | ?? | W1-11/W1-12/W1-13/W1-14 | receipt files_written |
| docs/jira/v26.11.2/HANDOFF.md | ?? | W1-06 | receipt "Sole writes" |
| docs/jira/v26.11.2/{GOAL-LOOP,OVERNIGHT-PROMPT}.md | ?? | coordinator | no lane receipt claims them |
| docs/jira/v26.11.2/patent/ + w1-19 receipt | ?? | W1-19 | receipt files_written (patent/claims.txt) |
| docs/jira/v26.11.2/research/ | ?? | W1-15 | receipt files_written (research/INDEX.md) |
| docs/jira/v26.11.2/plans/ | ?? | coordinator (planning artifacts) | no lane receipt claims; HANDOFF/GOAL-LOOP reference planning docs |
| docs/jira/v26.11.2/waves/bp-7-stardf-claude.md | ?? | coordinator | referenced (not owned) by HANDOFF.md:83 |
| docs/jira/v26.11.2/waves/w1-*.md (16 receipts) + bp waves | ?? | each W1 lane | self-receipts |
| scripts/receipt_lint.py | ?? | UNOWNED — no receipt claims it | grep over waves/ + *.md: no owner |
| scripts/registry_sweep.py | ?? | UNOWNED — referenced by w1-13 receipt commands, never claimed as written | w1-13 files_written excludes it |
| tests/court_verdict.rs | M | UNOWNED MUTATION — live anti-vacuity mutation left in tree | see finding F1 |

## 2. Finding F1 (critical): un-restored mutation in shared tree

`tests/court_verdict.rs:294` — K256 constant mutated in place:
`0x391c0cb3 → 0x391C0CB4` (uppercase + value flip). This is a live
anti-vacuity mutation that was never restored. Consequence: `cargo test
-p cgap-court` fails on this tree (court_verdict.rs asserts the pinned
K256 table). No W1/W2 receipt claims this write. Per lane contract
(read-only except receipt) I did NOT restore it — coordinator action
required: restore line 294 to `0x391c0cb3` (note mtime-bump restore
law: bump mtimes after restore).

Also the w2-09 receipt (w2-09-build-baseline.md:15) cites `cargo test
-p cgap-court` exit 0 / 14 passed on /tmp/w2-09-target — that run
predates this mutation or ran a pre-mutation tree; as of this session
the mutation is present, so w2-09's green receipt is stale on this
subject.

## 3. Credential sweep (report-only)

`git diff | grep -iE '(token|secret|password|api[_-]?key|passwd|credential|AKIA|BEGIN [A-Z]+ PRIVATE KEY|ghp_|sk-)'`
→ zero hits across the entire modified-file diff surface (26 tracked
files). Untracked files not swept byte-for-byte; no filename pattern
suggests credential material. CLEAN.

## 4. /tmp lease debris census (report-only; none deleted)

47 `*-target` dirs in /tmp plus non-target debris. W1/W2-leased:

| Lease | Size |
|---|---|
| /tmp/w1-lane7-target | 294M |
| /tmp/w2-l14-target | 1.0G |
| /tmp/w2-15 | 80K |
| + /tmp/w1-02-*.log/txt, w1-16-relcheck.out, w1-lane12-*, w1-msg.txt, w2-1-* (poll scripts, blake3 jsons) | small |

Non-W1/W2 leases observed (NOT mine, not deleted): a6-, avec-lane18-,
b1/b3/b4-, c11-, cd2/cd3-, cgap-audit-A01-, cgap-court-cli-,
coord-publish-, doc4-, doe1-, g3-, gap-3-, l14-, lane1-n3-, p311-,
pa6/pa7/pa8-, purrdf-*, r2m/r3/r13/r17-, s4-, stardf-* (excluded per
contract — /Users/sac/stardf untouched), v2-, v7-, vac1..vac5-, w29-,
wh6-target. Per the lease law these belong to other lanes/repos;
deletion is a coordinator integration transition, not a lane action.

## 5. Lease spot-check (5 receipts vs git status)

- W1-01 (ontology audit): claimed w1-01 receipt only — matches (report-only lane).
- W1-02 (cli regen): claimed receipt only; crates/cli generated sources unchanged in status — matches.
- W1-13 (registry sweep): claimed 08-registry-sweep.md + receipt — matches; registry_sweep.py left unclaimed (F2).
- W1-16 (release pin): claimed scripts/release_check.py — matches.
- W1-19 (patent claims): claimed patent/claims.txt + receipt — matches.
- W2-07: claimed 03-ost + receipt — matches.

No lane wrote outside its claimed lease except the two UNOWNED items
(scripts/receipt_lint.py, scripts/registry_sweep.py) and finding F1.

## Verdict

PARTIAL_ALIVE: census complete, credential sweep CLEAN, no lane
lease-violation among spot-checked receipts. Two unowned artifacts
(receipt_lint.py, registry_sweep.py) and one un-restored mutation
(tests/court_verdict.rs:294) are coordinator action items.

```
{"lane":"W2-18","files_written":["docs/jira/v26.11.2/waves/w2-18-hygiene.md"],
"commands":["git status --porcelain (exit 0, 56 entries)",
"git diff credential sweep (exit 0, 0 hits)",
"receipt files_written greps (exit 0)",
"ls/du /tmp census (exit 0, 47 target dirs)"],
"courts_run":[],"standing":"PARTIAL_ALIVE",
"findings":["F1 tests/court_verdict.rs:294 un-restored K256 mutation 0x391c0cb3->0x391C0CB4, unowned, breaks cargo test -p cgap-court",
"F2 scripts/receipt_lint.py and scripts/registry_sweep.py unowned (no receipt claims)",
"F3 w2-09 green cgap-court receipt stale on mutated subject"],
"divergences":[],"lease":"/tmp none created by this lane"}
