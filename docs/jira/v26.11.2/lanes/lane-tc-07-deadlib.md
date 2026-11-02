# Lane TC-07 — dead-library exceptions: prefix_max_u8x8 removal attempt

Date: 2026-11-02. Branch main. NO git commands run.

## W1 Subject

`scripts/dead_library_exceptions.json` (owned), receipt (owned).
Read-only subjects: `scripts/verify_dead_library.py` (243 lines, lane AE5),
`crates/csct/src/bin/cscb_c1_c2_ingress.rs:82` (lane cull-05),
`crates/cgap-branchless/src/scan.rs`.

## W2 Dispatch premise — REFUTED EMPIRICALLY (divergence, doc yields)

Dispatch asserted: remove the `prefix_max_u8x8` entry, run the gate,
"expect exit 0 with the gate now ENFORCING consumption" because the symbol
is "now consumed by crates/csct/src/bin/cscb_c1_c2_ingress.rs:82 per
cull-05."

Fresh read of `scripts/verify_dead_library.py` (lines 33-35, 88-104): the
census scans ONLY release-rlib rcgu objects of
`CONSUMERS = ["cgap-gate", "cgap-parquet", "cgap-p28", "cgap-ffi"]`
(`consumer_object_blob` globs `lib<crate>*.rlib`, excludes rmeta members).
`crates/csct` is a standalone-crate BIN (`[workspace]` of its own, per its
Cargo.toml) whose codegen lands in an executable, not an rlib the gate
globs. Source-level consumption at cscb_c1_c2_ingress.rs:82 is real
(`scan::prefix_max_u8x8(x.to_le_bytes())`) but CENSUS-INVISIBLE.

## W3 Commands + exits (this session)

| # | Command | Exit |
|---|---|---|
| 1 | baseline `python3 /Users/sac/cgap/scripts/verify_dead_library.py` | 0 (26 justified exceptions incl. prefix_max_u8x8 refs=0) |
| 2 | remove entry (Edit), `json.load` validity check | 0 |
| 3 | gate run with entry removed | **1** — `DEAD_LIBRARY_SHOWROOM: prefix_max_u8x8 unconsumed in production` |
| 4 | FLIP-1 / FLIP-2 sandbox python (real `run_gate`, real rlib census, /tmp JSON copies) | FLIPS_EXIT=0 (both flips as expected) |
| 5 | `verify_dead_library.py --self-check` | 0 (MUT-A FIRED, MUT-B FIRED-proof) |
| 6 | gate run after restore | 0 — PASS (2 consumed, 26 justified exceptions) |
| 7 | diff vs pre-lane snapshot + other-25 integrity | diff_lines=13, confined to the one entry; `changed other entries: []`, 29->29 keys |

Before-sha256 `a5d013ff0d8286479e509f749ca42aaaa29dc66aeffd8ad86c7bdacd352580bd`;
after-sha256 `5ac8b6a238eb06e0c9d114a7855ac246597d7049b75c253073d5f3ea2334e114`.

## W4 Courts run (anti-vacuity)

- **FLIP-1 (mutation = exception-entry removed while gate-visible refs=0)**:
  gate exit 1 naming `prefix_max_u8x8` (DEAD_LIBRARY_SHOWROOM). FIRED.
- **FLIP-2 (dispatch's demanded flip: /tmp sandbox JSON copy re-adds the
  entry as unconsumed while symbol is consumed-at-source)**: gate
  **reclassifies per its own law** — exit 0, EXCEPTION-JUSTIFIED. The
  dispatch's "gate must FAIL or reclassify per its own law" is satisfied on
  the reclassify branch.
- **--self-check** (gate's own court, shared tree untouched): MUT-A stub
  mutant FIRED (exit 1 names `unconsumed_stub`); MUT-B negative control
  FIRED-proof (cgap-parquet objects census 0 branchless tokens). Exit 0.
  No [SKIP] anywhere.
- Both flips ran the real `run_gate` against real release rlibs under
  `/tmp/ae5-deadlib-target`; only the JSON was sandboxed in /tmp
  (`/tmp/tc07-flip-removed.json`, `/tmp/tc07-flip-readded.json`) because
  `EXCEPTIONS_FILE` is a module constant — the gate script itself was never
  modified.

## W5 Outcome — PARTIAL (removal deferred with cause, not abandoned)

The removal was executed and is REAL: with the entry gone the gate enforces
(exit 1, true positive under its own census scope). But the dispatch's
expected exit-0 post-state is unreachable without a gate-script change
(census blind to csct bin objects), and `verify_dead_library.py` is not an
owned file of this lane. Leaving the shared gate red would freeze every
other lane's gate run under the shared-surface / compile-freeze-analog law,
so the entry was RESTORED with an amended justification recording:

- the source-level consumption (cscb_c1_c2_ingress.rs:82, lane cull-05),
- the measured census blindness (refs=0, this session),
- the exact unblock: gate-owner lane extends the census to csct bin
  objects (or admits csct as a consumer), THEN REMOVE this entry —
  FLIP-2 is the standing witness that removal then holds exit 0.

Net diff on the shared tree: the `prefix_max_u8x8` entry's justification +
plan text only; the other 25 symbol entries and the 3 `_`-metadata keys are
byte-identical to pre-lane state (verified by unified diff, 13 diff lines,
and by per-key deep compare: `changed other entries: []`).

## W6 Fences this diff could break

- `verify_dead_library.py` gate exit code (re-run post-restore: exit 0).
- `--self-check` non-vacuity court (re-run: exit 0).
- No cargo build tree was touched; no mtime restore hazard (JSON only).
- /tmp lease artifacts of this lane: `/tmp/tc07-before.json`,
  `/tmp/tc07-flip-removed.json`, `/tmp/tc07-flip-readded.json`; the cargo
  lease `/tmp/ae5-deadlib-target` is the AE5 gate's own DEFAULT_LEASE, not
  this lane's — left standing for the gate owner.

## W7 Divergences + errata duty

1. Dispatch entry count "26 (9 C05 + 17 legacy)" — confirmed fresh on disk
   (29 JSON keys = 3 `_` metadata + 26 symbols). No divergence there.
2. DIVERGENCE: "expect exit 0 with the gate now ENFORCING" is impossible
   with the entry removed: measured exit 1 (SHOWROOM). This receipt is the
   errata source for any prior receipt (cull-05) claiming the removal is
   consummated; cull-05's claim is true at source level, false at census
   level. The gate-owner lane must extend the census before the entry can
   be lawfully dropped.
