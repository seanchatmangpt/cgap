# ERRC-D: packs s-z evaluation

Lane: ERRC-D (court-auditor, read-only on ~/ggen-marketplace except this receipt).
Subject: /Users/sac/ggen-marketplace/packs — 78 packs s-z enumerated; manifest
read per pack (pack.toml via tomllib / grep).

## Commands + exits (mine)

- `ls packs | awk s-z` -> 78 packs, exit 0
- per-pack pack.toml header read (grep/tomllib), exit 0
- `python3 scripts/marketplace.py validate` -> exit 2
  `REFUSED:MANIFEST_MISSING:smalt-core-pack` +
  `REFUSED:ONTOLOGY_SOURCE_MISSING:smalt-core-pack`
- `python3 packs/wasm4pm-cognition-pack/qualification/verify.py` -> exit 0
  (ADMITTED; see vacuity finding on gate 020 below)
- consumer sweep `grep -rl "urn:ggen:pack:<name>" --include=pack.toml` x78,
  exit 0; cross-checked against a known consumer edge (affidavit-pack ->
  wasm4pm-facts/star-toml) — pattern validated

## Findings

1. STANDING GATE DEFECT: marketplace-wide `validate` is RED (exit 2).
   smalt-core-pack has no pack.toml (only ontologies/, PACK-NOTES.md, etc.),
   so the registry refuses it. Every downstream catalog/fingerprint command
   is blocked on this one pack. ELIMINATE candidate as-is or REDUCE by adding
   a manifest; today it poisons the whole-plane admission.
2. VACUITY CELL: wasm4pm-cognition-pack verify.py prints
   `witness 020_single_valued.rq: pass_rows=0 fail_rows=0` and still
   ADMITS. A fail-witness that never fires is a vacuous court cell; gate 020
   has no negative witness in witness_fail.ttl for that shape. REDUCE: add a
   fail fixture or drop gate 020 from the admitted set.
3. Consumer economy (requires-edge count, validated pattern):
   hubs = standing-ladder-pack (6), strata-protocol-pack (5),
   star-toml-pack (4), wasm4pm-facts-pack (4), semantic-gate-witness-court (3),
   sa2a-bridge (3). Tail of 1-consumer packs is wide (~55).
4. strata family = 16 packs, 0.1.0-era, mostly ontology+src with 0-1
   templates, consumed only by other strata packs (self-referential ring;
   only strata-protocol escapes with 5 requires). REDUCE: consolidate ring
   members into strata-protocol/-hash cores or mark SIM-PASS/PROPOSED.
5. wasm4pm family: consolidation already honest (facts-pack single owner,
   drift log, anti-vacuity court) — best-in-family, RAISE template.
6. Zero-template AND zero-gate packs: sa2a-governed-process,
   semantic-procedural-graph, strategic-doctrine-compiler, supplier-equilibrium,
   temporal-truth-observability, typer-azure (and smalt-core which is worse:
   no manifest). These are vocabulary-only — fine as ontology capital, but
   none should be counted as manufacturing packs.

## Courts run

- marketplace.py validate: RED on smalt-core-pack (real flip evidence: the
  registry refuses a manifest-missing subject; fail-closed, not vacuous).
- wasm4pm-cognition verify.py: GREEN but with one non-firing witness (020) —
  partially vacuous, disclosed above.

## Packs cgap should consume

- wasi-json-abi-pack -> cgap-ffi / wasm-footprint (FFI shell, guards, ABI
  doc, artifact-pin skeleton — matches cgap's C-ABI + no_std wasm lane).
- star-toml-pack -> crates/cli config admission (generated validate() with
  range contracts, checksum freeze, self-generating proof tests).
- tcps-release-pack -> crates.io 26.10.29 five-crate release surface (CI
  matrix + packaging manifests projected from ontology individuals).
- standing-ladder-pack + semantic-gate-witness-court-pack -> cgap-court
  receipt/standing vocabulary and gate-witness court schema (ggen.semantic-
  gate-witness-court/1) for court_verdict receipts.
- workflow-corpus-pack -> court subjects for cgap-court (adversarial exact
  semantic subjects with pinned capability IDs).
- value-innovation-errc-pack -> the ERRC calculus itself for A01-A20 lane
  backlog scoring (RDF Data Cube canvas, Pareto dominance w/ witnesses).

## ERRC classification (top movers)

- ELIMINATE: smalt-core-pack as a pack (no manifest, unregistrable) or
  promote to manifest + shapes; strata ring consolidation (16 -> ~4).
- REDUCE: single-consumer tail (~55 packs at requires=1) — fold family-
  internal-only packs; fix wasm4pm-cognition gate 020 witness.
- RAISE: wasm4pm-facts consolidation pattern to strata/shacl-to-* families;
  standing-ladder as the marketplace-wide standing receipt vocabulary.
- CREATE-gap: no s-z pack projects cgap's binary-ontology (B-ONT/bn3)
  surface; guard-flip+metadata publish law has no s-z court either.

## Top-3 slice innovations

1. smalt-core-pack manifest repair (one pack.toml) — unblocks the entire
   marketplace validate/catalog/fingerprint plane (highest leverage per byte).
2. wasm4pm-cognition gate-020 fail-witness — turns a partially vacuous court
   fully non-vacuous; canonical shim-court pattern applies.
3. wasi-json-abi-pack -> cgap-ffi consumption receipt: first external pack
   consumer for cgap's wasm FFI shell, proving marketplace<->cgap composition.

Standing: PARTIAL_ALIVE — enumeration + manifest reads + 2 courts run; the
78-pack census is receipted, per-pack qualification runs are not (only
wasm4pm-cognition executed end-to-end).

Divergences: launching message scoped me as "one lane's receipt in
/Users/sac/cgap" auditor in the system prompt while the task assigns the
ERRC-D evaluation lane; I executed the task directive (read-only on
ggen-marketplace, receipt written to the cgap waves dir as instructed).
