# DOC-4-moonshot-b — moonshot crate README wave (receipt)

Lane DOC-4, wave doc-4-moonshot-b, repo /Users/sac/cgap @ main (cfd085a9
+ in-flight wave state). Lease honored: only the six READMEs + the
readme-archive copy + this receipt.

## Commands run this session (with exits)

| command | exit | result |
|---|---|---|
| `cargo test -p cgap-gate` | 0 | 11 passed; 0 failed |
| `cargo test -p cgap-crypto` | 0 | 18 + 6 = 24/0 |
| `python3 crates/cgap-crypto/scripts/wave7_shim_battery.py` | 0 (refused) | `BASELINE NOT GREEN (exit 101); battery refused` — fail-closed refusal under wave contention |
| `cargo test -p cgap-p28` | 0 | 36 + 6 = 42/0 |
| `cargo test -p cgap-audit` | 0 | 37 passed; 0 failed |
| `cargo test -p cgap-ffi` | 0 | 45 + 4 + 5 = 54/0 |
| `cargo test -p cgap-court` | 0 | 14 passed; 0 failed |
| `curl -A "cgap-publish-receipt (...)" api.crates.io/api/v1/crates/<c>` | 0 | see table below |

All test runs with `CARGO_TARGET_DIR=/tmp/doc4-target`.

## crates.io state (verified live, 2026-11-02)

| crate | state |
|---|---|
| cgap-gate | NOT-PUBLISHED (no record) |
| cgap-crypto | PUBLISHED 26.11.1 (created 2026-11-02T03:25:33Z) |
| cgap-p28 | NOT-PUBLISHED |
| cgap-ffi | NOT-PUBLISHED |
| cgap-court | no record (publish N/A — test harness, no src/) |
| cgap-audit | NOT-PUBLISHED |
| cgap (umbrella, context) | PUBLISHED 26.11.1, created 2026-11-01 |

## Courts

No new courts authored (docs lane). Re-ran the crates' own courts:
gate 11/0, crypto 24/0, p28 42/0 (incl. the 640-bit sweep + 11-survivor
law), audit 37/0 (incl. `a20_stress_one_million_rows_powl_verify`),
ffi 54/0, court harness 14/0. The crypto 9-mutation battery refused
fail-closed (baseline 101 under concurrent cargo lock contention) — loud
refusal recorded, not a pass.

## Divergences

1. **include/cgap.h missing** — named in `crates/cgap-ffi/src/sovereign.rs`,
   `oabi_lookup.rs`, Cargo.toml comment, and the a19 receipt as the frozen
   C-side twin of the §6.1 header court; the file does not exist on disk.
2. **crypto battery refusal** — a16 receipt witnesses exit 0; standalone
   re-run under this wave refused fail-closed (baseline exit 101).
3. **Existing cgap-ffi README** described the pre-moonshot Arrow C Data
   Interface membrane; archived byte-exact at
   `docs/jira/v26.11.1/readme-archive/cgap-ffi-pre-v26.11.1.md` and
   rewritten for the sovereign surface.
4. **dispatch test counts matched observed**: gate 11/0, crypto 24/0,
   p28 42/0, audit 37/0. Not pre-stated: ffi 54/0, court 14/0 (measured).
