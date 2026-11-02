# Lane SPDX — cgap-core dod10 manifest court

## Law cite

`crates/cgap-core/tests/manifest_court.rs:46` — `dod10_no_spdx_license_identifiers_anywhere`:
bans `license =`, `license-file`, and `LicenseRef-` shapes in Cargo.toml / README.md.
DoD 10's letter: the P28-SIL URN (`urn:avec:license:p28-sil:v2.1.0-supreme` … correction:
`urn:avec:license:p28-sil:v2.1.0-sovereign`, frozen per decision D2) is the only lawful
license reference; the `license`/`license-file` keys are the SPDX expression slots.

## Note on dispatch text

The dispatch said the law "requires SPDX license identifiers"; the actual court asserts the
inverse (SPDX shapes banned). The doc yields to the referent. cgap-gate's manifest carries
`license-file` too but has no such court — it is not a counterexample to the law.

## Diff (crates/cgap-core/Cargo.toml)

Removed line 7: `license-file = "../../LICENSE.md"` — the sole failing
SPDX shape (court panicked at manifest_court.rs:56 citing Cargo.toml:7).
The frozen URN comment block (lines 6-9) stays; `dod10_p28_urn_cited_in_manifest_and_readme`
still passes.

## Commands + exits

- `cargo test -p cgap-core --test manifest_court` (pre-fix): FAILED, 2 passed / 1 failed —
  `DoD 10: SPDX-shaped license reference in Cargo.toml:7: license-file = "../../LICENSE.md"`
- `cargo test -p cgap-core` (post-fix): all targets green — 17+8+2+11+4+6+3+2 passed, 0 failed;
  one PRE-EXISTING failure in `source_scan_gate::gate_exits_clean_on_real_cgap_core_src`
  (banned token `alloc::` in `crates/cgap-core/src/tracking_alloc.rs`, an untracked file
  owned by another lane — outside this lane's file ownership, recorded not repaired)
- Flip (anti-vacuity, /tmp sandbox `/tmp/spdx-flip-core`, mutated copy only):
  injected `license-file = "../../LICENSE.md"` → court FAILED with
  `SPDX-shaped license reference in Cargo.toml:32`; the court flips.
- Restore: mutation removed → sandbox court 3 passed / 0 failed. Sandbox target dir
  `/tmp/spdx-flip-target` deleted (shutil.rmtree, exit 0).

## Standing

ALIVE for the dod10 manifest court fix: command + exit run this session, flip witnessed,
restore witnessed. tracking_alloc.rs source-scan failure: PRE-EXISTING, not this lane's.
