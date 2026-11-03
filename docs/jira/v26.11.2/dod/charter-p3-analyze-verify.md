# Charter §4 Phase 3 — Analyze Verification (Lane CE-04)

Lane CE-04, 2026-11-02, repo /Users/sac/cgap @ main. Sole owned file:
this one. No git commands, no bench re-runs. Per-item RECEIPTED /
CODE-CITED status for the OPT-01..05 consolidation, every cite re-read
on disk this session (cite-verification law, same method as
dod/v26.11.3-forward-backlog.md).

## 1. OPT-03 — RECEIPTED

- Receipt on disk: `bench-validate/opt-03-channel-curve.md` (242
  lines, read in full this session). Landed receipt carrying the
  tail-knee-not-bus-knee attribution: §4/§5 show the fastest column
  nearly dose-invariant (10.67-12.18 ns/f, max +13.8% at bw6) while
  the tail loses up to 4.1x (cpu16 slowest 52.65 vs 12.82 baseline) —
  the onset is the scheduler-tail knee, not a bus-saturation knee.
- Additional landed finding retained: §3.2 leg 4 (bw6) recorded the
  max/min-ratio guard's blindness to COMMON-MODE (uniform) shifts —
  whole distribution shifted with 0 SUSPECT markers.
- Standing: RECEIPTED. No re-run performed (per contract); the
  receipt's own scoping (hot-ambient, mix-dependent magnitudes, row 28
  stays REGISTERED DIVERGENCE) carries forward unchanged.

## 2. OPT-01 — CODE-CITED

- Cite: `crates/cgap-git/src/egress_ring.rs:422-427`. Verified on disk
  this session: `pub struct EgressRing<const CAP, const STRIDE>` with
  bare `head: AtomicUsize` + `tail: AtomicUsize` + co-resident
  `quota: RefusalQuota` (lines 422-427 verbatim, false-sharing surface:
  the two counters and the quota line-share).
- The item is the ingress-ring PaddedIndex layout port (backlog §5
  OPT-01). No receipt; perf TARGET-class. Standing: CODE-CITED.
- Court design (unexecuted, from backlog): layout-equality court vs
  ingress PaddedIndex, seed 0xA20C_0FFE_E5A7_E000; mutation = break
  the pad constant, must flip.

## 3. OPT-02 — CODE-CITED

- Cite: `crates/cgap-gate/src/penstock.rs:548-556`. Verified on disk:
  the digest-all-zero early-exit scan (manual `while` loop breaking on
  the first nonzero digest byte, `digest_all_zero` flag) — an
  early-exit scan over a fixed header window, exactly the class the
  SWAR fold targets.
- Complement: `crates/cgap-branchless/src/penstock.rs` exists on disk
  with early-exit SWAR forms documented at :16-23 and :55-66
  ("shipped early-exits on the first nonzero BYTE; this early-exits on
  the first nonzero 8-byte chunk") — the branchless twin the
  equivalence courts compare against.
- Equivalence courts: exist (differential courts vs the shipped
  byte-exit form; the module header pins the corpus seed and forbids
  conflation). Standing: CODE-CITED. Perf TARGET; byte-identical
  replay pins hold without it.

## 4. OPT-04 — CODE-CITED

- Cite: `crates/cgap-ffi/src/sovereign.rs:931-990`. Verified on disk:
  `canonical_key` and `identity_key` in the spill-grant identity path.
  The syscall enumeration is the tax itself: per claim the path runs
  `Path::canonicalize()` (full path-walk; potentially two, if the
  first full canonicalize fails and the parent canonicalize fallback
  fires, :946-963), `fs::metadata()` (:987, in `identity_key`), plus
  the FNV-1a key derivation on the resolved bytes. The memoization
  proposal (cache canonical identity per lease grant so later
  same-path claims skip the re-walk) targets exactly this block.
- Standing: CODE-CITED. No court; unmeasured tax — TARGET-class,
  UNVERIFIED until a stated method measures the syscall count.

## 5. OPT-05 — RECEIPT-CITED (with a terminology finding)

- The dispatch phrased this as "1.07x demux floor; ring-relay 4x
  amplification, code cite pack_demux.rs". Findings:
- (a) The "1.07" figure is NOT a demux floor and does NOT live in
  `pack_demux.rs` (grep for 1.07/floor/relay in that file: no
  match). It is the measured COMMIT COST 1.05-1.07 us/tx on the FFI
  ring path: `02-pareto-levers.md:174` (L4 receipt, `waves/
  a3-ring-sink.md` witness) and `15-ctq-tree.md:48,57` (CTQ row
  "Commit cost | us/tx, FFI ring path | ceiling 4 | 1.05-1.07").
  RECEIPT-CITED, MEASURED.
- (b) The ring-relay ~4x memcpy amplification IS receipt-cited:
  `11-measured-claims.md:149` row 36 — SoC relay path copies window
  bytes ~4x (ingress relay -> demux staging -> object closure ->
  egress projection), with the code cites `pack_demux.rs:288`
  (`copy_from_slice` staging copy) and the pipeline court
  `tests/soc_full_chip_court.rs:12,286-306`. The `pack_demux.rs:288`
  code cite resolves (staging `copy_from_slice` present); TARGET
  rows P1/P2 (borrow/span refactor) unexecuted.
- Standing: RECEIPT-CITED (both numbers), with the correction that
  "1.07x demux floor" is a mislabel of the 1.05-1.07 us/tx commit
  cost. The backlog's §5 note that "OPT-05 not found on disk" was
  written against the mislabeled form; the underlying figures exist
  under rows 36 and the L4/CTQ receipts.

## 6. Lane receipt (CE-04)

- Files written: this file, sole owned file. No shared-tree mutation.
- Commands run (read-only, exit 0 unless noted): ls of dod/;
  sed reads of egress_ring.rs:415-435, gate/penstock.rs:540-565,
  sovereign.rs:925-995; wc of the two receipts; head of
  opt-03-channel-curve.md; greps over pack_demux.rs,
  branchless/penstock.rs, ring files, and docs/jira/v26.11.2/ for
  1.07 / relay / amplification. All cites re-read, not recalled.
- No courts run (documentation/verification lane). Anti-vacuity: the
  finding in §5 is the flip witness — the dispatched cite ("1.07x
  demux floor" @ pack_demux.rs) did not resolve, and the correction
  is recorded rather than smoothed over.
- Standing: ALIVE for the verification itself; per-item standing as
  stated in §1-§5. Fences: no code touched, so no regression surface;
  the only must-not-regress fences are the pinned figures cited in
  §5 (L4/CTQ/row-36), quoted verbatim above.
