# ZD-4: dod-pack ontology dod-006 TARGET-UNMEASURED -> MEASURED (YD-7 Finding F2)

- **Date**: 2026-11-04
- **Lane**: ZD-4 (create/update agent), YD-7 finding F2
- **Status**: ALIVE

## Finding

The dod-pack ontology projected dod-006 (AB-5 ML-KEM-768 round-trip) as
TARGET-UNMEASURED citing `af-1-corridor.md`, while the landed truth is MEASURED
per `bk-1-takeover.md`: `cargo test -p cgap-crypto` green x2 (75 passed / 0
failed), corridor join court green x2 on the real ML-KEM-768, AF1-KD-1 CLOSED.

## Gate 020 analysis (before flipping)

`packs/dod-pack/gates/020_target_unmeasured_not_blocking.rq` forbids
TARGET-UNMEASURED gates from being publish-blocking. dod-006 was
TARGET-UNMEASURED ∧ publishBlocking=false (lawful). All other MEASURED gates in
the pack are publishBlocking=true; with standing now MEASURED, dod-006 becomes
publishBlocking=true, consistent with the pack and lawful under gate 020 (which
only constrains TARGET-UNMEASURED rows). dod-006 therefore joins the
publish-blocking set.

## Diff

`packs/dod-pack/ontology.ttl` dod:g-006 row only:

- dod:description: "AB-5 ML-KEM-768 round-trip ... — IN FLIGHT: mlkem_sanity
  FAILS at HEAD, TestKem fallback still bound (AF1-KD-1)" -> "AB-5 ML-KEM-768
  round-trip (encaps/decaps shared-secret agreement): cgap-crypto suite green
  x2, corridor join court green x2 on the real ML-KEM-768, AF1-KD-1 CLOSED"
- dod:evidenceReceipt: `docs/jira/v26.11.3/receipts/af-1-corridor.md` ->
  `docs/jira/v26.11.3/receipts/bk-1-takeover.md`
- dod:standing: TARGET-UNMEASURED -> MEASURED
- dod:publishBlocking: false -> true

Gates in `packs/dod-pack/gates/` cite no dod-006 receipt; no gate edit needed.

## Commands / exits

- `rm ggen.lock` — first `ggen sync run` (exit 1) hit FM-PACK-008 pack hash
  mismatch for dod-pack after the ontology edit (expected; documented
  remediation in CLAUDE.md `make gate0` row: rm lock and re-lock).
- `ggen sync run` (run 1): exit 0; decisions show
  `docs/jira/v26.11.3/dod-checklist.md: "written"`. The working-tree checklist
  had the stale TARGET-UNMEASURED row (YD-7's sync had regenerated it from the
  stale graph); this run restored the MEASURED row.
- `ggen sync run` (run 2): exit 0; `written: []`; dod-checklist decision
  "skipped: unchanged: content identical".
- `sha256sum docs/jira/v26.11.3/dod-checklist.md` after run 1 and run 2:
  both `659d211e6416e3e820bc4c7bcbb24768cad6468d3e30d00b6eecb9a01143a778`
  — byte-identical convergence at the new fixed point (F2 close condition).

## Before / after row (checklist line 17)

- **Before** (sha256 `6c4beccb...aa46` tree):
  `| dod-006 | AB-5 ML-KEM-768 round-trip (encaps/decaps shared-secret agreement) — IN FLIGHT: mlkem_sanity FAILS at HEAD, TestKem fallback still bound (AF1-KD-1) | cargo test -p cgap-crypto --test mlkem_sanity | docs/jira/v26.11.3/receipts/af-1-corridor.md | TARGET-UNMEASURED | false |`
- **After**:
  `| dod-006 | AB-5 ML-KEM-768 round-trip (encaps/decaps shared-secret agreement): cgap-crypto suite green x2, corridor join court green x2 on the real ML-KEM-768, AF1-KD-1 CLOSED | cargo test -p cgap-crypto --test mlkem_sanity | docs/jira/v26.11.3/receipts/bk-1-takeover.md | MEASURED | true |`

## Standing

- dod-006 ontology row: MEASURED, publishBlocking=true, citing bk-1-takeover.md.
- Checklist regenerated from graph, converged byte-identical over two sync runs.
- Unrelated: sync reported two frozen-file skips (cgap_aoq.h frame.rs,
  aoq-header-offsets.md, freeze_policy=checksum manual-edit detection) —
  pre-existing, untouched.
