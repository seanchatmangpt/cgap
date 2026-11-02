# CGAP-PRD-4 — FR-06: Instantaneous State Checkpointing
# Bounded-Static Snapshot Design (v26.11.2 audit lane)

Lane: PLAN. Status: DRAFT (design only; no code in this lane).
Date: 2026-11-02.

## 0. Honest scoping headline (read first)

FR-06 as written ("checkpoint the active universe") is NOT
implementable in v1 as a total-universe snapshot. Reason from the
measured receipt:

- POSH-4 (`crates/cgap-git/tests/posh_alloc_sentry_court.rs`,
  standing ALIVE per
  `docs/jira/v26.11.2/lanes/lane-posh-04-sentry.md` W3): the bn3
  `unify_to_fixpoint` derived-set computation **allocates by
  construction** — measured **2,662 allocs / 650,294,994 bytes**
  (3 batches, 2,929,939 derived rows, fixpoint converged).
- The e5 honesty pin
  (`crates/cgap-bn3/tests/edge_closure_courts.rs:422-477`) pins
  the fixpoint warmup allocation as a standing finding: buffers
  reach high-water in a bounded warmup, quiet by the 10th call.
- Consequence: a v1 snapshot claiming to include the derived-set
  must either (a) redesign the fixpoint onto a bounded arena
  (a bn3-side redesign), or (b) serialize derived rows into a
  static region that at the measured scale does not fit any
  honest bounded-static buffer (~650 MB measured).

**v1 scope decision: the fixpoint derived-set is OUT.** Named
loudly here with the measured allocs, deferred to a bounded-arena
bn3 lane. Everything else in the active universe is fixed-size
by construction and IS in scope.

Carried consequence, stated honestly: a restore yields a **cold**
derived-set — the checkpoint captures inputs, not the cache. The
first post-restore fixpoint run pays the full warmup (e5
discipline: quiet by the 10th call,
edge_closure_courts.rs:462-476).

## 1. What constitutes the "active universe" today

Enumerated from real fields of live modules; all zero-heap,
fixed-capacity by construction.

### 1.1 Ingress ring head/tail — IN SCOPE

`crates/cgap-git/src/ingress_ring.rs:146-150`:

```rust
pub struct IngressRing<const CAP: usize> {
    buf: [u8; CAP],
    head: PaddedIndex,   // PaddedIndex(AtomicUsize), :120
    tail: PaddedIndex,
}
```

- Real fields: `buf`, `head`, `tail`. Live bytes =
  `head.wrapping_sub(tail)` in `[0, CAP]` (module doc :20-30);
  full exactly when distance == CAP, so all CAP bytes usable.
- `StdIngressRing = IngressRing<INGRESS_RING_BYTES>`, CAP =
  32_768 (ingress_ring.rs:30, :280).
- Snapshot image per ring: head (8 B) + tail (8 B) + full
  32_768-B byte image = **32,784 B**.
- Why a flat image is complete: bytes are copied, never
  aliased, across the wrap boundary (:20-30), so position in
  the ring is fully determined by (buf, head, tail).
- Single-accessor law: the dma_ring law (module doc) makes the
  ring SPSC; the snapshot is well-defined only at a quiescent
  point (producer and consumer parked). Stated, not hand-waved.
- Needed change: `buf`/`head`/`tail` are private; add one
  read accessor on IngressRing, e.g.
  `pub fn snapshot_parts(&self) -> (&[u8; CAP], usize, usize)`
  (one added method, cgap-git-owned, non-frozen crate).

### 1.2 Receipt chain nonce/parent state — IN SCOPE

`crates/cgap-git/src/commit_synth.rs`:

- `LeaseReceipt { nonce: u64, parent: [u8; 32],
  payload: [u8; 32], tag: [u8; 8] }` (:114-123);
  `RECEIPT_LEN = 80` = nonce u64 LE + parent 32 B + payload
  32 B + tag 8 B (layout comment :28-31).
- `expected_tag` = BLAKE3 over
  `DOMAIN_LEASE_RECEIPT || nonce_le || parent || payload`
  (:125-135) — a pure function of the other three fields.
- Chain framing per cscb_c7 (receipt sealing; `crates/csct`
  bin `cscb_c7_receipt_sealing`; mutation court:
  `crates/cgap-blake3/tests/chain_mutation_court.rs`): each
  receipt's `parent` binds it to its predecessor.
- Checkpoint stores **next-nonce + last parent + payload**
  (8 + 32 + 32 = **72 B**, aligned). The tag is NEVER stored —
  recomputed via `expected_tag` on restore, because it is a
  pure function of the stored fields; a stored tag could be
  corrupted independently of its inputs, which the digest
  would catch, but recomputation is strictly stronger and free.
