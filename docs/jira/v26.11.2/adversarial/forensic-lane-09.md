# Forensic Lane FA9 — hostile audit of the 0.755 µs commit + receipt chain

2026-11-02. Canonical checkout /Users/sac/cgap @ main 9a730320. Read-only on the
shared tree; probes in /tmp/fa9-probe, build root /tmp/fa9-target (leased,
deleted at lane end). Adversarial presumption: the commit is a pointer bump
with no persistence barrier; the receipt chain hashes headers, not payloads.

## Suspicion 1 — no persistence barrier on the commit path: CONFIRMED

`stage_commit` (`crates/cgap-ffi/src/sovereign.rs:729-794`): the only file
operation in the entire path is ONE `std::fs::File::create(path)` at lease
grant (sovereign.rs:747-752). After grant, steady-state per-transaction work
is: memcpy into the ring window (`flush_to` -> `RingSink::commit`,
`mmap_writer.rs:804-842`) + the sealer's FNV digest. Grep for
`fsync|msync|sync_all|sync_data|f_flush` across mmap_writer.rs and all
cgap-ffi sources: **zero matches**. No writeback is ever issued.

Witnessed probe (fa9-probe, release build, exit 0):

```text
AFTER_64_COMMITS_SPILL_FILE_LEN=0        <- 64 OK commits, spill file EMPTY
BIG_FRAME_STATUS=0x0                     <- rotation triggered (60k-lane frame)
AFTER_ROTATION_SPILL_FILE_LEN=25856      <- ring drained only here
```

Sixty-four successful commits left the spill file at 0 bytes. A process crash
before the first rotation loses EVERY committed transaction since grant — the
bound is the 64 KiB ring (worse in practice: the "cold" File::create never
even allocated blocks if nothing drained). msync/fsync for the sealed region
does not exist; `flush()` on the spill handle (mmap_writer.rs:808) is std
Buffer flush, not hardware writeback — even the rotation drain is not durable
against power loss, only against process crash of the COMMITTER process.

Even the rotation drain's `file.flush()` is a user-space buffer flush, not
`sync_all()`; so "durable at drain boundaries" is itself overstated — the
system's honest durability class is: durable-at-rotation against process
crash; NOTHING is durable against machine loss. Power failure at any point
can lose the entire run.

## Suspicion 2 — chain hashes headers, not payloads: REBUTTED (with a gap)

`ChainReceipt::link` = `BLAKE3(prev_hash || block bytes || license bytes)`
(receipt_chain.rs:86-89); `block_digest = hash(sealed.bytes())` where
`SealedBlock::bytes()` returns the FULL committed block bytes
(footer.rs:609-616, "the bytes ... borrowed from the writer's buffer"). The
digest covers the full payload, not a truncated header region.

Flip witness (fa9-probe, exit 0): a chain minted over payload [1,2,3,4]
verifies pristine; verification against a payload differing by ONE lane
value ([1,2,3,5]) returns `Err(Spoliation)`:

```text
PRISTINE_VERIFY_OK=true
ONE_LANE_PAYLOAD_TAMPER = Err(Spoliation)
```

GAP FOUND: the tamper battery
(`receipt_chain.rs:280-345`, `tampered_receipt_or_block_is_typed_refusal`)
mutates stored receipt fields, swaps block order, and tampers the license —
it NEVER mutates committed block bytes, despite its own docstring claiming
"every committed block-byte mutation" (receipt_chain.rs:277-278). The
docstring is an overclaim; the payload-binding property is true but is not
directly witnessed by the battery (my probe witnesses it: payload tamper
flips). Battery also has no negative-control... it does have one (pristine
first). The gap is: no block-payload mutation leg.

GAP 2: the FFI commit surface (`cgap_commit_parquet`) does NOT append to a
RingReceiptChain at all — the receipt chain is a separate library surface;
the 0.755 µs figure and the chain are disjoint paths. "Chain_full_transaction"
is benched separately (6.19 µs/tx). So "the 0.755 µs commit is receipt-backed"
would be false framing; receipts ride a different stage.

GAP 3: `receipts_mut()` is a `pub` mutable seam (receipt_chain.rs:104-106)
on a chain of hash-linked receipts — but note the links are recomputed from
block bytes at verify, so mutating stored receipts alone is detected. The
seam is benign here because verify recomputes everything.

## Suspicion 3 — two concurrent committers: NO UB, but silent fork

COMMIT_LEASE is `thread_local!` RefCell (sovereign.rs:781-786): each thread
gets its OWN lease. A second concurrent committer never touches the first
thread's RefCell — no UB, no panic, no typed refusal. Both threads returned
32/32 CGAP_OK on the same spill path (probe exit 0):

