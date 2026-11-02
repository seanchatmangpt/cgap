# w6 — FA9 commit gating + honest relabel (errata receipt)

Lane FA9FIX, 2026-11-02. Repo /Users/sac/cgap @ main 9a730320. Fixes the three
confirmed FA9 findings in `docs/jira/v26.11.2/adversarial/forensic-lane-09.md`.
Build root /tmp/fa9fix-target (lease, deleted at lane end). No git writes; the
shared tree was never mutated (sha256 baseline vs post-lane: OK, see flip leg).

## Diff summary

| File | Change |
|---|---|
| `crates/cgap-ffi/src/sovereign.rs` | +131/-2: new register value `CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x4347_0006` (beside the 0x4347_* family, sovereign.rs:87); `spill_gate` module (per-(path-hash, thread-id) ownership registry, FNV-1a 64 path key, minted thread ids, thread-local Drop release at lease-thread exit); fail-closed acquisition at the top of the `cgap_commit_parquet` body BEFORE the lease-grant `File::create`; FA9 durability disclosure block on the commit path; status-court vals/pins/class-list extended. |
| `crates/cgap-parquet/src/mmap_writer.rs` | +4: durability-boundary disclosure comment at `drain_to_spill`. |
| `crates/cgap-ffi/tests/fa9_commit_gating.rs` | new 3-leg court (226 lines). |
| receipt | this file. |

## Status-code registration cite

`crates/cgap-ffi/src/sovereign.rs:87` —
`pub const CGAP_ERR_CONCURRENT_WRITER_DENIED: CgapStatus = 0x4347_0006;`
registered beside the existing `'CG'`-class family (0x4347_0001..0004 at
sovereign.rs:73-80). Registered extension, NOT an A19 §2 pin: the §6.2 status
court's header-parity pin (`got.len() == 6`) is unchanged — include/cgap.h was
NOT in this lane's ownership, so the C enum is not extended this wave
(follow-up for the header owner). The value IS pinned Rust-side (exact pin +
'CG'-class + pairwise-distinct hygiene, in-lib status court) and in the new
court's register-pin leg. The reachability ledger was deliberately NOT
extended: by design it counts `return CGAP_ERR_<NAME>;` producers OUTSIDE
sovereign.rs, and this code's only producer is inside sovereign.rs; the new
court is the producer witness instead.

## Durability disclosure (landing sites)

- Commit path: `crates/cgap-ffi/src/sovereign.rs` — in-memory transaction
  staging-latch disclosure block directly above the `COMMIT_LEASE.with` body:
  no msync/fsync/sync_all exists; data reaches spill ONLY at rotation drain
  (write_all + user-space flush()); crash before first rotation loses the
  64 KiB ring; power loss can lose the run; CGAP_OK means STAGED, never
  DURABLE.
- Drain site: `crates/cgap-parquet/src/mmap_writer.rs` `drain_to_spill` —
  write_all + flush() is user-space buffer flush, not sync_all; drained data
  is crash-of-committer durable, never power-fail durable.

## Honest relabel

stage_commit µs/tx figures (0.755 µs class) measure the in-memory STAGING
LATCH, not a durable commit. Cited basis: forensic-lane-09.md suspicion 1
(64 OK commits -> spill len 0; zero fsync/msync/sync_all grep hits; rotation
drain is flush()-only). MB9-vs-FA9 "22x" footnote: the dispatch premise does
not exist in the tree (forensic-lane-09.md suspicion 4, NOT FOUND); the only
"22x" is the L3 radix leg (w5-l3-remeasure.md:37), unrelated — MB9's cold arm
is FD-create cost, not a durability comparison; do not read the delta as a
durability number.

## Court (crates/cgap-ffi/tests/fa9_commit_gating.rs, 3 legs)

1. fa9_gate_register_pin: 0x4347_0006 exact pin, 'CG' class, sequence
   nonzero, != OK.