- Restore continues the chain with
  `LeaseReceipt::seal(nonce, parent, payload)` (:138).

### 1.3 Ref/lease tables — IN SCOPE

`crates/cgap-git/src/ref_store.rs`:

- `RefStore { entries: [Option<RefEntry>; REF_TABLE_CAP],
  len }` (:208-212); `REF_TABLE_CAP = 16` (:39);
  `RefEntry { name: [u8; 64], name_len: usize, oid: [u8; 20],
  lease: u64 }` (:169-178).
- Fixed per-slot raw bytes: 64 + 8 + 20 + 8 = 100; decision:
  **128-B stride**, 16 slots = 2,048 B entries image, plus
  len (8 B). The FNV fingerprint (`fingerprint()`, :234) is
  recomputed on restore, never stored — same law as the
  receipt tag: derived values are recomputed, only inputs are
  snapshotted.
- Journal: `Journal { records: [Option<JournalRecord>;
  JOURNAL_CAP], len, last_refusal }` (:200-206);
  `JOURNAL_CAP = 32` (:41). `JournalRecord` per record: seq
  u64 + name[64] + name_len + new_oid[20] + lease u64 +
  committed flag = 104 raw -> 128-B stride -> 32 x 128 =
  4,096 B + len 8 B.
- Swarm leases (`crates/cgap-git/src/a2a_swarm.rs`,
  "fixed capacity, no atomics" :104-110): `LeaseSlot {
  agent: AgentId(u64), zone: ZoneMask([u64; 4]) = 32 B,
  head: [u8; 20], active: bool, revoked: bool }` (:57-63);
  `COORD_WORDS = 4` (:20), `REF_LEN = 20` (:22). Per-slot
  raw 8+32+20+2 = 62 -> **64-B stride**. Table cap: the fixed
  array constant at a2a_swarm.rs:106 — pin the exact constant
  at implementation time (source comment says fixed cap).

### 1.4 Fixpoint derived-set — OUT OF SCOPE (v1)

- `unify_to_fixpoint` allocates by construction (W6-BFX owned
  delta tails; e5 pin, edge_closure_courts.rs:422-477).
- POSH-4 measured: **2,662 allocs / 650,294,994 bytes**.
- v1 decision: OUT. Either fix (a) bounded-arena bn3 redesign
  (own lane) or fix (b) static-region serialization bounded
  by ~650 MB measured — both fail the bounded-static mandate.
- Out-of-scope is stated loudly, not buried: **the fixpoint
  derived-set is not checkpointed in v1**; restore gives
  inputs only; first post-restore fixpoint pays warmup.

## 2. Module shape

### 2.1 Location and mount law

- cgap-core's lib.rs is lane-frozen — the exact situation
  already handled in `crates/cgap/tests/alloc_boundary.rs:50-53`
  (quoted): "cgap-core's lib.rs is lane-frozen (off limits), so
  the interceptor module is compiled into this binary directly
  from its owned source file. If a later lane lands `pub mod
  tracking_alloc;` in cgap-core's lib.rs, this include becomes
  a plain `use cgap_core::tracking_alloc;`."
- Convention adopted: `#[path = "../../cgap-git/src/checkpoint.rs"]
  mod checkpoint;` in each consumer until a mount lane lands
  `pub mod checkpoint;` in the owning crate's lib.rs, at which
  point the `#[path]` include flips to `use cgap_git::checkpoint;`.
- Location decision: **`crates/cgap-git/src/checkpoint.rs`**,
  NOT cgap-core. cgap-core is the zero-dependency kernel; the
  snapshot inputs (`IngressRing`, `RefStore`, `Journal`,
  `LeaseTable`, `LeaseReceipt`) live in cgap-git, and cgap-git
  depends on cgap-core, not the reverse. Hosting in cgap-core
  would require cgap-core to see cgap-git types — forbidden by
  the dependency direction. cgap-git's lib.rs is not
  lane-frozen (verify at implementation time with one grep).

### 2.2 API sketch

```rust
// crates/cgap-git/src/checkpoint.rs
pub const SNAP_MAGIC: [u8; 8] = *b"CGAPSNAP1";
pub const SNAP_VERSION: u64 = 1;

/// One typed refusal per validation failure class.
pub enum CheckpointError {
    MagicMismatch,
    VersionMismatch { expected: u64, actual: u64 },
    DigestMismatch { expected: [u8; 32],
                     actual: [u8; 32] },
    LayoutMismatch { field: &'static str,
                     expected: usize, actual: usize },
}
```

- `CheckpointError` is the typed refusal surface, matching the
  refusal-tag convention (ref_store.rs:48-75, `Refusal::tag()`).
