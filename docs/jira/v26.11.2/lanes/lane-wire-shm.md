# Lane wire-shm — crates/wire-shm substrate (P-03/P-04)

Subject: new standalone crate `wire-shm` (host wire tier, std-only, zero-dep).

## 1. Subject / file inventory
- `/Users/sac/cgap/crates/wire-shm/Cargo.toml` — standalone `[workspace]`, name `wire-shm`,
  version 26.11.2, edition 2021, zero dependencies, license Apache-2.0.
- `/Users/sac/cgap/crates/wire-shm/src/lib.rs` — frame format doc, WireError, MAGIC 0x5749_5230
  ("WIR0"), module wiring, re-exports.
- `/Users/sac/cgap/crates/wire-shm/src/crc32.rs` — table-driven CRC-32/IEEE (reflected,
  poly 0xEDB8_8320, init/xorout 0xFFFF_FFFF), OnceLock table.
- `/Users/sac/cgap/crates/wire-shm/src/ring.rs` — WireRing: create/open/stamp_frame/read_frame/
  read_frame_with_crc/verify_frame, offset-addressed file-backed segment (FileExt, NOT mmap —
  mmap needs libc, forbidden by zero-dep law), 4096B bookkeeping region (next_ordinal u64 BE,
  capacity u32 BE) + 4096B slots, 16-byte BE header (magic|crc32|payload_len|ordinal).
- `/Users/sac/cgap/crates/wire-shm/tests/wire_courts.rs` — 7 Chicago courts.

## 2. Commands + exits
- `cargo build` (crates/wire-shm) — exit 0, zero warnings after cleanup.
- `cargo test` (crates/wire-shm) — exit 0; results: lib 1 passed, integration 7 passed,
  doc 0. 8 tests total, 0 failed. (also ran `cargo tree`: only the crate itself -> zero deps.)
- Mutation court (SHIM-COURT, /tmp/wire-shm-mutation-sandbox, repo tree untouched):
  - BASELINE: `cargo test` on unmutated sandbox copy -> 3x "test result: ok".
  - MUTATION: poly 0xEDB8_8320 -> 0xEDB8_8321 -> `test result: FAILED. 0 passed; 1 failed`
    (known-vector court flips).
  - CONTROL: shared-tree `cargo test` -> 8 passed, 0 failed (final run this session).
  - Repo sha256 recorded post-court: crc32.rs e92f70f5..., ring.rs 5c3883fe...,
    lib.rs 867f3159... (never mutated; no restore path -> mtime-bump law not triggered).

## 3. Invariants proven
- ordinals strictly increasing per writer (bookkeeping next_ordinal = highest+1;
  stamp with lower ordinal -> StaleOrdinal).
- stale-slot read -> typed StaleOrdinal {expected, found} (wraparound court).
- payload_len overflow -> FrameTooLarge {len 5000, max 4080} (typed court).
- crc mismatch -> CrcMismatch (crc tamper court, byte-flip of payload).
- persistence: create -> drop -> WireRing::open -> read_frame(42) byte-equal, next_ordinal 43.

## 4. Anti-vacuity
- In-test flip: `tamper_court_flips` runs CONTROL (unmutated, must pass) + MUTATED sandbox
  segment (ordinal byte corrupted -> typed StaleOrdinal). Both branches asserted; a court
  that cannot flip is FAIL.
- Command-level flip: CRC poly mutation flips the known-vector court (evidence above).
- No [SKIP] anywhere; no Process.sleep; no mocks (real file segments throughout).

## 5. Standing
- crate: ALIVE — `cargo test` exit 0, 8/8 passed this session against the exact on-disk
  subject.
- publish: not attempted (per lane law: future 26.11.3 Wave 4 candidate, not now).
- divergences from lane doc: none material. Note: lane doc named WireRing method set; the
  doc's `verify_frame` "checks magic + crc32" is implemented as full read-path validation
  (magic + ordinal + len + crc) — strictly stronger.
- fences: none crossed — root workspace untouched (wire-shm NOT a root member, confirmed
  via grep on /Users/sac/cgap/Cargo.toml), zero git writes, /tmp sandboxes removed.
