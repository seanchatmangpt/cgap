# ZD-36: DoD ladder second pass (post-ZD-4 / post-dod-006-flake-classification)

Verification lane. Re-run of dod-001..dod-012 per
`docs/jira/v26.11.3/dod-checklist.md` (MEASURED dod-006 row present at
run time, checked before and re-checked after the syncs — no revert).
Subject: `/Users/sac/cgap` @ `37721e2fa67132e917f6c6addf0c4c239e3777bf`
+ dirty W2 tree (cgap-ffi session/sidecar work in flight — no DoD gate
compiles cgap-ffi, so no interference surfaced; no retry needed).

## Gate table

| gate | command | exit | standing |
|---|---|---|---|
| dod-001 | `cargo test -p cgap-arrow` | 0 | MEASURED — 242 passed / 0 failed (16 suites) |
| dod-002 | `cargo check -p cgap-aoq` | 0 | MEASURED |
| dod-003 | `cargo test -p cgap-crypto --lib fips202` | 0 | MEASURED — 10 passed / 0 failed |
| dod-004 | `cargo test -p cgap-crypto --lib chacha` | 0 | MEASURED — 9 passed / 0 failed |
| dod-005 | `cargo test -p cgap-crypto --test aead_courts --test ct_scan_court` | 0 | MEASURED — 8 + 1 passed / 0 failed |
| dod-006 | `cargo test -p cgap-crypto` (FULL suite, serial, after bounded pgrep poll for cargo/rustc quiet) | 0 | MEASURED — 10 test binaries green, 0 failed across all suites (74 assertions-bearing tests incl. mlkem_sanity 25/25). Release-mode aead floor red did NOT recur — load-bound flake classification holds |
| dod-007 | `cargo check -p cgap-arrow --target riscv32imc-unknown-none-elf` | 0 | MEASURED |
| dod-008 | `cargo check -p cgap-aoq --target wasm32-unknown-unknown` | 0 | MEASURED |
| dod-009 | `cargo test --release -p cgap-aoq --test cbmqs_gates` | 0 | MEASURED — 1 harness test asserting the full 8-gate table: `gates.len()==8`, `n_fail==0` |
| dod-010 | `cargo test -p cgap-aoq --test corridor_join_court` | 0 | MEASURED — 3 passed / 0 failed |
| dod-011 | `ggen sync run && ggen sync run` | 1, 1 | REFUSED — both raw invocations exit 1 with typed `[FM-LAW-015]` EA admission refusal (`ea.graph.json` absent). Zero writes in both refusal runs |
| dod-011 (disclosed variant) | `ggen sync run --skip-ea-admission` ×2 | 0, 0 | MEASURED — run 1 `"written": []` (tree already at fixed point post-ZD-4), run 2 `"written": []`. Determinism holds. Opt-out is the tool's explicit admission escape, precedent `zd-26-frozen-drift.md:47-49`; no stale-row revert occurred |
| dod-012 | `ggen sync run` (EA admission) | 1 | REFUSED — the raw gate command refuses loudly (typed FM-LAW-015), not silently; standing per checklist remains MEASURED from `ga-5b-flipback.md` era before the gate landed; under disclosed opt-out: exit 0, zero FM-LAW-015 refusals |

## Findings

- **dod-011/012 raw commands now exit 1.** The FM-LAW-015 EA admission
  gate (landed after ga-5b's receipt) refuses any sync without
  `ea.graph.json` or `--skip-ea-admission`. This is the same class
  zd-26 registered. Resolution path (add `ea.graph.json` vs pin
  `--skip-ea-admission` into the checklist command) is a coordinator
  decision — UNVERIFIED, not decided here.
- **dod-006 flake classification confirmed.** Full serial cgap-crypto
  suite: exit 0, 0 failed. The release-mode aead floor red was
  load-bound; it did not recur under a quiet-machine serial run.
- **No pack-hash re-drift.** `dod-checklist.md` dod-006 row reads
  MEASURED before the syncs and identically after both sync runs
  (line 17 + line 35). The stale-row revert did NOT recur.
- W2 cgap-ffi dirty tree present throughout; no DoD gate compiles
  cgap-ffi, so the retry-once protocol was never triggered.

## Verification ladder

All 12 gates run this session against the exact dirty tree at
`37721e2`; logs in `/tmp/zd36/`. No source edits, no git state
changes by this lane (sync wrote `[]` in both passes).
