# Lane RC-13 — cgap-fleet-ops membrane crate (A3/A6 skeleton)

- **Lane**: RC-13 (hardening code wave, FIX-14 ops register A3/A6)
- **Repo**: /Users/sac/cgap, branch main. No git commands run.
- **Files written** (all NEW, standalone crate `crates/cgap-fleet-ops/`,
  own `[workspace]`, NOT a root member, `publish = false`):
  - `crates/cgap-fleet-ops/Cargo.toml` — zero deps, publish=false,
    own `[workspace]` table (standalone, same pattern as crates/cli).
  - `crates/cgap-fleet-ops/src/lib.rs`
  - `crates/cgap-fleet-ops/src/fleet.rs` — A3 lifecycle: `NodeId`,
    `ReceiptDigest`, `NodeState` {Commissioned, Healthy, Degraded,
    Retired}, `Transition` (Promote/Degrade/Recover/RmaRetire/
    ForceRetire), `FleetRefusal` (MissingRmaEvidence,
    RetireFromWrongState, AlreadyRetired, UnknownNode), `FleetNode`
    with hash-chained transition log (fnv chain tip advances per
    event), `Fleet` register (BTreeMap) with `ingress_eligible()`
    (Retired never eligible).
  - `crates/cgap-fleet-ops/src/slo.rs` — A6: pinned status codes
    (0x4347_0004 / 0x4347_0006 / 0x5032_38FF), `SloSnapshot` 53-byte
    fnv-checksummed
    wire round-trip, `SloSource` trait (read-only kernel feed seam),
    `AlertPredicate` strictly-greater thresholds,
    `poll_until` bounded poll (no sleeps; stall => typed
    SourceUnhealthy).
  - `crates/cgap-fleet-ops/tests/courts.rs` — 4 Chicago courts.
- **Commands + exits (this session)**:
  - `cd crates/cgap-fleet-ops && cargo test` → exit **0**, 4 passed,
    0 failed (court1 legal lifecycle, court2 illegal refusals, court3
    SLO round-trip + corruption rejection, court4 predicate + bounded
    poll with stall verdict).
  - `cargo metadata --no-deps` at root → exit 0, 23 members,
    `cgap-fleet-ops` NOT a member (root workspace unaffected).
  - Mutation 1 (evidence checks removed from fleet.rs in
    /tmp/rc13-fleetops-sandbox): `cargo test court2_...` → exit 101,
    court2 FAILED — court flips, non-vacuous.
  - Mutation 2 (threshold `>` → `>=` in slo.rs, same sandbox):
    `cargo test court4_...` → exit 101, court4 FAILED — court flips.
  - Sandbox + `/tmp/rc13-target` deleted via python rmtree (build
    root lease honored). Shared tree never mutated; fleet.rs sha
    unchanged (9d346a1a…bd62).
- **Courts run**: court1–court4 green on shared tree; 2 sandbox
  mutation flips witnessed (court2, court4). A court that cannot
  flip is FAIL — both flipped.
- **Standing**: ALIVE (commands + exits above, run this session
  against the exact subject).
- **Divergences**: FIX-14 names crate owner "NEW membrane crate
  cgap-fleet-ops" — landed exactly as recommended. Fix-14 A6 host
  poller is implemented here (dispatch contract overrides the doc's
  cgap-ffi-owner suggestion for the SLO counter definitions; the
  host-side poller half of A6 lives here per dispatch, counter
  minting in cgap-ffi remains that crate's future lane). Two
  Write-tool corruptions witnessed and fixed in-session (stray
  token in fleet.rs, doubled `0 0x` in slo.rs); both caught before
  build. Receipt-cited counts re-read from /tmp/rc13-test.log at
  use time.
