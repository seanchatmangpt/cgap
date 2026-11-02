# Crate reality receipt: cgap-crypto

Lane: REAL-cgap-crypto (reality wave, 2026-11-01). Repo /Users/sac/cgap.

## Verdict: ALIVE (REAL)

## Commands + exits (this session)

| # | Command | Exit | Evidence |
|---|---|---|---|
| 1 | `CARGO_TARGET_DIR=/tmp/real-crypto-target cargo test -p cgap-crypto` | 0 | unittests 1 passed; chatman_courts 18 passed; lease_courts 6 passed; doc-tests 0; 0 failed anywhere |
| 2 | rdflib p:CrateManifest drift-check (below) | 0 | no drift |

## Drift check (rdflib vs Cargo.toml)

Subject: `/Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl`
`p:manifest-cgap-crypto` (lines 715-720).

| Fact | ontology | crates/cgap-crypto/Cargo.toml | Match |
|---|---|---|---|
| crateName | cgap-crypto | cgap-crypto | yes |
| crateVersion | 26.11.2 | workspace = true (workspace 26.11.2) | yes |
| cratePublish | true | publish = true | yes |

Note: repo-root `ontology/cgap.ttl` carries NO p:CrateManifest facts (20
lines, project identity only) — the canonical manifest facts live in the
cgap-pack ontology.ttl, per `docs/jira/v26.11.2/waves/g1k-01-manifest-facts.md:12,76`.
Divergence recorded, not a drift.

## Substance

- LOC: src 911 (chatman.rs 676, lease.rs 219, lib.rs 16) + tests 912
  (chatman_courts.rs 724, lease_courts.rs 188) = 1823 total.
- pub fn: 17 (chatman 9, lease 8).
- Tests: 25 executing (18 chatman courts + 6 lease courts + 1 unit).
  Test-to-code ratio healthy; courts are Chicago-style (real process trap
  witness: `exhaustion_court_trap_observed_in_real_process`).

## Standing

ALIVE — cargo test run this session, exit 0, exact subject
crates/cgap-crypto.

```json
{"lane":"REAL-cgap-crypto","files_written":["docs/jira/v26.11.2/waves/crate-reality/cgap-crypto.md"],"commands":["cargo test -p cgap-crypto -> exit 0 (25 passed, 0 failed)","rdflib p:CrateManifest drift-check -> exit 0 (no drift)"],"courts_run":["chatman_courts (18)","lease_courts (6)"],"standing":"ALIVE","divergences":["p:CrateManifest facts live in ~/ggen-marketplace/packs/cgap-pack/ontology.ttl, not repo ontology/cgap.ttl"]}
```
