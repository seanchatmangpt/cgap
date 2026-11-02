# bp-5-skills receipt

Lane: BP-5 SKILLS · 2026-11-01 · repo /Users/sac/cgap (canonical checkout).

## Files written (all lane-leased, all new)

- .claude/skills/release/SKILL.md (62 lines)
- .claude/skills/run-gates/SKILL.md (66 lines)
- .claude/skills/bench-chains/SKILL.md (53 lines)
- docs/jira/v26.11.2/waves/bp-5-skills.md (this receipt)

.claude/skills/ did not exist at lane start (ls exit 1); all three skills
are new, nothing was overwritten.

## Procedure provenance (cited, not redefined)

- release: docs/jira/v26.11.1/waves/coord-publish-record.md (one-attempt
  law, chain order, 15 s propagation, UA, --allow-dirty disclosure);
  p3-pub-core-storage.md + p3-pub-aux-final.md (Retry-After-informed
  retries, slide => STOP, +10 min sliding window observation);
  p3-api-verify-all.md (num_versions == 2 EXACTLY witness);
  f3-15-api-verify.md (registry acceptance is the API, never the local
  exit code; marker-without-ALIVE is BLOCKED).
- run-gates: docs/jira/v26.11.2/01-release-gates.md G1-G8 blocks verbatim
  pointers + expected exits/counts; exit-code contract from repo CLAUDE.md.
- bench-chains: crates/bench/Cargo.toml lines 79-97 (five [[bench]]
  targets); P3-16/P3-17 receipts (bench-chains-ingress.md,
  bench-chains-parquet.md); 02-pareto-levers.md (receipt coordinates +
  measurement method); AllocProfiler pattern from parquet.rs:29-34 and
  chains-parquet.rs:43-46.

## Commands + exits (this session)

- ls .claude/skills -> exit 1 (absent; created fresh) | witnessed above.
- awk 'length>100' over all three SKILL.md -> no output, all lines <=100
  chars (first draft of run-gates exceeded on 8 table lines; rewritten as
  definition list before landing).
- python3 -c "frontmatter check": release + run-gates carry
  disable-model-invocation: true (side effects: publish uploads, gate
  mutations). bench-chains does not (read-only measurement; runs only
  cargo bench into a /tmp target lease).

## Standing

PARTIAL_ALIVE. The three skills are on-disk witnesses to real procedures
(provenance receipts listed above were themselves executed in v26.11.1
with exits recorded). No court applies to a docs/skills lane beyond the
line-width and frontmatter checks above; the skills' own procedures are
courts and remain the authority. UNVERIFIED residue: none of the three
skills has been exercised end-to-end as a skill invocation yet.

## Divergences

- Task said "the five gates G1-G5" in run-gates; the source doc
  01-release-gates.md defines G1-G8. The doc yields; the skill points at
  G1-G8 and records this divergence.
- Task named chains-* "five benches"; confirmed exactly five in
  crates/bench/Cargo.toml (ingress, parquet, bn3, seal, sovereign).
