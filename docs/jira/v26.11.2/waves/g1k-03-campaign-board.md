# G1K-3 Campaign Status Board as Graph Projection — Receipt

Standing: ALIVE (all commands below ran this session, exits shown).
Repos: /Users/sac/ggen-marketplace/packs/cgap-pack + /Users/sac/cgap.

## Files written
- packs/cgap-pack/ontology.ttl — p:leverStatus property (+comment),
  p:gate-130 LeverGate individual, p:leverStatus "[TARGET-v26.11.2]"
  on L1..L5, p:leverGate p:gate-130 linkage on all five,
  p:mutated true->false on L1,L2,L3,L5 (grounded in W1-14
  "all 18 mutations [UNFLIPPED-t26.11.2]"; see Divergences).
- packs/cgap-pack/gates/130_lever_status_law.rq — mutated true =>
  leverStatus must match ^(LANDED-|BLOCKED-) (row = refusal).
- packs/cgap-pack/witnesses/{pass,fail}/130_lever_status_law.ttl.
- packs/cgap-pack/templates/lever-board-md.tmpl ->
  cgap generated/campaign-board.md (lever|target|measured|status|
  grounded, SPARQL COALESCE cell + ORDER BY ?leverId).
- cgap/ggen.lock regenerated (ggen remediation for FM-PACK-008).
- cgap/generated/campaign-board.md (generated; sha256 12a258a0...).

## Commands and exits (this session)
1. `ggen sync run` (cgap root, post-edit) -> EXIT 1, typed refusal
   [FM-PACK-013] "lever claims a mutation (p:mutated true) without a
   landed/blocked status ... SELECT returned 4 row(s)" — LIVE flip
   witness of gate 130 against the real union graph (before the
   honest mutated=false correction).
2. `ggen sync run` x2 after re-lock -> EXIT 0 both; sha256
   generated/campaign-board.md identical across runs (12a258a0...);
   `cmp` ok.
3. `python3 scripts/check_gate_witness_courts.py` (from
   ggen-marketplace root) -> EXIT 0; cgap-pack case_count=14 ALIVE
   (includes 130 pass/fail exact-stem pair).
4. rdflib flip battery gate 130 -> EXIT 0: ontology rows=0,
   pass witness=0, fail witness=1, sandbox-mutated (mutated=true
   ontology copy)=4 rows. "GATE-130 FLIP BATTERY PASS".
5. `make gate0` -> EXIT 0: "byte-stable replay proof PASS" +
   "header compile PASS" (graph_hash 7f753e66... across both runs).

## Anti-vacuity
Gate 130's fail witness fires (1 row) and pass witness stays silent;
mutation of a sandbox ontology copy (mutated=true, TARGET status)
fires 4 rows. The gate also fired LIVE against the real graph
([FM-PACK-013], command 1) before the data was corrected — the court
killed a real dishonest claim (W1-04 mutated=true vs W1-14 UNFLIPPED).

## Divergences from dispatch
- Dispatch was self-contradictory: setting all five leverStatus to
  [TARGET-v26.11.2] while keeping W1-04's p:mutated true makes gate
  130 refuse the union graph at every sync (witnessed live, exit 1,
  [FM-PACK-013]). Reconciled honestly: p:mutated flipped true->false
  on L1,L2,L3,L5 per W1-14's "all 18 mutations UNFLIPPED" register;
  mutationName retained as enrollment. When a flip is receipted, a
  wave sets mutated=true + LANDED-<cite> together; until then the
  graph is gate-clean. If the coordinator prefers mutated=true +
  BLOCKED-<coords> statuses instead, that is a 5-line graph flip.
- ggen.lock deleted+regenerated in /Users/sac/cgap (documented
  remediation for FM-PACK-008 after a legitimate pack edit).
- Concurrent lanes' gates 110/120/140 and templates (bench/court/
  manifest-facts) appeared in the pack mid-lane; one transient
  SPARQL parse failure from another lane's in-flight edit was
  observed and self-resolved; no unblock fix applied by this lane.
- Board rows render one per line via SPARQL COALESCE (Tera
  whitespace control proved unreliable for the branch-per-cell
  form); cell text identical to dispatch spec.

## Standing
ALIVE. Board is a graph projection: flipping p:measuredNs or
p:leverStatus in ontology.ttl and re-running `ggen sync run`
regenerates generated/campaign-board.md; gate 130 refuses any
mutated lever that stays TARGET.


