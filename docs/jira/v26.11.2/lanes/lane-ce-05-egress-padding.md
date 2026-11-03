# Lane CE-05: egress PaddedIndex port (charter §4 Phase-4 design verify)

Standing: **ALIVE** — every claim re-run this session, 2026-11-02.

## W1 Identity

- Subject: `crates/cgap-git/src/egress_ring.rs` + NEW court
  `crates/cgap-git/tests/egress_layout_court.rs` + receipts.
- Branch main, shared tree sha of owned src file pre/post court:
  `4971406059ced3a0…` (sha256-exact across the mutation court).

## W2 Authority

Dispatch: CE-05 lane contract (charter §4 Phase-4 design verification;
OPT-01 hazard egress_ring.rs:422-427; ingress pattern
ingress_ring.rs:226-242).

## W3 Consequence (diff)

Path (a) chosen by diff size (~35 lines, small-coherent; mirrors
ingress's landed PaddedIndex):
- `CACHE_LINE_BYTES = 64` restated (pin `ingress_ring.rs:33`).
- `PaddedIndex` `#[repr(align(64))]` wrapper + const size/align asserts.
- `EgressRing.head`/`tail` wrapped; `quota` structurally on its own line
  (offset `CAP + 128`, pinned by the new struct-size const assert).
- NEW layout court, 3 tests, names its mutation.

## W4 Replay (commands + exits, this session)

1. `cargo test -p cgap-git --test egress_layout_court
   --test quota_alloc_purity_court --test andon_liveness_court
   --test egress_ring_court --test breach_regression_court`
   → egress_layout 3/3, quota purity 1/1, egress_ring 5/5,
   andon 5/5, breach 5/5 — all `test result: ok`.
2. Mutation court (/tmp sandbox): drop `repr(align(64))` → cargo aborts,
   3× E0080 const-assert failures (exact asserts shown in
   `dod/charter-p4-design-verify.md`). FLIP WITNESSED.
3. Shared-tree integrity: sha256 of owned src file identical pre/post
   court (`4971406059ced3a0…`); mutation confined to /tmp; leases
   /tmp/ce05-target, /tmp/ce05-mut, /tmp/ce05-mut-target deleted.

## W5 Standing

**ALIVE** — path (a) landed, layout court + fences green, mutation flip
witnessed this session, shared tree byte-exact.

## W6 Fences

Named must-not-regress fences and their runs: quota_alloc_purity_court
(1/1), andon_liveness_court (5/5), egress_ring_court (5/5),
breach_regression_court (5/5) — all green post-diff. No other lane files
touched; lib.rs untouched (`pub mod egress_ring;` already exported all
new public items via the module).

## W7 Divergences

- None vs contract. Sandbox transport note: the /tmp mutation court
  needed sibling crates (cgap-blake3/parquet/bn3) copied + manifests
  de-workspaced + versions unified — 3 failed attempts before the flip;
  recorded in `dod/charter-p4-design-verify.md` for replay.
- Prior receipt correction: none (no errata issued).
