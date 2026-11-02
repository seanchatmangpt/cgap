# r10-gate3

Lane R10 (lane-runner), repo /Users/sac/cgap, branch main, 2026-11-01.
Task: GATE-3 completion — witness the 4-of-4 sovereign composition
(admit → eval_bn3 → commit → seal → verify) at 1M rows.

## Standing

ALIVE.

## Lease honored

- tests/court_verdict.rs (sole active writer this session; re-Read before
  first Edit)
- docs/jira/v26.11.1/waves/gate-3-ledger-proof.md (APPEND completion
  section; header status line reconciled to point at it — recorded
  divergence below)
- docs/jira/v26.11.1/waves/r10-gate3.md (this receipt)

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/r10-target)

- `cargo build -p cgap-ffi --lib` → exit 0 (fresh cdylib, 0.86s)
- `nm -gU /tmp/r10-target/debug/libcgap_ffi.dylib` → exit 0:
  `_cgap_admit_frame` T @0x1e1c, `_cgap_eval_bn3` T @0x2630,
  `_cgap_commit_parquet` T @0x21b0, `_cgap_verify_p28` T @0x32e8
  (plus `_cgap_p28_invariant_check` T riding the link, undeclared in
  include/cgap.h per GAP-1's typed UNSUPPORTED record)
- `cargo test -p cgap-court --test court_verdict a19_seal_1m_eval_court
  -- --nocapture` (CARGO_PROFILE_DEV_OPT_LEVEL=2) → exit 0, 1 passed,
  2.47s; witness line "SEAL+EVAL 1M LEDGER (4-of-4): admitted=1000000
  (zero-alloc) eval=1000000 lanes (differential pin vs fold, 6 mutation
  legs) committed + twin-deterministic; verify_seal ok; body/footer flips
  refused; lanes decoded from disk; keyed receipt sealed+verified;
  verify_p28 green + 8 spoliation refusals + digest pin + 79/81 B refused"
- `cargo test -p cgap-court --test court_verdict` (same env) → exit 0,
  **14 passed / 0 failed**, 2.46s (existing `a19_seal_1m_ledger_court`
  un-weakened, green in the same run)
- Anti-vacuity flip: `cargo test --manifest-path /tmp/r10-flipcrate/Cargo.toml
  --test gate3_flip a19_seal_1m_eval_court` → exit 101 (fail-closed) at the
  closure-count leg; shared-tree tests/court_verdict.rs sha256
  8184d1e961e1c59c8986e746f1c79adb3095136b2d104ce89963e924c9cb055c
  byte-identical before/after the flip run

## Court: a19_seal_1m_eval_court (new, in mod a19_seal)

1M corpus rows through all four sovereign stages:

1. ADMIT — every row's 152-B lane-triple frame (24-B payload: s = truncated
   instrument id, p = EVAL_LANE_PRED 7001, o = lane fold, A11 sigil words,
   LE) through the frozen `CgapAdmitFrameFn` (the sovereign body: A14
   penstock + A04 validate + BLAKE3 digest re-derivation); zero-alloc sweep
   asserted (`AllocReport::is_zero`), zero-copy view pinned (payload pointer
   inside the frame buffer). 1M/1M admitted.
2. EVAL — the frozen `CgapEvalBn3Fn` over each ADMITTED zero-copy view with
   the one-rule network `( ?0, 7001, ?1 ) => ( 7, 8, ?1 )`; lane = projected
   head object word; per-row differential pin lane == harness fold. 1M/1M
   lanes; histogram floor (>= 128/256 buckets).
3. COMMIT — eval output through the frozen `CgapCommitParquetFn` (the same
   8-row-group body the existing court composes) → twin commit
   byte-identical → `verify_seal` from disk (footer rows == 1M, B+M+80+16
   region arithmetic) → body-flip → ChecksumMismatch{Body}; footer-flip →
   ChecksumMismatch{Footer}; wrong handle → DigestMismatch; lane identity
   from DISK (first + last group) vs the EVAL lanes.
4. SEAL/VERIFY — keyed receipt (cgap-crypto, real Lease, one tick) + tamper
   flip; frozen `CgapVerifyP28Fn`: green + NULL-slot probe + from-disk slot
   + null guard + 8 intrinsic spoliation refusals + digest-predicate pin +
   79/81 B refusals.

