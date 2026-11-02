# W-SWEEP post-receipt (registry sweep re-run)

Lane W-SWEEP, 2026-11-01, repo /Users/sac/cgap (read-only except receipt + cell table).

## Verification ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| registry sweep | `python3 scripts/registry_sweep.py` (/Users/sac/cgap) | 1 | FAIL, 1 missing: stardf-datalog MISSING (no-ledger-row); 17 REGISTERED-DIVERGENCE, 5 PASS (bench/cgap-beam/cgap-blake3/cgap-integration/cgap-court) + cgap-cli PASS DELISTED |
| precondition | grep 26.11.2 across member manifests (/Users/sac/cgap) | 0 | root workspace version 26.11.2 (Cargo.toml:42); sweep rows local_version=26.11.2 for 22 members |

Run repeated 3x this session (exit 1 each, same missing count) — live
crates.io API, UA cgap-registry-sweep. Non-vacuous: exit 1 with a
named missing subject; a mutation-free green run would be VACUOUS.

## Live-truth deltas vs the prior census-carried table

- cgap-blake3: registry max now 26.11.2 (num=3) — only member at target.
- cgap-gpu: UNPUBLISHED -> PUBLISHED-26.11.1 (num=1).
- cgap-integration: PUBLISHED in old census prose -> UNPUBLISHED (404) live.
- bench/cgap-beam/cgap-court/cgap-cli: DELISTED (script classification).
- 17 members REGISTERED-DIVERGENCE (registry 26.11.1 vs local 26.11.2).
- stardf-datalog: loudly MISSING, no ledger row — sole exit-1 cause.

## Writes (sole writes, disclosed)

- docs/jira/v26.11.2/08-registry-sweep.md §1 — cell table rows replaced
  with live sweep truth (+ loud-MISSING row + note paragraph).
- This receipt.
- registry-sweep-table.json written into the tree by the script at
  docs/jira/v26.11.2/waves/registry-sweep-table.json — script has no
  output-path flag; write disclosed, not rerouted.

## JSON footer

```json
{
  "lane": "W-SWEEP",
  "files_written": [
    "docs/jira/v26.11.2/08-registry-sweep.md",
    "docs/jira/v26.11.2/waves/w-sweep-post.md"
  ],
  "script_tree_writes_disclosed": [
    "docs/jira/v26.11.2/waves/registry-sweep-table.json"
  ],
  "commands": [
    "grep -c 'version = \"26.11.2\"' crates/*/Cargo.toml (exit 0)",
    "python3 scripts/registry_sweep.py (exit 1, FAIL 1 missing)",
    "python3 scripts/registry_sweep.py --help (exit 0)"
  ],
  "verdict": "FAIL",
  "missing_count": 1,
  "registered_divergence_count": 17,
  "pass_count": 5,
  "standing": "PARTIAL"
}
```
