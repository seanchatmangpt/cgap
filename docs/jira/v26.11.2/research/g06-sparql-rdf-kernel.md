# R6 — ggen RDF/SPARQL kernel prior-art (Explore brief)

Lane: R6. Source: `/Users/sac/ggen`. Claims cited file:line. Purpose: starDF campaign
prior-art + incumbent-weakness survey.

## Architecture

Two crates + one external reasoner:

- `crates/ggen-graph/` — thin deterministic wrapper over **Oxigraph 0.5.11**
  (features=["rdf-12"], workspace Cargo.toml:171). Owns parsing, canonicalization, hashing,
  intent-segregated datasets, SPARQL syntax checks, LSP located parsing, vocab.
- `crates/ggen-engine/` — vendored from ~/praxis; the pipeline, `GraphEngine` trait,
  `LawEngine` seam.
- `graphlaw` crate (external, 26.10.13) — actual N3/Eyeron reasoner + PurRDF SHACL/ShEx
  kernel, reached only through a string-typed seam (law_engine.rs:1-12). Design constraint:
  no oxrdf/spargebra type crosses the LawEngine seam — only N-Triples/N3/Turtle strings
  (ggen-graph/Cargo.toml:45-46; law_engine.rs:5-12).

## Parsing

All delegates to oxigraph `oxrdfio`. Bulk parse `parse_from_reader` materializes via a
**temporary throwaway Store** then drains quads_for_pattern(None,None,None,None)
(ggen-graph/src/graph/parse.rs:9-20) — wasteful (full indexed store per parse). Located
parsing for LSP streams RdfParser with 1-based line/col diagnostics (locate.rs:24-31).
Atomic multi-document insert with per-document parser state never shared +
OntologyBatchReceipt (graph.rs:714-722, graph/ontology_batch.rs). state_hash = BLAKE3 over
sorted canonical quad lines (graph.rs:760-766).

## Query evaluation

**ggen implements zero query-evaluation machinery.** Everything is Oxigraph's engine.
ggen adds policy wrappers:

- AST-level pre-flight refusals via a second spargebra parse (pinned 0.4.7,
  ggen-engine/Cargo.toml:109-116): refuses GRAPH clauses [FM-GRAPH-008]
  (graph.rs:183-226, :276+), non-SELECT/CONSTRUCT/DESCRIBE/ASK, BIND-rebind diagnostics
  (:446+). Per-query double parsing (oxigraph + spargebra) is a structural overhead.
- Syntax check + hand-rolled query-form classifier (ggen-graph/src/sparql.rs:64-73, 26-62).
- Query-guarded codegen: when: ASK guards + named sparql: SELECTs feed Tera, capped
  MAX_QUERY_RESULT_ROWS ("Fortune 5 limit", sync.rs:2283).

## Materialization / inference

1. Manifest inference rules: ordered [[inference.rules]] — optional when: ASK guard then
   CONSTRUCT; derived triples folded before generation rules (generation_rules.rs:55-61,
   284-330; guard types enforced :315-329).
2. N3 forward-chaining to fixpoint via graphlaw/Eyeron: `GraphEngine::materialize`
   (graph.rs:1223-1310) — dumps mirror store to N-Triples **strings**, calls
   law_engine::n3_run (law_engine.rs:147-189, denial-fuse support), folds derived_nt back
   through canonical Turtle insert (idempotent), fires kh: Knowledge Hooks to fixpoint with
   DENIED fail-closed. Rules eagerly probe-parsed at load (:1211-1220).
3. SHACL/ShEx via graphlaw PurRDF on N-Triples strings (graph.rs:1310-1322;
   law_engine.rs:248-300) + separate ggen-graph shacl.rs.

## Internals

- Storage: oxigraph Store behind Arc (ggen-graph/src/graph/dataset.rs:27-38);
  intent-segregated DatasetStore with CrossGraphLeak typed refusal (dataset.rs:64-90, 98-101).
- Canonicalization: color-refinement blank-node canonicalization, 5 fixed iterations
  (graph/canonical.rs:35-130); BLAKE3 over sorted canonical strings (graph/hash.rs:6-27);
  per-intent digests (dataset.rs:13-16). Bookkeeping uses plain HashMap/BTreeMap — nothing
  query-relevant.

## Perf claims (all CLAIMS from the repo's own receipts — not independently verified)

- schema_layer_slo receipt: transpiler ~245 ms/100-shape sig (target <500 ms), schema-gen
  ~12 ms, validation ~4 ms/object, pipeline ~650 ms/project (PERFORMANCE_RECEIPT_TEMPLATE.md
  numbers — see R20 caveats: receipt-voiced but foreign-platform provenance).
- Encoded targets: 10k-triple load+query <10s p90 (ggen_benchmarks.rs:8); SPARQL p90 <100ms;
  CLI startup <100ms (fortune500_performance.rs:1-25).
- MAX_QUERY_RESULT_ROWS hard cap (sync.rs:2283).

## What the stack does well (prior-art takeaways)

1. String-typed engine seam (LawEngine) — swappable reasoner, independently publishable
   crates; but costs full store→string→reparse→reinsert round-trips per materialization pass
   (graph.rs:1224-1230) — an obvious perf ceiling.
2. Determinism first-class: canonical blank-node relabeling + BLAKE3 state hashes; identical
   hash for isomorphic graphs (graph.rs:756-766).
3. AST-level query governance with typed refusals.
4. Fail-closed typed errors everywhere (law_engine.rs:79-126).
5. Declarative inference pipeline, inference-before-generation ordering, guard-gated
   CONSTRUCT.

## Exploitable weaknesses (starDF openings)

- Every materialization pass = full store→string→reparse→reinsert; no incremental fixpoint,
  no in-memory derived-fact path.
- Parse-via-temporary-Store (parse.rs:9-20).
- No native indexes — perf ceiling is Oxigraph's; per-query double parsing; canonical string
  sorting per hash.
- Blank-node canonicalization O(iterations × bnodes × quads), fixed 5 iterations, linear
  scans in loop (canonical.rs:49-130) — quadratic-ish on dense graphs.
- Benchmarks are SLO checks with markdown-voiced claims, not reproducible published numbers.