2. fa9_race_grid_one_writer_no_truncation: real owner thread grants + rotates
   at path P (ready signal = drained bytes on disk), then commits
   back-to-back; a real racer thread tight-loops REAL commit attempts at the
   SAME path, bounded by a 20k-attempt budget (no sleeps — bounded spin on
   the ready observable only). Final-state asserts: owner never refused; no
   third status; racer observed DENIED >= 1 (gate fires under contention);
   spill bytes append-only — post-grid read has the pre-race drained snapshot
   as an exact byte prefix (no truncation by any second-grant IO).
3. fa9_release_after_owner_thread_exit: while the owner is alive, a different
   thread at the same path is DENIED exactly; after the owner thread EXITS,
   the path is claimable again (the gate is not a stuck deny; also the
   anti-vacuity of the release mechanism itself).

### Run receipts (this session)

- Court baseline: `CARGO_TARGET_DIR=/tmp/fa9fix-target cargo test -p
  cgap-ffi --test fa9_commit_gating` — exit 0, 3 passed / 0 failed, 0.06s.
- Court MUTANT (gate denial disabled, marked in-sandbox), /tmp/fa9fix-sbx
  (deps absolutized to repo paths; CARGO_TARGET_DIR=/tmp/fa9fix-sbx-target):
  exit 101, 2 FAILED (race grid: racer never denied, prefix leg flips on
  truncation; release leg: same-path attempt while owner alive succeeds).
  The court cannot pass the mutant — not vacuous. Sha256 baseline vs
  post-lane on sovereign.rs + mmap_writer.rs: OK — the shared tree was
  never mutated by the flip battery; sandbox restored sha256-exact (cp from
  repo source) and its build-root lease deleted.
- Full regression: `CARGO_TARGET_DIR=/tmp/fa9fix-target cargo test -p
  cgap-ffi` — exit 0: 50 lib + 4 zero_copy + 3 fa9 + 2 alloc + 5
  lever_courts = 64 passed, 0 failed. No pre-existing failures to disclose.
- Workspace build: `cargo build --workspace` — exit 0.
- gate0 note: bare `python3 scripts/boundary_gate.py` requires a mode flag
  and exits 1 (fail-closed usage refusal, pre-existing design); not run as
  a court (no mode named by the dispatch).

## Divergences (recorded)

1. GATE DESIGN (main divergence): the dispatch prescribed a single
   AtomicBool claim/release per call. A per-call global flag cannot deliver
   the court's required invariant ("no truncation of the first thread's
   data"): with release-per-call, the racer can win the inter-call gap,
   GRANT (O_TRUNC) at the owned path, and legally truncate the holder's
   drained bytes — the exact FA9 finding. The gate is therefore a
   per-(path-hash, thread-id) ownership registry (FNV-1a 64-bit path key)
   with lease-lifetime (thread-exit) release: denial BEFORE any IO for any
   second thread on an owned path; owner re-entry allowed; same-path claim
   after owner exit allowed. Residual bound: 64-bit FNV-1a path-hash
   collision probability, documented not silenced. (A per-call GLOBAL flag
   would also deny concurrent commits to DIFFERENT paths — existing tests
   run in parallel with distinct paths and would flake.)
2. "Exactly one succeeds per contention window" is asserted as: the racer
   observes only {OK, DENIED}; every DENIED coincides with the owner
   holding the path (owner all-OK); the owner's drained bytes are
   byte-prefix preserved. The dispatch's literal wording holds in the
   release leg (one OK, one 0x4347_0006) and in every grid DENIED.
3. include/cgap.h not extended: header not in lane ownership. The new
   status is FFI-side only; C consumers cannot match it by name until the
   header owner lands the enum entry. Recorded parity gap, not a silent
   skip.
4. One Edit call corrupted two lines of sovereign.rs (status-court pins
   region) mid-session; fixed same-session, final state verified by the
   passing status court + full suite exit 0.
5. cgap-parquet was not needed as a dev-dep: the race grid asserts
   prefix-preservation against a real on-disk snapshot rather than a
   reconstructed byte stream (which would have required a manifest edit
   outside lane ownership).

## Standing

ALIVE for the three fixes: typed concurrent-writer refusal (court + mutant
flip exit 101 this session), durability disclosure (comment blocks landed at
commit path + drain site), honest relabel (above). UNVERIFIED: C-header
parity for the new status code (header not owned); post-fix perf figures
(none measured — perf claims remain TARGETs).
