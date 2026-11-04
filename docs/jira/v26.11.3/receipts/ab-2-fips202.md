# Receipt: AB-2 fips202.rs — KECCAK-p[1600,24] + SHAKE-128/256 (AoQ Wave-1 W1-2a)

Status: ALIVE (all gates ran and passed this session, exact subject
`/Users/sac/cgap/crates/cgap-crypto/src/fips202.rs`)

## Deliverable

`crates/cgap-crypto/src/fips202.rs` (new, 396 lines incl. tests):
- `keccak_f1600(&mut [u64; 25])` — θρπχι, 24 rounds, spec-constant RC
  (FIPS 202 Table 2, cross-checked against mjosaarinen/tiny_sha3
  `keccakf_rndc` after a transcription defect was caught by the KAT court)
  and PI_DST precomputed at const-eval from the π mapping.
- `Sponge` struct: incremental `absorb`/`squeeze_into`, rate-in-bytes
  parameterized; SHAKE padding 0x1F..0x80 (0x9F single-byte case handled).
- `Shake128` (rate 168) / `Shake256` (rate 136): one-shot `xof` +
  `instance()` incremental.
- One-line registration in `src/lib.rs` (`pub mod fips202;`).
- Safe Rust throughout; no `unsafe` anywhere; `#![forbid(unsafe_code)]`
  at crate root unchanged and clean. Zero new external dependencies
  (charter lib stays 1 path dep + dev-dep).

## Courts

- **KAT courts (7)**: SHAKE-128/256 official vectors — empty (32B and
  64B continuation), 0xD0, 0xA3×200 both rates, zeros×167 (rate-1
  padding edge, 0x9F case), all byte-for-byte against the hashlib
  (OpenSSL-backed) oracle. Two KAT constants initially from memory were
  caught wrong by the court and corrected from the oracle (empty-string
  SHAKE-256 "…dadd7" was the pre-standard Keccak value, not FIPS 202).
- **Stress property court**: 10,000 randomized incremental
  absorb/squeeze interleavings (xorshift-seeded, chunked absorbs + two
  squeezes) vs one-shot `xof` — byte-for-byte equal, all 10k cases pass.
- **Misuse court**: absorb-after-squeeze panics; the guard is an
  unconditional `assert!` (release-safe), witnessed by
  `catch_unwind` test in both profiles.
- **Throughput bench** (release, plain Instant, min-of-N):
  SHAKE-128 **4.958 ns/byte @ 1KB**, **4.301 ns/byte @ 1MB** —
  baseline for future lanes. Fence: <20 ns/byte release-only
  (debug builds run unoptimized ~40× slower; fence is skipped there
  and this is disclosed, not a silent skip).

## Gates (commands + exits, this session)

- `cargo test -p cgap-crypto --lib fips202` → `10 passed; 0 failed`,
  exit 0 — **twice** (plus release run).
- `cargo test --release -p cgap-crypto --lib fips202` → `10 passed;
  0 failed`, exit 0.
- `cargo check -p cgap-crypto --target riscv32imc-unknown-none-elf` →
  exit 0 (RISCV_OK; warnings pre-existing from sibling lane's
  poly1305.rs, not this lane's files).
- `cargo check -p cgap-crypto --target wasm32-unknown-unknown` →
  exit 0 (WASM_OK).

## Defects found and fixed (all session-introduced, caught by courts)

1. Round-constant table mis-transcribed from memory (RC tail botched) —
   caught by KAT court, fixed from authoritative tiny_sha3 source
   (fetched, not recalled).
2. RHO table initially transposed (x-major vs y-major read of FIPS 202
   Table 2) — corrected to x-major; a Python replica confirmed the
   permutation matched the reference lane-for-lane once RC was fixed.
3. Pad-byte `if/if` precedence bug in `finalize` — swallowed the 0x80
   terminator exactly on the empty-input path; caught by the
   empty-string KATs alone.
4. Misuse guard was `debug_assert!` (compiled out in release) — made an
   unconditional `assert!`.

## SLA unblocks (disclosed in both receipts per same-checkout law)

- chacha.rs:134 `Vec` import: AB-3 fixed it themselves on restart
  before my unblock landed — no diff applied by AB-2.
- chacha.rs:244 `&u8` vs `u8`: AB-2 applied the minimal one-character
  unblock (`p` → `*p` in `assert_eq!`) under the compile-freeze SLA.
  Owning lane AB-3 supersedes; disclosed here and to be named in AB-3's
  receipt.

## Standing

- fips202 module: ALIVE (gates above).
- Baseline fence for future lanes: 4.301 ns/byte (1MB, release,
  min-of-N); regression fence 20 ns/byte release-only.
- Not done (out of scope): criterion harness integration,
  KYBER-grade unrolled variants — future lanes may fence against the
  baseline number above.

Per charter: no git operations performed.