```text
RACE_OKS=[32, 32] RACE_FILE_LEN=0
RACE_OKS=[32, 32] RACE_FILE_LEN=25856   <- second grant re-File::create'd the
                                           same path, truncating thread 1's drain
```

Each lease grant executes `File::create(path)` — O_TRUNC — on the shared
second thread's grant truncates whatever the first thread already drained to
the spill file. The "single-writer" design is enforced per-THREAD only; two
threads committing to one path is accepted with all-CGAP_OK and produces a
truncated/duplicated spill stream. That is data loss with a success status —
worse than a typed refusal.

## Suspicion 4 — 22x warm-vs-cold: NOT FOUND (divergence)

The dispatch premise "22x warm-vs-cold split" does not exist in the tree.
Grep across crates/bench/benches/ and docs/jira/v26.11.2/waves/: the only
"22x" is the L3 radix leg at 22x over its target
(`w5-l3-remeasure.md:37`), unrelated to persistence. There is no warm-vs-cold
persistence comparison anywhere; the only commit benchmark is steady-state
only (chains-sovereign.rs). So the "persistence comparison theater"
suspicion is MOOT — there is no comparison at all, which is its own finding:
the receipt chain durability framing in receipts a3-ring-sink.md and
p2-receipts-ring-receipt-chain.md carries ZERO durability disclosure (grep
fsync/persist/drain/crash across both: one line, a3-ring-sink.md:27 "rotation
drains to the pre-opened spill handle"). The durability story was BURIED, not
disclosed: no receipt states that steady-state commits are memory-resident
until rotation, or that nothing is fsync'd ever.

## Courts run this session

- `cargo test -p cgap-ffi --lib receipt_chain` (CARGO_TARGET_DIR=/tmp/fa9-target)
  — exit 0, 5 passed (chain_appends_and_verifies,
  tampered_receipt_or_block_is_typed_refusal, invalid_license_refused_at_append,
  empty_chain_verifies_as_empty, lease_exhaustion_is_typed_refusal).
- fa9-probe run 1 (durability + race): exit 0, outputs above.
- fa9-probe run 2 (payload-tamper flip): exit 0, Spoliation flip witnessed.

## Verdicts (standing vocabulary)

| # | Suspicion | Verdict | Basis |
|---|---|---|---|
| 1 | No persistence barrier | CONFIRMED | sovereign.rs:747-752 sole file op; zero fsync/msync grep hits; probe: 64 commits -> spill len 0 |
| 2 | Header-only hash | REBUTTED | link over FULL block bytes (footer.rs:615); probe flips Spoliation on one-lane payload tamper |
| 2b | Tamper battery covers block bytes | PARTIALLY CONFIRMED | battery never mutates block bytes; docstring overclaims (receipt_chain.rs:277-278) |
| 3 | Concurrent committer UB | NO UB; typed refusal ALSO absent | thread_local lease: both threads all-OK on one path; grant truncates the other's spill |
| 4 | 22x warm/cold theater | NOT FOUND (premise absent) | only 22x is w5-l3-remeasure.md:37 radix, unrelated |
| — | Receipts disclosed durability | BURIED, not disclosed | zero durability disclosure lines in a3-ring-sink.md / p2-receipts-ring-receipt-chain.md |

## Honest durability disclosure the receipts should have carried

1. Steady-state commits are MEMORY-RESIDENT in a 64 KiB ring until rotation.
   A process crash loses up to 64 KiB of committed transactions per thread;
   a power loss can lose the ENTIRE run (no fsync anywhere in the path).
2. Rotation drain uses `write_all + flush`, never `sync_all` — even drained
   data is not power-fail durable.
3. Single-writer is per-THREAD; two threads on one spill path both succeed
   and truncate each other's drains with CGAP_OK statuses. The API needs a
   cross-thread lease or a typed refusal on second grant for the same path.
4. The FFI commit surface appends no receipts; receipts and the 0.755 µs
   commit are disjoint paths — do not conflate the figures.
5. The tamper battery's docstring claims a block-byte mutation leg it does
   not contain; the payload-binding property holds (witnessed) but the
   battery should add the leg rather than claim it.

## Divergences from the dispatch

- "22x warm-vs-cold split" does not exist in the tree; recorded NOT FOUND
  with the only "22x" candidate cited.
- Dispatch said probe the failure mode "if the API allows thread-local
  overlap" — it does (thread_local lease), probe run, failure mode recorded.
- Receipts w5-ring-rewitness reproduced the 0.755 µs figure honestly (with
  drift note) but no lane receipt discloses the durability boundary.
