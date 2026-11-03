# Charter §4 Phase-4 Design Verification: Egress PaddedIndex (CE-05)

Standing: **ALIVE** — commands + exits run this session, 2026-11-02.

## Path chosen: (a) code port

After reading OPT-01's hazard (egress_ring.rs:422-427 pre-CE-05: bare
`AtomicUsize` `head`/`tail` co-resident with `RefusalQuota` on one 64-B
line) and measuring the diff, path (a) wins: the port is ~35 lines
mirroring ingress_ring.rs:226-242, well under small-coherent. Path (b)
(charter future-tense fix) is NOT taken; the charter claim is now TRUE
for the egress ring, not future-tense.

## Diff

`crates/cgap-git/src/egress_ring.rs`:
- `CACHE_LINE_BYTES = 64` restated (pin: `ingress_ring.rs:33`).
- `PaddedIndex` (`#[repr(align(64))]` over `AtomicUsize`, private,
  `load`/`store` delegation) + two const asserts (size == 64, align == 64).
- `EgressRing.head`/`tail` now `PaddedIndex`; `quota` unchanged — with
  both indices exactly 64 B, `quota` starts at offset `CAP + 128`, a
  fresh cache line, no co-residency.
- New const assert: `size_of::<EgressRing<EGRESS_RING_BYTES, 64>>() ==
  CAP + 2*64 + round64(size_of::<RefusalQuota>())` — flips if either
  index loses its padding.

`crates/cgap-git/tests/egress_layout_court.rs` (NEW): 3 tests —
padded-index-is-one-line (struct-size formula), quota-own-line
(struct align == 64), padded-ring-still-pushes-and-serves (behavioral
sanity: real ring, real push/window, final-state asserts, no mocks).
Names its anti-vacuity mutation in the header comment.

## Court evidence (this session)

| Court | Result |
|---|---|
| egress_layout_court (NEW) | 3 passed / 0 failed |
| egress_ring_court | 5/5 (fence: green) |
| quota_alloc_purity_court | 1/1 (fence: green) |
| andon_liveness_court | 5/5 (fence: green) |
| breach_regression_court | 5/5 (fence: green) |

Command: `CARGO_TARGET_DIR=/tmp/ce05-target cargo test -p cgap-git
--test egress_layout_court --test quota_alloc_purity_court
--test andon_liveness_court --test egress_ring_court
--test breach_regression_court` — all `test result: ok`.

## Anti-vacuity mutation (witnessed flip, /tmp sandbox)

Mutation: drop `#[repr(align(64))]` from `PaddedIndex` in a /tmp sandbox
copy of cgap-git (+ de-workspaced manifests, sibling deps cgap-blake3 /
cgap-parquet / cgap-bn3 copied, versions unified to 26.11.2). Build
aborts with three E0080 const-assert failures:

```
error[E0080]: assertion failed: size_of::<PaddedIndex>() == CACHE_LINE_BYTES
error[E0080]: assertion failed: align_of::<PaddedIndex>() == CACHE_LINE_BYTES
error[E0080]: assertion failed: size_of::<EgressRing<EGRESS_RING_BYTES, 64>>() == ...
```

Shared tree never mutated; post-court sha256 of
`crates/cgap-git/src/egress_ring.rs` = `4971406059ced3a0…`, identical
pre/post. Build-root leases deleted at lane end
(/tmp/ce05-target, /tmp/ce05-mut, /tmp/ce05-mut-target removed).

## Divergences

None vs dispatch. One sandbox-transport note: the mutation court required
copying three sibling zero-dep crates + de-workspacing manifests (the
sandbox crate alone cannot resolve its path deps); recorded here so the
next egress sandbox court skips the three failed attempts. Level-triggered
quota semantics untouched — FS-14's landed gate matrix is preserved
(quota_alloc_purity_court and breach_regression_court green).
