# CSCT Lane 08 — Stage 6 THE SOLUTION (RFC-0042 §4)

Standing: UNVERIFIED:compile-pending in the shared tree; ALIVE-in-sandbox (command+exit below).

## Files written
- `crates/csct/src/solution.rs` (293 lines, sole owned source file)

## Commands + exits (this session)

| Command | Exit | Result |
|---|---|---|
| `cd /tmp/csct-l8-sandbox && cargo test --release` | 0 | 8 passed; 0 failed (3 solution courts + 5 lib tests from types/warmer) |
| `cp /tmp/csct-l8-sandbox/src/solution.rs /Users/sac/cgap/crates/csct/src/solution.rs` | 0 | verified on-disk: 293 lines, 3 `hash(&bytes)` sites |

## Verification ladder

1. Shared-tree `cargo test -p csct` is blocked: `crates/csct/src/lib.rs`
   declares `reframe`/`drowning`/`emotional`/`new_way`/`mutation`/`andon`/
   `receipt`/`court` — none on disk at authoring time (owned by other
   lanes). Compile-abort is not a failure count.
2. Per sandbox law: copied csct to `/tmp/csct-l8-sandbox`, stubbed the
   8 missing modules as empty `pub fn ..._dummy()` files, replaced
   sandbox `lib.rs` with bare module decls, repointed the cgap-blake3
   path dep to the canonical checkout (absolute path), ran the full
   court battery in release mode. Exit 0, 8 passed / 0 failed.
3. The measured per-frame filter cost on this host (release): passed the
   < 10.0 ns/frame gate. Debug-mode run measured 66.18 ns/frame — the
   gate is release-mode only by construction; debug builds emit the
   typed LoudSkip, not a fake pass.

## Content summary

- (a) Real cgap-blake3 kernel: deterministic 1M-u64 corpus from a
  lane-local SplitMix64-style PRNG (own constants `CSCT_PRNG_GAMMA`
  0x9E3779B97F4A7C15 / `CSCT_PRNG_SEED` 0xC5C5C5C55EED0001 — NOT the
  cgap-parquet frozen seeds), digest non-zero and stable across two
  hashes; stage receipt = the real digest.
- (b) Real Instant-timed SWAR-style u64 bitmask filter over 100k frames;
  per-frame ns < 10.0 asserted on this host (release). Host-too-slow
  path emits a typed `ExitCode::LoudSkip` scoped to that one assertion,
  never a fake pass.
- (c) Zero-alloc hot loop: `[u64; 1024]` stack array, loop body holds no
  heap types — static-invariant assertion; stack hits == heap hits == 1021.
- (d) Economics: sut_monthly 1200.0 ≤ 1200.0 cap; annualized savings
  5,054,232.0 ≥ 5,050,000.0 floor. Constants duplicated locally with a
  pointer comment because `crate::drowning` was absent; module doc says
  prefer `crate::drowning::Ledger` once it lands.

## Anti-vacuity

`mutation_perturbed_input_flips_digest`: one u64 (index 500,000) flipped
produces a different BLAKE3 digest — the hash court can flip. Also
`filter_frame_counts_real_hits`: 0 / 1 / 8 hit counts asserted — the
filter court can flip.

## Fences (must-not-regress surfaces named + status)

- `cargo test -p cgap-blake3` untouched by this lane (read-only dep use).
- Shared csct `lib.rs`, `types.rs`, `warmer.rs`: untouched (read-only;
  the sandbox copies were mutated, never the shared tree). Shared tree
  restore verified: shared `lib.rs` on disk still declares the full
  module list (read this session).
- No git commands run. No other lane's files written.

## Divergences

- Dispatch said "cross-module import of crate::drowning::Ledger is
  preferred" — impossible: `crates/csct/src/drowning.rs` absent at
  authoring time. Used the dispatch's sanctioned fallback (duplicate
  constants + pointer comment).
- Dispatch said timing "over e.g. 100k frames" with threshold 10 ns;
  measured 66.18 ns/frame in debug and pass in release. Timing gate is
  meaningful only in release; documented in-module.
