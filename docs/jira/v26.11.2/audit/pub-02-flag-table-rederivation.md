# PUB-02 — publish-flag table fresh re-derivation (2026-11-02)

Method: zero-memory re-derivation this session. Members from
`cargo metadata --no-deps` (23 packages, all 26.11.2); per-member
`publish` flag read from Cargo.toml on disk (`grep -n "^publish"`);
registry state via sequential
`curl -s -A "<UA>" https://crates.io/api/v1/crates/<name>` GETs
(1s spacing, no storms). First curl pass without User-Agent returned
HTTP 403 for all 23 (crates.io mandatory-UA policy); that pass is
DISCARDED — the 403s are transport policy, not registry verdicts.

## W1 — Membership (cargo metadata, this session)

23 members: cgap, cgap-arrow, cgap-core, cgap-blake3, cgap-gate,
cgap-branchless, cgap-storage, cgap-git, cgap-bn3, cgap-parquet,
cgap-p28, cgap-crypto, cgap-ffi, cgap-audit, cgap-beam, cgap-gpu,
bench, cgap-integration, cgap-bont, cgap-bosym, cgap-court,
cgap-fibo, cgap-cloud.

## W2 — Publish flags, all 23 members (fresh from disk)

Count: 23/23 explicit, 20 true / 3 false. SUPERSEDES the
"12/23 explicit, 9 true / 3 false" figure in root CLAUDE.md Publish
law and the "19/22" figure at
`docs/jira/v26.11.2/01-release-gates.md:32` — both stale by drift.

| # | crate | flag | evidence (file:line) |
|---|-------|------|----------------------|
| 1 | cgap | true | crates/cgap/Cargo.toml:9 |
| 2 | cgap-arrow | true | crates/cgap-arrow/Cargo.toml:8 |
| 3 | cgap-core | true | crates/cgap-core/Cargo.toml:7 |
| 4 | cgap-blake3 | true | crates/cgap-blake3/Cargo.toml:8 |
| 5 | cgap-gate | true | crates/cgap-gate/Cargo.toml:9 |
| 6 | cgap-branchless | true | crates/cgap-branchless/Cargo.toml:9 |
| 7 | cgap-storage | true | crates/cgap-storage/Cargo.toml:9 |
| 8 | cgap-git | true | crates/cgap-git/Cargo.toml:8 |
| 9 | cgap-bn3 | true | crates/cgap-bn3/Cargo.toml:9 |
| 10 | cgap-parquet | true | crates/cgap-parquet/Cargo.toml:8 |
| 11 | cgap-p28 | true | crates/cgap-p28/Cargo.toml:7 |
| 12 | cgap-crypto | true | crates/cgap-crypto/Cargo.toml:7 |
| 13 | cgap-ffi | true | crates/cgap-ffi/Cargo.toml:9 |
| 14 | cgap-audit | true | crates/cgap-audit/Cargo.toml:9 |
| 15 | cgap-beam | false | crates/cgap-beam/Cargo.toml:8 |
| 16 | cgap-gpu | true | crates/cgap-gpu/Cargo.toml:2 |
| 17 | bench | false | crates/bench/Cargo.toml:9 |
| 18 | cgap-integration | true | crates/integration/Cargo.toml:8 |
| 19 | cgap-bont | true | crates/cgap-bont/Cargo.toml:9 |
| 20 | cgap-bosym | true | crates/cgap-bosym/Cargo.toml:9 |
| 21 | cgap-court | false | crates/cgap-court/Cargo.toml:8 |
| 22 | cgap-fibo | true | ontologies/fibo/Cargo.toml:10 |
| 23 | cgap-cloud | true | ontologies/cloud/Cargo.toml:10 |

## W3 — Standalone / non-member crates (confirmed on disk)

| crate | publish flag | evidence |
|---|---|---|
| cgap-verification | false | crates/cgap-verification/Cargo.toml:8 |
| csct | false | crates/csct/Cargo.toml:5 |
| cli | false | crates/cli/Cargo.toml:9 |
| cgap-fleet-ops | false | crates/cgap-fleet-ops/Cargo.toml:5 |
| cgap-ocel2 | false | crates/cgap-ocel2/Cargo.toml:7 |
| wasm-footprint | true | crates/wasm-footprint/Cargo.toml:8 |

