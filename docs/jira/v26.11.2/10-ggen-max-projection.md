# 10 — GGEN-max projection: roadmap to >95% machinery share

Lane G1K-5 (Explore; artifacts landed coordinator-side). Baseline at v26.11.2 (receipt
`waves/g1k-05-machinery-share-audit.md`). Method: projected bytes / total tracked bytes,
both via `git ls-files | xargs wc -c`.

## Baseline (measured 2026-11-01)

- Total tracked: 903 files, 9,156,129 bytes.
- GGEN-projected: 73 files, 109,885 bytes = 1.20% machinery share.
- Largest residue: docs/jira prose 3,162,488 B (34.5%); hand-written crate rs ~4,078,900 B;
  scripts gates 590,104 B.
- Correction to the lane's report: the live bench surface is
  `crates/bench/benches/` (15 tracked files incl. the five chains-* benches); the repo-root
  `benches/` is an empty 0-file directory (the lane's `git ls-files` census counted the root
  dir).

## Ladder to >95%

1. Manifest facts in graph [TARGET-v26.11.2] (G1K-1): every Cargo.toml fact (name,
   version, deps, publish flag) becomes an RDF individual; Cargo.tomls become projections.
   Removes the hand-maintained-manifest drift class this session witnessed (25-manifest
   license/repository repair).
2. Court individuals [TARGET-v26.11.2] (G1K-2): each Chicago court named + typed in the
   graph; the court table projects from the graph, drift becomes a typed refusal.
3. Campaign board [TARGET-v26.11.2] (G1K-3): campaign docs + waves receipts as graph
   individuals rendered to markdown; re-measurement flips `p:measuredNs` in the graph,
   never prose. This alone moves share ~34%.
4. Bench registry [TARGET-v26.11.2] (G1K-4): the bench surface (crates/bench/benches,
   15 files) registered as p:Bench individuals with witness-pair gates.
5. Named-next [TARGET-v26.11.2]: (a) waves receipts as graph individuals (docs/jira/**/
   waves/**, 414 files); (b) gate scripts (scripts/*.py|sh, 590,104 B) projected from gate
   specs; (c) dispatch/work-orders as RDF rendered to agent prompts.

## End state

Hand-written residue confined to the five kernel hot paths
(`crates/cgap-{arrow,blake3,git,parquet}` src + bont scalars, CLAUDE.md:48-51) plus
`crates/cli/src/custom_handlers.rs` and its `custom_handlers/` modules (36,920 B — the sole
cnv:CustomBehavior seam, crates/cli/src/generated_cli.rs:4-8). Everything else is a
deterministic projection with byte-identical replay and typed drift refusals.

## Non-goals

- Never hand-edit generated files; regen law CLAUDE.md:59-66.
- Court contract unchanged: exit 0/1/2/3, anti-vacuity mutation per court (CLAUDE.md:24-26,
  :81-83).

## Footer

{"lane":"G1K-5","files_written":["docs/jira/v26.11.2/10-ggen-max-projection.md"],
"commands":["git ls-files census (agent session)","wc -c class totals (agent session)"],
"courts_run":[],"standing":"ALIVE",
"divergences":["landed coordinator-side (Explore lane has no write tools)",
"benches/ correction: live surface is crates/bench/benches, root benches/ is empty"]}