- No `String` in the error; `&'static str` field tags only.
- Digest via injected digest: cgap-git already path-depends on
  cgap-blake3 (cgap-git/Cargo.toml:22), so the default digest
  is `cgap_blake3::hash`; for cgap-core-side purity tests the
  fn is generic over `F: Fn(&[u8]) -> [u8; 32]` so the digest
  is injected at the call site, mirroring the injected-digest
  law (frame.rs:316-318: "The digest function is injected,
  not linked").
- Dev-dep-in-test alternative: if the generic digest causes
  monomorphization friction, fall back to dev-dep cgap-blake3
  in tests only. Decision: generic-injected first, dev-dep
  fallback documented but not implemented.

### 3. Fixed-buffer serialization (no heap)

Single flat layout, size computed at compile time.
Authoritative offsets:

```
off      size     region
0        8        SNAP_MAGIC
8        8        SNAP_VERSION (u64 LE)
16       8        ring head counter (u64 LE)
24       8        ring tail counter (u64 LE)
32       32_768   ring byte image (buf[0..CAP])
32_800   8        receipt nonce (u64 LE)
32_808   32       receipt parent
32_840   32       receipt payload
32_872   8        ref table len (u64 LE)
32_880   2_048    16 x 128-B RefEntry stride
34_928   8        journal len (u64 LE)
34_936   4_096    32 x 128-B JournalRecord stride
39_032   8        lease table len (u64 LE)
39_040   64*N     N x 64-B LeaseSlot stride
39_040+64*N  32   BLAKE3 digest (trailing)
```

Total = 39_104 + 64*N bytes, N = LEASE_TABLE_CAP (pin the
a2a_swarm fixed-cap constant at implementation time;
N = 64 -> total 43_136 B). Every offset is a `pub const`
in checkpoint.rs, derived from the source constants
(INGRESS_RING_BYTES, REF_TABLE_CAP, JOURNAL_CAP,
LEASE_TABLE_CAP) so the layout re-derives if a constant
changes and the court fails loudly on drift.

Size law: the whole snapshot fits a single
`static SNAPSHOT: [u8; SNAP_LEN]` region — no heap anywhere:
no Vec, no Box, no format!-with-alloc, no String.

## 4. BLAKE3 digest

- Digest input: the snapshot region up to but excluding the
  trailing 32-B digest field.
- Digest provider: injected, default `cgap_blake3::hash`
  (cgap-git already path-depends on cgap-blake3,
  Cargo.toml:22); generic
  `D: Fn(&[u8]) -> [u8; 32]` parameter so cgap-core-side or
  no-crypto tests can inject a stub — mirroring the
  injected-digest law, frame.rs:316-318.
- Dev-dep-in-test: acceptable alternative if generics cause
  monomorphization friction; documented, not implemented.

## 5. Restore validation

Order matters; each failure is a typed variant:

1. Magic == SNAP_MAGIC, else `MagicMismatch`.
2. Version == SNAP_VERSION, else `VersionMismatch`.
3. Length == SNAP_LEN, else `LayoutMismatch`.
4. Recompute BLAKE3 over region[0..SNAP_LEN-32]; compare
   against the trailing 32 B; mismatch =
   `DigestMismatch { expected, actual }`.
5. Refingerprint: recompute the RefStore FNV fingerprint
   (:234) over restored entries and compare against a
   live RefStore in the court; recompute `expected_tag`
   over the restored receipt fields. Recomputed-derived
   law: derived values are never stored in the snapshot;
   on restore they are recomputed and must match the live
   object's derived values.
6. Only after 1-5 pass does any mutation of live state
   begin — restore is transactional in shape: validate the
   whole region first, then copy in. No torn restore.

## 6. The court (mutation-flip, positive control FIRST)

Court file: `crates/cgap-git/tests/checkpoint_court.rs`,
pattern authority POSH-4 / soc_full_chip_court.rs:57-104
(CountingAlloc-style window, positive control FIRST).

Driver-fn law: single `#[test]` fn, courts in one driver,
because allocator/window state is process-global (same
sequencing law as e5, edge_closure_courts.rs:427-431).

1. **Positive control FIRST** (anti-vacuity): corrupt one
   snapshot byte in a throwaway region, run restore against
   it, assert `DigestMismatch` fires — i.e. prove the
   validator CAN refuse before trusting the clean run.
   A window that cannot flip is VACUOUS. This is the same
   anti-vacuity twin as POSH-4 W3 court 1 and e5 window 0.
   Ordering is load-bearing: flip-first, then clean-run.
2. Clean run: build a live universe (ring with known
   content, receipt chain sealed via
   `LeaseReceipt::seal`, populated RefStore with
   fingerprint(), populated Journal, populated LeaseTable),
   snapshot, digest, restore into a fresh universe,
   assert byte-identity of all regions + equality of
   recomputed derived values (fingerprint, expected_tag).
3. Mutation flip: flip exactly ONE byte anywhere in the
   snapshot region, restore must refuse with
   DigestMismatch. (Not "detects corruption somewhere" —
   the refusal must be the typed DigestMismatch variant.)
   Also assert: flipping a byte in the digest field itself
   also refuses (digest-of-payload vs stored digest).
   Byte index law: flip a deterministic index from the
   frozen splitmix64 stream (seed law
   0xBF58_476D_1CE4_E5B9, corpus seed
   0xA20C_0FFE_E5A7_E000 per repo CLAUDE.md).
4. Layout-drift court: change a source constant in the
   court's local re-derivation and assert the offsets
   const math diverges loudly (the offset consts are
   derived; a constant change must fail the court, not
   silently re-layout).

## 7. Honest latency claim

- "Sub-millisecond checkpoint/restore" is a **TARGET**, not
  a claim. It is not measured in this lane. It stays a
  target until measured with the batch-amortized method
  (per-snapshot cost amortized over a
  batch of snapshots of the same populated universe,
  reported with the alloc ledger from an AE7-style
  tracking window, no `Process.sleep`, no
  `std::thread::sleep` in any court or bench — the e5
  warmup law: reach steady state by bounded iteration on
  a real observable, never sleep).
- Claim format when measured (later lane):
  "checkpoint N snapshots of a populated StdIngressRing
  universe, batch-amortized cost X us/snapshot,
  allocs = 0 (AE7 window), bytes = 0".
- Until that receipt exists, the doc claims only:
  fixed-size copy, zero heap, digest over ~43 KB region
  (BLAKE3 over 43 KB is fast; still unmeasured here).

## 8. Cross-reference: AX-9 suspend/resume probe

AX-9 (suspend/resume, live cluster + ax CLI) is
**UNVERIFIED-by-design** per ax-00 gate
(`docs/jira/v26.11.2/lanes/lane-ax-00-gate.md:47`): both
`ax` (127) and cluster (exit 1) absent; "No simulated pass
permitted." (also pinned in dflss/10-ax-control-plan.md:39).
This design is the **in-process analog**: capture/restore of
in-process state, entirely independent of AX-9. It makes NO
claim about, and provides NO evidence toward, AX-9 — AX-9
stays UNVERIFIED-by-design until a cluster exists, and no
checkpoint court may be cited as an AX-9 pass. Boundary
stated to prevent receipt laundering across lanes.

## 9. Out-of-scope items (loud)

1. **Fixpoint derived-set, v1** — OUT. Measured 2,662
   allocs / 650,294,994 bytes (POSH-4 W3); bounded-arena
   bn3 redesign is its own lane. Restore yields cold
   derived-set; warmup paid post-restore (e5 law, quiet
   by 10th call).
2. Durable WAL / disk-backed checkpoint — OUT. The
   `Journal` is "in-memory crash-consistency journal
   (court simulation, honestly labeled; not a durable
   WAL)" (ref_store.rs:198-200), so the checkpoint is
   in-process, not crash-durability.
3. Multi-ring / multi-universe snapshot — OUT (v1
   checkpoints one StdIngressRing universe; extension is
   a layout version bump, SNAP_VERSION).
4. Concurrent snapshot of a live SPSC ring — OUT.
   Quiescent-point law only (section 1.1).
5. AX-9 evidence — OUT (section 8).

## 10. Implementation steps (sequenced)

1. Add `snapshot_parts()` accessor on IngressRing (one
   method, cgap-git-owned).
2. Create `crates/cgap-git/src/checkpoint.rs`; mount
   via `#[path]` from tests until
   a mount lane lands `pub mod checkpoint;` in cgap-git
   lib.rs.
3. Offset consts derived from source constants; SNAP_LEN
   as `const`.
4. snapshot()/restore() as fixed-buffer copies, zero
   heap, injected digest generic.
5. checkpoint_court.rs: positive control FIRST, clean
   run, one-byte flip -> typed DigestMismatch, layout
   drift court. Single driver fn.
6. Latency TARGET stays unclaimed until the
   batch-amortized receipt lands.

### Critical Files for Implementation

- /Users/sac/cgap/crates/cgap-git/src/ingress_ring.rs
- /Users/sac/cgap/crates/cgap-git/src/ref_store.rs
- /Users/sac/cgap/crates/cgap-git/src/commit_synth.rs
- /Users/sac/cgap/crates/cgap-git/src/a2a_swarm.rs
- /Users/sac/cgap/crates/cgap/tests/alloc_boundary.rs
