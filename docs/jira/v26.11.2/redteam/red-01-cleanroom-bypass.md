# RED-01 — Clean-Room Deconstruction (SWAR sieve + 24B fact frame)

Lane RED-01 · /Users/sac/cgap · branch main · 2026-11-02 · READ-ONLY lane
(writes: this file + /tmp/red01 build root only)

## 1. Claim referent (found — not UNSUPPORTED)

AHDCLT is defined at `docs/jira/v26.10.31/16-ahdclt.md`. The exact
"epistemic unconstructability" wording lives in two places:

- `docs/jira/v26.10.31/16-ahdclt.md:20` — "Unauthorized forks/bypasses:
  technically unconstructable and legally toxic" (Pillar 1 table row).
- `docs/jira/v26.10.31/16-ahdclt.md:43-44` — "The mapping must be
  technically unconstructable without the original substrate: the
  predicates are load-bearing in the type/shape system, not decorative
  comments."

Critical framing the doc itself supplies: line 3-4 marks the whole
doctrine "Status: UNVERIFIED doctrine... not a measured result", and
lines 100-116 pre-register the falsifier: "If frontier agents defeat the
tripwires with cheap retrieval or tool use ... the anti-LLM pillar is
decoration" (#1) and "AHDCLT is a doctrine/thesis, not a measured
result; every claim below fails closed to UNKNOWN until falsified
adversarially." So this lane's mission is exactly the adversarial test
the doc says is pending.

Subject surfaces named by dispatch:

- SWAR sieve: `crates/cgap-branchless/src/swar.rs` (221 lines; kernels
  `swar_lt` :33, `swar_eq_u64` :44, `swar_has_zero_byte` :53,
  `swar_range_mask` :60) and `crates/cgap-branchless/src/penstock.rs`
  (`penstock_prefilter8_mix` :35 with mix constant
  `0x2544_5491_4F6C_DD1D` at :30; `swar_all_zero_vl` :64).
