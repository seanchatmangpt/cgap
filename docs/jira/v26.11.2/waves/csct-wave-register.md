# CSCT Wave Register (RFC-0042)

Date: 2026-11-02 · Lane: CSCT-L14 · Repo: /Users/sac/cgap (canonical checkout)

## Lane table

Standing for every lane at wave-1 is
`UNVERIFIED:compile-pending`: the `csct` crate exists on disk
(`crates/csct/`) but is not yet a root-workspace member; the
coordinator adds root `Cargo.toml` membership and runs the first
`cargo test -p csct` after lane drain. Until then no standing claim
rounds up beyond UNVERIFIED.

| Lane | Scope | Primary owned files | Standing | Anti-vacuity mutation |
|---|---|---|---|---|
| 01 | scaffold + lib | `crates/csct/Cargo.toml`, `crates/csct/src/lib.rs` | UNVERIFIED:compile-pending | n/a (scaffold lane) |
| 02 | types + receipt | `crates/csct/src/{types.rs,receipt.rs}` | UNVERIFIED:compile-pending | ledger-drift |
| 03 | warmer | `crates/csct/src/warmer.rs` | UNVERIFIED:compile-pending | shard-starved |
| 04 | reframe | `crates/csct/src/reframe.rs` | UNVERIFIED:compile-pending | zero-tax |
| 05 | drowning | `crates/csct/src/drowning.rs` | UNVERIFIED:compile-pending | shard-starved |
| 06 | emotional | `crates/csct/src/emotional.rs` | UNVERIFIED:compile-pending | gc-claim |
| 07 | new-way | `crates/csct/src/new_way.rs` | UNVERIFIED:compile-pending | input-flip |
| 08 | solution | `crates/csct/src/solution.rs` | UNVERIFIED:compile-pending | input-flip |
| 09 | mutation | `crates/csct/src/mutation.rs` | UNVERIFIED:compile-pending | ledger-drift |
| 10 | andon | `crates/csct/src/andon.rs` | UNVERIFIED:compile-pending | gc-quiesced |
| 11 | integration court | `crates/csct/tests/integration_court.rs` | UNVERIFIED:compile-pending | input-flip |
| 12 | runner bin | `crates/csct/src/bin/csct-runner.rs` | UNVERIFIED:compile-pending | ledger-drift |
| 13 | docs | `docs/jira/v26.11.2/waves/csct-wave-docs.md` | UNVERIFIED:compile-pending | n/a (docs lane) |
| 14 | this register | this file + `csct-lane-14-register.md` receipt | ALIVE (files written this session) | n/a |

## Exit-code contract

All stage courts and the `csct-runner` binary share one contract:

| Code | Meaning |
|---|---|
| 0 | pass (court holds under its mutation-battery) |
| 1 | gate fail / court cannot hold |
| 2 | subject missing (input artifact absent) |
| 3 | loud skip (`[SKIP] <reason>` printed; never a fake pass) |
| VACUOUS | mutation-free green run is VACUOUS, exits nonzero |

## Receipt census (read this session)

- Lane 01 receipt: FOUND but under a DIVERGENT filename —
  `docs/jira/v26.11.2/waves/csct-lane-01-scaffold.md`
  (not the `csct-lane-NN-receipt.md` convention). Divergence recorded.
- Lanes 02-13 receipts: LOUD-PENDING (13 of 13 `csct-lane-NN-receipt.md`
  absent at read time). Peers may still be writing; the coordinator
  re-censuses at drain. No lane created by L14.

## Known risks

- **(a) Clippy debt policy**: the new `csct` crate must introduce zero
  NEW warnings beyond the workspace baseline. Any warning above the
  pre-csct baseline is a lane defect against the owning lane, fixed or
  reverted within the compile-freeze SLA.
- **(b) No serde dependency anywhere in csct**: no `serde`,
  `serde_derive`, or `serde` re-export may appear in
  `crates/csct/Cargo.toml` or any `crates/csct/src/**` import. The
  crate stays zero-dep beyond its declared `cgap-blake3` path dep.
- Receipt-filename convention drifted at lane 01 (above); if lanes
  02-13 land under non-`-receipt` names, the drain census must map
  them explicitly or the register table stands as-of-date only.
