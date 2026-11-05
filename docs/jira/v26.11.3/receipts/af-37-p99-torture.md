# AF-37 Receipt: the 5-injection p99 torture bench
Standing: ALIVE

Lane: AF-37 IMPLEMENTATION. Date: 2026-11-04.
Bench: `crates/cgap-aoq/tests/macro_p99_torture.rs` (new, ~880 lines,
5 courts). Receipt: this file. Never ran git.

## Subject

MEASURED-vs-TARGET torture matrix for the sovereign AOQ1 pipeline
(expected-tag re-derivation -> verify_window -> risk gate -> chain
append) under five named injections, composed on LANDED surfaces only:
AB-6 codec, AB-4/AB-8 verify+writer law, AB-7 chain (real cgap-arrow
blake3 digest seam), AB-19 slab pattern (Box::leak preallocation),
AB-10/11 stall/retry patterns. Real UDS socketpairs, real threads, no
mocks (Chicago).

## Essay-claims law (binding)

The four ceilings (<850 ns heap-storm, <12 us stall-walk, <450 ns
cache-stride, <1.5 us tamper-inundation) were TARGET-UNMEASURED going
in (tort-2/tort-5 components ab-10/ab-17/ab-18 receipted). This bench
MEASURED them; nothing rounded; misses print typed UNMET and stand.

## Gates (ran this session, exits witnessed)

| command | exit |
|---|---|
| `cargo test --release -p cgap-aoq --test macro_p99_torture -- --test-threads=1 --nocapture` (pass 1) | 0 |
| `cargo test --release -p cgap-aoq --test macro_p99_torture -- --test-threads=1 --nocapture` (pass 2) | 0 |
| `cargo test -p cgap-aoq --test macro_p99_torture` (debug, default parallel harness, x2) | 0 |
| `cargo test -p cgap-aoq` (22 test binaries ok) | 0 for all integration binaries |
| `cargo test -p cgap-aoq --doc` | 1 (PRE-EXISTING, see below) |

Pre-existing failure, not session-introduced: the
`src/profile_guard.rs` doctest (`assert_release`, line 70) panics
"SKIP: non-release profile" because doctests compile in the dev
profile by design (loud-SKIP pattern placed inside a doctest that runs
in debug). Unrelated to this lane's files; owner should either
`no_run`/`ignore` the example or move the SKIP to a non-panicking
signal. Left untouched (file ownership).

## The torture matrix (MEASURED, release, serial, aarch64 darwin)

Ranges across green passes of the final bench; per-run values printed
by the bench, never rounded.

| injection | measured p99 (range) | target | verdict |
|---|---|---|---|
| I1 bimodal-size storm (processing p99) | 26,250-281,667 ns across 6 passes | <850 ns | UNMET |
| I1 64B class only | p99 3,625-4,041 ns | (component) | INFO |
| I1 3.8KB class only | p99 28,375-44,917 ns | (component) | INFO |
| I2 healthy-lane p99 under stall | 1,250-3,875 ns | <12 us | MEETS |
| I3 decode p99 under cache pressure | 167-291 ns | <450 ns | MEETS |
| I4 admit p99 under 60% tamper | 583-584 ns | <1.5 us | MEETS |
| I5 relay e2e p99 vs ab-17/ab-18 band | 1.27-5.10 ms | band 5.8-7.4 ms | UNMET (below band) |

Fences (asserted, all passes): I4 parity p99(refuse)/p99(admit)
0.93-1.00 <= 1.5x; I4 refusal census 3000/3000/3000/3000 exact vs plan
(60% tamper, 4 classes); I2 isolation min-of-3-vs-min-of-3 ratio
0.62-0.98x (one earlier pass hit 1.80x, see noise-floor disclosure);
I1/I3/I4 alloc census 0 calls / 0 bytes (AB-19 SC-3 law holds on admit
AND refusal paths); I5 acked 10000/10000 exactly once, kills 9,
generations 10 on the receipt-grade passes.

## Court findings

