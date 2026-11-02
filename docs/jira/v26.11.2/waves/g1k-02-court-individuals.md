# G1K-02 Courts as Graph Individuals — Receipt

- **Lane**: G1K-2 (lane-runner, "1000x ggen")
- **Date**: 2026-11-02
- **Repos**: /Users/sac/ggen-marketplace/packs/cgap-pack + /Users/sac/cgap
- **Files written** (both repos, no git commands run):
  - packs/cgap-pack/ontology.ttl — `p:Court` class + `p:courtId`,
    `p:courtSubject`, `p:courtMutation`, `p:courtKillExit`,
    `p:courtReceipt`; 13 individuals for the §5 flip-summary rows
  - packs/cgap-pack/templates/court-table-md.tmpl — projects
    `generated/court-table.md` (SPARQL rows, `ORDER BY ?id`)
  - packs/cgap-pack/gates/120_court_subject_regex.rq — courtSubject
    must match `^crates/[^ ]+:[0-9]+(-[0-9]+)?$` or `^scripts/` or
    `^G[0-9]-`; invented subjects refuse
  - packs/cgap-pack/witnesses/{pass,fail}/120_court_subject_regex.ttl
- **Standing**: ALIVE (every claim below ran this session with exits)

## Commands and exits (2026-11-02, this session)

| Command | Exit | Result |
|---|---|---|
| `python3 scripts/check_gate_witness_courts.py --packs packs` (ggen-marketplace) | 0 | cgap-pack ALIVE, case_count 13 (12 prior + 120) |
| `ggen sync run` (root /Users/sac/cgap, run 1) | 0 | all projections |
| `ggen sync run` (run 2) + `sha256sum -c` on generated/court-table.md | 0 | byte-identical (OK) |
| `make gate0` (root) | 0 | byte-stable replay PASS + C header compile PASS |
| Sandbox baseline sync (/tmp/g1k2-sandbox, gate 120 active) | 0 | silent |
| Sandbox mutant: subject -> "bench kernels L1-L5" | 1 | `gate 120_court_subject_regex.rq refused the sync` |
| Sandbox restore (byte-exact, mtime bumped, re-locked) | 0 | `RESTORE-BYTE-EXACT` (diff clean) |

## Court

Anti-vacuity mutation: replace L1 courtSubject with the invented string
"bench kernels L1-L5" in the /tmp/g1k2-sandbox pack copy. Gate 120 fires
(exit 1, refusal names the gate); restore is sha/diff byte-exact against
the shared pack, mtime bumped per the stale-fingerprint law. First mutant
attempt exit 1 was FM-PACK-008 lock mismatch, not the gate — vacuous
flip caught and redone with per-step re-lock; recorded here per
no-fake-pass discipline.

## Differential vs 07-court-corpus.md §5 (doc loses, graph wins)

13/13 rows present; 11 subject-column drifts, all grounded to the
2026-11-01 on-disk tree:
- L1-L4: doc subjects omit the `crates/` prefix (files verified at
  crates/cgap-{gate,parquet,bn3,ffi}); graph normalizes, gate-lawful.
- L5: doc has no line cite; graph cites asc450.rs:328 (`pub fn verify`,
  on-disk verified).
- G2/G3/G4/G8: doc subjects are non-path descriptions ("bench kernels
  L1-L5" etc.); graph uses the `^G[0-9]-` escape-hatch form.
- G6: doc "datafusion grep gate" -> graph grounds to the real script
  `scripts/zero_datafusion_gate.py` (exists on disk).
- G7: doc names `scripts/verify_ost_design_around.py`, which DOES NOT
  EXIST on disk — graph keeps the escape-hatch form `G7-FLIP:
  verify_ost_design_around.py`. DRIFT FINDING: the doc's G7 subject is
  ungrounded; the named script is absent from cgap scripts/.

## Divergences / disclosures

- The pack-template discovery law (ggen 26.10.11 `discover_templates`)
  only self-projects frontmatter templates that START with `---`;
  leading `force: true` makes the template rules-consumed and invisible.
  Initial court-table-md.tmpl and (another lane's) bench-registry and
  lever-board templates hit this; fixed by leading with `---` and moving
  `force: true` into the frontmatter. Disclosed here for both lanes.
- Lane 130/140 gates briefly refused the root sync (in-flight lever
  statuses); unblocked by their owner within the SLA window; my gate 120
  ordering after 100 was unaffected.
- Sandbox court set aside the other lanes' in-flight 130/140 gates to
  establish a green baseline; restored byte-exact afterward.
- ggen.lock in the consuming project pins pack content hash: any pack
  edit requires lock deletion (intentional re-lock) or FM-PACK-008 —
  relevant to every lane editing cgap-pack.

## Standing

**ALIVE** — command + exit for every claim in the tables above; mutation
court flips; restores byte-exact; /Users/sac/stardf untouched; no git
write commands.
