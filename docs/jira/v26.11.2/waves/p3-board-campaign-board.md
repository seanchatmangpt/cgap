# p3-board — Campaign Board Flip from the Graph

Lane: p3-board (ggen surface). Subject: exact on-disk
/Users/sac/cgap + /Users/sac/ggen-marketplace/packs/cgap-pack.
Contract: no git writes; publishes frozen; no new crates; naming law;
exit contract 0/1/2/3; standing vocabulary; <=100-char lines.

## Bounded poll (p1/p2, 60 min, 60s cadence on receipt FILES)

Poll started 23:59 PST, receipt-file diff poll (bounded, deadline
checked). Landed set at first check: p1-blake3-optlevel-ub-fix.md,
p1-repair-unify-delim.md, p1-repair-unify-repair.md,
p2-footprint-flash-ladder.md, p2-penstock-escalation-law.md.
Poll witnessed 3 additional p1 receipts land mid-poll
(p1-closure-closure.md, p1-dred-retraction-antijoin.md,
p1-lineage-workflow.md — none measure the five Pareto levers, so no
further measuredNs update). Poll stopped early: all p1/p2 lanes
landed, L3 evidence fixed.

## L3 flip (from the graph, never hand-edited generated output)

Evidence receipt: `waves/p1-repair-unify-repair.md` (ALIVE: cargo
build/test -p cgap-bn3 exit 0, 119 unit + 7/7 lever courts incl.
`l3_antivacuity_probe_drop_flips` — the named-mutation court flips).
Flip in `ggen-marketplace/packs/cgap-pack/ontology.ttl`:

- `p:leverStatus` -> `LANDED-waves/p1-repair-unify-repair.md
  (semi-join landed in unify.rs; 7 lever courts exit 0 incl.
  l3_antivacuity_probe_drop_flips; perf re-measure pending)`
- `p:mutated` -> `true` (court flipped per receipt)
- `p:measuredNs` -> `1470000000.0` (P3-18 baseline, 1.470 s @ 1M —
  same baseline convention as L1/L4/L5 measuredNs; semi-join perf
  re-measure remains pending)
- removed `p:skipReason "unmeasured-this-wave"`

## Naming-law fix (pre-existing, in-lane file)

`grep -ic` sweep found a pre-existing lowercase violation in the pack
graph: ontology.ttl:616 `p:courtSubject "G8-FLIP:purrdf stardf
scripts"` — replaced with "purrdf campaign-graph scripts". CamelCase
"starDF" (campaign proper name, used by 00-ard-prd.md and the board
title) remains; only the lowercase ecosystem word is banned.

## Sync + gates

- FM-PACK-008 re-lock: `rm ggen.lock` + `ggen sync run` — exit 0
  (intentional re-lock after graph edits, remediation path named in
  FM-PACK-008; ggen.lock delta is the pack blake3 hash line only).
- Root sync x2 — exit 0 both; second run wrote nothing new.
- `generated/campaign-board.md` byte-stable: `cmp` run1 vs run2 —
  PASS, sha256 74ee922303635ee0...f890767f identical across syncs
  and across the final gate0 replay.
- `make gate0` (CARGO_TARGET_DIR=/tmp/p3-board-target) — exit 0:
  byte-stable replay proof PASS + freestanding header compile PASS.
- Board row now: `| L3-bn3-plnf-semi-join |
  unbounded-until-measured | 1470000000 ns | LANDED-waves/
  p1-repair-unify-repair.md (...) | unify.rs:330-342 |`

## Contract checks

Zero `Process.sleep` in any gate; no git writes; no new crates; no
DataFusion; frozen pins untouched; /tmp lane target removed at lane
end; receipts consumed, not fabricated — L3 evidence is the p1-repair
receipt's witnessed exits, not this lane's claim.

```json
{"lane":"p3-board","files_written":["/Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl","/Users/sac/cgap/ggen.lock","/Users/sac/cgap/generated/campaign-board.md","/Users/sac/cgap/docs/jira/v26.11.2/waves/p3-board-campaign-board.md"],"commands":["ggen sync run x2 after re-lock -> exit 0 x2","cmp board run1 vs run2 -> byte-stable PASS (sha256 74ee9223...)","CARGO_TARGET_DIR=/tmp/p3-board-target make gate0 -> exit 0 (replay cmp PASS + header compile PASS)","grep -ic naming-law sweep on ontology.ttl + campaign-board.md -> lowercase word 0 after fix"],"courts_run":["gate0 byte-stable replay cmp","gate0 freestanding C header compile","bounded receipt-file poll (p1/p2 set diff, 60s cadence, early-stop on landed set)"],"standing":"ALIVE","divergences":["L3 LANDED carries perf re-measure pending: p:measuredNs set to the P3-18 nested-loop baseline (1.470 s @ 1M) under the L1/L4/L5 baseline convention; semi-join chain re-measure is an open close condition","FM-PACK-008 intentional re-lock required (pack graph edited after prior lock)","pre-existing lowercase naming-law violation fixed in ontology.ttl:616 (courtSubject string); camelCase starDF retained as the campaign proper name"]}```
