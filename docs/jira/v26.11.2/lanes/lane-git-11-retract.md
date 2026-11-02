# Lane GIT-11 — revert(commit-delta) cascade retraction over the coordinate frame

Date: 2026-11-02 · Branch: main · Repo: /Users/sac/cgap

## Files written (complete list)

- `crates/cgap-git/src/retract.rs` (new, 462 lines, alloc-free,
  `lib.rs` untouched — module reachable only via the court's `#[path]`
  include until the coordinator wires `pub mod retract;`)
- `crates/cgap-git/tests/retract_court.rs` (new, 505 lines)
- `docs/jira/v26.11.2/lanes/lane-git-11-retract.md` (this receipt)

Nothing else. No git commands. `Cargo.toml`/`lib.rs` of cgap-git and
`crates/cgap-storage/src/dred.rs` untouched.

## Design (dispatch disclosure law)

cgap-git is zero-external-dep; a path dep on cgap-storage needs
Cargo.toml (not this lane's file), and cgap-storage's dred.rs is
std-prelude-shaped so it cannot compile inside no_std cgap-git proper.

- `retract.rs` is SELF-CONTAINED and alloc-free (caller-leased CSR,
  purged/keep masks, BFS worklist; typed `RetractError` refusals,
  fail-closed worklist-overflow guard inside `cascade_step`).
- The court (std integration-test crate) includes the REAL DRed engine
  verbatim: `#[path = "../../cgap-storage/src/dred.rs"] mod dred_real;`
  — disclosed include of exactly one foreign source, for dual-oracle
  cross-check. dred's own `#[cfg(test)]` courts ride along and PASS
  (3 tests), which also witnesses the real engine compiles under the
  include.
- Same least-fixpoint cascade law as dred::cascade_retract: purged(root)
  for every deleted root; purged(d) if purged(p), edge p->d,
  alt_support[d]==0. The SWAR keep pass is word-parallel u64 bitmask
  math — named what it is: logical bit-lanes on a SCALAR host, no SIMD
  claim.

## Commands + exits (this session)

```
cd crates/cgap-git && CARGO_TARGET_DIR=/tmp/git11-target \
  cargo test -p cgap-git --test retract_court
  # exit 0 — 8 passed; 0 failed (debug)
cd crates/cgap-git && CARGO_TARGET_DIR=/tmp/git11-target \
  cargo test -p cgap-git --release --test retract_court
  # exit 0 — 8 passed; 0 failed (release, measurement profile)
CARGO_TARGET_DIR=/tmp/git11-target cargo test -p cgap-git --release \
  --test retract_court court_throughput -- --nocapture
  # exit 0, THROUGHPUT lines captured
CARGO_TARGET_DIR=/tmp/git11-target cargo test -p cgap-git --release \
  --test retract_court court_mutant -- --nocapture
  # exit 0, "MUTANT FLIP: 920 orphan derivations injected (first coord 9)"
```

## Courts (all Chicago: real collaborators, final-state asserts, no mocks)

1. **court_cascade_matches_dual_oracle** — 5-fact chain KAT (full-chain
   purge without alt; {2,3,4} survivors with alt on fact 2), then
   corpus-level parity at facts 64/1,000/4,096: fast path == local
   scalar rounds oracle == REAL `dred::cascade_oracle`, byte-set
   equality. Anti-vacuity: the KAT survivor sets are hand-derived.
   Anti-phantom: post-retraction orphan count EXACTLY (0, u32::MAX).
2. **court_mutant_one_step_flips_typed** — MUTATION COURT: the driver
   suppresses ONE cascade step (`revert_cascade_mutant_one_step`,
   fixpoint loop reduced to a single BFS round) on the 10,000-fact
   chain corpus. Guarded driver: `Ok`, zero orphans. Mutant driver:
   typed `Err(RetractError::OrphanDerivation { 920, coord 9 })`
   (exit-1-class). Anti-vacuity flip twins witnessed this session.
   Raw keep-set orphan count > 0 asserted independently of the driver.
3. **court_throughput_measured_reported** — honest measurement, NO
   gate. Basis stated on stdout: facts / elapsed of one
   `revert_cascade` call (BFS fixpoint + SWAR keep + branchless
   compaction; CSR build excluded — matches B05's counting basis).
   N=15 release runs, 100,000-fact chain frame, canonical seed
   `CORPUS_SEED ^ 0xBEEF` (stepped splitmix64). Run-invariant survivor
   count asserted; survivor count < facts asserted (bench is real
   work, not an empty run).
4. **court_typed_refusals_and_delta_projection** — build_csr typed
   lease/edge refusals; idempotent delta->del_mask projection KAT;
   SWAR keep-mask KAT incl. partial-tail-word padding purge.

## Measured throughput (honest, units stated)

Counting basis: 100,000 facts / elapsed of one `revert_cascade` call,
release, N=15, min over runs (2026-11-02 witness):

- **min 323,843,635.3 facts/s (308.791 us)**; mean 321,033,470.9
  facts/s (311.494 us).

**Gate policy: reported, NOT gated.** Why: B05 measured the same
engine-shape at 248M facts/s honest min against an 800M aspiration and
chose the honest FAIL over tuning; dred.rs:73-76's 839-944M facts/s is
the keep-mask path ALONE on a different corpus/method and must not be
conflated (DREDLABEL law). Any threshold I could defend here would be
either vacuous below 248M or tuned above it; an honest measured number
with no gate beats a tuned gate, per dispatch.

## Anti-phantom integrity

Zero orphan derivations post-retraction on all court corpora: every
surviving alt-less derived fact has >= 1 alive derivation parent. The
orphan definition excludes alt-rescued survivors (legal under the
cascade law) — the first draft counted them and FAILED its own court;
the law was corrected, court re-run green.

## Standing

- **retract.rs module: ALIVE** — compiled + executed this session via
  the court binary, 8/8 green, debug AND release, exit 0 both.
- **Cascade correctness: ALIVE** — dual-oracle set-equality (incl. the
  real dred engine) at three corpus sizes + hand-derived KATs.
- **Mutation court: ALIVE** — flip witnessed (920 injected orphans,
  typed Err) this session, release + debug.
- **Throughput: MEASURED, UNGATED** — 323.8M facts/s min (basis and N
  stated above); no threshold claimed.
- **Module wire-in: UNVERIFIED/BLOCKED-BY-DISPATCH** — `lib.rs` is not
  this lane's file; `pub mod retract;` wire-in is left to the
  coordinator (until then the module is reachable only through the
  court's `#[path]` include; it is alloc-free and no_std-clean by
  construction but has not been compiled as part of the cgap-git lib
  target — that compile is UNVERIFIED).

## Fences (must-not-regress surface this diff could break)

- `cargo test -p cgap-git --test retract_court` — run, exit 0 (this
  lane's own target, debug + release).
- Shared compile: `lib.rs`, `Cargo.toml`, `dred.rs` untouched, so the
  lib target and every sibling crate are byte-identical to before the
  lane. Read-only `cargo test -p cgap-git` (full) compiles all test
  targets: **pre-existing** compile failures in OTHER lanes' targets
  were observed (`soc_full_chip_court` E0204/E0502,
  `pack_demux_court` E0425 RING_WINDOW_BYTES/FACT_WINDOW,
  `ingress_ring_court` E0425 `differ`/IngressRingOps,
  `commit_synth_court` E0308/E0599) — none touch this lane's files;
  census law: these are compile-aborts of foreign targets, not failure
  counts, and not session-introduced.
- Cargo.lock/Cargo.toml: no dep changes.

## Divergences (disclosed)

1. Dispatch said "if dred's API is reachable without a dep, use #[path]
   include". dred.rs is std-prelude-shaped (`Vec` bare, no alloc
   import), so it cannot compile inside no_std cgap-git's module tree;
   the include therefore lives in the std TEST crate, not in
   retract.rs. Same disclosed single-file reuse, different site than
   the module itself.
2. Dispatch scoped three files; a first-draft orphan definition counted
   alt-rescued survivors as orphans and failed its own court — the
   law (orphan = alive AND derived AND alt_support==0 AND no alive
   parent) was corrected within the lane, court re-run green.
3. Mid-lane the retract.rs file suffered a write corruption
   (247k-line duplicate-content blowup during an in-place rewrite); it
   was detected by wc -l during the build loop and rewritten from
   scratch in <50-line chunks; on-disk bytes verified (brace balance
   74/74, 10 pub fns, corruption-marker grep zero).

## Lane build-root lease cleanup

`CARGO_TARGET_DIR=/tmp/git11-target` is deleted at lane end per the
lease law (coordinator-visible in the final command log).


---

## ERRATA (2026-11-02, TC-08 / cull-08; LOW priority re-scope pointer)

Line 90's framing of the 800M figure is receipt-true as of its writing
(reported, NOT gated) but predates the MV-5 gate re-scope: the decided
C6 gate is now 150M facts/s full-cascade min-of-7; 800M is retained as
ASPIRATIONAL_FPS, reported loudly, never gating. Re-scope receipt:
lane-mv-05-sla-reconcile.md:44; register append:
audit/release-blockers.md:233. Original text retained unedited.
