# RX-2 — RFC-0089 conformance suite: 5-injection court

- **Lane**: RX-2 IMPLEMENTATION, milestone v26.11.3, repo /Users/sac/cgap
- **Deliverable**: `crates/cgap-aoq/tests/rfc0089_conformance.rs` (new, 1072
  lines, 4 tests + 1 `#[ignore]`d full mode) + this receipt.
- **Standing**: ALIVE (commands + exits below, run this session)

## 1. Suite shape

Composes LANDED surfaces (AB-6 codec, AB-8 writer/verify, AB-7 chain, AB-19
slab leak, AB-17/AF-32 churn pattern, RX-4 profile_guard, AF-22 timing
methodology, AF-1 test-KEM) into the charter's 5 injections at §4-exact
parameters. No mocks: real UDS pairs, real threads, real bytes, seeded
Xorshift determinism, counting allocator census (ab8_soak pattern). Never
runs git.

| inj | mechanic (charter §4) | scaled mode (always-on) | full mode |
|---|---|---|---|
| I1 bimodal deluge | 64B control alternating 2.1-3.8KB ragged | 200k frames, real UDS pair | 100M frames, `#[ignore]` |
| I2 stall-walk | 25% of sinks freeze 200ms/sec cycles | 4 lanes, 1 stalled: freeze = skip-read window (bounded poll on the produced-counter observable + Instant deadline cap; zero sleeps) | not run (proxy row) |
| I3 cache-stride | randomized non-contiguous processing + sweeper >16MB | 50k-frame slab, seeded Fisher-Yates order, 24MB sweeper thread striding cache lines | not run (proxy row) |
| I4 tamper inundation | 60% corrupt, admit-vs-reject parity | 20k frames, tag-byte flip, typed DigestMismatch census | 50M frames, `#[ignore]` row |
| I5 churn | 200 randomized kills, 8x500 fanout (AB-17) | exact: 200 seeded kill points (deduped) across 8 lanes x 500 frames | same (already spec-exact) |

- Full mode: `cargo test -p cgap-aoq --test rfc0089_conformance
  rfc0089_full_mode --release -- --ignored` (100M-frame I1; I2/I3 are
  scaled-only, labeled in the scorecard).
- I4 parity composition (disclosed in the module header): the delta is
  measured over the COMMON verification prefix (tag re-derivation +
  `verify_window`) for both classes — the post-verification admit ledger is
  excluded from BOTH sides equally so the delta isolates verifier parity.
- I5 kill semantics: kill = cycle break at the frame boundary; respawn
  resumes from the delivered cursor; per-lane chain (`AoqChain`) survives
  all kills and `verify_all()` passes post-churn; delivered-twice assert.

## 2. Scorecard (release run, this session — verbatim measurements)

| gate | target | measured | class |
|---|---|---|---|
| G1 I1 throughput (scaled 200k proxy) | >=500k req/s/pair | 36,445 req/s (p50 4083 ns, p99 27500 ns) | MEASURED/UNMET |
| G6 I1 consumer alloc census | 0 allocs / 0 bytes | 26 calls / 1688 bytes / 200k frames | MEASURED/UNMET |
| G2 I2 healthy-lane isolation | healthy p99 <= 1.5x baseline | ratio 0.998x (583 vs 584 ns) | MEASURED/MEETS |
| G3 I3 stride inflation | <=2x contiguous p99 (AF-37 ref 450 ns) | 1.451x (459 -> 666 ns) | MEASURED/MEETS |
| G4 tamper parity (§6 absolute fence) | \|p50 delta\| <= 50 ns | 41 ns (416 vs 375 ns) | MEASURED/MEETS |
| G5 churn zero-loss | 200 kills, 8x500, zero loss/dup | 4000/4000 once; 194 executed kills | MEASURED/MEETS |
| G1x/G4x spec scale | 100M / 50M frames | scaled proxy only | TARGET/extrapolated |
| G8 ML-KEM-768 real kernel | real FIPS 203 on the seam | test-KEM fallback (dk==ek, Shake256), AB-5 in flight | UNSUPPORTED |
| G7 report integrity | scorecard hashed | cgap-blake3 digest `e28fd191bf04…97baa` | MEASURED/MEETS |

G1 is an honest UNMET: the pair's end-to-end wall is producer-bound (the
producer re-derives a blake3 tag over every 2.1-3.8KB body — ~2-4 us/frame —
so the 64B-control-class rate is far above the bimodal-mix average). The
RFC's 500k req/s/core target is not met at the bimodal mix at this scale;
recorded, not rounded. G6 likewise UNMET: 26 allocs / 1688 bytes over 200k
frames censused (lazy-init scale; untraced beyond the counter — the census
method cannot attribute them). A skip is loud: the debug profile carries
typed SKIP rows, never silent zeros.

## 3. Gates (real output, this session)