- I1 UNMET is a REAL, stable miss, and the components say where: the
  64B class p99 is 3.6-4.0 us (timer-inclusive; the ceiling was 850 ns)
  and the 3.8KB class p99 is 28-45 us, dominated by blake3 over
  header+body twice per frame (expected-tag re-derivation + chain
  record digest; ab-12's receipted 1768 MiB/s puts a 3.9 KB digest at
  ~2.2 us floor each, so ~4.5 us of the 64B figure is digest work the
  850 ns ceiling cannot include even at zero overhead). The 850 ns
  heap-storm ceiling is UNREACHABLE for the composed op as specified
  (R-K1 tag + AB-7 chain digest over the body); it is reachable only
  for a digest-free decode+gate path, which I1's structure does not
  measure. The 0-alloc GC-immunity witness HOLDS: the consumer side
  never allocates under the bimodal storm.
- I3 MEETS with margin: decode p99 167-291 ns under a 16 MB
  cache-sweeper + randomized frame order — the 64B one-cache-line
  containment claim is CONFIRMED as measured (clean baseline p99
  84-84 ns; pressure adds ~2-3x, well inside 450 ns).
- I4: reject-equals-admit-speed CONFIRMED (parity 0.93-1.00 vs 1.5x
  fence) with typed 4-class refusal census exact vs the deterministic
  plan and 0 allocs on refusal paths. Oversize body_len word, header
  bit-flip, magic flip, and re-sealed seq regression all refuse via the
  correct typed class (DigestMismatch/BadMagic/SeqGap/HeaderTooShort).
- I2: isolation CONFIRMED (healthy p99 1.25-3.9 us under 24 real
  200 ms producer freezes; ratio 0.62-0.98x min-of-3 vs min-of-3).
- I5: below the receipted ab-17/ab-18 band (1.27-5.10 ms vs 5.8-7.4 ms)
  — faster than the band's lower edge; typed UNMET against the band as
  written, disclosed direction. Kill cycles are supervisor-observed
  (cadence approximate by design); conservation exact.

## Method disclosures

- Timing composition: I1-I4 time the PROCESSING op only (post-
  transport): expected-tag re-derivation (blake3(header[0..60] || body)
  [..16], R-K1) + verify_window (all framing/header/tag gates) + gate +
  chain append per admit; transport read syscalls excluded. I5 times
  true e2e (send -> verified-done) incl. queueing and kill gaps — the
  quantity ab-17/ab-18 receipted. Instant::now() pair overhead
  (~30-40 ns) rides in every sample (AF-22 disclosure pattern).
- Window law as landed: window = prefix(8) + span(120) + body +
  pad64(body) + EOS(8); probed against the real writer before the
  first measurement (wl=264 for a 64 B body) after an initial
  wrong-const desync was caught by the socket-read gate.
- Landed surfaces: AB-6 codec (fnv1a32 re-seal for the seq-regression
  tamper class), AB-7 chain with a real blake3 ChainDigest, AB-8
  writer, AB-19 leak-slab preallocation.
- I2 fence: min-of-3 paired rounds per the no-sleep-testing ratio-
  fence law, with a disclosed 2,000 ns timer/scheduler quantum
  allowance (disjunctive fence; a real HOL regression is 200 ms-scale,
  5 orders above the allowance). The allowance and the ratio both
  print on every pass.
- Courts run isolated for the alloc census (process-global counting
  allocator; each census court re-execs itself via a sibling-lane
  contributed `census_isolated()` guard + this lane's poison-tolerant
  SERIAL mutex; both reconciled in-flight, both lanes disclosed per
  the SLA law).
- I3's verify+gate context figure (~417-542 ns p99) shows the composed
  op WITHOUT the chain digest; it is the honest upper-bound context
  for what a digest-free decode+gate path measures.

## Anti-vacuity (per court)

- I1: census conservation (admitted==frames==chain links) + 0-alloc.
- I2: 24 stalls producer-counted (8 x 3 rounds); ratio printed.
- I3: 16 MB sweeper runs concurrently; baseline-vs-pressure delta
  printed; 0-alloc in timed ops.
- I4: 4-class typed census == plan exactly; parity ratio printed;
  0-alloc.
- I5: acks exactly-once (compare-and-set dedup), kills>=1 asserted,
  generation census asserted.

## Standing

ALIVE — all five courts pass x2 (release serial) and x2 (debug
parallel harness) this session against the exact subject on disk;
full `cargo test -p cgap-aoq` integration binaries 22/22 ok; one
pre-existing doctest failure in src/profile_guard.rs disclosed above.
Matrix verdicts: I2/I3/I4 MEETS, I1 UNMET (component-measured with the
digest-floor analysis), I5 UNMET-vs-band (below band, disclosed).
