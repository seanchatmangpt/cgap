# A03 Finish Receipt (wave, v26.11.1)

- **Lane**: A03 — Git Packfile + Content-Addressed Merkle DAG (finisher
  pass over the 62215326 landing)
- **Repo**: `/Users/sac/cgap` (one canonical checkout, branch `main`)
- **Date**: 2026-11-01
- **Standing**: **ALIVE** — every claim below carries a command+exit run
  this session on the exact subject bytes.
- **Lease honored**: only `crates/cgap-git/src/dag.rs` (+17 lines),
  new `crates/cgap-git/src/pack.rs`, this receipt. `lib.rs`, root
  `Cargo.toml`, `pack_write.rs` untouched (sha256 `7750d2af…9442b`,
  identical to the wave-5 audit's recorded value).

## 1. Assessment at start (the "prior wave died mid-flight" question)

- `dag.rs` landed in **62215326** ("wave checkpoint 1-9 — 225-agent
  implementation landing"), not 9be9109d as the dispatch said (dispatch
  yields; recorded as divergence D-7 below). Working tree was clean for
  the whole crate before this pass.
- The wave-5 audit's defect **F-1 is already repaired in the landed
  bytes**: the 2L-1 node-count law is evaluated in u64
  (`dag.rs:1175-1177`) with the hostile-input witnesses
  (`hostile_dag_file`, witnesses A `0x8000_0000`/`0xFFFF_FFFF` and B
  `0x8000_0001`/`1`) courted at `f1_hostile_counts_typed_refusal`
  (dag.rs:1625). Witnessed green this session.
- The only open Definition-of-Done item was **DoD-2**: the `pack.rs`
  stable facade did not exist. This pass lands it.

## 2. Landed this pass

1. **`crates/cgap-git/src/pack.rs` (new, DoD-2).** Stable facade over
   both content-addressing registers: git-compat (`write_pack_v2`,
   `write_idx_v2`, `IdxEntry`, `PackBuilder`, `ParsedObject`, `IdxV2`,
   parsers, `git_object_oid`, `sha1`) + receipt register (`commit_blocks`,
   `merkle_root`, `lineage_proof`, `verify_lineage`, `Commit`, `DagTree`,
   `LineageProof`, `FrameFields`, wire codecs). `pub use` only — no code
   moved (ADR-03-3). Wired as `cgap_git::dag::pack` via `#[path]` from
   `dag.rs`; promotion to the DoD-2 top-level `crate::pack` is a one-line
   coordinator edit (divergence D-1).
2. **Facade court `facade_court_both_registers_round_trip`**
   (pack.rs:42): one real snapshot through both registers —
   commit -> pack -> parse -> idx v2 write/read `(oids, crcs, offs)`
   round-trip, cross-register OID-set binding
   (`commit.oids` sorted == idx OIDs), parent-edge root binding
   (foreign parent moves the root — anti-vacuity leg), §4.2
   `frame_fields` with the pack-trailer transport checksum
   (`sha1(pack[..len-20])` == trailer, pack_sha1 != root prefix),
   §5.1 encode/decode round-trip.
3. **D-1 strengthened to the dispatch court formula**: byte-exact
   structural offset stability across independent derivations —
   `encode_dag_file(&c1.tree) == encode_dag_file(&c2.tree)`
   (dag.rs:1305-1312). Two fresh derivations of the same block set
   produce byte-identical `.cgap-dag` files. The mutate-one-byte leg
   (bit flip -> root address moves) is unchanged and witnessed.
4. **F-1 verdict**: closed as already-repaired (u64 law + court), no
   code change needed this pass.

## 3. Verified surfaces (this session, exact commands)

| Surface | Command | Exit | Evidence |
|---|---|---|---|
| Pre-edit baseline | `CARGO_TARGET_DIR=/tmp/cgap-lane-A03-target cargo test -p cgap-git` | 0 | 87 passed / 0 failed, 2 loud ignored (`ladder_result_constancy_under_cpu_noise`, `perf_baseline_git_plane`) |
| Build + zero warnings | `cargo build -p cgap-git` after `touch dag.rs pack.rs` | 0 | 0 warnings (grep count 0) |
| Full suite post-edit | `CARGO_TARGET_DIR=/tmp/cgap-lane-A03-target cargo test -p cgap-git` | 0 | **88 passed / 0 failed** (10 binaries; 59+1 facade in-lib = 60), 2 loud ignored |
| Lib courts | `cargo test -p cgap-git --lib` | 0 | 60/60 incl. d1-d9, f1, dod4, r3, wave9_powercut, abi_frozen, facade |
| D-10 sweep | `grep -rn "mock\|Mock\|MOCK\|Process.sleep\|std::thread::sleep" crates/cgap-git/src/` | 0 | one comment-only hit `idx_read.rs:219` ("no mocks"); zero real hits |
| Zero new deps | read `crates/cgap-git/Cargo.toml` | — | `[dependencies]` empty |
| Ownership | `git diff --stat -- crates/cgap-git/` | — | `dag.rs +17`; `pack.rs` untracked new; nothing else |
| Byte pins | sha256 this session | — | dag.rs `3fce9310…b46a5`, pack.rs `33b52d59…19a67`, pack_write.rs `7750d2af…9442b` |

## 4. Adversarial sandbox courts (`/tmp/a03-finish-sb-1793575431`)

Standalone workspace (own manifest, `resolver = "2"`, own target dir
`/tmp/a03-sb-target`). Baseline: `cargo test -p cgap-git --lib` exit 0,
**60/60 green**. One mutation at a time; restore = copy from the repo +
sha256 fail-closed verify; post-restore re-run 60/60.

| Shim | Mutation (sandbox only) | Predicted | Observed (exit 101 each) |
|---|---|---|---|
| **SHIM-OFFSET** | `encode_dag_file` node region shifted +4 past the frozen 72B header | d9 round-trip must fail | **d9 FLIPPED** + 3 wire-leg collateral (`f1_hostile_counts_typed_refusal`, `wave9_powercut_sweep_dag_wire`, facade court) — 56/60 |
| **SHIM-STABILITY** | allocation-address entropy XORed into the encoded root field (safe `as` cast; `forbid(unsafe_code)` intact) | d1 byte-stability assert must flip | **d1 FLIPPED** + same 3 wire-leg collateral — 56/60 |
| **SHIM-PARENT** | parent gate at dag.rs:1084 masked (`return Err(...)` -> `let _ = ...`) | d3 + d8 must fail | **exactly d3 + d8 FLIPPED** — 58/60, isolated, matches wave-5 audit isolation on the old bytes |

All three target courts flipped under their named mutations; zero
vacuous courts. Restores: sandbox dag.rs `3fce9310…b46a5` byte-exact
after every mutation; post-restore baseline 60/60.

Dispatch mission mapping:
- "derive twice from same input -> byte-identical structure" = D-1's new
  wire-level assert (SHIM-STABILITY kills it if derivations diverge).
- "mutate one byte -> different address, detected" = D-1 bitflip leg
  (root moves) + D-9 mutation legs (mismatch refused).
- "corrupt an offset computation -> stability court must fail" =
  SHIM-OFFSET (d9 flipped, exit 101).

## 5. Divergences

- **D-1 (DoD-2 path).** `pack.rs` is reachable as `cgap_git::dag::pack`
  (`#[path]` wired from `dag.rs`), not `crate::pack`: `lib.rs` is
  contract-off-limits to this lane and is the only place a top-level
  module can be declared. Promotion is a one-line coordinator edit
  (`pub mod pack;` + `pub use dag::pack::*;` in lib.rs); facade court
  and re-exports are already DoD-2-shaped.
- **D-2 (pre-existing, unchanged).** Lane doc §10 still reads
  "Status: UNVERIFIED — no code landed"; stale since wave 2. The lane
  doc is not in this lane's lease.
- **D-3 (provenance).** dag.rs sha256 moved `d06c9ad5…` (wave-5 audit
  subject) -> `3fce9310…` (this pass): the intermediate state was the
  62215326 landing, which repaired F-1 and added its court; this pass
  added the facade wiring + D-1 wire assert (+17 lines).
- **D-4 (pre-existing).** Internal payload `left||right` (64 B) vs
  §3.2.3's `…||seq_parent||depth`; postorder vs §5.1's "preorder" —
  disclosed in-code, bound by the frozen 618-byte wire pin; doc yields
  (unchanged from wave-5 audit notes 3/4).
- **D-5 (court-authoring honesty).** The facade court's first draft
  asserted `sha1(&pack) == pack trailer`; it FAILED at pack.rs:106
  because the trailer is `sha1(pack[..len-20])` — the court itself
  caught the wrong frame-field semantics; fixed to the trailer
  convention with a real content-checksum assert. No mock involved.
- **D-6.** `cargo test` totals include 2 loud `ignored` perf/timing
  tests (named above); skips are loud, never silent.
- **D-7 (dispatch).** Dispatch said the wave landed in 9be9109d; the
  crate's landed state actually entered at **62215326**. Dispatch
  yields to `git log` evidence.
- **D-8.** Per contract, `cargo test --workspace` was never run; all
  runs are `-p cgap-git` with the lane lease target dir.

## 6. Commands (exact, re-runnable)

```
CARGO_TARGET_DIR=/tmp/cgap-lane-A03-target cargo test -p cgap-git   # exit 0, 88/0 (post-edit)
CARGO_TARGET_DIR=/tmp/cgap-lane-A03-target cargo build -p cgap-git  # exit 0, 0 warnings
CARGO_TARGET_DIR=/tmp/cgap-lane-A03-target cargo test -p cgap-git --lib  # exit 0, 60/60
# sandbox /tmp/a03-finish-sb-1793575431 (deleted after receipt):
#   baseline 60/60; SHIM-OFFSET / SHIM-STABILITY / SHIM-PARENT each
#   exit 101 with the flips tabled above; sha256 byte-exact restores
#   between and after; post-restore 60/60.
```

## See Also

- `docs/jira/v26.11.1/lanes/A03-git-merkle.md` (normative lane doc)
- `docs/jira/v26.11.1/waves/a03-wave5-audit-receipt.md` (prior audit;
  F-1 and DoD-2 statuses updated by this pass)
- `crates/cgap-git/src/dag.rs`, `crates/cgap-git/src/pack.rs`