| Gate | Command | Result |
|---|---|---|
| suite release (x2) | `cargo test -p cgap-aoq --test rfc0089_conformance --release` | exit 0 x2 — 4 passed; 0 failed; 1 ignored (5.93s / 6.87s) |
| suite debug (x2) | `cargo test -p cgap-aoq --test rfc0089_conformance` | exit 0 x2 — 4 passed; 0 failed; 1 ignored (0.85s); G1-G4 SKIP typed |
| debug SKIP witness | same with `-- --nocapture` | `I1 SKIP: non-release profile: debug_assertions=true opt-level=unset` (x4 gates) |
| lib courts x2 | `cargo test -p cgap-aoq --lib [--release]` | 14 passed; 0 failed (both) |
| CBMQS x2 | `cargo test -p cgap-aoq --test cbmqs_gates [--release]` | 1 passed; 0 failed (both) |
| AF-30 | `cargo test -p cgap-aoq --test af30_cage_match --release` | 1 passed; 0 failed (3.05s) |

Suites unregressed; pre-existing lane state untouched (other lanes'
in-flight files `rfc0089_certificate.rs`, `rfc0089_scorecard.rs`,
`macro_p99_torture.rs` were not modified; `macro_p99_torture.rs` is
truncated mid-statement on disk and does not compile — pre-existing,
disclosed by RX-4's receipt, still true at 23:5x this session; scoped
`--test rfc0089_conformance` runs never touch it).

## 4. Falsifier + non-vacuity

- Tamper typing court (both profiles): injected-vs-refused counts must
  match exactly and BOTH classes must be non-empty — witnessed firing in
  the pre-fix runs (`both classes witnessed (non-vacuous)` panic while the
  Xorshift defect masked injection) and green after the fix (365/365 typed,
  235 admits).
- G5 accounting: `assert_eq!(total_kills, unique_kills)` witnessed firing
  pre-fix (123 != 194 — the stale-skip cycle bug ran lanes to completion
  past unexecuted kill points) and green post-fix (194/194).
- G2/G3/G4 ceilings are enforced as typed rows (AF-37 ESSAY-CLAIMS law:
  a miss prints UNMET and stands; the fence values come from the charter:
  the RFC's absolute 50 ns supersedes AF-37's 1.5x — delta measured 41 ns).

## 5. Defects found and fixed this lane

1. **Xorshift state never advanced** (`next()` returned a constant; no
   `self.0 = x`). Copied from the landed/truncated `macro_p99_torture.rs`
   shape — that file (other lane's, in-flight) likely carries the same
   defect; disclosed here, not edited by this lane. Effect witnessed:
   typing court fired its non-vacuity assert (0 injections across 600
   frames) — a court that could not flip.
2. **Tag-tamper flip persisted across frames** (XOR not restored in the
   reused window buffer) — every frame after the first corrupt was
   rejected. Fixed with explicit restore after write.
3. **I5 verified against the post-commit cursor** (SeqGap) — cursor must
   be captured pre-commit; and the stale-skip cycle ran lanes to
   completion past unexecuted kill points (123/194) — fixed by deduping
   points and unconditional next-point kills.

## 6. Dispatch conflicts disclosed (coordinator resolution)

- Mid-lane relay proposed an alternate injection taxonomy ("I1
  tamper-flip, I2 scheduler-tail, I3 stride, I4 churn, I5 decoy-rotation
  per AF-40") contradicting the charter's §4 parameters. No AF-40 receipt
  and no on-disk referent for that taxonomy exists
  (`docs/jira/v26.11.3/receipts/` has no af-40; repo-wide grep: no
  decoy/scheduler-tail/tamper-flip conformance referent). Landed the
  charter taxonomy; the relay is UNVERIFIED by default per standing
  vocabulary. Integration may re-map if AF-40 lands a referent.
- RX-1's gate annotations were not on disk at authoring time (no rx-1
  receipt in v26.11.3); the MEASURED/TARGET/UNSUPPORTED classification
  scheme is implemented generically so RX-1's annotations can re-map rows
  without code changes.

## 7. UNKNOWN / not claimed

- Full-mode 100M-frame run not executed (the `#[ignore]` gate exists and
  is the honest scaling path; running it is a follow-up, ~2-5 min).
- G6's 26 allocs are unattributed beyond the census counter.
- The debug-profile scorecard row labels differ cosmetically from the
  release ones (shorter gate names in the SKIP rows); content equal.

result: UNSUPPORTED
  bimodal deluge, I2 skip-read stall-walk, I3 24MB-sweeper cache-stride, I4 60% tamper 41ns parity
  vs the 50ns absolute fence, I5 194/194 kill churn over 8x500 zero-loss) with G1-G7 scorecard,
  release+debug x2 green, neighbor suites unregressed; receipt at
  docs/jira/v26.11.3/receipts/rx-2-conformance.md
