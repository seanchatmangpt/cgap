# lane-dod-02-branchless-pub

2026-11-02. Repo /Users/sac/cgap, main. Fixes PUB-02's 2 NO-GOs
(cgap, cgap-gate) via the pub-16 F3 candidacy: cgap-branchless
publish flip + root version pin + ledger row.

## 1. Files written

- `crates/cgap-branchless/Cargo.toml` — publish=false -> true;
  added `repository.workspace = true`, `authors.workspace = true`
  (PUB-01 A2 house style). First Write duplicated the two keys on
  disk; rewritten clean and verified.
- `/Users/sac/cgap/Cargo.toml` — root [workspace.dependencies]
  branchless entry gained `version = "26.11.2"` (PUB-02 NO-GO 2
  root cause; cgap-gate `workspace = true` now validates).
- `scripts/registry_sweep.py` — LEDGER gained the cgap-branchless
  row, state UNPUBLISHED, pointer
  `docs/jira/v26.11.2/waves/w6-branchless-workspace-wire.md:9`
  (closes the pub-16 `MISSING no-ledger-row` exit-1 cause).
- `CLAUDE.md` publish paragraph — branchless moved out of the
  publish=false list into the true list (9 true / 3 false /
  11 default), DOD-02 receipt cited.
- `scripts/dead_library_exceptions.json` NOT created (sweep row
  needed no exception).

## 2. Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/dod02-target cargo package --no-verify
  --allow-dirty -p cgap-gate -p cgap -p cgap-branchless`
  -> EXIT 0. Both PUB-02 NO-GOs flip to GO: cgap packaged (21
  files), cgap-gate packaged (9 files), branchless packaged
  (19 files, 156.4KiB).
- `python3 scripts/registry_sweep.py` -> EXIT 0, "PASS 0 missing";
  row reads `cgap-branchless UNPUBLISHED PASS`. (Sweep wrote
  registry-sweep-table.json into the tree — known script
  behavior, disclosed; not a lane-owned file.)
- Shared-tree fence: `cargo check -p cgap-branchless -p cgap-gate
  -p cgap` -> EXIT 0.
- Anti-vacuity court 1 (package flag): git archive HEAD -> /tmp
  sandbox + working-tree manifests overlaid; publish=true ->
  false via sed -> `cargo package -p cgap-branchless -p cgap-gate
  -p cgap` EXIT 101, error `no matching package named
  cgap-branchless found` (crates.io index, required by cgap-gate)
  — the exact PUB-02 NO-GO error; sed restore -> diff vs shared
  file byte-exact -> EXIT 0. Court FLIPS.
- Anti-vacuity court 2 (ledger): script copied to /tmp with
  UNPUBLISHED -> DELISTED expectation, run --repo /Users/sac/cgap
  -> EXIT 1, `[MISSING] cgap-branchless: manifest lost the
  publish guard`. Court FLIPS.
- Build lease /tmp/dod02-target deleted at lane end (exit 0).

## 3. Fences run

- cargo check on the three affected crates (exit 0 above).
- registry_sweep gate re-run green (exit 0 above).
- No other gate touches the root workspace.dependencies table in
  this diff's surface; make gate0 / doctor untouched by these
  edits but NOT re-run this session (coordinator surface).

## 4. Divergences / disclosures

- D1: the lane doc's "§4 guarded-table move" targets
  `08-registry-sweep.md`, which is NOT in this lane's owned-file
  list — not edited. Recorded here instead: branchless is no
  longer a guarded row candidate; 08-sweep §4 + §1 table should
  gain the UNPUBLISHED/PASS row on the next owner edit.
- D2 (witnessed mid-flight, not mine): shared
  `crates/cgap/Cargo.toml` and ~16 other member manifests
  transiently carried `categories`/`keywords` as comma-joined
  STRINGS (invalid manifest types; cargo exit 101
  `invalid type ... expected a vector`) during this lane's
  sandbox court. By the time of the final shared-tree check the
  shared tree was fixed by the owning lane (sandbox-only python
  type fixes were applied to /tmp/dod02-sandbox to let the court
  run; shared tree untouched by me). Disclosed per
  disclose-in-both-directions; the owning lane supersedes.
- D3: first Write of branchless Cargo.toml landed duplicated
  repository/authors keys (write-integrity corruption class);
  detected by cargo (duplicate key error), rewritten, verified.
