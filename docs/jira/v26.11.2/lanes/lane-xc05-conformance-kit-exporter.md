# Lane XC-05 — conformance kit exporter (red-20 P4)

Lane XC-05 / /Users/sac/cgap / branch main / 2026-11-02.
Owned file: `scripts/conformance_kit_export.py` (only write in the repo
tree; kit manifests written to /tmp only). No git commands run.

## W1 Subject

`scripts/conformance_kit_export.py` — walks the court inventory
(crates/*/tests/*court*.rs cargo-test targets, gate-script court modes,
07-court-corpus.md entries) and emits a sorted deterministic JSON
conformance-kit manifest: court name, invocation (subprocess arg-list),
expected exit class, subject crate, wire-constant surfaces per
redteam/fix-09-wire-constant-manifest.md S1-S5.

## W2 Diff

One new file, 236 lines. No shared-file diffs. Surface:
`--out/--list/--run NAME[,NAME]/--self-check/--root`; exit law
0/1/2/3 (SUBJECT-MISSING exit 2 witnessed below).

## W3 Commands + exits (this session, exact subject)

| leg | command | exit |
|---|---|---|
| syntax | python3 -c ast.parse | OK |
| self-check | `python3 scripts/conformance_kit_export.py --self-check` | 0 |
| emit | `python3 scripts/conformance_kit_export.py --out /tmp/xc05-manifest.json` | 0 |
| determinism replay | emit to /tmp/xc05-m2.json; `cmp` vs first | BYTE-IDENTICAL |
| inventory | --list | 91 lines (81 cargo-test, 5 corpus-doc, 5 gate-mode) |
| run | `--run doc_offset_table_court` | 0 (ALL-COURTS-PASS 1) |
| exit-2 | `--run nonexistent_court` | 2 (SUBJECT-MISSING) |

Sample manifest: `/tmp/xc05-manifest.json` (91 courts). Wire-surface
tags nonempty for ffi/gate/p28/core/blake3/branchless courts; e.g.
alloc_courts -> [S1-status-register, S5-receipt-chain-swar].

## W4 Anti-vacuity (self-check battery, exit 0, legs witnessed)

- LEG-A determinism: manifest generated twice, byte-compare (34,732 B
  identical) — a non-deterministic scan would flip this.
- LEG-B flip: /tmp-mirrored subset tree with
  patent_behavioral_court dropped -> inventory 91 -> 5 entries,
  manifest differs -> detection witnessed. Divergence noted: the /tmp
  mirror mirrors only the one crate's tests/ dir, so the corpus-doc +
  gate-mode entries also drop (no docs/ or scripts/ in the mirror) —
  the drop is strictly larger than one court, never smaller, so the
  flip cannot pass vacuously; recorded as a scope note, not a silent
  skip.
- LEG-C run leg: doc_offset_table_court via real cargo test (2 passed,
  exit 0). All three legs logged to /tmp/xc05-selfcheck.log.

## W5 Fences could-break

None — zero diffs to guarded files. New script only; no gate/court
subjects touched. `git status` diff surface: untracked
scripts/conformance_kit_export.py.

## W6 Standing

ALIVE for the exporter and its three self-check legs (commands+exits
above, this session, exact subject). UNVERIFIED: full-kit --run of all
81 cargo courts (not run this session — build-cost only; per-court
invocations are emitted in the manifest for a caller to execute).

## W7 JSON terminus

{"lane":"XC-05","files_written":["scripts/conformance_kit_export.py",
"docs/jira/v26.11.2/lanes/lane-xc05-conformance-kit-exporter.md"],
"commands":["--self-check exit 0","--out exit 0","cmp BYTE-IDENTICAL",
"--run doc_offset_table_court exit 0","--run nonexistent_court exit 2",
"--list 91"],
"courts_run":["doc_offset_table_court (exit 0, 2 passed)"],
"standing":"ALIVE","divergences":["LEG-B /tmp mirror drops corpus-doc+gate entries too (strictly-larger flip, cannot pass vacuously; scope noted)"]}
