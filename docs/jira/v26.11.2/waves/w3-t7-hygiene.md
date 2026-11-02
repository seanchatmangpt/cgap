# W3-T7 — Hygiene Window Two (gate-runner census)

Date: 2026-11-01. Repo: /Users/sac/cgap (canonical; read-only except this receipt).
Never touched /Users/sac/stardf.

## Census commands

| Check | Command | Exit | Evidence |
|---|---|---|---|
| git census | `git status --porcelain` | 0 | 52 tracked-modified + 70 untracked paths |
| credential sweep (tracked diff) | `git diff HEAD | grep -iE 'AKIA.../ghp_/sk-/xox/gsk_/PRIVATE KEY/api_key='` | grep 1 (no matches) | zero hits over 2459 insertions |
| credential sweep (untracked) | `grep -ilE ...` over `git ls-files --others` | 1 (no matches) | zero hits |
| frozen-pin tripwire (license URN) | grep over *.rs/*.ttl/*.py/*.toml/*.md | 0 | all sites pre-existing lawful (see findings) |
| seed-conflation spot-check | grep both constants over *.rs | 0 | bitslice.rs:687-699 pair INTACT |

## Classification table (content-attributed; receipts only on disk for R3/R7)

Only two W3 lane receipts exist at census time: w3-r3-court-reruns.md (files_written:
its own receipt only) and w3-r7-template-frontmatter.md (ggen-marketplace templates +
ggen.lock — matches `M ggen.lock`). All other attribution is by diff content signature.

| Files | Class | Owning lane (attributed) |
|---|---|---|
| Cargo.lock, Cargo.toml, 17x crates/*/Cargo.toml + ontologies/*/Cargo.toml, crates/cgap-verification/* | manifest/version alignment | coordinator (commit-manifest lane, v2-13) |
| crates/bench/benches/{chains-ingress,chains-seal,ingress}.rs, crates/bench/benches/mat_closure.rs | bench corpus doc + new mat_closure bench | W3 lever bench lane (B-lane family) |
| crates/cgap-arrow/src/slab.rs | FrameView prefilter/escalate fields + wave9 test fixup | W3-L? lever lane (arrow ingress lever) |
| crates/cgap-bn3/src/unify.rs | JoinProbeScratch + radix semi-join (L3 lever) | W3-L3 |
| crates/cgap-bn3/tests/lever_courts.rs, crates/cgap-gate/tests/lever_courts.rs, crates/cgap-{ffi,parquet,p28}/tests/lever_courts.rs | new lever courts, seed-law headers | W3 R-lanes (court authors) |
| crates/cgap-gate/src/penstock.rs | seed-law doc comment at :669-670 | W3 lever lane (gate) |
| crates/cgap-parquet/src/{footer,mmap_writer}.rs | (unclassified detail) | W3 lever lane (parquet) |
| crates/cgap-p28/src/asc450.rs, crates/cgap-ffi/src/sovereign.rs | lever code | W3-L? / W3-R? lever lanes |
| crates/stardf-datalog/ | NEW crate (src/{lib,engine,seed}.rs, tests/oracle_courts.rs) | W3-T lane (stardf-datalog T-lane) |
| crates/cli/{Cargo.lock,Cargo.toml,ontology/cgap-cli.ttl,.clap-noun-verb/ocel.json,src/bin/doctor.rs} | CLI regen + doctor c02/c04/c06/c07/c08 extensions | W3 T-lane (cli) / w1-05 doctor extension |
| crates/cli/Cargo.lock/Cargo.toml, ggen.lock | ggen regen artifacts | w3-r7 (ggen.lock), coordinator (cli manifest) |
| docs/jira/v26.11.2/{00..10,GOAL-LOOP,HANDOFF,OVERNIGHT-PROMPT,RESTART-NOW,OVERNIGHT-LOOP,05-wave-plan,waves/bp-10,sm-4-hooks,v2-12,v2-13} | campaign docs | coordinator / doc lanes |
| docs/jira/v26.11.2/waves/g1k-0*.md, waves/registry-sweep-table.json, generated/* | G1K outputs | G1K lanes 01-05 |
| docs/jira/v26.1.2 waves w1-*, w2-*, w3-r3, w3-r7 | receipts | their lanes |
| justfile, scripts/{boundary_gate.py,release_check.py,receipt_lint.py,registry_sweep.py,zero_datafusion_gate.py} | gate/tool edits | coordinator / R3 (registry_sweep routing repair-owed per w3-r3 receipt) |
| crates/cgap-core/tests/cross_lane_gate_court.rs | lever court | W3 R-lane |

## Findings

1. **Attribution gap (structural, not a defect)**: 18 of 20 W3 lanes (L1-L5, R1-R7 minus
   R3/R7, T1-T6) have NOT yet filed receipts. Every un-receipted modification is
   content-attributed only; cross-check against files_written was possible for exactly
   2/52 tracked files. The two receipted lanes are clean: R3's files_written matches
   (receipt only), R7's matches (ggen.lock + marketplace templates outside this repo).
2. **Credential sweep CLEAN.** Zero AKIA/PRIVATE KEY/ghp_/sk-/xox/gsk_ hits over the
   full tracked diff (2459 insertions) and all 70 untracked paths.
3. **Frozen-pin tripwire: no NEW unlawful site.** All 30+ license-URN sites are
   pre-existing lawful declarations/tests/docs. Seed-law doc comments added this window
   (penstock.rs:669-670, bn3 lever_courts.rs:15-16, stardf-datalog/src/seed.rs:4-14)
   are prose pins, not new constant use sites in computation. One finding-class item:
   stardf-datalog/src/seed.rs:10-14 defines BOTH constants as pub consts — new crate,
   new binding, carries the seed-law doc; acceptable but it is a NEW binding site.
   stardf-datalog/tests/oracle_courts.rs:213-214 asserts the pair — good.
4. **Seed-conflation: PASS.** bitslice.rs:687-699 pair intact (multiplier 0xBF58... at
   :694, spread 0xA20C... at :699; CourtRng instances all seed 0xBF58...). All other
   uses are correctly classed (corpus/spread: court_verdict.rs:136, slab.rs:1295
   W9_SEED, p28 lever_courts.rs:93-94; multiplier: all SplitMix64 mixers).
5. **/tmp lease census (report-only, none deleted).** W3 leases live: w3-l2-target
   (183 MB, mtime Nov 1 22:35), w3-l4-target (227 MB, Nov 1 22:42), w3r1-target
   (78 MB, Nov 1 22:38), w3-r7-backup (24 KB, Nov 1 22:39), avec-w3-rehearsal (Oct 31).
   Stale non-W3 debris >90 days-class: cd3-target 6.7 GB, lane1-n3-target 5.3 GB,
   cd2-target 3.3 GB (mtimes today — possibly live; NOT deleted). Total /tmp target
   debris ~21 GB across ~60 dirs. Coordinator-owned cleanup at integration only.
6. ~21 GB /tmp target debris with several touched today — flagged for the
   coordinator's integration-cleanup pass, never lane-side deletion.

## Standing

PARTIAL_ALIVE — census ran clean this session; attribution limited by receipt gap
(2/20 lanes receipted). No credential, pin, or seed finding.

<!-- json-footer
{"lane":"W3-T7","receipt":"docs/jira/v26.11.2/waves/w3-t7-hygiene.md","files_written":["docs/jira/v26.11.2/waves/w3-t7-hygiene.md"],"commands":["git status --porcelain (exit 0, 52M+70? paths)","credential grep over git diff HEAD (exit 1, zero hits)","credential grep over untracked (exit 1, zero hits)","frozen-pin + seed grep (exit 0)","/tmp lease du/ls census (report-only)"],"standing":"PARTIAL_ALIVE","findings":6,"findings_clean":["credentials","frozen-pin","seed-conflation"],"divergences":["18/20 W3 lanes unreceipted at census time; attribution content-only for 50/52 files","~21 GB /tmp target debris flagged for coordinator cleanup; nothing deleted"]}
json-footer -->
