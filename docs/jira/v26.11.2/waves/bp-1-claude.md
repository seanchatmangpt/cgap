# bp-1-claude

Lane BP-1: CLAUDE.md reduced to Claude Code best-practice shape.
2026-11-01.

## Archive

- Pre-edit archive:
  `docs/jira/v26.11.2/readme-archive/CLAUDE-bestpractice-pre.md`
- sha256: `ec19b1f8043e0ff65d978873be955681da50e7cff12d7e8963467c4f23255fe7`
- Pre-edit wc -l: 158

## Keep-list (all commands re-verified live this session)

- Commands block (8 commands, verbatim): cargo build --workspace,
  cargo test -p cgap-court, CLI build/test, just doctor,
  boundary_gate.py, zero_string_gate.py, doctor/status side-bins,
  CLI regen law.
- Gate exit-code contract (0/1/2/3, VACUOUS law).
- Publish law (ONE attempt, 1h backoff, publish=false set,
  OVERNIGHT-LOOP.md:128 + 01-release-gates.md:28-32 anchors verified).
- Both seeds: `0xBF58_476D_1CE4_E5B9` and `0xA20C_0FFE_E5A7_E000`,
  anchor `crates/cgap-parquet/src/bitslice.rs:687-699` verified by sed.
- mtime-bump restore law, anchor
  `docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184` exists.
- cwd-reset hazard (kept once, merged Conventions+Bash duplication).
- No-Process.sleep law.
- Generated-artifacts table (verbatim regen commands).
- Chicago court discipline, frozen byte-pins, commit -F law.

## Cut-list (old line -> reason)

- L3-15 header prose (substrate description, publish census detail) ->
  re-describes what the repo shows; compressed to 6 lines keeping only
  non-discoverable facts (registry-absent crates, bench name
  collision, datafusion removal pointer, campaign pointers).
- L17-24 Agents paragraph -> compressed to 3 lines; hook behavior is
  not visible in repo code, kept as compressed hazard.
- L54-63 CLI prose (commands, graph/lock paths) -> commands duplicated
  the Commands block and graph paths are readable from the tree; kept
  only the two consumer seams + hand-maintained side-bins.
- L65-88 Layout -> compressed to 9 pointer lines (was 24); group prose
  dropped, crate-group table kept.
- L79-80 "152 receipts" citation kept, surrounding census prose cut.
- L85-88 datafusion REMOVED bullet -> merged into header pointer
  (single reference, removal receipt only).
- L99-133 Conventions -> prose compressed ~35%: "THE SEED DISTINCTION"
  header stack removed, seed bullet kept; standing-vocabulary detail
  compressed.
- L128-130 + L151 cwd-reset stated twice -> stated once (Conventions).
- L134-147 fan-out section -> compressed, all four laws kept.
- L149-158 Bash section -> compressed, all five hazards kept.
- Removed emphasis-stacking ("THE SEED DISTINCTION", "STANDALONE"
  shouting reduced) and the header's older-model narration.

## Verification (this session)

- Final `wc -l CLAUDE.md` = 113 (<= 200, law met; was 158).
- Long-line sweep: only offenders are the two verbatim commands
  (bash fence + regen-table row, each carrying an explicit
  do-not-wrap exception comment) — zero prose lines > 100 chars.
- Path-existence check: every path referenced in the final file was
  `ls`-verified this session (commands, receipts, anchors, seeds
  anchor bitslice.rs:687-699, gen-templates, .claude/agents,
  cgap-verification) — zero MISSING.
- datafusion sweep: 2 occurrences, both in the single removal pointer
  to `docs/jira/v26.11.1/waves/cgap-datafusion-removal.md:6` (itself
  verified on disk). No other datafusion references.
- Seed check: exactly one occurrence each of
  `0xBF58_476D_1CE4_E5B9` and `0xA20C_0FFE_E5A7_E000`, values
  byte-identical to the archived pre-edit file and to
  bitslice.rs:687-699.

## Standing

ALIVE for the mechanical claims above (commands run this session with
exits shown in the lane transcript). The 8 kept commands were not
re-executed as gates this lane — their last live runs are cited in
their respective wave receipts; the line-count/path/seed/datafusion
verifications ARE this session's runs.

