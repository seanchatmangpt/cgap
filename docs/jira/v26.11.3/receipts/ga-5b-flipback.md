# GA-5b: staged aoq-pack flip-back to the canonical marketplace pack

Date: 2026-11-04 (lane GA-5b). Repo: `/Users/sac/cgap`. Owner files:
`/Users/sac/cgap/ggen.toml`; `/Users/sac/cgap/Makefile` GATE0_FILES unchanged
(both packs emit `include/cgap_aoq.h` — no Makefile edit needed).

## Before (staged fallback, GA-5)

- `ggen.toml` (pre-edit copy preserved at `/tmp/ggen.toml.before`):
  - `imports = ["ontology-packs/aoq/ontology.ttl"]`
  - `[packs].aoq-pack = { path = "ontology-packs/aoq" }`
- Staged pack: 8 files, tree-hash
  `sha256 of sha256s: 3b0803d8efc5b073002172a4d44d0dd282e6ef260c96c392aa2862d17ede69da`
  (ontology.ttl, pack.toml v0.1.0-staged, gates/010_offsets_dense.rq, 5
  templates querying namespace `https://cgap.chatmangpt.com/aoq#`).
- Generated surface (staged): `crates/cgap-ffi/src/aoq/{mod,records,offsets}.rs`
  + `include/cgap_aoq.h` + `docs/aoq-header-offsets.md`; 9 AoqRecord
  individuals, RECORD_COUNT = 9, ENTRY/ENVELOPE field defines.

## Flip

1. `ggen.toml`: `[packs].aoq-pack` re-pointed to
   `../ggen-marketplace/packs/aoq-pack`. FM-CONFIG-003 discovery:
   `ontology.imports` REJECTS `..` traversal in ggen 26.10.11
   (`Invalid path '../ggen-marketplace/packs/aoq-pack/ontology.ttl': path
   traversal ('..') is not allowed`). Since `[packs]` paths allow `..`
   (cgap-pack already used one) and the pack loader imports the pack's own
   `ontology.ttl` into the union graph, the `imports` line was dropped
   entirely — no in-repo vendored copy minted. Union-graph proof: generated
   `frame.rs`/`include/cgap_aoq.h` carry the full canonical AOQ-02 field
   table (8 header fields, MAGIC@0 .. HEADER_DIGEST@60, header size 64,
   FNV-1a32 at 0x3C), which exists only in the canonical ontology.
2. FM-PACK-008 re-lock ×2 (both pre-authorized in-lane): first on pack
   pointer flip (blake3 963fa30a… → 678e8e2f…), second after the court
   runner was added to the pack (678e8e2f… → 99adbf4c…). Both re-locks
   green, sync `written: []` on consecutive runs afterward.
3. Sync receipts: sync1 `written` = 5 canonical slots (frame.rs,
   kind_enums.rs, status_maps.rs, include/cgap_aoq.h,
   docs/aoq-header-offsets.md); sync2, sync3 `written: []`
   (witness `/tmp/sync3.json`).
4. Freeze slots (addendum item 2): `.ggen/freeze/aoq-pack/` minted all 5
   `.blake3` pins on first canonical sync — include/cgap_aoq.h.blake3,
   docs/aoq-header-offsets.md.blake3,
   crates/cgap-ffi/src/aoq/{frame,kind_enums,status_maps}.rs.blake3.
   No fix needed: canonical templates carry `freeze_policy: checksum` +
   `freeze_slots_dir` frontmatter; the staged pack's templates did not,
   which is why the dir never existed before.
5. Rust surface swap: staged `records.rs`/`offsets.rs` deleted (canonical
   pack does not generate them); `mod.rs` rewritten as a hand-written module
   root (irreducible residue — the canonical pack generates no mod.rs),
   re-exporting frame/kind_enums/status_maps with
   `#[allow(non_camel_case_types)]` (graph-carried variant names
   ROTATED_1/ROTATED_2). `lib.rs` doc comment updated. Disclosed
   hand-written fixes: `crates/cgap-ffi/src/aoq/mod.rs` (22 lines) +
   one lib.rs doc-comment line. No generated file hand-edited.
6. Gates ran LIVE in cgap: 9 canonical gates evaluated against the union
   graph on every sync, zero violations (sync exits 0). GA-4 court receipt:
   `/Users/sac/arcus/docs/jira/v26.11.3/receipts/ga-4-aoq-court.md`
   (cited per coordinator; not re-run — the flip was re-proven live here by
   the executable court, item 7).
7. Witness court made executable (coordinator addendum item 1): authored
   `/Users/sac/ggen-marketplace/packs/aoq-pack/qualification/verify.py`
   (gate-court.toml declared `runner = "python3 qualification/verify.py"`
   with no such file — a config-only court). Implementation: per gate stem
   (gates/ directory listing = truth), SELECT-any-row / ASK-true = violation;
   pass witness must give 0 violations, fail witness >= 1; typed exits:
   CONFORMANT=0, VIOLATION=1, COURT-BROKEN=2 (missing witness / query error,
   never silent). Run 2026-11-04: **CONFORMANT, 9 gates, 9/9 FLIP-OK,
   exit 0** (fail_rows: 1 per gate, 2 for 030_offset_contiguity). The pack
   file addition caused re-lock #2 above.
8. Staged pack retired: `ontology-packs/aoq/` deleted ONLY after canonical
   sync proved green (sync ×2 `written: []` + gate0 PASS first). Tree-hash
   captured above; recoverable from cgap git history (no git commands run
   in-lane). `ontology-packs/` retained (unrelated `cloud`, `fibo` packs).

## Verification ladder

| # | Check | Command | Result |
|---|---|---|---|
| 1 | sync1 | `ggen sync run` | exit 0, written = 5 canonical slots |
| 2 | determinism | `ggen sync run` ×2 more | `written: []` both |
| 3 | gate0 | `make gate0` | byte-stable replay PASS (incl. include/cgap_aoq.h, sha256 0bd81821…) + freestanding cgap_c_data.h compile PASS |
| 4 | header compile | `cc -std=c99 -ffreestanding -Wall -Wextra -pedantic -c include/cgap_aoq.h` | PASS exit 0 |
| 5 | cargo | `cargo check -p cgap-ffi` | clean, 0 warnings |
| 6 | unit tests | `cargo test -p cgap-ffi --lib aoq` | 6 passed, 0 failed |
| 7 | witness court | `python3 qualification/verify.py` (pack root) | CONFORMANT 9/9 FLIP-OK, exit 0 |
| 8 | re-lock | `rm ggen.lock; ggen sync run` ×2 | green, `written: []` |

Freestanding C99 compile of `include/cgap_aoq.h` PASS — the richer canonical
output (static-inline little-endian accessors) is freestanding-clean.

## Standing

ALIVE. Canonical marketplace aoq-pack is the single live AOQ source; staged
fallback deleted; court executable at 9/9; freeze slots 5/5 minted; gate0
byte-stable incl. the canonical header.

## Falsifiers

- "A pack flip produces output drift" — REFUTED: outputs byte-stable across
  3 syncs (gate0 cmp PASS).
- "The witness court cannot flip" — REFUTED: 9/9 FLIP-OK (010–090).
- "Freeze slots never mint" — REFUTED: 5/5 `.blake3` pins minted on first
  canonical sync.

result: CONFORMANT
in cgap, sync written:[] deterministic, gate0 PASS, header freestanding-C99
PASS, cargo 6/6 aoq tests, witness court executable 9/9 flip, staged pack
retired; receipt /Users/sac/cgap/docs/jira/v26.11.3/receipts/ga-5b-flipback.md
