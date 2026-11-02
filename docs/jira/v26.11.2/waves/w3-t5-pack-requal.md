# W3-T5 Pack Re-qualification (cgap-pack post-G1K)

Lane W3-T5 (gate-runner). Subject: `/Users/sac/ggen-marketplace/packs/cgap-pack`
(post G1K expansion, gates 060-140). Date: 2026-11-01. No source edits; no
git writes in either repo.

## Ladder

| Check | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| 1 witness courts | `check_gate_witness_courts.py --packs packs` (ggen-marketplace) | 0 | see E1 |
| 2 rdflib flip battery | `python3 /tmp/w3t5_flip.py` | 0 | see E2 |
| 3 marketplace check | `marketplace.py check cgap-pack` (ggen-marketplace) | 2 | see E3 |
| 3a real-ggen qualification | `qualify_packs.qualify_pack(cgap-pack, ggen, 5.0)` | 0 | see E4 |
| 3b scoped turtle | covered by check 2 | - | see E2 |
| 4 ggen.lock content_hash | `python3 /tmp/w3t5_lock2.py` | 0 | see E5 |
| 5 cross-gate lever IRIs | `python3 /tmp/w3t5_xgate.py` | 0 | see E6 |
| 5a bench registry | same run | 0 | see E7 |

E1: cgap-pack `case_count=14`, `require_pass=true`, `require_fail=true`,
exact-stem correspondence complete (no missing/orphan pass or fail cases),
standing ALIVE. 47 packs configured marketplace-wide, all ALIVE.

E2: all 14 gates flip against the witnesses with rdflib: pass witness
0 rows, fail witness >=1 rows (gate 030 fail=2, gate 140 fail=2).
VACUOUS list empty.

E3: BLOCKED, pre-existing, not cgap-pack: untracked `packs/smalt-core-pack/`
(another lane's in-flight dir, no `pack.toml`) trips the whole-marketplace
preflight `MANIFEST_MISSING`/`ONTOLOGY_SOURCE_MISSING` inside
`require_admitted()` (scripts/marketplace.py:408-415) before the pack
filter applies. cgap-pack contributed zero refusals.

E4: scoped `qualify_packs.qualify_pack("cgap-pack", ggen, 5.0)` returned
`ALIVE`, code None — same path `check()` uses, bypassing the
smalt-core-pack preflight.

E5: independent recompute (blake3 unframed chain
`rel_path_bytes || file_bytes`, byte-wise rel-path sort, `.git`-only
exclusion, 54 files) =
`blake3:cc8c83d3087bd0ce6f98dc4d6d5a9a8eb74d9ef738603be20fd5656b9b671986`
== `/Users/sac/cgap/ggen.lock:5`. A first attempt with a length-prefixed
framing variant produced a different digest, confirming sort+framing is
load-bearing. Algorithm source:
`/Users/sac/ggen/crates/ggen-engine/src/pack.rs:1094-1123`.

E6: 5 `p:ParetoLever` individuals (L1-ingress-digest-escalation,
L2-parquet-swar-fnv1a, L3-bn3-plnf-semi-join, L4-sovereign-ring-sink,
L5-seal-fused-asc450) are the same IRIs across gates 060/070/080/090/100/130
(single ontology.ttl graph, class-scoped `?lever a p:ParetoLever`; no
per-gate duplicate individuals). All 5 `groundedIn` values match the gate
090 regex `^crates/[^ ]+:[0-9]+(-[0-9]+)?$`.

E7: 5 `p:Bench` individuals; each `benchLever` points at the matching
lever IRI (ingress->L1, parquet->L2, bn3->L3, sovereign->L4, seal->L5);
`benchFile` values exist under `/Users/sac/cgap/crates/bench/benches/`.

## Defects found (session, not pre-existing categories: anchor drift)

1. **Stale citation (drift, not a gate failure)**: ontology.ttl pins
   `bench-chains-sovereign` `benchConsts`/`benchSeedLaw` to
   `chains-sovereign.rs:60`, but `const CORPUS_SEED` is at line 77
   (`grep -n CORPUS_SEED crates/bench/benches/chains-sovereign.rs` -> 77, 120).
   Line 60 is `use cgap_parquet::mmap_writer::{RingSink, SlabView, StaticPageBuffer};`.
   Gate 090's regex checks format only, so gates correctly pass; the anchor is
   stale against the working tree (file was modified this wave per git status).
   Owner: whichever lane moved the const. All 9 other cited anchors verified
   present with the named constant on the exact line (receipt table above run).
2. smalt-core-pack manifest missing (pre-existing, other lane's in-flight dir)
   - blocks marketplace-wide commands, not cgap-pack itself.

## Standing

- cgap-pack re-qualification: ALIVE (14/14 gates structural-courts green,
  14/14 flip, lock hash match, real-ggen qualification ALIVE).
- Receipts as-of the current working trees of both repos, this session.

```json
{
  "lane": "W3-T5",
  "subject": "/Users/sac/ggen-marketplace/packs/cgap-pack",
  "standing": "ALIVE",
  "checks": {
    "witness_courts": {"exit": 0, "case_count": 14},
    "flip_battery": {"exit": 0, "gates": 14, "flipped": 14, "vacuous": 0},
    "marketplace_check_scoped":
      {"exit": 2, "blocked_by": "packs/smalt-core-pack missing pack.toml (pre-existing)"},
    "real_ggen_qualification": {"status": "ALIVE"},
    "lock_hash":
      {"match": true, "hash": "blake3:cc8c83d3...671986 (full in E5)"},
    "cross_gate":
      {"levers": 5, "iri_consistent": true, "bench_registry": 5, "grounding_regex_ok": 5}
  },
  "defects": [
    "stale anchor: chains-sovereign.rs:60 cited, CORPUS_SEED at :77",
    "smalt-core-pack: no pack.toml (pre-existing whole-marketplace blocker)"
  ]
}
```
