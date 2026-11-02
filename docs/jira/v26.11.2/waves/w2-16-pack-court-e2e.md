# W2-16 — pack court E2E (cgap-pack W1-04 extension)

Lane: W2-16 (gate-runner). Date: 2026-11-01. Repos: /Users/sac/ggen-marketplace
(read + scripts), /Users/sac/cgap (receipt only). /Users/sac/stardf untouched.

## Verdict: ALIVE — all 5 new gates flip both directions, consumer lock relocked.

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| witness-court harness | `cd /Users/sac/ggen-marketplace && python3 scripts/check_gate_witness_courts.py` | 0 | cgap-pack: case_count=10, standing=ALIVE, require_pass/fail=true; 47 packs swept |
| exact-stem pairing | python walk (gates 060-100 vs witnesses/pass+fail) | 0 | 5/5 stems have both `<stem>.ttl` witnesses; PAIRING_OK |
| independent flip battery | rdflib 7.6.0 SPARQL SELECT per gate | 0 | see 5x2 matrix below; ALL_FLIP |
| consumer lock | blake3 fold over packs/cgap-pack per ggen pack.rs:1094-1124 (global rel-path sort, .git excluded) | 0 | recomputed 1f5b9bee...114411714036 == /Users/sac/cgap/ggen.lock [packs.cgap-pack].content_hash |

## 5x2 flip matrix (rdflib, rows = SELECT result count)

| gate .rq | pass witness rows | fail witness rows | verdict |
|---|---|---|---|
| 060_lever_identity.rq | 0 | 1 | FLIP both directions |
| 070_mutation_law.rq | 0 | 1 | FLIP both directions |
| 080_target_measured_pair.rq | 0 | 1 | FLIP both directions |
| 090_grounding_regex.rq | 0 | 1 | FLIP both directions |
| 100_gate_linkage.rq | 0 | 1 | FLIP both directions |

No VACUOUS verdicts: every gate fires on its fail witness (>=1 rows) and is
silent on its pass witness (0 rows).

## Consumer-side

`/Users/sac/cgap/ggen.lock` `[packs.cgap-pack]`:
source = path:../ggen-marketplace/packs/cgap-pack
content_hash = blake3:1f5b9bee0fc525b7d6e091674495c028033df940bc292e317937114411714036
Independently recomputed (python blake3 module, replicating
ggen-engine/src/pack.rs:1094 content_hash: rel-path-keyed fold, globally
sorted, .git skipped) -> byte-identical hex. The lock carries the relocked
hash of the W1-04-extended pack. (First attempt walked in per-directory DFS
order and MISmatched; global sort is load-bearing — recorded as method note.)

## Standing

- cgap-pack witness courts: ALIVE (this session, exits above).
- W1-04 extension: verified end-to-end, non-vacuous.
- Files touched: this receipt only.
\n```json\n{"receipt":"w2-16-pack-court-e2e","schema":"lane.receipt/1","date":"2026-11-01","verdict":"ALIVE","courts":[{"tool":"check_gate_witness_courts.py","exit":0,"cgap_pack_cases":10},{"gates_flipping":5,"vacuous":0}],"lock":{"repo":"/Users/sac/cgap/ggen.lock","hash":"blake3:1f5b9bee0fc525b7d6e091674495c028033df940bc292e317937114411714036","match":true},"commands":[{"cmd":"python3 scripts/check_gate_witness_courts.py","cwd":"/Users/sac/ggen-marketplace","exit":0}]}\n```\n