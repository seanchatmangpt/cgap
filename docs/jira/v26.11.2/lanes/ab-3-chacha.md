Canonical location: docs/jira/v26.11.3/receipts/ab-3-chacha.md

# AB-3 Lane Receipt — ChaCha20 (RFC 8439) — AoQ Wave-1 W1-3a

- **Lane**: AB-3, coordinator-dispatched (PG-A3 spec).
- **Date**: 2026-11-03
- **Subject**: `crates/cgap-crypto/src/chacha.rs` (new, 289 lines + tests), `src/lib.rs` (+1 line `pub mod chacha;`).
- **Standing**: ALIVE — every gate command below was run this session, exit 0.

## Delivered

- SIGMA consts (`expand 32-byte k` as u32 words).
- `chacha20_block(key: &[u8;32], counter: u32, nonce: &[u8;12]) -> [u8;64]` free fn —
  scalar, 10 double rounds, `#[inline(always)]` quarter-round, zero alloc, zero unsafe
  (crate is `#![forbid(unsafe_code)]`).
- `chacha20_xor_stream(key, nonce, counter, buf)` pull model: `chunks_exact_mut(64)` +
  stack tail, no alloc, counter `wrapping_add`.
- rv32imc perf note in the module header (no M-ext → the core uses only
  add/xor/shift/rotate, zero soft-mul `__mulsi3` calls; rotation-bound, est. 3-5x slower
  per byte than M-ext targets; inherent to ChaCha20 on RV32IMC, not this impl).

## Verification ladder (all run this session)

| Gate | Command | Result |
|---|---|---|
| RFC 8439 fixtures | `cargo test -p cgap-crypto --lib chacha` | 9 passed / 0 failed (run ×2 debug + ×1 release) |
| Throughput bench | same binary, `-- --nocapture` (release) | see table |
| Cross-target | `cargo check -p cgap-crypto --target riscv32imc-unknown-none-elf` | exit 0 |
| Cross-target | `cargo check -p cgap-crypto --target wasm32-unknown-unknown` | exit 0 |

Fixtures are RFC-verified: after hand-transcribed vectors failed, the real vectors were
pulled from `https://www.rfc-editor.org/rfc/rfc8439.txt` (fetched to /tmp/rfc8439.txt) and
the tests assert the exact serialized §2.3.2 block, §2.4.2 keystream block 1 (first 64 of
the 114-byte RFC keystream dump), and the exact 114-byte §2.4.2 Sunscreen ciphertext
(key 00..1f, nonce 000000000000004a00000000, counter 1). Note: **RFC 8439 has no §2.4.4**
— the Sunscreen encryption vector lives inside §2.4.2; the spec's "§2.4.4" reference is
corrected in the test doc-comments and here.

## Throughput baseline (release, min-of-8, `std::time::Instant`, aarch64 macOS)

| Buffer | Min ns | Throughput |
|---|---|---|
| 64 B | 166 | 367.7 MB/s |
| 1 KB | 2,416 | 404.2 MB/s |
| 64 KB | 109,375 | 571.4 MB/s |
| 1 MB | 1,622,667 | 616.3 MB/s |

Small-buffer penalty ~1.7x vs 1MB (fixed per-call overhead); large-buffer ~616 MB/s
scalar. Bench test `chacha::tests::bench_keystream_throughput_baseline` (kept in-tree so
future lanes re-run the identical court; eprintln output, `#[cfg(test)]` std only).

## Stress (both green)

- Counter-wrap walk: xor_stream over counters 0xFFFFFFF0..(+32 blocks) byte-identical to
  32 independent `chacha20_block` calls across the 2^32 wrap.
- 100k random-position round-trips: xor twice == identity across lengths 1..=600,
  random keys/nonces/counters, exercising stack-tail path.
- Extra: tail 1..=63 exactness vs independent block calls.

## SLA-disclosed unblocks (AB-3 in AB-4-owned files; AB-4 later superseded on disk)

Applied while AB-4's mid-flight files blocked the crate compile, per the 10-min SLA:
1. `aead.rs` Nonce96 const asserts (16/8 memory vs 12-byte wire; original conflated
   memory size with wire length) — later superseded/evolved by AB-4 on disk.
2. `poly1305.rs` `absorb_block(state, &state.buffer)` double-borrow → copy buffer out
   (Copy) — superseded on disk by AB-4.
3. `disclosed` in both-lanes rule: named here; AB-4's receipts should name the same.
4. One fix in MY file by another lane (disclosed by that lane): `*p` deref +
   `nth().expect()` in `rfc8439_sunscreen_independent_blocks`.

## Coordination

AB-4 landed `aead.rs`+`poly1305.rs` mid-wave; lib.rs registrations merged cleanly
(`chacha` line intact). Never ran git, per contract.

## Falsifier note (anti-vacuity)

The RFC fixtures fail loudly on a wrong keystream (witnessed: the first fixture set —
hand-transcribed from memory — failed and forced a real RFC re-derivation; a court that
cannot flip is FAIL, and these flipped).