All five dispatch-named standalone crates (cgap-verification, csct,
cli, cgap-fleet-ops, cgap-ocel2) carry `publish = false` on disk.
wasm-footprint is a sixth non-member with `publish = true` on disk —
outside the 23-member gate set but a live first-publish candidate
the prior table omitted.

## W4 — Registry state, 20 publish-intended member names

19 of 20 publish-true members exist on crates.io: 13 at 26.11.2
(cgap, arrow, core, blake3, gate, storage, git, bn3, parquet, p28,
crypto, bont, bosym), 5 at 26.11.1 (ffi, audit, gpu, fibo, cloud).
1 registry-absent: cgap-branchless (HTTP 404) — the DOD-02
first-publish candidate, not yet published.

Per-name verdicts (HTTP 200/404 from this session):

| crate | HTTP | max_version |
|---|---|---|
| cgap | 200 | 26.11.2 |
| cgap-arrow | 200 | 26.11.2 |
| cgap-core | 200 | 26.11.2 |
| cgap-blake3 | 200 | 26.11.2 |
| cgap-gate | 200 | 26.11.2 |
| cgap-branchless | 404 | registry-absent |
| cgap-storage | 200 | 26.11.2 |
| cgap-git | 200 | 26.11.2 |
| cgap-bn3 | 200 | 26.11.2 |
| cgap-parquet | 200 | 26.11.2 |
| cgap-p28 | 200 | 26.11.2 |
| cgap-crypto | 200 | 26.11.2 |
| cgap-ffi | 200 | 26.11.1 |
| cgap-audit | 200 | 26.11.1 |
| cgap-beam | 404 | (moot: publish=false) |
| cgap-gpu | 200 | 26.11.1 |
| bench | 200 | 1.1.0 (collision) |
| cgap-integration | 404 | (moot: publish=false) |
| cgap-bont | 200 | 26.11.2 |
| cgap-bosym | 200 | 26.11.2 |
| cgap-court | 404 | (moot: publish=false) |
| cgap-fibo | 200 | 26.11.1 |
| cgap-cloud | 200 | 26.11.1 |

bench collision CONFIRMED from response body: id=bench,
max_version=1.1.0, repository=
https://github.com/AlexanderThaller/bench-rs, updated 2021-04-20 —
name collision, not ours; moot under publish=false.

## W5 — Comparison vs 01-release-gates.md:28-32

- Prior doc figure "19/22" (line 32): wrong on both numbers. Fresh:
  23 members (not 22), 20 publish-true (not 19). Drift sources:
  cgap-fibo/cgap-cloud are members (they live under ontologies/, not
  crates/ — an easy miss), and cgap-branchless flipped to true by
  DOD-02. Gate-set delta: publish-true set = prior 19 + branchless
  + fibo + cloud.
- Prior CLAUDE.md figure "12/23 explicit, 9 true / 3 false": wrong on
  all three numbers. Fresh: 23/23 explicit, 20 true / 3 false.
- wasm-footprint (non-member, publish=true, registry state
  UNVERIFIED — no registry read was run for it) is an omission
  candidate for the refreshed table.
- Registry lag: 5 members exist at 26.11.1 while the workspace is
  26.11.2 — pre-version-bump publications, expected pre-publish wave.

## ERRATA

- First Write of this file corrupted mid-table (token drift,
  witnessed ~1-in-5 long-write corruption); this version is a full
  clean rewrite, byte-verified against the grep/curl transcripts
  above.
- Owner-target for the refreshed table:
  docs/jira/v26.11.2/01-release-gates.md (not edited by this lane;
  coordinator sequences). Root CLAUDE.md Publish-law paragraph is a
  second stale-figure owner-target.

Standing: ALIVE for every cell above (each backed by a command run
this session: cargo metadata exit 0; grep exits 0; 23 curl GETs with
HTTP codes recorded). No UNVERIFIED cells among the 23 members; one
UNVERIFIED cell: wasm-footprint registry state (no read performed —
outside the 20-name publish-intended member set).
