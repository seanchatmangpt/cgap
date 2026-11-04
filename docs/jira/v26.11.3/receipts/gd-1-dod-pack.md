# Receipt: GD-1 cgap definition-of-done pack (v26.11.3)

| Field | Value |
|---|---|
| Subject | /Users/sac/cgap — packs/dod-pack/ + ggen.toml [packs] wiring + rendered projections |
| Lane | GD-1, IMPLEMENTATION |
| Date | 2026-11-04 (work started 2026-11-03; 429 restart mid-pass, coordinator relay honored) |
| Tool | ggen 26.10.11 (ambient installed binary) |
| Standing | ALIVE for the pack surface (sync ×2 determinism + witness court + shim-court flip, all green this session); PARTIAL overall — the DoD graph itself honestly carries dod-006 (AB-5 ML-KEM) at TARGET-UNMEASURED |

## Diff (lane-owned files)

- `packs/dod-pack/pack.toml` — NEW: pack identity + `[validation] gates` listing
  the 4 gate files (declarative; the gates/ directory listing is the runner).
- `packs/dod-pack/ontology.ttl` — NEW: `dod:` (urn:cgap:dod:) graph. Classes
  `DoDGate` + `DoDTarget`; properties gateId, targetId, description,
  checkCommand, evidenceReceipt, standing, publishBlocking, closeCondition.
  12 DoDGate individuals (dod-001..dod-012), 2 DoDTarget individuals
  (dod-t-001..002). Standing never rounded up: dod-006 (AB-5 ML-KEM) is
  TARGET-UNMEASURED (mlkem_sanity FAILS at HEAD per AF1-KD-1,
  receipts/af-1-corridor.md); dod-t-001 (index 177/177) and dod-t-002
  (arcus-plane goldens/gofmt) are DoDTargets with named close conditions —
  no exit record for either was found on disk this session.
- `packs/dod-pack/gates/{010_required_fields,020_target_unmeasured_not_blocking,030_gate_id_uniqueness,040_target_close_condition}.rq` — NEW: 4 sync-time
  SPARQL closure gates (SELECT-any-row = violation, FM-PACK-013).
- `packs/dod-pack/templates/{dod-checklist.md,dod-targets-open.md}.tmpl` — NEW:
  render `docs/jira/v26.11.3/dod-checklist.md` (12 gates, ORDER BY gateId, with
  publish-blocking set) and `docs/jira/v26.11.3/dod-targets-open.md`
  (TARGET-UNMEASURED rows + named close conditions).
- `packs/dod-pack/witnesses/witness_court.sh` — NEW: per-gate PASS witness
  (real sync exit 0) + FAIL witness (SHIM-COURT probe → FM-PACK-013) with
  sha256 fail-closed restore. Verdicts CONFORMANT=0 / VIOLATION=1 /
  COURT-BROKEN=2.
- `ggen.toml` — dod-pack added to `[packs]` with rationale comment.
- `ggen.lock` — RE-LOCKED (deleted and recreated) three times this lane:
  (1) pre-existing drift vs ../ggen-marketplace cgap-pack content hash
  ([FM-PACK-008], stale lock, not caused by dod-pack); (2) template edits after
  lock; (3) court protocol re-locks. Prior lock preserved at /tmp/gd1-ggen.lock.bak.
- Rendered: `docs/jira/v26.11.3/dod-checklist.md`, `docs/jira/v26.11.3/dod-targets-open.md`
  (generated; never hand-edit).
- This receipt.

## Grounding (per-gate, receipts re-read on disk)

- dod-001 cgap-arrow tests → cg-1.md (220/0) + sa-4.md (228/0), MEASURED.
- dod-002 cargo check -p cgap-aoq → ab-1.md:53 exit 0, MEASURED.
- dod-003 fips202 → ab-2-fips202.md (`--lib fips202`, 10/0), MEASURED.
- dod-004 chacha → ab-3-chacha.md (`--lib chacha`, 9/0), MEASURED.
- dod-005 aead/poly1305 → ab-4-aead.md (aead_courts + ct_scan_court), MEASURED.
- dod-006 AB-5 ML-KEM → af-1-corridor.md AF1-KD-1 (mlkem_sanity FAIL at HEAD,
  TestKem fallback), TARGET-UNMEASURED, publishBlocking false.