In-leg mutations, executed every run: M1 predicate flip → count 0; M2 o-word
flip → projected lane tracks the admitted bytes; M3 reserved sigil →
CGAP_ERR_OABI_MISMATCH; M4 cap 0 → CGAP_ERR_LEASE_EXHAUSTED; M5 digest flip
→ admit-refused (OABI); M7 lane word > i32::MAX → CGAP_ERR_SHACL_VIOLATION;
plus the disk body/footer flips and the sealed-receipt tamper.

## Courts run (name -> evidence)

- a19_seal_1m_eval_court (new): pass exit 0 above; flip = the neutered-eval
  sandbox run exit 101 (below)
- a19_seal_1m_ledger_court (existing): still green in the full-suite run —
  the composition extension did not weaken it
- full court_verdict suite: 14/14, exit 0

## Anti-vacuity (the dispatch's leg 5)

Sandbox crate /tmp/r10-flipcrate (mirrors cgap-court dev-deps, [[test]]
gate3_flip): the court copy's eval hop NEUTERED — the sovereign eval call in
the closure loop replaced by a stub returning CGAP_OK with count 0 (skip
closure). Result: the court FAILS at its closure-count leg
("the lane rule matches exactly the one triple": left 0, right 1), exit 101.
The mutation lived only in /tmp; the shared tree's sha256 is byte-identical
before/after (nothing to restore in-tree; restore proof is the hash
equality). No skip was needed — the flip is a real executed FAIL.

## Divergences (recorded, not silent)

1. Dispatch CONTEXT stale: "CgapEvalBn3Fn declared at sovereign.rs:174,
   unimplemented" — on arrival GAP-1 had already landed all three bodies
   (cgap_eval_bn3 at sovereign.rs:520, cgap_admit_frame :411,
   cgap_commit_parquet :673). The poll (30s interval, 30-min budget) hit
   gap-1-sovereign.md standing ALIVE at iteration 13 (~6.5 min).
2. Commit stage of the composition goes through the EXISTING court's
   8-row-group body behind the frozen alias (reused, per "extend the a19_seal
   chain"), not the new sovereign `cgap_commit_parquet` body: that body
   carries a documented fixed 64 KiB single-slab lease
   (StaticPageBuffer::<65_536>, sovereign.rs:705) and refuses a 1M-row batch
   typed (LEASE_EXHAUSTED — witnessed by GAP-1's own oversized-lease probe).
   The 1M-row disk tier therefore remains the frozen-alias body that already
   held the disk-tier witness. No lane doc named a different shape.
3. Header status line of gate-3-ledger-proof.md reconciled from "PARTIAL" to
   ALIVE with a pointer to the appended completion section — an edit beyond
   strict append, taken because a stale PARTIAL header above an ALIVE
   section is a standing-drift hazard; the prior record body is unchanged.
4. Frame payload of the eval hop is the pinned sovereign eval encoding
   (EVAL_ENCODING_PIN: columnar s|p|o, 24 B/row) over the corpus lane fold —
   the a19 lane docs' row encoding (54 B) is not the eval surface's payload
   format; the differential pin (eval == harness fold, 1M/1M) is the
   cross-format binding. SlotConst truncation to 56 bits (A06 law) applied
   to the instrument-id word.
5. Suite runtime context: 2.46s under CARGO_PROFILE_DEV_OPT_LEVEL=2 in a
   warm target dir vs the ledger's earlier 8.64s single-court figure —
   different profile/build, not a claim change; counts are the receipt.
6. cgap-bn3's eval path allocates on host (documented in-code XIP
   divergence, sovereign.rs:344-353); the zero-alloc assert therefore lives
   on the ADMIT stage only (which held: is_zero).

## Cleanup (lease law)

- /tmp/r10-target deleted via python3 shutil.rmtree (below, after receipt).
- /tmp/r10-flipcrate deleted (same step).
- /tmp/r10-poll.sh, /tmp/r10-flip-run.log, /tmp/r10-suite.log removed.

## Falsifier for the next wave

Any change that breaks the chain must flip a19_seal_1m_eval_court: neutering
the eval hop flips at the closure-count leg (witnessed exit 101); skipping
admit flips at the admit-count leg; corrupting a committed body byte flips
verify_seal; regressing the lane fold flips the differential pin.
