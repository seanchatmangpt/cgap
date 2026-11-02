# W2-05 Registry Sweep Receipt

Lane: W2-5 · repo `/Users/sac/cgap` · no git commands · registry
read-only (no publish attempts).

## Files written

- `scripts/registry_sweep.py` (236→249 lines, syntax-verified)
- `docs/jira/v26.11.2/waves/registry-sweep-table.json` (sweep output,
  machine table, sorted by crate)
- this receipt

## Real sweep (this session)

`python3 scripts/registry_sweep.py` → exit 0,
`REGISTRY-SWEEP: PASS 0 missing`. 22 members via
`cargo metadata --no-deps` (exit 0, 22 packages) + standalone cgap-cli
row = 23 subjects.

Key live cells (API 2026-11-01, UA `cgap-registry-sweep/26.11.2`):

| crate | registry state | cell |
|---|---|---|
| cgap-gpu | PUBLISHED-26.11.1 (num=1, yanked=false) | REGISTERED-DIVERGENCE — ledger
| | | expected UNPUBLISHED; census is provenance |
| cgap-ffi | PUBLISHED-26.11.1 (num=1) | REGISTERED-DIVERGENCE (local 26.11.2 vs registry 26.11.1) |
| cgap-integration | UNPUBLISHED (404) | PASS |
| bench / cgap-cli | DELISTED | PASS (court pointer resolves on disk) |
| beam / court | DELISTED | PASS (pointer resolves) |
| remaining 16 | PUBLISHED-26.11.1 vs local 26.11.2 | REGISTERED-DIVERGENCE, pointer-covered |

REGISTERED-DIVERGENCE count 18 == court-pointer count 18 → exit 0.

## Flip courts (/tmp sandbox only)

- Flip A: `cgap-gpu` ledger row deleted in
  `/tmp/w2-05-sandbox/registry_sweep_flipA.py` →
  `[MISSING] cgap-gpu: no ledger row` → exit 1. FLIP CONFIRMED.
- Flip B: `publish = false` stripped from a /tmp copy of
  `crates/cli/Cargo.toml`, run via `--cli-manifest` →
  `[MISSING] cgap-cli: manifest lost the publish guard; ... a crates.io
  row is now demanded` → exit 1. FLIP CONFIRMED.
- Shared `crates/cli/Cargo.toml` untouched (sha256
  `85db11ecadf31bc8…`). Restore law satisfied: mutations never touched
  the shared tree.

## Divergences

- Dispatch names cgap-verification in the publish=false DELISTED set;
  it is NOT a `cargo metadata` member (on-disk only) and the sweep
  enumerates members + cli, so it has no row. Recorded, not forced.
- Exit contract follows the dispatch (exit 1 on any MISSING); spec §5
  would permit loud-MISSING-with-[SKIP] at exit 0 — dispatch yields.
- First flip run crashed (`KeyError: registry_state` on unregistered
  rows) instead of exiting 1 — fixed with `.get()` and re-run; the
  court now flips by contract, not by crash.
- `publish_flag` default corrected to cargo semantics (absent
  line = publishable); all 22 members carry explicit `publish` lines,
  so the real run's classification is unchanged.
- cgap-gpu's census-era UNPUBLISHED expectation is stale: registry now
  returns 200 / 26.11.1. Ledger keeps it as REGISTERED-DIVERGENCE with
  the census pointer; the coordinator should re-pin the ledger or the
  pointer after the publish window.
- The sweep wrote `registry-sweep-table.json` into `waves/` (output
  artifact, not a lane-owned doc edit) — disclosed here.

```json
{"lane":"W2-5",
 "files_written":["scripts/registry_sweep.py",
  "docs/jira/v26.11.2/waves/registry-sweep-table.json",
  "docs/jira/v26.11.2/waves/w2-05-registry-sweep.md"],
 "commands":["cargo metadata --no-deps --format-version 1 (22 packages, exit 0)",
  "python3 scripts/registry_sweep.py (exit 0, REGISTRY-SWEEP: PASS 0 missing)",
  "python3 /tmp/w2-05-sandbox/registry_sweep_flipA.py --repo /Users/sac/cgap (exit 1)",
  "python3 scripts/registry_sweep.py --cli-manifest /tmp/w2-05-sandbox/cli_Cargo.toml (exit 1)"],
 "courts_run":["flip-A ledger-row-deletion (exit 1)",
  "flip-B cli publish-guard strip (exit 1)"],
 "standing":"ALIVE",
 "divergences":["cgap-verification on-disk but non-member: no sweep row",
  "dispatch exit contract (1 on any MISSING) over spec §5 loud-skip exit 0",
  "cgap-gpu registry row appeared since census: expected UNPUBLISHED, found PUBLISHED-26.11.1",
  "flip-A first run crashed before contract exit; fixed and re-run",
  "publish_flag default fixed to cargo publishable semantics"]}
```
