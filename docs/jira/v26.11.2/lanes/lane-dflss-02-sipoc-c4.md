# Lane DF-2 receipt - SIPOC + C4 architecture migration

- Date: 2026-11-02. Lane: DF-2 (DMEDI Define/Measure).
- Owned files (all written this session):
  - `docs/jira/v26.11.2/dflss/16-sipoc.md` (new)
  - `docs/jira/v26.11.2/dflss/17-c4-architecture.md` (new)
  - `docs/jira/v26.11.2/16-sipoc.md` (replaced with redirect stub)
  - `docs/jira/v26.11.2/17-c4-architecture.md` (replaced with redirect stub)

## W1 Subject + diff

Migrated the two orphaned-without-receipt docs into the canonical
`dflss/` home and enriched them per the DF-2 dispatch: SIPOC now
carries per-stage real-module + court-name citations
(dma_ring.rs, cgap-branchless scan.rs SWAR, unify.rs:840
RADIX_BUCKETS=4096, cgap-storage dred.rs, csct receipt.rs +
cgap-blake3, RingSink mmap_writer.rs:721); C4 carries an ASCII
container diagram of the real crate graph from a fresh
`boundary_gate.py --deps` run, plus the mandatory honest-notes
register (abstract seal register 0x0001_FF80 / GIT-13; no-SMP swarm
leases a2a_swarm.rs:1-13; mcp_driver fixed-schema text NOT JSON-RPC;
no hardware lore).

## W2 Commands + exits

- `python3 scripts/boundary_gate.py --deps` -> exit 0, "DEPS GATE: E=0
  runtime externals across 15 covered crates (§6.4)".
- `grep -n RADIX_BUCKETS crates/cgap-bn3/src/unify.rs` -> line 840
  (exit 0).
- `ls crates/cgap-git/tests/` -> exit 0; confirms `a2a_swarm_court.rs`,
  `ref_store_court.rs`, `mcp_driver_court.rs` court names cited.
- `sed -n '1,15p' a2a_swarm.rs`, `sed -n '1,20p' ref_store.rs`,
  `sed -n '1,14p' mcp_driver.rs` -> exit 0 each; honesty notes read
  from the modules themselves, not recalled.
- Token sweep `grep -nE "17-cf|transcribed|锚|hmm"` on dflss/*.md ->
  1 hit (a filename typo in the 17 header), fixed via Edit; re-sweep
  clean (0 hits).

## W3 Standing

ALIVE for the migration itself (files on disk, commands + exits above).
The gate table transcribed into `dflss/17-c4-architecture.md` is a
transcription of this session's run — the on-disk gate script is
authoritative; re-run `python3 scripts/boundary_gate.py --deps` rather
than trust the table bytes.

## W4 Fences (must-not-regress surfaces this diff could break)

This diff is documentation-only (4 markdown files); it touches no
Rust, no gate script, no generated artifact. Checked fences:
- `make gate0` / ggen replay: unaffected (no ontology or cli input).
- `boundary_gate.py` / `zero_string_gate.py`: unaffected (no code).
- Old-file consumers: any doc linking `16-sipoc.md` /
  `17-c4-architecture.md` at the old path still resolves to a stub
  that names the canonical dflss path. A repo-wide link audit of
  back-references is NOT run (out of lane scope) — UNVERIFIED.

## W5 Divergences

- The dispatch said "each step cites its REAL module + its court
  name". Verified against `ls crates/*/tests/` this session; all cited
  court files exist. One dispatch framing ("penstock -> SWAR ingress
  scan.rs") was reconciled as two sources: penstock (cgap-gate
  admission) and the SWAR scan primitives live in
  `crates/cgap-branchless/src/scan.rs`, not cgap-core — the SIPOC P1
  row cites both.
- Migration preserved the prior docs' grounding corrections
  (corrections 1-6, divergences D2/D3) verbatim in substance; the
  mermaid container diagram of the old 17 was replaced by the
  dispatch-mandated ASCII container graph, with the mermaid L1/L3/L4
  views kept.
- One Write-tool corruption was caught by the token sweep (stray
  tokens in the first 17-c4 draft) and the file was fully rewritten;
  final on-disk bytes verified clean.
