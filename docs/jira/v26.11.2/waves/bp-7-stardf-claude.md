# BP-7 stardf CLAUDE.md best-practice pass

Subject: /Users/sac/stardf/CLAUDE.md (pre-edit snapshot archived at
/Users/sac/stardf/docs/bestpractice-archive/CLAUDE-pre.md, sha256
`0e991e33115b3ee65df0cccd99d675e2bac0c311e6a780d5326792c22c278749`).

## Commands run (exit codes)

- `cd /Users/sac/stardf/arrow && cargo build --workspace` — exit 0 (1m06s, Finished dev).
- `python3 scripts/verify.py` — exit 0; [ALL PASS]; header/impl parity 13==13,
  three-way ontology/header/impl equality witnessed.
- `python3 scripts/verify_stardf_licenses.py` — loud `[SKIP] compat register missing:
  /Users/sac/authority/stardf/license-compat.json`, gate SKIP exit 1 by design (REPO
  = parents[3] path bug, disclosed in CLAUDE.md authority item 3).
- `nm -gU target/debug/libstardf_arrow_capi.dylib | grep -c stardf_` — 13 (census
  witness, matches the v1 surface table).
- Path-resolution audit: all 18 referenced paths resolve (see Keep list sources).

## Cut list

- Redundant "What this is" H2 wrapper — lede paragraph carries it.
- STALE: "register is only PARTIALLY re-homed ... pending a copy-in to a top-level
  `authority/stardf/` here" — the copy-in LANDED: all six files present at
  /Users/sac/stardf/authority/stardf/ (witnessed by ls). Replaced with the true
  remaining defect: `verify_stardf_licenses.py:20 REPO = parents[3]` resolves to
  /Users/sac, so the gate's inputs dangle (executed: loud [SKIP] exit 1).
- Compressed D16/D17/D20 prose (code+register show it); D15/D18/D19 kept verbatim.

## Keep list (irreplaceable, preserved)

- Deference chain ABI-FREEZE -> OPERATOR-DECISIONS -> authority receipts -> receipt
  law -> verdict/commit docs (items 1-5, order unchanged).
- Receipt law `ggen.legacy.stardf.arrow.receipt.v1` + RECEIPTS.md ledger.
- D15-D21 register summary with D15/D19 decisions intact.
- Export-freeze warning + ticket-gated surface changes (GL-STARDF-002).
- Standing vocabulary, code-discovery block.

## Added

- Commands block: just build/check/test, `cargo test --workspace --no-fail-fast`
  (census law: compile-abort != failure count), just ci-all ladder, verify.py,
  R1 license gate, blake3/header-freeze/duckdb-rehome courts, columnar-oracle,
  and the `nm -gU` C-ABI witness census (== 13, executed exit 0).

## Verification

- `wc -l /Users/sac/stardf/CLAUDE.md` — 118 lines (<= 200).
- `awk 'length>100'` — zero lines over 100 chars.
- Every referenced path exists (18/18 cited in commands above).

Standing: ALIVE (all cited commands executed this session with exits).