- "24-byte coordinate frame": the on-disk referent is the CSCB fact
  frame, 24 B = 3x u64 LE, FRAME_BYTES=24
  (`docs/jira/v26.11.2/00-ard-prd.md:163-165`, CGAP-PRD-05). Note: the
  phrase "coordinate frame" does not appear anywhere in the repo; it is
  a dispatch-local name for this frame (divergence #1). The A04
  CgapFrameHeader is a different, 128 B object — not what the dispatch
  pointed at, recorded to prevent conflation.
- Perf claim under test: ingress chain 10.20 ns/frame fastest / 10.30
  median; penstock admission stage 7.71 ns/frame; dispatch's 5.70 ns
  penstock row explicitly NOT reproduced on this host
  (`docs/jira/v26.11.2/11-measured-claims.md:26-29`, row 7;
  `waves/w3-l1-ingress.md:63-72`).

## 2. Method

Hostile cloner, restricted to in-repo public specs only: swar.rs and
penstock.rs doc comments (which state the exact algebra and every
constant), the PRD frame note, and the measured-claims numbers. No
reverse-engineering of binaries, no access to anything not on disk.
Constructed a C99 translation of the full kernel set + a 24 B-frame
ingress-shaped chain, differential-tested against byte-wise oracles,
then measured. Artifacts: /tmp/red01/cleanroom.c (124 lines, sha256
8644903414f0f4402dafeaa0d0c590b68ddcdb579799472e986a5a24af806e44),
built with Apple clang 17.0.0, `-std=c99 -O2 -march=native`.

## 3. Clean-room C99 source (constructed from doc comments alone)

The four kernels below are line-for-line the algebra stated in the
swar.rs/penstock.rs DOC COMMENTS (the source bodies were not needed —
the comments are complete constructions). Included so the court is
replayable; the on-disk artifact at /tmp/red01/cleanroom.c carries the
full benchmark harness too.

```c
#define HI 0x8080808080808080ULL
#define NOT_HI 0x7F7F7F7F7F7F7F7FULL

/* swar.rs:33 swar_lt — from its own doc comment (lines 27-31) */
static inline uint64_t swar_lt(uint64_t x, uint64_t y) {
    uint64_t low_ge  = ((x | HI) - (y & ~HI)) & HI;
    uint64_t high_gt = x & ~y & HI;
    uint64_t high_eq = ~(x ^ y) & HI;
    return ~(high_gt | (high_eq & low_ge)) & HI;
}
/* swar.rs:44 swar_eq_u64 — borrow-free nonzero-per-byte detect per
 * doc comment lines 16-19 */
static inline uint64_t swar_eq(uint64_t w, uint64_t t) {
    uint64_t x = w ^ t;
    uint64_t nz = ((x & ~HI) + ~HI) | x;
    return ((nz & HI) ^ HI) >> 7;
}
/* swar.rs:53 swar_has_zero_byte */
static inline int swar_has_zero(uint64_t v) {
    return ((((v & ~HI) + ~HI) | v) & HI) != HI;
}
/* penstock.rs:35 — mix constant read straight off penstock.rs:30 */
#define MIX 0x254454914F6CDD1DULL
static inline uint64_t prefilter8_mix(uint64_t w) {
    return (w ^ (w >> 7)) * MIX;
}
/* penstock.rs:64 swar_all_zero_vl, 24B-frame specialization */
static inline int frame_all_zero(const uint8_t *p) {
    uint64_t v0, v1, v2;
    memcpy(&v0, p, 8); memcpy(&v1, p+8, 8); memcpy(&v2, p+16, 8);
    if ((((v0 & NOT_HI) + NOT_HI) | v0) & HI) return 0;
    if ((((v1 & NOT_HI) + NOT_HI) | v1) & HI) return 0;
    return ((((v2 & NOT_HI) + NOT_HI) | v2) & HI) == 0;
}
```

Zero-dependency, C99, compiles clean on clang 17 with -O2. No
substrate-private structure was required at any point: every constant,
every mask, every borrow-free trick is printed in the repo's own prose.

## 4. Commands + exits (this session)

- `clang -std=c99 -O2 -march=native -o cleanroom cleanroom.c` (cwd
  /tmp/red01) → exit 0, no warnings surfaced.
- `./cleanroom` → exit 0.
  - Correctness: 200k differential pairs per-byte eq + lt vs byte
    oracles → 0 fails; EXHAUSTIVE 65,536-pair full-byte-domain swar_lt
    check across all 8 lanes (524,288 lane instances — mirrors the
    subject's own `swar_lt_exhaustive_full_byte_domain` court,
    swar.rs:173-186) → 0 fails; 50k all-zero-detect windows vs byte
    oracle → 0 fails.
  - Perf (bench-shaped proxy: prefilter8 fold + 24B all-zero pre-scan
    over 1,048,576 24B frames, 21 reps, 5 runs, CLOCK_MONOTONIC,
    best-of): 1.02 ns/frame best, sink stable nonzero
    (1598077633064197941) across all 5 runs — non-vacuous by sink
    assert (returns 1 if any differential fails or sink == 0; exit 0).
  - sha256 (final): cleanroom.c
    8644903414f0f4402dafeaa0d0c590b68ddcdb579799472e986a5a24af806e44;
    binary 60ee50f383b94f6813d8ba165616e94d6a3ae156fbedac4cd1f2f6f8e24e3f9b.
- Shared tree untouched (read-only lane; `git status` not run — no git
  commands per contract; writes limited to this file + /tmp).

## 5. Verdict

**BREACH-CONFIRMED at the algorithm-constructability layer, with scope
limits stated.**

- The Pillar-1 wording "technically unconstructable without the
  original substrate" (`16-ahdclt.md:43`) is defeated by construction:
  a hostile cloner reading only in-repo public prose rebuilt every
  SWAR kernel and the prefilter mix byte-algebra-exactly, differential-
  clean against byte oracles including an exhaustive 524,288-lane
  swar_lt sweep. Nothing in the type system, no load-bearing predicate
  annotation, was needed. The kernels are textbook bit tricks whose
  exact derivation is printed in their own doc comments (swar.rs:27-31,
  16-19) — the substrate's documentation is itself the complete
  construction recipe.
- The statutory-vice half of Pillar 1 is a different layer (the
  P28-SIL predicate mapping, LICENSE-P28-SIL.md /
  crates/cgap-ffi/src/sovereign.rs) and was NOT under test here; this
  breach does not by itself void that mapping. What it does void is the
  "technically unconstructable" qualifier as applied to the compute
  kernels: they are constructable from public specs at zero cost.
- Perf parity: the clean-room kernel chain ran at 1.02 ns/frame best
  on this host — faster than the subject's measured 7.71 ns/frame
  penstock stage and 10.20 ns/frame full chain — but this is NOT an
  apples-to-apples full-chain number: the clean-room proxy omits A04
  project-validate, dispatch, and FrameView materialization that the
  Rust chain includes (w3-l1-ingress.md:24-31). The grounded claim is
  kernel-level parity-or-better with a structurally lighter workload;
  full-chain C parity remains UNVERIFIED (never round up).
- The doc's own pre-registered falsifier #1
  (`16-ahdclt.md:104-106`) fires: cheap retrieval + construction
  defeated the "unconstructable" pillar. Per the doc's own register
  rule (`:100-101`), AHDCLT fails closed — the Pillar-1
  "technically unconstructable" claim is DECORATION for the SWAR
  surface as of this test.

## 6. Divergences

1. "24-byte coordinate frame" has no on-disk referent as phrased;
   mapped to the CSCB fact frame (24 B = 3x u64 LE, FRAME_BYTES=24,
   `00-ard-prd.md:163-165`). The 128 B A04 CgapFrameHeader is a
   different object — recorded to block future conflation.
2. The dispatch's "5.70 ns ingress" target is already recorded as
   NOT-reproduced by the owning campaign
   (`11-measured-claims.md:28`, row 7: 7.71 measured, dispatch row
   not reproduced on this host); this lane did not chase it, it
   benchmarked against the MEASURED numbers instead.
3. Clean-room perf is kernel-proxy, not full-chain parity (scope note
   in verdict above).

## 7. Standing

ALIVE — clang compile + run with exits captured this session
(exit 0 both), differential + exhaustive courts green, benchmark
sink-asserted non-vacuous. Claim verdict: BREACH-CONFIRMED (algorithm
layer), full-chain perf parity UNVERIFIED.

## See Also

- docs/jira/v26.10.31/16-ahdclt.md
- docs/jira/v26.11.2/11-measured-claims.md
- docs/jira/v26.11.2/waves/w3-l1-ingress.md
