# Lane GIT-03 receipt — ingress ring (retry lane)

Standing: ALIVE (all commands run this session, exits cited per command).

## Files written (owned only)

- `crates/cgap-git/src/ingress_ring.rs` — repaired module (lib.rs untouched).
- `crates/cgap-git/tests/ingress_ring_court.rs` — repaired courts.
- `docs/jira/v26.11.2/lanes/lane-git-03-ingress-ring.md` — lane doc (was absent).
- This receipt.

## Repair ledger (dead-lane diagnostics, verified against disk this session)

1. **Sieve flip twin was broken twice over** (diagnostic confirmed, plus
   worse): `differ` was never declared (compile abort), and the
   `!0x41/!0x7E` "complement band" trick produced a min>max empty band by
   accident, not design. Replaced with a REAL flip twin: the named
   mutation is "swap the min/max band arguments", applied to the SAME real
   `push_sieved` code path; frames are constructed so >=128 genuinely
   diverge (hit-payload byte 0x50 vs 0x20), and the twin additionally
   asserts the mutant ring stays empty and the correct-band ring admits.
   The prior `!hit != hit` tautology is gone — `differ >= 128` is
   falsifiable (a vacuous sieve yields 0).
2. **E0401 const assert in generic impl**: diagnostic STALE — the size/align
   const asserts on `PaddedIndex` were already at module scope (module
   lines 121-122 post-repair); compiles clean. No edit needed; recorded as
   a stale-diagnostic errata.
3. **Defects found beyond the two named diagnostics** (the dead lane's
   courts themselves were wrong):
   - `wrap_around_byte_exact_court`: arithmetically impossible geometry —
     after filling to CAP and popping 100, a 300-byte push overflows
     (only 100 free). Worse, the original layout could never straddle the
     boundary at all (head pos 0 after a full fill). Rewritten with pinned
     geometry: 32,468-byte first frame + pop 100, then a 400-byte frame
     copies 300 bytes to the physical edge and 100 wrapped — both
     segments asserted byte-exact.
   - `overflow_mutation_flip_twin`: `[0u8; cap]` used a runtime `let` as
     an array length (E0435) → `const CAP`.
   - `penstock_sieve_court`: compared frame count to byte count
     (`admitted` vs `ring.pending()`); now asserts
     `admitted * frame_len == pending` plus explicit `dropped > 0`.
   - Unused `LO` const (warning) removed; docstring corrected (the range
     predicate never used it).

## Commands + exits (this session)

| Command (CARGO_TARGET_DIR=/tmp/git03-target) | Exit | Result |
|---|---|---|
| `cargo test -p cgap-git --test ingress_ring_court` (debug) | 0 | 6 passed, 0 failed |
| `cargo test -p cgap-git --release --test ingress_ring_court` | 0 | 7 passed, 0 failed |
| `cargo test -p cgap-git --release --test ingress_ring_court -- --nocapture sieve_throughput_release_report` | 0 | `MEASURED [sieve-throughput] method=batch-amortized(...) per_frame_ns=4 target_ns_per_frame=10 status=MEETS-TARGET` |
| `cargo build -p cgap-git` (lib gate) | 0 | clean |

## Fences (must-not-regress surfaces this diff could break)

- `cargo test -p cgap-git` full crate run: my lane's compile is clean; the
  run aborts compiling `tests/soc_full_chip_court.rs` (unclosed delimiter,
  NOT a GIT-03-owned file — another lane's in-flight work). Per dispatch
  law compile-aborts on other lanes' in-flight files are not a failure
  count. Recorded as observed pre-existing, not introduced by this diff.
- Zero-string/boundary gates not run (no gate changes; additive module +
  tests only).

## Verification honesty

- The 10 ns/frame number is a TARGET; measured value 4 ns/frame via the
  disclosed batch-amortized method, release profile, run this session.
  Per-frame `Instant` literal is impossible on macOS (disclosed method).
- Perf numbers are REPORTED measurements with method disclosed, not gated.
- Court non-vacuity: overflow twin flips on guard deletion (assert_ne
  against computed mutant head); sieve twin flips on swapped-band mutant
  (`differ >= 128`, falsifiable). No skips; no `[SKIP]` needed — every
  court ran.

## Divergences from dispatch

- SWAR sieve is in-module documented re-implementation, not `#[path]`
  import of `cgap-branchless/src/scan.rs` (disclosed in module docstring
  and lane doc): `cgap-git` is a zero-external-dep published kernel;
  importing foreign source would fake an undeclared dependency edge.
- Both dead-lane diagnostics were checked against disk; diagnostic #2 was
  stale (assert already at module scope). Errata, not silent rewrite.
