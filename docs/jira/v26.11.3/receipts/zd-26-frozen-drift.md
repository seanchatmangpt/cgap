# ZD-26 — frozen-drift close-out: both GGM-SRC-002 drifts lifted into source
Standing: ALIVE

Date: 2026-11-04. Lane: ZB-1 audit follow-up. Two repos touched (disclosure:
ggen-marketplace committed+pushed on branch `zd-26-frozen-drift`; cgap left
uncommitted for the coordinator).

## Drift 1 — kind_enums no_std hygiene (std::error → core::error)

- Template `/Users/sac/ggen-marketplace/packs/aoq-pack/templates/kind_enums.rs.tmpl`
  emitted `impl std::error::Error for AoqUnknownEpoch/AoqUnknownStreamType`
  (lines 58/97) while the on-disk generated file
  `crates/cgap-ffi/src/aoq/kind_enums.rs` carried the lawful no_std hand edit
  `core::error::Error`. Every regen would have reintroduced the std form.
- Fix: lifted the hand edit into the template (sed on the two impl lines).
  After sync, `ggen` renders byte-identical output to the on-disk file
  (blake3 `54a359157b81...` pin = actual, run-5 skip reason
  "unchanged: content identical").

## Drift 2 — XC-5r taxonomy prose on generated doc (lifted into graph)

`docs/aoq-header-offsets.md` carried hand prose (statuses-vs-refusals band
split + seams-placeholder kernel-refusal census) absent from
`offsets_tbl-md.tmpl`. Chose the LIFT disposition (not freeze-exemption):
the prose is per-constant/ontology facts and the template already renders
doc content from the graph.

ggen-marketplace changes:

- `ontology.ttl`: new `aoq:docBand` DatatypeProperty ("status" | "refusal");
  8 StatusConstants annotated (001..003 status, 004..008 refusal); new
  `aoq:SeamNote` class + `aoq:noteId` property + individual
  `aoq:note-kernel-refusals` carrying the census paragraph verbatim.
- `offsets_tbl-md.tmpl`: two new band-filtered SPARQL selects (`statuses`,
  `refusals`), a `seamNotes` select, and a body rewrite rendering the band
  split + seams-placeholder section from the graph.

cgap changes:

- `docs/aoq-header-offsets.md` regenerated (written by sync; one-line prose
  paragraph instead of the hand-wrapped variant — content identical,
  now graph-projected).
- `.ggen/freeze/aoq-pack/docs/aoq-header-offsets.md.blake3` re-pinned.

## Re-lock + regen proof (ggen 26.11.3, cwd /Users/sac/cgap)

- `rm ggen.lock && ggen sync run` — first invocation (no flag) exited 1 with
  `[FM-LAW-015]` EA admission gate (`ea.graph.json` absent; gate added in
  ggen 26.11.3, postdates every prior cgap sync receipt). Lawful opt-out used:
  `--skip-ea-admission` (disclosed; the tool's explicit admission escape,
  disclosed here rather than faking an EA graph). All subsequent runs used it.
- Sync write/pin sequence (all runs exit 0):
  - run 1 re-locked at the new pack hashes (closure table shows new
    ontology/template hashes); stale freeze pins for the two hand-edited
    files caused `frozen: manual edit detected` skips — the exact drift
    mechanism, witnessed.
  - stale pins removed; next run wrote `docs/aoq-header-offsets.md` (re-pin)
    but skipped kind_enums as content-identical WITHOUT re-pinning (pin
    file absent). Fix: removed `kind_enums.rs`, re-sync wrote it
    byte-identical (cmp vs /tmp backup: identical, blake3 unchanged
    `54a35915...`) and re-pinned.
  - convergence run: `written: []`; all 5 aoq-pack skips
    "unchanged: content identical"; zero manual-edit skips; all 5
    `.ggen/freeze/aoq-pack` pins re-verified against on-disk blake3
    (status_maps `400f05b6`, kind_enums `54a35915`, frame `33b54f8e`,
    header `bd22ae5f`, doc `a77d169b`) — 5/5 MATCH.

## Verification

- `cargo check -p cgap-ffi` — exit 0.
- `cargo test -p cgap-ffi --lib` — 60 passed / 0 failed (58/58 baseline green;
  +2 tests from other lanes' uncommitted in-flight files in this shared tree;
  zero failures).

## Standing

ALIVE (drifts 1+2 closed at source; regen converges byte-identically, pins
re-converged). Residual: EA admission gate policy for cgap syncs
(--skip-ea-admission vs ea.graph.json) is a coordinator decision, UNVERIFIED
here.