- dod-007 riscv32imc arrow → cg-1.md:43-44 exit 0, MEASURED; dod-008 wasm32 aoq
  → ab-1.md:58 exit 0, MEASURED.
- dod-009 CBMQS release → af-10-cbmqs.md (cbmqs_gates green x2 release), MEASURED.
- dod-010 corridor join → af-1 grounding; corridor_join_court 3/0 (af-1-corridor.md:69), MEASURED.
- dod-011/012 sync determinism + FM-LAW-015 class zero → ga-5b-flipback.md
  (sync ×2 exit 0; no FM-LAW-015 refusal witnessed at cgap sync), MEASURED.
- dod-t-001 "index 177/177" and dod-t-002 "gofmt/goldens arcus-plane" — NO exit
  record found in cgap receipts this session (grepped 177/177, gofmt/goldens);
  modeled as DoDTarget + named close conditions, TARGET-UNMEASURED. Close
  condition for t-002 cites arcus tests/goldens/verify.sh (witnessed green in
  config-lsp integration-close.md arcus census).

## Gates run this session (exact commands, observed exits)

- `ggen sync run` ×2 — exit 0 / 0; run 1 wrote the 2 dod projections
  (written: ['docs/jira/v26.11.3/dod-checklist.md',
  'docs/jira/v26.11.3/dod-targets-open.md']); run 2 `written: []`
  (DETERMINISM-OK). Final clean-up sync after the court: written: [].
- Witness court: `bash packs/dod-pack/witnesses/witness_court.sh` — exit 0,
  CONFORMANT, 4/4 FLIP-OK. PASS witness = unshimmed sync exit 0 (all 4 gates
  green vs union graph). Per-gate FAIL witness = shim to always-violating
  `SELECT ?s WHERE { ?s ?p ?o } LIMIT 1` → `[FM-PACK-013] pack dod-pack gate
  <name> refused the sync` exit 1, restore sha256 byte-exact.
  Non-vacuity witnessed: first court run caught a real defect — the script's
  restore path was inverted (shim without saved original); COURT-BROKEN exit 2
  fired, script fixed, rerun green. An earlier court run also left gate 010
  shimmed on disk (RESTORE-FAIL); restored from lane memory and re-verified
  (sha256 84e45d0d…, real content, sync green after).
- `ggen graph validate` — named 2 template violations in PRE-EXISTING cgap
  files (gen-templates/version_rs.tmpl, cgap-pack subject-count.txt.tmpl,
  SELECT-without-ORDER-BY [FM-TPL-010]); none in dod-pack files. Pre-existing,
  left for owning lanes, disclosed here.
- `ggen packs validate --pack-id dod-pack` — registry-lenient
  `is_valid: false` (dod-pack is project-local, not a marketplace pack —
  expected, not a defect). Marketplace holds 348 packs on disk (≥ 347 counted
  2026-11-04: `ls /Users/sac/ggen-marketplace/packs | wc -l`).

## Discovered mechanism (run-ggen skill addition candidate)

Pack self-discovery of `templates/*.tmpl` requires the file to START with
`---` frontmatter delimiters; a .tmpl without them is SILENTLY skipped by
`discover_templates` (treated as rules-consumed) — gates still ran, no error,
written:[] with zero mention of the template. cgap-pack's own .tmpl files
lack `---` yet render (via a different discovery path this lane did not chase;
source tree `/Users/sac/ggen` sync.rs:2960-2968 shows the check). Cost: two
debug cycles. Fix was `---\n` prefix + a YAML-indent repair.

## Falsifier / replay

- Replay: `cd /Users/sac/cgap && rm -f ggen.lock && ggen sync run && ggen sync run`
  (2nd written:[]) then `bash packs/dod-pack/witnesses/witness_court.sh` (exit 0).
- Falsifier: a DoD edit that flips a standing to MEASURED without an exit
  record in a receipt would be refused by nothing in the pack itself (the
  standing is data) — but gate 010 refuses any DoDGate missing its fields and
  gate 020 refuses TARGET-UNMEASURED + publishBlocking=true; the court proves
  both flip.
